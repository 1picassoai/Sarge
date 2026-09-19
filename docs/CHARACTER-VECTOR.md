# The character as a control vector

*15 Sep 2026, on the Captain's ruling: rules stay text at the tail of the task; the
character goes one rung deeper, into a control vector. Ladder in THE-BIBLE.md, Part One.*

## What it is

A control vector is one direction per layer, added to the residual stream at inference.
llama.cpp's own `cvector-generator` makes it from pairs of prompts — the character and its
opposite — by PCA over the difference in hidden states. No weights touched. The organ takes
it natively: `--control-vector-scaled character.gguf:SCALE --control-vector-layer-range 10 35`.
Nothing new in the agent, the handshake or the console. It is baked into llama.cpp.

## The files

```
harness/cvector/positive.txt     12 lines · the character, ChatML-framed, from CHARACTER.md
harness/cvector/negative.txt     12 lines · the opposite disposition, same questions
harness/cvector/character.gguf   the vector · 35 layers · generated on the GPU, PCA
harness/cvector/smoke.ps1        second server on :8422, one question, one scale · the ruler
organ/llama-src/organ-character.cmd   the organ with the vector · scale is the argument
```

## What the smoke showed — a signal, not a number

Greedy, one question, one run per scale. The live organ (no vector) is the baseline.

**Sign is settled: positive is the character.** At scale −4 the model said *"Yes … I am
finished"* to a green build, and blamed a 500 on a *Startup.cs* that does not exist in a
minimal API — the two things the character forbids. At +4 and at baseline it said no,
run it first; and blamed its own handler.

**The baseline already holds the character in prose.** On all three questions the bare
model answered the way CHARACTER.md wants. The vector's visible work so far is making the
*opposite* direction worse, not making the right direction better. Whether it helps
**inside the loop** — fewer identical rewrites, more asks, running before stopping — is not
shown yet and is the only test that counts.

**Scale.** 0.8 and 2: no visible change. 4: visible. Nothing above 4 tried. 4 is the
first scale that moved the model, not a tuned constant. It is the default in
`organ-character.cmd` because it is the only measured point; sweep before believing it.

## What is not proven

- Any effect in the real loop. That is a page run with the organ started from
  `organ-character.cmd`, same task as a run without it, verdict and ledger compared.
- That scale 4 does not degrade the code. In the stall question at +4 the answer lost its
  "ask the tutor" and went to *"I would…"* — prose where the character wants a tool call.
  A vector that makes the model politer and less decisive is the wrong vector.
- Twelve pairs is few. If the loop shows nothing, more pairs before more scale.

**One run per point. @Gareth owns anything that goes outward.**
