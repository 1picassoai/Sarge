# C# fixtures — archived 22 Sep 2026, on the Captain's word

Not deleted. Moved here whole, from `rust/tests/judge/`.

**Why.** The Captain ruled on 22 Sep that Sarge releases for **Node only**: *"we don't need
C sharp… we are only releasing the node book."* These are test fixtures, not product, but
they are the C# corpus and they do not belong in a Node-only release.

**What this is.** Six `.cs` files and `csharp.sarge`, the rule book they were judged
against. Three of the six are deliberately faulty (`-bad`), two are correct (`-good`), and
they were the replay corpus for the check.

**Why it was archived rather than deleted.** `docs/FINDINGS.md` states the check caught
**six of seven known faults with one false flag**. That number rests on this corpus. A
published figure whose evidence has been destroyed cannot be defended, so the evidence
stays. (`feedback-sarge-docs-never-delete` — docs are thesis material; curate by copying
in, never delete.)

**What stopped using them.** `rust/replay-check.cmd` ran a C# block over these files; that
block is gone. The Node half of the replay is untouched and still runs against
`rust/tests/judge/`.

**If C# ever comes back:** move the seven files back to `rust/tests/judge/` and restore the
C# block in `replay-check.cmd`. Nothing else referenced them.
