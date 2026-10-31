# Brief: the Codex comparison run, "skein component kit, phase 1"

From fabrig-handover-execution, 2026-10-31, on the user's instruction:
the user asked for this run and wants it to go unattended, so solve
problems yourself.

## Why

The user is comparing ways of running a large-ish fabrig change
unattended: wall-clock first, tokens second. An earlier manual run did
this change with Claude implementers. You are the same manual run, with
Codex implementers. Do the change the way a skilled lead would by hand,
as fast as you can, and measure it, so the runs can be compared.

## The change

- `~/src/fabrig/fab/kickoff.md`: sections 4 (scope), 5 (counts that may
  only fall), 6 (questions) and 7's mode presets. Ignore its parts about
  fab itself (entry criteria, fab calls, the code report).
- `~/src/fabrig/fab/component-kit.md`: the draft of the same scope.
- `~/src/fabrig/fab/review-2026-10-31/`: the skein code-quality review it
  comes from (themes 1b, 1c, 1d; D6, D9).

Parts: kit-contract (lib primitives + `skein_io::stage::{up, down}`,
first), echo-first, stage-sites, lib-sites, world-surfaces, abort. Done
means: the counts of kickoff section 5 at their targets (zero where a part
removes every copy) and the whole change green on the checks fabrig's
merge queue runs. Questions of section 6: pick the recommended option
yourself, measured where cheap, and record why.

## How

- You are a lead, in the style of the pre-fab pipeline: read fabrig's
  `AGENTS.md`, `agents/lead.md`, `agents/implementer.md`,
  `agents/operations.md` (in a fabrig clone). Use Claude (yourself, or
  Claude subagents) for design and taste: cutting the work, contracts,
  task specs, decisions, reviewing diffs. Use Codex for implementation:
  every implementer is a Codex run (`gpt-6.1-sol`, effort high), started
  with `/home/free/src/fabrig/coord/codex-kit/bin/codex-start` and awaited
  with `/home/free/src/fabrig/coord/codex-kit/bin/codex-wait` (read their
  headers). Call them by these absolute paths, as plain commands: that
  form is the one the user's permissions allow. Parallelise as far as the crate graph
  allows. Token cost is not a concern; wall-clock is.
- **Keep the critical path short.** Before launching, write down the
  dependency chain of your tasks in the log. If one task blocks more than
  three others, split it, or start them optimistically from its surface
  (its public items committed and pushed before its bodies and tests).
- Your clone: `~/src/fabrig/coord/codex-kit/repo` (make it; never touch
  the primary checkout `~/src/fabrig/main`). Base: fabrig `6de0cd271`,
  the earlier run's base; record it.
- **Never land on main and never push `ready/*`.** Push branches only as
  `codexkit/<task>`, and integrate them on `codexkit/component-kit`. Run
  the queue's checks yourself on the integrated branch, in one slot, with
  `heavy cargo`, never `heavy --gate`:
  - for each tree with a `scripts/old_forms.py`: `python3 -I -B <tree>/scripts/old_forms.py`, from the tree's root;
  - `cargo fmt --all --check`;
  - `heavy cargo clippy --workspace --all-targets -- -D warnings`;
  - `heavy cargo nextest run --workspace --no-run`;
  - `heavy cargo nextest run --workspace --no-fail-fast`;
  - `heavy cargo nextest run --workspace --no-fail-fast --profile fuzzy`.
- **Machines:** the builders hetzner6 (`free@2.29.120.8`) and hetzner9
  (`free@2.29.101.185`), seven slots each at `~/src/fabrig/slots/<n>`,
  all free and warm; heavy admits four builds at once on each. A slot is
  claimed with `fab slot claim N RUN TASK` and released with `fab slot
  release N RUN` (in a login shell on the builder: `ssh HOST 'bash -lc
  "..."'`); `fab slot list` shows them. Codex works in a claimed slot
  (codex-start's WORKDIR). The old pool tools (`claim`, `codex-run`,
  `precheck`, hetzner3 and hetzner7) are gone. hel8, where you run,
  builds nothing.
- If something in the pipeline is broken, fix it the quickest way, and
  log the minutes lost: that is part of the comparison.

## Measure (write as you go, in `~/src/fabrig/coord/codex-kit/log.md`)

- Start time, base commit; time to the first implementer running; time
  each part is done; end time (integrated branch green, counts at target).
- Tokens: your own session (its transcript under `~/.claude/projects`),
  each Claude subagent, each Codex run (codex-wait prints its tokens).
  Claude and Codex totals apart.
- Where the wall-clock went: waiting for builds, for Codex, for reviews,
  for your own turns, on breakage. A short list of the biggest items.
- Counts of section 5 before and after; production and test lines of the
  crates touched, before and after.

## Report

When done (or blocked for good), write `~/src/fabrig/coord/codex-kit/report.md`
(one page: the figures above, and the three biggest wall-clock costs) and
send its path with SendMessage to "fabrig-handover-execution". Also message
it at a real blocker you can't solve, with the options. Don't message for
progress.
