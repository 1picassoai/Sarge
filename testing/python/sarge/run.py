"""
One task in one repository: the tools the model gets, the state the loop keeps, and the
verdict at the end. This is the harness - the thing between the developer's task and the
model that runs the loop so the model cannot cheat.

Ported on 17 Sep 2026 from agent/Sarge.AgentFramework/SargeRun.cs, which was itself lifted
that evening from four days of measured failures in Sarge.Agent/Program.cs. Every guard
below has a reason; none was designed in advance. This is the SECOND copy of the harness
brain. The right home is the Rust binary, so every client is thin - written down, not
done tonight.
"""
from __future__ import annotations

import json
import os
import re
import shutil
import subprocess
import time
from pathlib import Path
from typing import Dict, List, Optional, Set, Tuple

from . import handshake
from .options import SargeOptions

SOURCE_EXT = (".cs", ".csproj", ".json", ".js", ".mjs", ".cjs", ".jsx", ".ts", ".tsx", ".html", ".css", ".md", ".py")
SKIP_DIRS = {"bin", "obj", "node_modules", ".git", ".vs", "dist", ".venv", "__pycache__"}


class StopRun(Exception):
    """The stop that stops. Raised from a tool when the same failure has come back four times."""


class _Log(list):
    """The run log, written through to disk on every line. 17 Sep: a runaway host was killed
    at ten minutes and left no log at all, because the log was written at the end. Evidence
    that only exists after a clean finish is not evidence."""
    sink = None  # a callable taking the whole list, or None

    def append(self, line: str) -> None:  # type: ignore[override]
        super().append(line)
        if self.sink is not None:
            try:
                self.sink(self)
            except Exception:  # noqa: BLE001
                pass


class Run:
    def __init__(self, options: SargeOptions, task: str):
        self.o = options
        self.task = task
        self.repo = options.repo
        if not self.repo.is_dir():
            raise FileNotFoundError(str(self.repo))
        self.home = options.home
        self.book = options.book
        sln = next(self.repo.glob("*.sln*"), None) or next(self.repo.rglob("*.csproj"), None)
        self.repo_name = self.repo.name if sln is None else sln.stem.split(".")[0]

        # state the host reads
        self.log: _Log = _Log()
        self.tool_calls = 0
        self.builds = 0
        self.last_build_ok = False
        self.ran = False
        self.last_run_ok = False
        self.written: Set[Path] = set()
        self.last_failure = ""
        self.standing_hit = ""       # the last rule check that failed and has not been cleared
        self.stopped = False
        self.rules_held = 0
        self.deliveries = 0

        # private
        self._dirty_since_build = False
        self._read_any = False
        # Folders that a project file actually compiles. A file written anywhere else builds
        # "green" because nothing looks at it.
        self._project_dirs: List[Path] = list({p.parent for p in self.repo.rglob("*.csproj")}
                                              | {p.parent for p in self.repo.rglob("package.json")
                                                 if "node_modules" not in p.parts})
        self._started_empty = not self._project_dirs
        self._last_write_size: Dict[Path, int] = {}
        self._idle_rewrites = 0
        self._idle_builds = 0
        self._edit_rejects: Dict[Path, int] = {}
        self._questions_asked = 0
        self._forced_asks = 0
        self._forced_asks_for: Dict[str, int] = {}
        self._seen_failures: Dict[str, int] = {}
        self._not_green = 0          # run_app asked for without a green build, in a row
        self._last_refusal = ""
        self._refusal_count = 0
        self.stopped_by_refusal = False
        self._t0 = time.monotonic()
        self._last_call = 0.0

    # ── what the host asks for ───────────────────────────────────────────────
    def rules_block(self) -> str:
        """The whole book for this repo, framed, as the handshake renders it. Empty for RAW."""
        if not self.o.harness:
            return ""
        _, block = handshake.run(self.home, self.book, "--task", self.task, "--repo", self.repo_name, "--all")
        if block:
            self.rules_held = sum(1 for l in block.splitlines() if l.startswith("rule "))
            self.deliveries += 1
        return block

    def character(self) -> str:
        p = self.home / "harness" / "CHARACTER.md"
        return p.read_text(encoding="utf-8") if self.o.harness and p.is_file() else ""

    @property
    def verdict(self) -> str:
        if not self.ran:
            # A run the check refused is not a build failure; say what actually stood in the way.
            if self.standing_hit and self.builds > 0:
                return "BLOCKED BY A RULE"
            return "NEVER RUN" if self.builds == 0 else ("BUILDS, NEVER RUN" if self.last_build_ok else "DOES NOT BUILD")
        return "WORKS" if self.last_run_ok else "RUNS BUT FAILS"

    # ── the floor under every refusal ────────────────────────────────────────
    # THE SAME REFUSAL FOUR TIMES IS THE END. 18 Sep, task 6: the build was refused as idle
    # ("nothing changed since the last build") and the model pressed run_build again - forty
    # times in eight minutes - because a STOP written in words is advice, and a 4B model
    # does not take advice. The four-strike rule already ends a run on a repeated FAILURE;
    # this is the same law for a repeated REFUSAL. Any tool answering the model with the
    # identical refusal four times running raises StopRun, and stop() ends the graph.
    def _refused(self, text: str) -> str:
        if text == self._last_refusal:
            self._refusal_count += 1
        else:
            self._last_refusal, self._refusal_count = text, 1
        if self._refusal_count >= 4:
            self.log.append(f"- STUCK: the same refusal has come back {self._refusal_count} times - RUN CANCELLED")
            self.stopped = True
            self.stopped_by_refusal = True
            raise StopRun("STOP. You have made the same refused call four times in a row. Nothing changed between them. The run is over.")
        return text

    def allow_one_more_turn(self) -> None:
        """The host's one exception: after a refusal-cancel with the build green and no run,
        the nudge gets a single turn. The refusal counter starts again; the four-strike
        failure count does not."""
        self.stopped = False
        self.stopped_by_refusal = False
        self._last_refusal, self._refusal_count = "", 0

    # ── timing ───────────────────────────────────────────────────────────────
    def _timed(self, what: str) -> None:
        now = time.monotonic() - self._t0
        think = now - self._last_call
        self._last_call = now
        self.log.append(f"  [t+{now:6.1f}s  model thought {think:5.1f}s]  {what}")

    # ── paths ────────────────────────────────────────────────────────────────
    def _safe(self, rel: str) -> Path:
        leaf = Path(rel.replace("\\", "/").rstrip("/")).name
        if len(Path(leaf).suffix) < 2 or not Path(leaf).stem.strip():
            raise ValueError(f"'{rel}' is not a filename - it needs a name AND an extension, like server.js or client/src/App.jsx. "
                             "A folder cannot be read; list_files shows every file, and writing a file creates its folders.")
        full = (self.repo / rel).resolve()
        if self.repo not in full.parents and full != self.repo:
            raise ValueError(f"path escapes the repo: {rel}")
        return full

    def _is_node(self) -> bool:
        return (self.repo / "package.json").is_file()

    def _node_main(self) -> str:
        try:
            main = json.loads((self.repo / "package.json").read_text(encoding="utf-8")).get("main")
            if isinstance(main, str) and main:
                return main
        except Exception:  # noqa: BLE001
            pass
        return "server.js" if (self.repo / "server.js").is_file() else "index.js"

    def _sh(self, args: List[str], timeout: int = 300, cwd: Optional[Path] = None, env: Optional[dict] = None) -> Tuple[int, str]:
        p = subprocess.run(args, cwd=str(cwd or self.repo), capture_output=True, text=True,
                           encoding="utf-8", errors="replace", timeout=timeout, env=env, shell=False)
        return p.returncode, (p.stdout or "") + (p.stderr or "")

    def _npm(self, rest: str) -> List[str]:
        """npm on Windows is npm.cmd, which subprocess cannot exec with shell=False - hence
        the cmd.exe /c wrapper that was hardcoded here. cmd.exe does not exist on macOS or
        Linux, so resolve the real npm there instead."""
        if os.name == "nt":
            return ["cmd.exe", "/c", f"npm {rest}"]
        return [shutil.which("npm") or "npm", *rest.split()]

    def _source_files(self) -> List[str]:
        out = []
        for p in self.repo.rglob("*"):
            if p.is_file() and p.suffix.lower() in SOURCE_EXT and not (set(p.relative_to(self.repo).parts) & SKIP_DIRS):
                out.append(str(p.relative_to(self.repo)))
        return sorted(out)

    # ── the check ────────────────────────────────────────────────────────────
    def _check_after_write(self, full: Path, what: str) -> Optional[str]:
        """The rule is a check, not a request. A hit bounces straight back as the tool result."""
        if not self.o.harness:
            return None
        ok, report = handshake.run(self.home, self.book, "--check", str(self.repo), "--file",
                                   str(full.relative_to(self.repo)), "--repo", self.repo_name, "--task", self.task)
        self.standing_hit = "" if ok else report
        if not ok:
            # A rule hit IS a failure. 18 Sep, the Captain's first run through this host:
            # the check stood, the model sent the same no-op edit five times, and the stall
            # floor never fired because last_failure was empty - only builds and runs had
            # ever set it. Now a hit is what the tutor may be asked about, and the same hit
            # coming back counts toward the four strikes.
            self.last_failure = report
            ids = re.findall(r"\[([\w-]+)\]", report)
            self.log.append(f"- CHECK FAILED after {what}: {', '.join(dict.fromkeys(ids)) or 'see report'}")
            return f"{what} - BUT A RULE CHECK FAILED. The app will not run until this is fixed:\n{report}"
        self.log.append("- checks passed")
        return None

    # ── tools ────────────────────────────────────────────────────────────────
    def list_files(self) -> str:
        self.tool_calls += 1
        files = self._source_files()
        self.log.append(f"- list_files -> {len(files)} files")
        self._timed("list_files")
        return "\n".join(files)

    def read_file(self, path: str) -> str:
        self.tool_calls += 1
        full = self._safe(path)
        if not full.is_file():
            self.log.append(f"- read_file({path}) -> NOT FOUND")
            return (f"not found: {path}\nThere is no such file. The files that exist are:\n"
                    + "\n".join(self._source_files()) + "\nRead one of these.")
        text = full.read_text(encoding="utf-8", errors="replace")
        self._read_any = True
        self.log.append(f"- read_file({path}) -> {len(text)} chars")
        return text

    def write_file(self, path: str, content: str) -> str:
        self.tool_calls += 1
        full = self._safe(path)
        scaffolding = self._started_empty
        if not self._read_any and not scaffolding:
            self.log.append(f"- write_file({path}) REJECTED: nothing read yet")
            return "REJECTED: you have not read any existing file yet. Read the file you are changing, or the file with the existing endpoints, before writing."
        trimmed = content.strip()
        small_json = trimmed.startswith("{") and trimmed.endswith("}") and ":" in trimmed
        # An empty stylesheet is a real file: Vite failed on `import './index.css'` and the
        # model's correct fix - an empty index.css - was refused as a placeholder five times
        # (18 Sep, task 6). Code needs content; a stylesheet, a text or a markdown file may be empty.
        may_be_empty = full.suffix.lower() in (".css", ".txt", ".md", ".env", ".gitignore")
        if ((len(trimmed) < 40 and not small_json and not may_be_empty)
                or (trimmed.startswith("<") and trimmed.endswith(">") and "\n" not in trimmed)):
            self.log.append(f"- write_file({path}) REJECTED: placeholder content ({len(content)} chars)")
            return self._refused(f"REJECTED: content looks like a placeholder ({len(content)} chars). Send the full file contents.")
        if not scaffolding and not any(d == full.parent or d in full.parents for d in self._project_dirs):
            self.log.append(f"- write_file({path}) REJECTED: not inside any project")
            dirs = ", ".join(str(d.relative_to(self.repo)) or "." for d in self._project_dirs)
            return f"REJECTED: {path} is not inside any project folder, so it would never be compiled. Project folders: {dirs}"
        # ESCAPED JSON IS A FORMAT, NOT A MEANING. A 4B model sends a JSON file through a
        # JSON tool-call argument and double-escapes it: the file lands as {\"database\":
        # \"tools.db\"} with literal backslashes, and JSON.parse dies at byte one. Seen on
        # RAW 17 Sep (three tasks never recovered) and on the Captain's run 18 Sep (six
        # identical writes, two tutor answers naming it, run cancelled). Same class as CRLF:
        # normalise the format, never the content. Only for .json, only when the raw text
        # is not valid JSON and the unescaped text is.
        if full.suffix.lower() == ".json":
            try:
                json.loads(content)
            except ValueError:
                fixed = None
                # ({...}) - the model wrapped the object in parentheses, twice on 18 Sep.
                t = content.strip()
                if t.startswith("(") and t.endswith(")"):
                    try:
                        json.loads(t[1:-1]); fixed = t[1:-1]
                    except ValueError:
                        pass
                if fixed is None and "\\\"" in content:
                    try:
                        u = content.encode("utf-8").decode("unicode_escape")
                        json.loads(u); fixed = u
                    except (ValueError, UnicodeDecodeError):
                        pass
                if fixed is not None:
                    self.log.append(f"- write_file({path}) normalised: the model sent JSON in a wrapper (parentheses or escaped quotes)")
                    content = fixed
        # Writing the identical file again is a stall, not progress. 18 Sep: six identical
        # 28-byte writes of config.json between two tutor answers.
        if full.is_file() and full.read_text(encoding="utf-8", errors="replace") == content:
            self._edit_rejects[full] = self._edit_rejects.get(full, 0) + 1
            self.log.append(f"- write_file({path}) REJECTED: identical to the file on disk")
            reply = (f"REJECTED: {path} already contains exactly this. Writing it again changes nothing. "
                     "Read the error again and change what it names.")
            if self._edit_rejects[full] >= 2 and self.last_failure:
                help_ = self._ask_on_its_behalf(self.last_failure)
                if help_:
                    self.log.append(f"- STALL DETECTED after {self._edit_rejects[full]} identical writes of {path}")
                    reply += help_
            return self._refused(reply)
        full.parent.mkdir(parents=True, exist_ok=True)
        full.write_text(content, encoding="utf-8")
        self._dirty_since_build = True
        self.written.add(full)
        if full.suffix.lower() == ".csproj" or full.name.lower() == "package.json":
            self._project_dirs.append(full.parent)
        self.log.append(f"- write_file({path}) <- {len(content)} chars")
        self._timed(f"write_file({path}) {len(content)} chars")
        return self._check_after_write(full, f"wrote {path} ({len(content)} chars)") or f"wrote {path} ({len(content)} chars)"

    def edit_file(self, path: str, old_text: str, new_text: str) -> str:
        self.tool_calls += 1
        full = self._safe(path)
        if not full.is_file():
            self.log.append(f"- edit_file({path}) -> NOT FOUND")
            return f"not found: {path}"
        if not old_text:
            self.log.append(f"- edit_file({path}) REJECTED: empty old_text")
            return "REJECTED: old_text must be the exact existing text to replace."
        new_text = new_text or ""
        if old_text.replace("\r\n", "\n").strip() == new_text.replace("\r\n", "\n").strip():
            # A no-op counted as an edit: told to delete a duplicate line, the model replaced
            # the block with itself three times and each one looked like progress. 17 Sep,
            # Python host, Node task 1: SEVEN of these in a row, 100 seconds, before the
            # student asked on its own. A no-op is a stall like any other rejection: the
            # second one forces the ask.
            self._edit_rejects[full] = self._edit_rejects.get(full, 0) + 1
            self.log.append(f"- edit_file({path}) REJECTED: new_text is identical to old_text")
            reply = ("REJECTED: new_text is the same as old_text - this edit changes nothing. If you mean to DELETE "
                     "lines, put them in old_text and make new_text the remaining lines only (or empty).")
            if self._edit_rejects[full] >= 2 and self.last_failure:
                help_ = self._ask_on_its_behalf(self.last_failure)
                if help_:
                    self.log.append(f"- STALL DETECTED after {self._edit_rejects[full]} no-op edits of {path}")
                    reply += help_
            # 18 Sep, task 6, fourth attempt: this exact reply came back twenty-five times
            # in a row. The refusal floor ends it at four, like every other refusal.
            return self._refused(reply)
        # Line endings are not content. Files on disk are CRLF; the model sends LF.
        raw = full.read_text(encoding="utf-8", errors="replace")
        crlf = "\r\n" in raw
        text = raw.replace("\r\n", "\n")
        old_text = old_text.replace("\r\n", "\n")
        new_text = new_text.replace("\r\n", "\n")
        n = text.count(old_text)
        if n == 0:
            # Indentation is not content either. A window of lines matching with leading and
            # trailing space ignored; if exactly one window matches, that is the text.
            file_lines = text.split("\n")
            want = [l.strip() for l in old_text.split("\n")]
            while want and not want[-1]:
                want.pop()
            hits = [s for s in range(len(file_lines) - len(want) + 1)
                    if want and all(file_lines[s + k].strip() == want[k] for k in range(len(want)))]
            if len(hits) == 1:
                old_text = "\n".join(file_lines[hits[0]:hits[0] + len(want)])
                n = 1
                self.log.append(f"- edit_file({path}) matched by trimmed lines")
            elif len(hits) > 1:
                n = len(hits)
        if n != 1:
            why = ("old_text does not appear in the file, character for character." if n == 0
                   else f"old_text appears {n} times. Include more surrounding lines so it matches once.")
            self._edit_rejects[full] = self._edit_rejects.get(full, 0) + 1
            tried = old_text.replace("\r", "").replace("\n", "⏎")[:100]
            self.log.append(f"- edit_file({path}) REJECTED: {'old_text not found' if n == 0 else f'old_text appears {n} times'}  tried: `{tried}`")
            reply = f"REJECTED: {why}\nThis is the file as it is NOW. Copy the exact lines from here:\n\n{text}"
            if self._edit_rejects[full] >= 2:
                reply += (f"\n\nYou have now failed to match this file {self._edit_rejects[full]} times. Stop editing it in pieces: "
                          f"call write_file with the COMPLETE new contents of {path}, based on the file above. Do not create a different file.")
                if self.last_failure:
                    help_ = self._ask_on_its_behalf(self.last_failure)
                    if help_:
                        self.log.append(f"- STALL DETECTED after {self._edit_rejects[full]} rejected edits of {path}")
                        reply += help_
            # The reply carries the whole file, so an identical one means the model sent the
            # identical wrong old_text against an unchanged file: a spin, floored at four.
            return self._refused(reply)
        self._edit_rejects.pop(full, None)
        replaced = text.replace(old_text, new_text)
        full.write_text(replaced.replace("\n", "\r\n") if crlf else replaced, encoding="utf-8")
        self._dirty_since_build = True
        self.written.add(full)
        self.log.append(f"- edit_file({path}) {len(old_text)} -> {len(new_text)} chars")
        # THE STALL. An edit that leaves the file the same length as the last edit, with no
        # build in between, is the model rewriting the same bytes.
        new_len = len(replaced)
        if self.o.harness and self._last_write_size.get(full) == new_len:
            self._idle_rewrites += 1
            if self._idle_rewrites >= 2 and self.last_failure:
                help_ = self._ask_on_its_behalf(self.last_failure)
                if help_:
                    self._last_write_size[full] = new_len
                    self.log.append(f"- STALL DETECTED after {self._idle_rewrites} identical rewrites of {path}")
                    return (f"edited {path} - BUT YOU HAVE NOW WRITTEN THIS FILE {self._idle_rewrites + 1} TIMES "
                            "AT THE SAME LENGTH WITHOUT BUILDING. You are not making progress." + help_)
        else:
            self._idle_rewrites = 0
        self._last_write_size[full] = new_len
        return self._check_after_write(full, f"edited {path}") or f"edited {path}"

    def _with_file_context(self, failure: str) -> str:
        """The tutor sees the lines: the file the error names, around the line it names, numbered."""
        m = re.search(r"((?:[A-Za-z]:)?[\w./\\ -]+?\.(?:js|mjs|cjs|ts|tsx|jsx|cs|json|py))[:(](\d+)", failure)
        if not m:
            # A RUN failure names no line, so a start failure carries the entry file whole.
            if self._is_node():
                entry = self.repo / self._node_main()
            else:
                entry = next((p for p in self.repo.rglob("Program.cs") if "bin" not in p.parts), None)
            if not entry or not entry.is_file():
                return failure
            lines = entry.read_text(encoding="utf-8", errors="replace").splitlines()
            if len(lines) > 220:
                return failure
            numbered = "\n".join(f"{i + 1:4}  {l}" for i, l in enumerate(lines))
            return failure + f"\n\nTHE ENTRY FILE AS IT IS ({entry.relative_to(self.repo)}, {len(lines)} lines) - the fault is in here, not in the environment:\n{numbered}"
        named, line = m.group(1).strip(), int(m.group(2))
        full = Path(named) if Path(named).is_absolute() else self.repo / named
        full = full.resolve()
        if self.repo not in full.parents or not full.is_file():
            return failure
        lines = full.read_text(encoding="utf-8", errors="replace").splitlines()
        a, b = max(0, line - 16), min(len(lines), line + 8)
        excerpt = "\n".join(f"{i + 1:4}  {lines[i]}" for i in range(a, b))
        return failure + f"\n\nTHE FILE AROUND THE ERROR ({full.relative_to(self.repo)}, lines {a + 1}-{b}):\n{excerpt}"

    def _ask_on_its_behalf(self, failure: str) -> str:
        """
        THE FLOOR UNDER ASKING. A small model does not refuse a tool, it simply never reaches
        for one. When the SAME failure comes back, the loop asks on the model's behalf and
        hands the answer back inside the tool result. Only on a repeat, never more than twice,
        its own budget. Four sightings end the run - a cancellation, not advice.
        """
        if not self.o.harness:
            return ""
        codes = sorted(set(re.findall(r"\b(CS\d{4}|NU\d{4}|MSB\d{4})\b", failure)))
        key = ",".join(codes) if codes else "".join(c for c in failure if c.isalnum())[:120]
        self._seen_failures[key] = self._seen_failures.get(key, 0) + 1
        n = self._seen_failures[key]
        if n >= 4:
            self.log.append(f"- STUCK: the same failure ({key}) has come back {n} times after two answers - RUN CANCELLED")
            self.stopped = True
            raise StopRun(f"STOP. This exact failure has now come back {n} times, and you have had two answers about it. The run is over.")
        # The net's budget is per DISTINCT failure, two each, not two per task. 18 Sep, the
        # Captain's run: three different faults in one task (bad package.json, a string
        # broken across lines, a JSON import without its attribute). The first two spent
        # the budget with correct answers; the third - the fatal one - got nothing and
        # the run died at four strikes with the tutor never asked about it. The four-strike
        # stop still bounds the whole run.
        if n >= 2 and self._forced_asks_for.get(key, 0) < 2:
            self._forced_asks_for[key] = self._forced_asks_for.get(key, 0) + 1
            self._forced_asks += 1
            if self.o.outbound_blocked():
                self.log.append("- forced ask REFUSED: nothing leaves this machine while the work VPN is up")
                return ""
            failure = self._with_file_context(failure)
            q = "This failure has come back after I changed the code. What is actually wrong and exactly what should I write instead?"
            _, answer = handshake.run(self.home, self.book, "--ask", q, "--failure", failure, "--task", self.task, "--repo", self.repo_name)
            self.log.append(f"- ASKED ON ITS BEHALF ({key} seen {n} times)")
            self.log.append("  > tutor said: " + (answer.strip().replace("\n", "\n  > ") if answer else "(nothing - unreachable)"))
            self._timed(f"asked on its behalf ({key})")
            if answer:
                return ("\n\nYou have hit this same failure before. A senior engineer looked at it:\n\n"
                        + answer + "\n\nDo exactly that. Do not write the same code again.")
        return ""

    def run_build(self) -> str:
        self.tool_calls += 1
        self.builds += 1
        if not self._dirty_since_build:
            self._idle_builds += 1
            if self._idle_builds > 2:
                self.log.append("- run_build REFUSED: nothing changed since the last build, third idle build")
                # Say what to do NEXT, not only what not to do. 18 Sep, task 7: the code was
                # already complete, the build was green, and the model pressed build four
                # more times because this message only said "write your change". If nothing
                # needs changing, the next call is run_app.
                return self._refused("STOP. You have run the build three times without changing a file. Nothing has changed. "
                                     "If the code is already complete, call run_app NOW with every endpoint the task named. "
                                     "Otherwise read a file, change it, then build:\n" + "\n".join(self._source_files()))
        else:
            self._idle_builds = 0
        self._idle_rewrites = 0
        self._last_write_size.clear()
        t = time.monotonic()
        if self._is_node():
            if self.o.outbound_blocked():
                self.last_build_ok = False
                self.log.append("- run_build REFUSED: work VPN is up, npm install would leave the machine")
                return "BUILD REFUSED: the work VPN is up, so nothing leaves this machine and packages cannot be installed. Wait for it to drop."
            code, output = 0, ""
            pkgs = sorted((p for p in self.repo.rglob("package.json") if "node_modules" not in p.parts), key=lambda p: len(str(p)))
            for pkg in pkgs:
                nm = pkg.parent / "node_modules"
                if not nm.is_dir() or pkg.stat().st_mtime > nm.stat().st_mtime:
                    code, out = self._sh(self._npm("install --no-audit --no-fund --loglevel=error"), cwd=pkg.parent)
                    if code != 0:
                        output = f"npm install in {pkg.parent.relative_to(self.repo)} failed:\n{out}"
                        break
                    if nm.is_dir():
                        os.utime(nm, None)
            if code == 0:
                code, output = self._sh(self._npm("run build --if-present"))
            if code == 0:
                # `node --check file.js` passes an ES module with a syntax error on Node 24 -
                # measured 18 Sep, task 7: an extra `)` on line 26 was "green" through two
                # builds and only died at start. Feeding the source through stdin with
                # --input-type=module parses it as the module it is, and names the line.
                main_path = self.repo / self._node_main()
                src = main_path.read_text(encoding="utf-8", errors="replace") if main_path.is_file() else ""
                # Module by its own syntax, not by package.json: the model rewrote the root
                # package.json without "type": "module" while server.js still said `import`,
                # and Node 24 ran it as a module anyway (syntax detection). A file that opens
                # a line with import or export is parsed as one. Format, not meaning.
                is_esm = any(l.lstrip().startswith(("import ", "export ")) for l in src.splitlines())
                if is_esm:
                    p = subprocess.run(["node", "--input-type=module", "--check"], input=src,
                                       cwd=str(self.repo), capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=60)
                    code = p.returncode
                    # stdin reports "[stdin]:26"; say the file's name so the tutor can find the line.
                    output = ((p.stdout or "") + (p.stderr or "")).replace("[stdin]", self._node_main())
                else:
                    code, output = self._sh(["node", "--check", self._node_main()], timeout=60)
        else:
            code, output = self._sh(["dotnet", "build", "--nologo", "-v", "q"], timeout=180)
        self.last_build_ok = code == 0
        self._dirty_since_build = False
        self._timed(f"run_build took {time.monotonic() - t:.1f}s -> exit {code}")
        file_line = re.compile(r"\.(?:js|mjs|cjs|ts|tsx|jsx|cs|json|py)[:(]\d+")
        errors = [l for l in output.split("\n")
                  if "error " in l.lower() or "Error" in l or "ERR!" in l or file_line.search(l)][:20]
        if not self.last_build_ok and not errors:
            errors = [l for l in output.split("\n") if l.strip()][-20:]
        self.log.append(f"- run_build -> exit {code}" + (f", {len(errors)} error line(s)" if errors else "") + ("" if self.written else " (nothing written yet)"))
        if self.last_build_ok and not self.written:
            return "BUILD SUCCEEDED - but you have not written any file yet, so nothing has changed. The task is not done."
        if self.last_build_ok:
            return "BUILD SUCCEEDED"
        failed = "\n".join(errors or output.split("\n")[-30:])
        self.last_failure = failed
        return "BUILD FAILED\n" + failed + self._ask_on_its_behalf(failed)

    def run_app(self, paths: str) -> str:
        """Start the app, ask it the questions the task named, read the answers, stop it."""
        import urllib.request
        import urllib.error
        self.tool_calls += 1
        if not self.last_build_ok:
            # Asking to run without a green build is a button press; three in a row is a spin.
            # 17 Sep, the Python host's first task: a fixed refusal came back the same size
            # 26 model calls running and nothing else changed. The loop needs a floor here too.
            self._not_green += 1
            self.log.append("- run_app REFUSED: the build is not green" + (f" ({self._not_green} in a row)" if self._not_green > 1 else ""))
            if self._not_green >= 3:
                self._not_green = 0
                return self._refused("STOP. You have asked to run the app three times without a green build. Nothing changes until you "
                                     "fix the code and call run_build. Do that now.")
            return self._refused("The build is not green. Run the build first - an app that does not compile cannot start.")
        self._not_green = 0
        if self.standing_hit:
            self.log.append("- run_app REFUSED: a rule check is still failing")
            return self._refused("REFUSED. A rule check is failing and the app does not run until it passes. Fix exactly this:\n" + self.standing_hit)
        if self._dirty_since_build:
            b = self.run_build()
            if not self.last_build_ok:
                return "You changed files since the last build, so it was built first - and it FAILED:\n" + b
        # A free port every time, never a fixed one. 18 Sep, task 4: "THE APP DID NOT START"
        # twice, and the tutor's diagnosis was right - the port was still held by an earlier
        # instance. A run must never fail on the harness's own leftovers.
        import socket
        with socket.socket() as s:
            s.bind(("127.0.0.1", 0))
            port = s.getsockname()[1]
        env = dict(os.environ, ASPNETCORE_ENVIRONMENT="Development", NODE_ENV="development", PORT=str(port))
        args = ["node", self._node_main()] if self._is_node() else ["dotnet", "run", "--no-build", "--urls", f"http://127.0.0.1:{port}"]
        app = subprocess.Popen(args, cwd=str(self.repo), stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                               text=True, encoding="utf-8", errors="replace", env=env)
        app_out: List[str] = []
        import threading

        def pump():
            for line in app.stdout:  # type: ignore[union-attr]
                app_out.append(line.rstrip("\n"))
        threading.Thread(target=pump, daemon=True).start()
        report: List[str] = []
        try:
            up = False
            for _ in range(40):
                if app.poll() is not None:
                    break
                try:
                    urllib.request.urlopen(f"http://127.0.0.1:{port}/", timeout=1).read()
                    up = True
                    break
                except urllib.error.HTTPError:
                    up = True
                    break
                except Exception:  # noqa: BLE001
                    time.sleep(0.5)
            if not up:
                self.ran, self.last_run_ok = True, False
                self.log.append("- run_app -> THE APP DID NOT START")
                self._timed("run_app -> did not start")
                said = "\n".join(l for l in app_out if l.strip())[-2500:]
                self.last_failure = "THE APP DID NOT START.\n" + said
                return "THE APP DID NOT START. It compiled but will not run. Its own output:\n" + said + self._ask_on_its_behalf(self.last_failure)
            checks: List[str] = []
            for c in (x.strip() for x in paths.split(";") if x.strip()):
                checks += [c] if "{" in c else [y.strip() for y in c.split(",") if y.strip()]
            checks = checks or ["/"]
            all_ok = True
            for check in checks:
                method, path, body = "GET", check, None
                parts = check.split(" ", 1)
                if len(parts) == 2 and parts[0].isalpha() and len(parts[0]) <= 6:
                    method, path = parts[0].upper(), parts[1].strip()
                    brace = path.find("{")
                    if brace >= 0:
                        body, path = path[brace:].strip(), path[:brace].strip()
                try:
                    url = f"http://127.0.0.1:{port}{path if path.startswith('/') else '/' + path}"
                    req = urllib.request.Request(url, method=method, data=body.encode("utf-8") if body else None,
                                                 headers={"Content-Type": "application/json"} if body else {})
                    try:
                        with urllib.request.urlopen(req, timeout=10) as resp:
                            status, text = resp.status, resp.read().decode("utf-8", "replace")
                    except urllib.error.HTTPError as he:
                        status, text = he.code, he.read().decode("utf-8", "replace")
                    if len(text) > 400:
                        text = text[:400] + "…"
                    ok = status < 500          # answered means the app handled it; a 404 on an empty table is an answer
                    all_ok &= ok
                    report.append(f"{method} {path} -> {status}")
                    report.append(f"  {text.replace(chr(10), ' ').strip()}")
                except Exception as ex:  # noqa: BLE001
                    all_ok = False
                    report.append(f"{method} {path} -> FAILED: {ex}")
            if not all_ok:
                why = [l.strip() for l in app_out if "exception" in l.lower() or "error" in l.lower() or l.strip().startswith("at ")][:12]
                if why:
                    report.append("")
                    report.append("What the app itself said:")
                    report += ["  " + l for l in why]
            self.ran, self.last_run_ok = True, all_ok
            text = "\n".join(report)
            if not all_ok:
                self.last_failure = text
            self.log.append(f"- run_app -> {'ALL ANSWERED' if all_ok else 'SOME FAILED'} ({len(checks)} endpoint(s))")
            self._timed(f"run_app -> {'all answered' if all_ok else 'some failed'}")
            if all_ok:
                return "THE APP RAN AND ANSWERED\n" + text
            return "THE APP RAN BUT SOMETHING FAILED\n" + text + self._ask_on_its_behalf(text)
        finally:
            try:
                if app.poll() is None:
                    subprocess.run(["taskkill", "/PID", str(app.pid), "/T", "/F"], capture_output=True)
            except Exception:  # noqa: BLE001
                pass

    def ask_tutor(self, question: str) -> str:
        """The student asks: only after something has FAILED, quoting the real error, twice per task."""
        self.tool_calls += 1
        if not self.last_failure:
            return "Nothing has failed yet. Build it and run it first - then, if it fails, ask about the actual error."
        if self._questions_asked >= 2:
            return "You have asked twice on this task. Use what you were told and fix it."
        if len(question.strip()) < 15:
            return "Ask a specific question about the error you are seeing, in a full sentence."
        if self.o.outbound_blocked():
            self.log.append("- ask_tutor REFUSED: nothing leaves this machine while the work VPN is up")
            return "The tutor cannot be reached while the work VPN is up. Work it out from the error."
        self._questions_asked += 1
        _, answer = handshake.run(self.home, self.book, "--ask", question, "--failure", self.last_failure, "--task", self.task, "--repo", self.repo_name)
        self.log.append(f"- ask_tutor({question[:60]}…)")
        self.log.append("  > asked: " + question.strip().replace("\n", " "))
        self.log.append("  > tutor said: " + (answer.strip().replace("\n", "\n  > ") if answer else "(nothing - unreachable)"))
        self._timed("ask_tutor")
        return answer or "The tutor could not be reached. Work it out from the error."

    # ── after the run ────────────────────────────────────────────────────────
    def review(self) -> Tuple[str, str]:
        """The final check over the whole repo, and the tutor ONCE, only over code that builds."""
        if not self.o.harness or self.o.plan_first or not self.written:
            return "", ""
        ok, report = handshake.run(self.home, self.book, "--check", str(self.repo), "--repo", self.repo_name, "--task", self.task)
        final_check = "CHECKS PASSED" if ok else report
        if not ok:
            self.last_build_ok = False
        lesson = ""
        if self.last_build_ok and not self.o.outbound_blocked():
            import tempfile
            fd, answer_file = tempfile.mkstemp(prefix="sarge-answer-", suffix=".txt")
            with os.fdopen(fd, "w", encoding="utf-8") as f:
                f.write("\n\n".join(f"// File: {p.relative_to(self.repo)}\n" + p.read_text(encoding='utf-8', errors='replace') for p in self.written))
            _, lesson = handshake.run(self.home, self.book, "--learn",
                                      "The build is green. Review this finished code for anything that breaks a convention of this codebase - something no compiler would catch.",
                                      "--answer", answer_file, "--task", self.task, "--repo", self.repo_name)
            try:
                os.remove(answer_file)
            except OSError:
                pass
            if lesson:
                self.log.append(f"- tutor -> {lesson.splitlines()[0]}")
        return final_check, lesson
