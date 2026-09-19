# The check — design for the Captain's GO

*16 Sep 2026. Written while the Captain was away. Design only; nothing built.*

**GO given 17 Sep 07:00 — "that is the correct way, it is part of the loop." Built and
replayed the same morning: `rust/src/verdict.rs`, results in `rust/tests/judge/README.md`.
Six of seven faults of the week caught at their line, zero false flags on clean files.
What shipped differs from the design below in four measured ways: (1) only rules that
carry a `wrong` demonstration are enforced — a rule without one is delivered but not
judged, because every false flag in the replay came from an example-less rule; (2) the
organ's first pass names candidates, then each candidate gets a second look framed as
likeness — "doing what the WRONG examples do, or what the RIGHT ones do?" — twice, on the
line alone and in a three-line window, and both must agree; (3) a candidate must carry a
word the wrong examples use and the right ones never do (set difference on tokens, SQL and
boilerplate excluded) — not a pattern, a gate; (4) the agent names the file it just wrote
(`--file`), so one write costs one first pass plus a second look per candidate — seconds.
The one known miss is recorded. Regex is nowhere in it.*

## The fault, plainly

`--check` walks the rules and enforces the ones that carry a `check` field — a file glob
and a **regex**. Regex was banned on 14 Sep, so no rule has one. So the check enforces
nothing. **Every "CHECKS PASSED" since then has been vacuous**, including the one that
waved through `EnsureDeleted()` with no Development guard, the one that passed a second
`Tool` class in a new file, and the one that let `app.Services.GetRequiredService` through
in both handlers.

The rule text is good. The rule has never been *read against the file*.

## The design: the organ judges

No regex, no parser, no tree-sitter. **The check asks the local model.** For each file the
agent just wrote, and each rule that applies, one short question to the organ:

```
Here is a rule of this codebase:
  <shape>
Here is a file that was just written:
  <file>
Does this file break the rule? Answer YES or NO on the first line. If YES, give the
line number and the line.
```

Greedy, tiny, local, in-process — the handshake is already linked into the organ, so this
is a function call, not a network hop. A YES is a hit; the hit carries the line, and the
existing boundary does the rest (the tool result, and `run_app` refusing while it stands).

**Why this fits the laws:** it is baked into llama.cpp; it spends no frontier tokens; it
needs nothing the rule does not already have — the `shape` the tutor already writes is
the question. And it judges the thing the rule is about (meaning), not a pattern.

## What it costs

One local call per (file, rule) pair on every write. Fifteen rules and one file is fifteen
short prompts of ~300 tokens each — a second or two on the 4070. Batch by file, not by
rule, if it drags: all rules in one prompt, one answer per rule.

## What it must survive before it is trusted

The Captain's law: *a check that has never fired on a real violation is not a check.*
Before it goes near a run:

1. **Every fault of this week, replayed.** The unguarded `EnsureDeleted`, the duplicate
   `Tool` in `Tool.cs`, `app.Services.GetRequiredService` in a handler, `AddControllers`
   in a minimal API, `new ToolContext()`. It must say YES to each, with the line.
2. **The clean files, replayed.** Every WORKS file from the series. It must say NO to all
   fifteen rules on each. A false YES blocks a correct run — worse than a miss.
3. **One run per side, recorded** in a stickiness table like `EXPERIMENT-RULES-STICK.md`.

If a 4B model cannot judge a rule against a file reliably, that is a finding, and the
fallback is the tutor at the end (already there) — but then we say so and stop printing
"CHECKS PASSED".

## What I am asking for

GO to build it as `--check` in the handshake, replacing the regex path, with the replay
above run before it touches a live run. Half a day.

## What I am not asking for

Tree-sitter, an AST, a linter, or a regex in any coat. All banned, all correctly.

---

# Built, and one finding from the second student — 17 Sep

**Built 17 Sep, GO at 07:00.** Replay under Qwen3-4B: 6 of 7 faults caught, 0 false flags
(`rust/tests/judge/README.md`). Re-run after every change to the judge; unchanged at 09:35.

**The judge is the student.** The check asks the organ — whichever model is loaded — and
17 Sep's swap to `deepseek-coder-6.7b-instruct` showed what that means. With its own chat
template (`organ/llama-src/deepseek-coder.jinja`; the organ had been framing it as ChatML
and the end-token leaked into replies as text) the replay scored **0 of 7**. `SARGE_TRACE=1`
on the handshake prints what the organ was asked and what it said, and the trace is exact
about where it dies:

- **First pass, fine.** Asked for every rule the file breaks, DeepSeek names the real
  fault — `HIT use-config-for-connection line 7` — with the right rule and the right line.
  It also invents a line 15 that is not in the file; the parse throws that away, as
  designed.
- **Second look, dead.** Shown the rule's `wrong` example and the line, asked WRONG, RIGHT
  or NEITHER, it answers **NEITHER** to a line that *is* the wrong example, then wanders
  ("The marked line is doing what"). Both looks must agree, so the true hit is dropped.

The second look exists because Qwen's first pass produced three false flags; the likeness
question killed them. On a 2023 6.7B coder the same question kills the true hits instead.
**A check that is model-agnostic in code is not model-agnostic in result: it is only as
good as the loaded model's ability to compare two lines and say which one this is.** No
code changes for this — the motto. It is a design fact to carry into the DeepSeek arm: if
the student cannot judge, the run cannot refuse, and the arm measures the book without the
check. Which is a different experiment, and must be labelled as one.
