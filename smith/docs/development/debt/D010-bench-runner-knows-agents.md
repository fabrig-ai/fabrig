# D010: The benchmark runner knows each agent

- **Kind:** coupling
- **Where:** `benchmarks/src/run.rs`, `benchmarks/src/process.rs`, `benchmarks/src/agent/`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 12, finding 2)
- **Seen:** 2026-10-31 at `679eacb3c`, in the quality run's closing report (outside the repository): left as a benchmark redesign
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (a new arm is an adapter, not edits in the runner; the Claude Code arm is not built yet)
- **Size:** M
- **Status:** open
- **Paid when:** `run.rs` and `process.rs` name no agent: observation, archiving and source selection are methods of the agent adapters, and `Capture::Codex` and `ProcessResult.observer` are gone.

## What

`smith/docs/design/benchmarks.md`, 6 makes agents adapters. Yet the runner
branches on the agent: `run.rs` matches `Observing::Codex` and
`Observing::Smith` at `:587`, `:610`, `:672` and `:686`, and `:665` asks a
smith observation whether its source is a program. Every process carries a
`codex::Observer` (`process.rs:53`), built for smith runs too, and
`Capture::Codex` (`process.rs:150`, `:540`, `:560`) is reached in production
only for offline replays.

The benchmarks' status file does not record this departure
(`smith/docs/design/benchmarks.status.md`).

## Why it matters

Each arm adds branches to the runner and the process supervisor. The
status file lists the Claude Code arm and an `anthropic` arm as not built;
each would add more.

## What pays it back

`Observing::collect(&mut result)` and `Prepared::archive(..)` on the
adapters; replays become a Codex observation; drop the shared observer and
`Capture::Codex`. Do it before the next arm.
