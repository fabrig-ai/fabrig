# transcript-window: contracts

The run's binding calls for the shapes that cross tasks. `turn-cut`
builds the jig half; `temper-cut` reads it and builds temper's.
Design: `jig/docs/design/protocol/store.md`, 8.3 (*A window's turn past
its page's room*); `jig/docs/design/domain/engine.md`, 7.2 (*A turn too
large for a page*); `temper/docs/design/domain/root.md`, 8.

## The store's cut page (`jig/crates/jig-protocol-store/src/boundary.rs`)

```rust
/// A transcript's turns; `next` is the offset to continue from after
/// a nonempty prefix. An empty page is terminal (store.md 8.3).
Turns {
    owner: Token,
    turns: Box<[core::TurnRecord]>,
    next: Option<u64>,
    /// A window's page whose one turn is past the read's room (store.md, 8.3):
    /// `turns` holds that turn alone, its fields read and its `transcript`
    /// empty, and this is its transcript's length.
    cut: Option<u32>,
},
```

- Only a `TranscriptRead::Window` read sets `cut`; every other read
  answers `cut: None`, as today.
- A cut turn is alone in its page. A page that already holds a turn ends
  before a turn past its remaining room (`next` at that turn's frame), so
  the next page starts at it and answers it cut. `next` after a cut page
  is the offset past the cut turn's frame, or `None` at the window's end.
- The window's opening turn is checked as today (its attempt, number and
  `opens_window`), cut or whole.
- A window read never answers `Unread` for a turn's size. `Unread` stays
  for a window that starts at another turn, a missing window, and a frame
  unread or refused.

## The core's cut turn (`jig/crates/jig-domain-core`)

```rust
/// The store transport's transcript answer (jig/docs/design/domain/engine.md, 4.1 and 7.2).
pub enum TranscriptLoad {
    Page { rows: Box<[crate::TranscriptRow]>, done: bool },
    /// A window's turn past its store page's room, alone in its page: its
    /// fields, `turn.transcript` empty and never read, and its transcript's
    /// length (jig/docs/design/domain/engine.md, 7.2).
    Cut { turn: crate::TurnRecord, bytes: u64, done: bool },
    Unread,
    Failed,
}

/// The text a cut turn keeps in a fresh brief's tail (domain/engine.md, 7.2).
pub const CUT_TURN_TEXT: &[u8] = b"[a turn too large to load]\n";
```

- `CUT_TURN_TEXT` lives in `runs/transcript_text.rs`, exported from the
  crate root beside `TranscriptRow`.
- The core admits a cut turn as `append_transcript` admits a turn (its
  task, attempt, order, and the window's opening turn); one it refuses
  fails the preparation as a refused page row does today. An admitted one
  counts `bytes + 4` in `Transcript.bytes`, sets `Transcript.last`, keeps
  `CUT_TURN_TEXT` as its text (cut to the tail's budget as any text is),
  keeps no body, and sets a new `pub cut: bool` on `Transcript`.
- A transcript with `cut` set resumes nothing: the claim's `resume`
  (`routing/run/preparation.rs`), `transcript_oversized` and
  `resumed_turns` each treat it as past the resume limit. Nothing is
  counted: no failure, no transient try, no refusal.
- A root maps a store page with `cut: Some(bytes)` to
  `TranscriptLoad::Cut { turn, bytes: u64::from(bytes), done: next.is_none() }`
  and asks for its next page as after any page.

## temper's half (`temper-cut` builds it)

- The store's cut page reaches temper's root through the service
  (`temper-service/src/store/route.rs`) and the archive
  (`temper-engine-domain/src/archive.rs`) as `TranscriptLoad::Cut`.
- Until `temper-cut` lands, `turn-cut`'s compiler-forced arm in the
  service answers a cut page as the store's `Unread` was answered
  (`StoreInput::Unloaded`), so temper keeps its trunk behaviour.
- The default shape inside temper: `TranscriptTexts` becomes
  `Box<[TurnText]>`, with `pub enum TurnText { Rendered(Box<[u8]>),
  Unrendered, Cut(u32) }` in `temper-engine-domain/src/boundary.rs`; a
  page whose one text is `Cut(bytes)` is the core's `Cut`. temper-cut may
  choose another temper-internal shape and says so.
