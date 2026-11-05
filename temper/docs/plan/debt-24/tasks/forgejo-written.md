+++
id = "forgejo-written"
part = "spot-review"
kind = "feature"
needs = ["forgejo-repository"]
files = ["skein/crates/skein-forge/src/forgejo/plans/writes.rs", "skein/crates/skein-forge/src/forgejo/plans/tests/calls.rs"]
effort = "high"
minutes = 40
+++

# skein-forge: `Writing` keeps what its answer needs, not the write

## Goal

Finding 6's first half and the second test of finding 7 (`temper/docs/plan/debt-24/inputs/debt22-review.md`). `State::Writing { row, write }` holds a whole `api::Write`, body and title included, through the exchange, and `initial` clones it to do so. What the answer needs is small: which answer to read, what status it expects, what a 409 means.

Acceptance:

- `State::Writing { row: Repository, written: Written }`, with a private `Written`, `Copy`, built by one function from `&api::Write`. Its shape is the implementer's; by the cut's reading these answers differ and nothing else does:

  ```rust
  #[derive(Clone, Copy, Debug)]
  enum Written {
      Issue,                        // CreateIssue
      Comment,                      // Post
      Review,                       // Review
      Edited,                       // Edit, Close, Reopen
      Status,                       // Status
      Branch,                       // CreateBranch
      Bare,                         // Dispatch, Rerun
      Deleted,                      // DeleteBranch
      Update(api::UpdateStyle),     // Update
      Pull,                         // OpenPull
      Merge { number: u64 },        // Merge
  }
  ```

  `SetReviewers` never reaches `Writing`: it keeps today's `unreachable!` where the arm is forced.
- `initial` no longer clones the write, and `compared` no longer rebuilds one to store. `asked(write)` and `validate` stay for `New` and `Resolving`.
- Every error and answer stays: `Asked`, the expected status, `conflict`, and `written`'s answers read the same for every `Write`. `heavy cargo nextest run -p skein-forge --lib`, `-p skein-forge-connection --lib` and `-p skein-forge-machine-world` pass with no assertion changed, but the next point.
- `write_errors_keep_the_exchange_state` (`plans/tests/calls.rs`) has explicit rows: no derived range, no `if expected == Timeout`:

  ```rust
  // write, bodies heard, what `Timeout` becomes, what `Forbidden` becomes
  (open_pull(), &[][..], Error::Unavailable, Error::Forbidden),
  (open_pull(), &[REPOSITORY][..], Error::Unavailable, Error::Forbidden),
  // ... the other two counts; then the merge after its two bodies: Timeout, Timeout
  ```

  It checks the same five cases with the same three assertions, reading the rows.
- **The size gate:** `Written` stays only if the branch's production net lines are at or below 0. The enum and its builder cost about 30 lines; `conflict`'s and `expected`'s long matches and `written`'s arms pay them. If it can't get there, it's skipped: keep the old `Writing`, say so with the measured size, and do the test's rows alone.
- **The review's correctness note:** `writes::Plan::errors` panics in `Ended`. By the cut's reading it isn't reachable: `begin` and `heard` leave `Ended` only by returning `Err` or `Step::Done`, and `skein-forge-connection/src/component/calls.rs` finishes the call on either before any `error` call. Read those paths again as you change them, and say so in the report. If a path reaches it, stop and ask. It stays a panic, as it is.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/inputs/debt22-review.md`, findings 6, 7 and the correctness notes
- `skein/docs/foundation/programming-model.md`, 5.4

## Owns and may change

- **Owns:** the files in the header.
- **Nothing outside `writes.rs`'s private items changes:** no `api` type, no `pub` signature. `Written` isn't `pub`.

## Steps

1. The test's rows first, against today's code; it passes.
2. `Written`; `Writing`; its four readers; `initial`; `compared`.
3. Measure the net lines; apply the gate.
4. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the gate's numbers; the correctness note's reading. No new rule: the rows are the old cases made explicit.
