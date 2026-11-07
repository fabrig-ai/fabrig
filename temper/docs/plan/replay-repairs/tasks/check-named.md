+++
id = "check-named"
part = "check-refusal"
kind = "feature"
needs = []
files = [
  "smith/crates/smith-domain-run/src/boundary.rs",
  "smith/crates/smith-domain-run/src/land.rs",
  "smith/crates/smith-domain-run/src/tests/landing.rs",
  "smith/crates/smith-domain/src/feedback.rs",
  "smith/crates/smith-protocol-machine/src/tests.rs",
  "smith/tests/agent/tests/suite/feedback_table.rs",
  "smith/docs/design/domain/run.status.md",
]
effort = "high"
minutes = 55
+++

# A failed check names itself, the time it was given and its tail

## Goal

Three `deliver` calls came back `checks-failed ... exit=timed-out cut=0 tail=""`. What the cut found by reading: `deliver` runs each writable repository's convention check, temper's `.temper/pre-pr`, under one deadline, `check_timeout` (temper init's 60 s, `temper/crates/temper-init/template/agent.json`) or what is left of the call's own (`smith/crates/smith-domain-run/src/land.rs`, `next`). The repository's `.temper/pre-pr` writes nothing until it ends: git's snapshot is quiet, `cargo fmt --all --check` prints only a diff, and each tree's `old_forms.py` prints its counts at its end. So a run past 60 s is killed with no byte written, and the empty tail is true. The figure stays as it is. What the feedback lacks is the check it ran and the time it had: `smith/docs/design/domain/run.md`, 8.1 now asks for both. Add them to `Returned::ChecksFailed` and its words, as `notes/contracts.md` says, and pin with a step test that what a check writes before its deadline reaches the tail of a timed-out answer.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 5).

Acceptance:

1. `smith-protocol-machine` (`src/tests.rs`): beside `deadline_sends_group_term_kill_close_and_answers_after_reap`, or as a row of it, a check that writes bytes on its pipes before its deadline (and some during stopping, before close) answers `ToDomain::Checked { ran: Ran { exit: TimedOut, output, cut } }` whose `output` is the last `tail` bytes written and `cut` the bytes dropped before them. If it already holds, it is the pin; if it fails, the machine keeps what it read, and the report says what dropped it.
2. `smith-domain-run` (`src/tests/landing.rs`): a delivery whose check answers `Exit::TimedOut` with a tail returns `ChecksFailed { repository, check, given, ran }`: its repository's name, the charter's convention path (`.smith/check` unless the charter names another), `given` the run's `check_timeout` when the call had longer left, and the tail and cut as the machine answered. Default: a row beside `a_mid_run_delivery_without_a_change_contract_needs_passing_checks`, or that test made a table over `(exit, output, cut)` with its code-1 row as now.
3. The words: `every_run_call_terminal_has_canonical_feedback` (`smith/tests/agent/tests/suite/feedback_table.rs`)'s `ChecksFailed` row reads `checks-failed repository="repo" check=".smith/check" exit=timed-out given_ms=60000 cut=3 tail="tail"`, the fields in the contract's order, and `given_ms` written for every exit. `feedback_bound` prices the check's path and `given_ms` within its `checks` term.
4. No other refusal or exit changes: the feedback table's other rows pass unchanged.
5. The entry *8.1: a failed check's feedback names its repository alone...* is gone from `smith/docs/design/domain/run.status.md`.
6. Focused tests pass in `smith-domain-run` (`--lib`), `smith-domain` (`--lib`), `smith-protocol-machine` (`--lib`), `smith-agent-world`, `smith-run-world` and `smith-machine-world`; `heavy cargo check --tests -p jig-channel-kit -p jig-protocol-worker-agents -p temper-engine-smith` compiles.

## Inputs

- `temper/docs/plan/replay-repairs/notes/common.md`.
- `temper/docs/plan/replay-repairs/notes/contracts.md`, *A failed check's feedback*: it binds.
- `smith/docs/design/domain/run.md`, 8.1 (*A failing check is feedback*); `smith/docs/design/protocol/agent.md`, the machine's *Output* and *Deadlines*; `smith/docs/design/protocol/limits.md`, `check_timeout` and `check_tail`.

## Owns and may change

**Owned:** the `files` above.

**Readers** (the compiler names them; each takes the new fields and no other edit): `smith-domain-run/src/facts.rs`, `src/run.rs`; `smith-domain/src/route.rs`, `src/translate.rs`; `smith/tests/run/src/partner.rs`; `smith/tests/agent/src/host_referee.rs`; `smith/tests/agent/tests/suite/feedback.rs`. `smith-protocol-events`'s `calls.rs` and `children.rs` classify the variant without reading its fields: unchanged.

**Not touched:** `.temper/pre-pr` and every timeout figure. The repository's check writing its phases would give a timeout's tail something to say; that is the repository's, outside smith, and left to a later plan.

**Options, each with its default:**

- `given`: the span from the check's request to its deadline, `deadline - env.now` at `next`, kept in `Stage::Checking` beside `check`. If keeping the deadline itself and subtracting at the answer reads better, do that.
- The path: `conventions::checks(&run.charter)`, as `next` builds the `Place`, copied into the refusal.
- If a reader outside smith matches `ChecksFailed` by its fields after all, it takes the new fields as a compiler-forced edit, and the report names it.

## Steps

1. Acceptances 1 and 2's tests; run them and see them fail (2 does not compile until the fields exist; write the fields' types first, then the test, then the bodies): `heavy cargo nextest run -p smith-protocol-machine --lib -E 'test(/deadline/)'`, `heavy cargo nextest run -p smith-domain-run --lib -E 'test(/landing::/)'`.
2. The fields, `land.rs`, `render_checks` and `feedback_bound`; the feedback table's row (acceptance 3). Break the path once (render the repository twice), see the table fail, restore.
3. The readers, the status entry, acceptance 6.
4. `cargo fmt`, `python3 -I -B smith/scripts/old_forms.py`, `heavy cargo clippy -p smith-domain-run -p smith-domain -p smith-protocol-machine -p smith-agent-world -p smith-run-world --all-targets -- -D warnings`, commit and push.

## Report

As the implementer's text says; also whether the machine's tail on a timeout already held, and how `given` is kept.
