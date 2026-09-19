# The bridge — Sarge inside Picasso for Microsoft Agent Framework

*Design, 17 Sep 2026 ~18:00. The Captain's change of direction: "we have some traction on
Picasso.AgentFramework.Persistence so we should keep building a bridge between this and
Sarge." Gareth's caveat carried in: Picasso's numbers say nothing about Sarge; Sarge has no
baseline until it ships, and no figure from Picasso's board goes near this README.*

## The one sentence

**Picasso shows you the moment compaction threw your instructions away. Sarge is the part
that survives it.**

## Why these two fit without force

Picasso's whole reason to exist is a single event: the context fills, Agent Framework
rewrites the working set, and the first user message — the one that said what the rules
were — is the first casualty. Picasso records it and shows it. It does not stop it; the
README says so.

Sarge's proven result (`KV-POINTING.md`, Run 2, twelve of twelve handlers) is that a rule
delivered **as text at the tail of the turn, every turn, from a file** binds reliably.
Not from history — from the file. A rule that is re-attached from disk on every call cannot
be compacted away, because it was never in the history to begin with.

So the bridge is not a feature bolted on. It is Picasso's one event and Sarge's one
finding, meeting in the one method that already exists for it: `InvokingCoreAsync`, the
final message list on its way to the model, *after* the reducer has run. Picasso already
overrides it to detect the compaction. The bridge appends the rules right there, last.

## What the developer types

```bash
dotnet add package Picasso.AgentFramework.Persistence
```

```bash
dotnet add package Sarge.AgentFramework
```

```csharp
ChatHistoryProvider = new SqliteChatHistoryProvider("./picasso.db").WithSarge(".sarge")
```

One line, wrapping whatever provider they already have. The `.sarge` file at the repo
root, in the Sarge language, git-ignored as ruled. No organ, no llama.cpp, no local model required — this leg works with whatever
model the agent already uses, frontier included. That is the MVP audience ("Claude and
Cursor-type apps") and it costs the developer nothing they do not already have.

## The Captain's second correction, 18:15: "we want to use it exactly how you are using it now with MAF"

Not delivery alone. **The whole loop, as it runs on the page today, with an Agent
Framework agent as the harness instead of `Sarge.Agent`'s own loop.** Piece by piece,
nothing new invented:

| today, on the page | on MAF |
|---|---|
| `Sarge.Agent` calls the organ over HTTP | the organ is the agent's `IChatClient` — its OpenAI-shaped endpoint already exists |
| `edit_file` · `run_build` · `run_app` · `ask_tutor` | the same four, as MAF `AIFunction` tools |
| rules block at the tail of the task | the `.sarge` block appended at the tail of every turn by the provider decorator |
| the check judges every written file; the run refuses while a rule is broken | unchanged — the edit tool calls the handshake `--check`, a standing hit blocks `run_app` |
| the tutor answers from failure and writes the rule | unchanged — `ask_tutor` |
| the ledger row and the page's rules report | Picasso's store and dashboard: every turn, every compaction, *rules held* |

What a .NET developer gets is the page the Captain presses Run on, inside the framework
they already use, with the conversation kept and the compaction visible. `Sarge.Agent`
stays as the reference loop; `Sarge.AgentFramework` is the same loop, hosted by MAF.

## What it does, exactly

1. On every turn, after the reducer, read `.sarge` (cached by file mtime — a changed book is
   picked up on the next turn without a restart).
2. Render the block (`=== sarge ===` … `=== end sarge ===`, `do` first, `wrong |` / `right |`
   lines intact — the demonstrations are what bind).
3. Append it as the **last user-role message** before the model call. Tail, not head —
   the measured position.
4. Record on the compaction row, when one fires, that the rules were re-attached — so the
   dashboard's banner can say *"12 → 3 · dropped: the first user message · rules held: 4"*.
   Picasso shows the loss; the same line shows what survived it.

Nothing is stored to the database from the rules block: it is transient, re-attached, and
the base class strips provider-contributed messages before `StoreChatHistoryAsync` anyway.

## What it does NOT do, stated before a stranger finds out

- **It does not check the model's output.** The check (the organ judging the written file)
  needs a local model and is a second leg, later. This leg is delivery only: the rules reach
  the model, at the position that works, every time, through compaction. The claim is
  *"corrections that survive"*, exactly as the README already says — not "rules bind".
- **It does not write rules.** The tutor is the user's own session (MVP ruling). The book
  is authored by Sarge's loop, or by hand; the bridge only delivers it.
- **It does not know .NET from Node.** The block is text; the code between the bars is
  whatever the repo is written in.

## Where it lives — corrected by the Captain, 18:10: "we need Sarge as a product on its own"

The first draft put this inside Persistence as a property. That makes Sarge a feature of
Picasso. **Wrong way round.**

- **Sarge is the product.** Its own repo, its own package: the language, the book, the
  delivery, the check, the tutor. It stands alone for Claude Code, Cursor and Node people
  exactly as the MVP says. Nothing in Sarge knows Picasso exists.
- **The bridge is one small package, `Sarge.AgentFramework`.** One line plugs Sarge into
  any Microsoft Agent Framework agent — a `ChatHistoryProvider` decorator that wraps
  whatever provider the developer already has (Picasso's, or MAF's in-memory one) and
  appends the framed `.sarge` block at the tail of every turn. MAF is the first framework
  Sarge plugs into, not the only one.
- **Picasso is the doorway.** It references Sarge, never the reverse: its dashboard shows
  *"rules held: 4"* on the compaction line, and its README points at Sarge. The .NET
  developers who already restored Persistence are the first strangers to meet the product
  — that is what the traction buys, and all it buys.

```csharp
ChatHistoryProvider = new SqliteChatHistoryProvider("./picasso.db").WithSarge(".sarge")
```

Version bump, tag, an independent reviewer signs, or it does not go out — the release law is unchanged.

## What is measured before it is claimed

The same experiment as always, in MAF this time, sequential on one machine, criteria
fixed before running:

```
ARM A   rules in the first user message only     → compaction drops them   (Picasso shows it)
ARM B   rules via SargeRules at the tail          → compaction fires, rules still present
```

Pass: arm B's model output obeys a rule that arm A's stops obeying after the compaction
banner. One conversation long enough to force the reducer. Recorded, then written up,
then the README. Not before.

## The ask

GO to build the property, the class and the two-arm test, in the Picasso repo, on a branch.
Half a day. The README section comes after the test passes, with no number in it.
