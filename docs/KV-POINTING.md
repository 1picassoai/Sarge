# Pointing the model at a rule — through the KV cache, not the prompt

*Design, 15 Sep 2026, from the Captain's question after the Stanford CME295 attention slide:
"how do we point the local model at the correct rule guide but converted to KV?"*

**Status, 15 Sep 20:35:** lever 2 **built and smoked** as `rust/src/bin/point.rs`. **Run it
from the page:** arm **POINTING** on the console — the binary runs all four arms itself
(`--arm all --json`, model loaded once) and judges every handler; the page only shows what
comes back. `rust/point.cmd` is a manual fallback. Lever 3 remains design only.

**Where it ends up, the Captain's correction:** inside the organ. The handshake is already
linked into llama-server; per request it selects the rule block, the server prefills it as
its own sequence, shifts and copies it into the slot's cache before the assistant turn.
The agent never knows the rule was pointed. Lever 3's mask bias lives in the same place.
`point.exe` is the proof before that carving — a proof's success is a finding, not a feature.

**What the smokes proved:** the kv arm prefills the rule as its own sequence, shifts its K/V
to sit at positions `p..p+r` right before the assistant header, copies it into the main
sequence, and generates coherent code afterwards — correct usings, namespace, context
registered from configuration. The RoPE shift did not corrupt anything.

**Two faults found building it, both llama.cpp semantics, both now in the file's comments:**
the context is *divided* between sequences by default (size it for both), and a
position-range copy between per-sequence streams aborts with no message — the cache must
be **unified** (`with_kv_unified(true)`).

**One greedy sample, not a result:** with the rule at distance one from the write, the model
still wrote `new ToolContext()`. If the four-arm run agrees, position alone is not the
lever, and the answer is the mask.

---

## The formula, and what it lets us touch

```
softmax( QK^T / √d_k ) V
```

Every token the model writes computes a query `Q` against the keys `K` of everything in
context, and the softmax decides how much of each value `V` to blend in. **The KV cache is
literally the `K` and `V` of the prefix, computed once and kept.**

Today's A/B result (`EXPERIMENT-RULES-STICK.md`) is this formula failing us: the rule was in
`K`, verified, and the softmax gave it too little weight when the GET handler was written.
**Present is not attended.**

Without training the weights, there are exactly three levers.

## Lever 1 — which rule tokens exist at all

SELECT. Already built. Decides which rules are in the context. Necessary, and today's
result shows it is not sufficient.

## Lever 2 — where the rule sits

Attention carries a strong recency bias. A rule at the top of a long prompt is a long way
from the token being generated, and the "lost in the middle" effect is well measured.

**The design:** cache each rule as its own KV block, once. When the model is about to write
a file, the handshake selects the one or two rules that file needs and **re-attaches their
cached KV right before the assistant turn** — llama.cpp can copy a cached sequence and
shift its positions (`seq_cp`, `seq_add`, with the RoPE shift handled as context shifting
already does) without recomputing a single token.

The rule's `K` vectors now sit next to the query instead of thousands of positions away.
**No re-tokenising, no recompute. Zero inference cost.**

This alone might be enough. It is the cheap arm and it runs first.

## Lever 3 — the attention logits themselves. This is the answer to the question.

llama.cpp adds a **mask tensor** to `QK^T` before the softmax
(`ggml_soft_max_ext(kq, mask, scale, bias)`). Today that mask holds only `0` and `−∞`, for
causal masking. It is added, not multiplied — which means it is already the `B` in:

```
softmax( (QK^T + B) / √d_k ) V
```

**Put a small positive bias on the columns belonging to the rule's tokens.** The rule is no
longer merely present — it is weighted. The model is *pointed* at it at the moment it
matters, by the mechanism itself rather than by hoping.

**Why this is cheap for us specifically:** the mask tensor already exists and already flows
through both the normal and the flash-attention paths. We compile llama.cpp from source
with our own `build.rs`. This is a targeted patch on a tensor that is already there — tag
the injected rule positions, add `+β` on those columns when the mask is built — not a new
kernel.

## Measurement — before anything is claimed

The same A/B as today, extended. Same fault (`scoped-context-in-handler`), same task, same
model, sequential on one GPU, criteria fixed before running:

```
ARM A   rule in the prompt, at the top            ← today's result: broke it, 1 of 2
ARM B   no rule                                    ← today's control: broke it, 2 of 2
ARM C   rule as cached KV, re-attached before the write      (lever 2)
ARM D   rule as cached KV, re-attached, with +β in the mask   (lever 3)
```

**The rule worked if** arm C or D writes both handlers parameter-injected where arm A did
not. **β is measured, not picked** — sweep it, because too high and the model parrots the
rule text instead of writing code.

## Honest caveats

- **Untested.** Every line above is design.
- **β distorts.** Over-weight the rule and generation degrades. This must be swept, and the
  Captain's law stands: never tune a constant blind.
- **Prior art, from memory, to be verified before citing:** *PASTA* — post-hoc attention
  steering on user-marked spans, reported to improve instruction-following without
  training. If real, it is the academic version of lever 3 and should be cited by us
  before it is cited at us.
- **The flash-attention path** takes the mask too, but the shapes and dtype (F16, padded)
  must be respected or it silently falls back or breaks. Check both paths.
- **One pair is one pair.** Whatever arm D shows wants repeats on other faults.

---

# RESULT — the four arms, 15 Sep 21:15, run by the Captain from the page

Same rule (`scoped-context-in-handler`), same task, same model, greedy. One run per arm.

| arm | verdict | handlers |
|---|---|---|
| none | **broken** | 1 · `new ToolContext()` |
| head | held | 2 · GET used no context · POST injected |
| tail | **held, both clean** | 2 · GET injected · POST injected |
| kv | **broken** | 1 · `new ToolContext()` |

## What it says

**Position is a real lever, and the cheap version wins.** The rule as the *last thing in the
user turn* held in both handlers. At the *head* it held but weaker — the GET handler simply
avoided the context. Recency bias is doing exactly what the literature says.

**The cached block is not invisible — it is weak.** The kv arm looked like `none` in
verdict, so I diffed the two outputs: **they diverge at character 223**, in the usings, and
the kv output is longer. The attached K/V *reached* the model and changed what it imported.
It did not change how it wrote the handler. **Present in K, under-weighted where it
mattered.** This is not a plumbing bug and it is not proof the mask is needed either.

## The likeliest cause, and the next arm

The block was prefilled as **bare rule text** — no chat framing, no system-message wrapper,
attending only to itself from position 0. Its K/V carry "here are some words", not "here is
an instruction". The `tail` arm's rule sat *inside* a user turn the template had framed.

**Next arm — `kvf`:** prefill the rule *rendered through the chat template as a system
message*, then shift and attach exactly as now. If that holds where `kv` broke, framing was
the missing weight and the mask can wait. If it still breaks, lever 3.

## What changes tonight, in the real loop

The agent hands its rules to the model as part of the **system** instructions — the head
position, the weaker one. The experiment says: **put the rules block at the end of the
task**, the tail. One-line change, and it is the experiment's actual product.

**One run per arm. A signal, not a number. @Gareth owns anything outward.**

---

# RESULT 2 — six arms, 17 Sep 07:56, the rule in the Sarge language

Same rule (`scoped-context-in-handler`), same task, same model, greedy, one run per arm —
but the rule now carries `wrong` and `right` code lines, and two arms are new: **kvs** (the
block rendered through the chat template as a system turn, cached, attached before the
assistant header) and **kvr** (the block cached at the **root**, positions 0..199, the
whole prompt shifted after it — where the compiled core lives).

| arm | 15 Sep (sentence rule) | 17 Sep (demonstrated rule) |
|---|---|---|
| none | broken | broken |
| head | held, weaker (1 of 2) | **held, both** |
| tail | held, both | **held, both** |
| kv (bare block, cached at the tail) | **broken** | **held, both** |
| kvs (templated, cached at the tail) | — | **held, both** |
| kvr (cached at the root) | — | **held, both** |

**The isolation arm, 07:59 — same six arms, the rule stripped back to its sentence:**

| arm | sentence only | with `wrong`/`right` code |
|---|---|---|
| none | broken | broken |
| head | held, weak (GET used no context) | held, both |
| tail | held, both | held, both |
| kv (bare, cached at tail) | **broken in GET** | held, both |
| kvs (templated, cached at tail) | **broken in GET** | held, both |
| kvr (cached at root) | **broken in GET** | held, both |

**Now it says which.** The frame is not what made the cache bind — `kvs` with a sentence
broke exactly like `kv`. **The demonstration is.** A rule that carries the wrong line and
the right line as code binds from the cache from *any* position, including the root; a
rule that is only a sentence binds only as text at the tail. Position is the lever for
sentences; **examples make position stop mattering.** Monday's finding stands for
sentence rules and is superseded for demonstrated ones. n = 1 per arm, one rule, one
task — repeat with a second rule before it goes outward. @Gareth owns anything outward.

**Run 2, 08:20 — a second task (three handlers: GET, GET by id, PUT), same six arms, both
rule forms.** This is the n = 2 that was owed, and it changes the reading.

| arm | task 2, demonstrated | task 2, sentence |
|---|---|---|
| none | broken 3/3 | broken 3/3 |
| head | **held 3/3** | **held 3/3** |
| tail | **held 3/3** | **held 3/3** |
| kv (bare, cached at tail) | **broken 3/3** | held 3/3 |
| kvs (templated, cached at tail) | held 3/3 | **broken 3/3** |
| kvr (cached at root) | held 3/3 | "held" — 0 clean, 0 faulty: every handler avoided the context entirely |

**The honest reading, across both tasks (n = 2 per arm):** the **text arms held every
time** — head and tail, twelve of twelve handlers across four runs. The **cache arms are
unstable**: the same attachment held on one task and broke on the other, in both rule
forms, with no pattern that survives two runs. Demonstrations helped the head arm (weak
on task 1 as a sentence, solid with code) and did not make the cache reliable. So the
claim written at 08:03 — "a demonstrated rule binds from the cache from any position" —
**does not survive its second run and is withdrawn.** What stands: *rules as text, at the
tail (or head), with code demonstrations, bind reliably; cached K/V does not, yet.*

~~**What this means for the design.** The cache tier is real *for rules written in Sarge* —~~
*(withdrawn 08:20, see Run 2)* The cache tier was to be real *for rules written in Sarge* —
which is every rule the tutor writes now. And it is cheaper than the surgery: a block at
the root of every request is already cached by llama-server's own prompt cache
(`cache_prompt`), with no `seq_cp` at all. So "baked as K/V" can ship as: *the organ puts
the repo's `.sarge` block at the root, beside the compiled core, and the server's prompt
cache keeps its K/V.* One place, no patch to attention.

## Why this matters beyond the fix

It closes the gap the whole series is about. **"In the context window" and "binding" are
different things** — that is the disobedience thesis. Lever 3 is the first mechanism we
have that operates on the *binding* side of that line rather than the *presence* side. If
it works, "corrections that survive" becomes a claim about attention weight, not about
prompt text, and that is a claim with a formula under it.
