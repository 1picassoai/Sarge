"""
The handshake binary: select + shape the rules for a task, check a file, ask the tutor,
learn a rule. Vinn's ruling, 14 Sep 2026: no logic outside the Rust binary. This module
holds no law of its own.

A rule violation and a broken handshake are different things and must never look the
same. Exit 1 is a real hit. Anything else - a missing book, a bad path, no binary - is OUR
failure, and it must not reach the model as "you broke a rule". It did once: the binary
could not find its book, exited 2, and a correct edit was bounced back as a violation.
"""
from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Optional, Tuple


def run(home: Path, book: Optional[Path], *args: str) -> Tuple[bool, str]:
    try:
        exe = home / "rust" / "target" / "release" / "handshake.exe"
        if not exe.is_file():
            return True, f"(handshake binary not built: {exe})"
        cmd = [str(exe), *args]
        if book is not None:
            cmd += ["--rules", str(book)]
        p = subprocess.run(cmd, cwd=str(home), capture_output=True, text=True,
                           encoding="utf-8", errors="replace", timeout=60)
        text = (p.stdout or "").strip()
        if p.returncode == 0:
            return True, text
        # Exit 1 is a rule hit - UNLESS nothing actually ran. On an empty repo every rule's
        # glob matches no file, and "checks incomplete" was once reported to the model as
        # "you broke a rule" on its very first write. Incomplete is not a violation.
        if p.returncode == 1:
            return ("CHECKS INCOMPLETE" in text), text
        return True, f"(handshake error, not a rule violation: {(p.stderr or '').strip()})"
    except Exception as ex:  # noqa: BLE001
        return True, f"(handshake unavailable: {ex})"
