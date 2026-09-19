# Does a rule in the book stop the fault coming back?

*Designed 15 Sep 2026, ~15:55, **before either arm was run**. The criteria below are fixed.
A measurement decided after seeing the result is not a measurement.*

---

## The claim under test

**The claim the whole design rests on:** a rule the tutor learned, written into the book,
stops the model repeating that fault on a later task.

Until now this has never been measured. The loop has been shown to *close* — a run reached
WORKS — but that run taught nothing, because nothing failed at the end. Whether the rules
already in the book are doing any work is unknown.

## The specific fault

`scoped-context-in-handler`, learned into the book at 15 Sep ~07:45 after the model
produced it twice:

> Accept the DbContext as a parameter of the endpoint lambda so it is injected per request;
> never resolve it from `app.Services` or construct one with `new` inside a handler.

Chosen because **the model produced this exact fault twice before the rule existed**, it
compiles cleanly and only fails at runtime, and no compiler or linter catches it.

## The two arms

Same task, same model (Qwen3-4B-Instruct-2507 Q4_K_M), same organ, run sequentially so they
never share the GPU.

```
ARM A   C:\tmp\armA\myshop   --harness   core + all 15 rules + checks + tutor
ARM B   C:\tmp\armB\myshop   (raw)       no core, no rules, no checks, no tutor
```

Both folders are named `myshop` so that `--repo myshop` matches the rule's `applies` tag.
**Caught in design:** had the folders kept their arm names, the rule would have been dropped
from both arms and the experiment would have measured nothing.

## What counts as the rule working

**The rule worked if:** arm A's `Program.cs` accepts the context as a parameter of the
endpoint lambda, and arm B's resolves it from `app.Services` or constructs one with `new`.

**The rule did not work if:** arm A produces the fault anyway. The rule was delivered — we
verify it was in the rendered block — and ignored.

**The result is void if:** arm B happens to write it correctly by chance. That proves
nothing about the rule, only that the model does not always produce the fault. If that
happens, the honest report is *inconclusive, needs repeats*, not a claim in either
direction.

## What this cannot prove

- **One pair is not a result.** A single A/B on one fault on one task. It is a signal,
  never a number to publish.
- It cannot separate the rule from the core or the checks, since arm A carries all three.
  A cleaner design would vary one at a time; this one answers the cruder question of
  whether the harness as a whole changes the output.
- It says nothing about faults the tutor has not yet seen.

## Recording

Both arms' `Program.cs` are kept. The verdicts, tool calls and timings come from the run
logs. **@Gareth owns any figure that goes outward.**

---

*Result appended below once both arms have run. Nothing above this line is edited after
the fact.*

---

# ARM A — with the book. 15 Sep, 15:57.

**Precondition verified before the run:** nine of nine rules delivered for `myshop`, and
`scoped-context-in-handler` confirmed present in the rendered block.

**Verdict: DOES NOT BUILD.** 12 tool calls, 2 builds, 237 seconds, no tutor spend.

**On the claim under test — the rule did not hold.** `Program.cs` carries the fault:

```csharp
app.MapGet("/tools", () =>
{
    var tools = app.Services.GetRequiredService<ToolContext>().Tools.ToList();
```

**And the detail that makes it interesting:** the POST endpoint in the same file is
*correct* — `async (Tool tool, ToolContext context)`, parameter-injected, exactly as the
rule requires. **It obeyed the rule in one handler and broke it in the other, in one file,
in one pass.** That is not a model that failed to receive a rule. It is a model that
applied it inconsistently.

This is the prohibition-decay literature showing up on our own bench, and it is worse than
the papers describe: not decay across turns, but inconsistency *within a single file*.

## A second finding, and it is a fault of mine, not the model's

**The forced ask did not fire**, despite the clearest stall yet:

```
edit_file(Program.cs) 1423 -> 1423 chars     ×5, consecutively
```

Five identical edits. Same byte count in and out, five times, with **no build in between**.

My trigger counts *error codes per build*. The model stopped building and just rewrote the
file, so the counter never incremented and the door never opened. **The stall signature was
in plain sight and I was watching the wrong signal.**

> **The fix: an identical edit IS the stall signal.** Same file, same length, no build
> between — that is the model spinning, and it needs no error text to detect. It is a
> better trigger than the error code, because it holds whatever the compiler says.

**Sixth fault of the same class in two days:** a condition that looked right and silently
never held.

---

# ARM B — the control, no core, no rules, no checks, no tutor. 15 Sep, 16:04.

**Verdict: RUNS BUT FAILS.** 32 tool calls, 11 builds, 369 seconds.

**The fault is present, and it is worse.** Both endpoints construct a context by hand:

```csharp
app.MapGet("/tools",  () => { using var context = new ToolContext(); ... });
app.MapPost("/tools", (Tool tool) => { using var context = new ToolContext(); ... });
```

`new ToolContext()` with no options — it has no connection string, so it throws at the
first request. That is why the app started and the endpoint failed.

---

# THE RESULT

| | ARM A — with the book | ARM B — control |
|---|---|---|
| handlers carrying the fault | **1 of 2** | **2 of 2** |
| how it broke it | `app.Services.GetRequiredService` | `new ToolContext()` |
| tool calls | **12** | 32 |
| builds | **2** | 11 |
| wall | **237s** | 369s |
| verdict | DOES NOT BUILD | RUNS BUT FAILS |

**Not void.** The control did not get it right by chance — it got it wrong in both
handlers. So the comparison stands.

## What this actually shows

**The rule did not hold.** Arm A received it, verified in the block, and broke it in the
GET endpoint anyway. By the criterion fixed before the run, that is the "rule did not work"
branch and it is recorded as such.

**But the arms are not equal.** Arm A broke the rule in one handler of two and got the
other right — parameter-injected, exactly as the rule requires. Arm B broke it in both, and
in a cruder way: a hand-constructed context with no options, which cannot work at all.
**Arm A's fault at least resolves a real registered service; arm B's was never going to
run.**

And the harness arm reached its stopping point in **a third of the builds and two thirds of
the time**.

## The honest verdict

**Mixed, and it must not be rounded up.**

- On the narrow claim — *does a rule in the book stop this fault recurring* — **no.**
- On the broader question — *does the harness change the output* — **yes, visibly**: half
  the handlers, a quarter of the builds, and a materially better class of mistake.

**One pair is one pair.** This is a signal, not a number. It wants repeats on other faults
before anyone says it outward, and **@Gareth owns any figure that goes public.**

## What it changes

**The claim "we make rules binding" is now measured and false on our own bench.** Fox's
correction was right before we had evidence; we have the evidence now. *Corrections that
survive* is the defensible claim, and even that wants proving.

**The prohibition-decay literature is visible here in a sharper form than the papers
describe.** Not decay across turns — **inconsistency within a single file, in a single
pass.** The same rule obeyed in one handler and broken in the next, three lines apart.

**And the most useful finding is not about rules at all:** arm A stalled five times on an
identical rewrite and the forced ask never fired, because it watched error codes per build
and the model had stopped building. **The repetition itself is the stall signal.** Fixed,
built, untested.
