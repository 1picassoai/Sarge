# Sarge v0.3.1 — release notes

**The check is fixed.** v0.3.0 shipped with a judge that lost track of which rule was which
on any file longer than its test fixtures, and reported CHECKS PASSED over real faults. It is
replaced. The hook, the installer, the organ and the model are unchanged.

**macOS, Apple silicon. One command.**

```bash
curl -fsSL https://raw.githubusercontent.com/1picassoai/Sarge/main/install.sh | bash
```

**Linux is coming** — the installer is in the tree and refuses to run until the organ is built
for it. **Windows is not in this release**; if there is demand for it, say so in
[Discussions](https://github.com/1picassoai/Sarge/discussions).

---

## What changed

**The check reads statements and asks two rules at a time.** Given ten numbered rules and a
73-line file, the model found every fault and then labelled them 1, 2, 3 — by position, not
by the rule each one broke — and a file with four real faults came back CHECKS PASSED. Rules
are now named, not numbered; the file is judged two rules at a time and the hits pooled; and a
call that wraps over five lines is read as one statement, not as its first line.

**A rule the check cannot tell apart is dropped, not waved through.** The gate keeps a
candidate only when the line carries a word the rule's wrong example has and its right example
lacks. When that set was empty it used to mean *keep everything* — and on a 69-line file of
correct code that put twelve false flags on `res.json(rows)`. An empty set now means the check
cannot enforce that rule, and says so.

**The book is 22 rules, and the check enforces 19 of them.** Ten rules were struck since
v0.3.0. Seven needed proof that lives elsewhere in the file — `const app = express()` is only
wrong if `app.disable('x-powered-by')` appears nowhere, and a line judge cannot see nowhere.
Three have faults that are *absences* — a missing `await`, a missing `Error`, `||` where `??`
was meant — and the check finds a fault by what a line carries. The README states both numbers.

**The test corpus went from four files to eleven.** Every fixture the check had ever been
measured on was 14 to 16 lines with one fault on one line. Two bugs lived a week in the gap
between that and a real file. There are now four *clean* files it must pass, three long ones,
and one recorded known miss.

**The README's comparison against a linter was re-measured on the new book.** Sarge now catches
the hardcoded database path at all three sites; it still misses the per-request connection,
and the table says so. The file it was measured on is in the tree — run it yourself.

---

## Checksums

**Unchanged from v0.3.0.** Nothing the organ is built from changed, so the same asset is
attached to this release and the installer pins the same value. The model is the same file.

```
sarge-organ-macos-arm64.zip     d0fdf88e5c4bb9467f3dd3a31ba1f3943eb1e915b9eb0e802b180dedd8d8bb61
Qwen3-4B-Instruct-2507-Q4_K_M   3605803b982cb64aead44f6c1b2ae36e3acdb41d8e46c8a94c6533bc4c67e597
```

A CI job re-verifies every published asset against what the installer pins, anonymously, on
every release.

---

## What is proven

- **The replay: every bad fixture hits at the right line, every clean fixture passes**, on the
  shipping tree's own binary, with the shipped book. Eleven files.
- **The hook: 8 of 8**, unchanged — the loop and the guard, and it never reports a pass while
  checking nothing.
- **The false positives are gone**, measured three consecutive runs on each of four clean files
  by a second reviewer on their own hand, not the author's.
- **22 listed, 19 enforced** was checked against what the binary reports, not against the
  sentence that claims it.

---

## What is NOT proven

- **The book lists more rules than it judges**, and the reason is the tokeniser: it lowercases
  before comparing, so `Error` dies on `error`; and punctuation is a separator, so `??` and
  `||` cannot be tokens. Fixing that changes how every rule is gated and is its own release.
  `rust/tests/judge/js-absence-knownmiss.js` holds four real faults of which one is caught;
  the day the tokeniser is fixed it goes to four, and that is how you will know.
- **Three rules that caught real faults in v0.3.0 are gone.** They caught by the same accident
  that flagged correct code, and a mechanism that cannot tell the two apart is guessing. But
  the miss is real and it is not hidden.
- **The check is still not deterministic.** It is a model judging. If a verdict surprises you,
  run it again, and tell us.
- **Metal has never executed, no stranger has installed this, and nobody has compiled the tag
  on a Mac** — all exactly as stated for v0.3.0. This release changes none of that.
- **Linux is not released. Intel Macs are refused.**

---

## Assets

`install.sh`, `install-linux.sh` and the source. The organ archive is what the installer fetches
for you, not something to download by hand — its checksum is above if you want to verify it
yourself.

---

MIT. What broke, what it got wrong, what you wish it caught:
[Discussions](https://github.com/1picassoai/Sarge/discussions).
