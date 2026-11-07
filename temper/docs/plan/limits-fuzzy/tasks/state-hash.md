+++
id = "state-hash"
part = "state"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-domain-*/src/**",
  "skein/crates/skein-forge-client/src/**",
  "skein/crates/skein-lib/src/env.rs",
  "temper/docs/plan/limits-fuzzy/figures/state-hash.md",
]
effort = "medium"
minutes = 80
+++

# skein's and jig's state types derive `Hash`

## Goal

`temper-engine-domain-world`'s walking and escalation sweeps compare the `{:?}` text of two whole worlds at every iteration. An old comment puts that at 84% of the walk. First, profile it. If the profile holds that up, derive `Hash` on the skein and jig types those worlds reach, so that `engine-sweeps` can compare digests. Those are one-word edits, about 286 of them, and they change no behaviour.

Acceptance:

- `temper/docs/plan/limits-fuzzy/figures/state-hash.md` holds, in `notes/measuring.md`'s form, a profile of `fuzzy::walking::fuzzy_walking_shard_0` and of `fuzzy::escalation::fifteen_small_escalation_histories_cover_every_story_and_durable_cut`, and a section `## Gate` whose first line is `Gate: holds, <share>` or `Gate: fails, <share>`.
- **The gate:** the share is the inclusive Ir of formatting and comparing both worlds' state in `run_replayed`, as a share of each sweep. It holds if that's at least 25% of each, or of the two together weighted by their CPU s (18 and 3.5). At 25%, about 5.4 CPU s is formatting, and that's what pays for this width. Say how you read it off the profile: `run_replayed`'s own calls, not the formatting `iterate` does for its trace.
- **If the gate holds:** each type in `notes/contracts.md`, *`state-hash`*, derives `Hash`, and each changed line differs by that word alone, but for `cargo fmt`'s wrapping. `git diff` shows nothing else in `skein/` or `jig/`.
- **If the gate fails:** no code changes. Commit the figures, say so in the report's first line, and `engine-sweeps` goes on without the digest.
- `heavy cargo check --tests` passes for each package you changed and for `temper-engine-domain` and `temper-engine-domain-forge`, and `heavy cargo clippy` passes for each package you changed, with `--all-targets -- -D warnings`.

## Inputs

- `temper/docs/plan/limits-fuzzy/notes/common.md`, `temper/docs/plan/limits-fuzzy/notes/measuring.md`, `temper/docs/plan/limits-fuzzy/notes/contracts.md`.
- `skein/docs/design/lib.md`, 5.1; `skein/docs/foundation/testing-strategy.md`, 6, its Replay line; `skein/docs/foundation/programming-model.md`, 10.2, which permits `Hash`.
- `temper/tests/engine/domain/src/walking.rs`, `run_replayed`; `temper/tests/engine/domain/src/escalation.rs`, `run_replayed`; for reading the profile.

## Owns and may change

Owns the derive lines in `notes/contracts.md`, *`state-hash`*, and the figures file. It changes no test and no temper file: temper's lines and the compare are `engine-sweeps`'.

Departures from the implementer's text, each for a reason:

- **A branch stays in its tree, and this one has two trees' lines:** skein's are 15 one-word lines that only this work reaches, and a task of their own would be more than they're worth.
- **Tests:** a derive adds an unused impl and changes no behaviour, so this task runs no test suites and adds no test. `heavy cargo check --tests` and clippy are its checks. The compare that uses these impls, and its test, are `engine-sweeps`'.

Options, each decided:

- **Overlap:** `account-email/facts-record` holds `jig/crates/jig-domain-accounts/src/{changes,domain,restore}.rs` and `jig/crates/jig-domain-core/src/start.rs`, which this list touches. A one-word edit to a derive line merges with its work unless it changes that same line. If your merge meets it, keep their change and add the word.
- **A type on the list isn't there, or is named otherwise:** the list was read at the plan's base. Find it, or leave it and say so: the compiler has the last word in `engine-sweeps`.
- **You apply the list with a script:** keep it untracked and delete it before you push.
- **A derive won't compile** because a field's type lacks `Hash` and isn't on the list: add the word there, in whichever skein or jig crate, and name it in your report. If it needs more than the word, leave that type out, carry on, and put it under `## Left` with what it needs.

## Steps

1. Profile both tests on your base, as `notes/measuring.md` says, and take their native CPU. Write the figures and the gate.
2. If it holds, make the edits, then `heavy cargo check --tests -p ...` for each package you changed, one compile repair at a time.
3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p PKG [-p PKG ...] --all-targets -- -D warnings`; commit and push.

## Report

The gate and its figures, how many lines changed in each crate, each type you added or left beyond the list, and your implementation and check minutes apart.
