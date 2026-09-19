# The Node + React series — the public comparison

*Captain's ruling, Wed 16 Sep ~08:05: the comparison test moves to Node.js + React "as the
GitHub crowd understands that better." The .NET series (`SERIES-160926.md`) stands as the
first proof: six of seven passed with the harness. This series is the one that gets
written up. Same shape, a stack people know, HARNESS and RAW on identical tasks.*

## The repo

`repos\workshop-node` — **WorkshopTools again**, so the two series
compare across stacks. Express API over SQLite using Node's built-in `node:sqlite`
(no native build, no ORM — Node 24 on this machine), a React client built with Vite in
`client/` and served as static files by the same Express process, so **one process, one
port, one probe**. Plain JavaScript, ES modules.

Nothing leaves the machine except `npm install` fetching packages from the registry — that
is the one outbound call, and only when no VPN is up.

## The book

A fresh one, in the language: `workshop-node\.sarge` at the repo's root, created as an
empty frame the first time the page sees the folder. The three universal laws
(`book/universal.sarge` — config not code · archive not delete · explicit columns) are
loaded under it by the binary. Everything else the tutor teaches from the model's own
failures, as `.sarge` blocks with `wrong` and `right` lines. **Watching the book grow from
three rules on a stack people know is the demo.**

Where a file sits is its scope — the Captain's ruling, 16 Sep.

## The seven tasks — HARNESS first, then RAW, same words, same order

**1. Create.** Create a Node.js Express API named WorkshopTools in this folder using ES
modules. Write exactly three files: package.json (with "type": "module", a "start" script
running server.js, and express as the only dependency), server.js, and config.json holding
the database file name. Use Node's built-in node:sqlite for storage — `import { DatabaseSync }
from 'node:sqlite'` and `new DatabaseSync(file)` with `prepare(...).all()` and `.run()` —
reading the database file name from config.json with `import.meta.dirname`. Create a tools
table with id, name and price at startup if it does not exist. Add two endpoints: GET /tools returns all tools, POST /tools adds one.
Listen on the port in the PORT environment variable, default 3000. Build it, run it, and
check GET /tools answers. It must install, start and answer.

**2. By id.** Add GET /tools/:id returning one tool or 404, and PUT /tools/:id updating a
tool's name and price, to the existing WorkshopTools API. Build it, run it, and show both
endpoints answering.

**3. Validation.** Add validation: POST /tools and PUT /tools/:id must return 400 with a
short message when name is empty or whitespace, or when price is not a number greater than
zero. Build it, run it, and show a bad request returning 400 and a good one still working.

**4. Search and paging.** Add search and paging to GET /tools: optional query parameters
search (matches name, case-insensitive), page (default 1) and size (default 10, max 50).
Return only id, name and price. Build it, run it, and show GET
/tools?search=ham&page=1&size=5 answering.

**5. Archive.** Add DELETE /tools/:id. This codebase never hard-deletes: add an is_archived
column to tools (adding it to the existing table if it is missing), DELETE sets it and
returns 204, and GET /tools and GET /tools/:id no longer return archived tools. Build it,
run it, and show a tool disappearing from GET after DELETE.

**6. React client.** Add a React client built with Vite in a client/ folder that shows the
list of tools from GET /tools in a table with name and price. Add a "build" script to the
root package.json that builds the client, and make server.js serve client/dist as static
files at /. Build it, run it, and show GET / returning the page and GET /tools answering.

**7. Stats.** Add GET /tools/stats returning the number of tools that are not archived and
the total of their prices, and show it on the React page above the table. Build it, run
it, and show GET /tools/stats answering.

## The smoke before the series — 16 Sep, scratch folder `smoke-node`

Two runs of task 1 on a scratch folder before the Captain's first Run, to catch plumbing:

- **Run 1 (VPN came up mid-run):** RUNS BUT FAILS. A one-key `config.json` rejected as a
  placeholder (fixed); the tutor unreachable (VPN — the law is now in the binary); the
  model does not know `node:sqlite` and wrote the npm `sqlite3` API — the task now names
  the API.
- **Run 2 (VPN down):** WORKS in 223s. Forced ask fired twice and pulled it through. **The
  tutor wrote the first rule in the language** into `smoke-node\.sarge` — `do-not-hardcode-
  config`, with real `wrong`/`right` lines. **But by hand: POST killed the process** — no
  `express.json()`, `req.body` undefined. `run_app` only sent GETs. Fixed: checks now carry
  a method and a body, and the model is told to exercise every endpoint the task named.

Three tool faults found and fixed before a single recorded run. That is what a smoke is for.

## What passes

Same three criteria as the .NET series, fixed before the first Run: the agent's verdict is
WORKS · it is correct by hand · the rules in the book held. Every run logged in the table
below. Verdicts are the agent's; the by-hand column is mine; anything outward is
@Gareth's.

## The check's first live run — 17 Sep 07:33, HARNESS on the raw folder by mistake

The Captain ran task 1 with the arm still on HARNESS, on `workshop-node-raw`. Not the bet
— but the first run with the organ-judge live, and it did its job: **`server.js:7`
`const databaseFile = 'tools.db';` — a real hit on `config-not-code` — and the run was
refused.** Verdict NEVER RUN: the app never started because a rule stood broken. First
live refusal on record. One false flag beside it: the check judged `config.json` itself
for "hardcoding the database name" — the config file is where the name belongs — and the
model chased that instead of the real line. Fixed: only source files are judged; data and
config are not. The folder is contaminated for RAW purposes and must be emptied first.

## The bet

The Captain: RAW fails every task. Merlin: RAW fails the series on the rules standard, but
the easy tasks will build and run.

| # | task | HARNESS run | verdict | by hand | RAW run | verdict | by hand |
|---|------|-------------|---------|---------|---------|---------|---------|
| 1 | create | 170639 | WORKS | ✔ GET 200 · POST 201 · GET shows it · 73s · forced ask ×1 · tutor wrote `use-config-for-connection` with wrong/right into `.sarge` | 074656 (074034 void) | **RUNS BUT FAILS** | ✘ app dies at start: `const fs = require('fs')` in an ES module. Zero asks, zero rules. Three identical whole-file rewrites. Then: *"I'll now submit that this task is complete despite the build/run issues, as the code structure and logic are correct."* 123s. (074034 is void — the net fired on RAW before it was gated; it passed *because* of one tutor answer) |
| 2 | by id | 171411 | RUNS BUT FAILS | ✘ routes right, but `GET /tools/1` kills the process: `.all([id])` — the old npm-sqlite array style; `node:sqlite` wants `.all(id)`. **Knowledge gap, not a rule.** 209s · forced ask ×2 · then the **same 881-byte edit four times** · tutor: same law | 085038 | **RUNS BUT FAILS** | ✘ both routes written and shaped right (`.get(id)`, `changes === 0` → 404) — but the app still dies at start: `config.json`, which RAW itself wrote in task 1, holds **backslash-escaped JSON** (`{\"database\": …}`), so `JSON.parse` throws at position 1. In two tasks the bare model never once read `config.json`; it rewrote `server.js`'s module syntax three times blaming itself, sent the **same no-op edit four times**, then: *"the code logic is sound and would work in a proper environment."* 202s. RAW 0 of 2 |
| 2b | by id, API named in the task | 172212 | RUNS BUT FAILS | ✘ the parameter fix is **right** (`.get(Number(id))`) — but partial edits doubled the imports (`path`, `DatabaseSync` declared twice), then `import config from './config.json'` without `with { type: 'json' }` kills the app at start. 287s · forced ask ×2 · **the new STUCK stop fired** and ended it instead of the context dying · the tutor then wrote a junk rule with the template's placeholder id — my prompt's fault, fixed, the loader now refuses a placeholder id | | | |

| 2c | by id, original wording, book holds 2 rules, tutor's words logged | 194919 | RUNS BUT FAILS | ✘ **the tutor was wrong**: told the student `import … assert { type: 'json' }` — removed syntax; Node 24 wants `with`. Proved on this machine. Student obeyed, build went red, thrashed; second answer correct but generic; **four-strike stop ended it at 167s** · tutor: same law | | | |

| 2d | by id, same words, **tutor = Claude Sonnet 5** | 202537 | **WORKS** | ✔ GET /tools/1 200 · PUT 200 with the update · GET shows it · GET /tools/99 404 · **50s, one edit** · the tutor wrote `sync-db-no-await` (a near-duplicate of the rule already there — the same-law check let it through; VET note) | | | |

**The finding of the series so far: the tutor's knowledge cutoff is the ceiling of the
loop — and swapping the teacher moved the ceiling.** Same task, same words, same folder,
same student: gpt-4.1 as tutor failed three times; Claude Sonnet 5 as tutor passed in
50 seconds with one edit. The ask fired, the answer was delivered, the student did as told, the stop worked
— every part below the teacher worked; the teacher was out of date. The MVP ruling
(the user's own session as tutor, able to read the docs) is not a convenience; this run
is why it is the design.

**Honest read after 2 and 2b (superseded by the Captain's ruling, 17:35 — "that's
rigging"; the series stands as written):** the Node series is measuring the model's *knowledge* of
2024-era Node (`node:sqlite`, JSON import attributes), not the rules. The .NET series
scored six of seven because the model knows .NET 8 cold. Recommendation to the Captain:
re-base the Node series on things the model knows — Express with a **JSON file as the
store** (`fs`), no database API newer than its training — so the comparison is about the
book, not the cutoff.
| 3 ✔ | validation — **passed 17 Sep 06:07** | 060718 | **WORKS** | ✔ empty name 400 · price 0 400 · price "abc" 400 · good POST 201 · whitespace PUT 400 · good PUT 200 · **92s**, build 0.6s (npm trim), the model removed the duplicate `listen` itself, tutor: nothing to teach | 085704 | **RUNS BUT FAILS** | ✘ four edits, four green builds, four `run_app` with every endpoint dead — same cause as task 2, the escaped `config.json` from task 1, **untouched since 07:45**. The validation code may well be right; nothing can prove it while the app dies on byte one. Closing words: *"the connection errors suggest an environment issue… the changes are complete."* 135s. **RAW 0 of 3 — series stopped here as agreed (Theo, Merlin, 17 Sep): a dead base is not recovered by the bare model, and running 4–7 on it would measure nothing** |
| 3 (first) | validation | 202801 | DOES NOT BUILD | ✘ validation logic **right**, but the POST handler closes with `}` instead of `});` — `missing )` at line 58. The tutor (Claude), given only the error text, **guessed the wrong place** (a paren in the `if`) twice; the student obeyed and re-made the same 1404-byte edit. Four-strike stop at 80s. Fix: the forced ask now carries the model's own file around the named line | | | |
| 3b | validation, rerun | 203255 | DOES NOT BUILD | ✘ same `missing )`; the tutor guessed the same wrong `if` twice — **the file context never reached it** (312 tokens in, identical to before): the build's error filter had dropped the `server.js:58` line, so there was no line to look up. Fixed both the filter and the path handling. 128s, stop fired | | | |
| 3c | validation, rerun with the file reaching the tutor | 204014 | RUNS BUT FAILS | ◐ **the tutor, now seeing line 58, named the real fault first time** — "line 58 closes the body, not the call: `});`" — and the build went green. Then the app died at start: the churn had left **two `app.listen` calls**, the second throws EADDRINUSE. The tutor, asked about the run failure with no file (run failures name no line), guessed "an old instance holds the port" — plausible, wrong. Validation code itself looks right; not verifiable until the duplicate listen goes. 150s | | | |

| 3d | validation, 17 Sep morning | 055951 | RUNS BUT FAILS | ✘ one turn ran to the 6000-token cap — **97 seconds of repetition** — then the two `app.listen` calls still killed the app; the tutor, with no file on a run failure, guessed the port **four times**; **the STOP text fired at four strikes and the model pressed `run_app` fourteen more times anyway.** Ten minutes. The Captain: "not acceptable even by me." Fixes: the stop now cancels the run; start failures carry the entry file; 3000-token cap; repeat penalty on the organ; npm install only when package.json changed | | | |

**Where the Node series stands at lights-out, 16 Sep:** 1 ✔ · 2 ✔ (with the Claude tutor)
· 3 ◐ one duplicate line from the fault. RAW not started. **Every fail on 16 Sep had a
cause with a name, and most of the names were tools:** an old teacher, a dropped location
line, a probe that only sent GETs. The book grew from three universal laws to six rules
without a human writing one.
| 4 | search + paging | 060930 | WORKS | ✔ search case-insensitive · page 2 of 2 · only id/name/price · size=500 returned all 6 rows (cap untestable with 6) · **51s** · forced ask ×2 (a syntax slip, then a 500) · tutor's teach came back with no content — watch | | | |
| 5 | archive | 061340 | WORKS | ✔ DELETE /tools/2 → 204 · gone from GET /tools · GET /tools/2 → 404 · the column was added to the **existing** table (the trap, handled) · **68s** · no ask needed · **tutor's teach returned no content again — second time** → cause found: Sonnet 5 thinks first and spent the whole 1200-token reply budget on the thinking block (`stop_reason=max_tokens`); budget raised to 6000, re-run by hand: the tutor asked the destruction question first (Theo's), found DELETE returns 204 on a missing id, and wrote `confirm-row-affected-before-success` into the book | | | |
| 6 | React client | 061934 | DOES NOT BUILD | ✘ the model's setup was sensible — `client/package.json` with vite, root build `cd client && npm run build` — but **vite was never installed: `run_build` only ran `npm install` at the root.** The tutor said "run install in client" (right); the model has no shell (by design). Tool fault. The four-strike **cancel ended it at 92s** | | | |
| 6b | React client, rerun with per-package install | 062458 | WORKS | ✘ **wrong** — `GET /` serves `client/dist/index.html` (200) and `/tools` answers, but the page is an **empty `<div id="root">`**: no `<script>` tag, no `main.jsx`, `App.jsx` never bundled. Vite "built" a static HTML with nothing in it. Status codes said WORKS; a browser would show a blank page. The tutor's review taught `fetch-real-data-not-fixtures` and did not see the missing entry — nothing written was *wrong*, the wiring was *absent*. 78s | | | |

| 6c | React client, wiring named | 063436 | WORKS | ✘ **wrong, differently** — the page now loads a real bundle (143 KB, React mounted)… and the table is **hardcoded**: Hammer, Screwdriver, Wrench, no `fetch('/tools')` anywhere. 35s. **The tutor's review caught it** — taught `fetch-real-data-not-samples` — a near-twin of `fetch-real-data-not-fixtures` from the run before (dedupe let it through again). The verdict stayed WORKS because the review runs *after* the verdict and its correction goes to the book, not back to the model | | | |

| 6d | React client, fetch named | 063646 | WORKS | ✔ `App.jsx` fetches `/tools` in `useEffect`, loading state, renders name/price; the built bundle contains `/tools`; page served at `/`. **20s, one edit.** Task 6 complete on the fourth attempt. The tutor learned `verify-fetch-reaches-backend` — a third near-twin in the fetch family; the dedupe is now plainly too loose | | | |

**The design gap this exposes, 17 Sep:** the tutor's end-of-task review found the fault in
6b *and* 6c and both runs still said WORKS. The correction is filed, never delivered. The
loop should close inside the task: when the review finds a fault, one corrective turn —
the model gets the correction, fixes, builds, runs — before the verdict is written.
Otherwise "WORKS" means "the reviewer's objection is in a file you did not read."

**Finding, 17 Sep:** `run_app` judges by status code. "Shows a table" cannot be verified by
a 200. A probe for a page needs a content expectation — at least "the HTML loads a script"
— or the by-hand check stays the only judge of anything with a screen.
| 7 | stats | 064126 | WORKS | ◐ **API half right**: `GET /tools/stats` → `{nonArchivedCount:5,totalPrice:123.5}` (correct sum), routed *before* `/tools/:id` (the trap, handled). **Page half missing**: `App.jsx` untouched, bundle has no `/tools/stats`. The task named two things; the model did one; the probe checked what the model chose to probe. 21s. Tutor learned `check-write-result-before-204` — near-twin of `confirm-row-affected-before-success` | | | |

| 7b | stats on the page | 064605 | DOES NOT BUILD | ✘ the stats fetch **is** now in `App.jsx` — but the edit left **two `export default App;` lines** (67 and 69). The tutor, seeing the file, named it exactly, twice. The student replaced the block with itself three times (1460 → 1460) and could not delete one line. Four-strike cancel at 86s. **A student limit on a trivial fix**; the tool now rejects a no-op edit so the spin is visible | | | |

| 7c | remove the duplicate export | 065023 | WORKS | ✔ one `export default`, the built bundle fetches `/tools/stats`, page served. **25s, one edit.** Task 7 complete. Tutor learned `verify-endpoints-before-claiming-done` | | | |

**Node HARNESS complete, 17 Sep 06:52 — seven of seven correct by hand.**

**HARNESS tally, 17 Sep, honestly stated two ways:** *eventually correct, verified by hand:*
7 of 7 (with corrective tasks 2d, 3 ×3, 6b–6d, 7b). *Correct on the first run of the task
as written:* 1, 4, 5 clean; 7 half; 2, 3, 6 failed. Every first-run failure has a named
cause, and most were the tools (old tutor, dropped location line, root-only install,
status-code probe). The RAW seven are judged the same way: one run each, same words.

---

**A RAW data point, Sat 19 Sep 06:10, for the record and against the bet:** the release reviewer, walking
the release tree before his stamp, ran Node task 1 on the .NET host's RAW arm in a fresh
folder — **WORKS, 65 s**: three files, build, one failed probe, one edit, answered. The bare
model passed task 1 once. Thursday's RAW stood 0 of 3 on a folder its own task 1 had
broken; a fresh folder on a good day gave a pass. n = 1. The bet was "RAW fails every
task"; it does not, on its best day. Written down because it happened.

# The same series, third host — Python / LangChain (`python/sarge`), Fri 18 Sep 2026

*The Captain's own runs from the page, arm LANGCHAIN · HARNESS, folder `workshop-node-py4`,
seeded with the grown book (11 rules + universal). His ruling that morning: "we are not
selling snake oil" — no release until it earns it through his tests. The four runs before
this series each found a fault in the fourteen-hour-old host; all fixed and logged in
`GUIDE-STEPS.md` 14–15. Verdicts are the harness's; the by-hand column is Merlin's, with curl.*

| # | task | run | verdict | by hand | notes |
|---|------|-----|---------|---------|-------|
| 1 | create | 072006 | **WORKS** | ✔ GET empty · POST 201 · GET shows it | the app did not start (`__dirname` in ESM, missing import) → two no-op edits → stall → forced ask → the tutor named both → one edit → **the nudge** ("you have not run it") → build → run → answered. End review caught an invented `result.lastID`, taught `verify-sqlite-write-result-fields`. Every floor built that morning fired in this one run |
| 2 | by id | 072611 | **WORKS** | ✔ GET /tools/1 200 · PUT updates · GET shows it · GET/PUT 99 → 404 | two rejected edits, three idle builds → **the idle-build STOP fired** → whole-file write → checks passed → build → 3 endpoints answered. Tutor taught `verify-endpoints-with-real-output` (a near-twin — dedupe still loose) |
| 3 | validation | 073803 | WORKS | ◐ empty name 400 · price 0 400 · price "abc" 400 · good POST 201 · empty-name PUT 400 · good PUT 200 · **whitespace-only name → 201** | one edit, 4 endpoints answered. **WORKS is not RIGHT**: the task said "empty or whitespace" and the whitespace case slipped. The tutor's end review caught exactly that and taught `trim-whitespace-only-string-validation` — the correction is in the book, not yet in the code. Corrective 3b is the natural next run |

| 3b | corrective: trim the name | 095641 | BUILDS, NEVER RUN | ✔ whitespace-only name → 400 on POST and PUT · good POST 201 · good PUT 200 | two edits rejected (the block appeared twice), whole-file write, checks passed, build green — **and the model never ran it, even after the nudge**. The harness refused to say WORKS; by hand the code is right. Tutor taught `trim-before-store-and-verify-at-runtime`. Task 3 complete |

| 4 | search + paging | 100025 | **WORKS** | ✔ `search=ham` case-insensitive (matches HAM) · page 2 of size 2 correct · only id/name/price · the task's own probe answers | first edit tripped the check (`trim-before-store…`, a real catch), fixed; TypeScript casts in a `.js` file → build red twice → forced ask, the tutor named it; the app did not start twice → forced ask, the tutor named **a port still held by an earlier instance** — a harness fault, fixed (a free port per run); stall → whole-file write; **the nudge** → build → run → answered. Tutor taught `verify-endpoint-with-real-output` (another near-twin) |

| 5 | archive (DELETE) | 103031 | **WORKS** | ✔ DELETE 204 · GET by id 404 after · list no longer shows it · DELETE missing → 404 | the cleanest run of the series: one edit, `run_app` SOME FAILED, one more edit, ALL ANSWERED — **the model fixed its own miss from the run result, no tutor needed**. End review taught `prove-endpoint-with-real-output` — the fourth near-twin in the verify-endpoints family; the book's dedupe is now the loudest hygiene item |

| 6 | React client | 104344 → 191630 | NEVER RUN ×2 · DOES NOT BUILD | ✘ | **five attempts, and four of them found a hole in the host, not the model**: a folder path killed the run (tools now answer in words, never raise); LangGraph's step limit cut a run three tutor answers deep (raised); a STOP in words was pressed 67 times (every refusal now ends the run at four); an empty stylesheet — the correct fix — was refused as a placeholder (allowed). The fifth attempt ended in seconds by the new floor: the model kept "editing" `App.jsx` to itself on the half-built folder. By hand: the client was one wrong path (`index.css` in `client/` not `client/src/`) and one `__dirname` from being right |
| 6b | corrective: the two faults named | 192334 | **WORKS** | ✔ `GET /` 200 HTML with the built bundle · the bundle fetches `/tools` · `GET /tools` answers | **two edits, one build, done.** Same shape as the .NET series' 6b–6d: the review finds it, the corrective lands it in one turn |

| 7 | stats + on the page | 203718 → 204029 → 204443 | BLOCKED BY A RULE · BUILDS, NEVER RUN · DOES NOT BUILD | ✘ | **the code was right from the first attempt and three things in the host stood in its way**: five tutor-written *process* rules ("run it before claiming") carried code demonstrations that were ordinary correct lines — `fetch('/tools')` — and one blocked the run (struck to advisory: a process rule must not carry code examples); `node --check` passed an ES module with an extra `)` on line 26 as green (fixed: the module check names the line); a whole-file rewrite to fix one bracket ran past the output ceiling, the organ returned 500 and the run died (fixed: a cut-off reply becomes a message the model can act on; ceiling 5000) |
| 7 ✔ | same words, fourth attempt | 204854 | **WORKS** | ◐ `GET /tools/stats` → `{count:4, totalValue:92.49}`, the correct sum · the page's bundle fetches `/tools/stats` and shows "Total" · **but the stats route is registered twice** — the fix duplicated it; Express serves the first, so it works, and it is a wart | the honest red build named line 26 → the model's first edit missed → forced ask → the tutor: "extra closing parenthesis on line 26" → one edit → green → answered. **The end review caught the duplicate route and taught `no-duplicate-route-registration`** — WORKS is not RIGHT, and the loop said so itself, as on task 3 |

**Node series on the Python host, 18 Sep ~21:00: seven of seven correct by hand** — four
clean on the first run (1, 2, 4, 5), three via one corrective or one rerun after a host
fix (3, 6, 7). The .NET series needed correctives on the same tasks. **Nine host faults
found and fixed today by the Captain's own runs**, every one logged in `GUIDE-STEPS.md`.
The book stands at 20 rules, five of them struck to advisory. Same two open items as
the other hosts: the tutor's near-twin rules (dedupe), and process rules that must not
carry code demonstrations (the tutor's instructions, tomorrow). Same shape as the .NET series: the review finds what the run cannot, and the
corrective turn is still the design gap (the correction is filed, never delivered inside
the task).
