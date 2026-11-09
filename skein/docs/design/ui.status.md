# ui: status

2026-11-09. Where skein's code lags or departs from [ui.md](ui.md).

Section 6's binding of `Near` to a continuing list's foot is present.
Unlisted sections have no known gap.

Sections 2 and 16 include the full-screen shell's two crates; its own
remaining departure and deferred work are in `ui-screen.status.md` and
`ui-screen.md`, section 10.

Sections 3.2, 4.2 and 14 have no known gap in a field's options: the
builder restricts the state to a menu, and the outline requires a
preceding text field in its innermost form. Section 10 has no known gap
in underscore emphasis and strong, matching delimiter bytes and keeping
underscores within words literal. Neither leaves deferred work here.

Sections 3.2, 5, 7 and 15 have no known gap in a node's hint: the
builder charges its bytes, including replacements, against the text
bounds, and a changed hint travels in an attribute patch.

Sections 3.3, 3.5, 10.1, 11.2, 11.5, 15 and 17 have no known gap in
the flow's owner, reader, bounds and layout, the scene's own height and
its own width, or the acceptance cases and text golden. Further flow syntax is deferred
in section 18.

Sections 11.2, 15 and 17 have no known gap in a mark's fade:
`Scene::fade` takes it after a line, a polyline or a path, refuses the
cases 11.2 names, and charges `scene::fade_bytes` on the scene's bytes
and its stops' tokens on its mark's; `Scene::mark_fade` gives it back.
`a_fade_is_taken_or_refused_as_ui_md_says` and the golden scene hold
it. A fill's, a radial and other marks' fades are section 18's.

- **Not yet:** 3: the builder's assertion against a node inside a named button, tab or menu item (`skein-ui/src/builder.rs`, `close`).
- **Departs:** 3.1: the `debt-28` request of 2026-11-06 has each of its items end with fewer lines than it started; the role sets' named predicates ended 80 lines of Rust up, since one exhaustive `match` in `skein-ui/src/tree.rs` outweighs the short sites it replaced. `temper-client-view/src/tests/choice.rs` keeps one such slice, in a file that request keeps out of. 3.4: a progress still takes a text child: `close` asserts only that a fraction has runs of its own, so a progress with no fraction may hold its words in a child (`temper-client-shell-terminal/src/tests/fixtures.rs`), and one with a fraction may hold a child beside its runs. The text shell prints such a child's words; the web mapping gives them no `aria-valuetext`. It stays so that no fixture moves.
