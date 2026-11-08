# look-round-24: the ways page's render-check misses

The request: `inputs/request.md`, in place of `look-round-23`, whose cut built nothing. The render check after `look-ask-ways` found eight misses of the look request `ask-temper-ways` (`98322b3565` on `origin/look-requests`), landed by `temper/docs/design/draft/look-requests.md`, 3 and 5. `notes/common.md` holds the rules every task follows.

## Parts and tasks

| Part | Task | Request | Needs | What it makes run | Minutes |
|---|---|---|---|---|---|
| `stroke` | `tasks/path-stroke.md` | 1 | | one rule of `temper.css` under the path: skein's default stroke gives way to `stroke: none`, every mark's own stroke kept | 20 |
| `scale` | `tasks/scene-width.md` | 3 | | skein: a scene says its width, drawn at it and scaled to its box (`data-scene-width`, a press's point scaled) | 50 |
| `scale` | `tasks/path-scale.md` | 3, 4 | `scene-width` | across under 720 the path drawn at 720 and scaled; an open stop's room 14 box pixels, across with its caret and down the page without | 60 |
| `gallery` | `tasks/gallery-narrow.md` | 2 | | the gallery prints each path a second time at 388, shown at 760 and under | 45 |

Tasks 5 to 8 (a card's first sentence, its link, the CI card's words, the terminal's repeated headings) follow in a later batch, once their surveys answer.

**Critical path so far:** `scene-width` → `path-scale`, about 110 implementer-minutes. `path-stroke` and `gallery-narrow` run beside `scene-width`.

## What was decided

- **The outline's cause (task 1)** is skein's `[data-drawn] > svg :is(g, line, polyline, path, rect, circle, text) { stroke: currentColor; }`, (0,1,2). temper's fix is one rule, `.temper-ways-path[data-drawn] > svg :is(...) { stroke: none; }`, (0,2,2): it outweighs skein's whatever the order and is outweighed by each of the path's own stroke rules, all `#app > .temper-session ...`, (1,2,0) or more. A rule under `#app` would have outweighed the path's own strokes and erased the line; skein's rule stays. No test: a rendered fact, measured by the render check after landing.
- **The scaling (task 3) needs skein:** a scene is always its box's width (skein's `ui.md`, 11.2), so a picture laid out at 720 can't be scaled to a 658 box. skein gains a scene that says its width, beside its height (`ui.md`, 11.2, *Its own width*; `ui-web.md`, 8 and 12): its SVG at its own size, `data-scene-width` on the figure, `skein-ui.css` scaling it to the figure's width, and a press's point scaled back to the scene's pixels. A generic mechanism goes into skein before its consumer (`AGENTS.md`, 1). temper's path then draws across under 720 at 720 and says so; at 720 and over and down the page it says no width. Across is still chosen on the box's width, so no path changes layout.
- **The open card's place (tasks 3 and 4) is the scene's room:** the card follows the drawing with no gap of its own (`README.md`, 3.5), and the scene grows at its foot by 14 box pixels while a stop is open: across `round(14 × 720 / W)` scene pixels (15 at 658, so the card stands at 141.65 against the request's 141.94), with the caret in it scaled likewise; down the page 14, with no caret. Down the page grew nothing before, which put the phone's card 14 high. The request's `.temper-ways-zoom { margin: 14px 0 0 }` stays out: a viewport rule can't know which layout the box chose, since the shell switches by the box (560) and the request by the viewport (760).
- **The gallery at 420 (task 2):** a static page measures no box, so the gallery prints each path twice in its figure, across at 720 and down the page at 388 (the box at 420, 16 in each side), with a `<style>` of its own in the page's head showing the second at 760 and under. Nothing of the view, the shell or `temper.css` changes for it (`browser.md`, 7.1, *A path printed for both widths*).
- **Design written here:** skein's `ui.md`, 11.2, *Its own width*; `ui-web.md`, 8, *A scene that says its width* and the point's caveat, and 12's list; temper's `client/README.md`, 5.16, *Its layout* (across at `A`) and *Pressed* (the open stop's room); `client/browser.md`, 2.5, *Ask temper's ways* (the stroke) and 7.1, *A path printed for both widths*. A **Not yet** entry per gap in `skein/docs/design/ui.status.md`, `ui-web.status.md`, `temper/docs/design/client/README.status.md` and `browser.status.md`, each deleted by its task; `browser.status.md`'s two entries stand on either side of the 7.3 entry, so that their deletions never meet in a merge. No draft status changes.

No stored format, protocol or public interface changes in this batch.

## Overlaps with drafts beside this one

- **`temper/docs/design/client/README.status.md`** is also several `tui-box-front` tasks': this cut adds one entry at its top, which `path-scale` deletes.
- **`skein`:** no draft beside this one changes `skein-ui`, `skein-ui-web` or `skein-ui-bundle`.
- **`temper.css`, `temper-client-view`'s paths and `tools/gallery`:** no draft beside this one names them.
