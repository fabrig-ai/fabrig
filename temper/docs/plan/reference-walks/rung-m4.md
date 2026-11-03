# Rung M4, surveys as sub-agents: a Claude Code chat asks two inspect-only sub-agents side by side

With `kit.md` and `kit-claude.md` beside this file (the machine, the login's move, the
configuration, findings and the report). Backends: **Claude Code with an owned Forgejo**.

## What this is

The walk of the reference case's milestone M4, surveys as sub-agents
(`temper/docs/design/draft/reference-case.execution.md`, M4, its walk folded into M3's; the
design: `temper/docs/design/protocol/setup.md`, 8.1). A chat started with two sub-agent choices
asks two inspect-only sub-agents to read two parts of the repository side by side; each runs at
its own LLM choice, is seen through ctl, and its session ends on its budget. The happy path only.
`temper help` and `temper ctl help` are the person's guide. This brief is the rung's acceptance:
it is rerun unchanged until it passes.

## The scenario

1. **The box,** as rung 0 on Claude Code walks it: configured, the four checks `true`, started as
   a user unit, signed in, the budget raised before any forge is set up.
2. **The playground's repository on the owned Forgejo,** made through ctl as the help names it,
   adopted as a project's home. Read it through the API with the read header: its files and
   `main`'s head.
3. **The login in,** as kit-claude.md says.
4. **The chat,** through ctl in the widget's project, started with a model and two sub-agent
   choices among those `settings show charter 1` offers (`chat start --project PROJECT --model
   MODEL --sub-agent MODEL_A[:EFFORT] --sub-agent MODEL_B[:EFFORT]`, as `temper ctl help chat
   start` names it; two different choices if the charter offers two), its words exactly:

   > Have two sub-agents read `src/main.rs` and `README.md` side by side, then tell me what each found.

5. **Expected:** two inspect-only sub-agents run side by side, each a session of its own at its
   LLM choice, none refused as too many; each child's session ends on its budget; the chat's answer
   is true to the two files. Read through ctl (the chat's activity, the sub-agents' sessions and
   their choices) and on the forge (nothing changed: `main`'s head, no new branch or pull request).
6. **Clean up:** the login back, the box stopped. Keep all of `W`.

## What passes

The two sub-agents, side by side, each at its own choice, inspect only, each ended on its budget,
read through ctl; the answer true to the files; nothing changed on the forge; the login back
before the stop, its directory checked by `stat`; every step from `temper help` and `temper ctl
help`, or each place they did not get you there filed as a finding.

Say in the report: the choices given, each sub-agent's session and spend, the minutes from the
chat's start to its answer.
