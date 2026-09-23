# Sarge

**Prompts negotiate. Sarge doesn't.**

### **Runs on macOS and Windows. One command, either way.**

> **Stop babysitting your coding agent.** Sarge checks every file it writes against your
> repository's rules and refuses to run the code until it complies. When the agent breaks
> the same rule twice, Sarge teaches it the fix — permanently. **The result: a small local
> model that gets better at coding your repo with every run.**

## Why I built this

Every session with a frontier model started the same way: me explaining how my repo is
written — the structure, the conventions, the things we never do. It would agree, code for
a while, and the moment the session ended the explanation was gone. Next session, same
speech. That cost twice: the tokens to say it, and the frustration of saying it again. A
rule I have to repeat is not a rule the model holds. Sarge is the part that makes it hold.

## What it does

Sarge ships a book of best-practice rules for Node and Express — taken from the
documentation, written as code — and holds a small local model to them. A generic small
model writes generic code. It does not know that `express.json()` has to be registered
before a route reads `req.body`, that `DatabaseSync` is synchronous and `await` on it is a
bug, or that a write should check its row count before reporting success. The book knows —
sixty-four rules, each carrying a wrong line and a right line, because
a next-token predictor follows a demonstration better than a description. Sarge is the part
that makes the model *keep* them: the rules ride at the tail of every turn, the check
enforces them on every file, and the ones the model breaks anyway are taught back.

Your own conventions go in a `.sarge` at the repo root — same syntax, seven keywords — and
override the shipped book wherever they clash.

![Sarge architecture: your own agent writes a file; the check judges it on your machine against your book; while a rule is broken the run is refused; the tutor is the one thing that leaves](docs/architecture.png)

**Seven tasks on one repo, seven of seven correct.** The same model without Sarge: none.
**The check itself: six of seven known faults caught, with one false flag** — it is your
local model judging a line, so it is only as good as that model.

It is a **hook on the agent you already use** — your agent keeps its own loop, its own
tools and its own build; Sarge judges the file it writes and refuses the run while a rule
is broken. Claude Code today. Node and TypeScript. The tutor is the one thing that leaves
your machine, and only if you give it a key.

## Get started

One command.

**Windows x64**, in PowerShell:

```powershell
irm https://raw.githubusercontent.com/1picassoai/Sarge/main/install.ps1 | iex
```

**macOS on Apple silicon**, in a terminal:

```bash
curl -fsSL https://raw.githubusercontent.com/1picassoai/Sarge/main/install.sh | bash
```

*macOS support is new. It is built and tested on Apple silicon by CI, which has no GPU —
so the install and the check are proven there, and Metal has never been exercised. No Mac
speed figures are published for that reason. If it misbehaves on your machine, say so in
[Discussions](../../discussions) and it gets fixed.*

**Every download is checked.** Both installers verify the sha256 of the organ, the model and
the CUDA runtime against the published hashes, and delete the file and stop on a mismatch.

**One trade-off you should know about, on macOS.** The organ is not yet signed or notarised
by Apple, so the installer runs `xattr -dr com.apple.quarantine` on the folder it just
unpacked — and only that folder. Without it macOS shows a security dialog for every library
in the organ. This is a documented compromise, not a hidden one: the alternative pushes
people into clicking through a stack of warnings or disabling Gatekeeper system-wide, which
is worse. Notarisation is the real fix and it is on the list.

**What it does, and nothing else:** clones the `v0.2.0` tag into a `Sarge` folder, downloads
the prebuilt organ for your platform and the model (2.3 GB) from Hugging Face, writes a
start script with your own paths in it, then starts the organ and **proves the check can
catch a known-bad file before it tells you it is done.** Nothing is installed system-wide,
and nothing is pip-installed.

On Windows the CUDA runtime (405 MB) is fetched only if you have an NVIDIA card and no
toolkit already. No card, no download, and Sarge runs on the CPU. On Apple silicon Metal
ships with the OS and nothing extra is downloaded.

Then start the organ — `start-organ.cmd` on Windows, `./start-organ.sh` on macOS — and
leave it running.

**Wire it into your agent.** In the repo you want guarded, copy
`hooks/settings.example.json` into `.claude/settings.json`, and put a `.sarge` at the
repo's root (start from `book/universal.sarge`). From then on every file your agent writes
is judged against your rules, and the run refuses while a rule is broken.

## What you need

**8 GB RAM and 4 CPU cores.** A GPU makes it faster; it is not required.

Measured on Qwen3-4B (Q4_K_M), the model Sarge runs with, on a quiet Windows machine with
an 8 GB NVIDIA card. No Mac figures yet:

| | 4 cores, no GPU | 8 GB GPU |
|---|---|---|
| writing code | 11.4 tok/s | 72 tok/s |
| checking one 2,250-token file | ~70 s | a few seconds |

**It runs on a plain laptop, and it is comfortable on a card.**

MIT. Bring what your coding agent keeps getting wrong: [Discussions](../../discussions).
