+++
id = "fade"
part = "fade"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui/src/scene.rs",
  "skein/crates/skein-ui/src/tests/scene.rs",
  "skein/crates/skein-ui/src/tests/golden/scene.txt",
  "skein/crates/skein-ui-web/src/svg.rs",
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/tests/svg.rs",
  "skein/crates/skein-ui-web/src/tests/page.rs",
  "skein/crates/skein-ui-web/src/tests/fixtures.rs",
  "skein/crates/skein-ui-web/src/tests/golden/scene.svg",
  "skein/crates/skein-ui-bundle/skein-ui.css",
  "skein/docs/design/ui.status.md",
  "skein/docs/design/ui-web.status.md",
  "temper/docs/design/client/journeys/skein-ui.css",
]
effort = "high"
minutes = 80
+++

# A scene's mark may have its stroke painted by a linear fade

## Goal

skein's half of the request's item 3, third bullet, "the line painted as the request's continuous gradient" (`temper/docs/plan/look-round-25/inputs/request.md`). The look request `ask-temper-ways` paints the path's line with an SVG `linearGradient`; temper draws two flat tones, because a scene carries no gradient. A scene's line, polyline or path may now carry a fade: its stroke painted along the way from one point to another by stops, each stop's colour the stylesheet's by its tokens, so that a scene still says no colour. temper's path uses it in `path-marks`, which needs this task. This task writes nothing of temper's but one call site and a regenerated copy.

Acceptance, by `skein/docs/design/ui.md`, 11.2, *A fade*, *No colour* and *Bounded*, 15, *A fade's share*, and `skein/docs/design/ui-web.md`, 8, *A fade is a gradient of its SVG's own*, *A fade's id is unique in the document*, *A fade's paint and the stylesheets*, and 12, all written by this cut, and `temper/docs/plan/look-round-25/notes/contracts.md`, *skein's fade*:

1. **`skein_ui::scene`** (`skein/crates/skein-ui/src/scene.rs`), step code under the subset, gains:

   ```rust
   /// A fade's stop: its place in thousandths of the way, 0 to 1000, and its tokens (ui.md, 11.2).
   #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
   pub struct FadeStop<'a> {
       pub offset: u16,
       pub styles: &'a [Style],
   }

   /// A mark's fade as its scene holds it (ui.md, 11.2).
   #[derive(Clone, Copy, Debug)]
   pub struct Fade<'a> {
       pub from: Point,
       pub to: Point,
       // private: the stops' places and token counts, and their tokens
   }
   impl<'a> Fade<'a> {
       /// Its stops in order, as they were given.
       pub fn stops(&self) -> impl Iterator<Item = FadeStop<'a>> + 'a;
   }

   /// The scene's bytes a fade of `stops` stops takes: 8 and 4 for each stop (ui.md, 15).
   #[must_use]
   pub const fn fade_bytes(stops: u32) -> Option<u32>;

   impl Scene {
       /// Paint the last mark's stroke, a line's, a polyline's or a path's, with a linear fade (ui.md, 11.2).
       pub fn fade(&mut self, from: Point, to: Point, stops: &[FadeStop<'_>]) -> bool;
       /// The mark's fade, if it has one.
       #[must_use]
       pub fn mark_fade(&self, index: u32) -> Option<Fade<'_>>;
   }
   ```

   - **Stored** beside `hint` and `act` in `Properties` (`fade: Option<..>`): the two points and each stop's place and token count in the scene's bytes, `fade_bytes(stops)` of them, 4 bytes a point as `write_point` writes it and 2 and 2 a stop, little-endian; the stops' tokens in the scene's token store, right after the mark's own, which nothing else can have followed since a fade is on the last mark. `clear` and a refusal forget it with the rest.
   - **Refused,** the scene with it (`refuse()`, returning false), in this order: a scene already refused returns false; no mark; a last mark that isn't `Line`, `Polyline` or `Path`; a mark that has a fade already; no stop; a place past 1000, or below the one before it (equal places are a hard edge, accepted); the mark's own tokens and all its stops' together past `styles_per_mark`; a stop's token count past `u16` (only a `styles_per_node` past 65535 reaches it); `fade_bytes` past the bytes' room. Every check comes before any write, so the token store's `expect("reserved token capacity")` stays true: a mark's tokens and its fade's never pass `styles_per_mark`, which sized the store.
   - **`Display`** prints a fade on its mark's line, after the act: ` fade X,Y X,Y`, then each stop as ` P[o:t o:t]`, its place and its tokens as a mark's tokens print, `[]` for a stop with none. A mark with no fade prints as today.
2. **`skein_ui_web::svg::draw`** (`skein/crates/skein-ui-web/src/svg.rs`) takes the stem:

   ```rust
   /// A refused scene is an empty fragment, so the page keeps its fallback.
   /// `ids` is a stem unique in the document; a fade's gradient is `<ids>-<mark index>` (ui-web.md, 8).
   #[must_use]
   pub fn draw(scene: &Scene, names: &[Names<'_>], width: u16, height: u16, ids: &str) -> Element;
   ```

   Where any mark has a fade, the SVG's first child is a `<defs>` holding, in the marks' order, one `<linearGradient id="<ids>-<index>" gradientUnits="userSpaceOnUse" x1 y1 x2 y2>` per faded mark, its points' whole numbers; inside it one `<stop offset="0.500">` per stop, its place as `format!("{}.{:03}", p / 1000, p % 1000)`, with `class` its tokens' names joined by spaces as a mark's are, and no `class` where none resolves. The faded mark's element sets, after its `class` and `data-act`, `stroke="url(#<ids>-<index>)"` and `data-fade=""`. No `style` attribute anywhere (`skein/docs/design/ui-web.md`, 4.4). A scene with no fade prints byte for byte as today: no `<defs>`, no `stroke`, no `data-fade`.
3. **The page part** (`page.rs`, `draw`): passes `&format!("skein-ui-fade-{}", node.0)`. Nothing else of it changes: a press finds marks by `page/web.rs`'s selector of the marks' tags, which `defs`, `linearGradient` and `stop` are not, and a redraw makes the SVG whole.
4. **`skein-ui.css`:** the rule `[data-drawn] > svg :is(g, line, polyline, path, rect, circle, text) { stroke: currentColor; fill: none; }` becomes two, keeping its weight, so that a flow's `stroke: none` rules (`[data-drawn] > svg .skein-ui-flow-name`, one attribute, one class, one type) still outweigh it:

   ```css
   [data-drawn] > svg :is(g, line, polyline, path, rect, circle, text) {
     fill: none;
   }

   /* A faded mark's stroke is its fade's (ui-web, 8); :where keeps the default's weight. */
   [data-drawn] > svg :is(g, line, polyline, path, rect, circle, text):where(:not([data-fade])) {
     stroke: currentColor;
   }
   ```

   No other rule changes.
5. **Rows, and two new test functions, said why:**
   - `bytes_past_scene_bytes_refuse_the_scene` (`skein-ui/src/tests/scene.rs`, `scene_bytes: 4`) gains kind 6, its range `0_u8..7`: a line, then `scene.fade(ZERO, ZERO, &[FadeStop { offset: 0, styles: &[] }])`, 12 bytes, refused.
   - `too_many_tokens_on_a_mark_refuse_the_scene` (`styles_per_node: 1`) gains a second scene: a line with one token accepted, then a fade whose one stop has one token refused, the scene refused and empty.
   - `a_scene_prints_one_line_per_mark` and its hand-written `golden/scene.txt`: the path (mark 3, in the group) gets `fade(ZERO, b, ..)` with stops at 0 (`7:9`), 500 (none) and 1000 (`8:10`), within the fixture's `styles_per_node: 2`; its golden line becomes `  path M 0,0 L 1,2 Q 1,2 3,4 C 0,0 1,2 3,4 Z fade 0,0 3,4 0[7:9] 500[] 1000[8:10]`, edited by hand. No other line of the golden changes.
   - **New, `a_fade_is_taken_or_refused_as_ui_md_says`** (`skein-ui/src/tests/scene.rs`): no table holds a fade's own rules. Rows by name, the mark made by a `match` on a small enum of the test's own (no closure): accepted on a line, a polyline and a path, with places 0, 500, 500, 1000, and with one stop at 1000, each read back by `mark_fade` (its points and its stops, places and tokens, as given); refused with no mark, on a group, a rectangle, a circle and a text, with no stop, a place of 1001, places 500 then 0, and a second fade on one mark, each leaving `scene.refused()` and no mark. One assertion beside: `fade_bytes(3) == Some(20)`.
   - `a_scene_draws_as_its_golden_svg` (`skein-ui-web/src/tests/svg.rs`) and `golden/scene.svg`: after the line's act, `assert!(scene.fade(p, q, &[FadeStop { offset: 0, styles: &[Style { owner: OWNER, token: 0 }] }, FadeStop { offset: 1000, styles: &[] }]))`, drawn with `ids` `"test-fade"`; the golden gains, first in the SVG, `<defs><linearGradient id="test-fade-1" gradientUnits="userSpaceOnUse" x1="1" y1="2" x2="3" y2="4"><stop offset="0.000" class="test-mark"></stop><stop offset="1.000"></stop></linearGradient></defs>`, and the line `stroke="url(#test-fade-1)" data-fade=""` after its `data-act`. Nothing else of the golden changes.
   - `a_scene_uses_its_own_size_or_its_boxs`, `a_refused_scene_draws_nothing`, `a_marks_text_is_escaped_never_markup` and `a_flows_scene_prints_its_case_byte_for_byte` take a stem at their call and keep their expected bytes: they hold that a scene with no fade prints as today.
   - **New, `two_drawings_paint_their_fades_by_ids_of_their_own`** (`skein-ui-web/src/tests/page.rs`): no test builds two drawings on one page. The fixture's drawing code (`tests/fixtures.rs`, `drawing`) gains a `b"fade"` branch beside `b"refuse"`: a line from `(0, 0)` to `(width, height)` and a fade from `(0, 0)` to `(0, height)`, stops at 0 and 1000 with no tokens. A page of a main and two drawings of `b"fade"`, each resized to 100 by 50: its HTML holds two gradients, `id="skein-ui-fade-A-0"` and `id="skein-ui-fade-B-0"`, A and B the two drawings' node ids (read from the view's tree, not written by hand), each figure's line naming its own (`url(#skein-ui-fade-A-0)` inside A's figure). The goldens `drawing.html`, `drawing-changed.html` and `fallback.html` stay as they are.
6. **The gallery's second print** (`temper/tools/gallery/src/lib.rs`, `page_with`, the one `skein_ui_web::svg::draw` call, about line 215) passes a stem of its own, `&format!("temper-gallery-narrow-{}", node.id.0)`, so that its narrow print of a drawing never shares an id with the page part's print of it (`skein/docs/design/ui-web.md`, 8). A call-site edit alone.
7. **Status:** delete the **Not yet** entries naming `fade` and `look-round-25` in `skein/docs/design/ui.status.md` (11.2, 15 and 17) and `skein/docs/design/ui-web.status.md` (8, 12 and 13). No other design line.
8. **The journeys' copy:** `temper/docs/design/client/journeys/skein-ui.css` is `skein_ui_bundle::STYLESHEET`, written beside the journeys at each print (`temper/tools/journeys/src/print.rs`, `beside`) and held by `the_committed_journeys_are_their_print`, so the new rules make it stale: regenerate it with `heavy cargo run -p temper-client-journeys` (no arguments) and commit the copy alone. No page of the reprint may change: a scene with no fade prints as today, and no drawing has a fade yet, so a page that changes is a red, not stale.
9. **Runs, through `heavy`, one at a time, counts in the report** (a change of meaning runs its readers' tests: a scene's print, an SVG's bytes, the stylesheet):
   - `cargo nextest run -p skein-ui`, `-p skein-ui-web`, `-p skein-ui-bundle` (its tests read `skein-ui.css`), `-p skein-ui-tests` (scenes at their limits), `-p skein-ui-example-view` (prints its chart's scene);
   - `cargo nextest run -p temper-client-view` (its paths' scenes, their exact `SCENE_BYTES` and their prints), `-p temper-client-gallery` (the call site; its prints of each path twice), `-p temper-client-journeys` (after the reprint), `-p temper-client-shell-web` (its host test prints a flow's SVG byte for byte through the page part), each with `TEMPER_BUNDLE=none` (`AGENTS.md`, 5);
   - compiled alone, slow tier: `cargo check -p skein-ui-web-tests --tests` and `-p skein-browser-world --tests` (it compiles `skein-ui.css` in);
   - clippy: `cargo clippy -p skein-ui -p skein-ui-web -p skein-ui-bundle -p temper-client-gallery --all-targets -- -D warnings`.

## Inputs

- `temper/docs/plan/look-round-25/notes/common.md` and `notes/contracts.md`, *skein's fade*.
- `temper/docs/plan/look-round-25/inputs/request.md`, item 3, third bullet.
- `skein/docs/design/ui.md`, 11.2 (*Every mark*, *A fade*, *No colour*, *Bounded*), 11.4 (the browser's row), 15 (the table's `styles_per_node` and `scene_bytes` rows, *A fade's share*), 17 (*Step tests*), all as this cut wrote them.
- `skein/docs/design/ui-web.md`, 8 (*A fade is a gradient of its SVG's own*, *A fade's id is unique in the document*, *A fade's paint and the stylesheets*, *Refused, never cut*, *Acted on*), 12 (`skein-ui.css`'s list), 13, and 4.4 (*The strict policy*: no `style` attribute), as this cut wrote them.
- The request's gradient, for reading only: `git show 98322b3565:temper/docs/design/client/journeys/ask-temper-ways/01.html`, its `<linearGradient>`s.
- `skein/AGENTS.md` (skein's clippy rules); `skein/docs/foundation/programming-model.md`, 10 (the subset `scene.rs` is under).

## Owns and may change

Owns the files above; `temper/docs/design/client/journeys/skein-ui.css` by its tool alone (acceptance 8).

Readers, surveyed with `rg` (`svg::draw`, `scene::`, `Scene`, `skein-ui.css`, `STYLESHEET`), and the edits allowed there:

- `temper/tools/gallery/src/lib.rs`: the one `svg::draw` call's stem (acceptance 6), nothing else.
- `skein-ui-web/src/tests/svg.rs`, `page.rs`: their own (owned).
- No edit: every other `Scene` reader (`temper-client-view`'s `paths`, `skein-ui`'s `flow`, `skein/examples/ui/view`'s `draw.rs`, `skein/tests/ui`'s `memory.rs` and `limits.rs`) builds marks and prints scenes, neither of which changes for a scene with no fade; every other dependent of `skein-ui-web` (jig's and temper's web shells, the playgrounds, the browser worlds) reaches scenes through the page part, whose API stays; `skein-ui-bundle/src/tests.rs`'s `the_stylesheet_names_only_the_kits_flow_classes` reads no attribute selector and holds; `temper/tools/journeys/src/tests.rs` reads the copy (acceptance 8). jig's `check::Scene` is a type of its own.
- No design text but the two status entries (acceptance 7); `notes/common.md` departs from the implementer's text there.

Options, each decided:

- **`Fade::stops`** returns `impl Iterator`; if the subset's lints or a borrow make that awkward, a named iterator `FadeStops<'a>` beside `Fade` in `scene.rs`, yielding `FadeStop<'a>`, is fine: say so.
- **Attribute order in the SVG golden** follows `dom::Element::set`'s order of calls as acceptance 2 gives it; if `dom::print` orders otherwise, the golden follows the code, and the report says so. Only the golden scene and the new page test may show a fade: any other golden that changes is a red, not stale.
- **`the_stylesheet_names_only_the_kits_flow_classes`** (`skein-ui-bundle/src/tests.rs`, about line 150): the new rules name no class, so it passes; if it trips on them, extend that assertion for these rules alone and say so.
- **`a_fade_is_taken_or_refused_as_ui_md_says`'s rows** may share `ZERO` and the fixture `LIMITS` (`styles_per_node: 2`, `scene_bytes: 256`); a row that needs a tighter limit builds its own `Limits { .., ..LIMITS }`, as the other tests do.
- **temper's path** draws no fade in this task: its marks, `SCENE_BYTES` and its stylesheet's `temper-ways-*` rules are `path-marks`'s. If `temper-client-view`'s tests fail, the failure is this task's (a scene with no fade must print as today): repair it here, never there.

## Steps

1. Read the Inputs, `scene.rs` (`Properties`, `add`, `store`, `hint`, `act`, `Display`), `svg.rs`, `page.rs`'s `draw`, `page/web.rs`'s `drawing_event` (the marks' selector), `skein-ui.css`'s scene rules, and the tests named above.
2. The scene's fade, its reader, `fade_bytes` and its print (acceptance 1), with the rows of `skein-ui` (5's first four); `heavy cargo nextest run -p skein-ui --lib -E 'test(/scene::/)'`. Bite: drop the falling-place check, see the row "places 500 then 0" fail; drop the mark's own tokens from the tokens' sum, see `too_many_tokens_on_a_mark_refuse_the_scene` fail; restore each.
3. The SVG, the page part's stem and the stylesheet (acceptance 2 to 4), with the rows of `skein-ui-web` (5's last three); `heavy cargo nextest run -p skein-ui-web --lib -E 'test(/svg::/) | test(/page::/)'`. Bite: make the page part's stem a constant without the node id, see `two_drawings_paint_their_fades_by_ids_of_their_own` fail; restore.
4. The gallery's call site (acceptance 6); `heavy cargo check -p temper-client-gallery --tests`.
5. The status entries (7); the journeys' copy (8), then `git status --short temper/docs/design/client/journeys` shows that one file alone.
6. The runs (9); `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py` and `python3 -I -B temper/scripts/old_forms.py`; commit; `python3 -I -B tools/citations/citations.py --base origin/main`.
7. Commit with the trailer `Look-request: ask-temper-ways` (a miss of that request, `temper/docs/design/draft/look-requests.md`, 3 and 7) and, in the last commit, the account; push with `git push origin HEAD:refs/heads/<your branch>`.

The account's rows, at least:

| Item | Result |
|---|---|
| A mark's fade in a scene (`Scene::fade`, `FadeStop`, `Fade`, `mark_fade`, `fade_bytes`), refused past the limits and its rules | landed (`skein-ui`, `scene.rs`) |
| A fade as its SVG's own gradient, its id unique in the document, its stops' colours by classes | landed (`skein-ui-web`, `svg.rs`, `page.rs`) |
| A faded mark's stroke kept from `skein-ui.css`'s default | landed (`skein-ui.css`; the journeys' copy regenerated) |
| The gallery's narrow print under ids of its own | landed (`temper/tools/gallery/src/lib.rs`) |
| temper's path line painted as the request's continuous gradient | deferred to `path-marks`, this plan's, which needs this task |
| Render check | follows landing, temper/ui's (`temper/docs/design/draft/look-requests.md`, 4) |

## Report

The implementer's report: the API as written, the stored form chosen, the rows and the bites with their failures, the runs' counts, the reprint's changed files, and the account as committed.
