+++
id = "path-draw"
part = "drawing"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-view/src/paths.rs", "temper/crates/temper-client-view/src/paths/**", "temper/crates/temper-client-view/src/tests/paths.rs", "temper/crates/temper-client-view/src/lib.rs", "temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/limits.rs", "temper/crates/temper-client-shell-web/src/host.rs", "temper/crates/temper-client-shell-web/temper.css"]
effort = "high"
minutes = 130
+++

## Goal

Draw a path as temper's own drawing kind, beside skein's flows: its stops
on a line, a person's and an agent's above it, code's below, each branch's
bar, the dotted line no merge tool crosses, with a text fallback a stop a
line. This task makes the drawing and its registration in the browser
from a local, wire-free description of a path; task `views-draw` feeds it
from the `View` read, and the look request's plan feeds Ask temper's page.

Acceptance:

- `temper-client-view` has a module `paths` with:

  ```rust
  /// Who acts at a stop, for the drawing alone.
  pub enum Mark { Person, Agent, Code, Bar }
  pub struct Stop<'a> { pub mark: Mark, pub words: &'a [u8] }
  /// temper's drawing kind for a path: temper's owner (`frame::owner()`), kind 0.
  pub const KIND: skein_ui::DrawingKind;
  /// Add a path to the tree: the drawing, with its fallback children.
  pub fn path(builder: &mut skein_ui::Builder<'_>, label: &[u8], stops: &[Stop<'_>]) -> Result<(), skein_ui::Problem>;
  /// Draw a path's data into a scene (skein_ui::Draw's shape).
  pub fn draw(kind: skein_ui::DrawingKind, data: &[u8], width: u16, height: u16, scene: &mut skein_ui::Scene) -> bool;
  /// temper's draw: its own kind, else skein's flows.
  pub fn draw_any(kind: skein_ui::DrawingKind, data: &[u8], width: u16, height: u16, scene: &mut skein_ui::Scene) -> bool;
  ```

  Use the builder and problem types the crate's other drawing users use;
  where `skein_ui`'s names differ from these, take skein's and keep the
  shape.
- The drawing, at a width of 640: one row of stops at even steps from the
  left edge, a person's and an agent's marks above a centre line and
  code's below it, a bar's on the base line; the line, a path rising to
  each judgement and falling back between stops; a dotted horizontal line
  between the two rows (no merge tool crosses it); a person's mark a
  ring, an agent's a four-point star, code's a small square, a bar a
  short vertical bar, the last bar the landing branch at the right end;
  each stop's words over an upper mark and under a lower one; the look
  request's `01.html` path (`git show
  c342a9f113:temper/docs/design/client/journeys/ask-temper-ways/01.html`,
  its first `temper-ways-wide` SVG) as the model of the geometry, not a
  golden. Too narrow for its stops at a 48-pixel step, it refuses
  (`false`), and the fallback stays.
- The fallback: a list named by `label`, a stop an item, "WORDS, an
  agent", "WORDS, a person", "WORDS, code", "WORDS, a branch", which the
  terminal prints and a page shows when the drawing refuses.
- The browser's look: `temper-client-shell-web`'s `page` draws with
  `paths::draw_any` and registers the drawing's style names beside
  `view::look::NAMES` and `skein_ui::flow::NAMES`; its rules go in
  `temper.css` under `/* ==== ui: chats ==== */`, in fabrig.ai's language
  as the look request's `overlay.css` has it (`git show
  c342a9f113:temper/docs/design/client/journeys/overlay.css`, the
  `temper-ways-*` rules): judgement marks in the signal colour, code's and
  the line in the line colour, the dotted line faint, words small.
- Limits: `temper-client-view`'s drawing and scene limits hold a path of
  32 stops, each of 64 bytes of words, beside a flow's (`limits.rs`).
- Tests (`src/tests/paths.rs`): a table of the three look request paths
  (contracts, section 2's words; `01.html` to `03.html`'s figure labels
  give each list) drawn at 640: each stop's mark kind and row, the bars,
  the dotted line, words placed; at 240 the drawing refuses; the fallback
  in both shells; `draw_any` passing a flow's kind to skein's.

## Inputs

- `temper/docs/plan/ui-ask-m3a/notes/common.md`.
- `temper/docs/design/client/README.md`, 5.16 (*Drawn*); the look
  request `ask-temper-ways` (`notes/common.md`, *The look request*).
- skein's `docs/design/ui.md`, sections 3.3, 10.1 and 11 (drawings, flows,
  scenes); `skein/crates/skein-ui/src/flow.rs`, `flow/layout.rs` and
  `scene.rs` as the model of a drawing kind and its `draw`;
  `temper/crates/temper-client-shell-web/src/host.rs`, `page`.

## Owns and may change

- **Owns:** `temper-client-view/src/paths.rs` (and `paths/**`),
  `src/tests/paths.rs`; the style constants and names it adds to
  `look.rs` (new tokens after the last, never renumbering); its rows in
  `limits.rs`; `temper-client-shell-web/src/host.rs`'s `page`;
  `temper.css` under the chats marker.
- **Wiring:** `lib.rs`'s `pub mod paths;`, and the tests' `mod` line.
- **Options, pre-approved:** if `Look` takes one `draw` function only, as
  it does today, `draw_any` is that function. If the gallery's page
  (`temper/tools/gallery/src/lib.rs`) must name the new style names to
  keep its checks green, add them there and say so. If a host test of the
  shell compares the page's names list, update it.

## Steps

1. Read skein's flow drawing end to end; write `paths` after it: the data
   encoding (marks and words, bounded), `path`, `draw`, `draw_any`.
2. Styles, limits, the shell's `page`, the CSS rules.
3. Tests as above; break the rule once (draw a code mark above the line)
   to see the table fail, and restore.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-view -p temper-client-shell-web --all-targets
   -- -D warnings`; `heavy cargo nextest run -p temper-client-view` and
   `-p temper-client-shell-web` (`TEMPER_BUNDLE=none`). Commit and push.

## Report

As the implementer's text asks; name the geometry's constants and the
style names you added.
