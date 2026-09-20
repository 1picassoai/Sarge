# Sarge

**Prompts negotiate. Sarge doesn't.**

> **Stop babysitting your coding agent.** Sarge checks every file it writes against your
> repository's rules and refuses to run the code until it complies. When the agent breaks the
> same rule twice, Sarge teaches it the fix — permanently.

![The test web UI showing the agent coding in real time when presented with a coding challenge](docs/console.png)

## Why I built this

I wrote the rules of my codebase down for my coding agent. It read them, agreed with them,
and then shipped code that broke them anyway. Sometimes it did not even compile. So I
reviewed everything it wrote, every time, paying twice: once in tokens for code I could not
trust, and once in my own hours to find out why. The rules file was not the problem. Nothing
made the agent *keep* a rule after it had read it.

## What it does

A small model runs on your own machine and writes the code. Every file it writes is judged
against a rule book that lives in your repo — not by a regex, by the model itself — and
**the run refuses while a rule is broken.** When it gets the same thing wrong twice, a tutor
teaches it that rule once, from its own mistake, and writes it into the book. Next time, the
rule is already there, and it holds.

**Seven tasks on one repo, seven of seven correct.** The same model without Sarge: none.
Every failure on the way is written down with its cause, most of them ours.

![Sarge architecture: your agent hands a task to the harness; the local model writes a file; the check judges it against the book; a hit stops the run; the tutor is the one thing that leaves your machine](docs/architecture.png)

Works with **LangChain / LangGraph** and as a **Claude Code hook**. The tutor is the one
thing that leaves your machine, and only if you give it a key.

## Get started

One command, in PowerShell:

```powershell
irm https://raw.githubusercontent.com/1picassoai/Sarge/main/install.ps1 | iex
```

**What it does, and nothing else:** clones the `v0.1.0` tag into `.\Sarge`, downloads the
prebuilt organ (46 MB) from this repo's release and the model (2.4 GB) from Hugging Face,
`pip install`s the Python harness into that clone, writes two start scripts with your own
paths in them, then starts the organ and **proves the check can catch a known-bad file
before it tells you it is done.** Nothing is installed system-wide.

The CUDA runtime (405 MB) is fetched only if you have an NVIDIA card and no toolkit already.
No card, no download, and Sarge runs on the CPU.

Then: `start-organ.cmd`, `start-console.cmd`, and open `http://127.0.0.1:8420`.

Prefer to do it by hand? **[The long way, every command explained →](../../wiki/Set-up-with-LangChain)**

## What you need — no GPU required

**8 GB RAM and 4 CPU cores.** A GPU makes it faster; it is not required.

Measured on Qwen3-4B (Q4_K_M, the model Sarge ships with), same prompt on each:

| | 4 cores, no GPU | 8 cores, no GPU | 8 GB GPU |
|---|---|---|---|
| **writing code** | 11.6 tok/s | 13.7 tok/s | **72 tok/s** |
| **reading a file to judge it** | 300 tok/s | 259 tok/s | 405 tok/s |

**Judging is fast everywhere** — it is nearly all reading, and a file comes back in about a
second even with no GPU at all. **Writing is the slow half**: roughly a minute for a
hundred-line file on a CPU, about six times quicker on a card.

MIT. Bring what your coding agent keeps getting wrong: [Discussions](../../discussions).
