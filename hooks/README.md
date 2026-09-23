# Sarge for Claude Code

Claude Code writes the code. Sarge checks every file it writes against your repo's rules,
using your own local model, and refuses to let the app run while a rule is broken. No key,
no network, nothing leaves your machine.

## What you get

- **The check on every write.** After `Write`, `Edit` or `MultiEdit`, the organ judges the
  file against `.sarge` and the universal laws. A hit comes back to Claude as the tool's
  own feedback: the rule, the line, and what the rule says instead. Claude fixes it.
- **The run refuses.** While a hit stands, `dotnet run`, `npm start`, `npm run`, `node`,
  `python` and friends are blocked with the same message. Fix first, run after.
- **Your rules, in your words.** `.sarge` at the repo root, git-ignored. Claude Code itself
  is the tutor here: when it gets something wrong twice, ask it to write the rule in the
  Sarge language, with a `wrong |` line and a `right |` line. Copy `book/universal.sarge`
  to start.
  From then on the check has eyes for it.

## Install

1. Build the organ and the handshake (`organ/README.md`). Start the organ.
2. Set `SARGE_HOME` to your Sarge clone.
3. Put a `.sarge` at your repo's root. Start from `book/universal.sarge`'s three laws or an
   empty frame:
   ```
   === sarge ===

   === end sarge ===
   ```
   and add `.sarge` to `.gitignore`.
4. Merge `hooks/settings.example.json` into your project's `.claude/settings.json`.

That is all. Open Claude Code in the repo and work as you do.

## What it does not do

- It does not run the tutor loop. Claude Code is the frontier model already; you ask it.
- It does not check files outside the source extensions (`.cs .js .mjs .cjs .jsx .ts .tsx .py`).
- It is only as good as the loaded model's judgement of a line. When it misses, add a
  `wrong |` line to the rule. Demonstrations are its eyes.

## When it cannot check

If `SARGE_HOME` is unset or wrong, the handshake is not built, there is no `.sarge`, or
the organ is down, **the hook says so on every write, loudly, with the reason** (exit 2,
the one exit Claude Code shows to the model). It never reports a pass while checking
nothing. The release review caught the first version doing exactly that, 18 Sep; the fix is this rule.

## Tested

`python hooks\test_hook.py` — eight cases, from the clone, with the organ up:

- the loop (5): a bad write is a hit with the rule and the line · a run is refused while
  it stands · a harmless command is allowed · a clean write clears it · the run is allowed
- the guard (3): the hook copied *outside* the tree with `SARGE_HOME` unset → loud, never
  a pass · with `SARGE_HOME` wrong → loud, never a pass · with it right → the hit

Last run 18 Sep 2026, 8 of 8. Not yet tested from inside a live Claude Code session by a
stranger — tell us.
