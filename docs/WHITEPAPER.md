# Sarge — whitepaper

*Named CompilerGPT when written; renamed Sarge on 16 Sep 2026 (the old name is LLNL's).*

*Vinn Joseph's design. Written down by Merlin, Mon 14 Sep 2026. Draft; nothing here is
released.*

---

## The idea

> **Your agents learn from their own mistakes, and the model never forgets them.**

A frontier model is expensive and forgets everything between turns. A local model is free
and cannot code on its own. CompilerGPT makes the frontier teach the local model once, and
makes the lesson permanent. Every repeat of that work then costs nothing.

## The problem

Every turn of an agentic coding session re-sends the whole context. The same understanding
is re-derived, over the wire, at frontier prices, every single time. Most of those turns are
variations on work already understood.

## The design

The compiler frame.

```
   source   →   frontend   →   IR    →   backend   →   binary
   the task     frontier        rules     local model    the code
                expensive                 cheap
                runs ONCE                 runs often
```

The frontier reads, decides and teaches. The local model does the work. The rules are the
thing that carries the understanding from one to the other.

## The loop

```
   rules file ──▶ THE HANDSHAKE loads it ──▶ local model follows
                                                 │ fails
                                          frontier writes a better rule ──▶ rules file
```

That is the whole loop. Everything below is how it stays that simple.

**Where each thing lives in the model, ruled 15 Sep.** Rules are text, at the tail of the
task — position was measured, tail won. The character is a control vector — a disposition,
not a fact, so it goes one rung deeper at no inference cost. A proven book, and only a
proven one, ends in LoRA. The full ladder is in `THE-BIBLE.md`, Part One.

## Who teaches, who rules, who proves

**The frontier teaches and rules.** When the local model fails, the frontier reads the
model's reasoning and the error, never the repo, corrects it, and writes the rule. The same
hand decides whether that rule is a law for this repo or a one-off. There is no human in the
loop, because the human expects the frontier to do the teaching and the ruling.

**The run proves.** A rule is vetted by being tried. The binary loads it and reruns the task
it came from. Held, it stays. Broke, it goes back to the frontier to rewrite or withdraw.
No opinion judges a rule. No pattern judges a rule. The result does.

## Who holds what

**Vinn's ruling, 14 Sep 2026, and it settles a collision that had both sides holding the
same thing.**

```
   THE AGENT      its character and its job. Static, and static is right -
                  who it is does not change when the repo grows.
                  Held by the agent SDK, as a system prompt. Any SDK:
                  MAF, LangChain, CrewAI. This is how every one of them
                  builds an agent, and we do not fight it.

   THE HANDSHAKE  every law, in one place. Selected per task, injected at
                  runtime, updated as the repo grows.
```

**Why the laws live in one place.** *"If we start moving them around, if the law changes
there is no way to inject it at runtime."* A law scattered between a prompt, a framework
and a binary has no single moment where it can be changed. Held in one place, a changed
law reaches the next run with nothing to redeploy and nothing to restart.

**Why the laws cannot be static.** They grow with the repo: a security issue, a way a class
must be declared, a rule the developer writes by hand. Three sources feed them — the tutor
when a build goes red, compaction, and the developer. Anything that grows cannot be baked,
and anything baked cannot grow.

**What the character may never do** is contain a law, and what a law may never do is change
the character. A law says how this repo archives a customer. It can never say be something
else.

## The files

```
   TOP LAWS          one file, every repo      stable   → part of the core
   <repo>/rules.md   this repo's rules         moving   → follows the code
   <repo>/vetted.md  what held here            moving   → the frontier's rulings + the reruns
```

A new repo starts with the top laws and nothing else. A rule climbs from repo to top only
after it has held in more than one repo. The repo files move because the code moves; a rule
about a class dies when the class is renamed, and the frontier retires it on the next
failure. The top laws never move because they are about no code at all.

## The handshake

**The Rust binary. This is the most important thing in the design, and it is the product.**
Everything else is a file or a frontier. The handshake is what makes them one thing.

One binary. No HTTP, no subprocess, no pipe.

```
   THE HANDSHAKE — llama.cpp with the Rust handshake linked in, one exe
     build.rs    compiles the core into the binary
     read        reads the compaction file and the rule files, fast
     decide      has a new rule arrived since the last update — yes or no
     inject      core decoded once → the KV cache. This is "the blood".
     model       every layer on the GPU, in-process
```

**Every algorithm lives inside it.** Reading the compaction, telling a law from a passing
remark, telling a new rule from one already baked, choosing what rides per task: all in the
binary. No tool beside it, no script that runs first. The Python tools that exist today are
the sketches of what it does; the handshake is where they end up.

It is the layer between the rules and the model's reading habits. The frontier writes a
file; the handshake sees the delta and the blood changes without a restart. The core sits
in the attention state, where the model cannot argue with it the way it argues with a
prompt. Costumes ride per task. The core is never re-sent. It is the guard at the vein:
nothing enters the blood except through it.

Caching is not teaching. Caching makes the core cheap to hold. Teaching is the loop above.
The handshake is what makes the loop run without anyone in the room.

## What is banned

**Regex. Tree-sitter. Any authored pattern that judges anything.** The product is generic and
public; nobody writes patterns for it. Three judges only, all measured: the build, the
rerun, the frontier's eye. Format parsing stays, because a format is not a meaning.

**Nothing leaves the machine.** The local model, the rules, the repo, the blood: all on the
customer's box. The frontier sees ten lines of reasoning and an error. It never sees the
repo. A client's name never enters a prompt, a file, a log or a build.

## Known hole — when compaction happens

Compaction is the feed, and compaction happens on its own clock. It hits organically when
the context fills, or early when the user forces it. Either way the rules only move when it
does. Between two compactions nothing the frontier learned reaches the file, however long
that gap is. Force it early and a half-formed session gets distilled; leave it and a
lesson can sit unwritten for a day. The rules never update on their own.

**The design for it, 14 Sep 2026, same morning, before 12:17.** The handshake reads the live session file, not
only the compaction summary. The frontier writes its rule block in its answer, as it does
when it teaches; the session file on disk grows with every turn; the handshake sees a new
block since its last read, decides, injects. Nothing waits for compaction and nothing
bypasses the handshake: no hook, no side door, no second writer. Compaction stays as the
persistence judge inside the handshake. A rule seen once in the live file is a costume. A
rule that keeps surviving compactions is core. Forcing compaction early now costs nothing,
because nothing waits on it.

**And it compares before it reloads, because reloading llama.cpp is costly.**

```
new block seen ──▶ same as a ruling already held?
                       exact     → hash match           → ignore, no reload
                       reworded  → cosine to the held   → same law, ignore, no reload
                       new       → costume or core?
                                     costume → file only, rides per task, NO reload
                                     core    → re-decode the blood, the one costly path
```

A hash catches the identical rule; cosine catches the same law reworded; neither is a
pattern. A costume never reloads anything. Only a change to the core touches the blood,
and the core changes rarely by construction, because a rule must survive compactions to
reach it.

Designed, not built.

## The number

Two lines over one stream of tasks.

```
   frontier tokens per task      must fall
   tasks green with no teaching  must rise
```

First line flat: the frontier is being called too often. Second line flat: the rules are
not holding and the blood is the fault. Either way the design is wrong, not the code.

## Where it stands, 14 Sep 2026

Built, and **all of it inside the handshake**: Run → the local model answers with the core
(`harness/CHARACTER.md`) as its first message and the arm's rules as costumes → files land
in the sandbox → the build and then THE RUN judge it → the app must start and its endpoints
must answer, because a build only proves the code compiles → a red build never reaches the
tutor, the compiler has already said what is wrong in words the model can read → the tutor
is called once at the end over code that BUILDS, hunting what no compiler can catch, with
the compiler's errors, writes the correction and the rule as a form, the rule is compared
against the book before it is added, and the next Run reads it. No button. The RAW arm gets
neither core nor rules, so the comparison stays honest.

The modules are `sandbox.rs` (the write protocol and the fail-closed path check),
`judge.rs` (the build), `book.rs` (the compare), `tutor.rs` (the one outbound call).
`compile.rs` carries `--loop`, `--sandbox`, `--ledger` and `--json`. The console is a page
that shells the binary once and renders what comes back; the Python that once held this
logic is retired. **If something outside the binary starts making a decision, it belongs
inside it.**

**Closed live once, 15:32, by the Captain's hand:** task 0, build red, tutor called by the
loop, ninth rule in the book. The second Run showed the rule was not reaching the model:
a rule carries the repo it was learned in, and the handshake drops tagged rules unless the
run names the repo. Lesson written into the console: **the book is per repo, so the run
names the repo.**

Also built: the organ (llama-server with the handshake linked in, core compiled by
`build.rs`, KV reuse measured); the stick and shopfloor benches.

Not built: the same-law threshold from the matrix (a knob today); the KV state carrying the
core (today the core rides as a message; `--inject` states must be rebuilt with it).

### Designed, not built — the live session file

The handshake reads the compaction summary. Compaction runs on its own clock, so between
two of them nothing the frontier learned reaches the book. The fix is that the handshake
reads the **live session file** as well: it grows every turn, the frontier writes its rule
block into its own answer, and the handshake sees a new block since its last read, compares
it, and injects. Nothing waits for compaction and nothing bypasses the handshake — no hook,
no second writer. Compaction stays as the persistence judge: seen once is a costume, seen
across compactions is core.

### Designed, not built — the climb

A rule is learned in one repo and tagged with it. It becomes a **top law** only when it has
held in more than one repo: same law by cosine, green builds in both books, no rewrite in
between. The climb is what separates *this codebase archives customers* from *never write a
secret into code*. Until a rule has been proven somewhere it was not born, it stays in its
own book — which is what stops one project's convention becoming everyone's law.

Not released. Design → build → test → the Captain's approval → release.
