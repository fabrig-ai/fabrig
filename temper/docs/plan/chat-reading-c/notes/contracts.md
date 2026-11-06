# chat-reading-c: the shapes that cross tasks

Binding for every task of this plan. A later batch adds to this note and changes nothing in it.

## What didn't hold in the request

`Thread::newer` exists but reads nothing against a real engine: `jig_domain_core::reads::Page.before` is `None` on every path, so a page's `after` cursor never arrives, `Edges.before` stays `Edge::End` after a slide lets the newest lines go, and `newer` falls through to `moved`, which reads nothing while `committed <= loaded`. Calling `newer` from the chat page alone wouldn't bring the end back. And a cursor names a page's edge, where a window bounded by size lets lines go anywhere in a page. So a window reads again beside a line, by the line's position.

## skein-ui (task `log-surface`)

```rust
// skein-ui/src/binding.rs, exported from lib.rs
pub enum Act { /* as today */ Near { edge: LogEdge } }
/// An edge of a log (ui.md, section 6).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LogEdge { Head, Foot }
// takes(Role::Log, _, Act::Near { .. }) is true; every other role refuses it, and a log takes nothing else.

// skein-ui/src/tree.rs
pub enum State { /* as today */ Continues }   // asserted on Role::Log, as FoldsAway is on a disclosure
pub struct Attribute { /* as today */ pub end_request: Option<u64> } // beside focus

// skein-ui/src/builder.rs
pub fn end_request(&mut self, count: u64); // asserted on Role::Log; several logs may each hold one
```

`acted_on(Role::Log)` is true, so `Builder::bind` takes a log. A bound log is not numbered by the text shell.

## The page part (task `page-reading`)

- `Op::HoldView { node }`, at the start of an apply's ops, for each log not following; `Op::PutView { node }` after them.
- `Op::Measure { node }`, last, for each bound log, on any apply with patches. The platform half reads `scrollTop`, `scrollHeight`, `clientHeight` and gives the pure half `Event::Scrolled`.
- `Op::ScrollScreen { node, down: bool }`, for PageUp and PageDown.
- `data-log-away` while a log isn't following, `look-round-6`'s attribute, so temper's pill shows for a log that continues.

## The text shell (task `text-reading`)

`up`, `down` and the lines `ESC [ 5 ~` and `ESC [ 6 ~` make `Command::Act { node, first: Act::Near { edge }, submit: false }` on the first bound log in preorder. Printed first under a bound log's name, at its children's indent: `(up, down: more lines)`.

## The activity read (task `beside-surface`)

```rust
// jig-client-domain/src/contract/reads.rs
pub enum Read {
    /* as today */
    /// The page next to a line of a task's activity, that line left out (contract.md, section 6).
    ActivityBeside { task: u64, position: u64, side: Side },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Side { Earlier, Later }
```

- **Client wire:** in `jig/crates/jig-client-protocol-wire/schema/`, `record ReadActivityBeside { task: u64 position: u64 side: ActivitySide }`, `enum ActivitySide { earlier, later }`, and `activity_beside: ReadActivityBeside` last in `enum Read`. Its answer is `ReadValue::activity`: an `ActivityPage`, no new value.
- **The answer:** `Earlier`: rows just below `position`, newest first; `before` the usual older cursor, `after` none. `Later`: rows just above `position`, `after` some cursor when more remain above and none at the end, `before` none. The thread orders a page's rows itself. A `Later` page's `after` is a flag to the client: it reads on beside its newest line, and never gives that cursor back.
- **Core:** `jig_domain_core::reads::Read::ActivityBeside { task, position, side: reads::Side }`: `Scope::Activity { task }`, `Range.after = Some(Position::Owner(Key::Tasks(Activity { task, position })))`, `Range.backwards = side == Earlier`, read with `activity{n}`'s access.

## The thread's window (task `thread-window`)

```rust
// jig-client-domain/src/contract/documents.rs
/// What a line weighs in a window: its bytes, and this for it and for each
/// message, block, object and part it holds (client.md, section 10).
pub const LINE_ROOM: u32 = 64;
impl DecodedActivityLine { #[must_use] pub fn weight(&self) -> u64; }

// jig-client/src/thread/window.rs
pub enum Edge {
    End,
    More(Cursor),
    /// The window let lines go at this end; they are read again beside the line next to it.
    Dropped,
}
// Edges::reaches holds for Edge::End alone.

// jig-client/src/thread.rs
impl Thread {
    /// `More(cursor)`: as today. `Dropped`: `ActivityBeside { position: oldest held, side: Earlier }`.
    pub fn older(&mut self) -> Option<Read>;
    /// `End`: `moved`. `More(_)` or `Dropped`: `ActivityBeside { position: newest held, side: Later }`.
    pub fn newer(&mut self) -> Option<Read>;
    /// The list's end again: `Read::Activity { before: None }`, whose page replaces the window.
    pub fn latest(&mut self) -> Option<Read>;
}
pub struct Thread { /* as today, without `left` */ pub read_failed: Option<End> }
```

- `Line::Left` and `Thread.left` are gone. `thread::Limits` keeps its four fields: `lines` is the reference UI's and the tools', whose trees are priced by rows. temper's is `document_bytes / LINE_ROOM`, set by `chat-reads`, so that it never bites: the draft's own alternative ("or it grows far past what the bytes hold").
- `Thread::lines()` holds no `Words` or `Provisional` line while `lines.edges` doesn't reach `End::Before`.
- `reads` on `Thread` (`Reading::Older`, `Reading::Newer`, `Reading::Latest { known }`) keep their names. `latest` reads with `known: 0`, and `moved` still reads a new tail only while the window reaches it.

## A log's busy state (task `log-busy`)

`Event::Scrolled`, on a measure after `State::Busy` left a bound log whose edge is near, returns `Near` for that edge whatever its height. A log that holds `Busy` hears nothing from its height moving or from an apply; a person's move a quarter of a screen toward the edge still reports.

## temper's domain (task `chat-reads`)

```rust
// temper-client-domain/src/pages.rs
pub enum Act { /* as today, Older and Latest kept */ Newer }

// temper-client-domain/src/pages/chat.rs
impl Chat {
    /// The log's end request (client/README.md, 5.15): moves when the person sends or presses Latest.
    #[must_use] pub fn to_end(&self) -> u64;
}
pub fn thread_limits(limits: &Limits) -> jig_client::thread::Limits; // `lines`: document_bytes / LINE_ROOM
```

- `Older`: `thread.older()`, `Newer`: `thread.newer()`, on a chat's page and an agent's. A second one while a read is out does nothing.
- `Latest` and `Send` move `to_end`, and read `thread.latest()` where the window doesn't reach its end.
- `room`, `fill` and `offers_older` go, and `pub const LINES` stays until `chat-page` deletes it. An empty page with an older cursor reads on at once, so a shell that measures nothing isn't stranded: what `fill` did for an empty window.

## temper's view (tasks `view-window`, `chat-page`)

- `frame.rs`: `Act::Reading`, a binding's meaning alone: `(Act::Reading, skein_ui::Act::Near { edge: LogEdge::Head })` decodes to `Act::Older`, and `{ edge: LogEdge::Foot }` to `Act::Newer`.
- `temper_client_view::pages::chat::line_key(&Line<'_>) -> Option<skein_ui::Key>`, its kinds apart from `pages::inbox::card_key`'s.
- `Limits::of`, the window's part: `window_nodes = max_nodes(document_bytes) + document_bytes / LINE_ROOM * 10`, and `max_text_bytes(document_bytes)` beside it in `text_bytes`, in place of every `LINES *`. Each room priced at 10 nodes holds for the turn's structure, the most a block makes (`structure`'s `2 + 8`).
- `Word::LoadingEarlierFailed`: "Couldn't load earlier messages". `Word::Older` goes.
