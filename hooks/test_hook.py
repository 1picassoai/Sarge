"""
The hook's test, by hand, from the clone. Needs the organ on :8421 and the handshake built.

    python hooks\test_hook.py

Eight cases. Five are the loop: a bad write is a hit; a run is refused while it stands; a
harmless command is allowed; a clean write clears it; the run is allowed. Three are the
guard itself (Galahad, 18 Sep): with the hook copied outside the tree and SARGE_HOME
unset, wrong, or right, it must never report a pass while checking nothing.
"""
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
HOME = HERE.parent
FIXTURES = HOME / "rust" / "tests" / "judge"


def fire(hook: Path, repo: Path, event: str, tool: str, tool_input: dict, sarge_home):
    env = dict(os.environ)
    env.pop("SARGE_HOME", None)
    if sarge_home is not None:
        env["SARGE_HOME"] = str(sarge_home)
    data = {"hook_event_name": event, "tool_name": tool, "tool_input": tool_input, "cwd": str(repo)}
    p = subprocess.run([sys.executable, str(hook)], input=json.dumps(data), capture_output=True, text=True, env=env, cwd=str(repo))
    return p.returncode, (p.stderr or "").strip()


def main() -> int:
    tmp = Path(tempfile.mkdtemp(prefix="sarge-hook-"))
    repo = tmp / "repo"
    repo.mkdir()
    shutil.copy(FIXTURES / "node.sarge", repo / ".sarge")
    results = []

    def case(name, got, want_exit, want_text=None):
        code, err = got
        ok = code == want_exit and (want_text is None or want_text in err)
        results.append(ok)
        print(f"{'PASS' if ok else 'FAIL'}  {name}: exit {code}" + (f"  [{err[:90]}]" if err else ""))

    # ── the loop, hook in place ─────────────────────────────────────────────
    hook = HERE / "sarge_check.py"
    shutil.copy(FIXTURES / "js-hardcoded-db-bad.js", repo / "server.js")
    case("bad write is a hit", fire(hook, repo, "PostToolUse", "Write", {"file_path": str(repo / "server.js")}, HOME), 2, "RULE CHECK FAILED")
    case("run refused while the hit stands", fire(hook, repo, "PreToolUse", "Bash", {"command": "node server.js"}, HOME), 2, "REFUSED")
    case("harmless command allowed", fire(hook, repo, "PreToolUse", "Bash", {"command": "git status"}, HOME), 0)
    shutil.copy(FIXTURES / "js-clean-good.js", repo / "server.js")
    case("clean write clears the hit", fire(hook, repo, "PostToolUse", "Write", {"file_path": str(repo / "server.js")}, HOME), 0)
    case("run allowed after", fire(hook, repo, "PreToolUse", "Bash", {"command": "node server.js"}, HOME), 0)

    # ── the guard, hook copied OUTSIDE the tree ─────────────────────────────
    outside = tmp / "sarge_check.py"
    shutil.copy(hook, outside)
    shutil.copy(FIXTURES / "js-hardcoded-db-bad.js", repo / "server.js")
    case("no SARGE_HOME: loud, never a pass", fire(outside, repo, "PostToolUse", "Write", {"file_path": str(repo / "server.js")}, None), 2, "NOT CHECKING")
    case("wrong SARGE_HOME: loud, never a pass", fire(outside, repo, "PostToolUse", "Write", {"file_path": str(repo / "server.js")}, tmp), 2, "NOT CHECKING")
    case("right SARGE_HOME from outside: the hit", fire(outside, repo, "PostToolUse", "Write", {"file_path": str(repo / "server.js")}, HOME), 2, "RULE CHECK FAILED")

    shutil.rmtree(tmp, ignore_errors=True)
    print(f"\n{sum(results)} of {len(results)}")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
