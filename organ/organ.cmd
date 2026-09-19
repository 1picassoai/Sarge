@echo off
REM The organ: llama-server with the core compiled in. Agents call it on :8421.
REM No rules flag - the rules are in the binary. New rules = new build.
set "CUDA_PATH=C:\Program Files\NVIDIA GPU Computing Toolkit\CUDA\v13.4"
set "PATH=%CUDA_PATH%\bin\x64;%CUDA_PATH%\bin;%PATH%"
"%~dp0build\bin\Release\llama-server.exe" ^
  -m C:\llama-b9213\models\Qwen3-4B-Instruct-2507-Q4_K_M.gguf ^
  -ngl 99 -c 32768 -np 1 -ctk q8_0 -ctv q8_0 --repeat-penalty 1.1 --repeat-last-n 256 --port 8421 %*
REM --repeat-penalty 1.1 over the last 256 tokens: 17 Sep, a turn ran to the 6000-token cap
REM in 97 seconds saying the same paragraph over and over. A loop should die in seconds.
REM -c 32768 with the KV cache in q8: a 29-call run blew 16384 on 16 Sep (task 6). q8 halves
REM the cache's memory so 32k fits beside the model on 8GB.
REM -np 1: ONE slot with the whole 16384. The default gave four slots of 4096 each and an
REM agent turn of 8458 tokens hit "Context size has been exceeded" - the organ cancelled
REM mid-reply, the agent retried, and wrote the same bytes again. Seen 15 Sep 21:37.
