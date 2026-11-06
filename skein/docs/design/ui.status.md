# ui: status

2026-11-06. Where skein's code lags or departs from [ui.md](ui.md).

- **Not yet:** 3.2: the state *to log end*; the plan `look-round-6` builds it.
- **Not yet:** 3: the builder's assertion against a node inside a named button, tab or menu item (`skein-ui/src/builder.rs`, `close`).
- **Departs:** 3.4: a progress still takes a text child: `close` asserts only that a fraction has runs of its own, so a progress with no fraction may hold its words in a child (`temper-client-shell-terminal/src/tests/fixtures.rs`), and one with a fraction may hold a child beside its runs. The text shell prints such a child's words; the web mapping gives them no `aria-valuetext`. It stays so that no fixture moves.
