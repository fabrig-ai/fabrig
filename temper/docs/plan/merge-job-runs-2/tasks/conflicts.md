+++
id = "conflicts"
part = "conflicts"
kind = "feature"
needs = []
files = ["temper/crates/temper-worker-protocol-git/**", "temper/tests/worker/checkout/src/translate.rs", "temper/tests/worker/checkout/src/merges.rs", "temper/tests/worker/checkout/tests/suite/merges.rs"]
effort = "high"
minutes = 120
+++

## Goal

A merge with more files in conflict than the checkout's `conflicts`
prepares, as `temper/docs/design/protocol/git.md`, 4.6 now says: the
listing of unmerged paths keeps the first `conflicts`, in git's order,
and is no longer `Broken`, so a merge job or a resolution of a large
merge is no longer failed as transient on every try. The fake git layer
gets the same rule, so the worlds can show it.

Acceptance:

- In `temper-worker-protocol-git`, a `Merge` whose `diff-files` listing
  (exit 0) holds more than the checkout's `conflicts` paths ends
  `Done::Conflicted { files }` with the first `conflicts` of them; one
  whose stdout passed `stdout_bytes` is read as far as it was kept, a
  last path cut short dropped, and ends the same way. A path longer than
  `path_bytes` among those kept stays `Broken`, as does an empty
  listing.
- Every other invocation whose stdout passed its bound still ends
  `Broken` (git.md, section 6), a merge's commit's listing (4.7) among
  them: its scan must read every path (git.md, section 10).
- Rows in `src/tests/sequence.rs`, `standard_rows` (or the table that
  holds `Unmerged` today): `conflicts + 1` paths for a `Merge` op, the
  first `conflicts` kept; the same listing for a `Commit` op, `Broken`.
  A test in the operation's tests (`src/tests/component.rs` or where a
  dropped stdout is driven today) for a merge's listing past
  `stdout_bytes`, the kept paths named, and for another invocation past
  it, `Broken`.
- `temper_worker_checkout_world::translate::perform_within`
  (`notes/contracts.md`, section 4) gives the fake the rule;
  `perform` is unchanged.
- The checkout world: a `Case` row in
  `merges_conflicts_saving_head_movement_and_ambiguous_pushes_settle`
  (`temper/tests/worker/checkout/tests/suite/merges.rs`), a merge in
  progress with `conflicts + 1` files in conflict through
  `perform_within`: the prepare ends ready, it names the first
  `conflicts` files, every one of them and the one past them holds its
  markers on the fake disk with `MERGE_HEAD` at the base; settled and
  pushed as the `Conflict` row is, the push carries both parents.
- Failing first: the protocol rows and the checkout row are committed
  failing (the protocol rows `Broken`; the checkout row as the fake
  without the rule shows it, or as the domain's admission refuses it),
  then the fix.

## Inputs

- `temper/docs/plan/merge-job-runs-2/notes/common.md`;
  `temper/docs/plan/merge-job-runs-2/notes/contracts.md`, section 4.
- `temper/docs/design/protocol/git.md`: 4.6 (*Exit 1* and *More than the
  checkout's `conflicts`*), 4.7 (the merge's commit listing), section 6
  (*What is kept*, stdout), section 8 (`stdout_bytes`' rule), section 10
  (*A merge's commit past the checkout's `conflicts`*).
- `temper/docs/design/domain/worker.md`, section 5, *A merge in progress*
  (the sentence on more files than `conflicts`).
- The code: `temper-worker-protocol-git/src/output.rs`, `paths`;
  `src/sequence.rs`, `after` (the `Merge` and `Unmerged` arms) and
  `unmerged`; `src/operation.rs`, the stdout cap (`child.dropped`, lines
  190 to 205 and 349 to 354). The checkout's admission of the list:
  `temper-worker-domain-checkout/src/hold.rs`, `paths_fit` and its
  `assert!` in `prepared_merge` (it holds once the list is cut).
- The fake: `temper/tests/worker/checkout/src/translate.rs`, `perform`'s
  `Op::Merge` arm; the checkout world's merge stories:
  `temper/tests/worker/checkout/src/merges.rs` (`Case`, `run`).

## Owns and may change

- **Owns:** `temper-worker-protocol-git` (its sources and tests);
  `temper/tests/worker/checkout/src/translate.rs` (`perform_within`
  added); `temper/tests/worker/checkout/src/merges.rs` (a `Case` and what
  it needs, calling `perform_within`); the checkout world's
  `tests/suite/merges.rs` table rows.
- **Not yours:** `temper-worker-domain-checkout` (task `lines` changes its
  `Prepared::Failed`); the composed world (`temper/tests/system`, tasks
  `story` and `story-rows`: `story-rows` switches it to
  `perform_within`); the worker world.
- **Options:** how the operation tells `sequence::after` that a merge's
  stdout was cut is yours (a flag beside `stdout`, or the dropped count);
  the stdout of other invocations keeps today's path. If `perform_within`
  cannot tell a merge's commit's unmerged count in the fake, leave the
  fake's commit as `perform` has it and say so. If the checkout world's
  `Case` needs the world's limits with `conflicts` lowered (as the worker
  world's `whole/merge.rs` sets 1), lower them in that world alone.
- **No figure:** `conflicts`, `path_bytes` and `stdout_bytes` keep their
  values everywhere (`notes/common.md`).

## Steps

1. Write the protocol rows and the checkout row; run each alone and see
   it fail; commit.
2. Fix `paths`/`unmerged`/the operation's stdout cap for a merge's
   listing; add `perform_within`; run the rows alone, then
   `heavy cargo nextest run -p temper-worker-protocol-git --lib` and
   `heavy cargo nextest run -p temper-worker-checkout-world --test suite`.
3. Bite: restore the `count > checkout.conflicts` refusal for the merge's
   listing and see the new row fail `Broken`; restore the fix.
4. `cargo fmt`; `heavy cargo clippy -p temper-worker-protocol-git -p temper-worker-checkout-world --all-targets -- -D warnings`;
   the fuzzy suites of both packages (`--profile fuzzy`); the direct
   dependents that call `translate::perform`:
   `heavy cargo check -p temper-worker-domain-world -p temper-system-world --tests`.
5. Commit and push.

## Report

`Done:`, `Failed:` or `Question:` first. Then: what the listing now does
past the bound and past `stdout_bytes`, and what stays `Broken`; the rows
and their failure before the fix; the bite; implementation and check
minutes apart.
