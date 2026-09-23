# The core design

**Settled Wed 23 Sep 2026, by the Captain, before v0.3.0 goes out.** His reason, in his
words: *"we need to finalise the core design as if someone installs this then we cannot go
in diff direction."*

Once a stranger installs Sarge, its shape is a promise. This is that shape. Everything in
here is fixed until the Captain moves it. Everything outside it is allowed to change
without breaking anyone.

---

## Three things

### 1. The book

Your rules, in your repo, in Sarge syntax. `.sarge` at the repo root, git-ignored. Seven
keywords. Every rule carries a `wrong` line and a `right` line as real code, because the
demonstration is what binds and a sentence is not.

Three layers, and yours always wins:

```
book/universal.sarge     laws true in any language        ships with Sarge
book/<stack>.sarge       the stack's laws                 ships with Sarge
<your repo>/.sarge       yours, and the tutor's           yours
```

A book declares its own `stack` markers; the engine loads whichever matches your repo or
your task. **Nothing in the engine names a language.** Books are per-stack; the engine is
not.

### 2. The organ

The model, the check, and — when it is proven — the adapter. Everything that judges.

llama.cpp with Sarge compiled in as a static library. One binary, in-process, no pipes.
It runs on your machine and nothing it serves leaves.

**The adapter belongs here.** The Captain's ruling the same day: *"adaptor goes in the
organ because we can change the attention shape in the adaptor so it plugs into the
organ."* It is not a fourth thing — it is the organ's attention, shaped by your book.

### 3. The refusal

While a rule is broken, the code does not run.

This is the product. Everything else serves it. A check that reports and does not stop is
a linter, and there are enough of those.

---

## The boundary

**Your agent writes a file. Sarge judges that file.** That is the whole contract.

What is ours:

- judging one file against the book
- refusing the run while a hit stands
- growing the book from failures (the tutor)

What is **never** ours:

- the agent's loop
- its tools
- its build
- any verdict about whether a job got done

We learned this the expensive way. We wrote our own agent loop to develop against, then
spent two days fixing it as though it were the thing being sold — a verdict reading WORKS
over a repo whose POST returned 500, a tutor asked to go looking instead of being shown
the failure, no regression check at all. Those were faults in **test equipment**. It lives
in `testing/` now and it is not the product.

**Sarge sits on code, not on tool usage.** One file in, one verdict out.

---

## What is allowed to change, and what is not

### Fixed — a stranger's install depends on it

| | |
|---|---|
| the `.sarge` format | people write these by hand; it cannot churn |
| `.sarge` at the repo root, git-ignored | where a book lives |
| `SARGE_HOME` | points at the clone |
| the hook's contract | exit 2 with the reason on a hit; exit 2 and loud when it cannot check; never a silent pass |
| the refusal | a run does not proceed while a hit stands |
| local by default | the check talks to a model on your own machine unless you explicitly opt out |

### Free to change — nobody outside can see it

- how the rules reach the model: **text today, attention later**
- which model the organ runs
- how rules are selected for a file or a task
- the check's internals — one pass, two passes, however it gets to a verdict
- the tutor's prompt, and when it is called

---

## Text, then attention

**v0.3.0 — rules as text.** Selected per file and per task, delivered in the prompt. The
hook, the book, the check, the refusal. This is what ships.

**v1.0.0 — rules as attention.** The adapter in the organ. Rules as learned keys and
values the model attends to, instead of tokens it re-reads.

**The core does not move between them.** The book goes in; the verdict comes out. A user
who installs v0.3.0 and upgrades to v1.0.0 changes nothing about how they work — the
`.sarge` file is the same file, the hook is the same hook, the refusal is the same refusal.
What changes is invisible from outside.

That is why the design is settled now rather than after: **the adapter has to fit inside
this, and it does.**

### The book and the adapter are the same thing at two stages

Source and compiled. The book stays readable, editable and diffable; the adapter is the
fast form of it. **If they ever disagree, the book wins and the adapter is rebuilt.**

### Where the adapter stands today

Proven on the working clone, 22 Sep, and **not shipped** — the trainer is not in this
tree at all, because it is research and a user has no use for it:

- per layer, learned vectors and a per-head gate **initialised to zero** — with the gate
  shut the model is bit-for-bit the one we shipped (max logit change `0.000e+00`)
- the vectors go through the model's own **frozen** `W_k` and `W_v` to become keys and
  values, scored against Q in their own softmax
- trained on three rules: loss `3.73 → 0.0128`, gates `0.002 → 0.385`
- **with no rule in the prompt**, the model wrote
  `const rows = db.prepare('SELECT id, name, price FROM tools').all();` where without the
  adapter it rambled
- 164,000 parameters; every model weight frozen; holds on GPU in fp16

**What is not done:** llama.cpp cannot load an adapter. That is `llama-graph.cpp` and it is
untouched. Until then this is research, it is claimed nowhere outward, and it waits.

The mechanism came from arXiv 2303.16199. That repository is GPL-3.0 and Sarge is MIT, so
not a line of its code is here — the paper describes the idea and the implementation is ours.

**The trainer never ships.** It is PyTorch on the Captain's machine. What would ship, if
this is ever proven, is a file of a few hundred kilobytes beside the book. A user installs
no Python package, runs no second process, and downloads no second model.

---

*Written by Merlin on the Captain's instruction, 23 Sep 2026. Merlin cannot sign his own
release; this is design, not a signature.*
