# Sarge — the bible

*Renamed 16 Sep 2026: CompilerGPT is LLNL's name (github.com/LLNL/CompilerGPT). The product
is **Sarge** — "drills it until it sticks, doesn't negotiate." The Captain's line. Older
documents in this folder still say CompilerGPT; they are history and are not rewritten.*

*Everything learned building this, 12–15 September 2026. For the Captain and Merlin to
refer to later. Written to be re-read cold, months from now, by someone who has forgotten
all of it.*

**Status at the time of writing:** the loop closed once, verified. Nothing is committed.
Nothing is released.

---

# PART ONE — THE ARCHITECTURE

## v2, as it actually runs

```
                        ┌──────────────────────────────────┐
   YOU  ───task+folder──▶│  THE CONSOLE  :8420              │
                        │  a page. holds no logic.         │
                        └────────────────┬─────────────────┘
                                         │ shells once
                                         ▼
   ┌──────────────────────────────────────────────────────────────────────┐
   │  THE AGENT   ·  .NET  ·  Microsoft Agent Framework                   │
   │                                                                      │
   │  character ← harness/CHARACTER.md      (who it is. never changes)    │
   │  rules     ← the handshake, per task   (what this codebase holds)    │
   │              placed at the TAIL of the task, not the system prompt   │
   │                                                                      │
   │  tools:  list_files · read_file · edit_file · write_file             │
   │          run_build · run_app · ask_tutor                             │
   └───────┬──────────────────────────────────────────┬───────────────────┘
           │ HTTP  (the one network hop)              │ shells
           ▼                                          ▼
   ┌───────────────────────────┐          ┌───────────────────────────────┐
   │  THE ORGAN     :8421      │          │  handshake.exe                │
   │  llama-server             │          │                               │
   │   + handshake linked in   │          │  --task    SELECT the rules   │
   │   + core compiled by      │          │  --check   what was written   │
   │     build.rs              │          │  --learn   teach, vet, write  │
   │                           │          │  --ask     answer the student │
   │  Qwen3-4B-Instruct-2507   │          └──────┬─────────────┬──────────┘
   │  Q4_K_M · RTX 4070 8GB    │                 │             │
   │  n_gpu_layers(99)         │                 ▼             ▼
   └───────────────────────────┘        ┌────────────────┐  ┌──────────────┐
                                        │  THE BOOK      │  │  THE TUTOR   │
                                        │  rules.jsonl   │  │  gpt-4.1     │
                                        │  one per repo  │  │              │
                                        │  grows         │  │  sees: error │
                                        └────────────────┘  │  + what was  │
                                                            │  written +   │
                                                            │  file NAMES  │
                                                            │  never the   │
                                                            │  repository  │
                                                            └──────────────┘
```

## Where a thing lives in the model — the ladder, ruled 15 Sep

Every llama.cpp lever, shallow to deep, each verified in `include/llama.h`. The rule is
**each thing goes to the shallowest rung that fits what it is.**

```
   rung                          llama.cpp                          carries         what goes there
   ─────────────────────────────────────────────────────────────────────────────────────────────────
   1  text, TAIL of the task     the chat template                  a specific rule   THE RULES   ← today
   2  saved KV state             llama_state_seq_save/load_file     the same, cheaper  the book, once stable
   3  control vector             llama_set_adapter_cvec             a disposition      THE CHARACTER ← next
   4  LoRA                       llama_set_adapters_lora            knowledge          a PROVEN book, endgame
   5  attention mask bias        our patch on ggml_soft_max_ext     weight on rung 1   only if 1 fails
```

**Why the rules are text, at the tail** — the pointing experiment (`KV-POINTING.md`): the
same rule held in both handlers at the tail of the task, in one at the head, in none as
bare cached K/V. Recency is the lever; a rule is a fact and a fact needs its words.

**Why the character is a control vector** — a control vector is a direction added to the
residual stream at chosen layers, made from contrastive prompt pairs by the bundled
`cvector-generator`, minutes on the GPU, no weights touched. It steers *who the model is*
and cannot carry a specific fact. That is exactly the character and exactly not a rule.
It is the first "deeper" that costs nothing at inference and fits what it is for.

**Why LoRA waits** — it burns the book into weights. Burn a wrong rule and it is wrong in
every task with no file to fix. The A/B must say the book is right first.

**The claim to state carefully.** The rules and the model share a process — putting a rule
in front of the model is a function call, not a network call. **The agent and the organ
talk over HTTP.** Never say "no HTTP between the parts".

## The two loops

```
THE FAST LOOP — free, runs constantly
   task → write → build → run → fails → fix → build → run → WORKS
   no frontier model, no money, electricity only

THE SLOW LOOP — costs money, runs rarely
   the same fault twice  ──▶ ask the tutor ──▶ answer now      (finishes THIS task)
   the task ends green   ──▶ teach         ──▶ rule in the book (makes the NEXT cheaper)
```

**Both halves are needed and only one was built first.** The Captain's correction,
14 Sep: *"we gave the tutor the rights to teach, but we never gave the student the right
to ask."*

## The verdict

A build proves the code compiles. It proves nothing about whether the thing works.

```
DOES NOT BUILD      the compiler refused it
BUILDS, NEVER RUN   compiled, never started — not done
RUNS BUT FAILS      started, an endpoint returned an error
WORKS               started and answered
```

His ruling, 14 Sep: *"we need to move away from this red and green build, it's daft."*

---

# PART TWO — THE RULINGS

Every one of these came from him, and every one of them changed the design.

**No logic outside the Rust binary.** The handshake IS the product. Python tools are
sketches that end up inside it.

**Regex is banned. Tree-sitter is banned.** No authored pattern judges anything. The
product is generic and public; nobody writes patterns for it. Three judges only: the
build, the run, and the tutor's eye. *Format* parsing stays legal — a JSON fence is a
format, not a meaning.

**The agent holds its character. The handshake holds every law.** Character is static and
correct to be static — who it is does not change when the repo grows. Laws grow with the
code, live in one place, and are injected at runtime. *"If we start moving them around, if
the law changes there is no way to inject it at runtime."*

**Anything that grows cannot be baked, and anything baked cannot grow.**

**A red build never reaches the tutor.** The compiler has already said what is wrong in
words the model can read. Paying a frontier model to restate a CS0246 is spending on a
fault that explains itself.

**The tutor runs once, at the end, over code that builds** — hunting the class of fault no
compiler can catch.

**The number on the screen is the provider's own count**, never an estimate.

**A task must name the project**, or the model invents a name. It invented a client-sounding one
once, which is a client repo name out of its own weights — verified not to have leaked
from any binary, book or repo of ours.

**Never complicate the code, attack the design.** The build motto, and it caught me every
time I reached for another patch.

**Test the condition, never the flag.** Mine, learned five times in one day. See Part Four.

---

# PART THREE — THE RULE BOOK

## The form

```json
{
  "id":      "scoped-context-in-handler",
  "kind":    "prohibit",
  "topic":   "words a task about this would contain",
  "applies": "the repo it was learned in",
  "never":   "what the model reached for",
  "instead": "the replacement — required, never empty",
  "allow":   "the one case where the never does not apply",
  "shape":   "<instead>; never <never>."
}
```

**`instead` is required.** A rule that says only *never* leaves the model with nothing to
do. Every bare prohibition we wrote broke; every rule naming its replacement held.

## Rule shape is not a style preference — it is measured

Research Fox surfaced, 15 Sep: **positive instructions hold at 100% across all turns.
Prohibitions decay from 73% at turn five to 33% by turn sixteen.** Agents remember what to
do and forget what not to do.

**Every rule that matters is a prohibition** — *never delete a customer*. So the form
matters enormously: lead with the instruction, carry the prohibition second.

Checked against our own book: fourteen of fifteen already did this. The one exception
predated the form and was reshaped.

## The fifteen, and a fault in them

The book holds fifteen rules. **Five of them are near-duplicates** — `define-domain-types`,
`define-dbcontext-and-entities`, `define-entities-in-program-cs`,
`missing-using-for-domain-types`, `using-correct-namespace`. All circling the same lesson:
*define the types you reference and import their namespace.*

**The cosine same-law check was built to prevent exactly this and did not.** The threshold
(0.90) is too strict for rules that say the same thing in different words. **Open
question:** lower it, or have the tutor merge rather than add when it is close.

This matters more than it looks. A book that accumulates five versions of one lesson will
accumulate fifty, and the whole design rests on the book staying small enough to send in
full.

---

# PART FOUR — EVERY FAULT, AND THE PATTERN

## The pattern, stated first because it is the most valuable thing here

**Five separate faults in one day, all the same shape: a condition that looked correct and
silently never held.** Not one of them threw an error. Every one reported success.

```
the tutor's rule had a missing comma      → discarded silently, reported "nothing to teach"
the dead-run flag                          → absent on every existing row, filtered nothing
the rules were never delivered             → --all was never passed, model saw 2 of 15
--all did not exist on handshake.exe       → would have failed every call, caught pre-ship
the forced-ask key                         → whole error text, never matched twice
```

**The rule: test the condition, never the flag.** A flag is a claim about the world. The
condition is the world.

**And the corollary: exercise it once before shipping it.** A syntax check is not a test.
I shipped a page I had never loaded (`date` never imported) and three guards I had never
run.

## The full list, 12–15 September

**Guards that reported non-violations as violations.** The handshake could not find its
book from the agent's folder, exited non-zero, and the agent told the model it had broken a
rule — when its edit was *correct*. It then spun trying to undo good work. Same with
`CHECKS INCOMPLETE` on an empty repo.

> **A guard that cannot tell *nothing ran* from *you broke a rule* teaches the model to
> undo correct work.**

**A flag computed from a list the code mutates.** The scaffolding flag read `projectDirs`,
and writing the first `.csproj` appended to it — so the first successful write disabled
the next twenty-two.

**A silent output ceiling.** 1500 tokens cut the model off mid-write with no error. It
looked like the model stopping; it was the client giving up.

**No network timeout.** Default 100 seconds, fine for a datacentre, wrong for a 4B writing
a long file. Four retries, a run killed at 489 seconds, and it looked like stubbornness.

**A path check that trusted a string prefix.** `C:\tmp\box-evil` passes a `StartsWith` test
for `C:\tmp\box`.

**A nameless file.** The model asked to write `csproj` — no name, just an extension. My
check tested that the name had an extension and never that it had a name. It then rewrote
that nameless file four times and blamed the environment.

**The escape check on the wrong path.** In the Rust port I checked for a drive prefix on
the *joined* path — and the sandbox lives on `C:`, so every legitimate file was refused.

**Two binaries, one build recipe.** `build-compile.cmd` built `compile` only, so the agent
was calling a `handshake.exe` from the day before that rejected flags added minutes
earlier.

**The prompt, twice.** The agent was sending the Captain's own Round Table governance
charter as the coding brief — credentials, banned channels, who signs a release — ending
with an instruction to call a tool the agent does not have. Then, after swapping to
`CHARACTER.md`, a line *I* had written — *"you say what you are about to do before you do
it"* — made a small model narrate instead of calling tools. Zero tool calls.

> **Anything in a small model's prompt that can be read as "describe" will beat anything
> that says "do".**

---

# PART FIVE — WHAT WE LEARNED ABOUT SMALL MODELS

## They do not refuse a tool. They never reach for one.

Two runs, `ask_tutor` registered, the instruction in the prompt, **zero calls** — while
rewriting identical bytes three and four times against a failure it could not diagnose.

This is a known weakness and we have it measured on our own bench. **Telling a 4B it may
ask is not enough.**

## The fix that worked: take the choice away

When the same fault comes back, the loop stops offering a question and **asks on the
model's behalf**, handing the answer back inside the tool result. It does not have to
choose to ask — it finds the answer in its hands.

**The measurement, same task, same model, same error:**

```
before   7 builds · 298s · tutor never called · DOES NOT BUILD
after    7 builds · 161s · asked twice        · WORKS
```

The error both times: `CS8803 — top-level statements must precede type declarations`. A
layout rule whose text names the fix. It could not do it alone across seven builds. Two
sentences from the tutor and it was past it. **Cost: half a penny.**

## When it is stuck, it blames the environment

Every single time. The port, the toolchain, the build system, *"environment limitations
beyond my control"*. It is never any of those. **The core now says so explicitly:** the
machine works, the fault is in your code, and if you cannot see it that is what asking is
for.

## It will rewrite identical bytes rather than stop

Measured: four consecutive edits of 938 chars → 938 chars. Four writes of the same nameless
`csproj`. **Repetition is the signature of being stuck**, and it is a better trigger than
any error text, because it is the same whatever the error says.

## Speed

Qwen3-8B gave about 41 tokens a second on the 4070 and took four minutes for fifty lines.
**Qwen3-4B-Instruct-2507 at Q4_K_M** thinks one to four seconds per step and built a
five-file project in under three minutes. The swap was the Captain's call and it was right.

**Do not confuse a slow model with a stuck one.** Look at the thinking times in the log.

---

# PART SIX — THE OPERATOR

## Bring it up

```
organ     cmd /c organ\llama-src\organ.cmd           → :8421, ~25s to load  (moved from C:\tmp 16 Sep)
console   python tools\console.py                    → :8420
juvina    watchdog brings it back at logon           → :8080
```

Start detached (`Invoke-CimMethod Win32_Process Create`) or it dies with the session.

## Build

```
rust\build-compile.cmd      BOTH binaries. sets vcvars, CUDA 13.4, CUDAARCHS=89, -j 4
agent\CompilerGPT.Agent     dotnet build
```

**`cargo build` must be `-j 4`.** `CMAKE_BUILD_PARALLEL_LEVEL` alone does nothing — cargo
passes its own `NUM_JOBS` and cmake built with 32, which put the laptop at 88% memory.

**A half-configured `llama-cpp-sys` out dir fails forever.** Rename it aside; a recursive
delete is refused by the permission layer, a rename is not.

## Landmines

**Clear the sandbox between runs.** Otherwise the agent inherits rubble and spends the run
repairing what a previous run broke. Rename aside, never delete — the old state is
evidence.

**The sandbox lock.** A shell sitting in it, or the dotnet build server, holds it.
`Set-Location` out, then `dotnet build-server shutdown`.

**The book is per repo — the run must name the repo.** Without `--repo`, every rule
carrying an `applies` tag is dropped silently.

**`--all` or the model sees a BM25 top-five.** Fifteen in the book, two reaching the model.

**A `.md` write can clobber an untracked `.md`.** Windows is case-insensitive: writing
`CORE.md` destroyed `core.md`.

**VPN silence.** While his work VPN is up, nothing outbound. The local loop still runs; the
tutor cannot be reached.

---

# PART SEVEN — WHAT IS PROVEN, AND WHAT IS NOT

## Proven

- The loop closes. One verified WORKS, by hand, not by the agent's claim.
- The forced ask changes the outcome, measured against a control.
- A direct question gets a correct answer in three lines for a fraction of a penny.
- Rules reach the model in full — ten of ten for the repo, verified.
- The tutor never sees the repository. Architecturally enforced: no file tool, no path.
- **16 Sep, the day it had to prove itself, before 07:00:** the create task WORKS
  (run 065055, 58s) and a feature added to that working code WORKS (run 065618, one edit,
  14s, five endpoints answered). Both under the rule: every handler, old and new, takes
  the context as a parameter. The feature-add had never had a clean run before.
- **Position is a lever — for sentences.** Same rule at the tail of the task held in both
  handlers; at the head, one; bare cached K/V, none (`KV-POINTING.md`). Rules go at the
  tail now.
- **Text delivery is robust; the cache is not (17 Sep, Results 2 and 3).** Across two
  tasks and both rule forms, the text arms — head and tail — held twelve of twelve
  handlers. The cache arms flipped between tasks with no pattern that survived a second
  run. The 08:03 claim that a demonstrated rule binds from the cache was **withdrawn at
  08:20 on its own second run.** What stands: rules as text at the tail, with code
  demonstrations, bind; cached K/V is an experiment still, not a tier.
- **The check is real (17 Sep).** The organ judges every written file against every
  demonstrated rule; six of seven faults of the week caught at their line, zero false
  flags on clean files; first live refusal at 07:33 (`server.js:7`, a hardcoded filename).
- **The student asks.** Three runs on 16 Sep, an unprompted `ask_tutor` in each, quoting
  the real error. The right to ask, used.
- **The teacher is the ceiling of the loop, and swapping it moves the ceiling.** 16 Sep
  evening, Node task 2, same words, same folder, same student: three fails with gpt-4.1
  as tutor (it taught a JSON-import syntax Node had removed; the student obeyed), then
  WORKS in 50s with one edit under Claude Sonnet 5. The tutor is Claude from now on — one
  tutor, never both. (`SERIES-NODE.md`)
- **The tutor must see the lines.** Given only "missing )", it guessed the wrong place
  twice; given the file around line 58, it named the real fault first time.
- **Three verdicts this morning were wrong because of the tools, not the model** — a stale
  binary run, a 404-on-empty counted as failure, a BOM hiding rule 1. Every one fixed the
  same morning. Judge the model on what it wrote, and check the judge first.

## Not proven — and this is the honest list

- **That a rule in the book stops the model repeating a fault, reliably.** Two clean runs
  on 16 Sep held `scoped-context-in-handler` in four of four handlers; the A/B on 15 Sep
  was mixed. Two is not a series. Keep counting.
- **That the forced ask is reliable rather than lucky once.** Fired twice on 16 Sep and
  the run went green; it is a net, not a proof.
- **That the core as character moves behaviour.** As a control vector at scale 4 it broke
  generation outright (`CHARACTER-VECTOR.md`). On the bench.
- **Anything about cost.** The framework does not surface token counts on multi-turn tool
  runs, though the organ returns them. The counter under-reports.

## The claim we must not make

**"We make rules binding."** The handshake puts a rule in front of the model; the model can
still ignore it — and did, 15 Sep morning with all fifteen delivered, and 16 Sep when the
check flagged `AddControllers` and the model wrote "none here" with it on line 7. What we
make is a **boundary**: the check fires on the write, and since 16 Sep `run_app` refuses
while a hit stands. Read-recited-skipped is still possible; it now costs the run.

**The honest claim is *corrections that survive*.** Fox's correction and he is right: the
series is credible because `FINDINGS.md` published a negative result, and overclaiming here
would undo that permanently.

---

# PART EIGHT — THE BLOCKER

**Update 16 Sep, corrected 14:45:** the repo was never without history — it holds **94
commits from 12–13 Sep** (the `git init` this morning re-initialised an existing repo).
**That history contains client names** in two committed files. The working tree is
scrubbed; history is not. **The MVP must go out as a fresh repository, not a push of this
one** — `docs/RELEASE-MVP-160926.md`, risk 1. The organ and the book live inside the tree
now (`organ/`, `book/`), not in `C:\tmp`. Still no remote, no LICENSE — the Captain's word.

**Nothing from 14–15 September is in git.** Five Rust modules — `book.rs`, `tutor.rs`,
`judge.rs`, `sandbox.rs`, `ffi.rs` — the whitepaper, the integration doc, this file, and
every Part 2 draft. **No remote. No LICENSE.** `agent/` is gitignored with a stale comment
saying it moved elsewhere, and zero files are tracked under it.

A stranger reading Part 2 cannot obtain the thing it describes. **A stranger following our
own `STRANGER.md` fails at step two.**

**Three decisions, all the Captain's:** commit, choose a licence (MIT or Apache-2 — Docker's
catalogue refuses GPL), add a remote.

**His ruling, 15 Sep, and it was the right one:** no commit until the work is proven.
Committing unproven work makes it look settled. It is now proven once.

---

*Written 15 Sep 2026. Update it when something is learned, not when something is claimed.*
