# tui-whole-t1: contracts

The shapes that cross this plan's tasks. Each section is binding for the
specs that cite it; a later batch adds sections, never changes one.

## 1. Printed views (`tui-views`)

`temper-client-tui`, `src/view/printed.rs`, new, the one place a view is
printed (`temper/docs/design/client/README.md`, 6.14, *A chat*, *a view*):

```rust
/// What the chat page holds for one view address, resolved at read time.
#[derive(Debug)]
pub(crate) enum Printed {
    Asking(Box<[u8]>),          // the address
    Failed(Box<[u8]>),          // the address
    Drawn(Box<w::View>),        // temper_client_protocol_wire::v1::View
}
pub(crate) fn read(address: &[u8], shown: Option<&Shown>) -> Printed;
pub(crate) fn draw(b: &mut Builder, printed: &Printed, words: &Words);
```

`Shown` is `temper_client_domain::pages::chat::Shown`. `view/chat.rs`'s
`Entry` gains one variant, `Viewed(Box<[Piece]>)`, with
`pub(crate) enum Piece { Text(Box<[u8]>), View(Printed) }`, for committed
agent text holding at least one view block; `Entry::Agent` keeps its shape
for text without one and for the words being written. A later task that
draws a chat's document (T2) calls `printed::read` and `printed::draw`.

No new TUI word, look token or theme entry: the marks are the module's
glyph constants, painted by existing tokens (`tui-signal`, none,
`tui-hint`, `tui-faint`), so the launchpad's tasks own `words.rs`,
`look.rs` and `theme.rs` alone.
