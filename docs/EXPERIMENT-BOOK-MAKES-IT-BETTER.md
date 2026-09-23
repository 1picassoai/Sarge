# Does the book make the same model better at THIS repo?

*Designed Tuesday 22 September 2026 ~07:00, **before either arm was run.** The criteria below
are fixed. A measurement decided after seeing the result is not a measurement.*

---

## Why this one exists

The Captain's ruling this morning, on why the concept is not landing: *"people see Sarge and
it feels like a linter. The pitch is wrong, the product is right. It should be: make your
local model better at YOUR repo's coding. A small local model is always decent at generic
coding. How does it get better at your own repo? That's the concept."*

A linter never improves. If that pitch is true, it has to be demonstrable: **the same model,
same task, same machine — one of them has the book this repo taught it, and it writes better
code.** This experiment is the picture that is missing from the README.

## The claim under test

**Given an identical task, a model carrying a book of rules learned from its own past
failures on this stack produces code that a model with an empty book does not.**

## The two arms

Same model (Qwen3-4B-Instruct-2507 Q4_K_M), same organ on :8421, same task, run one at a time
so they never share the GPU.

```
BEFORE  C:\tmp\demo\before\workshop   .sarge is an empty frame; only the three universal
                                      laws load underneath, from the binary
AFTER   C:\tmp\demo\after\workshop    .sarge carries 16 rules the tutor learned from THIS
                                      model's own failures on this exact stack, 16-18 Sep,
                                      copied from workshop-node-py4
```

Both folders are named `workshop` so a rule's scope matches in both arms. **Two rules were
left out of the AFTER book on purpose** — `verify-fetch-reaches-backend` and
`verify-endpoints-before-claiming-done` — because both were struck on 18 Sep as process
advice whose `wrong` line was actually correct code. Carrying a known-bad rule into a
demonstration would be dishonest.

## The task, identical in both arms

> Create a Node.js Express API named WorkshopTools in this folder using ES modules. Write
> package.json with "type": "module" and express, server.js, and config.json holding the
> database file name. Use node:sqlite - `import { DatabaseSync } from 'node:sqlite'`. Create
> a tools table with id, name and price at startup. Add GET /tools and POST /tools. Listen on
> PORT or 3000. Build it, run it, and check GET /tools answers.

## What counts, decided now

**The book worked if** the BEFORE arm produces at least one fault that an AFTER-book rule
names, and the AFTER arm does not produce it — either because it never wrote it, or because
the check caught it and the run refused until it was fixed.

The faults in scope, and the rule that names each:
- a hardcoded database filename or connection string -> `use-config-for-connection`
- `await` on a synchronous `DatabaseSync` call -> `no-await-on-sync-database` / `sync-db-no-await`
- the same route path+method registered twice -> `no-duplicate-route-definitions`

**The book did not work if** the AFTER arm produces one of those faults anyway and reaches a
verdict without the check stopping it. A rule that is delivered and ignored is a failure of
the claim, and `EXPERIMENT-RULES-STICK` (15 Sep) already recorded exactly that happening
within a single file.

**The result is VOID if** the BEFORE arm happens to write everything correctly. That proves
nothing about the book, only that the model does not always produce the fault. The honest
report is then *inconclusive, needs repeats* - not a claim in either direction.

## What this cannot prove

- **One pair is not a result.** One task, one stack, one model. A signal, never a number to
  publish, and never a percentage.
- It cannot separate the book from the check or the tutor: the AFTER arm carries all three.
- It says nothing about faults the tutor has not already seen. The book only holds what this
  model got wrong before.
- It is not the Drill. The book is learning by notes, in the prompt. Whether learning by
  weights does better is a separate question and a separate week.

## Recording

Both arms' `server.js` and `config.json` are kept. Verdicts, tool calls and timings come from
the run logs. **@Gareth owns any figure that goes outward.**

---

*Result appended below once both arms have run. Nothing above this line is edited after the
fact.*
