+++
id = "words-wire"
part = "words"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema",
  "temper/crates/temper-client-protocol-wire/src/generated/v1.rs",
  "temper/crates/temper-client-protocol-wire/golden/**",
  "temper/crates/temper-client-protocol-wire/src/tests.rs",
  "temper/crates/temper-protocol-store/schema/store-v11.schema",
  "temper/crates/temper-protocol-store/src/generated/v11.rs",
  "temper/crates/temper-protocol-store/golden/v11/**",
  "temper/crates/temper-protocol-store/src/records/settings.rs",
  "temper/crates/temper-service/src/family/ways.rs",
  "temper/tools/gallery/src/worlds/ways.rs",
  "temper/crates/temper-client-view/src/tests/ask.rs",
  "temper/crates/temper-client-view/src/tests/how.rs",
  "temper/crates/temper-client-view/src/tests/toast.rs",
  "temper/crates/temper-client-domain/src/tests/forge_stock_ways.rs",
  "temper/crates/temper-client-domain/src/tests/pages/how.rs",
  "temper/tests/codecs/tests/suite/fuzzy/store.rs",
]
effort = "medium"
minutes = 60
+++

# The words on the wire and in the store: the records, nothing filled

## Goal

The client wire and the store gain the records the words travel and rest in, so
that the tasks after this one build on them: `CharterWords` and `ProjectWords` on
both, a stock way's `headline` and the ways document's `words` on the wire, and
the store's setting value's two variants. Nothing fills them yet: the service
sends an empty headline and no words, and the store answers its two new variants
`Undecodable` until `words-value` gives the forge its values.

Acceptance:

1. **The client wire's schema**
   (`temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema`):
   exactly `temper/docs/plan/ui-p3e-words/notes/contracts.md`, section 2: the
   records `CharterWords` and `ProjectWords` after `ProjectWays`; `StockWay`
   with `headline: text 120` after `name`; `WaysPanel` with `words: option
   ProjectWords` after `own`. No family version moves (a stored format and the
   wire change in place, `notes/common.md`).
2. **The store's schema** (`store-v11.schema`): the same two records after its
   `ProjectWays`, and `SettingValue`'s variants `line: CharterWords` and `words:
   ProjectWords`, last, as section 2 has them.
3. **Generated, never by hand:** each schema's generated Rust and goldens by the
   codecs world's drift test, `TEMPER_BUNDLE=none SKEIN_CODEGEN_WRITE=1 heavy
   cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'`,
   then the same without `SKEIN_CODEGEN_WRITE`, passing.
4. **The stand-ins,** each a comment naming the task that replaces it:
   - the store's codec (`temper-protocol-store/src/records/settings.rs`): the
     arms the new `SettingValue` variants force answer `Undecodable` on decode
     (`words-value` replaces them);
   - the service's `ways_panel` (`temper-service/src/family/ways.rs`): each
     `StockWay` an empty `headline` (`way-headlines`), and `words: None`
     (`project-words-read`).
5. **Every builder of `StockWayParts` and `WaysPanelParts`** takes the new
   fields, `headline` empty and `words: None`: the gallery's `worlds/ways.rs`;
   `temper-client-view`'s `tests/ask.rs`, `tests/how.rs`, `tests/toast.rs`;
   `temper-client-domain`'s `tests/forge_stock_ways.rs` and
   `tests/pages/how.rs`. A reader that matches the store's `SettingValue` takes
   the stand-in's arm (the codecs world's `fuzzy/store.rs`, if it enumerates
   the variants: give each new one a generator beside `ways`').
6. **Rows,** no new test function: the client wire's round trip of records
   (`temper-client-protocol-wire/src/tests.rs`, where `ProjectWays` or a
   `WaysPanel` is round-tripped), a `ProjectWords` at 120 and 480 bytes and a
   `WaysPanel` with `words` some; nothing in the store's tests until
   `words-value`.
7. **Unchanged behaviour:** the focused tests of `-p temper-client-protocol-wire`,
   `-p temper-protocol-store`, `-p temper-service`, `-p temper-client-domain`,
   `-p temper-client-view`, `-p temper-client-gallery` and `-p
   temper-client-journeys` pass. If a journey's
   focused replay check fails for the wire's change, regenerate it with
   `temper-client-journeys again` (`notes/common.md`) and name it; a gallery
   golden or print likewise, by its own tool.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.2 (*A setting's words*) and
  13.10 (the `ways` read).
- `temper/docs/design/client/README.md`, 4.6 (`WaysPanel`, `StockWay`).
- `temper/docs/design/protocol/store.md`, section 2, family 20 `Setting`.
- `temper/docs/plan/ui-p3e-words/notes/contracts.md`, sections 1 and 2.
- `temper/docs/plan/ui-p3e-words/notes/common.md`.

## Owns and may change

- Owns: both schemas' lines above, their generated code and goldens, the
  stand-ins and the builders' new fields.
- Compile-forced, the new fields or arms alone, in any file of temper's tree a
  `StockWayParts`, `WaysPanelParts` or the store's `SettingValue` reaches beyond
  those listed; name each in the report.
- The schemas are also ui-reaches' and look-round-25's (`plan.md`, *Overlaps*):
  if a catch-up meets their lines, keep both sides' records and fields in the
  schema and regenerate; never hand-merge generated code or a golden.
- Not yours: the forge's values (`words-value`), the ways' headlines
  (`way-headlines`), the root's words (`project-words-read`) and the drawing
  (`how-words`).

## Steps

1. Read the inputs; `ProjectWays`, `StockWay` and `WaysPanel` in both schemas and
   their builders (`rg -n 'StockWayParts|WaysPanelParts' temper`).
2. The schemas; regenerate (acceptance 3); the stand-ins and builders until
   `heavy cargo check --tests` passes for every package of acceptance 7, one at a
   time.
3. The wire's rows; the runs of acceptance 7.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-client-protocol-wire -p temper-protocol-store -p temper-service
   --all-targets -- -D warnings`, with every other package you changed.
5. Commit (why, in the message); `python3 -I -B tools/citations/citations.py --base
   origin/main`; push.

## Report

As the implementer's text says, and: the regenerated files; each compile-forced
file; the encoded size of the largest `ProjectWords`; anything regenerated beyond
the schemas'.
