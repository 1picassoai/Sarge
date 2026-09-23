"""
The console: a page, and nothing else.

Vinn's ruling, 14 Sep 2026: no logic outside the Rust binary. So this file holds no
loop, no judge, no tutor, no rule book and no writing. It serves a page, shells
`compile.exe --loop --json` once per Run, and renders what comes back.

Everything that used to live in serve.py - the sandbox writer, the dotnet build, the
call to the tutor, the compare before a rule lands, the ledger row - is compiled into
the handshake. If something here starts making a decision, it belongs in the binary.

    python tools/console.py            # then open http://127.0.0.1:8420
"""

from __future__ import annotations

import argparse
import http.server
import json
import os
import re
import socketserver
import subprocess
import threading
from datetime import date
from pathlib import Path
from urllib.parse import urlparse

ROOT = Path(__file__).resolve().parent.parent
LEDGER = ROOT / "runs" / "ledger.jsonl"
CUDA_BIN = r"C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4\bin\x64"

# What the binary is given. Paths only - the binary decides what to do with them.
CORE = ROOT / "harness" / "CHARACTER.md"
BOOKS = {"harness": ROOT / "book" / "shopfloor" / "rules.jsonl",
         "raw": ROOT / "harness" / "rules.jsonl"}
SCOPES = {"harness": "shopfloor"}

MODELS = [
    "Qwen3-8B-Q4_K_M.gguf",
    "Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf",
    "Qwen3-0.6B-Q4_K_M.gguf",
    "MiniCPM5-2B-Q4_K_M.gguf",
]


def compile_exe() -> Path:
    target = Path(os.environ.get("CARGO_TARGET_DIR", str(ROOT / "rust" / "target")))
    return target / "release" / "compile.exe"


AGENT = ROOT / "agent" / "Sarge.Agent"


POINT_EXE = ROOT / "rust" / "target" / "release" / "point.exe"
POINT_RULE = "scoped-context-in-handler"
# Must match the rule's `applies` tag, or the binary drops it and measures nothing.
POINT_REPO = "myshop"


def run_pointing(task: str) -> dict:
    """THE POINTING EXPERIMENT, from the page. The Captain's ruling: he gives a task and
    presses Run - never a terminal, and no logic outside the Rust binary. So this shells
    point.exe ONCE with --arm all; the binary runs every arm, judges every handler, and
    hands back one JSON. This lifts it and nothing more."""
    if not POINT_EXE.exists():
        return {"error": f"no binary at {POINT_EXE}. Build it: rust\\build-compile.cmd"}
    env = dict(os.environ)
    if CUDA_BIN not in env.get("PATH", ""):
        env["PATH"] = CUDA_BIN + os.pathsep + env.get("PATH", "")
    cmd = [str(POINT_EXE), "--task", task, "--rule-id", POINT_RULE, "--repo", POINT_REPO,
           "--rules", str(BOOKS["harness"]), "--core", str(CORE), "--arm", "all", "--n", "900", "--json"]
    try:
        p = subprocess.run(cmd, cwd=str(POINT_EXE.parent.parent.parent), capture_output=True,
                           text=True, encoding="utf-8", errors="replace", env=env, timeout=1200)
    except subprocess.TimeoutExpired:
        return {"error": "the four arms took longer than twenty minutes and were stopped"}
    marker = "<<<JSON>>>"
    if marker not in (p.stdout or ""):
        tail = "\n".join((p.stderr or "").splitlines()[-6:])
        return {"error": f"the binary returned no result (exit {p.returncode})\n{tail}"}
    out = json.loads(p.stdout.split(marker, 1)[1])
    out["totals"] = totals()
    return out


def run(task: str, folder: str, arm: str) -> dict:
    if arm == "point":
        return run_pointing(task)
    """Drive THE AGENT. The task box is the Captain's, and it talks to the agent - the
    thing with eyes, tools and a task loop - not to the binary underneath it.

    The agent reads and writes the folder he names, asks the handshake for its laws, and
    calls the tutor when the build is red. Everything it needs is in its own log, so this
    reads that and renders it. No decision is made here."""
    if not AGENT.exists():
        return {"error": f"no agent project at {AGENT}"}
    target = Path(folder).expanduser()
    if not target.is_absolute():
        return {"error": f"give a full path, like C:\\Users\\you\\source\\repos\\myshop  (got: {folder})"}
    try:
        target.mkdir(parents=True, exist_ok=True)
    except OSError as e:
        return {"error": f"cannot use that folder: {e}"}

    # Each repo its own book (the Captain's ruling): book/<folder name>/rules.jsonl, seeded
    # from the universal laws the first time a folder is seen. The tutor grows it from
    # there. The shopfloor book stays as it is for the pointing experiment.
    # 16 Sep, the language: the book is a `.sarge` file at the repo's root - it lives with
    # the code. The universal laws (book/universal.sarge) are loaded by the handshake
    # itself under every repo book, so a new repo starts with an empty frame.
    book = target / ".sarge"
    if not book.exists():
        book.write_text("=== sarge ===\n\n=== end sarge ===\n", encoding="utf-8")
        # The Captain's ruling, 17 Sep: the file stays in the project and is GIT-IGNORED -
        # "no dev would let this be checked into the main branch." It is .env: the dev gets
        # the rules on their machine, the team never sees a machine-written file in a
        # branch. So the first time the book is created, it goes into the repo's .gitignore.
        gi = target / ".gitignore"
        lines = gi.read_text(encoding="utf-8", errors="replace").splitlines() if gi.exists() else []
        if not any(l.strip() in (".sarge", "/.sarge") for l in lines):
            with gi.open("a", encoding="utf-8") as f:
                if lines and lines[-1].strip():
                    f.write("\n")
                f.write("# Sarge's rule book for this repo - machine-written, stays on this machine\n.sarge\n")
    # Since 17 Sep the host is the MAF bridge (Sarge.AgentFramework); the command line is the
    # same, plus Picasso's store beside the ledger so every turn and every compaction is kept.
    # 17 Sep, late: the LangChain host (python/sarge) as its own pair of arms, same command
    # line, same log and ledger. The Captain's ruling that evening: LangChain is the host
    # that matters; the .NET one stays as one client.
    if arm.startswith("py-"):
        py = ROOT / "python" / ".venv" / "Scripts" / "python.exe"
        if not py.exists():
            return {"error": f"no Python venv at {py} - see docs/GUIDE-STEPS.md step 8"}
        cmd = [str(py), "-m", "sarge", "--repo", str(target), "--book", str(book), "--task", task]
        if arm == "py-harness":
            cmd.append("--harness")
        cwd = ROOT / "python"
    else:
        cmd = ["dotnet", "run", "--no-build", "--", "--repo", str(target),
               "--book", str(book), "--task", task, "--max-turns", "12",
               "--picasso", str(ROOT / "runs" / "picasso.db")]
        if arm == "harness":
            cmd.append("--harness")
        cwd = AGENT

    before = newest_log()
    try:
        p = subprocess.run(cmd, cwd=str(cwd), capture_output=True, text=True,
                           encoding="utf-8", errors="replace", timeout=1800)
    except subprocess.TimeoutExpired:
        return {"error": "the agent ran longer than thirty minutes and was stopped"}

    out = read_log(newest_log(), before)
    out["rules"] = rules_report(target, (newest_log() or Path("nul")).read_text(encoding="utf-8", errors="replace")
                                if newest_log() and newest_log() != before else "")
    out["said"] = p.stdout.strip()[-4000:] if p.stdout else ""
    # The tutor's real cost, as the provider counted it. The binary prints it; this only
    # lifts it. No estimate is made here - the number on the screen is the number billed.
    m = re.search(r"COST (\d+) in, (\d+) out, \$([\d.]+)", p.stdout or "")
    out["cost"] = ({"prompt": int(m.group(1)), "completion": int(m.group(2)),
                    "usd": float(m.group(3))} if m else None)
    out["arm"] = arm
    out["folder"] = str(target)
    out["tree"] = tree(target)
    out["totals"] = totals()
    return out


def newest_log() -> Path | None:
    logs = sorted((ROOT / "runs").glob("agent-*.md"), key=lambda f: f.stat().st_mtime)
    return logs[-1] if logs else None


def read_log(path: Path | None, before: Path | None) -> dict:
    """The agent's own log, as it wrote it. Nothing is judged here - the tool calls, the
    build colour and the rule checks are its words, lifted straight out."""
    if path is None or path == before:
        return {"calls": [], "build": None, "checks": "", "log": ""}
    text = path.read_text(encoding="utf-8", errors="replace")

    def section(name: str) -> str:
        m = re.search(rf"^## {name}[^\n]*\n(.*?)(?=\n## |\Z)", text, re.S | re.M)
        return m.group(1).strip() if m else ""

    calls = [l.strip("- ").strip() for l in section(r"Tool calls").splitlines() if l.strip()]
    build = section("Final build").strip()
    return {"calls": calls, "build": build or None, "checks": section("Rule checks"),
            "log": str(path.relative_to(ROOT)), "seconds": seconds_went(text)}


def seconds_went(text: str) -> dict:
    """Where the seconds went, from the agent's own timing lines:
        [t+  25.3s  model thought  24.0s]  run_build took 2.5s -> exit 1
    Each line is one tool call: the gap since the previous call minus the thinking is the
    tool's own time. Bucketed so a developer can see that the wait is the model, not
    Sarge - the Captain's release requirement, 17 Sep: latency must never look like ours."""
    out = {"model": 0.0, "build": 0.0, "run": 0.0, "tutor": 0.0, "sarge": 0.0, "total": 0.0}
    prev = 0.0
    for m in re.finditer(r"^\s*\[t\+\s*([\d.]+)s\s+model thought\s*([\d.]+)s\]\s*(.*)$", text, re.M):
        t, think, what = float(m.group(1)), float(m.group(2)), m.group(3).lower()
        tool = max(0.0, t - prev - think)
        prev = t
        out["model"] += think
        if what.startswith("run_build"):
            out["build"] += tool
        elif what.startswith("run_app"):
            out["run"] += tool
        elif "ask" in what:
            out["tutor"] += tool
        else:
            out["sarge"] += tool
    out["total"] = prev
    return {k: round(v, 1) for k, v in out.items()}


def book_ids(path: Path) -> list[str]:
    """Rule ids in a .sarge book. The frame is the file; `rule <id>` opens each block."""
    if not path.exists():
        return []
    return re.findall(r"^rule\s+(\S+)", path.read_text(encoding="utf-8", errors="replace"), re.M)


def book_block(path: Path, rule_id: str) -> str:
    """One rule's block, verbatim, for the page - the thing worth a screenshot."""
    if not path.exists():
        return ""
    m = re.search(rf"^rule\s+{re.escape(rule_id)}\b.*?^end\s*$", path.read_text(encoding="utf-8", errors="replace"), re.M | re.S)
    return m.group(0) if m else ""


def rules_report(folder: Path, log_text: str) -> dict:
    """What the rules did on this run, lifted from the agent's own log and the repo's
    book. Nothing is judged here: `added` is what the tutor wrote, `violated` is what the
    check hit, `held` is everything delivered that was not hit. Held means no hit was
    recorded - the check is as strong as the check is, and the page says so."""
    book = folder / ".sarge"
    universal = ROOT / "book" / "universal.sarge"
    delivered = book_ids(universal) + [i for i in book_ids(book) if i not in book_ids(universal)]
    added = re.findall(r"^- tutor -> LEARNED\s+(\S+)", log_text, re.M)
    known = re.findall(r"^- tutor -> (?:ALREADY HELD|SAME LAW\s+as)\s+(\S+)", log_text, re.M)
    hits = [{"rule": r, "where": w, "matched": m.strip()}
            for r, w, m in re.findall(r"^\s*\[([^\]]+)\]\s+(\S+)\s+matched\s+`?([^`\n]*)`?", log_text, re.M)]
    hit_ids = {h["rule"] for h in hits}
    # A rule taught on THIS run was not delivered before the run - it is new, not held.
    held = [i for i in delivered if i not in hit_ids and i not in added]
    return {"delivered": delivered, "added": added, "known": known, "held": held, "violated": hits,
            "blocks": {i: book_block(book, i) for i in added},
            "book": str(book.relative_to(folder.parent)) if book.exists() else None}


def tree(folder: Path) -> list[dict]:
    out = []
    for p in sorted(folder.rglob("*")):
        if p.is_file() and not {"bin", "obj", ".git"} & set(p.parts):
            out.append({"path": str(p.relative_to(folder)).replace("\\", "/"),
                        "bytes": p.stat().st_size})
    return out


def totals() -> dict:
    """Read-only over the ledger the agent writes. No counting decisions here - every
    figure was computed at the moment of the run and is only summed."""
    if not LEDGER.exists():
        return {"tasks": 0, "recent": []}
    rows = [json.loads(l) for l in LEDGER.read_text(encoding="utf-8").splitlines() if l.strip()]
    # Only rows the agent wrote carry a verdict. The older binary-driven rows are left
    # in the file as history but never counted - two counting bases would be two truths.
    # Only rows the agent wrote carry a verdict, and a row whose run DIED carries no
    # token count - counting it computes a negative saving and puts a false number on
    # the counter. A number with nothing under it is worse than no number.
    # Only rows the agent wrote carry a verdict. A row with NO tokens did no measurable
    # work - the run died before the framework reported usage - and counting it computes
    # a negative saving. Test the tokens, not a flag: the flag is absent on every row
    # written before it existed, so filtering on it silently did nothing.
    mine = [r for r in rows
            if r.get("verdict")
            and not r.get("usage_lost")
            and (r.get("local_in", 0) + r.get("local_out", 0)) > 0]
    today = date.today().isoformat()
    t = [r for r in mine if r.get("day") == today]
    return {
        "tasks": len(mine),
        "today_tasks": len(t),
        "saved_usd": round(sum(r.get("saved_usd", 0) for r in mine), 4),
        "today_usd": round(sum(r.get("saved_usd", 0) for r in t), 4),
        "tutor_usd": round(sum(r.get("tutor_usd", 0) for r in mine), 4),
        "tokens": sum(r.get("local_in", 0) + r.get("local_out", 0) for r in mine),
        "recent": mine[-8:][::-1],
    }


PAGE = r"""<!doctype html>
<html lang="en"><head>
<meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Sarge Console</title>
<link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;600&family=IBM+Plex+Sans:wght@400;500;600&display=swap">
<style>
  :root{--bg:#0D0F12;--panel:#14171C;--line:#242832;--text:#E8E6E1;--muted:#7A7668;
        --ok:#5E9B76;--warn:#C9A227;--bad:#C4553D;--accent:#6E93C4;
        --mono:"IBM Plex Mono",ui-monospace,Consolas,monospace;
        --sans:"IBM Plex Sans",system-ui,sans-serif}
  @media(prefers-color-scheme:light){:root{--bg:#FAFAF8;--panel:#fff;--line:#E2E0DA;
        --text:#16181C;--muted:#6B6860;--ok:#3D7A54;--warn:#96751A;--bad:#A63D28;--accent:#3A5F8F}}
  *{box-sizing:border-box}
  body{background:var(--bg);color:var(--text);font-family:var(--sans);font-size:15px;
       line-height:1.55;margin:0;padding:0 20px 8rem}
  .bar{position:fixed;left:0;right:0;bottom:0;background:var(--panel);
       border-top:1px solid var(--line);padding:.9rem 20px}
  .bar .inner{max-width:980px;margin:0 auto;display:flex;gap:2rem;flex-wrap:wrap;align-items:baseline}
  .bar .m b{font-family:var(--mono);font-size:1.35rem;font-weight:600;display:block;line-height:1.1;
            font-variant-numeric:tabular-nums}
  .bar .m span{font-family:var(--mono);font-size:9.5px;letter-spacing:.11em;text-transform:uppercase;color:var(--muted)}
  .bar .m.save b{color:var(--ok)}
  .bar .note{font-family:var(--mono);font-size:10px;color:var(--muted);margin-left:auto;max-width:30em;line-height:1.5}
  .wrap{max-width:980px;margin:0 auto}
  header{padding:2.5rem 0 1.2rem;border-bottom:1px solid var(--line);margin-bottom:1.8rem}
  .eyebrow{font-family:var(--mono);font-size:11px;letter-spacing:.16em;text-transform:uppercase;
           color:var(--muted);margin:0 0 .7rem}
  h1{font-size:1.7rem;font-weight:600;letter-spacing:-.02em;margin:0}
  .sub{color:var(--muted);margin:.5rem 0 0;font-size:14px}
  textarea{width:100%;min-height:96px;background:var(--panel);color:var(--text);
           border:1px solid var(--line);padding:.9rem 1rem;font-family:var(--sans);
           font-size:15px;line-height:1.5;resize:vertical}
  textarea:focus{outline:2px solid var(--accent);outline-offset:-1px}
  .controls{display:flex;gap:.8rem;align-items:center;flex-wrap:wrap;margin:.9rem 0 0}
  select,input[type=number]{background:var(--panel);color:var(--text);border:1px solid var(--line);
          padding:.55rem .7rem;font-family:var(--mono);font-size:12px}
  button{font-family:var(--mono);font-size:12px;letter-spacing:.08em;text-transform:uppercase;
         background:var(--accent);color:#fff;border:0;padding:.7rem 1.3rem;cursor:pointer}
  button:disabled{opacity:.5;cursor:wait}
  .hint{font-family:var(--mono);font-size:11px;color:var(--muted)}
  .row{display:flex;gap:.5rem;flex-wrap:wrap;margin:1rem 0}
  .chip{font-family:var(--mono);font-size:11px;border:1px solid var(--line);
        padding:.3rem .6rem;color:var(--muted)}
  .chip b{color:var(--text);font-weight:600}
  .chip.ok{border-color:var(--ok);color:var(--ok)}
  .chip.bad{border-color:var(--bad);color:var(--bad)}
  .chip.warn{border-color:var(--warn);color:var(--warn)}
  .rules{display:grid;gap:.5rem;margin:.5rem 0 .75rem}
  .rules>div{display:flex;flex-wrap:wrap;gap:.35rem;align-items:center}
  .rules .k{font-family:var(--mono);font-size:11px;letter-spacing:.06em;text-transform:uppercase;
    color:var(--faint,#888);min-width:15em}
  pre.rule{font-family:var(--mono);font-size:12px;line-height:1.5;border:1px solid var(--line);
    border-left:3px solid var(--ok,#3c8);padding:.75rem 1rem;margin:.5rem 0;white-space:pre;overflow-x:auto}
  .wrote{font-family:var(--mono);font-size:11px;border:1px solid var(--line);
         border-left:2px solid var(--ok);padding:.45rem .7rem;margin-bottom:.3rem;
         display:flex;gap:.7rem;align-items:baseline}
  .wrote.no{border-left-color:var(--bad)}
  .wrote .act{color:var(--muted);letter-spacing:.08em;text-transform:uppercase;font-size:9.5px}
  pre{font-family:var(--mono);font-size:12.5px;line-height:1.65;background:var(--panel);
      border:1px solid var(--line);padding:1.1rem 1.2rem;overflow-x:auto;margin:0 0 1rem;
      white-space:pre-wrap;word-break:break-word}
  .err{border:1px solid var(--bad);color:var(--bad);padding:.8rem 1rem;font-family:var(--mono);font-size:12px}
  h3{font-family:var(--mono);font-size:11px;font-weight:600;letter-spacing:.14em;text-transform:uppercase;
     color:var(--muted);margin:2rem 0 .7rem}
  table{border-collapse:collapse;width:100%;font-family:var(--mono);font-size:11.5px;font-variant-numeric:tabular-nums}
  th,td{text-align:left;padding:.4rem .7rem;border-bottom:1px solid var(--line);white-space:nowrap}
  th{color:var(--muted);font-size:10px;letter-spacing:.1em;text-transform:uppercase;font-weight:500}
</style></head><body>
<div class="wrap">

<header>
  <p class="eyebrow">Sarge · drills it until it sticks · your GPU</p>
  <h1>Give it a coding task</h1>
  <p class="sub">Write, build, teach, learn — all inside the handshake. The model and the check stay on this machine; the tutor is the one thing that leaves,
     and only when the build is red.</p>
</header>

<textarea id="task" placeholder="Create a new ASP.NET minimal API project for an online shop selling workshop tools."></textarea>

<div class="controls">
  <input type="text" id="folder" value="C:\tmp\myshop" size="34" title="the folder the agent works in - it is created if it does not exist"
         style="background:var(--panel);color:var(--text);border:1px solid var(--line);padding:.55rem .7rem;font-family:var(--mono);font-size:12px">
  <button id="go">Run</button>
  <select id="arm" title="HARNESS gets the core and the book; RAW gets neither">
    <option value="py-harness">HARNESS — core + rules</option>
    <option value="py-raw">RAW — nothing</option>
  </select>
  <span class="hint" id="hint">local model · local check · only the tutor leaves</span>
</div>

<div id="out"></div>
<div id="book"></div>

<div class="bar"><div class="inner">
  <div class="m save"><b id="t-today">$0.00</b><span>saved today</span></div>
  <div class="m"><b id="t-all">$0.00</b><span>saved all time</span></div>
  <div class="m"><b id="t-tutor">$0.00</b><span>tutor spend</span></div>
  <div class="m"><b id="t-tasks">0</b><span>tasks</span></div>
  <div class="m"><b id="t-tokens">0</b><span>tokens local</span></div>
  <div class="note" id="t-note">saving = what a frontier model would have been billed for the same work,
    minus what the tutor actually cost. Local tokens cost power, not money.</div>
</div></div>

<h3 id="recent-h" style="display:none">Recent</h3>
<table id="recent" style="display:none"><thead><tr>
  <th>time</th><th>task</th><th>arm</th><th>build</th><th>lesson</th>
</tr></thead><tbody></tbody></table>

</div>

<script>
const $ = id => document.getElementById(id);
const esc = s => (s||'').replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));

fetch('/api/state').then(r => r.json()).then(s => paint(s.totals));

function paint(t) {
  if (!t) return;
  $('t-today').textContent  = '$' + (t.today_usd || 0).toFixed(4);
  $('t-all').textContent    = '$' + (t.saved_usd || 0).toFixed(4);
  $('t-tutor').textContent  = '$' + (t.tutor_usd || 0).toFixed(4);
  $('t-tasks').textContent  = t.tasks || 0;
  $('t-tokens').textContent = (t.tokens || 0).toLocaleString();
  const rows = t.recent || [];
  if (!rows.length) return;
  $('recent-h').style.display = ''; $('recent').style.display = '';
  $('recent').querySelector('tbody').innerHTML = rows.map(r => `<tr>
    <td>${esc(r.at || '')}</td>
    <td title="${esc(r.task)}">${esc((r.task || '').slice(0, 44))}</td>
    <td>${esc(r.arm || '')}</td>
    <td><span class="chip ${(r.verdict||'')==='WORKS' ? 'ok' : r.verdict ? 'bad' : ''}">${esc(r.verdict || (r.build_ok === true ? 'GREEN' : r.build_ok === false ? 'RED' : '—'))}</span></td>
    <td>${esc(r.lesson || '—')}</td></tr>`).join('');
}

// THE RULES REPORT. What the tutor added, what was delivered and not hit, what the check
// hit - each with its rule id, and the added rule's block verbatim. This is the thing to
// screenshot. `held` means delivered with no hit recorded, nothing stronger.
function rulesBlock(r) {
  if (!r) return '';
  const list = (ids, cls) => ids.length
    ? ids.map(i => `<span class="chip ${cls}">${esc(i)}</span>`).join(' ')
    : '<span class="act">none</span>';
  const hits = (r.violated||[]).map(h => `<div class="wrote no"><b>${esc(h.rule)}</b><span class="act">${esc(h.where)} · ${esc(h.matched)}</span></div>`).join('');
  const blocks = Object.values(r.blocks||{}).filter(Boolean).map(b => `<pre class="rule">${esc(b)}</pre>`).join('');
  return `<p class="treehead">rules · this run</p>
    <div class="rules">
      <div><span class="k">added by the tutor</span>${list(r.added||[], 'ok')}</div>
      <div><span class="k">held (delivered, no hit)</span>${list(r.held||[], '')}</div>
      <div><span class="k">violated (check hit)</span>${list((r.violated||[]).map(h=>h.rule), 'bad')}</div>
    </div>${hits}${blocks}
    ${r.book ? `<p class="hint">book: ${esc(r.book)} · ${(r.delivered||[]).length} rules delivered at the tail of the task</p>` : ''}`;
}

// Between runs: the book as it stands for the folder in the box.
async function showBook() {
  const folder = $('folder').value.trim();
  if (!folder) return;
  try {
    const b = await (await fetch('/api/book?folder=' + encodeURIComponent(folder))).json();
    const n = (b.universal||[]).length + (b.repo||[]).length;
    $('book').innerHTML = n ? `<p class="treehead">the book for this folder · ${n} rules</p>
      <div class="rules"><div><span class="k">universal</span>${(b.universal||[]).map(i=>`<span class="chip">${esc(i)}</span>`).join(' ')||'<span class="act">none</span>'}</div>
      <div><span class="k">learned in this repo</span>${(b.repo||[]).map(i=>`<span class="chip ok">${esc(i)}</span>`).join(' ')||'<span class="act">none yet</span>'}</div></div>
      ${b.text ? `<pre class="rule">${esc(b.text)}</pre>` : ''}` : '';
  } catch (e) {}
}
$('folder').addEventListener('change', showBook);
showBook();

$('go').addEventListener('click', async () => {
  const task = $('task').value.trim();
  const folder = $('folder').value.trim();
  if (!task) { $('task').focus(); return; }
  if (!folder) { $('folder').focus(); return; }
  $('go').disabled = true; $('hint').textContent = 'the agent is working…';
  $('out').innerHTML = '<pre>the agent is reading, writing and building…</pre>';
  try {
    const r = await fetch('/api/run', {method:'POST', headers:{'Content-Type':'application/json'},
      body: JSON.stringify({task, folder, arm: $('arm').value})});
    render(await r.json());
  } catch (e) {
    $('out').innerHTML = '<div class="err">' + esc(String(e)) + '</div>';
  }
  $('go').disabled = false; $('hint').textContent = 'local model · local check · only the tutor leaves';
});

function render(d) {
  if (d.error) { $('out').innerHTML = '<div class="err">' + esc(d.error) + '</div>'; return; }

  // The pointing experiment: one row per arm. The binary judged; `held` is its verdict
  // and the page only colours by it.
  if (d.arms) {
    const rows = d.arms.map(a => {
      const cls = a.held === true ? 'ok' : a.held === false ? 'bad' : 'warn';
      return `<div class="wrote"><span class="act" style="min-width:4em">${esc(a.arm)}</span>`
           + `<span class="chip ${cls}">${esc(a.result)}</span>`
           + `<span class="act">handlers ${a.handlers} · clean ${a.clean} · faulty ${a.faulty}</span>`
           + (a.details.length ? `<span class="act">${esc(a.details.join(' · '))}</span>` : '')
           + `</div>`;
    }).join('');
    $('out').innerHTML = `<p class="treehead">rule ${esc(d.rule)} — same task, same model, one variable: where the rule sits</p>${rows}`
      + `<p class="hint" style="margin-top:1rem">none = no rule · head = rule at the top · tail = rule last in the user turn · kv = rule as a cached block attached right before the write</p>`;
    if (d.totals) paint(d.totals);
    return;
  }

  // The verdict is the RUN, not the build. WORKS means it started and answered.
  const works = (d.build || '').toUpperCase() === 'WORKS';
  const chips = [
    d.build ? `<span class="chip ${works ? 'ok' : 'bad'}">${esc(d.build)}</span>` : '',
    `<span class="chip">${(d.calls||[]).length} tool calls</span>`,
    d.folder ? `<span class="chip">folder <b>${esc(d.folder)}</b></span>` : '',
    d.checks ? `<span class="chip ${d.checks.startsWith('CHECKS PASSED') ? 'ok' : 'warn'}">${esc(d.checks.split('\n')[0])}</span>` : '',
    d.cost ? `<span class="chip">tutor <b>$${d.cost.usd.toFixed(5)}</b> · ${d.cost.prompt}+${d.cost.completion} tok</span>`
           : `<span class="chip ok">tutor not called · $0</span>`,
  ].join('');

  // What the agent did, in its own words, one line per tool call.
  const calls = (d.calls||[]).map(c => {
    const bad = /REJECTED|NOT FOUND|FAILED|exit 1/.test(c);
    return `<div class="wrote${bad ? ' no' : ''}">${esc(c)}</div>`;
  }).join('');

  const files = (d.tree||[]).map(f =>
    `<div class="wrote"><b>${esc(f.path)}</b><span class="act">${f.bytes} B</span></div>`).join('');

  // Where the seconds went. The wait is the model; Sarge's share is the small number.
  const s = d.seconds || {};
  const secs = s.total ? `<p class="hint">where the seconds went — model ${s.model}s · build ${s.build}s · run ${s.run}s · tutor ${s.tutor}s · <b>sarge ${s.sarge}s</b> · total ${s.total}s</p>` : '';

  $('out').innerHTML = `<div class="row">${chips}</div>` + secs
    + rulesBlock(d.rules)
    + (calls ? `<p class="treehead">what the agent did</p>${calls}` : '')
    + (files ? `<p class="treehead">files in the folder</p>${files}` : '')
    + (d.said ? `<pre>${esc(d.said)}</pre>` : '');
  if (d.totals) paint(d.totals);
}
</script></body></html>
"""


class Handler(http.server.BaseHTTPRequestHandler):
    def log_message(self, *a):
        pass

    def _send(self, code, body: bytes, ctype="application/json"):
        self.send_response(code)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_GET(self):
        path = urlparse(self.path).path
        if path in ("/", "/index.html"):
            self._send(200, PAGE.encode("utf-8"), "text/html; charset=utf-8")
        elif path == "/api/state":
            self._send(200, json.dumps({
                "models": MODELS, "totals": totals(),
                "binary": str(compile_exe()), "have_binary": compile_exe().exists(),
            }).encode())
        elif path == "/api/book":
            # The book for a folder, as it stands: every rule id and the whole file. For
            # the page's RULES section between runs, and for screenshots.
            from urllib.parse import parse_qs
            folder = Path((parse_qs(urlparse(self.path).query).get("folder") or [""])[0])
            book = folder / ".sarge"
            universal = ROOT / "book" / "universal.sarge"
            self._send(200, json.dumps({
                "folder": str(folder),
                "universal": book_ids(universal),
                "repo": book_ids(book),
                "text": book.read_text(encoding="utf-8", errors="replace") if book.exists() else "",
            }).encode())
        else:
            self._send(404, b'{"error":"not found"}')

    def do_POST(self):
        if urlparse(self.path).path != "/api/run":
            self._send(404, b'{"error":"not found"}')
            return
        n = int(self.headers.get("Content-Length", 0))
        try:
            req = json.loads(self.rfile.read(n) or b"{}")
        except Exception:
            self._send(400, b'{"error":"bad json"}')
            return
        task = (req.get("task") or "").strip()
        if not task:
            self._send(400, b'{"error":"no task"}')
            return
        # One run at a time. Two clicks 25 seconds apart put two agents on the same
        # folder, the same file and the same port - 15 Sep 21:31. The second is refused.
        if not RUN_LOCK.acquire(blocking=False):
            self._send(409, b'{"error":"a run is already going - wait for it to finish"}')
            return
        try:
            out = run(task, (req.get("folder") or str(Path.home() / "source" / "repos" / "myshop")).strip(),
                      req.get("arm") or "harness")
        except Exception as e:
            out = {"error": f"{type(e).__name__}: {e}"}
        finally:
            RUN_LOCK.release()
        self._send(200, json.dumps(out).encode())


RUN_LOCK = threading.Lock()


class Server(socketserver.ThreadingMixIn, http.server.HTTPServer):
    daemon_threads = True
    allow_reuse_address = True


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--port", type=int, default=8420)
    a = ap.parse_args()
    exe = compile_exe()
    print("Sarge console — a page, and nothing else")
    missing = "" if exe.exists() else "   <-- MISSING, run rust\\build-compile.cmd"
    print(f"  binary   {exe}{missing}")
    print(f"  ledger   {LEDGER}")
    print(f"\n  open     http://127.0.0.1:{a.port}\n")
    with Server(("127.0.0.1", a.port), Handler) as srv:
        try:
            srv.serve_forever()
        except KeyboardInterrupt:
            print("stopped")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
