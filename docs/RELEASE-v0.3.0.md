# Sarge v0.3.0 — release notes

**Sarge is now the hook.** A small local model judges every file your coding agent writes
against your repo's rules, and refuses to let the app run while a rule is broken.

**Runs on macOS (Apple silicon) and Windows x64. One command, either way.**

```bash
# macOS
curl -fsSL https://raw.githubusercontent.com/1picassoai/Sarge/main/install.sh | bash
```
```powershell
# Windows
irm https://raw.githubusercontent.com/1picassoai/Sarge/main/install.ps1 | iex
```

---

## What changed

**The Python harness is no longer the product.** `python/`, `tools/console.py` and the console
scripts moved to `testing/` — moved, not deleted, with a README saying what each was. They were
our test equipment, and we had been fixing them as though they were the thing being sold.

**Sarge is `hooks/sarge_check.py` — 149 lines.** On every write the organ judges the file
against your `.sarge`; on every run, while a hit stands, the command is refused. Your agent
keeps its own loop. One file in, one verdict out.

**Point Sarge at a model you already run.** `SARGE_ORGAN` takes any OpenAI-shaped endpoint on
your own machine, so a developer with a local model does not have to download a second one.

**Your code cannot leave by accident.** `SARGE_ORGAN` accepts loopback only — `127.0.0.1`,
`::1`, `localhost`. Any other address is refused with an explanation, and the local organ is
used instead. If you genuinely mean to send your source elsewhere, `SARGE_ORGAN_ALLOW_REMOTE=1`
is a second, deliberate act. It never falls back silently.

**Both installers end by pointing you at the hook.** No pip install, no console script. Copy
`hooks/settings.example.json` into your project's `.claude/settings.json` and work as you do.

---

## Checksums

**Every download is verified before it is used.** The installer hashes the organ and the model
and compares them against the values below. A mismatch deletes the file and stops — nothing
unverified is ever installed, unpacked or run.

```
sarge-organ-macos-arm64.zip     d0fdf88e5c4bb9467f3dd3a31ba1f3943eb1e915b9eb0e802b180dedd8d8bb61
sarge-organ-win-x64-cuda13.zip  3d336f87c3be11d3e217c9091922656f943e952188206bf7fba6eb94f3c0c92d
cudart-win-x64-cuda13.zip       37d27a8ff3366f3f2d264dd693a88b0884f457985f00c9d0a416e74266155012
Qwen3-4B-Instruct-2507-Q4_K_M   3605803b982cb64aead44f6c1b2ae36e3acdb41d8e46c8a94c6533bc4c67e597
```

The model hash is Hugging Face's own, checked against the file on disk. A CI job re-verifies
every published asset against what the installers pin, anonymously, on every release.

---

## What is proven

- **The hook: 8 of 8**, on the shipping tree, on both GPU and CPU. Five cases are the loop — a
  bad write is a hit, the run is refused while it stands, a harmless command is allowed, a clean
  write clears it, the run is allowed again. Three are the guard: with the hook copied outside
  the tree and `SARGE_HOME` unset, wrong, or right, **it never reports a pass while checking
  nothing.**
- **When Sarge cannot check, it says so and stops.** No organ, no `SARGE_HOME`, no handshake, no
  `.sarge`, organ still loading — every one exits 2 with `SARGE IS NOT CHECKING THIS FILE`.
  A silent pass is the one failure this tool must never have.
- **The privacy gate**, five ways: default, loopback by IP, localhost by name, remote refused,
  remote explicitly allowed.
- **Every published asset matches what the installers pin**, verified by downloading each one
  anonymously from the public URL and hashing it.
- **The organ builds on Apple silicon with Metal**, and the Sarge core is linked in — checked by
  `nm` as a build step that fails the job.

---

## What is NOT proven

Stated plainly, because a tool that hides its limits gets found out by a stranger.

- **The check is not deterministic.** The same repo and the same book gave four hits on one run
  and passed on three after. This is the model judging, not a rule engine, and **it is not
  fixed.** If a verdict surprises you, run it again — and tell us in Discussions, because the
  cases where it disagrees with itself are exactly what we need.
- **Metal has never executed.** The hosted macOS runners have no GPU, so CI compiles a Metal
  binary and runs it on the CPU. No Mac speed figures exist.
- **No stranger has installed this.** Every run has been on our own machines.
- **The installer builds the handshake from source at the tag**, which is the right shape — the
  binary you get is the tag's, not one of ours, so there is no prebuilt artefact of ours for you
  to get a stale copy of. It also makes one thing release-critical that has never been tested:
  **the tag has to compile on a machine that is not ours.** On Windows that means an MSVC
  toolchain and a Rust install you supply, at whatever versions you happen to have. Every
  handshake binary that has ever existed was built here, on one machine, with one toolchain. If
  `cargo` fails on your box the install dies at that step with a Rust error, and the installer
  is not written to explain that.

  **The organ and the handshake fail in opposite directions.** The organ is a published artefact,
  so it can be stale — the checksum gate covers that. The handshake is built on your machine, so
  it can fail to build — **nothing covers that.**
- **Linux is not released.** `install-linux.sh` is in the tree and **refuses to run**: no Linux
  organ has been built yet. It is left in place because the work is nearly done and a guard that
  explains itself is worth more than a missing file.
- **Intel Macs are refused**, not supported.

---

## The macOS trade-off, stated plainly

The organ is not signed or notarised by Apple, so the installer runs
`xattr -dr com.apple.quarantine` on the folder it just unpacked — **and only that folder**.
Without it macOS shows a security dialog for every library in the organ. Notarisation is the
real fix and it is on the list.

---

## Assets

Two installers and the source. The organ archives are what the installer fetches for you, not
things to download by hand.

---

MIT. What broke, what it got wrong, what you wish it caught:
[Discussions](https://github.com/1picassoai/Sarge/discussions).
