"""
Sarge for LangChain. Your LangGraph agent, with a drill sergeant in it.

    from sarge import Sarge
    agent = Sarge(repo=".").agent()
    agent.invoke({"messages": [("user", "Add GET /tools/stats ...")]})

Or take the pieces and keep your own graph:

    s = Sarge(repo=".")
    graph = create_agent(s.llm(), s.tools(), system_prompt=s.character(), middleware=[s.rules(), s.stop()])
"""
from __future__ import annotations

import os
import sys
from typing import List, Optional

from langchain.agents import create_agent
from langchain.agents.middleware import before_model, wrap_model_call
from langchain_core.messages import AIMessage, HumanMessage
from langchain_core.tools import tool
from langchain_openai import ChatOpenAI

from .options import SargeOptions
from .run import Run, StopRun

BASE_INSTRUCTIONS = """You are a coding agent working inside one repository. You have tools: list_files, read_file,
edit_file, write_file, run_build, run_app. Work in this order: list the files, read the ones that
matter, make the change with edit_file (exact old_text -> new_text; write_file only for a new
file), then run_build. If the build fails, read the errors, fix, and build again.

A GREEN BUILD IS NOT DONE. When the build succeeds, call run_app with the endpoints the task
asked for. The app must start and answer. If it does not start, or an endpoint returns an error,
that is a real failure - read what the app said, fix it, and run it again.

IF YOU ARE STUCK, ASK. You have ask_tutor: one specific question to a senior engineer about an
error you are actually looking at. Quote the error. Do not guess twice at the same failure -
if a fix did not work, ask why rather than trying another guess.

You are finished when the app runs and its endpoints answer. Say which file you changed and why,
briefly."""

PLAN_INSTRUCTIONS = """Do NOT write any code and do NOT call edit_file, write_file or run_build. You may call list_files and
read_file. Then answer with a short numbered PLAN, ten lines at most:
  1. the files you read and what each one told you
  2. the ONE file you will change
  3. exactly what you will add or change, in plain words
  4. which rule from RULES FOR THIS TASK applies, and how your change obeys it
  5. how you will know it worked
Stop after the plan."""


class Sarge:
    """One repository, one task. Build the pieces, or the whole agent."""

    def __init__(self, repo: str = ".", task: str = "", **options):
        self.options = SargeOptions(repo_dir=repo, **options)
        self.run = Run(self.options, task)

    # ── the model ────────────────────────────────────────────────────────────
    def llm(self) -> ChatOpenAI:
        """The organ: llama.cpp with the handshake compiled in, spoken to as OpenAI."""
        o = self.options
        # Ten minutes, not the default: a 4B on a laptop GPU writing a whole file is slow,
        # and a client that gives up mid-thought looks like a stubborn model.
        return ChatOpenAI(model=o.model_id, base_url=o.organ_url, api_key="local",
                          temperature=o.temperature, max_tokens=o.max_output_tokens,
                          timeout=600, max_retries=1)

    # ── the harness ──────────────────────────────────────────────────────────
    def tools(self) -> List:
        """The seven tools of the harness. Plan mode: list and read only. RAW: no tutor."""
        r = self.run

        def guard(fn):
            # A TOOL ANSWERS IN WORDS, NEVER WITH AN EXCEPTION. 18 Sep, task 6: the model asked
            # to read `client/` - a folder - the path guard raised, and LangChain ended the
            # whole run at three seconds with AGENT ERROR. The .NET host hands the same error
            # back to the model as the tool's reply and the model recovers. So does this now.
            # StopRun is the one deliberate exception: its message reaches the model as the
            # tool's last word, and stop() ends the run at the next model call.
            def inner(*a, **k):
                try:
                    return fn(*a, **k)
                except StopRun as s:
                    return str(s)
                except Exception as ex:  # noqa: BLE001
                    r.log.append(f"- {fn.__name__} ERROR: {type(ex).__name__}: {ex}")
                    return f"ERROR: {ex}"
            inner.__name__ = fn.__name__
            inner.__doc__ = fn.__doc__
            return inner

        @tool
        def list_files() -> str:
            """List the source files in the repo, relative paths."""
            return guard(r.list_files)()

        @tool
        def read_file(path: str) -> str:
            """Read one file from the repo by relative path."""
            return guard(r.read_file)(path)

        @tool
        def edit_file(path: str, old_text: str, new_text: str) -> str:
            """Change part of a file: replace old_text with new_text. old_text must match the file exactly, once, including indentation. Prefer this over write_file - it cannot damage lines you did not name."""
            return guard(r.edit_file)(path, old_text, new_text)

        @tool
        def write_file(path: str, content: str) -> str:
            """Write the full new contents of one file. Use only for a NEW file; for an existing file use edit_file."""
            return guard(r.write_file)(path, content)

        @tool
        def run_build() -> str:
            """Build the project in the repo - dotnet build for a .NET project, npm install + npm run build + a syntax check for a Node project. Returns SUCCEEDED or the error lines."""
            return guard(r.run_build)()

        @tool
        def run_app(paths: str) -> str:
            """Start the app, call its endpoints, and stop it. Pass the checks separated by semicolons: a path is a GET; a method and a path sends that method; a JSON body may follow the path. Example: "GET /tools; POST /tools {"name":"Hammer","price":9.5}; GET /tools/1". Returns each status code and response body. A build that succeeds is not proof the app works - this is. Always exercise every endpoint the task named, with the method it named."""
            return guard(r.run_app)(paths)

        @tool
        def ask_tutor(question: str) -> str:
            """Ask a senior engineer ONE specific question about an error you are actually seeing. Quote the error in your question. Only works after a build or a run has failed, and only twice per task - so try to work it out first, and ask when you are genuinely stuck."""
            return guard(r.ask_tutor)(question)

        all_tools = [list_files, read_file, edit_file, write_file, run_build, run_app, ask_tutor]
        if self.options.plan_first:
            return all_tools[:2]
        return all_tools if self.options.harness else all_tools[:-1]

    # ── the rules, at the tail ───────────────────────────────────────────────
    def rules(self):
        """
        Middleware: on every model call, the framed .sarge block is appended as the LAST
        message, read from disk each time. Never in the history, so compaction cannot drop
        it; a book that changed is picked up on the next call. The tail is the measured
        position (docs/KV-POINTING.md).
        """
        r = self.run

        @wrap_model_call
        def sarge_rules(request, handler):
            block = r.rules_block()
            if os.environ.get("SARGE_TRACE"):
                # What the model is about to be sent, role by role - the only way to know where
                # the framework put the rules, and whether anything is piling up.
                print(f"--- to the model ({len(request.messages)} msgs + rules {'yes' if block else 'no'}) ---", file=sys.stderr)
                for m in request.messages:
                    c = m.content if isinstance(m.content, str) else str(m.content)
                    print(f"  {m.type:9} {len(c):6}  {c[:80].replace(chr(10), '⏎')}", file=sys.stderr)
            if not block:
                return handler(request)
            return handler(request.override(messages=[*request.messages, HumanMessage(content=block)]))

        return sarge_rules

    def recover(self):
        """
        Middleware: an organ error mid-call becomes a reply the model can act on, not a dead
        run. 18 Sep, task 7: a whole-file write_file ran past the output limit, llama-server
        answered 500 "failed to parse tool call arguments as JSON", LangChain raised, and the
        run died with the fix one bracket away. The model is told what happened and what to
        do instead; the nudge gives it the turn.
        """
        r = self.run

        @wrap_model_call
        def sarge_recover(request, handler):
            try:
                return handler(request)
            except Exception as ex:  # noqa: BLE001
                text = str(ex)
                r.log.append(f"- MODEL CALL FAILED: {type(ex).__name__}: {text[:160]}")
                if "parse tool call" in text or "max_tokens" in text or "length" in text:
                    return AIMessage(content="My last reply was cut off before the tool call was complete: the content was too long to send in one write_file. "
                                             "I will change only the lines that need changing with edit_file instead of rewriting the whole file.")
                raise
        return sarge_recover

    def stop(self):
        """Middleware: when the harness has cancelled the run, the graph ends instead of calling the model again."""
        r = self.run

        @before_model(can_jump_to=["end"])
        def sarge_stop(state, runtime):
            return {"jump_to": "end"} if r.stopped else None

        return sarge_stop

    # ── the character, at the head ───────────────────────────────────────────
    def character(self) -> str:
        """harness/CHARACTER.md (who the agent is) plus the working instructions. RAW: instructions only."""
        o = self.options
        instructions = PLAN_INSTRUCTIONS if o.plan_first else BASE_INSTRUCTIONS
        if o.review_path:
            from pathlib import Path
            p = Path(o.review_path)
            if p.is_file():
                instructions = ("A senior engineer reviewed your plan. Follow the corrected plan below EXACTLY.\n\n"
                                + p.read_text(encoding="utf-8") + "\n\n" + BASE_INSTRUCTIONS)
        character = self.run.character()
        return (character + "\n\n" + instructions) if character else instructions

    # ── the whole thing ──────────────────────────────────────────────────────
    def agent(self, model: Optional[object] = None, checkpointer=None):
        """A LangGraph agent with everything wired: the organ, the seven tools, the rules at the tail, the stop."""
        # A hard ceiling on model calls per run, ending the graph rather than raising. The
        # four-strike stop is the designed end; this is the floor under it. 17 Sep: the first
        # Python run made 26 model calls against one fixed refusal and had to be killed.
        from langchain.agents.middleware import ModelCallLimitMiddleware
        return create_agent(model or self.llm(), self.tools(), system_prompt=self.character(),
                            middleware=[self.recover(), self.rules(), self.stop(),
                                        ModelCallLimitMiddleware(run_limit=self.options.max_model_calls, exit_behavior="end")],
                            checkpointer=checkpointer)

    NUDGE = ("You have not run the app. A task is not done until the app has been built, started and its "
             "endpoints have answered. Call run_build now, then run_app with every endpoint the task named, "
             "with the method it named. If something fails, fix it and run again.")
