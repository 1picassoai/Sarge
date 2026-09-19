# Sarge

**Drills it until it sticks. Doesn't negotiate.**

**Corrections that survive.** A small local model writes your code. When it breaks a rule
of your codebase, a frontier tutor teaches it that rule — once, from its own mistake — and
writes it into a book that lives with the repo. From then on the rule rides at the tail
of every turn, every file the model writes is checked against it, and **the run refuses
while a rule is broken.** Not a promise that a rule binds; a loop that learns a rule from
the model's own failure and makes breaking it cost the run.

**What leaves your machine, exactly.** The model runs on your GPU. The check runs on your
GPU. The tutor is a frontier model, and when it is called it receives the error text, the
model's own file around the line the error names, and, at the end of a green task, the
files the model wrote. That is code, and it leaves. It never receives the rest of your
repository. If you never let the tutor run, nothing leaves at all. The Claude Code hook
has no tutor: nothing leaves, full stop.

You wrote a `CLAUDE.md` or an `AGENTS.md` and watched it get ignored. That effort was the
right idea. This is the second half of it.

## What it is, in one screen

```
you ──task──▶ the harness ──▶ the model (local, on your GPU)
                 │                │ writes a file
                 │◀── the check ──┘  the organ judges the file against the book: no regex
                 │  a hit? the app will not run until it is fixed
                 │  the same failure twice? the tutor is asked, on the model's behalf
                 │  green and answering? the tutor reviews once, and writes what it learned
                 ▼
            <your repo>\.sarge   — the book. Git-ignored. Grows from your own corrections.
```

- **The organ** — llama.cpp with Sarge compiled in. The universal laws are in the binary,
  at the root of every prompt. `organ/README.md` builds it.
- **The book** — `.sarge` at your repo's root, in the Sarge language: `do` · `never` ·
  `wrong |` · `right |` · `allow` · `since`. Every rule carries the wrong line and the right
  line as code. A rule without a demonstration is advice; the check enforces only the ones
  with one. `docs/SARGE-SYNTAX.md`.
- **The harness** — the tools the model gets and the floors under it: the check after every
  write, the refusal, the forced ask, the four-strike stop. A Python package for LangChain,
  `python/sarge`. One import, one line, in your own LangGraph agent.
- **The tutor** — Claude, called rarely: on a repeated failure, and once at the end of a
  green task. It sees the error, the model's own file around the line, and at the end the
  files the model wrote. Never the rest of your repo. This is the one thing that leaves.
- **The page** — `console.cmd`, a task box and a Run button. It holds no logic.

## Works with

- **LangChain / LangGraph** — `python/sarge`, the whole loop as a package. Tested; the
  series and the step log are in `docs/`. **Step by step, from a clean machine:
  `docs/GUIDE-LANGCHAIN.md`.**
- **Claude Code** — `hooks/`, the check as a hook: every file Claude Code writes is judged
  by your local model against your `.sarge`, a hit comes back as the tool's own feedback,
  and the app will not run while it stands. No key, no network. Tested by hand
  (`hooks/README.md`).
- **CrewAI, Cursor** — next. Not in this release, so not claimed.

## Start

```
organ\llama-src\organ.cmd     the model on :8421      (build it first: organ\README.md)
console.cmd                   the page on :8420
```

Then, on the page: a folder, a task in plain words, arm **HARNESS**, Run.
`STRANGER.md` is day one in four steps.

Use it from your own LangGraph agent instead:

```python
from sarge import Sarge
agent = Sarge(repo=".", task=task).agent()
agent.invoke({"messages": [("user", task)]})
```

`python/README.md` has the pieces if you keep your own graph.

## What is measured, honestly

Every claim below has a file behind it. No number here is for anything but reading.

- **A series of tasks on one .NET repo, seven of seven,** each verified by hand with curl,
  the book growing as it went. Every failure on the way written down with its cause, most
  of them ours. `docs/SERIES-160926.md`.
- **The same seven on Node, harness on, seven of seven by hand.** The bare model on the
  same tasks: none — it wrote a broken config in task one and never opened it again.
  `docs/SERIES-NODE.md`.
- **Where a rule sits decides whether it holds.** Text at the tail of the turn held every
  handler across four runs; cached K/V did not, and the claim that it did was withdrawn
  the morning it failed to repeat. `docs/KV-POINTING.md`.
- **The check: six of seven known faults caught at the right line, with one false flag,**
  on the replay that ships in `rust/tests/judge/` and runs from the clone
  (`rust\replay-check.cmd`). It is the loaded model judging a line against the rule's
  examples — so it is only as good as that model, the number moves with the wording of
  the book, and it once passed a hard-coded filename because the example was written
  differently. Add a `wrong |` line; that is how it learns. Run the replay yourself and
  read the number you get. `docs/CHECK-DESIGN.md`, `rust/tests/judge/README.md`.
- **What the tutor's first question is:** *what can this code destroy?* It caught a line
  that drops the database on every start, in Development, that a green run had passed.
  `docs/THE-BIBLE.md`, Part Seven.

**Not measured yet, so not claimed:** that a rule survives compaction across a long
multi-turn agent (the test is written, `python/experiments/compaction.py`; the first run
showed the loss and voided its own control arm); any model but Qwen3-4B on one 8 GB card;
any hardware but that card.

## Layout

```
organ/          the patch, the header, the build and start scripts   → organ/README.md
rust/           the handshake: the language, the check, the tutor, the book  (cargo build --release)
python/sarge    the harness for LangChain                                    (pip install -e python)
hooks/          the check as a Claude Code hook                              (hooks/README.md)
harness/        CHARACTER.md (who the model is) · VETTED.md (the compiled core)
book/           universal.sarge — the laws loaded under every repo's book
tools/          console.py — the page
docs/           the bible, the language, every experiment and its result, and GUIDE-STEPS.md
                (the .NET series and the Agent Framework steps in there are history: that host is parked)
```

## Laws

Nothing leaves the machine but what the tutor is handed, and the README says exactly what
that is. No regex in rules. Never complicate the code — attack the
design. A proof's success is a finding, not a feature. A withdrawn claim is written down
next to the claim.

## Name

CompilerGPT for four days; that name belongs to Lawrence Livermore National Laboratory's
compiler project. Sarge since 16 September 2026. MIT.
