# The walk kit: what every rung brief of the v0.1.0 ladder shares

Read with the rung's brief; together they are the rung's acceptance and are rerun unchanged.

## You

- **A QA session** walking one rung of temper's ladder (`temper/docs/design/draft/v0.1.0.md`,
  2 to 4) as one of temper's users: an agent working with temper for a person. Read, in the
  checkout `REPO` your task names (fabrig at the walk's commit, read-only for you): temper's
  `temper/docs/design/qa.md` (whom a pass plays, the box, the hands, reading the forge, what
  makes a finding), then the rung's brief.
- **You change nothing but your scratch.** No commit, push or landing anywhere, no edit to
  any checkout, no fix to what you find: you report it. No GitHub at all on the first pair;
  never any `fabrig-ai` repository.
- **temper through `temper ctl`, with `temper help` and `temper ctl help` as your guide;**
  the forge through its API with `curl` (qa.md 5). Where you stall, guess, or read a design
  or temper's source to get on, that is a finding against the help or ctl (v0.1.0.md 4):
  note it with the minutes it cost, then go on. Reading the forge is never a finding.

## The machine

- **The builder your task names** (`HOST`, reached as `ssh -n HOST`; your session runs on the
  dispatcher's machine). Commands there in a login shell: `ssh -n HOST 'bash -lc "..."'`; give every remote command `-n` or `</dev/null`, and every `temper ctl` call
  `</dev/null` unless it reads a file on stdin. Write helper scripts into `W` and run them,
  rather than quoting long commands through ssh.
- **Your work directory `W`** on `HOST` (0700), named by your task: your scratch, your box,
  your notes and evidence. Nothing outside it but reads: never fab's slots, another walk's
  directory, any login, credential or key store, or a tool that lends logins.
- **The binary** is built for you: `W/bin/temper`, at the walk's commit. Put `W/bin` first
  on `PATH`. Don't build.
- **The box runs as a user unit,** so it outlives your ssh calls and stops cleanly:

  ```
  systemd-run --user --unit=UNIT --collect -p StandardOutput=file:W/box.log \
    -p StandardError=file:W/box.log --setenv=PATH=W/bin:/usr/local/bin:/usr/bin:/bin \
    W/bin/temper box --config W/box/config --data W/box/data --cache W/box/cache
  ```

  `UNIT` is your task's. Read its output from `W/box.log`; stop it with `systemctl --user
  stop UNIT` (SIGTERM), only after the login is back (below). `DIR` (ctl's directory) is
  `W/box/ctl`. No other temper unit on the machine is yours: leave them alone.
- **Resources are shared** with other runs: run nothing heavy besides the box; read the forge
  at a person's pace (a wait on temper's side between reads, never a tight loop of reads).

## The model, and its login's move in and out

- **ChatGPT** (the first pair): the user's subscription, a temper account record lent to you
  at `W/account.json` before your session started (your task says so). Import it with ctl's
  account import (its key as `temper help box` step 5 says): the file leaves when the box takes
  it. From then on the box is its one holder.
- **At once after the import,** write `W/give-back`: an executable that exports the account
  back over `W/account.json` with `temper ctl` (every program by its absolute path; it runs
  outside your shell) and then checks by `stat` alone that the file is a non-empty regular
  file, mode 0600. **Before the box stops, for any reason** (the end, a pause, a failure, a
  restart case) run it and check it. If the export fails, stop everything else and say so
  first in your report; leave the box running.
- **Give back before you break it** (qa.md 3): a case that aborts `serve`, kills the box or
  risks its restore exports the login first, and imports it again only if a later step needs
  the model.
- **Never read, print, copy or hash-compare a credential's contents** (the account file,
  tokens, header files, `person_password`, anything temper writes as a secret): name files by
  path, check them by `stat`. Never run `fab logins`: the owner borrowed and returns it.

## The box's configuration (qa.md 3)

- `temper init`, then, before the first start, in `CONFIG` (`W/box/config`): the agents on
  `gpt-6-luna` at effort medium, init's model entry kept as it is (its window, prices and
  output 8,192), its efforts `["medium", "high", "low"]`; the room `engine.limits.overrides
  ["people.requests"] = 256` and `application.limits.overrides["root.loads.loads"] = 32`.
  A start refused for memory: raise `memory_bytes` by what it names, and say the figure.
- **Before the first start and before every restart,** these four lines must each print
  `true`; otherwise stop and report a setup to redo, never a finding:

  ```
  jq -e '.engine.llm.models[0] | .name == "gpt-6-luna" and .output == 8192 and .efforts[0] == "medium"' CONFIG/temper.json
  jq -e '.engine.limits.overrides["people.requests"] == 256' CONFIG/temper.json
  jq -e '.application.limits.overrides["root.loads.loads"] == 32' CONFIG/temper.json
  jq -e '.endpoints[0].models[0] | .name == "gpt-6-luna" and .output == 8192' CONFIG/agent.json
  ```

- **The budget,** raised right after the sign-in and before `forge setup` makes the project,
  as qa.md 3 says (one `settings apply` with its figures); say them in the report. After the
  start, `temper ctl DIR settings show charter 1` names `gpt-6-luna`; a box that shows
  otherwise is a setup to redo. Chats start with `--model gpt-6-luna --effort medium`.

## A finding, and a pause

- **A finding** (qa.md 6): temper giving no way to the forge, temper and the forge
  disagreeing, temper stale past its bound, the help not getting you there, temper's words
  wrong, a start that panics on a link. Each with its reproduction from a fresh box (the
  commands, in order), what was expected and what was seen, and whether it blocks.
- **A blocking finding** (you cannot reach the brief's "What passes" without a hand on the
  product) **pauses the walk:** write it up with its evidence, run `W/give-back` and check it,
  stop the box keeping all of `W` as it is, write the step you stopped at to `W/paused` (one
  line), and end with the report. A finding that doesn't block is filed and the walk goes on.
- **Wall cap: 3 hours** from your start. At the cap, give back, stop the box and report what
  you reached.

## The report

`REPORT` (your task names it), at most 15 KiB, written for the user, current as you
go, times from `date -u`:

- **Verdict:** does the rung pass, yes or no, against the brief's "What passes", and where it
  broke.
- **The login, in and out:** the import's time and line, the export's time and line, the
  file's `stat` after (type, size, mode); the box's last line.
- **The walk:** each step, the commands, its start and end times, what temper said and what
  the forge showed (qa.md 6's two accounts).
- **How hard to drive:** every place you stalled, guessed, read source or a design to get on,
  or worked around the tool, with the minutes it cost; what the help and `temper ctl help`
  lacked.
- **Tooling it lacked,** ranked, each with the friction it removes.
- **Findings,** numbered, each: blocking or not; reproduction from a fresh box; expected;
  seen; evidence (file paths under `W`, masked where a log line holds a secret).
- **The figures:** the budget you set, the memory you raised, the box's agents' model.
