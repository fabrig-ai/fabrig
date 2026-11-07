# look-round-15: the shapes that cross its parts

Binding for `node-hint` and `chat-title`.

## A node's hint (skein-ui, `skein/docs/design/ui.md`, 3.2)

Words a shell with a pointer shows on hover, never needed to decide; never the node's name. Any role takes one.

```rust
// skein/crates/skein-ui/src/tree.rs, on `Node`, after `placeholder`
/// Words a shell with a pointer shows on hover, never needed to decide (ui.md, 3.2).
pub hint: Option<Box<[u8]>>,

// skein/crates/skein-ui/src/diff.rs, on `Attribute`, after `placeholder`
pub hint: Option<Box<[u8]>>,

// skein/crates/skein-ui/src/builder.rs, after `placeholder`
/// Give the open node words a shell with a pointer shows on hover, within its text bound.
pub fn hint(&mut self, bytes: &[u8])
```

- The builder charges it to the node's text bytes as it does a description: a second call replaces the first and its bytes.
- `Node::text_bytes` counts it; a change of it alone is one attribute patch.
- The web mapping: `title="<the words>"` on the node's own element, escaped as `placeholder` is (`skein/docs/design/ui-web.md`, 6). The text shell prints nothing of it (`skein/docs/design/ui-text.md`, 5).

## A chat's heading (temper-client-view, `temper/docs/design/client/README.md`, 3.5, *A chat's heading is short*)

```rust
// temper/crates/temper-client-view/src/pages/chat.rs
/// A chat's heading shows its title's short form (client/README.md, 3.5, A chat's heading is short).
pub(crate) fn short_title(title: &[u8]) -> Box<[u8]>

// temper/crates/temper-client-view/src/pages/session.rs
pub(crate) enum Heading<'a> {
    Words(&'a [u8]),
    /// A chat's: named `whole`, "TN · TITLE"; showing `short` in a browser and `numbered`,
    /// "TN · SHORT", in the terminal; with `hint`, the whole title, where it is cut.
    Chat { whole: &'a [u8], short: &'a [u8], numbered: &'a [u8], hint: Option<&'a [u8]> },
}
```

- `pages/chat.rs` makes all four: `whole` and `numbered` by `Word::ChatTitle` with the title and with its short form; `hint` is the title where `short_title(title)` differs from it, `None` otherwise.
- `pages/session.rs`, `top`, draws them: the heading's name `whole`; its text `short` in `Shell::Browser`, `numbered` in `Shell::Terminal`; `b.hint(hint)` in both shells where there is one.
