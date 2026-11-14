# ui: status

2026-11-14. Where skein's code lags or departs from [README.md](README.md).

Section 6's binding of `Near` to a continuing list's foot is present.
Unlisted sections have no known gap.

`README.md`, section 2 and `shells.md` include the full-screen shell's two crates; its own
remaining departure and deferred work are in `screen.status.md` and
`screen.md`, section 10.

Sections 3.2, 4.2 and 12 have no known gap in a field's options: the
builder restricts the state to a menu, and the outline requires a
preceding text field in its innermost form. `markdown.md` has no known gap
in underscore emphasis and strong, matching delimiter bytes and keeping
underscores within words literal. Neither leaves deferred work here.

Sections 3.2, 5, 7 and 13 have no known gap in a node's hint: the
builder charges its bytes, including replacements, against the text
bounds, and a changed hint travels in an attribute patch.

`README.md`, sections 3.3 and 3.5, and `markdown.md`, section 1, and `scenes.md`, sections 2 and 5, and `README.md`, sections 13 and 14 have no known gap in
the flow's owner, reader, bounds and layout, the scene's own height and
its own width, or the acceptance cases and text golden. Further flow syntax is deferred
in `README.md`, section 15.

`scenes.md`, section 2, and `README.md`, sections 13 and 14 have no known gap in a mark's fade:
`Scene::fade` takes it after a line, a polyline or a path, refuses the
cases `scenes.md`, 2 names, and charges `scene::fade_bytes` on the scene's bytes
and its stops' tokens on its mark's; `Scene::mark_fade` gives it back.
`a_fade_is_taken_or_refused_as_ui_md_says` and the golden scene hold
it. A fill's, a radial and other marks' fades are `README.md`, section 15's.

- **Not yet:** 3: the builder's assertion against a node inside a named button, tab or menu item (`skein-ui/src/builder.rs`, `close`).
- **Not yet:** 14: the builder's capacity and fraction tests, `#[should_panic]`, go with `temper/docs/design/draft/panics.execution.md`, P8.
- **Departs:** 3.1: the `debt-28` request of 2026-11-06 has each of its items end with fewer lines than it started; the role sets' named predicates ended 80 lines of Rust up, since one exhaustive `match` in `skein-ui/src/tree.rs` outweighs the short sites it replaced. `temper-client-view/src/tests/choice.rs` keeps one such slice, in a file that request keeps out of. 3.4: a progress still takes a text child: `close` asserts only that a fraction has runs of its own, so a progress with no fraction may hold its words in a child (`temper-client-shell-terminal/src/tests/fixtures.rs`), and one with a fraction may hold a child beside its runs. The text shell prints such a child's words; the web mapping gives them no `aria-valuetext`. It stays so that no fixture moves.
