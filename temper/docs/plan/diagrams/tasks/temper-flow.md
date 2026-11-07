+++
id = "temper-flow"
part = "temper-flow"
kind = "feature"
needs = ["flow-read", "flow-layout"]
files = [
  "temper/crates/temper-client-shell-web/src/host.rs",
  "temper/crates/temper-client-shell-web/src/host_tests.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/limits.rs",
]
effort = "high"
minutes = 40
+++

# temper draws a flow

## Goal

temper's page routes skein-ui's owner to its drawing code and gives its names, and its limits leave a flow room. So a flow in any Markdown a person reads in temper, a turn's or a document's, is drawn, and a page with none costs nothing.

Acceptance:

- A test on temper's own `host::page`, with `Limits::of`'s bounds: `how-hello-works.md` read under a page and a main, then `resized` to 640, makes `page.html()` hold `how-hello-works.640.svg`'s bytes but its final newline. Resized to 240, it holds no `<svg`, and the fallback's words stand. This is the meeting point of temper and skein.
- `temper_client_view::Limits::of` gives `drawing_bytes: 2048`, `scene_marks: skein_ui::flow::marks(24)?` and `scene_bytes: skein_ui::flow::scene_bytes(24, 2048)?`, under a comment saying they are a flow's (`skein/docs/design/ui.md`, 10.1), and `valid` still holds.
- `a_window_full_of_markdown_builds` has a row of flows, `` ```mermaid\nflowchart LR\na-->b\n```\n `` repeated: a window full of them builds under temper's limits.
- `public.rs`'s limits stay as they are, with no `drawing_bytes`: a public page has no document.
- The gallery stays as it is: it measures no box, so its prints don't draw (`temper/docs/design/client/journeys.md`, 7).

## Inputs

- `temper/docs/design/client/README.md`, 3.5, the sentence *A fenced block reads as code, but for a flow*.
- `skein/docs/design/ui.md`, 3.3, *skein-ui's own*, and 11.5.
- `temper/docs/plan/diagrams/notes/contracts.md`, *An application that draws flows*.
- `temper/docs/plan/diagrams/notes/common.md`.

## Owns and may change

Owns the files in the header, each only for what's below.

Readers, surveyed by `rg`:

- `Look`'s other literals (`temper/tools/gallery/src/lib.rs` near line 154, `host_tests.rs` near line 511) keep `draw` false, since neither measures a box. Leave them.
- `Limits::of`'s `tree` is read by `valid` and by tests. `temper-client-view/src/tests.rs` near line 165 sets its own scene fields, and `tests/fixtures.rs` writes `Limits` whole with `drawing_bytes: 0`, so its Markdown stays code. Leave them: their fences hold no `mermaid`.
- `temper-client-shell-terminal` builds under these limits and prints a flow's fallback (`skein/docs/design/ui-text.md`, section 5). Its goldens hold no `mermaid`, so none moves.
- `temper/tools/journeys` uses `host::page` for its replays. No recording holds a flow, so no recording goes stale. `the_committed_journeys_are_their_print` is your check of it.
- `temper.css` isn't yours: `temper-flow-look` holds it, and this task doesn't need it merged.

Options, each pre-approved:

- If `skein_ui::look::check` is asserted on the page's names and prefers another order, follow it and say so.
- If a pinned figure follows `Limits::of`'s fields, set it to what these need and say so in the report.

## Steps

1. `host.rs`, `page`: `names: &[view::look::NAMES, skein_ui::flow::NAMES]`, `draw: skein_ui::flow::draw`. `flow::draw` returns false for any other kind: temper has no drawing of its own.
2. `limits.rs`: the three fields, with their comment.
3. `tests/limits.rs`: the row.
4. `host_tests.rs`, one test, `a_flow_draws_on_temper_page_as_its_case`:

   ```rust
   let bounds = fixture().bounds.tree;
   let mut view = skein_ui::View::new(&bounds);
   let mut builder = view.builder();
   builder.open(Role::Page);
   builder.open(Role::Main);
   skein_ui::markdown::read(CASE, 1, &mut builder);
   builder.close();
   builder.close();
   // view.finish into a Queue, patches into a Vec, as host.patches does
   let mut page = page(&bounds, Primary::Control);
   // apply, find the Role::Drawing node's id, page.resized(id, 640, 0, &mut ops)
   ```

   with `CASE` from `include_bytes!("../../../../skein/docs/design/draft/diagrams/how-hello-works.md")` and its `.640.svg` by `include_str!`.
5. Narrow tests: `heavy cargo nextest run -p temper-client-shell-web --lib -E 'test(a_flow_draws)'`, `heavy cargo nextest run -p temper-client-view --lib -E 'test(a_window_full_of_markdown) | test(validity)'`, `heavy cargo nextest run -p temper-client-journeys --lib -E 'test(=the_committed_journeys_are_their_print)'`. Then the closing checks of your text. Commit and push.

## Report

The implementer's report, with the rule you broke by hand for the test, and any pinned figure you moved.
