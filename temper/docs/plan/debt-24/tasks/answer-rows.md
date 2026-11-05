+++
id = "answer-rows"
part = "spot-review"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/measure.rs", "skein/crates/skein-forge/src/forgejo/plans/reads/size.rs", "skein/crates/skein-forge/src/forgejo/plans/tests.rs", "skein/crates/skein-forge/src/forgejo/plans/tests/edges.rs", "skein/crates/skein-forge-client/src/measure.rs"]
effort = "medium"
minutes = 25
+++

# skein-forge: one walk of an answer's row limits

## Goal

Finding 1 of `temper/docs/plan/debt-24/inputs/debt22-review.md`, checked against the code, and the first test of finding 7. `reads/size.rs`'s `within` walks the same lists as `skein-forge-client`'s `measure::answer_rows`. `answer_rows` moves into `skein_forge::measure`; `size::answer` calls it; `within` goes.

Acceptance:

- `skein_forge::measure::answer_rows(value: &Answer, rows: u32) -> bool` is the function that today's `skein_forge_client::measure::answer_rows` is, with its doc line and its `Queued` comment.
- `skein-forge-client/src/measure.rs` keeps `pub use skein_forge::measure::answer_rows;`, so `bounds.rs`, `tests/contracts.rs` and `temper/tests/engine/forge/src/world.rs` don't change. A public path stays.
- `size::answer` is the row check, then `crate::measure::answer`:

  ```rust
  pub fn answer(answer: &api::Answer, limit: u32) -> Result<u64, api::Error> {
      if !crate::measure::answer_rows(answer, limit) {
          return Err(api::Error::TooLarge);
      }
      crate::measure::answer(answer).ok_or(api::Error::TooLarge)
  }
  ```

  `within` goes. `rows` and `summary` stay; `summary` is read at `reads.rs` and `reads/done.rs`.
- `nested_answer_rows_keep_their_limit_beside_the_measure` (`plans/tests.rs`) goes: `answer_rows_past_bound_do_not_fit` (`skein-forge-client/src/tests/contracts.rs`) holds the lists. `collaborators_finish_without_measuring_the_rows_again` (`plans/tests/edges.rs`) goes: it pins `rows: 0`, a bound no client builds, for a code path nothing else can tell apart.
- **`size::answer`'s call still has a test.** Remove the `answer_rows` check by hand and run `-p skein-forge --lib` and `-p skein-forge-connection --lib`. If a test goes red, name it in the report and restore. If none does, add one short test beside `forgejo/plans/tests.rs`'s other tests: one `Answer::Branches` a row past the bound answers `Err(TooLarge)`, about six lines; break it the same way.
- `heavy cargo nextest run -p skein-forge --lib`, `-p skein-forge-client --lib`, `-p skein-forge-connection --lib` and `-p skein-forge-machine-world` pass with no other assertion changed; `heavy cargo check -p temper-engine-forge-world --tests` passes.
- Production's net lines fall by about 50.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/inputs/debt22-review.md`, findings 1, 7
- `skein/docs/design/forge.md`, 3.7; `skein/docs/design/forge-client.md`, the measure's paragraph

## Owns and may change

- **Owns:** the files in the header.
- **`within`'s `InvalidAnswer` arms.** By the cut's reading no production path brings them to `size::answer`: `terminal` is reached from `reads.rs`'s `Status` arm with a `Pull` and from `reads/done.rs`'s `ended`, and `actions.rs` passes `Checks`. Confirm it with `rg 'ended\(|terminal\(' skein/crates/skein-forge/src/forgejo/plans/reads*`. If one does reach it, stop and ask: the answer would change from `InvalidAnswer` to `Ok`.
- **A fixture only the two cut tests read,** if the compiler says so, goes with them.

## Steps

1. The move and the call; the two cuts.
2. The hand break above.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge -p skein-forge-client --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; what the hand break turned red.
