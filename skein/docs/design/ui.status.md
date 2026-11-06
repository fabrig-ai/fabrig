# ui: status

2026-11-06. Where skein's code lags or departs from [ui.md](ui.md).

- **Not yet:** 3: the builder's assertion against a node inside a named button, tab or menu item (`skein-ui/src/builder.rs`, `close`). 3.2: a list's order, its builder call and its attribute patch. 10: numbered lists: the reader keeps a marker as an item's words in a list with no order, and any line of that form ends a paragraph.
- **Departs:** 3.4: a progress still takes a text child: `close` asserts only that a fraction has runs of its own, so a progress with no fraction may hold its words in a child (`temper-client-shell-terminal/src/tests/fixtures.rs`), and one with a fraction may hold a child beside its runs. The text shell prints such a child's words; the web mapping gives them no `aria-valuetext`. It stays so that no fixture moves.
