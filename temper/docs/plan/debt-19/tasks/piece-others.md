+++
id = "piece-others"
part = "link-translate"
kind = "migration"
needs = ["piece-within"]
files = ["jig/crates/jig-protocol-worker-link/src/translate.rs", "jig/crates/jig-protocol-engine-clients/src/translate/operations.rs", "jig/crates/jig-protocol-engine-clients/src/translate/asks.rs", "jig/crates/jig-protocol-engine-clients/src/translate/llm.rs", "jig/crates/jig-protocol-store/src/codec/settings.rs", "jig/crates/jig-protocol-store/src/codec/tasks/pieces.rs", "jig/crates/jig-protocol-store/src/codec/people/pieces.rs"]
effort = "medium"
minutes = 25
+++

# jig: the piece loops outside the link read the same helper

## Goal

The other crates' part of `jig/docs/development/debt/D008-link-translate-split-by-number.md`: three loops and five wrappers. Where a site splits and sums, it reads `skein_codec::piece::split_within`; where it counts first, `count` and `counted`. A site that keeps its own bound and error stays as it is, and your report says why. Not one answer changes.

Acceptance:

- `rg -o 'piece::split\(&mut reader\)' jig/crates jig/examples -g '!**/tests/**' -g '!**/tests.rs' -g '!**/generated/**' | wc -l`, once `link-loops` has landed, reads at most 3, each named in your report with its reason. Measure your own files: `rg -n 'piece::split\(' ` over the files in `files`.
- `heavy cargo nextest run -p jig-protocol-worker-link -p jig-protocol-engine-clients -p jig-engine-clients-world -p jig-protocol-store -p jig-store-codecs-world -p jig-store-world` passes, no assertion changed. The store's formats and its patch levels don't change, and neither do its goldens.
- Net production falls (estimate -15), or holds, and if a site gains by the change, leave it.

## Inputs

- `temper/docs/plan/debt-19/notes/common.md`
- `jig/docs/development/debt/D008-link-translate-split-by-number.md`
- `skein/docs/design/codec.md`, 3
- `skein/crates/skein-codec/src/piece.rs`, as `piece-within` left it

## Owns and may change

- **Owns:** the files in `files`. A path that has moved is the same file.
- **The sites, by reading:**
  - `jig-protocol-worker-link/src/translate.rs`, about line 635, returns a sentinel and decodes nothing. It isn't `Result`-based, and likely stays.
  - `jig-protocol-engine-clients/src/translate/operations.rs`, about line 119, counts with `while reader.remaining() != 0`, then decodes. Its errors are `Untranslatable { path }`. `piece::count` may replace the counting.
  - `jig-protocol-store/src/codec/settings.rs`, about line 1322, loops to `VERDICT_FIELDS` with `admit_rule` and a `Path::VerdictFields` problem. Likely stays.
  - The wrappers: `asks.rs`, about line 397; `llm.rs`, about line 290; `operations.rs`, about line 245; `tasks/pieces.rs`, about line 12; `people/pieces.rs`, about line 386.
- **Stay off:** `jig-protocol-store/src/codec/tasks/record.rs` and `jig-protocol-engine-clients/src/translate/{queries,task}.rs`: other plans are changing them.
- **temper's own `split` calls** (`temper-settings-form`, `temper-protocol-store`) aren't this item's: its scope is jig.

## Steps

1. Each site; compile; test.
2. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-worker-link -p jig-protocol-engine-clients -p jig-protocol-store --all-targets -- -D warnings`; the tests above. The citations check. Commit and push.

No test pins a new rule, so nothing here needs breaking by hand.

## Report

The common note's items, each site that stayed and why, and the net lines.
