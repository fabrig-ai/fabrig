# chat-reading-a: the shapes that cross tasks

Binding. Each is built by the task named, and read by the tasks whose `needs` name it.

## `skein_ui::markdown::read_growing` (`growing-reader`)

```rust
/// A text that may yet grow (ui.md, section 10). `end` runs once, if the text holds a
/// word, inside the block that holds the last word, after its runs.
pub fn read_growing(input: &[u8], heading_base: u8, end: impl FnOnce(&mut Builder), builder: &mut Builder)
```

`read` keeps its signature and what it reads. `max_nodes` and `max_text_bytes` hold for `read_growing`, and its caller counts `end`'s nodes.

## `jig_client_domain::contract::TurnLimits` (`turn-order`)

```rust
pub struct TurnLimits {
    pub messages: u32,
    pub blocks: u32,
    pub text: u32,
    /// The most bytes of a call's outcome line, within `text`'s budget.
    pub outcome: u32,
}
```

`jig-client-protocol-link` sets `outcome: client.text`. What a decoded turn holds:

- `Call.tool`: priced first.
- `Block::Text`, and `Block::Opaque.dialect`: priced next.
- `Call.outcome`: the result's first line with words: leading blank lines skipped, a trailing `\r` dropped, at most `outcome` bytes.
- `Call.input`: priced next.
- `Block::Result.summary`: whole, priced last.
- A text cut, by `outcome` or by the budget, ends in `…` (U+2026, three bytes) inside the bytes it kept, on a UTF-8 boundary. Where fewer than three bytes are left it is empty, with no mark. `Turn.cut` is set as it is today, and stays in the contract.

## `jig_client::thread::Limits` (`live-whole`)

```rust
pub struct Limits {
    pub lines: u32,
    pub bytes: u64,
    pub text: u32,
    /// The most bytes a turn being written keeps, from its start.
    pub live: u32,
}
```

`text` keeps bounding a person's kept words. `Line::Provisional { name, text, cut }` keeps its shape. `live_turn` keeps the first `live` bytes of what the client holds, on a UTF-8 boundary, and its `cut` is what it is today.

Until `turn-prices` lands, each reader sets `live` to the `text` it passes today.

## The temper values (`turn-prices`)

`turn_text` is `LONG_TEXT` (131,072) in `temper-client-shell-web` and `temper-client-shell-terminal`, whose `TURN_TEXT` constants go. `pages::chat::thread_limits` sets `live: long_text`, which also serves an agent's page. `temper-client-tools` keeps its own `turn_text: TEXT` and `live: text`.

## The turn being written's end (`chat-live`)

The phrase `P::Provisional` keeps its number, and its words become `▍` (U+258D), drawn through `words.status`. It is passed as `read_growing`'s `end`. The who slot of a turn being written says `P::Temper`, as a committed turn's does.
