+++
id = "shapes"
part = "shapes"
kind = "feature"
needs = ["paths"]
files = ["hearth/crates/hearth-domain-render/src/tests/fixtures/hearth.rs", "hearth/crates/hearth-domain-render/src/tests/fixtures/debian.rs", "hearth/crates/hearth-domain-render/src/tests/compose/hearth.rs", "hearth/crates/hearth-domain-render/src/tests/debian.rs", "hearth/crates/hearth-domain-render/src/tests/fixtures/entries.rs", "hearth/tests/render/tests/suite/oracle.rs", "hearth/tests/render/tests/suite/oracle/engine.rs", "hearth/tests/builder/tests/suite/properties.rs", "hearth/docs/development/debt/D012-render-test-shapes.md"]
effort = "medium"
minutes = 45
+++

# Tables that read as tables

## Goal

The test shapes the render run left: six unnamed slots become named fields, one name stops holding two values, the
oracle's rows hold plain text, and the six numbered cut-pipe tests become one table of the same six tests.

Acceptance:

- `git grep -n '\[Option<&str>; 6\]\|\[None; 6\]' -- 'hearth/crates/hearth-domain-render'` finds nothing.
- `fixtures/entries.rs` defines neither `RANGE_BASE` nor `RANGE_COUNT`: `git grep -nE 'const RANGE_(BASE|COUNT)\b' --
  hearth/crates/hearth-domain-render` finds the two in `fixtures/collisions.rs` alone.
- D012's **Measure** reads 0: `git grep -E 'Outcome::Text\(Cow::Borrowed' -- hearth/tests/render | wc -l`.
- `tests/builder/tests/suite/properties.rs` has one table of cut-pipe batches, and
  `heavy cargo nextest list -p hearth-builder-world --test suite` is unchanged, name for name.
- Every assertion stands: `heavy cargo nextest run -p hearth-domain-render -p hearth-render-world -p
  hearth-builder-world` passes, and the fuzzy profile for `-p hearth-render-world`, where the oracle's fuzzy tests are.
  The counts equal those at your base.
- `hearth/docs/development/debt/D012-render-test-shapes.md` is gone.

## Inputs

- `temper/docs/plan/debt-14/notes/common.md`
- `hearth/docs/development/debt/D012-render-test-shapes.md`
- `hearth/docs/design/protocol/format.md`, 3.2: the six layers

## Owns and may change

- **Owns:** the files in the spec's `files` list, for the changes below.
- **May change:** nothing else. `hearth-domain-render/src/tests/fixtures.rs` re-exports by glob, so `SixLayers` needs no
  line there; if it does, add that one.
- **Not to touch:** `member_layers`' five slots: outside the item. `fixtures/collisions.rs`, whose `RANGE_BASE` and
  `RANGE_COUNT` keep their names and values. `hearth-domain-render/src/tests/fixtures/digests.rs`, which `digits` changes.
- **Options:**
  - If a table reads worse with `..SixLayers::NONE`, a small `const fn` constructor per slot is fine, as long as a row
    says a layer by name.
  - If another file under `hearth/tests/render` holds the Measure's form, it takes the same constructor.

## Steps

1. **`SixLayers`,** in `fixtures/debian.rs`, named as `compose::Layer` names them (`fixtures::schemas::Layers` is
   another thing, so the name differs):

   ```rust
   /// What each of format.md 3.2's six layers holds; a row says the ones it sets.
   #[derive(Clone, Copy)]
   pub struct SixLayers<'a> {
       pub fleet: Option<&'a str>,
       pub base: Option<&'a str>,
       pub member: Option<&'a str>,
       pub environment: Option<&'a str>,
       pub base_env: Option<&'a str>,
       pub base_env_member: Option<&'a str>,
   }
   impl SixLayers<'_> {
       pub const NONE: Self = /* every slot None */;
   }
   ```

   `debian_layers(values: SixLayers<'_>)` and `target_with(placement: &str, values: SixLayers<'_>)` take it, and destructure
   it in place of the array. In `fixtures/hearth.rs` the locals `host` and `host_member` take the fields' names.
   `tests/compose/hearth.rs`'s `SETTINGS`, `CONTAINER_LAYERS` and `NULLS` become `&[SixLayers<'_>]` of
   `SixLayers { fleet: Some("settings"), ..SixLayers::NONE }`, and `tests/debian.rs`'s three calls name their layers. Rows keep
   their order, so the `row {row}` messages keep their meaning.
2. **Ranges.** In `fixtures/entries.rs`, `RANGE_BASE` becomes `ENTRY_RANGE_BASE` and `RANGE_COUNT` becomes
   `ENTRY_RANGE_COUNT`, with their uses in that file. They are 42 and 2, a typed field's range; `collisions.rs`'s are
   format.md's bound, a different thing. Nothing outside `entries.rs` reads the first pair.
3. **The oracle's rows.** In `oracle/engine.rs`:

   ```rust
   impl Outcome {
       /// A row's expected text.
       pub const fn text(expected: &'static str) -> Self {
           Self::Text(Cow::Borrowed(expected))
       }
   }
   ```

   Every `Outcome::Text(Cow::Borrowed("..."))` in `oracle.rs` and `oracle/engine.rs` becomes `Outcome::text("...")`, and
   `oracle.rs` loses `use std::borrow::Cow;`. `outcome`'s `Cow::Owned` stays: a run's text is not a row's. This is a
   mechanical replace: no row's text changes. `git diff --word-diff` must show nothing else.
4. **The cut-pipe tests.** The six are split on purpose: `CUT_BATCHES_PER_CASE` in
   `tests/builder/src/fixtures/properties.rs` says "Four cut schedules per test", so that the batches run apart. They
   stay six tests with their six names. Folding them would be one test of six times the length, past hearth's slow
   period. So the table writes them, in `tests/builder/tests/suite/properties.rs`:

   ```rust
   /// One test per batch, so that the batches run apart.
   macro_rules! cut_pipe_batches {
       ($($name:ident = $batch:literal,)*) => {
           $(#[test]
           fn $name() {
               fixtures::properties::cut_pipe_digests($batch);
           })*
       };
   }
   cut_pipe_batches! {
       generated_repositories_have_the_render_worlds_digests_through_cut_pipes = 0,
       generated_repositories_have_the_render_worlds_digests_through_cut_pipes_1 = 1,
       // to _5
   }
   ```

   Keep the six names, so the lists compare.
5. Delete D012's file. The tests above, `cargo fmt`, the ratchet, clippy on the three packages. Commit and push.

## Report

The list comparison, the counts before and after, what the Measure reads. No test here pins a new rule: shapes only, so
no bite. Net lines, which should be negative.
