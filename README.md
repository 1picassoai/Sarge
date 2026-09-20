# Sarge

**Drills it until it sticks. Doesn't negotiate.**

![The test web UI showing the agent coding in real time when presented with a coding challenge](docs/console.png)

## Why I built this

I wrote the rules of my codebase down for my coding agent. It read them, agreed with them,
and then shipped code that broke them anyway. Sometimes it did not even compile. So I
reviewed everything it wrote, every time, paying twice: once in tokens for code I could not
trust, and once in my own hours to find out why. The rules file was not the problem. Nothing
made the agent *keep* a rule after it had read it.

## What it does

A small model runs on your own GPU and writes the code. Every file it writes is checked
against a rule book that lives in your repo, and **the run refuses while a rule is broken.**
When it gets something wrong twice, a tutor teaches it that rule once, from its own mistake,
and writes it into the book. Next time, the rule is already there.

![Sarge architecture: your agent hands a task to the harness; the local model writes a file; the check judges it against the book; a hit stops the run; the tutor is the one thing that leaves your machine](docs/architecture.png)

Works with **LangChain / LangGraph** and as a **Claude Code hook**. The tutor is the one
thing that leaves your machine, and only if you give it a key.

## Get started

**[Installation and first run →](../../wiki/Set-up-with-LangChain)**

MIT. Bring what your coding agent keeps getting wrong: [Discussions](../../discussions).
