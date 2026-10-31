# Brief: the playground-reset run (temper's `playground-reset.md`)

From fabrig-handover-execution, 2026-10-31, on the user's instruction:
the user asked for this run and wants it to go unattended, so solve
problems yourself.

## Why

The user is comparing ways of running large fabrig changes unattended:
wall-clock first, tokens second. Earlier today the same flow (a Claude
lead, Claude for design, Codex for implementation) did skein's component
kit in 55 minutes. You run it on temper's new draft
`docs/design/draft/playground-reset.md` (the user, 10-31, on main at
`c30cf6a03`): forges as records in the store, removable forges,
deletable projects, and the playground and its reset on them.

## The change

1. **Collect where the code stands** against the draft's section 2
   ("today"): how forges are named by `temper.json` now, the playground's
   own store, the set up held in memory, what jig offers for deleting a
   project, and anything earlier work left (the draft it replaces,
   `playground-apart.md`; branches on origin; notes under
   `~/src/fabrig/coord/` and `~/src/fabrig/leads/`). Write it to
   `status.md` beside this file.
2. **Design.** Give the draft, the status and the binding calls to a
   Claude designer (a subagent, Opus 5.5, high), which cuts sections 3 to
   7 into parts and tasks, contracts first, in the draft's order. Write
   the plan to `plan.md` beside this file and send its path, in one line,
   to `fabrig-handover-execution`; then go on without waiting.
3. **Build** it with Codex, as below.

Done means: the draft's sections 3 to 7 built, and the whole change green
on the checks fabrig's merge queue runs.

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
  form is the one the user's permissions allow. Other runs use the same
  wrappers, so begin every run NAME with `pg-`; codex-wait copies reports
  to `~/src/fabrig/coord/codex-kit/reports/`.
- **Use every available slot and build lane to finish as fast as
  possible.** Parallelise as far as the crate graph allows. Token cost is
  not a concern; wall-clock is.
- **Keep the critical path short.** Before launching, write down the
  dependency chain of your tasks in the log. If one task blocks more than
  three others, split it, or start them optimistically from its surface
  (its public items committed and pushed before its bodies and tests).
- Your clone: `~/src/fabrig/coord/playground-kit/repo` (make it; never
  touch the primary checkout `~/src/fabrig/main`). Base: fabrig `main` at
  `c30cf6a03`; record it.
- **Never land on main and never push `ready/*`.** Push branches only as
  `pgkit/<task>`, and integrate them on `pgkit/reset`. Run the queue's
  checks yourself on the integrated branch, in one slot, with `heavy
  cargo`, never `heavy --gate`:
  - for each tree with a `scripts/old_forms.py`: `python3 -I -B <tree>/scripts/old_forms.py`, from the tree's root;
  - `cargo fmt --all --check`;
  - `heavy cargo clippy --workspace --all-targets -- -D warnings`;
  - `heavy cargo nextest run --workspace --no-run`;
  - `heavy cargo nextest run --workspace --no-fail-fast`;
  - `heavy cargo nextest run --workspace --no-fail-fast --profile fuzzy`.
- **Machines:** the builders hetzner6 (`free@2.29.120.8`, hostname
  `debian-16gb-hel1-6`) and hetzner9 (`free@2.29.101.185`, hostname
  `hetzner9`). hetzner6's hostname contains "hel1", but it is a builder,
  not hel8: say so in every task brief. Seven slots each at
  `~/src/fabrig/slots/<n>`, all free and warm at main; heavy admits four
  builds at once on each. (hetzner8, hetzner10 and hetzner11 belong to
  other runs; don't use them.) A slot is claimed with `fab slot claim N RUN
  TASK` and released with `fab slot release N RUN` (in a login shell on the
  builder: `ssh HOST 'bash -lc "..."'`); `fab slot list` shows them. Codex
  works in a claimed slot (codex-start's WORKDIR). The old pool tools
  (`claim`, `codex-run`, `precheck`, `post`, leases, the launcher) are gone.
  hel8, where you run, builds nothing.
- **Advisor:** ask `fabrig-handover-execution` about this setup (machines,
  wrappers, slots, earlier runs) or a doubt you can't settle; SendMessage,
  short and specific questions. Don't wait on it; decide yourself when an
  answer is slow.
- If something in the pipeline is broken, fix it the quickest way, and
  log the minutes lost: that is part of the comparison.

## Measure (write as you go, in `~/src/fabrig/coord/playground-kit/log.md`)

- Start time, base commit; when the status and the plan are done; time
  to the first implementer running; time each part is done; end time
  (integrated branch green).
- Tokens: your own session (its transcript under `~/.claude/projects`),
  each Claude subagent, each Codex run (codex-wait prints its tokens).
  Claude and Codex totals apart.
- Where the wall-clock went: waiting for builds, for Codex, for reviews,
  for your own turns, on breakage. A short list of the biggest items.

## Report

When done (or blocked for good), write `~/src/fabrig/coord/playground-kit/report.md`
(one page: the figures above, and the three biggest wall-clock costs) and
send its path with SendMessage to "fabrig-handover-execution". Also message
it at a real blocker you can't solve, with the options. Don't message for
progress.
