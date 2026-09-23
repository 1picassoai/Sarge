#!/usr/bin/env bash
# Sarge, in one command, on Linux (x86_64).
#
#   curl -fsSL https://raw.githubusercontent.com/1picassoai/Sarge/main/install-linux.sh | bash
#
# The twin of install.ps1, step for step, and every guard in it was bought with a real
# failure on the Windows side. Nothing is installed system-wide: the tree, the organ, the
# model and the start scripts all live in one folder you can delete.
set -uo pipefail

# ─────────────────────────────────────────────────────────────────────────────
# NOT RELEASED. This installer does not work yet and refuses to run.
#
# @Galahad's blocker, 23 Sep 2026, and he found a third fault neither of us had:
#
#   1. TAG pins v0.2.0, not the release this would ship in.
#   2. ORGAN_ASSET names sarge-organ-linux-x64-cpu.zip, which exists on NO release -
#      no Linux organ has ever been built. build-linux.yml fires on push to main and
#      nothing has been pushed.
#   3. ORGAN_SHA256 is the macOS organ's hash, copy-pasted with the rest of the file.
#      So even AFTER a Linux organ is built and published, this would download it,
#      compare it against the Mac hash, refuse it and delete it - the verification
#      gate working exactly as designed, on a pin that was never corrected.
#
# Fixing the build alone would not make this work. Build first, then the pins.
#
# It stays in the tree because the work is real and nearly done, and it refuses
# because an installer that points at a missing asset with a wrong checksum is worse
# than no installer at all. Shipping it as-is would be the v0.1.1-rc1 class a third
# time. Delete this block when the three pins above are right AND a Linux organ has
# been built, published and walked end to end on a real Linux box.
# ─────────────────────────────────────────────────────────────────────────────
echo "install-linux.sh is NOT RELEASED and does not work yet." >&2
echo "No Linux organ has been built. Use install.sh on macOS, or install.ps1 on Windows." >&2
echo "Follow ${REPO:-https://github.com/1picassoai/Sarge}/discussions for when Linux lands." >&2
exit 1


REPO="https://github.com/1picassoai/Sarge"
TAG="v0.2.0"                       # pins the TREE and the ORGAN ASSET together
MODEL="Qwen3-4B-Instruct-2507-Q4_K_M.gguf"
MODEL_URL="https://huggingface.co/unsloth/Qwen3-4B-Instruct-2507-GGUF/resolve/main/$MODEL"
ORGAN_ASSET="sarge-organ-linux-x64-cpu.zip"
NEED_GB=6

# EVERY DOWNLOAD IS CHECKED. Added 21 Sep after @Galahad's release review found that
# neither installer verified a single byte it fetched - while a comment claimed it
# "stops loudly on anything it cannot verify". It did not.
#
# This matters because the installer runs a binary it just downloaded. That
# argument only holds if we can SHOW it is ours. Now we can.
#
# The organ hash is the artefact published on the release. The model hash is Hugging
# Face's own X-Linked-ETag for that file, verified 21 Sep against the copy on the
# Captain's disk - the same 2,497,281,120 bytes.
ORGAN_SHA256="d0fdf88e5c4bb9467f3dd3a31ba1f3943eb1e915b9eb0e802b180dedd8d8bb61"
MODEL_SHA256="3605803b982cb64aead44f6c1b2ae36e3acdb41d8e46c8a94c6533bc4c67e597"

# A hash that does not match is not a warning. The file is deleted and the install stops:
# a half-trusted binary on disk is worse than no binary, because the next run would find
# it already there and skip the download.
verify_sha256() {
  path="$1"; want="$2"; what="$3"
  got="$(shasum -a 256 "$path" 2>/dev/null | awk '{print $1}')"
  if [ -z "$got" ]; then
    rm -f "$path"
    die "could not hash $what - shasum is missing, and nothing unverified is installed."
  fi
  if [ "$got" != "$want" ]; then
    rm -f "$path"
    die "$what does not match what we published, and has been deleted.
  expected  $want
  got       $got
  Do not run anything that was downloaded. Tell us: $REPO/discussions"
  fi
  good "$what verified (sha256 $(echo "$want" | cut -c1-16)...)"
}

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

[ "$(uname -s)" = "Linux" ] || die "this installer is for Linux. On macOS use install.sh, on Windows install.ps1."
ARCH="$(uname -m)"
if [ "$ARCH" != "x86_64" ]; then
  die "the published organ is built for x86_64; this machine is $ARCH.
  Build the organ from source - see organ/README.md."
fi
good "Linux on $ARCH"

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
  apt install python3   (or dnf install python3)"
good "Python: $($PY --version 2>&1)"

command -v git >/dev/null 2>&1 || die "git is needed to fetch Sarge.
  apt install git   (or dnf install git)"

FREE_GB="$(df -BG . 2>/dev/null | awk 'NR==2 {gsub(/G/,"",$4); print $4}')"
if [ -n "${FREE_GB:-}" ]; then
  [ "$FREE_GB" -ge "$NEED_GB" ] || die "$FREE_GB GB free here - about $NEED_GB GB is needed."
  good "disk: $FREE_GB GB free"
fi

# The published Linux organ is a CPU build. GitHub's hosted Linux runners have no GPU, so
# a CUDA organ would be something CI compiled and never ran - it is not published and not
# claimed. A machine with a card can build one; see organ/README.md.
MEM_GB=$(( $(awk '/MemTotal/ {print $2}' /proc/meminfo) / 1048576 ))
CORES="$(nproc)"
if [ "$MEM_GB" -lt 8 ]; then
  warn "$MEM_GB GB of memory - Sarge wants 8 GB. It may be slow or fail to load the model."
else
  good "$MEM_GB GB memory, $CORES cores - the organ runs on the CPU"
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
  # Verified BEFORE it is unpacked.
  verify_sha256 "$ZIP" "$ORGAN_SHA256" "the organ"
  unzip -oq "$ZIP" -d "$BIN" || die "could not unpack the organ archive."
  if [ ! -x "$SERVER" ]; then
    found="$(find "$BIN" -name llama-server -type f | head -1)"
    [ -n "$found" ] && mv "$(dirname "$found")"/* "$BIN"/ 2>/dev/null || true
  fi
  [ -f "$SERVER" ] || die "llama-server was not in the archive."
  chmod +x "$SERVER"
  rm -f "$ZIP"
  good "organ unpacked"
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
  say "checking the model (2.4 GB, this takes a few seconds)"
  verify_sha256 "$MODEL_PATH" "$MODEL_SHA256" "the model"
  good "model downloaded"
fi

# ------------------------------------------------------------------ 6. your paths
step "start scripts, with your paths"

# The published Linux organ is a CPU build, so no layers go to a GPU - asking for 99 here
# would spend the startup looking for a device that is not there. A machine with a card
# and a CUDA organ built from source overrides this with SARGE_NGL.
NGL="${SARGE_NGL:-0}"

cat > "$HERE/start-organ.sh" <<EOF
#!/usr/bin/env bash
# written by install-linux.sh on $(date '+%Y-%m-%d %H:%M')
exec "$SERVER" \\
  -m "$MODEL_PATH" \\
  -ngl ${NGL} -c 32768 -np 1 -ctk q8_0 -ctv q8_0 \\
  --repeat-penalty 1.1 --repeat-last-n 256 --port 8421 "\$@"
EOF
chmod +x "$HERE/start-organ.sh"
good "start-organ.sh"

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
  # Eight minutes, not four. On a Mac with Metal the model is ready in seconds; without a
  # GPU - a CI runner, a VM, an older machine - loading a 2.4GB model on a few cores is
  # genuinely slow, and the first run is the worst one. Measured on GitHub's 3-core
  # runner, 21 Sep: still loading at four minutes with no error in the log.
  for i in $(seq 1 480); do
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
[ "$UP" = "1" ] || die "the organ did not answer on :8421 within eight minutes.
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

  Now wire it into Claude Code, in the repo you want guarded:
    cp hooks/settings.example.json  ->  .claude/settings.json
    and put a .sarge at that repo's root (start from book/universal.sarge)

  From then on every file Claude Code writes is judged against your rules,
  and the run refuses while a rule is broken.

  Optional - the tutor is the one thing that leaves your machine:
    export ANTHROPIC_API_KEY=sk-ant-...

  What broke, what it got wrong, what you wish it caught:
    $REPO/discussions

EOF
