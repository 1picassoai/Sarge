# The Sarge syntax — a rule language the model reads as an order

**Name, ruled 17 Sep:** **the language is called Sarge.** "Rules are written in Sarge";
files are `.sarge`. One name for the language and the engine, the way SQL is both the
language and what every engine speaks. (The working name for a day was AIML — flagged
within the minute as the 2001 XML chatbot format, carried to the Captain by Fox, binned.
Same trap as CompilerGPT; the Table caught it before it went outward.)

**Built 16 Sep, while he napped:** `lang.rs` parses and writes it; `load()` reads either
the language or the old JSONL by looking at the file; `render()` delivers the block itself
to the model; the tutor writes rules in it; the book grows as blocks inside the frame;
`book/universal.sarge` is loaded under every repo's `.sarge` by the binary. Baking as K/V
in the organ is the experiment below, not yet built.

*16 Sep 2026. The Captain's idea ("a very particular syntax, not JSON, baked as KV");
Merlin's forging. Design, with an experiment behind it before anything is claimed.*

## What it is for

Three jobs, and every mark in the syntax earns its place against one of them:

1. **Frame.** When the block is cached as K/V in the organ, its markers are what tell the
   model "this is an order", not "some words". The bare-text cache lost for want of a
   frame (`KV-POINTING.md`). The syntax *is* the frame.
2. **Demonstrate.** A next-token predictor follows examples better than descriptions. Every
   rule carries the wrong line and the right line, as code. The model continues the right
   one.
3. **Judge.** The check asks the organ whether the file it just wrote has the shape of a
   `wrong` line. No regex. The examples are the question.

## The syntax

**A universal language, like SQL** — the Captain's ruling, 09:45: no repo name in the
syntax. Seven keywords, the same in every repo and every programming language: `rule do
never wrong right allow since end`. What a file applies to is where it sits: a `.sarge`
at a repo's root applies to that repo; `book/universal.sarge` applies everywhere. The code
between the `|` bars is whatever the repo is written in.

Line-based, no braces, no quotes, no escaping; the model has never seen these markers
around anything but rules.

```
=== sarge ===

rule scoped-context-in-handler
  do     take the DbContext as a handler parameter
  never  create or resolve the DbContext inside a handler
  wrong  | using var db = new ToolContext(options);
  wrong  | var db = app.Services.GetRequiredService<ToolContext>();
  right  | app.MapGet("/tools", async (ToolContext db) => await db.Tools.ToListAsync());
  since  2026-09-15 tutor
end

rule no-addcontrollers-in-minimal-api
  do     map endpoints with app.MapGet, app.MapPost, app.MapPut, app.MapDelete
  never  call AddControllers or MapControllers in a minimal API
  wrong  | builder.Services.AddControllers();
  right  | app.MapGet("/tools", async (ToolContext db) => ...);
  allow  a project that uses MVC controllers
  since  2026-09-16 tutor
end

=== end sarge ===
```

Every line, and why:

| line | why it exists |
|---|---|
| `=== sarge ===` … `=== end sarge ===` | the frame. Opens and closes the cached block. No scope in the syntax — location is scope. |
| `rule <id>` … `end` | one order. The id is what the tutor, the check and the log call it. |
| `do` | **first**, always — positive instructions hold across a session; prohibitions decay. The order leads with what to do. |
| `never` | the prohibition, one sentence, after the `do`. |
| `wrong \|` | a real line of code that breaks the rule. One or more. What the check looks for; what the model must not continue. |
| `right \|` | a real line that obeys it. What the model continues. |
| `allow` | the exception, if there is one. Optional. |
| `since` | date and who wrote it — `tutor`, or a name. The book's own history. |

Rules of the syntax itself: two-space indent inside a rule; `|` separates a keyword from
code so the code is never parsed; keywords are the only English on the left. Nothing
else. If a rule needs more than this, the rule is too big.

## How it loads — "baked as KV"

The handshake reads the repo's `.sarge` and `universal.sarge`, selects the block for the
scope, and the organ **prefills it once as its own sequence** — K/V computed, kept. Per
request the block's K/V is attached to the slot with a position shift (the `seq_cp` /
`seq_add` mechanics already built in `point.rs`). Zero re-tokenising, zero recompute. When
the book changes, the block is re-prefilled — cheap, once.

Two places it can sit, and the experiment decides: at the **root** beside the core, or
**shifted to the tail** right before the assistant turn. The tail is where text won.

## The experiment — RUN, 17 Sep 07:56 and 07:59

Six arms, twice — once with the rule as written in Sarge (with `wrong`/`right` code), once
with the same rule stripped to its sentence. Full tables in `KV-POINTING.md`, Result 2.
**With code demonstrations the rule held from every position, including three cache
attachments and the root. As a sentence, every cache arm broke and only the tail held.**
The frame (`kvs`) did not rescue the sentence. **The demonstration is what binds.** The
syntax earned its place — for the reason its `wrong |` and `right |` lines exist, not for
the frame. n = 1 per arm; repeat with a second rule before it is claimed outward.

## The experiment — as designed, before it ran

Same rule, same task, same model, greedy, one run per arm, as before:

```
none        no rule
tail-text   the rule as a sentence at the tail of the task     (today's winner)
kv-bare     the rule as bare text, cached                       (lost on 15 Sep)
kv-sarge    the rule in this syntax, cached, at the root
kv-sarge-t  the rule in this syntax, cached, shifted to the tail
```

**The syntax earned its place if** `kv-sarge` or `kv-sarge-t` holds in both handlers where
`kv-bare` held in none. If neither does, the frame was not the missing weight, the syntax
stays as the book's format (jobs 2 and 3 still stand), and the cache tier waits for the
mask. Either way we know.

## What it replaces, and how

`rules.jsonl` becomes `.sarge`. The tutor writes `.sarge` blocks instead of JSON (its
prompt changes; nothing else in it does). `--check` uses `wrong` lines as its question to
the organ (`CHECK-DESIGN.md`). The BOM that hid rule 1 for days cannot happen: a block that
does not open with `=== sarge` is not a block, and the loader says so out loud.

## Open, for the Captain

1. **Root or tail** for the cached block — or both, decided by the experiment. My bet: tail.
2. ~~One `.sarge` per repo at its root, or in `book/`~~ — **RULED, 17 Sep, against both
   bets:** the file stays at the repo root **and is git-ignored.** The Captain: *"no dev
   would trust or would let this be checked into the main branch… so it can stay git
   ignored."* It is `.env` — the dev gets the rules on their machine, the team never sees a
   machine-written file in a branch, and "where a file sits is its scope" still holds. The
   page adds `.sarge` to the repo's `.gitignore` when it first creates the book. (Theo
   carried the ruling to the Table and corrected himself within two minutes — the Table
   working.)
3. **Does `do` lead, or `never`?** The paper says `do`. The Captain may want the order to
   read like an order. Measured, not picked — it is one arm.
