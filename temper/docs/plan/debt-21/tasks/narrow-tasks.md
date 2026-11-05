+++
id = "narrow-tasks"
part = "narrowing"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/closing.rs", "jig/crates/jig-domain-tasks/src/compound/staged.rs", "jig/crates/jig-domain-tasks/src/questions.rs", "jig/crates/jig-domain-tasks/src/control.rs", "jig/crates/jig-domain-tasks/src/proposals.rs", "jig/crates/jig-domain-tasks/src/waits.rs", "jig/crates/jig-domain-tasks/src/domain.rs", "jig/crates/jig-domain-tasks/src/inbox.rs", "jig/crates/jig-domain-tasks/src/placed.rs", "jig/crates/jig-domain-tasks/src/standing.rs", "jig/crates/jig-domain-tasks/src/funders.rs"]
effort = "high"
minutes = 90
+++

# jig-domain-tasks: twenty-five sites match once

## Goal

Part of `jig/docs/development/debt/D017-unreachable-re-narrowing.md`. In `jig-domain-tasks`, a function re-reads what its caller or its own first match fixed, or a message's kind fixes who asked it. Each site takes a local fix, and one small field carries the question's source.

Acceptance:

- `rg -c 'unreachable!'` reads, under `jig/crates/jig-domain-tasks/src`: `closing.rs` 2, `compound/staged.rs` 2, `questions.rs` 2, `control.rs` 1, `proposals.rs` 2, `waits.rs` 5, and nothing in `domain.rs`, `inbox.rs`, `placed.rs` or `standing.rs`.
- `heavy cargo nextest run -p jig-domain-tasks --lib` and `heavy cargo nextest run -p jig-tasks-world --test suite --test lifecycle --test open` pass, with no assertion changed. `heavy cargo nextest run -p jig-core-world --test suite` passes too, since the core reads this crate.
- Net production lines: the cut's estimate is about +20.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/narrowing.md`, binding
- `jig/docs/development/debt/D017-unreachable-re-narrowing.md`
- `jig/docs/design/domain/tasks.md`, 5.2, 5.6 and 7

## The sites

- `closing.rs:161, 172, 181`: `cancelled() -> Closing`. `closing.rs:253, 318, 322`: `progress` passes `ending`.
- `placed.rs:554`: `active @ (…)`.
- `compound/staged.rs:644`: match once.
- `standing.rs:72, 91`: narrow the source at line 43.
- `questions.rs:36, 69`: destructure at dispatch, with an `#[expect(clippy::too_many_arguments, reason = "...")]` as `escalation.rs` has.
- `control.rs:811, 844`: `cancel()` and `release()` over a shared entrance.
- `proposals.rs:433, 900, 915`: the first match yields the narrow outcome and change.
- `waits.rs:909`: `circle_held` also returns `&Was`.
- `domain.rs:1349, 1379`: `original_period -> Option<Period>`.
- `inbox.rs:415`: `MessageKind::family()`, once.
- `questions.rs:55`, `inbox.rs:496, 673`, `compound/staged.rs:210`: `MessagePlan.asked_by: Option<u64>`, set where `inbox.rs:444` checks it.

`compound/staged.rs:596` stays: its `Held { why, source }` is stored.

## Owns and may change

- **Owns:** the files in `files`.
- **`run.rs` stays out:** it would copy a six-variant enum for four sites. So do `control.rs:985`, `waits.rs:1141` and `close_project.rs:29`.
- **`MessagePlan`:** if it turns out to be public or stored, leave its four sites, and say so.

## Steps

1. The sites, one file at a time, running `-p jig-domain-tasks --lib` between.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p jig-domain-tasks --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the counts; any site that stayed, and why.

## Answers

2026-11-05. May this task also change `funders.rs` to narrow `original_period` to `Option<Period>` and repair its local callers? Yes: `domain.rs`'s count of nothing can't hold otherwise, so `funders.rs` joins `files`, for this change only. Add a `pub(crate) struct Period { project: u32, period: u64 }` in `funders.rs`, `Copy` and `PartialEq`, with a small `funder()` returning `Funder::Period`; `original_period` builds it in its `Funder::Period` arm. `tree_made` loses its re-match the same way, as a further re-narrowing. `restore_fits`'s comparison at `funders.rs:312` and `representable` use `funder()` where they need the map's key. Behaviour, stored and wire forms stay as they are; `Period` is not stored and takes no public path. Every other `funders.rs` site, its `unreachable!` count (0) included, stays.
