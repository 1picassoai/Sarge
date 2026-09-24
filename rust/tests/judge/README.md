# The check's replay — every fault of the week, and its clean twin

The Captain's law: *a check that has never fired on a real violation is not a check.*
Before the organ-judge (`rust/src/verdict.rs`) touches a live run it is replayed here:
it must say HIT on every `*-bad.*` file at the right line, and NONE on every `*-good.*`.

| file | rule it must hit | from |
|---|---|---|
| `cs-ensuredeleted-bad.cs` | `archive-not-delete` (never remove data) | task 5e, 16 Sep — the unguarded drop went green |
| `cs-newcontext-bad.cs` | `scoped-context-in-handler` | 15 Sep A/B, the pointing experiment |
| `cs-addcontrollers-bad.cs` | `minimal-api` | 16 Sep — "no AddControllers here", with it on line 7 |
| `cs-appservices-bad.cs` | `scoped-context-in-handler` | 15 Sep evening, both handlers |
| `js-hardcoded-db-bad.js` | `use-config-for-connection` | Node task 1, the first rule the tutor wrote |
| `js-await-sync-bad.js` | `no-await-on-sync-database` | Node task 2 |
| `js-hardcoded-table-bad.jsx` | `fetch-real-data-not-fixtures` | Node task 6c |
| `js-long-bad.js` | faults inside wrapped multi-line calls | @Galahad, 24 Sep — the first long faulty file |
| `js-long-good.js` | **NONE** — 12 near-identical handlers, all correct | @Galahad, 24 Sep — the first long clean file |

Run: `rust\replay-check.cmd` — one line per file, HIT or NONE, and a verdict at the end.

**The two long files are here because of what their absence cost.** Until 24 Sep every
fixture above was 14 to 16 lines with a single fault on a single line — nothing the shape of
a repo. Two bugs lived in that gap for a week and neither was visible to this corpus: the
check lost track of which rule was which on a long file and reported CHECKS PASSED on four
real faults, and the gate waved through every hit for a rule it could not distinguish and
put **twelve false flags on sixty-nine lines of correct code**. Both were found by a file
that was not in here. A corpus of short files measures a check on short files.

## Replay result, 17 Sep 07:15 — the version that shipped

| file | expected | got |
|---|---|---|
| `cs-ensuredeleted-bad.cs` | HIT archive-not-delete | ✔ line 10 |
| `cs-ensuredeleted-good.cs` | NONE | ✔ |
| `cs-newcontext-bad.cs` | HIT scoped-context-in-handler | ✔ line 10 |
| `cs-appservices-bad.cs` | HIT scoped-context-in-handler ×2 | **✘ missed** — the judge reads the rule's "allowed at startup" clause and forgives the handler |
| `cs-addcontrollers-bad.cs` | HIT minimal-api | ✔ line 7 |
| `cs-clean-good.cs` | NONE | ✔ |
| `js-hardcoded-db-bad.js` | HIT use-config-for-connection | ✔ line 7 (+ config-not-code, correct) |
| `js-await-sync-bad.js` | HIT no-await-on-sync-database | ✔ line 11 |
| `js-clean-good.js` | NONE | ✔ |
| `js-hardcoded-table-bad.jsx` | HIT fetch-real-data-not-fixtures | ✔ line 10 |

**Six of seven faults caught; zero false flags on clean files.** Four versions on the way
here, each replayed: raw parse (0 caught — the organ names rules by number, not id) →
number-aware + yes/no confirm (7 caught, 3 false flags) → quote-the-forbidden-part (worse:
5 false flags, a model can always find a substring) → demonstrated-only + likeness + token
gate + windowed second look (6 caught, 0 false). A false flag blocks a correct run; a miss
is caught by the tutor's review or by hand. Precision was chosen.

**Known miss:** a DbContext resolved from `app.Services` *inside a handler* when the rule
carries an `allow` for the startup scope. To be revisited when the rule's `allow` is given
its own `right` example.

## Replay result, 18 Sep evening — the books now ship with the fixtures

The release review, at the gate (refused, 21:11): the replay pointed at two repos' books on one
machine, both since deleted, so nobody could re-run it. Now `csharp.sarge` (reconstructed
from `docs/SARGE-SYNTAX.md`; the original `myshop` book is gone) and `node.sarge` (the
original Node book) sit beside the fixtures, `replay-check.cmd` reads them, and
`book/universal.sarge` loads under both as it does for any repo.

| file | expected | got, 18 Sep ~22:30 |
|---|---|---|
| `cs-ensuredeleted-bad.cs` | HIT archive-not-delete | ✔ line 10 |
| `cs-ensuredeleted-good.cs` | NONE | **✘ false flag** — `EnsureCreated()` judged as "recreate" even with an `allow` naming it |
| `cs-newcontext-bad.cs` | HIT scoped-context-in-handler | ✔ line 10 |
| `cs-appservices-bad.cs` | HIT scoped-context-in-handler ×2 | **✘ missed** — the startup-scope `right` line forgives the handler |
| `cs-addcontrollers-bad.cs` | HIT no-addcontrollers-in-minimal-api | ✔ line 7 |
| `cs-clean-good.cs` | NONE | ✔ |
| `js-hardcoded-db-bad.js` | HIT use-config-for-connection | ✔ line 7 |
| `js-await-sync-bad.js` | HIT no-await-on-sync-database | ✔ line 11 (+ sync-db-no-await, its twin) |
| `js-clean-good.js` | NONE | ✔ |
| `js-hardcoded-table-bad.jsx` | HIT fetch-real-data-not-fixtures | ✔ line 10 |

**Six of seven caught; one false flag; on the books that ship.** Three C# runs tonight
with three wordings of the same two rules gave 5/7 + 0 false, 6/7 + 1 false, and 6/7 + 1
false on different files. That variance is the finding, not a tuning target: **the judge is
the loaded model, and the number moves with the wording of the book and the model that
reads it.** The 7-of-7 measured on 18 Sep morning was against the `myshop` book, which no
longer exists; it cannot be re-run and is not claimed. Run this yourself; read the number
you get.

## Replay result, 19 Sep afternoon — smaller judges, and a fault the blank answer hid (UNRELEASED tree, after v0.1.0; the tag is unchanged)

The question was whether the check could run on a smaller model than the 4B, so that a laptop
without an 8 GB card could judge. Same fixtures, same books, the organ swapped for
`Qwen3-1.7B-Q4_K_M` and then `Qwen3-0.6B-Q4_K_M`, same flags.

| judge | faults caught | false flags | note |
|---|---|---|---|
| Qwen3-4B-Instruct-2507 (shipped) | 6 of 7 | 1 | unchanged before and after the fix below |
| Qwen3-1.7B | **0 of 7** | 0 | answers `NONE` to every file, first pass, deterministic across three runs on a fresh organ |
| Qwen3-0.6B | **0 of 7** | 0 | same |

**The fault the run exposed first:** both small models are *thinking* models. The check's
request never turned thinking off, so they spent the whole answer budget inside `<think>` and
returned an empty string, and the check read an empty string as `NONE` and passed the file.
Fixed in `verdict.rs`: the request now sends `chat_template_kwargs.enable_thinking = false`
(a non-thinking model ignores it; the 4B's numbers did not move), and **an empty answer is an
error, not a pass** (`CHECKS INCOMPLETE`). A judge that says nothing has not judged.

**The finding once they could speak:** they still catch nothing. With thinking off both models
answer `NONE` on every fixture, including the `EnsureDeleted()` drop and the `await` on a
synchronous call that the 4B names at the right line. So the earlier note stands and hardens:
**the judge is the loaded model, and at this prompt the 4B is the floor.** The route to a
machine without an NVIDIA card is a Metal or CPU build of the same 4B, not a smaller model.

One caution from the day: two scripts that both swap the organ on :8421 were run at once,
and one traced "hit" from the small model turned out to be the 4B answering mid-swap. Swap
the organ from one place, sequentially, or the trace lies.

## 24 Sep — an empty gate was a free pass, and seven rules could never have held

**What @Galahad found.** The morning's fix (rules by name, two per call, statements not
lines) genuinely repaired a long faulty file. Run against a long *clean* file it produced
**twelve false flags**, every one `204-carries-no-body` on `res.json(rows)`, with no 204
anywhere in the 69 lines. A false flag refuses to let correct code run, which is the one
thing this tool must never do.

**The cause, which was not the morning's change.** The gate keeps a candidate only if the
line carries a token the rule's `wrong` examples have and its `right` examples lack. For
`204-carries-no-body` that difference is `json` and `true` — both on the NOISE list — so the
set came out **empty**, and an empty set used to mean *keep everything*. Twelve lines went to
a 4B asked "is JSON beside a 204 wrong?", and it said WRONG twelve times. The old code gated
on `h.matched`, which for these hits is the same `res.json(rows);`, so **v0.3.0 would have
done the same** — the widening never caused it.

**The fix, in two halves.**

*An empty set now means the check cannot enforce that rule*, so the hit is dropped and the
rule is never asked about at all (`enforceable()` in `verdict.rs`). Waving a rule through
because there is nothing to check it with is not a check.

*Seven rules were struck from `book/node.sarge`* — `disable-the-framework-banner`,
`port-from-the-environment`, `401-is-not-403`, `promise-all-for-independent-work`,
`set-explicit-content-type-for-non-json`, `204-carries-no-body`, `explicit-cors-origins`.
Each breaks the line-judge law: **the proof sits elsewhere in the file.**
`const app = express()` is correct code — it is only wrong if `app.disable('x-powered-by')`
appears nowhere, and on the clean file it is on line 6, outside the window. A rule whose
`right` example *contains* its `wrong` example can never be judged one line at a time. Book
32 → 25.

| | before | after |
|---|---|---|
| `js-long-good.js` (correct code) | **12 false flags**, 32s | **0**, 17s |
| `js-long-bad.js` (4 real faults) | 4 caught | **2 caught** |
| replay corpus | green | green |
| hook suite | 8 of 8 | 8 of 8 |

**The two faults now missed are `port-from-the-environment` and
`disable-the-framework-banner` — struck rules, and the trade is deliberate.** Both were only
ever "caught" by a rule that fires on correct code as readily as on broken code; the same
mechanism that flagged them flagged twelve good lines. A rule that cannot tell the two apart
is not catching anything, it is guessing and being right sometimes.

**What this does not fix:** a file-scope check. "This file never disables the banner" is a
real fault and a real rule — it is simply not a *line* question, and it wants a pass that
reads the whole file once rather than a window of eight lines. Struck, not forgotten.
