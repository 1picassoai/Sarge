"""
THE COMPACTION TEST - does a correction survive the context being cut?

Two arms, same model, same book, same three turns, fresh folder each:

  A  rules delivered ONCE, at the tail of the first task (the .NET host's way).
  B  rules re-attached by the middleware on EVERY model call (python/sarge's way).

Both arms run under a trim: on every model call the model sees only the current turn -
the first user message (which in arm A held the rules) is gone from turn 2 on. That is
the compaction Picasso records, done deliberately.

The rule under test is `use-config-for-connection`: read the database file name from
config.json; never hard-code it. Turns 2 and 3 each add a file that opens the database
itself, so each turn the model either reads the config or hard-codes 'tools.db'.

The judge is the check, not me: the harness runs `handshake --check` on every written
file, and a CHECK FAILED in the log is a hit. Count per arm, and the files at the end.

    python experiments\compaction.py [--arm A|B|both]

Pass: arm B's turns 2 and 3 obey the rule where arm A's do not. n = 1 per arm, one rule -
a signal, not a number. @Gareth owns anything outward.
"""
from __future__ import annotations

import argparse
import re
import shutil
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from langchain.agents import create_agent  # noqa: E402
from langchain.agents.middleware import ModelCallLimitMiddleware, wrap_model_call  # noqa: E402
from langchain_core.messages import HumanMessage  # noqa: E402
from langgraph.checkpoint.memory import InMemorySaver  # noqa: E402

from sarge import Sarge  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
# Where the scratch repos live (folders workshop-compact-a / -b are created there) and
# which book seeds them. Override with SARGE_REPOS / SARGE_BOOK_SRC.
import os  # noqa: E402
REPOS = Path(os.environ.get("SARGE_REPOS", str(Path.home() / "source" / "repos")))
BOOK_SRC = Path(os.environ.get("SARGE_BOOK_SRC", str(REPOS / "workshop-node" / ".sarge")))

TURNS = [
    "Create package.json (\"type\": \"module\", express as the only dependency, a start script running server.js), "
    "config.json holding the database file name, and server.js: an Express app that opens node:sqlite "
    "(import { DatabaseSync } from 'node:sqlite') using the file name read from config.json with import.meta.dirname, "
    "creates a tools table (id, name, price) if it does not exist, and serves GET /tools returning all rows. "
    "Listen on process.env.PORT or 3000. Build it, run it, and check GET /tools answers.",
    "Add categories.js exporting an Express Router with GET /categories that opens the database itself and reads "
    "from a categories table (id, name), creating the table if it does not exist. Mount the router in server.js. "
    "Build it, run it, and check GET /categories answers.",
    "Add stats.js exporting an Express Router with GET /stats that opens the database itself and returns the number "
    "of rows in the tools table. Mount the router in server.js. Build it, run it, and check GET /stats answers.",
]


def trim_to_current_turn():
    """The compaction: the model sees only the messages from the last human turn onward."""
    @wrap_model_call
    def compaction(request, handler):
        msgs = request.messages
        # The rules block is a human-role message too; it is never the turn boundary.
        last_human = max((i for i, m in enumerate(msgs)
                          if m.type == "human" and not str(m.content).lstrip().startswith(("=== sarge", "## RULES"))), default=0)
        if last_human > 0:
            return handler(request.override(messages=list(msgs[last_human:])))
        return handler(request)
    return compaction


def run_arm(arm: str) -> dict:
    repo = REPOS / f"workshop-compact-{arm.lower()}"
    if repo.exists():
        # A fresh folder every time: the model's own last run must not be the base.
        for p in repo.iterdir():
            if p.name in (".sarge", ".gitignore"):
                continue
            shutil.rmtree(p) if p.is_dir() else p.unlink()
    repo.mkdir(exist_ok=True)
    shutil.copy(BOOK_SRC, repo / ".sarge")
    (repo / ".gitignore").write_text("# Sarge's rule book for this repo - machine-written, stays on this machine\n.sarge\n", encoding="utf-8")

    s = Sarge(repo=str(repo), task=TURNS[0], harness=True)
    run = s.run
    block = run.rules_block()
    # ORDER MATTERS, measured the hard way (first run, 18 Sep 06:40): the first middleware
    # in the list is the OUTERMOST wrap. With rules before trim, the rules block was appended
    # first and the trim then cut everything before "the last human message" - which was
    # the rules block itself. Arm B saw rules and no task for sixteen calls and wrote
    # nothing. The trim must run first, on the real conversation; the rules go on after.
    middleware = [trim_to_current_turn(), s.stop(), ModelCallLimitMiddleware(run_limit=30, exit_behavior="end")]
    if arm == "B":
        middleware.insert(1, s.rules())
    agent = create_agent(s.llm(), s.tools(), system_prompt=s.character(), middleware=middleware,
                         checkpointer=InMemorySaver())
    cfg = {"configurable": {"thread_id": f"compact-{arm}"}, "recursion_limit": 100}

    results = []
    for i, turn in enumerate(TURNS, 1):
        run.task = turn
        text = turn + ("\n\n" + block if (arm == "A" and i == 1) else "")
        before = len(run.log)
        try:
            agent.invoke({"messages": [HumanMessage(content=text)]}, config=cfg)
        except Exception as ex:  # noqa: BLE001
            run.log.append(f"- AGENT ERROR on turn {i}: {type(ex).__name__}: {ex}")
        new = run.log[before:]
        hits = sum(1 for l in new if l.startswith("- CHECK FAILED"))
        writes = [l for l in new if l.startswith(("- write_file", "- edit_file")) and "REJECTED" not in l]
        results.append({"turn": i, "check_failed": hits, "writes": len(writes),
                        "verdict": run.verdict, "calls": run.tool_calls})
        print(f"  arm {arm} turn {i}: check failed ×{hits}, {len(writes)} write(s), verdict so far {run.verdict}", flush=True)

    # What the files actually say about the database file name - the line the rule is about.
    lines = {}
    for f in ("server.js", "categories.js", "stats.js"):
        p = repo / f
        if p.is_file():
            m = re.search(r"new DatabaseSync\(([^)]*)\)", p.read_text(encoding="utf-8", errors="replace"))
            lines[f] = m.group(1) if m else "(no DatabaseSync call)"
        else:
            lines[f] = "(not written)"
    return {"arm": arm, "turns": results, "db_lines": lines, "rules_held": run.rules_held,
            "deliveries": run.deliveries, "log": list(run.log)}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--arm", default="both", choices=["A", "B", "both"])
    a = ap.parse_args()
    arms = ["A", "B"] if a.arm == "both" else [a.arm]
    out = ROOT / "runs" / "compaction"
    out.mkdir(parents=True, exist_ok=True)
    for arm in arms:
        print(f"=== arm {arm} ===", flush=True)
        r = run_arm(arm)
        (out / f"arm-{arm}.log").write_text("\n".join(r["log"]), encoding="utf-8")
        print(f"  rules held {r['rules_held']}, deliveries {r['deliveries']}")
        for f, l in r["db_lines"].items():
            print(f"  {f:14} {l}")
        (out / f"arm-{arm}.txt").write_text(
            f"arm {arm}\n" + "\n".join(f"turn {t['turn']}: check failed x{t['check_failed']}, writes {t['writes']}, verdict {t['verdict']}" for t in r["turns"])
            + "\n" + "\n".join(f"{f}: {l}" for f, l in r["db_lines"].items()) + f"\nrules held {r['rules_held']}, deliveries {r['deliveries']}\n",
            encoding="utf-8")


if __name__ == "__main__":
    main()
