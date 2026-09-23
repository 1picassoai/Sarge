"""
Sarge for Claude Code - the check as a hook.

Two hook events, one script:

  PostToolUse on Write | Edit | MultiEdit
      The file Claude Code just wrote is judged by the organ against the repo's .sarge
      (and the universal laws). A hit is printed on stderr with exit 2, so Claude sees it
      as the tool's own feedback and fixes it. The hit is also left in <repo>/.sarge-hit.

  PreToolUse on Bash
      While .sarge-hit stands, a command that would RUN the app (dotnet run, npm start,
      npm run, node, python) is refused with exit 2 and the hit. The run refuses while a
      rule is broken - same law as the harness.

  When the check CANNOT run (no SARGE_HOME, handshake not built, no .sarge, organ down)
  the write stands and Claude is told so, loudly, on every write - exit 2 with the reason.
  A guard that cannot check never reports a pass.

Needs: the organ running on :8421, SARGE_HOME pointing at your Sarge clone (with
rust/target/release/handshake built, .exe on Windows), and a .sarge at the repo root. No key needed: the judge is a model on your own machine. SARGE_ORGAN can point it at
another local model; anything that is not loopback is refused unless
SARGE_ORGAN_ALLOW_REMOTE=1 is set as well, so your source cannot leave by accident.

Install: copy hooks/settings.example.json into your project's .claude/settings.json
(or merge the "hooks" block into the one you have).
"""
from __future__ import annotations

import json
import os
import re
import subprocess
import sys
from pathlib import Path

RUN_WORDS = ("dotnet run", "npm start", "npm run", "node ", "python ", "python3 ", "uvicorn", "flask run")
SOURCE_EXT = (".cs", ".js", ".mjs", ".cjs", ".jsx", ".ts", ".tsx", ".py")


def home() -> Path | None:
    env = os.environ.get("SARGE_HOME")
    if env and Path(env).is_dir():
        return Path(env)
    d = Path(__file__).resolve().parent
    while d != d.parent:
        if (d / "rust").is_dir() and (d / "README.md").is_file():
            return d
        d = d.parent
    return None


def repo_root(start: Path) -> Path:
    """The nearest ancestor holding a .sarge, else a .git, else the cwd itself."""
    d = start if start.is_dir() else start.parent
    for cand in [d, *d.parents]:
        if (cand / ".sarge").is_file():
            return cand
    for cand in [d, *d.parents]:
        if (cand / ".git").exists():
            return cand
    return d


class NotChecked(Exception):
    """The file was NOT judged, and the reason. Never a pass.

    The release review, 18 Sep, refused to sign: every failure path here used to return "ok" with an
    explanatory string that nothing printed - a stranger with SARGE_HOME wrong saw a clean
    run forever. A guard that cannot check must say so where Claude can read it."""


def check(file_path: Path, cwd: Path) -> tuple[bool, str]:
    h = home()
    if h is None:
        raise NotChecked("SARGE_HOME is not set and no Sarge clone was found above this script. Set SARGE_HOME to your clone.")
    exe = h / "rust" / "target" / "release" / ("handshake.exe" if os.name == "nt" else "handshake")
    if not exe.is_file():
        raise NotChecked(f"the handshake is not built: {exe} is missing. Run `cargo build --release` in {h / 'rust'}.")
    repo = repo_root(file_path)
    book = repo / ".sarge"
    if not book.is_file():
        raise NotChecked(f"no .sarge at {repo}. Put a rule book at the repo root (see docs/SARGE-SYNTAX.md).")
    rel = os.path.relpath(file_path, repo)
    cmd = [str(exe), "--check", str(repo), "--file", rel, "--repo", repo.name, "--task", "claude-code", "--rules", str(book)]
    try:
        p = subprocess.run(cmd, cwd=str(h), capture_output=True, text=True, encoding="utf-8", errors="replace", timeout=120)
    except Exception as ex:  # noqa: BLE001
        raise NotChecked(f"the handshake could not run: {ex}") from ex
    out = (p.stdout or "").strip()
    if p.returncode == 0:
        return True, out
    if p.returncode == 1 and "CHECKS INCOMPLETE" not in out:
        return False, out
    if "UNAVAILABLE" in out or p.returncode not in (0, 1):
        raise NotChecked(f"the organ did not answer (is it running on :8421?): {out[:200] or (p.stderr or '').strip()[:200]}")
    return True, out


def main() -> int:
    try:
        data = json.load(sys.stdin)
    except Exception:  # noqa: BLE001
        return 0
    event = data.get("hook_event_name", "")
    tool = data.get("tool_name", "")
    cwd = Path(data.get("cwd") or os.getcwd())

    if event == "PostToolUse" and tool in ("Write", "Edit", "MultiEdit"):
        fp = (data.get("tool_input") or {}).get("file_path")
        if not fp:
            return 0
        file_path = Path(fp)
        if file_path.suffix.lower() not in SOURCE_EXT:
            return 0
        try:
            ok, report = check(file_path, cwd)
        except NotChecked as why:
            # Fail open, but LOUD: the write stands (the tool already ran), and Claude is
            # told, on every write, that nothing is being checked and why. Exit 2 is the
            # only exit Claude Code shows to the model.
            print(f"SARGE IS NOT CHECKING THIS FILE - {why}. Until this is fixed, no rule is enforced.", file=sys.stderr)
            return 2
        hit_file = repo_root(file_path) / ".sarge-hit"
        if ok:
            try:
                hit_file.unlink()
            except FileNotFoundError:
                pass
            return 0
        hit_file.write_text(report, encoding="utf-8")
        print("SARGE - A RULE CHECK FAILED on the file you just wrote. The app will not run until this is fixed:\n" + report, file=sys.stderr)
        return 2

    if event == "PreToolUse" and tool == "Bash":
        command = (data.get("tool_input") or {}).get("command", "")
        if not any(w in command for w in RUN_WORDS):
            return 0
        hit_file = repo_root(cwd) / ".sarge-hit"
        if not hit_file.is_file():
            return 0
        print("SARGE - REFUSED. A rule check is still failing and the app does not run until it passes. Fix exactly this:\n"
              + hit_file.read_text(encoding="utf-8"), file=sys.stderr)
        return 2

    return 0


if __name__ == "__main__":
    sys.exit(main())
