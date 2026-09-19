# The agent SDK integration — where it stands

*Written 15 Sep 2026, so the next session does not rediscover today's faults.*

---

## The chain

```
   YOU (web UI, :8420)
        │  task + folder
        ▼
   THE AGENT  —  .NET, Microsoft Agent Framework
        │  tools: list · read · edit · write · run_build · run_app · ask_tutor
        │  character from harness/CHARACTER.md, rules from the handshake
        │
        ├──── HTTP ────▶ THE ORGAN (:8421)
        │                llama-server + handshake linked in, Qwen3-4B on the GPU
        │
        └──── shells ──▶ handshake.exe
                         --task   select the rules for this task
                         --check  what was written
                         --learn  call the tutor, vet the rule, write the book
                         --ask    the student's question, answered now
```

**The one architectural claim to state carefully.** The rules and the model share a
process — putting a rule in front of the model is a function call, not a network call.
**The agent and the organ talk over HTTP.** Do not say "no HTTP between the parts".

---

## What is wired and proven

- **Rules delivered in full.** `--all` on `handshake.exe`, verified ten of ten for the repo.
- **The run is the verdict**, not the build: WORKS · RUNS BUT FAILS · BUILDS, NEVER RUN ·
  DOES NOT BUILD.
- **The tutor teaches once**, at the end, over code that builds — never on a red build,
  because the compiler has already said what is wrong.
- **The student can ask** — `ask_tutor`, after a real failure, two per task. Smoke-tested
  against the real scoped-service fault: correct answer in three lines.
- **The counter is honest.** Local tokens from the framework's usage block, tutor spend
  from the provider's own count, dead runs excluded.

## What is built but unproven

- **The forced ask.** On a repeated identical failure the loop asks the tutor on the
  model's behalf and hands the answer back inside the tool result. Built 15 Sep, **never
  yet fired** — it needs a run where the same failure recurs.
- **The core as character.** `CHARACTER.md` now carries "you never make the same change
  twice", "you ask", "you do not diagnose the environment". Untested against a real run.

## The open problem

**A small model does not refuse a tool — it never reaches for one.** Two runs, `ask_tutor`
registered, the instruction in the prompt, **zero calls**, while rewriting identical bytes
three and four times against a failure it could not diagnose.

This is a known weakness of small models and we have it measured on our own bench. The
forced ask is the floor under it. Whether the core alone moves the behaviour is the next
thing to find out.

---

## Faults found on 15 Sep, all mine, all fixed

1. **The rules were never delivered.** The agent asked the handshake without `--all`, so it
   got a BM25 top-five. Fifteen rules in the book, two or three reaching the model. Every
   lesson the tutor had learned sat unread.
2. **`handshake.exe` had no `--all` flag** — only `compile.exe` did. Caught before shipping:
   it would have failed every rules call and sent the model in with *no* rules.
3. **The build recipe built one binary of two**, so the agent would have talked to a stale
   handshake that rejected a flag added minutes earlier.
4. **No network timeout**, so the client gave up at 100 seconds while a 4B was mid-file,
   retried four times, and killed a run at 489 seconds. Now ten minutes.
5. **A false number on the counter.** A dead run reported zero tokens and computed a
   *negative* saving. I added a flag to exclude those rows and **the filter did nothing** —
   the flag was absent on every existing row. Fixed on the real condition: no tokens means
   no measurable work.

**The pattern in all five:** a guard or a filter that silently did nothing, and reported
success. Same shape as the tutor's missing comma. **Test the condition, never the flag.**

---

## Landmines

- **Build with `rust/build-compile.cmd`** — it sets vcvars, CUDA paths, `CUDAARCHS=89`, and
  caps at four cores. It now builds **both** binaries.
- **`cargo build` must be `-j 4`.** `CMAKE_BUILD_PARALLEL_LEVEL` alone does nothing.
- **The sandbox lock.** A shell sitting in it, or the dotnet build server, holds it.
  `Set-Location` out, then `dotnet build-server shutdown`.
- **Clear the sandbox between runs**, or the agent inherits rubble and repairs what a
  previous run broke. Rename aside rather than delete — a recursive delete is refused.
- **The book is per repo — the run must name the repo.** `--repo`, or every rule carrying
  an `applies` tag is dropped.

---

## ⚠ The blocker that matters more than any of this

**Nothing from the last two days is in git.** The whole Rust rewrite — `book.rs`,
`tutor.rs`, `judge.rs`, `sandbox.rs`, `ffi.rs`, `build.rs` — is untracked. So is this file,
the whitepaper, and every Part 2 draft. `agent/` is gitignored with a stale comment saying
it moved elsewhere, and **zero files are tracked under it**. There is **no git remote** and
**no LICENSE**.

A stranger reading Part 2 cannot obtain the thing it describes.

**Three decisions, all the Captain's:** commit the agent, choose a licence (MIT or Apache-2
— Docker's catalogue refuses GPL), and add a remote. Until then the series describes
something nobody else can run.
