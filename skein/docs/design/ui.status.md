# ui: status

2026-11-07. Where skein's code lags or departs from [ui.md](ui.md).

- **Not yet:** 3: the builder's assertion against a node inside a named button, tab or menu item (`skein-ui/src/builder.rs`, `close`).
- **Departs:** 3.1: the `debt-28` request of 2026-11-06 has each of its items end with fewer lines than it started; the role sets' named predicates ended 80 lines of Rust up, since one exhaustive `match` in `skein-ui/src/tree.rs` outweighs the short sites it replaced. `temper-client-view/src/tests/choice.rs` keeps one such slice, in a file that request keeps out of. 3.4: a progress still takes a text child: `close` asserts only that a fraction has runs of its own, so a progress with no fraction may hold its words in a child (`temper-client-shell-terminal/src/tests/fixtures.rs`), and one with a fraction may hold a child beside its runs. The text shell prints such a child's words; the web mapping gives them no `aria-valuetext`. It stays so that no fixture moves.
