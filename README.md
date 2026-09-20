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

**[Installation and first run →](../../wiki/Set-up-with-LangChain)**

## What you need

**8 GB RAM and 4 cores.** A GPU makes it fast, it does not make it possible.

Measured on Qwen3-4B (Q4_K_M, the model Sarge ships with), no GPU layers at all:

| | 4 cores | 8 cores |
|---|---|---|
| writing code | 11.6 tokens/sec | 13.7 tokens/sec |
| reading a file to judge it | 300 tokens/sec | 259 tokens/sec |

So **the check is fast on a plain CPU** — judging is nearly all reading, and a file comes back
in about a second. **Writing is the slow half**: roughly a minute for a hundred-line file, so
a full task takes minutes rather than seconds. On an 8 GB GPU it is about ten times quicker.

MIT. Bring what your coding agent keeps getting wrong: [Discussions](../../discussions).
