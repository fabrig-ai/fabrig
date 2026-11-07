+++
id = "scene-height"
part = "scene-height"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-ui/src/scene.rs",
  "skein/crates/skein-ui/src/tests/scene.rs",
  "skein/crates/skein-ui-web/src/svg.rs",
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/tests/svg.rs",
  "skein/crates/skein-ui-web/src/tests/page.rs",
  "skein/crates/skein-ui-web/src/tests/fixtures.rs",
  "skein/crates/skein-ui-bundle/skein-ui.css",
  "temper/docs/design/client/journeys/skein-ui.css",
]
effort = "high"
minutes = 40
+++

# A scene says its height, and the web puts it in the figure's flow

## Goal

A scene may say its own height. The web mapping then gives its SVG that height, and the SVG sits in the figure's flow, so the figure is as high as its scene. The page part reserves no scene until it draws. `skein-ui.css` gains the rule for that, and a flow's plain default look. Scenes that say no height print exactly what they print today.

Acceptance:

- `Scene::set_height` and `Scene::height` as the contract has them; `clear` forgets the height; `Display` prints `height N` first only where the scene says one.
- A scene that says a height: `svg::draw` prints that height in `height` and `viewBox`, with the box's width, and nothing else differs. A scene that says none prints as today: `golden/scene.svg`, `golden/drawing.html` and `golden/drawing-changed.html` don't change.
- `Page`: after `resized`, a figure whose scene says a height carries `data-drawn` and `data-scene-height`; one whose scene says none carries `data-drawn` alone. Resized again to the scene's own height, the page's HTML is the same.
- A page that has built a tree with no drawing, and a page before its first `resized`, hold no `Scene` (`self.scene` is `None`; the test's module is outside `page`, so the field is `pub(crate)`).
- `skein-ui.css` holds the rules below, and `temper/docs/design/client/journeys/skein-ui.css` is its reprint: `the_committed_journeys_are_their_print` passes.

## Inputs

- `skein/docs/design/ui.md`, 11.2, *Its size*, and 11.4.
- `skein/docs/design/ui-web.md`, section 8, *A scene that says its height* and *No cost for no drawing*, and section 12.
- `temper/docs/plan/diagrams/notes/contracts.md`, the first two sections and *The tokens' selector*.
- `temper/docs/plan/diagrams/notes/common.md`.

## Owns and may change

Owns the files in the header, each only for what's below.

Readers, surveyed by `rg` across the trees:

- `Scene::new` is called only in `skein-ui-web/src/page.rs`, at construction, and in tests. `self.scene` is read in `draw` and at `Event::Drawing`, both in `page.rs`.
- `svg::draw` is called in `page.rs` and in `tests/svg.rs`. Its signature stays.
- `data-drawn` is written only in `page.rs`, `own_element`. It's read by `skein-ui.css`, `temper.css` near line 141 (unchanged) and two goldens (unchanged). The browser half (`page/web.rs`, `attributes`) sets and removes a mapped element's attributes generally, so it needs no change.
- `skein/tests/ui/tests/suite/memory.rs` and `skein/crates/skein-ui/src/tests/scene.rs` read a scene's fields, not its size. A new private field needs nothing from them.

Departs from the implementer's text: `temper/docs/design/client/journeys/skein-ui.css` is a generated file under `docs/design/`, and it's yours to reprint.

Options, each pre-approved:

- If reprinting changes any file under `temper/docs/design/client/journeys/` other than `skein-ui.css`, that's stale from elsewhere: keep only `skein-ui.css` in the commit, and name the others in the report.
- If `Page` needs a new field to hold the limits for the later `Scene::new`, keep a `Limits` copy; it is `Copy`.

## Steps

1. `scene.rs`: the field, `set_height`, `height`, `clear`, `Display`.
2. `svg.rs`: `let height = scene.height().unwrap_or(height);` before its uses.
3. `page.rs`: `scene: Option<Scene>`, made in `draw`. `Event::Drawing`, which draws first, reads it after. `Held` records whether the drawn scene says a height, and `own_element` sets `data-scene-height` beside `data-drawn`.
4. `skein-ui.css`, directly after the `[data-drawn] > svg :is(..)` rule:

   ```css
   /* A scene that says its height sits in its figure's flow (ui-web, 8). */
   [data-scene-height] > svg {
     position: static;
     display: block;
   }

   /* A flow's plain default (ui.md, 11.5): an owner's stylesheet may restyle it. */
   [data-drawn] > svg .skein-ui-flow-name,
   [data-drawn] > svg .skein-ui-flow-caption {
     fill: currentColor;
     stroke: none;
   }

   [data-drawn] > svg .skein-ui-flow-caption {
     font-size: smaller;
     opacity: .7;
   }

   [data-drawn] > svg .skein-ui-flow-box.skein-ui-flow-strong {
     fill: currentColor;
   }

   [data-drawn] > svg .skein-ui-flow-name.skein-ui-flow-strong {
     fill: Canvas;
   }
   ```

   Boxes and arrows keep the marks' default, outlined in the text's colour on no fill.
5. Reprint: `heavy cargo run -p temper-client-journeys`, which drives no browser.
6. Tests, as rows beside their siblings:
   - `tests/scene.rs`: a scene says its height and `clear` forgets it, in `a_mark_past_scene_marks_refuses_the_scene`'s neighbourhood, and a refused scene says none.
   - `tests/svg.rs`: one test with a scene that says a height, comparing the printed `<svg ...>` head.
   - `tests/page.rs`: a look whose draw says a height, beside `drawing_look` in `fixtures.rs`. One test walks: no scene before `resized`, then `data-scene-height`, then the same HTML resized to that height. `DRAWING` and `drawing` there are the fixture to use.
7. Narrow tests: `heavy cargo nextest run -p skein-ui --lib -E 'test(scene)'`, `heavy cargo nextest run -p skein-ui-web --lib -E 'test(scene) | test(draw)'`, `heavy cargo nextest run -p temper-client-journeys --lib -E 'test(=the_committed_journeys_are_their_print)'`. Then the closing checks of your text. Commit and push.

## Report

The implementer's report, with what you broke by hand for each new test, and what the reprint changed.
