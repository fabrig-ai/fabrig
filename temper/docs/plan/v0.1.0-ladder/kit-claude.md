# The walk kit, Claude: what changes from kit.md when the model is Claude Code

Read `kit.md` first; this file replaces its sections "The model, and its login's move in and
out" and "The box's configuration" for a walk on **Claude with an owned Forgejo**
(`temper/docs/design/draft/v0.1.0.md`, 2.2: Claude Code on a person's own login). Everything
else in `kit.md` holds. Where the rung's brief says "the login in", "`W/account.json`" or
"`W/give-back`", read this file's steps instead.

## The model, and its login

- **Claude Code on a person's own login:** a signed-in Claude Code configuration directory,
  lent to you at `W/programs/claude` before your session started (your task says so). temper
  uses it **in place** (`temper help box` step 5, "A Claude Code login ... To use an existing
  signed-in directory in place, set `PROGRAMS` to its parent and `NAME` to its name; skip
  `program-login`"; `temper/docs/design/protocol/credentials.md`, "A login kept elsewhere,
  used in place"): `PROGRAMS` is `W/programs`, `NAME` is `claude`. It is never moved, copied or
  read; only one box uses it at a time.
- **The login in:** after the sign-in and the forge, declare it as `temper help box` step 5
  says (`account declare 2 claude_code claude --key KEY`), then list the accounts and verify the
  program source (`account verify ACCOUNT --source program`), then wait until usable. Skip
  `program-login`: the directory is signed in.
- **The login out:** there is nothing to export. Before the box stops, stop using it: no chat
  or task running; then stop the box. After the stop, check `W/programs/claude` by `stat` alone
  (a directory, mode unchanged), and say in the report what temper's account list showed of it
  before the stop. No `W/give-back` is needed; write `W/give-back` anyway as an executable that
  prints `nothing to export: used in place` so the kit's steps read the same.
- Never `claude` by hand in that directory, never sign it in or out, never read its files.

## The box's configuration (qa.md 3, the Claude rung)

- `temper init`, then, before the first start, `temper help box` step 1's Claude Code
  configuration with `MODEL` = `claude-sonnet-5-5` and `PROGRAMS` = `W/programs` (`claude` is on
  `PATH` on `HOST`: `command -v claude` inside a login shell). The agents run Claude Code on
  `claude-sonnet-5-5` at effort high: the model entry's efforts `["high", "medium", "low"]`
  (qa.md 3, the Claude rung), init's model entry otherwise kept (its window, prices and output
  8,192). The room as in kit.md: `people.requests` 256, `root.loads.loads` 32.
- **Before the first start and before every restart,** these lines must each print `true`;
  otherwise stop and report a setup to redo, never a finding:

  ```
  jq -e '.engine.llm.models[0] | .name == "claude-sonnet-5-5" and .output == 8192 and .efforts[0] == "high"' CONFIG/temper.json
  jq -e '.engine.limits.overrides["people.requests"] == 256' CONFIG/temper.json
  jq -e '.application.limits.overrides["root.loads.loads"] == 32' CONFIG/temper.json
  jq -e '.endpoints[0].models[0] | .name == "claude-sonnet-5-5" and .output == 8192' CONFIG/agent.json
  jq -e '.programs.claude_code.directory == "W/programs"' CONFIG/agent.json
  ```

  (`W` written out as the absolute path.)
- **The budget,** as kit.md. After the start, `settings show charter 1` names
  `claude-sonnet-5-5`; chats start with `--model claude-sonnet-5-5 --effort high`; where a
  template's whole choice is applied, it names them.
