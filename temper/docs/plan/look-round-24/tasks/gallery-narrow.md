+++
id = "gallery-narrow"
part = "gallery"
kind = "feature"
needs = []
files = [
  "temper/tools/gallery/src/lib.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 45
+++

# The gallery at 420 runs the path down the page

## Goal

The request's task 2 (`temper/docs/plan/look-round-24/inputs/request.md`). The gallery's page is static and measures no box, so it draws each path once, across in a box 720 wide (`temper/tools/gallery/src/lib.rs`, about line 176, `page.resized(node.id, 720, 0, &mut ops)`), and a page 420 wide squeezes that scene to 75 pixels high with 8 pixel labels. The shell, measuring the box, draws the path down the page there. The gallery now prints each path twice in its figure, across at 720 and down the page at 388, and the page's own style shows one per width.

Acceptance, by `temper/docs/design/client/browser.md`, 7.1, *A path printed for both widths* (written by this cut):

1. **The second drawing:** in `page_with`, for each node whose drawing is `temper_client_view::paths::KIND`, after the page's HTML is taken (`let mut html = page.html()...`, about line 197), the gallery draws the same data again in a box 388 wide: a `skein_ui::scene::Scene::new(&bounds.tree)`, the gallery's `draw(kind, data, 388, 0, &mut scene)`, then `skein_ui_web::svg::draw(&scene, &[temper_client_view::look::NAMES], 388, 0)`, given the attribute `data-gallery-narrow` (empty) with `Element::set`, and printed with `skein_ui_web::dom::print`. A drawing refused or not drawn adds nothing. 388 is the box at 420 pixels with a phone's 16 pixels each side (`temper/docs/design/client/README.md`, 3.5, *The conversation and the composer's foot*); name it once, `const NARROW: u16 = 388;`, with that reason.
2. **Where it goes:** directly after the figure's own `</svg>`: the first `</svg>` after the figure's `data-node="ID"` attribute, `ID` the drawing node's number, as the page's other replacements find a node (`open_disclosure`, about line 199). Both SVGs are the figure's children, so every rule of `temper.css` under `.temper-ways-path > svg` reaches both.
3. **The page's own style:** a page holding a narrow drawing gets, in its head after the stylesheets' links, one `<style>`:

   ```css
   @media (max-width: 760px) { #app .temper-ways-path[data-drawn] > svg:not([data-gallery-narrow]) { display: none; } }
   @media (min-width: 761px) { #app .temper-ways-path[data-drawn] > svg[data-gallery-narrow] { display: none; } }
   ```

   Its specificity, (1,3,1), outweighs `temper.css`'s `#app > .temper-session .temper-ways-path > svg { display: block; ... }`, (1,2,1). A page with no path gets no `<style>`. 760 is the shell's phone width (`browser.md`, 2.5). Nothing of the view, the shell or `temper.css` changes.
4. **The print's row** (`print_writes_every_page_and_the_bundle`, `temper/tools/gallery/src/tests.rs`, about line 3738, no new test function): in its loop over the samples, a page holds `data-gallery-narrow` exactly as often as its view's tree holds a drawing of `paths::KIND`, each such SVG's `viewBox` beginning `0 0 388 `, and holds `@media (max-width: 760px)` once if it holds any narrow drawing and not at all otherwise. Find the drawings in the view's tree as `no_named_control_draws_a_text_child_in_any_state` does (about line 3420), through `crate::view(sample)`.
5. **Status:** delete the **Not yet** entry naming `gallery-narrow` in `temper/docs/design/client/browser.status.md` (under 7.1, below the 7.3 entry). No other design line.
6. Runs through `heavy`, counts in the report: `cargo nextest run -p temper-client-gallery`; clippy on `temper-client-gallery`. The gallery's tests are over their budget already (`browser.status.md`, 7.3): add no state and no test; time `cargo nextest run -p temper-client-gallery -E 'test(=print_writes_every_page_and_the_bundle)'` before and after the change and give both in the report.

One commit or more, each with the trailer `Look-request: ask-temper-ways`; the last carries the account (below).

## Inputs

- `temper/docs/plan/look-round-24/notes/common.md`.
- `temper/docs/plan/look-round-24/inputs/request.md`, task 2.
- `temper/docs/design/client/browser.md`, 7.1, *Static* and *A path printed for both widths*; 2.5, *Ask temper's ways*.
- `temper/docs/design/client/README.md`, 5.16, *Its layout* (down the page where the box is under 560).
- skein's `skein/docs/design/ui-web.md`, 8 (the SVG of a scene), read only.

## Owns and may change

Owns `page_with`'s printing, the print test's row and the status entry. Not the view, `paths`, the shell, `temper.css`, `names.txt` or any sample.

Options, each decided:

- **Another gallery test reads a page's HTML and counts `<svg`** or a drawing's children (`rg -n '<svg|data-drawn' temper/tools/gallery/src/tests.rs`): let it count the figure's first SVG alone, keeping what else it checks, and name it.
- **`path-scale` (this plan's) lands first or after:** neither changes this task's code. At 720 the across scene says no width; down the page an open stop's room grows by 14 once `path-scale` lands, which the print takes as it comes.

## Steps

1. Read the Inputs, `page_with` and the print test; time the print test (acceptance 6).
2. The second drawing, its place and the page's style (acceptance 1 to 3); the row (4); bite it: insert the narrow SVG without `data-gallery-narrow`, see the row fail, restore.
3. The status entry (5), the runs and the second timing (6); `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main` after committing.
4. Commit with the trailer and the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| The gallery at 420: the path down the page, as live | landed (`tools/gallery`, `page_with`: a second drawing at 388, shown at 760 and under) |
| The gallery over 760: the path across at 720, as today | kept |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4): the print's text establishes no rendered equality |

## Report

The implementer's report: where the second drawing goes and how it is found, the style as written, the row and its bite, both timings, the runs' counts, and the account as committed.
