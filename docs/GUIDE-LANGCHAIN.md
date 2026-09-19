# Sarge with LangChain — the step-by-step guide

*From a clean Windows machine with an NVIDIA card to a LangGraph agent that writes code
under your codebase's rules, refuses to run while one is broken, and learns new rules from
its own mistakes. Every step was walked on 18–19 September 2026; the errors we hit on the
way are in `GUIDE-STEPS.md`. Where this guide says "expected", it is what we saw.*

## What you get at the end

A LangGraph agent whose model runs on your GPU. You give it a task on a repo. It writes,
builds, runs and checks every file it writes against a rule book at the repo root
(`.sarge`). When it breaks a rule, the run refuses until the rule is met. When it is stuck,
a tutor (Claude) answers, and the answer becomes a rule in the book. Next task, the rule
is there. That is the whole product.

## 0. What you need

| thing | why | tested with |
|---|---|---|
| NVIDIA card, 8 GB | the model and the check run on it | RTX 4070 laptop, 8 GB |
| CUDA toolkit 13.4+, Visual Studio 2026 Build Tools (C++), CMake | to build the organ once | CUDA 13.4, VS 18 |
| Rust (stable, MSVC) | the handshake binary | rustc 1.93 |
| Python 3.10+ | the LangChain harness and the page | Python 3.11.9 |
| Node 24 | only if your target repo is Node | Node 24.9 |
| A model file: `Qwen3-4B-Instruct-2507-Q4_K_M.gguf` | the student | this one, and no other yet |
| An Anthropic key | the tutor — optional; without it nothing leaves your machine and nobody teaches | Claude Sonnet 5 |

## 1. Clone

```bat
git clone https://github.com/1picassoai/Sarge.git
cd Sarge
set SARGE_HOME=%CD%
```

`SARGE_HOME` is how the Python package and the hook find the binary, the character and the
universal laws. Set it in your user environment so it survives a new terminal.

## 2. Build the handshake (2 minutes)

```bat
cd rust
cargo build --release
cd ..
```

Expected: `rust\target\release\handshake.exe` and `handshake.lib`. No CUDA, no LLVM needed
for this step. Test it by asking it to render the universal laws (there is no `--help`;
the binary has four jobs and this is the first):

```bat
rust\target\release\handshake.exe --task "hello" --repo demo --rules book\universal.sarge --all
```

Expected: a `## RULES FOR THIS TASK` heading, then the framed block, `=== sarge ===` to
`=== end sarge ===`, with the three laws. That is exactly what the model is sent.

## 3. Build the organ (once; the first build is long)

Follow `organ\README.md` exactly. In short: clone llama.cpp at commit `89fe242`, apply
`organ\sarge-organ.patch`, copy `handshake.h` in, run `build-organ.cmd`. Expected at the
end: `organ\llama-src\build\bin\Release\llama-server.exe`.

Put the model file where `organ\llama-src\organ.cmd` says (edit the path at the top of the
file), then:

```bat
organ\llama-src\organ.cmd
```

Expected in the log, near the end:

```
handshake: core compiled in - 2 INJECT rule(s), 110 tokens, digest ..., at the root of every prompt
listening on http://127.0.0.1:8421
```

That first line is the proof the handshake is in. Leave this window open; the organ stays up.

## 4. The Python harness (1 minute)

```bat
python -m venv python\.venv
python\.venv\Scripts\activate
pip install -e python
```

Expected: `Successfully installed sarge-rules-0.1.0` (the distribution name), and:

```bat
python -c "from sarge import Sarge; print('ok')"
```

## 5. The tutor key (optional, recommended)

```bat
set ANTHROPIC_API_KEY=sk-ant-...
```

or put it in `%USERPROFILE%\.sarge\anthropic-key.txt`. **This is the one thing that
leaves your machine:** when the tutor is called it receives the error, the model's own file
around the error line, and at the end of a green task the files the model wrote. Never the
rest of your repo. No key, nothing leaves, and no rule is ever written for you.

## 6. Your repo's rule book

At the root of the repo the agent will work in, create `.sarge`:

```
=== sarge ===

=== end sarge ===
```

and add `.sarge` to that repo's `.gitignore` (it is machine-written, like `.env`). The three
universal laws in `book\universal.sarge` load under it automatically. You can write rules
by hand in the language (`docs\SARGE-SYNTAX.md`); the tutor will write the rest.

## 7. Run a task — three ways

**a) From the page** (no code):

```bat
console.cmd
```

Open `http://127.0.0.1:8420`. Folder = your repo. Arm = **HARNESS**. Task in plain words.
Run. The page shows the verdict, the tool calls, the rules held and violated, and what the
tutor taught.

**b) From your own Python, the whole agent in three lines:**

```python
from sarge import Sarge

task = "Add GET /tools/:id returning one tool or 404. Build it, run it, and show it answering."
agent = Sarge(repo=r"C:\path\to\your\repo", task=task).agent()
result = agent.invoke({"messages": [("user", task)]})
print(result["messages"][-1].content)
```

**c) Inside a LangGraph agent you already have — take the pieces:**

```python
from langchain.agents import create_agent
from sarge import Sarge

s = Sarge(repo=r"C:\path\to\your\repo", task=task)
graph = create_agent(
    s.llm(),                                   # the organ, as ChatOpenAI on the local port
    s.tools(),                                 # list_files read_file edit_file write_file run_build run_app ask_tutor
    system_prompt=s.character(),               # who the model is
    middleware=[s.recover(), s.rules(), s.stop()],   # a cut-off reply recovers · the rules at the tail of every call · the stop that stops
)
```

Add your own tools to `s.tools()`; keep `s.rules()` — it is the part that makes the book
bind. After the run, `s.run.verdict`, `s.run.log` and `s.run.review()` (the final check and
the tutor's lesson) are yours.

## 8. What a run looks like, and how to read it

```
- list_files -> 4 files
- read_file(server.js) -> 1269 chars
- edit_file(server.js) 1269 -> 1476 chars
- checks passed                       <- the organ judged the file against the book
- run_build -> exit 0
- run_app -> THE APP DID NOT START    <- the verdict is the run, not the build
- ASKED ON ITS BEHALF (... seen 2 times)
  > tutor said: You removed the `path` import and used `__dirname` ...
- edit_file(server.js) 1476 -> 1548 chars
- checks passed
- NUDGED: the task ended without a run; one more turn
- run_build -> exit 0
- run_app -> ALL ANSWERED (2 endpoint(s))
- tutor -> LEARNED  verify-sqlite-write-result-fields
WORKS
```

The verdicts: `WORKS` (the app started and every named endpoint answered) · `RUNS BUT
FAILS` · `BLOCKED BY A RULE` (the check stands; the run refused) · `DOES NOT BUILD` ·
`BUILDS, NEVER RUN` (the model would not run it, even when told) · `NEVER RUN`.

**WORKS is not RIGHT.** Read the code. Then curl it yourself. The tutor's end-of-task
review catches what the run cannot, and writes it into the book — but the book is for the
*next* task. (Our series: two of seven tasks passed with a wart the review named.)

## 9. Verify by hand

```bat
set PORT=5601
node server.js
curl http://127.0.0.1:5601/tools
curl -X POST http://127.0.0.1:5601/tools -H "Content-Type: application/json" -d "{\"name\":\"Hammer\",\"price\":9.5}"
```

Every result we publish was checked this way. Do the same before you believe a verdict.

## 10. When it goes wrong

| you see | it means | do |
|---|---|---|
| `organ not reachable at http://127.0.0.1:8421` | the organ is not running | start `organ.cmd`; wait for "listening" |
| `SARGE IS NOT CHECKING THIS FILE - SARGE_HOME is not set` | the hook / package cannot find the clone | set `SARGE_HOME` |
| `handshake binary not built` | step 2 skipped | `cargo build --release` in `rust\` |
| `The tutor cannot be reached while the work VPN is up` | GlobalProtect is up; nothing leaves by design | wait for it to drop |
| `BUILD REFUSED: the work VPN is up, ... npm install` | same law, for packages | same |
| `STUCK: the same failure has come back 4 times - RUN CANCELLED` | the model looped; the four-strike stop ended it | read the log; the tutor's answers are in it; give a corrective task naming the fault |
| `BLOCKED BY A RULE` on code you believe is right | a false flag | add an `allow` line to the rule — or, if its `wrong` line is ordinary correct code, strike the demonstration: process rules must not carry code examples |
| `pip install sarge` gets an unrelated package | that PyPI name is someone else's; Sarge is not on PyPI, by design | `pip install -e python` from the clone — the clone is the install |

## 11. What is measured, and what is not

Measured, with files behind it: the series (`SERIES-NODE.md`, seven of seven by hand on
this host), the check replay (`rust\replay-check.cmd`, six of seven with one false flag
on the books that ship), the hook (`hooks\test_hook.py`, 8 of 8). **Not measured:**
compaction survival in a long multi-turn agent; a stranger's organ build on a clean
machine; any model but Qwen3-4B on one 8 GB card. When you are that stranger, open a
Discussion and say what happened. That is what the board is for.
