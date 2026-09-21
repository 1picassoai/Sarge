#!/usr/bin/env bash
# Sarge, in one command, on macOS (Apple silicon).
#
#   curl -fsSL https://raw.githubusercontent.com/1picassoai/Sarge/main/install.sh | bash
#
# The twin of install.ps1, step for step, and every guard in it was bought with a real
# failure on the Windows side. Nothing is installed system-wide: the tree, the organ, the
# model and the start scripts all live in one folder you can delete.
set -uo pipefail

REPO="https://github.com/1picassoai/Sarge"
TAG="v0.1.1"                       # pins the TREE and the ORGAN ASSET together
MODEL="Qwen3-4B-Instruct-2507-Q4_K_M.gguf"
MODEL_URL="https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF/resolve/main/$MODEL"
ORGAN_ASSET="sarge-organ-macos-arm64.zip"
NEED_GB=6

# ------------------------------------------------------------------ voice
if [ -t 1 ]; then
  C_CY=$'\033[36m'; C_GR=$'\033[32m'; C_YE=$'\033[33m'; C_RE=$'\033[31m'
  C_DG=$'\033[90m'; C_WH=$'\033[97m'; C_OR=$'\033[33m'; C_NC=$'\033[0m'
else
  C_CY=""; C_GR=""; C_YE=""; C_RE=""; C_DG=""; C_WH=""; C_OR=""; C_NC=""
fi
say()  { echo "  $*"; }
step() { echo; echo "${C_CY}== $*${C_NC}"; }
good() { echo "  ${C_GR}OK${C_NC}  $*"; }
warn() { echo "  ${C_YE}!${C_NC}   $*"; }
die()  { echo; echo "${C_RE}STOPPED: $*${C_NC}"; echo; exit 1; }

# A progress bar, and silence when nothing is watching - a pipe or a CI log does not want
# 4,000 redraws.
get_file() {
  url="$1"; dest="$2"; what="$3"
  say "downloading $what"
  if [ -t 1 ]; then
    curl -fL --progress-bar "$url" -o "$dest" || die "download failed: $what"
  else
    curl -fsSL "$url" -o "$dest" || die "download failed: $what"
  fi
  [ -s "$dest" ] || die "downloaded nothing: $what"
}

echo
for l in \
'   ____                    ' \
'  / ___|  __ _ _ __ __ _  ___ ' \
'  \___ \ / _` | |__/ _` |/ _ \' \
'   ___) | (_| | | | (_| |  __/' \
'  |____/ \__,_|_|  \__, |\___|' \
'                   |___/      '; do
  echo "${C_OR}${l}${C_NC}"; sleep 0.06
done
echo
echo "  ${C_WH}Prompts negotiate. Sarge doesn't.${C_NC}"; sleep 0.2
echo "  ${C_DG}About 2.4 GB to download, once. No GPU required.${C_NC}"
echo

# ------------------------------------------------------------------ 1. the machine
step "checking the machine"

[ "$(uname -s)" = "Darwin" ] || die "this installer is for macOS. On Windows use install.ps1."
ARCH="$(uname -m)"
if [ "$ARCH" != "arm64" ]; then
  die "the published organ is built for Apple silicon (arm64); this machine is $ARCH.
  Intel Macs have to build the organ from source - see organ/README.md."
fi
good "macOS on Apple silicon ($ARCH)"

PY=""
for c in python3 python; do
  if command -v "$c" >/dev/null 2>&1; then
    v="$("$c" --version 2>&1)"
    maj="$(echo "$v" | sed -n 's/Python \([0-9]*\)\.\([0-9]*\).*/\1/p')"
    min="$(echo "$v" | sed -n 's/Python \([0-9]*\)\.\([0-9]*\).*/\2/p')"
    if [ -n "$maj" ] && [ "$maj" -ge 3 ] && [ "$min" -ge 10 ]; then PY="$c"; break; fi
  fi
done
[ -n "$PY" ] || die "Python 3.10 or newer is needed and was not found on PATH.
  brew install python   (or https://www.python.org/downloads/)"
good "Python: $($PY --version 2>&1)"

command -v git >/dev/null 2>&1 || die "git is needed to fetch Sarge.
  xcode-select --install"

FREE_GB="$(df -g . 2>/dev/null | awk 'NR==2 {print $4}')"
if [ -n "${FREE_GB:-}" ]; then
  [ "$FREE_GB" -ge "$NEED_GB" ] || die "$FREE_GB GB free here - about $NEED_GB GB is needed."
  good "disk: $FREE_GB GB free"
fi

# Metal is not optional on Apple silicon and not installable either - it ships with the OS.
# What varies is memory, and unified memory means the model and its cache share it.
MEM_GB=$(( $(sysctl -n hw.memsize) / 1073741824 ))
CORES="$(sysctl -n hw.ncpu)"
if [ "$MEM_GB" -lt 8 ]; then
  warn "$MEM_GB GB of memory - Sarge wants 8 GB. It may be slow or fail to load the model."
else
  good "$MEM_GB GB memory, $CORES cores - Metal will be used"
fi

# ------------------------------------------------------------------ 2. the tree
step "the tree"

HERE="$PWD"
if [ ! -f "$HERE/rust/Cargo.toml" ]; then
  HERE="$PWD/Sarge"
  if [ ! -f "$HERE/rust/Cargo.toml" ]; then
    say "cloning $TAG into $HERE ..."
    git clone --depth 1 --branch "$TAG" "$REPO" "$HERE" >/dev/null 2>&1 \
      || die "the clone failed. Check the network and that $TAG exists."
    [ -f "$HERE/rust/Cargo.toml" ] || die "the clone did not produce a tree at $HERE"
    good "cloned $TAG (the signed release; main is work in progress)"
  else
    good "using the clone at $HERE"
  fi
else
  good "running inside the clone at $HERE"
fi

BIN="$HERE/organ/bin"
MODELS="$HERE/models"
RUSTD="$HERE/rust/target/release"
mkdir -p "$BIN" "$MODELS" "$RUSTD"

# ------------------------------------------------------------------ 3. the organ
step "the organ  (llama.cpp with Sarge compiled in)"

SERVER="$BIN/llama-server"
if [ -x "$SERVER" ]; then
  good "organ already here"
else
  ZIP="$(mktemp -t sarge-organ).zip"
  get_file "$REPO/releases/download/$TAG/$ORGAN_ASSET" "$ZIP" "the organ (20 MB)"
  unzip -oq "$ZIP" -d "$BIN" || die "could not unpack the organ archive."
  if [ ! -x "$SERVER" ]; then
    found="$(find "$BIN" -name llama-server -type f | head -1)"
    [ -n "$found" ] && mv "$(dirname "$found")"/* "$BIN"/ 2>/dev/null || true
  fi
  [ -f "$SERVER" ] || die "llama-server was not in the archive."
  chmod +x "$SERVER"
  rm -f "$ZIP"
  # macOS quarantines anything downloaded. Without this the first run is a Gatekeeper
  # dialog per dylib, which reads like the install is broken.
  xattr -dr com.apple.quarantine "$BIN" 2>/dev/null || true
  good "organ unpacked (and un-quarantined)"
fi

HS="$RUSTD/handshake"
if [ ! -x "$HS" ]; then
  if [ -f "$BIN/handshake" ]; then
    cp "$BIN/handshake" "$HS"; chmod +x "$HS"
    good "handshake taken from the release"
  elif command -v cargo >/dev/null 2>&1; then
    say "building the handshake with cargo (about three minutes) ..."
    ( cd "$HERE/rust" && cargo build --release --lib --bin handshake >/dev/null 2>&1 )
    [ -x "$HS" ] || die "cargo did not produce a handshake binary."
    good "handshake built"
  else
    die "no handshake in the release archive and no Rust toolchain to build one.
  Install Rust (https://rustup.rs) and run this again."
  fi
else
  good "handshake already here"
fi

# ------------------------------------------------------------------ 4. the model
step "the model  (Qwen3-4B, 2.4 GB - the long part)"

MODEL_PATH=""
for c in "$MODELS/$MODEL" "$HOME/.sarge/models/$MODEL"; do
  [ -f "$c" ] && { MODEL_PATH="$c"; break; }
done
if [ -n "$MODEL_PATH" ]; then
  good "model already here: $MODEL_PATH"
else
  MODEL_PATH="$MODELS/$MODEL"
  say "2.4 GB, and it only happens once"
  get_file "$MODEL_URL" "$MODEL_PATH" "the model"
  good "model downloaded"
fi

# ------------------------------------------------------------------ 5. the harness
step "the Python harness"

( cd "$HERE" && "$PY" -m pip install --disable-pip-version-check -q -e python 2>&1 | grep -iE "ERROR|error:" || true )
"$PY" -c "import sarge" >/dev/null 2>&1 \
  || die "the Python package did not import after install.
  If pip refused with 'externally-managed-environment', make a venv first:
    python3 -m venv .venv && source .venv/bin/activate, then run this again."
good "sarge installed"

# ------------------------------------------------------------------ 6. your paths
step "start scripts, with your paths"

cat > "$HERE/start-organ.sh" <<EOF
#!/usr/bin/env bash
# written by install.sh on $(date '+%Y-%m-%d %H:%M')
exec "$SERVER" \\
  -m "$MODEL_PATH" \\
  -ngl 99 -c 32768 -np 1 -ctk q8_0 -ctv q8_0 \\
  --repeat-penalty 1.1 --repeat-last-n 256 --port 8421 "\$@"
EOF
chmod +x "$HERE/start-organ.sh"
good "start-organ.sh"

cat > "$HERE/start-console.sh" <<EOF
#!/usr/bin/env bash
# written by install.sh
export SARGE_HOME="$HERE"
cd "$HERE"
exec $PY tools/console.py "\$@"
EOF
chmod +x "$HERE/start-console.sh"
good "start-console.sh"

export SARGE_HOME="$HERE"
say "SARGE_HOME=$HERE  (add it to your shell profile to keep it)"

# ------------------------------------------------------------------ 7. prove it
step "proving it works"

# Do not start a second organ. The release review, 20 Sep: four were left running on one
# machine holding 10.8 GB between them, all fighting over :8421, and the winner changed
# three times while it was being watched. A stranger who reruns this after a failure -
# which is exactly what people do - would do the same to themselves.
UP=0
if curl -fsS --max-time 3 http://127.0.0.1:8421/health >/dev/null 2>&1; then
  warn "an organ is already answering on :8421 - using it, not starting another"
  say  "if it is running a different model, stop it first and run this again"
  UP=1
else
  "$HERE/start-organ.sh" >"$HERE/organ.log" 2>&1 &
  ORGAN_PID=$!
  frames='|/-\'
  for i in $(seq 1 240); do
    [ -t 1 ] && printf "\r  %s loading the model into memory ..." "${frames:$((i % 4)):1}"
    sleep 0.9
    if curl -fsS --max-time 2 http://127.0.0.1:8421/health >/dev/null 2>&1; then UP=1; break; fi
    if ! kill -0 "$ORGAN_PID" 2>/dev/null; then
      [ -t 1 ] && printf "\r%44s\r" ""
      die "the organ exited while loading. The last lines of organ.log:
$(tail -12 "$HERE/organ.log" 2>/dev/null)"
    fi
  done
  [ -t 1 ] && printf "\r%44s\r" ""
fi
[ "$UP" = "1" ] || die "the organ did not answer on :8421 within four minutes.
  Run ./start-organ.sh yourself and read organ.log."
good "organ answering on :8421"

FIX="$HERE/rust/tests/judge"
VERDICT="$("$HS" --check "$FIX" --file js-await-sync-bad.js --repo judge --rules "$FIX/node.sarge" 2>&1 || true)"
if echo "$VERDICT" | grep -q "CHECKS FAILED"; then
  good "the check caught a known-bad file - the instrument goes red when it should"
else
  echo "  ${C_RE}the check did NOT catch a file it is known to catch.${C_NC}"
  echo "  ${C_RE}Do not trust a pass from it until this is fixed.${C_NC}"
  die "self-test failed."
fi

cat <<EOF

  ${C_WH}DONE.${C_NC}

    ./start-organ.sh      the model, on :8421   (leave it running)
    ./start-console.sh    the page, on :8420    - a folder, a task, Run

  Optional - the tutor is the one thing that leaves your machine:
    export ANTHROPIC_API_KEY=sk-ant-...

  What broke, what it got wrong, what you wish it caught:
    $REPO/discussions

EOF
