"""
The reference host, same command line as agent/Sarge.Agent so the page's Run button can
drive it as another arm:

    python -m sarge --repo <dir> --task "<text>" [--harness] [--book <file>] [--plan-first] [--review <file>]

Runs one task, prints what the agent said, writes the run log and the ledger row.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
import time
from datetime import datetime
from pathlib import Path

from .agent import Sarge


def main(argv=None) -> int:
    ap = argparse.ArgumentParser(prog="sarge")
    ap.add_argument("--repo", required=True)
    ap.add_argument("--task", required=True)
    ap.add_argument("--harness", action="store_true")
    ap.add_argument("--book")
    ap.add_argument("--plan-first", action="store_true")
    ap.add_argument("--review")
    ap.add_argument("--max-turns", type=int, default=12)   # accepted for the page; the framework owns the loop
    ap.add_argument("--picasso")                            # accepted for the page; no store on this host yet
    a = ap.parse_args(argv)

    try:
        s = Sarge(repo=a.repo, task=a.task, book_path=a.book, harness=a.harness,
                  plan_first=a.plan_first, review_path=a.review)
    except Exception as ex:  # noqa: BLE001
        print(ex, file=sys.stderr)
        return 2
    run, o = s.run, s.options
    mode = "HARNESS" if a.harness else "RAW"

    model_name = "unknown model"
    try:
        import urllib.request
        props = json.loads(urllib.request.urlopen(o.organ_url.replace("/v1", "") + "/props", timeout=5).read())
        model_name = Path(props.get("model_path", "")).stem or model_name
    except Exception:  # noqa: BLE001
        pass

    print(f"Sarge agent  [{mode}]  (LangChain)\n  repo     {run.repo}\n  model    {model_name} via the organ {o.organ_url}\n"
          f"  home     {o.home}\n  rules    {'tail, every model call' if a.harness else 'none'}\n  task     {a.task}\n")

    # The run log is written through on every line, so a killed run still leaves evidence.
    runs = o.home / "runs"
    runs.mkdir(exist_ok=True)
    stamp = datetime.now().strftime("%Y%m%d-%H%M%S")
    log_path = runs / (f"plan-{stamp}-{mode}.md" if a.plan_first else f"agent-{stamp}-{mode}.md")
    run.log.sink = lambda lines: log_path.write_text(
        f"# Agent run {stamp} — {mode} — {model_name} — LangChain host (IN PROGRESS)\n\n## Task\n\n{a.task}\n\n"
        f"## Tool calls ({run.tool_calls})\n\n" + "\n".join(lines) + "\n", encoding="utf-8")

    from langgraph.checkpoint.memory import InMemorySaver
    agent = s.agent(checkpointer=InMemorySaver())
    # LangGraph counts every node - model, tools, each middleware hook - as a step, so a
    # limit of 120 was ~30 model calls, under the harness's own ceiling. 18 Sep, task 6:
    # the model was three tutor answers deep and writing index.html when the graph stopped
    # it with GraphRecursionError. The ceiling is the model-call cap; this only has to stay
    # out of its way.
    cfg = {"configurable": {"thread_id": "sarge-run"}, "recursion_limit": o.max_model_calls * 10}
    t0 = time.monotonic()
    final_text, local_in, local_out, usage_lost = "", 0, 0, False
    try:
        result = agent.invoke({"messages": [("user", a.task)]}, config=cfg)
        # THE VERDICT IS THE RUN. 18 Sep, the Captain's rerun: the model read the file, saw
        # yesterday's endpoints already there, and said "done" without building or running
        # - and the file did not even start. The harness says so once, and gives it one
        # more turn. Once: a model that ignores this too has made its choice.
        # …and not only when it never ran: a run that failed and was never run again after
        # the fix is the same claim. 18 Sep, py4: the app did not start, the model edited,
        # fought a check, and stopped with "Verified that GET /tools responds" - no run.
        # A refusal-cancel with a green build and no run gets the same one turn: 18 Sep,
        # task 7, the code was complete and the model pressed build until the floor ended
        # it. The failure-strike stop (STUCK on a repeated failure) is final and gets nothing.
        if run.stopped and run.stopped_by_refusal and run.last_build_ok and not run.ran:
            run.allow_one_more_turn()
            run.log.append("- refusal-cancel with a green build and no run: one more turn allowed")
        if a.harness and not a.plan_first and not run.last_run_ok and not run.stopped:
            run.log.append("- NUDGED: the task ended without a run; one more turn to build and run it")
            result = agent.invoke({"messages": [("user", Sarge.NUDGE)]}, config=cfg)
        msgs = result.get("messages", [])
        final_text = next((m.content for m in reversed(msgs) if m.type == "ai" and isinstance(m.content, str) and m.content.strip()), "")
        for m in msgs:
            u = getattr(m, "usage_metadata", None) or {}
            local_in += u.get("input_tokens", 0) or 0
            local_out += u.get("output_tokens", 0) or 0
        if run.stopped:
            final_text = "STOPPED: the same failure came back four times after two answers. The run was ended on purpose."
    except Exception as ex:  # noqa: BLE001
        final_text = f"AGENT ERROR: {type(ex).__name__}: {ex}"
        usage_lost = local_in == 0 and local_out == 0
    wall = time.monotonic() - t0

    final_check, lesson = run.review()
    print(final_text)
    if final_check:
        print(f"\n-- rule checks --\n{final_check}")
    if lesson:
        print(f"\n-- the tutor --\n{lesson}")
    verdict = run.verdict
    print(f"\n── {mode} ── {wall:.0f}s · {run.tool_calls} tool calls · {run.builds} build(s) · rules held {run.rules_held} "
          f"({run.deliveries} deliveries) · {verdict}")

    run.log.sink = None
    log_path.write_text(
        f"# Agent run {stamp} — {mode} — {model_name} — LangChain host\n\n## Task\n\n{a.task}\n\n## Repo\n\n`{run.repo}`\n\n"
        f"## Tool calls ({run.tool_calls})\n\n" + "\n".join(run.log) + f"\n\n## Final build\n\n{verdict}\n\n"
        f"## Rule checks\n\n{final_check or 'not run'}\n\n## Agent said\n\n{final_text}\n\n---\n"
        f"*{model_name} · {wall:.0f}s · header {'on' if a.harness else 'off'} · rules "
        f"{('tail via middleware, ' + str(run.rules_held) + ' held, ' + str(run.deliveries) + ' deliveries') if a.harness else 'none'} · memory none · host LangChain*\n",
        encoding="utf-8")

    FRONTIER_IN, FRONTIER_OUT = 3.00, 15.00
    would = local_in / 1e6 * FRONTIER_IN + local_out / 1e6 * FRONTIER_OUT
    m = re.search(r"COST (\d+) in, (\d+) out, \$([\d.]+)", lesson or "")
    tutor_usd = float(m.group(3)) if m else 0.0
    lm = re.search(r"LEARNED\s+(\S+)", lesson or "")
    row = {"day": datetime.now().strftime("%Y-%m-%d"), "at": datetime.now().strftime("%H:%M:%S"), "task": a.task,
           "arm": mode.lower(), "model": model_name, "verdict": verdict, "tool_calls": run.tool_calls, "builds": run.builds,
           "wall_s": round(wall, 1), "local_in": local_in, "local_out": local_out, "usage_lost": usage_lost,
           "would_cost_usd": round(would, 6), "tutor_usd": round(tutor_usd, 6), "saved_usd": round(would - tutor_usd, 6),
           "lesson": lm.group(1) if lm else "", "rules_held": run.rules_held, "host": "langchain"}
    with (runs / "ledger.jsonl").open("a", encoding="utf-8") as f:
        f.write(json.dumps(row) + "\n")
    print(f"logged {log_path.relative_to(o.home)}")
    return 0 if (run.last_run_ok or run.last_build_ok) else 1


if __name__ == "__main__":
    sys.exit(main())
