#!/usr/bin/env bash
# The organ: llama-server with the core compiled in. Agents call it on :8421.
# No rules flag - the rules are in the binary. New rules = new build.
#
# The twin of organ.cmd, and every flag below is carried over unchanged because each one
# was bought with a measured failure:
#
#   --repeat-penalty 1.1 over the last 256 tokens: 17 Sep, a turn ran to the 6000-token cap
#   in 97 seconds saying the same paragraph over and over. A loop should die in seconds.
#
#   -c 32768 with the KV cache in q8: a 29-call run blew 16384 on 16 Sep (task 6). q8 halves
#   the cache's memory so 32k fits beside the model on 8GB.
#
#   -np 1: ONE slot with the whole context. The default gave four slots of 4096 each and an
#   agent turn of 8458 tokens hit "Context size has been exceeded" - the organ cancelled
#   mid-reply, the agent retried, and wrote the same bytes again. Seen 15 Sep 21:37.
#
#   -ngl 99: every layer on the accelerator. On Apple silicon that is Metal, and unified
#   memory means the model and the cache share the same pool.
set -euo pipefail

cd "$(dirname "$0")"

MODEL="${SARGE_MODEL:-$HOME/.sarge/models/Qwen3-4B-Instruct-2507-Q4_K_M.gguf}"
if [ ! -f "$MODEL" ]; then
  echo "model not found: $MODEL"
  echo "set SARGE_MODEL to your .gguf, or let install.sh download it."
  exit 1
fi

exec ./build/bin/llama-server \
  -m "$MODEL" \
  -ngl 99 -c 32768 -np 1 -ctk q8_0 -ctv q8_0 \
  --repeat-penalty 1.1 --repeat-last-n 256 \
  --port 8421 "$@"
