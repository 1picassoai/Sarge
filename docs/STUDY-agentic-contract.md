# Study — agentralabs/agentic-contract

*16 Sep 2026, on the Captain's ask. Read from the public repo (README, crate source,
paper .tex) with no VPN up. Numbers quoted are theirs, from their README and paper — not
ours, and not verified by us.*

## What it is

A **runtime policy engine** for agents. Rust core (they say 7,382 lines), one binary file
per contract (`.acon`, fixed-size records, BLAKE3 checksum), exposed as a CLI, an MCP
server (38 tools), Python/npm/FFI/wasm bindings. MIT. One author (Omoshola Owolabi, Agentra
Labs). A LaTeX paper in the repo: *"A Runtime Policy Engine for Governed AI Agent Systems"*.

Six primitives: policies · risk limits · approvals · conditions · obligations · violations.
A policy is `{id, label, scope, action, active, conditions, tags}`; action is one of
`allow / deny / require_approval / audit_only`.

## How it actually works — read from the source, not the README

**Matching is a case-insensitive substring test between the policy's label and the
action string** (`contract_engine.rs`):

```
label_lower.contains(&action_lower) || action_lower.contains(&label_lower)
```

Deny wins, then require_approval, then audit_only; **nothing matched = allow.**
The `conditions` field is "expression strings" with **no evaluator in the code** — a
placeholder.

**Enforcement is voluntary.** The README: *"The agent checks policies before acting via
`policy_check()`… If denied, the agent stops."* The MCP server does not intercept anything;
the agent must choose to call the check, and must choose to obey the answer. The README
does not say what happens if it doesn't.

**The paper measures the engine, not the agent.** Evaluation is Criterion benchmarks
(policy evaluation in nanoseconds, file I/O in microseconds) and 288 unit tests. **There is
no experiment where an agent is shown to obey a policy.** Zero compliance data.

## What it is not

- Not a code-rules system. It governs *actions* ("deploy to production", "call external
  API"), by label, not code.
- Not enforcement at the point of action. It is a lookup the agent is asked to perform.
- Not evidence about model behaviour. It has no model in it.

## What it means for us

**It is the mirror image of our finding.** They built a fast, tidy place to *store* rules
and a clean API to *ask* about them, and skipped the only question that matters: does the
agent do what the answer says? Our whole series this week is that question. Read, recited,
skipped — their design assumes that never happens.

Three things worth taking, none of them the architecture:

1. **The vocabulary is good.** *Obligations* (things the agent must do after) and
   *violations* (a first-class record of a break) are two nouns our book does not have.
   Our `standingHit` is a violation without a table. Worth naming.
2. **Deny > require_approval > audit_only > allow** is a clean precedence. Our rules are
   prohibit/instruct; "require approval" (stop and ask the human) is a kind we could add
   — it is the forced ask with a human at the end instead of the tutor.
3. **One portable file with a checksum** is the right shape for a book that travels with
   a repo. Ours is JSONL — fine, but it has no integrity check, and a BOM hid rule 1 for
   days. A checksum would have shouted.

**Where we are ahead, and it is not close:** we enforce on the write and refuse the run;
we measured position; we have a tutor that writes the rule from the failure; we have
compliance data, including the negative results. They have nanoseconds.

**Where they are ahead:** packaging. Four language bindings, an MCP server, a paper, a
CHANGELOG, a SECURITY.md, a licence. A stranger can install it in a minute. A stranger
following our STRANGER.md fails at step two (THE-BIBLE, Part Eight).

## One recommendation

Do not build toward it and do not cite it as competition — it is a different thing wearing
a similar word. Take the two nouns (obligation, violation) and the checksum. And note,
for Fox: **"a policy engine with no compliance experiment" is exactly the gap our series
fills** — that is a sentence for a post, once Gareth has the numbers.
