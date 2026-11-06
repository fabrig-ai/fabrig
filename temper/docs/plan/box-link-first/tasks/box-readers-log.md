+++
id = "box-readers-log"
part = "box-readers"
kind = "migration"
needs = ["box-draws-link"]
files = ["temper/crates/temper/tests/process.rs", "temper/crates/temper/tests/support/boxed.rs", "temper/crates/temper/tests/support/first_repository.rs", "temper/crates/temper/tests/support/forge_land/rung.rs", "temper/crates/temper/tests/suite/box_kinds.rs", "temper/crates/temper/tests/suite/startover.rs"]
effort = "medium"
minutes = 55
+++

# What reads the box's lifecycle reads `box.log`

## Goal

Every test and harness that reads the box's moved words reads `DATA/box/logs/box.log` instead, the live and macOS stories among them. No assertion weakens: each still shows the same fact, from the file it now lives in.

Paths `tests/...` here are under `temper/crates/temper/`.

Acceptance, each reader below moved:

- `tests/process.rs`, about lines 416 and 434: `Forge 1 (Forgejo): starting|started`, once each.
- `tests/process.rs`, about lines 462-474: `box_terminal_line`'s list: delete the moved words, add the new block's lines, keep `box: `.
- `tests/process.rs`, about lines 600-601 and 723-724: `First run: wrote the configuration, `: read that run's `box.log`, before a next start moves it.
- `tests/suite/box_kinds.rs`, about line 186: the three lifecycle lines from `root/box/logs/box.log`. Line 189 stays.
- `tests/support/first_repository.rs`, about line 556: `Forge N (Forgejo): started`, from `data/box/logs/box.log`.
- `tests/suite/startover.rs`, about lines 135-137: the same, from the first box's data.
- `tests/support/forge_land/rung.rs`, about line 877, `setup_observer`: from `paths[1]/box/logs/box.log`. It is reached through `tests/live.rs` and `tests/proof_live.rs`.
- `tests/support/boxed.rs`, about line 588: the playground interrupt's gate on `Starting serve\n`: use `find(live_pids, temper, b"serve")`, about line 110, which no stale log can fool.
- `heavy cargo nextest run -p temper --test suite -E 'test(box_kinds) or test(startover)'` passes.
- `heavy cargo check -p temper --tests` passes, so `live` and `proof_live` compile. You cannot run the live tiers: say so.

## Inputs

- `temper/docs/design/protocol/setup.md`, 7.2, *Its own log*.
- `temper/docs/plan/box-link-first/notes/contracts.md`, section *What a reader of the box's output may rely on*.
- `temper/docs/plan/box-link-first/notes/common.md`.

## Owns and may change

Owns the files in the header. Add one helper in `tests/support/boxed.rs`:

```rust
/// The box's own log of this run, empty before its first line (protocol/setup.md, 7.2).
pub fn box_log(data: &Path) -> String;
```

Options, each pre-approved:

- A reader whose closure has no data directory to hand: pass it in, call sites only.
- `support/first_repository.rs`, about line 559, already reads `box/logs/`: keep to its form.
- No change needed, and leave alone: `suite/forge_land.rs`, about lines 1179 and 1193; `support/forge_land.rs`, about lines 749 and 795; `process.rs`, about line 533; `support/program_login.rs`, about line 11. Those are step lines or kept `box:` lines.

## Steps

Move each reader, then run its test alone. The macOS workflow's one story is `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart` (`.github/workflows/macos.yml`, about line 177). It reaches `first_repository.rs` and `process.rs`'s `First run` readers. Read it through by hand: nothing you move may be Linux's alone. Run it once under its profile, on Linux: `heavy cargo nextest run -p temper --profile process-temper --test process -E 'test(=a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart)'`.

Reds that come from taking the first link for serve's being up are `box-readers-early`'s: report them under `## Left`. Then fmt, clippy on `temper`. Commit and push.

## Report

As your text says, and what you read of the macOS story's path.
