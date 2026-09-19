# Steps log — Sarge on Microsoft Agent Framework, for the dev guide

*Started Thu 17 Sep 2026 ~18:40 on the Captain's GO: "build the bridge and we will start
testing, finish testing today and release MVP tomorrow, focus on .NET, myshop; record all the
steps." Every step below is written as it happens, in the order a developer would repeat it.
The guide is written FROM this file, not from memory.*

## 0. What was true before the first step

- `agent/Sarge.Agent/Program.cs` is **already** a Microsoft Agent Framework agent
  (`Microsoft.Agents.AI` 1.20.0): `chatClient.AsAIAgent(...)`, tools via
  `AIFunctionFactory.Create`, `agent.RunAsync(prompt, session)`. The organ is its
  `IChatClient` through the OpenAI client pointed at `http://127.0.0.1:8421/v1`.
- So the bridge is an **extraction**, not a rewrite: the loop lives in one 1200-line
  `Program.cs` of top-level locals and closures that no one can `dotnet add package`.
  The work is to lift it into a library a developer can reference, and leave `Sarge.Agent`
  as a thin host so the page keeps working unchanged.
- Picasso (`repos\picasso`) is on the same `Microsoft.Agents.AI`
  1.20.0, targets net8/9/10, and its `SqliteChatHistoryProvider` already overrides
  `InvokingCoreAsync` — the hook the rules go through.
- SDK on this machine: .NET 10.0.102. VPN down. Organ up on :8421 with Qwen3-4B.

## Steps

### 1. The library — `agent/Sarge.AgentFramework` (~18:45–19:15)

New class library, `Sarge.AgentFramework`, PackageId the same, MIT, net10.0 tonight (multi-target
8/9/10 like Picasso is a one-line change for the release). Same four package references
`Sarge.Agent` already had. Files, each a lift from `Program.cs` with its dated comments kept
where they carried the reason:

| file | what |
|---|---|
| `SargeOptions.cs` | `RepoDir` (required), `SargeHome` (default: `SARGE_HOME` env, else walk up from the app to a folder with `rust\` + README), `BookPath` (default `<repo>\.sarge`), `OrganUrl` (`http://127.0.0.1:8421/v1`), `Harness`/`PlanFirst`/`ReviewPath`, `Delivery` (ContextProvider or PromptTail), `MaxOutputTokens` 3000, `OutboundBlocked` (default: the PANGP check) |
| `Handshake.cs` | the Rust binary, unchanged contract: exit 1 = hit, anything else = our failure, never "you broke a rule" |
| `SargeRun.cs` | the harness: the seven tools, every guard, the stall signal, the forced ask, the four-strike cancellation, the check after every write, `Review()` at the end. Public state for the host: `Verdict`, `Log`, `ToolCalls`, `Builds`, `Written`, `StopToken` |
| `SargeRulesProvider.cs` | an `AIContextProvider`: re-attaches the framed `.sarge` block as a user message on every invocation, counts `RulesHeld` |
| `RecoveringChatClient.cs` | text-embedded tool calls → real `FunctionCallContent`; `SARGE_TRACE=1` prints every message sent to the model, role by role |
| `SargeAgent.cs` | `CreateOrganClient(options, run)` and `Create(options, run, chatClient?, history?)` → `(AIAgent, prompt, rules)`; the character at the head, the rules at the tail, `AIContextProviders = [rules]`, `ChatHistoryProvider = history` (Picasso's, or none) |

**One thing lost in the move, said plainly:** the old `Program.cs` carried four days of
long comments naming each failure by date. The guards moved intact; the comments were
shortened. The repo has never been committed, so the old text survives only in the run
logs and this session's transcript.

### 2. The host — `agent/Sarge.Agent/Program.cs` rewritten thin

Same command line the page uses (`--repo --book --task --harness --max-turns`), plus
`--tail` to switch delivery to PromptTail. Parses args, builds `SargeOptions` + `SargeRun`,
`SargeAgent.Create`, `RunAsync` with the stop token, `Review()`, prints the same lines,
writes the same run log and ledger row (one new field, `rules_held`). `--max-turns` is
accepted and ignored: the framework owns the loop. The csproj now references the library
instead of the packages directly.

### 3. Build and the first smoke — where does the framework put the rules?

```
cd agent\Sarge.Agent
dotnet build
```

First build: one error, `Home` was internal and the host needs it — made public. Second
build clean. Then a plan-mode run (list and read only, no writes) with `SARGE_TRACE=1`, on
`repos\myshop` with its 12-rule book. What the model was sent, role by role:

```
user      Add GET /tools/stats returning the number of tools …      ← the task
user      ## RULES FOR THIS TASK …                                    ← the .sarge block, from the provider
assistant                                                            ← tool call
tool                                                                 ← tool result
```

**The context provider's message lands AFTER the task — the tail — on every invocation,
and the run reported `rules held 12`.** That is the measured position (KV-POINTING, Run 2)
delivered by the framework itself, with no prompt surgery. `Delivery = ContextProvider`
stays the default; `--tail` (PromptTail) remains as the control arm.

### 4. The first real task through the bridge — .NET task 7 on myshop, HARNESS

```
dotnet run --no-build -- --repo ..\..\..\myshop --book ..\..\..\myshop\.sarge --harness ^
  --task "Add GET /tools/stats returning the number of tools that are not archived and the total of their prices as a decimal. Build it, run it, and show it answering."
```

Run `agent-20260917-181138-HARNESS`, 94 s, 15 tool calls, 2 builds, 12 rules held. **The
loop itself worked end to end**: list, read, edit, check after the edit, build, run refused
while a rule stood, the model argued, the four-strike guard held it. **The task was blocked
by the check, wrongly.** The hit was `Program.cs:156`, the startup line that resolves the
context inside `app.Services.CreateScope()` to create the database — a line that was there
before the task, and that the rule's own `allow` clause names word for word. The model said
so in prose, five times, and was right. The check offered the organ only WRONG, RIGHT or
NEITHER, so `allow` was a line it could read and not a word it could answer with.

**Fix, in the check not the harness** (`rust/src/verdict.rs`, `confirm`): when a rule has
an `allow` clause, the in-context question offers **ALLOWED** as a fourth answer, and
ALLOWED is not a hit. This was the "known check miss" on the pending list since the
morning; the bridge's first run is what made it bite.

```
cd rust
cargo build --release --bin handshake      (CARGO_BUILD_JOBS=4)
replay-check.cmd                            → 6 of 7 caught, 0 false flags, unchanged
handshake --check ..\myshop --file Program.cs --repo WorkshopTools --rules ..\myshop\.sarge
                                            → CHECKS PASSED on the startup line
```

### 5. Task 7 again, on the file restored to its pre-task state — WORKS

Run `agent-20260917-181526-HARNESS`: **29 s, 5 tool calls, one edit, one build, CHECKS
PASSED, 12 rules held, WORKS.** The tutor's end-of-task review (Claude, one call) taught a
new rule, `no-auto-drop-database-on-startup` — it found the `EnsureDeleted()` in Development
that task 5e had left behind, and noted the stats aggregation should happen in SQL. By hand:
`GET /tools/stats` → `{0, 0}` on an empty store; after three POSTs and one DELETE (archive)
→ `{"activeToolCount":2,"totalPrice":21.75}`. Correct.

(An in-between run, `181413`, ran while the file still held the first attempt's endpoint:
the model read it, said "already exists, no changes needed", built, and **did not call
`run_app`** — verdict BUILDS, NEVER RUN. The harness cannot force a run the model believes
is unnecessary; noted, not fixed tonight.)

### 6. Picasso underneath — the store the dashboard reads

Host gains `--picasso <db>`. It is the developer's one-liner, nothing more:

```csharp
history = new SqliteChatHistoryProvider(picasso) { ConversationIdSelector = () => conversationId };
SargeAgent.Create(options, run, history: history);
```

`Sarge.Agent.csproj` references the published package `Picasso.AgentFramework.Persistence`
0.1.6 (from the local NuGet cache; no network needed). The published 0.1.6 mints its own thread id (`thread_xxxxx`); the
`ConversationIdSelector` in the Picasso source on this machine is not released yet, so the
host does not name the thread — first build error of the evening was exactly that, fixed by
using the released API.

Plan-mode run with `--picasso runs\picasso.db`, then the store inspected:

```
messages by role: assistant 3 · tool 2 · user 2      thread_8561a      compactions 0
seq 0  user       Add GET /tools/stats …                 ← the task
seq 1  user       ## RULES FOR THIS TASK …               ← the rules block, from the provider
seq 2  assistant                                         ← tool call (list_files)
seq 3  tool
seq 4  assistant                                         ← tool call (read_file)
seq 5  tool
seq 6  assistant  1. Read Program.cs: Found existing …    ← the plan
```

**Works: the whole turn is in the store, in order, roles intact.** One finding for
Picasso's next release, not for tonight: the framework hands the provider's rules message
to the history provider as a request message, and Picasso stores it (seq 1). In a long
multi-turn agent the block would be stored once per turn and re-loaded as history — the
rules still survive compaction (a fresh copy is re-attached from disk every turn), but the
history bloats. The fix is a one-line source filter in Picasso's `StoreChatHistoryAsync`
(skip `AgentRequestMessageSourceType.AIContextProvider`), and then the dashboard can show
*rules held* from the same stamp. Picasso repo, tomorrow.

### 7. The Captain's first Run through the bridge — task 8, from the page

The page now passes `--picasso runs\picasso.db` on every run (console restarted). Task 8,
the corrective the tutor's review asked for: *"use EnsureCreated only, never
EnsureDeleted; make GET /tools/stats compute the count and the total in the database with
CountAsync and SumAsync."* Run `agent-20260917-184109-HARNESS`, **31 s, 6 tool calls, 13
rules held, RUNS BUT FAILS.**

- The model did exactly what was asked: the drop is gone, the stats use `CountAsync` and
  `SumAsync`. Two edits; the first tripped the check, the second passed. One build, green.
- `run_app` failed on **a trap in the task itself**: SQLite has no decimal type, and EF Core
  cannot translate `Sum` over a `decimal` column — *"cannot apply aggregate operator 'Sum'
  on expressions of type 'decimal'"*. My task text asked for the impossible.
- The model then did the right thing — it reached for the tutor — **and wrote the call as
  text**: `ask_tutor("Why does SQLite throw…")` as prose, not a tool call. The recovery
  layer that turns text-shaped calls into real ones only knew four tools (the original
  set from three days ago). The question was never asked and the run ended on it.
- The tutor's end review still ran and taught `select-fields-before-materializing`.

**Fix:** `KnownTools` now names all seven, so a text-shaped `edit_file`, `run_app` or
`ask_tutor` is recovered like the others. Rebuilt. The task is rerun as written, so the
trap stays: the point is whether the loop now asks and the tutor answers.

---

## Direction change, ~19:00 — MAF parked, LangChain is the host that matters

The Captain: "the MAF market is overly niche for us"; then ".NET parked for the moment;
build an agent based on LangChain and bridge Sarge with that — the new moat"; then
**Python**. Everything above stands as built; the .NET host is one client, not the front
door. Steps from here are the Python bridge. Design: `docs/BRIDGE-LANGCHAIN.md`.

### 8. The venv and the framework

```
cd <sarge clone>
python -m venv python\.venv
python\.venv\Scripts\python -m pip install langchain langgraph langchain-openai
```

Python 3.11.9 on this machine; installed langchain 1.4.1, langgraph 1.2.11,
langchain-openai 1.6.2. VPN was down (the one outbound step). LangChain 1.x's agent is
`langchain.agents.create_agent(model, tools, system_prompt=, middleware=)`; middleware is
`@wrap_model_call(request, handler)` with `request.override(messages=…)`, and
`@before_model(can_jump_to=["end"])` returning `{"jump_to": "end"}` — checked from the
installed source, not from memory.

### 9. The package — `python/sarge`

| file | what |
|---|---|
| `pyproject.toml` | `sarge` 0.1.0, MIT, `langchain>=1 langgraph>=1 langchain-openai>=1`, console script `sarge` |
| `options.py` | `SargeOptions`: `repo_dir`, `sarge_home` (SARGE_HOME or walk up from the package), `book_path` (default `<repo>\.sarge`), `organ_url`, `harness`, `plan_first`, `max_output_tokens` 3000, `outbound_blocked` (PANGP) |
| `handshake.py` | the binary, same arguments, same exit-code contract |
| `run.py` | the harness brain, ported from `SargeRun.cs` line for line (**second copy** — the Rust binary is its right home, later) |
| `agent.py` | `class Sarge`: `llm()` = `ChatOpenAI(base_url=organ, api_key="local", timeout=600)`; `tools()` = seven `@tool` closures; `rules()` = middleware appending the framed block as the **last** message on **every** model call; `stop()` = middleware ending the graph when cancelled; `character()`; `agent()` = `create_agent(...)` |
| `__main__.py` | `python -m sarge --repo --task --harness --book …` — the same command line as the .NET host; run log + ledger row with `host: langchain` |

```
python\.venv\Scripts\python -m pip install -e python
```

### 10. First smoke — plan mode on a fresh Node folder

`repos\workshop-node-py`, seeded with `workshop-node`'s book (10 rules + 3 universal).
`python -m sarge --repo … --harness --plan-first`: 6 s, **13 rules held, 3 deliveries** —
one per model call, which is the whole point of the middleware — and the plan cites
`use-config-for-connection`, `sync-db-no-await`, `check-write-result-before-204` by id.

### 11. Node task 1 through the Python host, HARNESS — first attempt: a runaway, killed

`repos\workshop-node-py`, the task as written in `SERIES-NODE.md`. **Killed after ten
minutes** — over the Captain's own line. What the organ's log showed: after the first
write and its two check calls, the prompt grew by **exactly the same 918 tokens on each of
26 model calls**, with no check call between them. A fixed-size tool refusal, pressed
again and again. The folder held a `server.js` importing `Router` but never `express`, and
a `config.json` of `{}`.

**What I suspected first, and tested before touching anything:** that the rules
middleware was leaking its block into the conversation state, so every call carried one
more copy. `SARGE_TRACE=1` on a plan-mode run printed the state the framework hands the
middleware: **1 message, then 3** — the task, the tool call, the tool result. The block is
transient; `request.override` is immutable and the framework stores only the model's
output. The organ confirmed it: 2641 → 2682 tokens between calls. **Not the middleware.**

**Three floors added, none of them new ideas:**

- `run_app` before a green build is counted; the third in a row returns STOP and names
  `run_build`. (The .NET host has the idle-build floor; this is its twin.)
- `ModelCallLimitMiddleware(run_limit=40, exit_behavior="end")` — a hard ceiling under the
  four-strike stop. The graph ends; it does not raise.
- The run log is written through on every line (`_Log` with a sink). The killed run left
  **no log at all**, because the log was written at the end. Evidence that exists only
  after a clean finish is not evidence.

Also noted, not fixed tonight: `edit_file` has no placeholder guard, so a whole-file edit
to `{}` goes through where `write_file` would have refused it.

### 12. Node task 1 again, fresh folder `repos\workshop-node-py2` — RUNS BUT FAILS, honestly

Run `agent-20260917-211457-HARNESS`: **157 s, 16 tool calls, 1 build, 13 rules held on 17
deliveries** (one per model call — the middleware working). The loop, in order:

1. list, write `package.json`, `server.js`, `config.json` — **each checked on the write**, all passed
2. `run_build` green (npm install, syntax check)
3. `run_app` → **THE APP DID NOT START** (`ERR_IMPORT_ATTRIBUTE_MISSING`: a static JSON import without the attribute)
4. **seven identical no-op edits rejected** in a row — 100 seconds of the model replacing a block with itself
5. **the student asked on its own**: quoted the exact error; the tutor (Claude) answered correctly — read the file with fs instead
6. one real edit, checks passed — **and then the model declared done without running again.** "The app now starts correctly" was claimed, not tested. Verdict stays on the last real run: RUNS BUT FAILS.
7. the tutor's end review named all four remaining faults (an extra `}` in `config.json`, `__dirname` in an ES module, `require('express')`, no `express.json()`) and taught **`no-commonjs-in-esm`** into the book. One call, three cents.

**What this says about the host:** the Python host ran the whole loop — write, check, build,
run, refusal, the student's own ask, the tutor's answer, the end review, the new rule. Same
loop as the .NET host, same model, same book. **What it says about the model:** the same
two failure shapes as every day this week — the no-op edit spin and "done" without a run.

**Fix from it (Python host):** an identical no-op edit now counts as a rejection like any
other, so the second one forces the ask instead of the seventh being the student's own.
The .NET host has the same gap; same fix tomorrow. **Not fixed:** "done without a run" —
the harness cannot force a run the model believes unnecessary; the verdict already
refuses to believe it.

**Comparison to the .NET host on the same task, for the record:** the first-ever run of
task 1 through `Sarge.Agent` was clean. Different day, same model, one run each — a
signal about the model's variance, not about the hosts.

### 13. Fri 18 Sep — the Claude Code hook (`hooks/`)

The Captain's ruling for v0.1.0: LangChain and Claude Code, "enough market coverage";
CrewAI and Cursor next, not claimed. One Python script, two hook events:

- `PostToolUse` on `Write|Edit|MultiEdit` → `handshake --check` on the written file; a
  hit is printed on stderr with exit 2 (Claude sees it as the tool's feedback) and left
  in `<repo>/.sarge-hit`.
- `PreToolUse` on `Bash` → while `.sarge-hit` stands, a run command (`dotnet run`,
  `npm start`, `npm run`, `node`, `python`, `uvicorn`, `flask run`) is refused with exit
  2 and the hit. A passing write clears it.

Tested by hand against the judge fixtures in a scratch repo seeded with
`workshop-node`'s book: bad write → 2 · run → refused · `git status` → allowed · clean
write → 0, hit cleared · run → allowed. Five of five. Claude Code itself is the tutor in
this host: ask it to write the rule, with `wrong |` and `right |` lines.

### 14. Fri 18 Sep — the Captain's first run from the page, and the check's window

Node task 2 on `repos\workshop-node-py2`, HARNESS, 275 s, 14 rules held on 14 deliveries.
The model wrote GET and PUT by id — the shape right, existence check, `changes === 0` →
404 — and **the check blocked it, wrongly**: `check-write-result-before-204` hit the UPDATE
line while the `changes` check sat three lines below, one line outside the window the
second look reads (two above, one below). The model said it was compliant five times, in
five identical no-op edits, and was right. Run refused; verdict printed DOES NOT BUILD,
which was also wrong — the build was green.

Three fixes, each one line of design:

- `verdict.rs`: the window is now **two above, five below** — a write is followed by what
  checks it. Replay after: **7 of 7 known faults caught** (the `app.Services` miss that
  had stood since Wednesday is caught by the same widening), **0 false flags**, and the
  Captain's file passes.
- `run.py`: **a rule hit is a failure** — it sets `last_failure`, so the no-op stall and
  the forced ask can fire on a hit, and the hit ids go in the log line.
- `run.py`: verdict **BLOCKED BY A RULE** when a hit stands after a green build, instead
  of DOES NOT BUILD.

### 15. Fri 18 Sep, 10:20–10:45 — four runs, four floors, then the pass

The Captain ran task 1 from the page four times this morning, and each failure named a
hole in the fourteen-hour-old host. In order: a rule hit did not count as a failure, so
the stall floor could not fire on it (fixed: a hit is a failure); the forced-ask budget was
two per task and was spent before the fatal fault (fixed: two per distinct fault); the
model wrote JSON wrapped in parentheses and, another time, with escaped quotes (fixed:
normalised as a format, like CRLF); "done" without a run after a failed run (fixed: the
nudge fires whenever the last run was not green). One false flag from a client-side fetch
rule hitting a server route: fixed in the language with an `allow` line, not in code.

**Run `072006` on a clean folder with the grown 11-rule book: WORKS.** Writes checked;
identical rewrites rejected; the app did not start; two no-op edits → the stall → the
forced ask → the tutor named `__dirname` in an ES module and the missing `express` import;
one real edit; the nudge ("you have not run it"); build; run; `GET /tools` answered. By
hand: empty list, POST, the list shows it. The end review caught the model inventing
`result.lastID` and wrote `verify-sqlite-write-result-fields` into the book.

### 16. Fri 18 Sep, afternoon and evening — tasks 2 to 7 from the page, and the last five floors

The Captain ran the rest of the series himself. Tasks 2, 4 and 5 passed clean; 3 needed
one corrective (a whitespace-only name); 6 and 7 found, between them, five more holes in
the host:

- **A folder path killed the run.** `read_file("client/")` raised inside the tool and
  LangChain ended the graph. Now every tool answers in words, never with an exception.
- **LangGraph's step ceiling.** `recursion_limit` counts every node, middleware included;
  120 was about thirty model calls and cut a run three tutor answers deep. Now ten times
  the model-call cap (60).
- **A STOP in words is advice.** The idle-build refusal was pressed sixty-seven times in
  eight minutes; later the identical no-op edit twenty-five times. Every refusal of any
  kind now ends the run after four identical repeats (`_refused`), the same law as the
  four-strike failure stop. And the idle-build message now says what to do *next*: if the
  code is complete, call `run_app`.
- **`node --check` is blind to ES modules on Node 24.** An extra `)` on line 26 was
  "green" through two builds and only died at start. The build now feeds the source
  through `node --input-type=module --check` when the file opens a line with `import` or
  `export`, and names the line.
- **A cut-off reply killed the run.** A whole-file rewrite ran past the 3000-token output
  ceiling, llama-server answered 500 "failed to parse tool call arguments", LangChain
  raised. Now a `recover` middleware turns that into a message the model can act on
  ("your reply was cut off; use `edit_file`"), and the ceiling is 5000.

Also: an empty stylesheet is a real file (the placeholder guard refused the correct fix
five times); a run that ends by refusal with a green build and no run gets the one nudge
turn.

**And the finding about the book:** five of the tutor's rules were the same lesson — "run
it before you claim it" — in five coats, each with a `wrong |` line that was ordinary
correct code (`fetch('/tools')`, `app.listen`, a route, a comment). The judge takes the
demonstration over the `allow` clause, so any normal file eventually tripped one. Struck
to advisory, kept in the book with a `since` note each. **A rule about process must not
carry code examples; only a rule about a line may.** That goes into the tutor's
instructions next.

**Node series on the Python host: seven of seven correct by hand.** Two warts the loop's
own review named (task 3's whitespace, task 7's duplicate route) — WORKS is not RIGHT,
and the loop said so itself.

**Not yet run, and the README claims nothing until it is:** the two-arm compaction test
from `BRIDGE-MAF.md` (rules in the first message vs. rules via the provider, one
conversation long enough to force the reducer). 0.1.6 has no `ChatReducer` property — the
source does — so the test runs against Picasso's next build or with the framework's own
in-memory provider. Tomorrow morning, before the README.
