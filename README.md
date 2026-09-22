# Sarge

**Prompts negotiate. Sarge doesn't.**

### **Runs on macOS and Windows. One command, either way.**

> **Stop babysitting your coding agent.** Sarge checks every file it writes against your
> repository's rules and refuses to run the code until it complies. When the agent breaks
> the same rule twice, Sarge teaches it the fix — permanently. **The result: a small local
> model that gets better at coding your repo with every run.**

![One run: the local model writes, the Sarge check judges every file on this machine, the tutor is the one thing that leaves - 17 seconds, ALL ANSWERED](docs/run-animation.gif)

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

![Sarge architecture: your agent hands a task to the harness; the local model writes a file; the check judges it against the book; a hit stops the run; the tutor is the one thing that leaves your machine](docs/architecture.png)

**Seven tasks on one repo, seven of seven correct.** The same model without Sarge: none.
**The check itself: six of seven known faults caught, with one false flag** — it is your
local model judging a line, so it is only as good as that model.

Works with **LangChain / LangGraph** and as a **Claude Code hook**. Node and TypeScript.
The tutor is the one thing that leaves your machine, and only if you give it a key.

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
the prebuilt organ for your platform and the model (2.3 GB) from Hugging Face, `pip install`s
the Python harness into that clone, writes two start scripts with your own paths in them,
then starts the organ and **proves the check can catch a known-bad file before it tells you
it is done.** Nothing is installed system-wide.

On Windows the CUDA runtime (405 MB) is fetched only if you have an NVIDIA card and no
toolkit already. No card, no download, and Sarge runs on the CPU. On Apple silicon Metal
ships with the OS and nothing extra is downloaded.

Then: `start-organ`, `start-console`, and open `http://127.0.0.1:8420`.
(`.cmd` on Windows, `.sh` on macOS.)

Prefer to do it by hand? **[The long way, every command explained →](../../wiki/Set-up-with-LangChain)**

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
