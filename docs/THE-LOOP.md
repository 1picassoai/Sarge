# The loop

**Vinn's design, Sun 13 September 2026, ~15:30.** Written as he said it, then the parts
we already have and the parts we do not.

---

## As he said it

> A tutor or orchestrator looks after his crew. Each crew member writes rules for their
> lane. The manager picks them and moves them to an md file. The handshake reads and
> VETs them based on an algorithm. If it finds a delta in the rules it boots llama.cpp
> and loads the rules at runtime. The cycle repeats 24/7.

---

## The parts

```
   CREW              MANAGER            VETTED.md           HANDSHAKE            MODEL
 one agent          the tutor         a closed file        Rust, in-process    llama.cpp, GPU
 per lane           (frontier or      the manager          reads the file      every layer
 writes rules       peer, doesn't     writes; humans       VETs each rule      on CUDA0
 from its own       matter)           can read and         detects a delta
 mistakes           picks which       edit it              boots and injects
                    rules are real
```

**Crew.** Each agent owns a lane — research, PR, coding a given repo — and does that task
over and over. Its mistakes are lane-shaped, so its rules are too. Already built: the
five-lane PR team, each lane with its own prompt, and a coding agent with a repo scope.

**Manager.** The tutor. Reads the whole run, writes ```rule blocks for the next compile.
Already built and already measured — the PR team tutor caught three failures the regex
scorer scored as passes. What it does *not* do yet is write to the file; today it prints
and a human runs `--learn`.

**VETTED.md.** The artefact. Not code, not regex baked into a binary — a closed, portable
document the manager writes and the handshake reads. Ships with the product. A human can
open it, read it, strike a line. **This is the thing that gets injected, and it is the
only thing.** Does not exist yet.

**Handshake.** Rust, one binary, no pipes. SELECT, SHAPE, LINK, CHECK — and now **VET**:
the algorithm that decides whether a rule is safe to put in the blood. Reads VETTED.md at
boot and on change. Built except for VET and the delta watch.

**Model.** llama.cpp in-process on the GPU, every layer, rules loaded at runtime. Built
today. Rules currently enter as a system message; the KV-cache injection is the next step
and is what "in the blood" means.

---

## VET — the one piece with no design yet

**What it decides:** does this rule fight the model's training prior, or not.

`archive-not-delete` argues with nothing — the training data has no opinion on whether
Harrogate Portal archives customers. It held at every rule count. `no-var` argues with
most C# ever written. It broke at every rule count. **Nothing in the rule's text tells
you which is which. Only running it does.**

So the algorithm is empirical, not clever. A rule earns injection by holding across
models and tasks. The threshold — how many models, how many tasks, what pass rate — is a
number the stickiness matrix gives us, not one to pick on a whiteboard. That is why the
matrix runs before VET is designed.

**The two-tier consequence.** A rule that passes VET goes into the blood, where the model
cannot argue with it. A rule that fails VET stays in the system message, where the model
reads it and can push back — and CHECK catches the push-back. Both tiers are useful. VET
decides which tier.

**Vinn's line that defines the job:** *"that is the job of the handshake — to not let
pollution in the blood."*

---

## The delta

The handshake watches VETTED.md. When it changes — a rule added, struck, or reworded — it
recomputes the injected state and reloads. This is what makes it 24/7 rather than a build
step: the crew keeps working, the manager keeps writing, the model keeps improving, and
nobody stops the engine.

Cheap to build once VET exists: hash the file, compare, reload on mismatch. The KV-cache
version is where it pays off — core rules computed once at boot, reused every turn,
recomputed only on delta.

---

## What runs 24/7

```
crew works  →  tutor reviews  →  writes rules  →  VETTED.md changes
     ↑                                                    │
     └──── model improves ← handshake injects ← VET ←────┘
```

Every part of that loop exists today except VET, the delta watch, and the KV-cache write.
The tutor already closed the loop once without a frontier model in it. What is missing is
the vein — and the guard at it.

---

## Order of work

1. **Stickiness matrix** — produces the numbers VET is built from. Ready, on GO.
2. **VET algorithm** — designed from those numbers, not before.
3. **VETTED.md** — manager writes it, handshake reads it.
4. **Delta watch** — hash, compare, reload.
5. **KV-cache injection** — the blood.

Attack the design, not the code. Step 1 is the design question.
