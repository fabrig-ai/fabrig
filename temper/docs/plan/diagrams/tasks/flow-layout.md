+++
id = "flow-layout"
part = "flow"
kind = "feature"
needs = ["flow-data", "scene-height"]
files = [
  "skein/crates/skein-ui/src/flow/layout.rs",
  "skein/crates/skein-ui/src/flow.rs",
  "skein/crates/skein-ui/src/tests/flow/layout.rs",
  "skein/crates/skein-ui-web/src/tests/svg.rs",
]
effort = "high"
minutes = 45
+++

# The flow's drawing code

## Goal

`flow::draw` lays a flow's data out for its box's width, left to right where it fits and top to bottom otherwise, so that the web mapping prints each case's scene byte for byte. It reads no height and measures no text.

Acceptance:

- A table test in `skein-ui-web` prints, through `flow::draw` and `svg::draw` with `flow::NAMES`, exactly the bytes of `how-hello-works.640.svg`, `how-hello-works.600.svg` and `review.400.svg`, each but its final newline, for box heights of 0, 74 and 999.
- `how-hello-works` at 240 returns false.
- `flow::draw` returns false for another kind and for malformed data, without a panic, and for a figure past a `u16`: a top-to-bottom flow of 1,100 nodes.
- The same inputs give the same scene, and a scene past `scene_marks` or `scene_bytes` is refused, so the fallback shows.
- A name's width counts scalar values: `é` counts one, and an invalid byte one, as a shell shows U+FFFD.

## Inputs

- `skein/docs/design/ui.md`, 11.5 (whole), 11.1 and 11.2.
- `skein/docs/design/draft/diagrams/README.md`, and its `.svg` cases, read and never edited.
- `temper/docs/plan/diagrams/notes/contracts.md`, *`skein_ui::flow`* and the section on `Scene`.
- `temper/docs/plan/diagrams/notes/common.md`.

## Owns and may change

Owns `flow/layout.rs` and `tests/flow/layout.rs`, which stand as a comment from `flow-data`, and one table test in `skein-ui-web/src/tests/svg.rs`, whose other tests stay. `flow.rs` gains only `pub use layout::draw;`.

Readers: nothing calls `flow::draw` yet. `temper-flow` routes it. It's a `scene::Draw`, the signature in `scene.rs` line 328.

Options, each pre-approved:

- If a case's bytes disagree with `ui.md`, 11.5, the case wins: change the code to match, and name the words in the report.
- `flow-cases` later changes this table to read each case's `.md`, once the reader has landed. Don't wait for it, and don't make the table general for it.

## Steps

1. `flow/layout.rs`, under the strict subset, all arithmetic checked in `u32`, past which it returns false:

   ```rust
   //! Lays a flow out as a scene (ui.md, 11.5).
   pub fn draw(kind: DrawingKind, data: &[u8], width: u16, _height: u16, scene: &mut Scene) -> bool;
   ```

   - `kind != KIND` or `decode` `None` is false.
   - A name's width is `(count × 15 + 1) / 2`, a caption's `(count × 13 + 1) / 2`, with `count` from `<[u8]>::utf8_chunks`.
   - Written `LR`: left to right if it fits, else top to bottom if it fits, else false. Written `TD`: top to bottom if it fits, else false. Nothing is written to the scene until one fits.
   - `scene.set_height` first. A mark's refusal returns false at once.
   - The marks in `ui.md`, 11.5's order: `scene.rect(.., &[BOX])` or `&[BOX, STRONG]`, `scene.text(.., Anchor::Middle, name, &[NAME])` or `&[NAME, STRONG]`, `scene.text(.., caption, &[CAPTION])`, with `Anchor::Middle` left to right and `Anchor::Start` top to bottom, and `scene.path(&[Move, Line, Move, Line, Line], &[ARROW])`, whose five commands take the 25 bytes `scene_bytes` counts.
2. `tests/flow/layout.rs`: rows for false (another kind, malformed, too narrow each way, 1,100 nodes top to bottom), the same scene twice by `Display`, a refused scene, the height said in each direction with and without a caption, and the scalar count (a name wide enough to pass 120 by its count).
3. `skein-ui-web/src/tests/svg.rs`: one table test, `a_flows_scene_prints_its_case_byte_for_byte`, its nodes written out in rows (`flow::encode` on the two cases' nodes, with `include_str!("../../../../docs/design/draft/diagrams/how-hello-works.640.svg")` and its siblings, `.trim_end()`), for each height. Limits are `LIMITS` with `scene_marks: flow::marks(4)` and `scene_bytes: flow::scene_bytes(4, 256)`.
4. Narrow tests: `heavy cargo nextest run -p skein-ui --lib -E 'test(flow)'`, `heavy cargo nextest run -p skein-ui-web --lib -E 'test(a_flows_scene)'`. Then the closing checks of your text, on `-p skein-ui -p skein-ui-web`, departing from your text's *direct dependents*: no signature any dependent reads changes. Commit and push.

## Report

The implementer's report, with the rule you broke by hand for each table, and any word of `ui.md`, 11.5 a case's bytes overruled.
