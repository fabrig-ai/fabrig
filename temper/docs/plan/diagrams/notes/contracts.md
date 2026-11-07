# diagrams: the contracts

The run's binding calls. The design is `skein/docs/design/ui.md`, 3.3, 3.5, 10.1, 11.2 and 11.5, and `skein/docs/design/ui-web.md`, sections 8 and 12. This note fixes what the design leaves to code and several tasks share.

## `skein_ui::scene::Scene` (`skein/crates/skein-ui/src/scene.rs`)

```rust
/// Say this scene's height, in place of its box's (ui.md, 11.2).
pub fn set_height(&mut self, height: u16);
/// The height it said, if it said one.
#[must_use]
pub const fn height(&self) -> Option<u16>;
```

- `clear` forgets it, so a refused scene says none.
- `Display` prints a first line `height N` only where the scene says one, so every golden that says none stays as it is.

## The web mapping (`skein/crates/skein-ui-web`)

- `svg::draw(scene, names, width, height)` keeps its signature, and writes `scene.height().unwrap_or(height)` in `height` and `viewBox`.
- The page part sets an empty `data-scene-height` on the figure, beside `data-drawn`, where the scene it drew says a height. It writes no attribute on the SVG.
- The page part holds `Option<Scene>`, made the first time it draws.

## `skein_ui::flow` (`skein/crates/skein-ui/src/flow.rs`, `pub mod flow` in `lib.rs`)

```rust
pub const OWNER: Owner = Owner { number: u32::MAX };
pub const KIND: DrawingKind = DrawingKind { owner: OWNER, kind: 0 };
pub const BOX: Style = Style { owner: OWNER, token: 0 };
pub const NAME: Style = Style { owner: OWNER, token: 1 };
pub const CAPTION: Style = Style { owner: OWNER, token: 2 };
pub const ARROW: Style = Style { owner: OWNER, token: 3 };
pub const STRONG: Style = Style { owner: OWNER, token: 4 };
/// `skein-ui-flow-box`, `-name`, `-caption`, `-arrow`, `-strong`, by those tokens; prefix `skein`.
pub const NAMES: Names<'static>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction { LeftToRight, TopToBottom }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Node<'a> {
    pub name: &'a [u8],
    pub caption: Option<&'a [u8]>,
    pub strong: bool,
}

/// None past `bound`, or for a name or caption past a `u16`'s bytes.
pub fn encode(direction: Direction, nodes: &[Node<'_>], bound: u32) -> Option<List<u8>>;
/// The whole data checked first: None for anything `encode` can't write.
pub fn decode(data: &[u8]) -> Option<(Direction, Nodes<'_>)>;
/// Iterates a decoded flow's nodes in the path's order. `Clone`.
pub struct Nodes<'a>;
impl<'a> Iterator for Nodes<'a> { type Item = Node<'a>; }

/// The marks a flow of `nodes` nodes takes at most: four each.
pub const fn marks(nodes: u32) -> Option<u32>;
/// The scene's bytes it takes at most: `data`, then 25 for each node's arrow.
pub const fn scene_bytes(nodes: u32, data: u32) -> Option<u32>;
/// The drawing code (ui.md, 11.5), a `scene::Draw`. In `flow/layout.rs`, re-exported here.
pub fn draw(kind: DrawingKind, data: &[u8], width: u16, height: u16, scene: &mut Scene) -> bool;
```

### The data's layout

| Bytes | What |
|---|---|
| 1 | the direction: 0 left to right, 1 top to bottom |
| then, for each node in the path's order: | |
| 1 | flags: bit 0 it has a caption, bit 1 it is strong; the other bits zero |
| 2 | its name's length, `u16` little-endian, at least 1 |
| n | its name's bytes |
| 2 | where it has a caption: the caption's length, `u16` little-endian, at least 1 |
| n | the caption's bytes |

At least one node. A node takes at least four bytes, which `skein/docs/design/ui.md`, 10.1 counts on.

## The reader's meeting point (`flow/read.rs`, `markdown.rs`)

```rust
pub(crate) struct Read<'a> {
    pub name: &'a [u8],     // empty for none
    pub data: List<u8>,
    pub fallback: List<u8>,
}
/// `info` the fence's opening line, `body` its lines up to the closing one.
/// None for anything that stays fenced code.
pub(crate) fn read<'a>(info: &[u8], body: &'a [u8], limits: &Limits) -> Option<Read<'a>>;
```

`markdown.rs` holds the tree's side, where a fence closes in `line`:

```rust
if let Some(flow) = flow::read::read(info, body, &builder.limits) {
    finish_closing(blocks, builder);
    builder.drawing(flow::KIND, flow.name, flow.data.as_slice(), builder.limits.drawing_bytes);
    open_block(Role::Paragraph, blocks, builder);
    builder.text(Run::Plain, flow.fallback.as_slice());
    close_block(blocks, builder);
    close_block(blocks, builder);
} else {
    // the fence's code, as today
}
```

`Builder::drawing` leaves the node with no name where `name` is empty (`skein/docs/design/ui.md`, 3.5).

## An application that draws flows

As temper's page does (`temper/crates/temper-client-shell-web/src/host.rs`):

```rust
Look { names: &[view::look::NAMES, skein_ui::flow::NAMES], draw: skein_ui::flow::draw, .. }
```

`flow::draw` returns false for any kind but `flow::KIND`, so an application with no other drawing gives it whole. Limits of its own, as temper's: `drawing_bytes: 2048`, `scene_marks: flow::marks(24)?`, `scene_bytes: flow::scene_bytes(24, 2048)?`. A flow past them shows its fallback.

## The tokens' selector

`skein-ui.css` gives every mark `stroke` and `fill` under `[data-drawn] > svg :is(..)`, which a lone `.skein-ui-flow-name` doesn't outweigh. Every rule for a flow's token, in `skein-ui.css` and `temper.css`, starts `[data-drawn] > svg `.
