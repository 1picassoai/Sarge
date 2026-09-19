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

Run: `rust\replay-check.cmd` — one line per file, HIT or NONE, and a verdict at the end.

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
