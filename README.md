# Sarge

**Drills it until it sticks. Doesn't negotiate.**

![The test web UI showing the agent coding in real time when presented with a coding challenge](docs/console.png)

*The test web UI showing the agent coding in real time when presented with a coding challenge: the task, the run, the rules held, the one rule the tutor added.*

## Why I built this

I wrote the rules of my codebase down for my coding agent. It read them, agreed with
them, and then shipped code that broke them anyway. Sometimes it did not even compile.
So I reviewed everything it wrote, every time, which meant I was paying twice: once in
tokens for code I could not trust, and once in my own hours to find out why. The rules
file was not the problem. The problem was that nothing made the agent *keep* a rule
after it had read it.

Sarge is what I built to stop that. A small model runs on my own GPU and writes the code.
When it breaks a rule, a tutor teaches it that rule once, from its own mistake, and
writes it into a book that lives with the repo. From then on the rule is put in front of
the model on every turn, every file it writes is checked against the book, and **the run
refuses while a rule is broken.** Corrections that survive.

## What it is

![Sarge architecture: your agent hands a task to the harness; the local model writes a file; the check judges it against the book; a hit stops the run; the tutor is the one thing that leaves your machine](docs/architecture.png)

- **The organ** — llama.cpp with Sarge compiled in. The universal laws sit at the root of
  every prompt. Prebuilt on the [release page](../../releases/tag/v0.1.0), or build it:
  `organ/README.md`.
- **The book** — `.sarge` at your repo's root, in the Sarge language: `do` · `never` ·
  `wrong |` · `right |` · `allow` · `since`. Every rule carries the wrong line and the right
  line as code; the check enforces only rules that have them. `docs/SARGE-SYNTAX.md`.
- **The harness** — the tools the model gets and the floors under it: the check after every
  write, the refusal, the forced ask, the four-strike stop. A Python package for LangChain.
- **The tutor** — Claude, called rarely: on a repeated failure, and once at the end of a
  green task. **This is the one thing that leaves your machine:** it receives the error,
  the model's own file around the line, and at the end the files the model wrote. Never
  the rest of your repo. No tutor key, nothing leaves.

## Install, in one line

Clone this repo, download the prebuilt organ from the
[release](../../releases/tag/v0.1.0) and one model file (`Qwen3-4B-Instruct-2507-Q4_K_M.gguf`),
then `pip install -e python` from the clone. Not on PyPI, by design: the clone is the
install. The whole walk from a clean machine, every command verified:
**`docs/GUIDE-LANGCHAIN.md`**.

```
organ\llama-src\organ.cmd     the model on :8421
console.cmd                   the page on :8420 — a folder, a task, Run
```

Or from your own LangGraph agent:

```python
from sarge import Sarge
agent = Sarge(repo=".", task=task).agent()
agent.invoke({"messages": [("user", task)]})
```

## Works with

- **LangChain / LangGraph** — `python/sarge`. Tested; `docs/GUIDE-LANGCHAIN.md`.
- **Claude Code** — `hooks/`: every file Claude Code writes is judged against your `.sarge`
  by your local model, a hit comes back as the tool's own feedback, and the app will not
  run while it stands. No key, no network. `hooks/README.md`.
- CrewAI, Cursor: next. Not in this release, so not claimed.

## What is measured, honestly

Every claim has a file behind it. No number here is for anything but reading.

- **Seven tasks on one Node repo, harness on, seven of seven correct by hand,** the book
  growing as it went. The bare model on the same tasks: none. Every failure on the way is
  written down with its cause, most of them ours. `docs/SERIES-NODE.md`.
- **Where a rule sits decides whether it holds.** Text at the tail of the turn held every
  time; cached K/V did not, and the claim that it did was withdrawn the morning it failed
  to repeat. `docs/KV-POINTING.md`.
- **The check, on the replay that ships with the tree:** six of seven known faults caught
  at the right line, one false flag. It is your local model judging a line, so it is only
  as good as that model; add a `wrong |` line when it misses, an `allow` when it flags
  something right. `rust/tests/judge/README.md`.
- **The tutor's first question is *what can this code destroy?*** It caught a line that
  drops the database on every start, in Development, that a green run had passed.

**Not measured, so not claimed:** that a rule survives compaction in a long multi-turn
agent; any model but Qwen3-4B on one 8 GB card; any hardware but that card.

## Layout

```
organ/          the patch, the header, the build and start scripts   → organ/README.md
rust/           the handshake: the language, the check, the tutor, the book  (cargo build --release)
python/sarge    the harness for LangChain                                    (pip install -e python)
hooks/          the check as a Claude Code hook                              (hooks/README.md)
harness/        CHARACTER.md (who the model is) · VETTED.md (the compiled core)
book/           universal.sarge — the laws loaded under every repo's book
tools/          console.py — the page
docs/           every experiment and its result, including the withdrawn ones
```

## Laws

Nothing leaves the machine but what the tutor is handed, and this file says exactly what
that is. No regex in rules. Never complicate the code — attack the design. A proof's
success is a finding, not a feature. A withdrawn claim is written down next to the claim.

MIT. Bring what your coding agent keeps getting wrong: [Discussions](../../discussions).
