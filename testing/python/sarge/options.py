"""Everything Sarge needs to run one repository. Defaults are the working ones."""
from __future__ import annotations

import os
from dataclasses import dataclass, field
from pathlib import Path
from typing import Callable, Optional


def _find_home() -> Path:
    env = os.environ.get("SARGE_HOME")
    if env and Path(env).is_dir():
        return Path(env).resolve()
    # Walk up from this file: python/sarge/options.py sits inside the Sarge clone.
    d = Path(__file__).resolve().parent
    while d != d.parent:
        if (d / "README.md").is_file() and (d / "rust").is_dir():
            return d
        d = d.parent
    raise RuntimeError(
        "Sarge home not found. Set SARGE_HOME to your Sarge clone (the folder with rust\\, "
        "harness\\ and book\\), or pass SargeOptions(sarge_home=...)."
    )


def _vpn_up() -> bool:
    """The work VPN (GlobalProtect) shows as a PANGP adapter. If it is there, nothing outbound."""
    import subprocess
    try:
        out = subprocess.run(["ipconfig", "/all"], capture_output=True, text=True, timeout=15).stdout
        return "PANGP" in out
    except Exception:
        return False


@dataclass
class SargeOptions:
    """
    repo_dir     the repository the agent works in; every path the model touches stays inside it
    sarge_home   the Sarge clone: rust/target/release/handshake (.exe on Windows), harness/CHARACTER.md, book/universal.sarge
                 default: SARGE_HOME, else the clone this package sits in
    book_path    the repo's rule book; default <repo>\\.sarge
    organ_url    llama.cpp with the handshake compiled in, OpenAI-shaped
    harness      HARNESS (rules, character, check, tutor) or RAW (bare model, nothing)
    plan_first   list and read only, then a ten-line plan
    max_output_tokens  a whole file arrives as one tool-call argument; 3000 fits a Program.cs and caps a runaway
    outbound_blocked   returns True when nothing may leave this machine (default: the PANGP check)
    """
    repo_dir: str
    sarge_home: Optional[str] = None
    book_path: Optional[str] = None
    organ_url: str = "http://127.0.0.1:8421/v1"
    model_id: str = "qwen"
    harness: bool = True
    plan_first: bool = False
    review_path: Optional[str] = None
    max_output_tokens: int = 5000  # a whole server.js of ~5k chars is ~3k tokens once JSON-escaped; 3000 cut one off mid-string (18 Sep, task 7). The organ's repeat penalty and the refusal floors hold the runaway case now.
    max_model_calls: int = 60      # the ceiling under the four-strike stop; the graph ends, it does not raise. (A React client task ran to ~30 calls with real progress - 18 Sep.)
    temperature: float = 0.1
    outbound_blocked: Callable[[], bool] = field(default=_vpn_up)

    @property
    def repo(self) -> Path:
        return Path(self.repo_dir).resolve()

    @property
    def home(self) -> Path:
        return Path(self.sarge_home).resolve() if self.sarge_home else _find_home()

    @property
    def book(self) -> Path:
        return Path(self.book_path).resolve() if self.book_path else self.repo / ".sarge"
