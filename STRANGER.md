# Clean-room test — what a stranger does

You have cloned this repo. There are no rules for your codebase. Nothing knows it.
This is the whole of day one, and it is four steps.

## 0. What you need

- A GPU with 8 GB. The model and the check run on it. The tutor is the one thing that
  leaves: the error, the lines around it, and the files the model wrote go to Anthropic
  when it is called. Never the rest of your repo. Without a tutor key, nothing leaves.
- **The organ** — build it once from `organ/README.md`. It is llama.cpp with Sarge's core
  compiled in, and a Qwen3-4B-Instruct GGUF.
- **Python 3.11** for the page and the LangChain host.
- **The tutor.** An Anthropic key in `ANTHROPIC_API_KEY` (or a file named by
  `ANTHROPIC_KEY_FILE`). The tutor is called rarely: when the same failure comes back
  twice, and once at the end of a green task. It is handed the error, the model's own file
  around the line the error names, and at the end the files the model wrote. Never the
  rest of your repository. No file tool, no path.

## 1. Start it

```
organ\llama-src\organ.cmd        the model, on :8421
console.cmd                      the page,  on http://127.0.0.1:8420
```

For the LangChain host the page shells: `python\.venv` with `pip install -e python` in it
(`docs/GUIDE-STEPS.md`, step 8).

## 2. Give it a task on an empty folder, harness ON

On the page: folder = any empty path, arm = **HARNESS**, task in plain words, **Run**.

The folder gets its own book — a `.sarge` file at its root, git-ignored, seeded with three
universal laws. The agent writes, builds, runs, and checks every write against the book.
When the run ends green the tutor reviews the code and writes what it learned as a rule.
**Open `<your folder>\.sarge`. That is your book, and it just grew.**

## 3. Give it the next task on the same folder

The rules from step 2 are delivered at the tail of every model call — measured to be
where a small model obeys them. Watch whether it repeats yesterday's fault.

## 4. Run the same task with the harness OFF, and compare

Arm = **RAW**. Same model, same tools, no rules, no character, no tutor. Same
task, a fresh folder. The difference is the product.

## What to watch for

- **A green build is not a working change.** The verdict is WORKS only when the app started
  and answered. And WORKS is still not RIGHT — read the code. One run went green with a
  line that drops the database (`docs/THE-BIBLE.md`, Part Seven).
- **The check is the loaded model judging a line.** On the replay that ships with the tree
  it caught six of seven known faults and raised one false flag — and it passed a
  hard-coded filename once because the rule's example was written a different way. When
  it misses, add a `wrong |` line to the rule; when it flags something right, add an
  `allow`. Demonstrations are its eyes. Run `rust\replay-check.cmd` and read your own number.
- **Verify by hand.** Every result we publish was checked by a person with curl. Do the
  same before you believe a verdict — ours or anyone's.

Every known hole is in `docs/THE-BIBLE.md`, Part Six (landmines) and Part Seven (what is
proven and what is not). Read them before you trust anything.
