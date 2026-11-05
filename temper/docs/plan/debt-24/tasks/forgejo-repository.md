+++
id = "forgejo-repository"
part = "spot-review"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge/src/forgejo/plans/tables.rs", "skein/crates/skein-forge/src/forgejo/plans/failure.rs", "skein/crates/skein-forge/src/forgejo/plans/artifact.rs", "skein/crates/skein-forge/src/forgejo/plans/actions.rs", "skein/crates/skein-forge/src/forgejo/plans/reads.rs", "skein/crates/skein-forge/src/forgejo/plans/reads/**", "skein/crates/skein-forge/src/forgejo/plans/contents.rs", "skein/crates/skein-forge/src/forgejo/plans/turn.rs", "skein/crates/skein-forge/src/forgejo/plans/writes.rs"]
effort = "medium"
minutes = 35
+++

# skein-forge: Forgejo's plans share `at`, `id` and a refusal

## Goal

Finding 6's second half (`temper/docs/plan/debt-24/inputs/debt22-review.md`), checked against the code. Six plans each build a `Repo` from a row, four an `ObjectId`, four classify a refused status and drop the repository, and `reads`' states keep a `limit` that is always `bounds.page_size`.

Acceptance:

- `tables::Repository` has `at(&self) -> Repo` and `id(&self, bytes: api::Commit) -> ObjectId`. `at` goes from `artifact.rs`, `actions.rs`, `reads.rs`, `contents.rs`, `turn.rs`, `writes.rs`; `id` from `actions.rs`, `contents.rs`, `turn.rs`, `writes.rs`. `rg -n 'fn (at|id)\(' skein/crates/skein-forge/src/forgejo/plans -g '!**/tests/**'` reads only `tables.rs`'s two.
- `failure.rs` has one helper for classify-and-drop, and every site that does both calls it:

  ```rust
  /// The refusal a status means, dropping the repository row where it says the row is stale (forge.md, 3.4).
  pub fn refused(
      asked: Asked,
      status: u16,
      refusal: Option<Refusal>,
      retry: Option<Duration>,
      repository: api::Repository,
      tables: &mut Tables,
  ) -> api::Error
  ```

  The sites are `turn.rs`, `artifact.rs`, `contents.rs`, `reads.rs` and `writes.rs`'s `done`, which then passes the answer through `self.error`. `classify` stays public: tests read it. `failure::read` (`actions.rs`) drops nothing and stays.
- `limit` goes from `reads.rs`'s five `State` variants that hold it; each reader takes `bounds.page_size`. A call's `bounds` are built from `waiting.page_size` on every entry (`skein-forge-connection/src/component/calls.rs`), so they don't change under a call. **If a reader has no `bounds` in reach** without a new parameter on a function already at clippy's limit, that variant keeps its `limit`, and the report says which.
- No test's assertion changes. `heavy cargo nextest run -p skein-forge --lib`, `-p skein-forge-connection --lib` and `-p skein-forge-machine-world` pass.
- Production's net lines fall by about 45.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `temper/docs/plan/debt-24/inputs/debt22-review.md`, finding 6
- `skein/docs/design/forge.md`, 3.4

## Owns and may change

- **Owns:** the files in the header, under `forgejo/plans/`. No GitHub plan and no other crate changes.
- **Tests in `forgejo/plans/tests/`** may change a call's spelling if one reads a removed private function; nothing else.
- **`writes.rs`'s `Writing` isn't yours:** `forgejo-written` takes it after you. Only `at`, `id` and the classify-and-drop in `done` change there.
- A `State` whose only `row` use was `at()` keeps its `row`: this is no state change.

## Steps

1. `at`, `id`; then `refused`; then `limit`. A commit each.
2. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; any `limit` left and why. This is a move: no rule to break by hand.
