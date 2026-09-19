# Sarge for LangChain — the second host, and the one that matters

*Design, Thu 17 Sep 2026 evening. The Captain's ruling: ".NET parked for the moment; build an
agent based on LangChain and bridge Sarge with that — that becomes the new moat." Python,
because that is where AI circles are. Every step of the build is recorded in
`GUIDE-STEPS.md` for the dev guide.*

## The one sentence

**Your LangGraph agent, with a drill sergeant in it.** The model is local, the rules of your
repo ride at the tail of every turn, every file it writes is checked against them, the run
refuses while a rule is broken, and when it is stuck a tutor answers and the answer becomes
a rule. Nothing leaves your machine but that one question.

## What the developer types

```bash
pip install sarge
```

```python
from sarge import Sarge

agent = Sarge(repo=".").agent()          # a LangGraph agent, the organ as its model
agent.invoke({"messages": [("user", "Add GET /tools/stats ...")]})
```

Or, keeping their own graph and taking the pieces:

```python
s = Sarge(repo=".")
llm   = s.llm()        # ChatOpenAI pointed at the organ (http://127.0.0.1:8421/v1)
tools = s.tools()      # list_files read_file edit_file write_file run_build run_app ask_tutor
graph = create_agent(llm, tools, system_prompt=s.character(), middleware=[s.rules()])
```

`s.rules()` is the tail hook: on every model call it appends the framed `.sarge` block as
the last message, from disk. `s.tools()` are the harness: the check after every write, the
refusal in `run_app`, the forced ask, the four-strike stop.

## What it is, piece by piece — nothing new invented

| today (C#, `SargeRun`) | Python (`sarge`) |
|---|---|
| the organ over the OpenAI client | `ChatOpenAI(base_url=organ, api_key="local")` |
| seven tools via `AIFunctionFactory` | seven `@tool` functions closing over one `Run` object |
| `SargeRulesProvider` (MAF context provider, tail) | a model-call hook that appends the block last |
| `Handshake.Run(...)` | `handshake.run(...)` — same binary, same arguments, same exit-code contract |
| `Review()` — final check + tutor once | `run.review()` |
| the host `Sarge.Agent` + ledger + run log | `python -m sarge --repo --task --harness --book` — the same command line, so the page's Run button can drive it as a third arm |

The harness brain (stall signal, forced ask, four-strike cancellation, check-after-write)
is ported line for line from `SargeRun.cs`, with its dated comments. **This is the second
copy of that brain.** The right home is the Rust binary, so every client is thin — the
motto. Written down here; not tonight.

## What stays exactly as it is

- The organ: llama.cpp with the handshake compiled in. Same port, same flags.
- The language, the book, `book/universal.sarge`, `.sarge` at the repo root, git-ignored.
- The check (`handshake --check`), the tutor (`handshake --ask`, `--learn`), the ledger row.
- VPN silence: nothing outbound while PANGP is up — the tutor and `npm install` refuse.

## What it does NOT claim

- Not "rules bind". *Corrections that survive*, plus a check that refuses the run.
- Not in-process. The agent speaks to the organ on a local port.
- Not a LangChain memory or a vector store. The book is a text file the model reads.

## Measured before claimed

Node series task 1 (HARNESS) through the Python host on `repos\workshop-node`, by hand,
the same verdict standard as the other two hosts. Then the same task RAW. Then, if both
match the C# host's behaviour, the page gets a third arm.

## The ask

Build the package under `python/sarge`, run Node task 1 through it, record the steps.
