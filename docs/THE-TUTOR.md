# The tutor — the flow

**Superseded by `WHITEPAPER.md`, Mon 14 Sep 2026, before 12:17.** Kept for the record of how the
design was reached; the whitepaper is the design.

*Vinn's design, Mon 14 Sep 2026. Written by Merlin. Not built.*

> *The whole idea is token saving and making the local model more capable, with the same
> design.* · *Caching is not the same as directing the local model to follow rules and
> teaching it.* · *No regex. I need an algorithm.*

---

## One sentence

**The frontier is called less over time, and the local model finishes more alone over
time.** Same curve, two ends. That is the whole flow.

## The flow

```
   compaction ──▶ core rules ──▶ VETTED.md
                                    │
            ┌───────────────────────▼──────────────────────────────────────┐
            │  llama-server.exe  — ONE BINARY, Rust handshake linked in   │
            │                                                              │
            │   build.rs  : compiles the core INTO the binary              │
            │   handshake : reads VETTED.md, VETs, watches for delta       │
            │   inject    : core decoded once → KV cache = THE BLOOD       │
            │   model     : Qwen, every layer on the GPU, in-process       │
            │                                                              │
            │   task ──▶ reasoning ──▶ code ──▶ build                      │
            └──────────────────────────────────┬───────────────────────────┘
                                               │
                        green ─────────────────┤   done. 0 frontier tokens.
                                               │
                        red / drift ───────────▼
                                    TUTOR (frontier)
                                    reads reasoning + error. never the repo.
                                    corrects → retry
                                    writes the rule → VETTED.md
                                               │
                                    delta seen by the handshake → re-inject → blood grows
```

**Rust is the vein and the guard at it.** No HTTP, no subprocess, no pipe. The tutor writes a
file; the binary sees the delta and the blood changes without a restart. That is what makes
it 24/7, and what makes "in the blood" mean the attention state rather than a prompt.

## Why it saves tokens

The frontier runs once per lesson, never per task. A task the local model finishes green
costs the frontier nothing. A task it fails costs one read of ten lines of reasoning and
one error, and buys a rule that stops that failure coming back. The calls taper to zero on
repeat work. The frontier never sees the repo.

## Why the local model gets more capable

It does not learn. Nothing sticks between calls. What sticks is the blood: the rules sit in
the attention state, so the model cannot argue with them the way it argues with a prompt.
Every lesson is one more thing it will not do wrong again. Capability is the size of the
blood, and the blood only grows.

## What the tutor does, on a failure

1. Reads the local model's reasoning and the build error. Never the repo, never a file.
2. Corrects it as a senior would, in as few lines as the fault needs. The local model
   retries from the correction.
3. Writes the lesson as a form. Every field required; an empty one is a format error:

   ```
   never:    what it reached for
   instead:  the replacement — always
   allow:    the one case where the never does not apply, or "none"
   applies:  the repo or lane this was learned in
   topic:    the words a task about this would contain
   ```

## VET — what keeps pollution out of the blood

No pattern judges a rule. Three judges, all measured:

- **Persistence.** Taught more than once, on different tasks → a law. Sameness is cosine
  between forms. The threshold is a number the stick matrix gives.
- **The stick test.** Injected, and the tasks it was learned on rerun. Held → the blood.
  Broke → back to the tutor. Three rounds still breaking → it fights the prior; costume
  tier, readable, never the blood.
- **The lane.** A rule applies where it was learned. A rule from the client box never
  leaves the box. A form naming a client is refused at the door.

## The number

Two lines over one stream of tasks:

- frontier tokens per task — must fall
- tasks finished green with no tutor — must rise

First line flat → the tutor is called too often. Second line flat → the rules are not
holding and the blood is the fault.
