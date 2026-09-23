# testing — not the product

**The Captain's ruling, Wed 23 Sep 2026:** *"strip everything that is not the product and
add it to testing."*

Everything in this folder is equipment we built to develop and prove Sarge. None of it is
installed, none of it ships, and nothing here is what a user runs.

| | what it is |
|---|---|
| `python/` | a LangChain host — our own agent loop. It gives a model seven tools, runs the build, hits the endpoints and prints a verdict. It exists so the check could be exercised end to end before there was anything to hook into. |
| `tools/console.py` | the page on :8420 — a folder, a task, Run. A test rig with a nice face. |
| `agent/` | the older .NET host, kept for its history. |
| `console.cmd`, `start-console.sh` | the scripts that started the page. |

## Why it is not the product

The product is the **check**, the **book**, the **tutor** and the **organ**, reached from
the agent a developer already uses — `hooks/sarge_check.py`, 149 lines, judging every file
Claude Code writes and refusing the run while a rule is broken.

The agent frameworks already have a loop, tools and a build. We wrote a second one to test
against, and then kept fixing it as though it were the thing being sold. Two days of faults
found in it — a verdict that said WORKS over a 500, a tutor asked to go looking instead of
being shown the failure, no regression check at all — were faults in test equipment.

**Sarge sits on code, not on tool usage.** One file in, one verdict out.

## It still earns its keep

This is how a change to the check or the book gets exercised over a real task before it
reaches anyone. Keep it working, keep it honest, and never quote a number from it as though
a user had produced it.
