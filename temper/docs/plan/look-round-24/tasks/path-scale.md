+++
id = "path-scale"
part = "scale"
kind = "feature"
needs = ["scene-width"]
files = [
  "temper/crates/temper-client-view/src/paths/layout.rs",
  "temper/crates/temper-client-view/src/paths/layout/line.rs",
  "temper/crates/temper-client-view/src/paths/layout/marks.rs",
  "temper/crates/temper-client-view/src/tests/paths.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 60
+++

# A path across is the request's picture scaled; an open card stands 14 under it

## Goal

The request's tasks 3 and 4 (`temper/docs/plan/look-round-24/inputs/request.md`). At 1000 pixels the box is 658 wide and the path is drawn again at 658 by 140, where the request's is its 720 by 140 picture scaled, 658 by 127.94, which puts the open card 12 pixels low. On a phone the open card sits right under the drawing, where the request's stands 14 pixels under it. Across, under 720, the path is now drawn 720 wide and says so (`Scene::set_width`, from `scene-width`), so that the page scales it; and an open stop grows the scene by what is 14 pixels in the box, across and down the page alike.

Acceptance, by `temper/docs/design/client/README.md`, 5.16, *Its layout* and *Pressed*, as this cut wrote them:

1. **The drawn width `A`:** in `paths/layout.rs`, `Layout` gains the scene's own width (`drawn: u32`, whole pixels) and the box's (`width: u32`). Across is chosen exactly as today, on the box: `width >= 560 && width - 80 >= gaps * 48`. Across, `A` is the box's width from 720 up and 720 under it; down the page, `A` is the box's width. Every across coordinate that today reads the box's width reads `A`: the span (`A - 80`) and so the stops' places, the landing bar's `A - 8`, the dotted line from 0 to `A`. Down the page nothing moves.
2. **The scene says its width** with `scene.set_width(A)` where it draws across in a box under 720, and only there, beside `set_height` in `paint`. So at 658 the SVG is `width="720" height="140" viewBox="0 0 720 140"` and the page scales it to 658 by 127.94; at 720 and over, and down the page, it says no width and draws as today.
3. **An open stop's room:** the scene's height grows at its foot by `g`, the box's 14 pixels in the scene's own: across `g = round(14 * A / W)` (half up; 14 at 720 and over, 15 at 658), down the page `g = 14`. Across that room holds the caret: its points `(x - c, H)`, `(x, H - c)`, `(x + c, H)`, `H` the scene's height (`140 + g`), `x` the open stop's place, `c = round(6 * A / W)` (6 at 720, 7 at 658); down the page there is no caret. The closed heights are today's: 140 across, `end + 14` down the page. Use checked `u32` arithmetic and the module's hundredths where they serve; a rounding helper of your own is fine.
4. **The rows, no new test function** (`temper/crates/temper-client-view/src/tests/paths.rs`):
   - `the_three_look_paths_keep_judgement_above_code_and_each_stop_in_words` (about line 76) draws at `[720, 658, 388]`: at 658 across, `scene.width() == Some(720)`, height 140 closed and 155 open, the dotted line `(0, 72)` to `(720, 72)`, every mark where it stands at 720 but the caret, which at 658 is `[x - 7, 155, x, 148, x + 7, 155]`; at 720 `scene.width() == None` and every assertion as today; at 388, open, the height is today's plus 14 and the dotted line still ends 14 above the last bar's foot as today (its end `end`, not the scene's height less 14: restate that assertion from `end`), `scene.width() == None`.
   - `path_bounds_refuse_bad_data_and_draw_any_keeps_flows`'s threshold rows (about line 444, `(559, APPROVE, 388)`, `(560, APPROVE, 140)`, `(607, DRAFT, 508)`, `(608, DRAFT, 140)`) keep their heights and assert `scene.width()`: `Some(720)` for 560 and 608, `None` for 559 and 607.
   - The open-stop rows (about line 478, drawn at 720, height 154 or 140) keep theirs.
5. **Who reads the scene,** checked and left as they are: the gallery's tests draw at 1000 (`temper/tools/gallery/src/tests.rs`, about line 3525), where `A` is 1000 and nothing changes; the web host's tests (`temper/crates/temper-client-shell-web/src/host_tests.rs`, about line 532) measure at 640, 240 and 640 and read `data-drawn` and the path's classes, which still hold; the journeys print no path. Run them so (acceptance 7).
6. **Status:** delete the **Not yet** entry naming `path-scale` in `temper/docs/design/client/README.status.md` (5.16). No other design line.
7. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-view`, `-p temper-client-shell-web` and `-p temper-client-gallery`; clippy on `temper-client-view`.

One commit or more, each with the trailer `Look-request: ask-temper-ways`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-24/notes/common.md`.
- `temper/docs/plan/look-round-24/inputs/request.md`, tasks 3 and 4.
- `temper/docs/design/client/README.md`, 5.16, *Its layout* and *Pressed*; 3.5, *Ask temper's page*, *a stop's card* (the card follows the drawing with no gap of its own).
- skein's `skein/docs/design/ui.md`, 11.2, *Its own width*, and `ui-web.md`, 8, *A scene that says its width* (from `scene-width`).
- The look request's pages (`notes/common.md`): `01.html` and `04.html`'s wide and narrow SVGs, and the overlay's `.temper-ways-zoom { margin: 14px 0 0 }`.

## Owns and may change

Owns the layout's three files, the path tests and the status entry. Not `paths.rs`'s data or fallback (`look-round-24`'s terminal task may change the fallback's name), not `temper.css` (the zoom's `margin: 0` stays: the scene's room is the gap), not the gallery.

Options, each decided:

- **A mark's place at 658 differs from its place at 720** in the main test because some code reads the box's width rather than `A`: it is a miss of acceptance 1; fix the layout, not the row.
- **The hit areas** (`marks.rs`, 60 by 52 and 60 by 44) stay in the scene's pixels and so scale with it: no change.
- **The web host's tests** assert an SVG's `width` or `viewBox` at 640 (step 5 finds one): it now reads 720 there; change that assertion to the drawn width, keeping what else it checks, and name it.

## Steps

1. Read the Inputs, `paths/layout.rs`, `layout/line.rs`, `layout/marks.rs` and the tests named above.
2. The drawn width and the scene's width (acceptance 1, 2); bite: make `paint` say no width, see the 658 row fail on `scene.width()`, restore.
3. The open room and the caret (3) and the rows (4); bite: drop the growth down the page, see the 388 open row fail on the height, restore.
4. The readers (5), the status entry (6), the runs (7); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main` after committing.
5. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| At 1000, the path the request's picture scaled, 658 by 127.94 | landed (`paths/layout.rs`: across under 720 drawn at 720, the scene saying its width) |
| At 1000, the open card where the request's is | landed: the scene's foot grows 15 of its pixels, about 14 of the box's; the caret in it |
| On a phone, the open card 14 under the drawing, no caret | landed (`paths/layout.rs`: down the page an open stop grows the scene 14) |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4): focused tests establish no rendered equality |

## Report

The implementer's report: the layout's fields as written, the rows and the bites with their failures, the readers checked, the runs' counts, and the account as committed.
