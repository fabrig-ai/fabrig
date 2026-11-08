+++
id = "scene-width"
part = "scale"
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
  "skein/docs/design/ui.status.md",
  "skein/docs/design/ui-web.status.md",
  "temper/docs/design/client/journeys/skein-ui.css",
]
effort = "high"
minutes = 50
+++

# A scene may say its width, scaled to its box

## Goal

skein's half of the request's task 3 (`temper/docs/plan/look-round-24/inputs/request.md`): at 1000 pixels the look request's path is its 720 by 140 picture scaled to its box, 658 by 127.94; temper's is drawn again at 658 by 140, since a scene is always its box's width. A scene may now say its own width beside its height: it is drawn at that size and scaled to its box's width, keeping its proportions. temper's path uses it in `path-scale`, which needs this task.

Acceptance, by skein's `skein/docs/design/ui.md`, 11.2, *Its own width*, and `skein/docs/design/ui-web.md`, 8, *A scene that says its width*, and 12 (`skein-ui.css`), all written by this cut:

1. **`skein_ui::scene::Scene`** (`skein/crates/skein-ui/src/scene.rs`) gains, beside `height`:

   ```rust
   width: Option<u16>,

   /// Say this scene's width, drawn at it and scaled to its box's (ui.md, 11.2).
   pub fn set_width(&mut self, width: u16);   // ignored once refused, as set_height
   /// The width it said, if it said one.
   #[must_use]
   pub const fn width(&self) -> Option<u16>;
   ```

   `clear` and a refusal forget it as they forget the height; `Display` prints `width W` on its own line before `height H` when said. Nothing else of the scene changes, and no drawing code changes: a scene that says no width is drawn as today.
2. **`skein_ui_web::svg::draw`** (`svg.rs`): `let width = scene.width().unwrap_or(width);` beside the height's line, so that the SVG's `width`, `height` and `viewBox` are the scene's own where it says them. Its signature stays.
3. **The page part** (`page.rs`): `Held` gains `scene_width: bool`, set and cleared beside `scene_height` in `draw`; `own_element` sets `data-scene-width` on the figure when it holds, beside `data-scene-height`. **A press** (`Event::Drawing`): where the drawn scene says its width `S` and the box's measured width is `W` (`held.size`), the point passed to `act_at` is `x * S / W` and `y * S / W`, in `u32` with checked arithmetic, rounded down, saturating to `u16::MAX`; a box of width 0 passes the point unscaled. A scene that says no width passes the point as today.
4. **`skein-ui.css`:** directly after the `[data-scene-height] > svg` rule,

   ```css
   /* A scene that says its width scales to its figure's (ui-web, 8). */
   [data-scene-width] > svg {
     position: static;
     display: block;
     width: 100%;
     height: auto;
   }
   ```

   No other rule changes; the flows' rules keep their `[data-drawn] > svg` prefix.
5. **Rows, no new test function:**
   - `a_scene_says_its_height_until_cleared_or_refused` (`skein-ui/src/tests/scene.rs`) is renamed `a_scene_says_its_size_until_cleared_or_refused` and says a width too in each row: `set_width(1)` then `set_width(width)` beside the height, `width()` asserted, `Display` printing `width W\nheight H\n` first, and both forgotten after `clear` or a refusal. Its rows' widths: `0`, `720`, `u16::MAX`, one per height row.
   - `a_scene_uses_its_own_height_with_the_box_width` (`skein-ui-web/src/tests/svg.rs`) is renamed `a_scene_uses_its_own_size_or_its_boxs` and takes rows `(width: Option<u16>, height)`: `(None, 0)`, `(None, 74)`, `(None, u16::MAX)` as today, and `(Some(720), 140)`, which prints `<svg aria-hidden="true" width="720" height="140" viewBox="0 0 720 140"></svg>` for a box of 100 by 50.
   - `a_drawing_reserves_its_scene_on_resize_and_says_its_height` (`skein-ui-web/src/tests/page.rs`, about line 425) is renamed `a_drawing_reserves_its_scene_on_resize_and_says_its_size` and gains a third row, a new fixture `scene_size_look()` in `tests/fixtures.rs` (beside `scene_height_look`, a drawing saying width `DRAWING_WIDTH = 720` and height `DRAWING_HEIGHT`): its HTML holds `data-scene-width=""` and `data-scene-height=""` and the SVG `width="720" height="74" viewBox="0 0 720 74"` for a box 100 wide; the other two rows hold no `data-scene-width`; measuring the figure again at its scaled height changes nothing, as the height's row checks today.
   - **The press's point:** an existing drawing-press test of `page.rs` (find it with `rg -n 'Event::Drawing' skein/crates/skein-ui-web/src/tests`) gains a row on `scene_size_look()` whose box is 360 wide: a press at `(90, 10)` reaches `act_at` as `(180, 20)`; read the test first and choose the row's shape from what it asserts (an act's appended point is the simplest witness). If no test there presses a drawing, add the row to `a_drawing_reserves_its_scene_on_resize_and_says_its_size` and say so.
6. **Status:** delete the **Not yet** entries naming `scene-width` in `skein/docs/design/ui.status.md` (11.2) and `skein/docs/design/ui-web.status.md` (8 and 12). No other design line.
7. **The journeys' copy:** `temper/docs/design/client/journeys/skein-ui.css` is `skein_ui_bundle::STYLESHEET` written beside the journeys at each print (`temper/tools/journeys/src/print.rs`, `beside`), so the new rule makes it stale: regenerate it with `heavy cargo run -p temper-client-journeys` (no arguments; it reprints the saved recordings too, which this task changes in no other byte: a print that changes is a red, not stale) and commit the copy alone.
8. Runs through `heavy`, counts in the report: `cargo nextest run -p skein-ui`, `-p skein-ui-web`, `-p skein-ui-bundle` (its tests read `skein-ui.css`) and `-p temper-client-journeys`; `cargo check -p temper-client-view --tests` and `-p temper-client-gallery --tests` (they build `Scene`); clippy on the three skein packages.

## Inputs

- `temper/docs/plan/look-round-24/notes/common.md`.
- `temper/docs/plan/look-round-24/inputs/request.md`, task 3.
- `skein/docs/design/ui.md`, 11.2, *Its size* and *Its own width*; 11.3 (a press's act and its point).
- `skein/docs/design/ui-web.md`, 8, *A scene that says its height*, *A scene that says its width*, *Acted on*; 12, `skein-ui.css`'s list.
- `skein/AGENTS.md` (skein's clippy rules).

## Owns and may change

Owns the files above, `temper/docs/design/client/journeys/skein-ui.css` by its tool alone (acceptance 7). Reader files, call sites only: none known outside skein; a `Scene` built elsewhere (`rg -n 'Scene::new' --type rust`) needs no edit, since the field is private and starts `None`.

Options, each decided:

- **`skein-ui-bundle`'s stylesheet test** (`skein/crates/skein-ui-bundle/src/tests.rs`, about line 160, which checks the classes the stylesheet names and that flow rules start `[data-drawn] > svg `): the new rule names no class, so it passes; if another assertion there trips on it, extend that assertion for this rule alone and say so.
- **A golden HTML or SVG of `skein-ui-web` changes** with the new attribute: only rows on `scene_size_look()` may print it; if any other golden changes, that is a red, not stale.
- **The web shell's own code** (`skein/crates/skein-ui-web/src/page/web.rs`) measures the press in the figure's box as today; the scaling is `page.rs`'s, so `web.rs` does not change. If it must (the box's width is not `held.size`'s), change it at the press alone and say why.

## Steps

1. Read the Inputs, `scene.rs`'s height, `svg.rs`, `page.rs`'s `draw`, `own_element` and `Event::Drawing`, and the tests named above.
2. The scene's width and its rows (acceptance 1, the first row of 5); bite it: make `clear` keep the width, see the row fail, restore.
3. The SVG and the page part (acceptance 2, 3), the stylesheet (4) and their rows (5); bite the press: drop the scaling, see the press row fail, restore.
4. The status entries (6); the journeys' copy (7); the runs (8); `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main` after committing.
5. Commit with the trailer `Look-request: ask-temper-ways` and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| A scene says its width (`Scene::set_width`, `width`) | landed (`skein-ui`, `scene.rs`) |
| Its SVG at its own width and height, `data-scene-width`, a press's point scaled | landed (`skein-ui-web`, `svg.rs`, `page.rs`) |
| Scaled to the figure's width (`[data-scene-width] > svg`) | landed (`skein-ui.css`) |
| temper's path at 1000, 658 by 127.94 | deferred to `path-scale`, this plan's, which needs this task |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4) |

## Report

The implementer's report: the API as written, the rows and the bites with their failures, the runs' counts, and the account as committed.
