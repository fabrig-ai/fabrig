# resume-fits-room: the shapes that cross tasks

These bind. Names, fields and words are as written here; what is inside
a function is the task's. The design is `jig/docs/design/protocol/channel.md`,
9 (*A resumed start fits its room first*) and 11.

## 1. Frames and the room

A frame's bytes are its body's, `measure()` of the generated record,
plus its eight-byte header. The room is the engine's stream's, io's
`Limits::largest_room()` (its `output`), as
`temper/docs/plan/worker-link-quiet/notes/contracts.md`, section 1 has
it. A frame fits when its bytes are at most the room.

## 2. The fit, `jig-protocol-worker-wire` (task `fit`)

```rust
// jig/crates/jig-protocol-worker-wire/src/fit.rs; `pub mod fit;` in lib.rs
//! A resumed start's committed calls fitted to a room (protocol/channel.md, 9 and 11).

/// What fitting did to an assignment.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fitted {
    /// Its frame fits as it came: nothing cut.
    Whole,
    /// `cut` host answers were cut, the oldest; its frame fits.
    Cut { cut: u32 },
    /// Past the room with every host answer cut; `bytes` is its frame so cut.
    Past { bytes: u32 },
}

/// The words a cut host answer's body becomes, `bytes` the body's length.
#[must_use]
pub fn cut_words(bytes: u32) -> Box<[u8]>;

/// `assignment` with its committed calls fitted to `room`, a frame's bytes.
#[must_use]
pub fn fit(assignment: v9::Assignment, limits: &v9::Limits, room: u32) -> (v9::Assignment, Fitted);
```

```rust
// jig/crates/jig-protocol-worker-wire/src/sizes.rs
/// The smallest resumed start's frame, its header included (protocol/channel.md, 11).
#[must_use]
pub fn smallest_resumed(calls: u32, name: u32, tool: u32, answer: u32) -> Option<u32>;
```

**The words.** ASCII, exactly:
`[this call ran; its answer of BYTES bytes was left out of this start]`,
`BYTES` in plain decimal (`skein_lib::Decimal`). Its length is 64 plus
BYTES's digits.

**The fit,** in this order:

1. The frame's bytes as it came. If they are at most `room`, it is
   returned as it came, `Whole`.
2. A host answer can be cut when its body is longer than its words. A
   delivery's answer, and a host answer no longer than its words, are
   never cut.
3. The frame with every host answer that can be cut, cut. If that is
   past `room`, that assignment is returned, `Past { bytes }`.
4. Otherwise, from the newest committed call back (the list is in
   commit order, the newest last), each host answer that can be cut is
   kept whole while the frame with it fits; the first that does not fit
   and every older one that can be cut stay cut. Returned `Cut { cut }`,
   `cut` the answers left cut, at least one.

A cut answer keeps its call's name and tool, its place in the list and
its error flag; only its body changes. Nothing else of the assignment
changes. A sum past `u32` is past the room: `Past { bytes: u32::MAX }`.
Rebuilding a record through its `Parts` with its own `limits` cannot
fail for a cut body, which is shorter than the one it replaces.

**The smallest resumed start** is an assignment's frame with no
workspace, `save` false, every live ask false, an empty charter and
transcript, no message and no grant, and `calls` committed calls, each
with a name of `name` bytes, a tool of `tool` bytes and a host answer
whose body is `cut_words(answer)`. By the generated encoding (header 8,
fixed fields 57, each call 14 + `name` + `tool` + the words), on the
stock limits (`calls` `core.call_records`, `name`
`core.fleet.call_name_bytes`, `tool` the wire's `call_tool` ceiling, 64,
`answer` `core.call_answer_bytes`): small `8 + 57 + 32 × (14 + 64 + 64 +
69) = 6,817`; standard `8 + 57 + 128 × (14 + 64 + 64 + 70) = 27,201`.
`None` past `u32`.

## 3. The engine's link (task `fit-room`)

- Where `jig-protocol-engine-workers` measures an assignment's frame
  against `Limits.room` (worker-link-quiet's `engine-room`), it first
  calls `jig_protocol_worker_wire::fit::fit(record, &wire_limits,
  limits.room)`. `Whole` and `Cut` send the returned record, with no
  `Unsent`; `Past` takes the refusal path for a frame past the room
  as it is, with the returned record, so its words name `Past`'s bytes.
  A job is never fitted.
- `valid(&Limits)` also holds `smallest_resumed(sizes.answered,
  sizes.call_name, sizes.tool_name, sizes.answer)` is `Some` and at most
  `room`.
- Nothing else of the component's surface changes: no new `Request`,
  `Output` or `Limits` field.

## 4. temper (tasks `named-floor`, `temper-fits`)

- `temper/crates/temper-service/src/startup/checks.rs`:

  ```rust
  /// The smallest resumed start within the engine's room (configuration.md, 7).
  pub fn resumed_start(engine: &config::Engine, problems: &mut Problems);
  ```

  It reads the engine's `limits.io.largest_room()` and
  `smallest_resumed(core.call_records, core.fleet.call_name_bytes,
  jig_protocol_worker_wire::CEILINGS.call_tool, core.call_answer_bytes)`,
  the same fields `crate::workers::limits` gives the engine's `Sizes`,
  and fails through `short`, path `engine.limits.io.output`, words
  `the smallest resumed start`:
  `engine.limits.io.output: 4096 bytes, short of the smallest resumed start, 27201 bytes`.
  A `None` start, or a name bound past `u32`, fails as `short`'s `None`
  does (`past its bound`).
- `temper-fits` corrects worker-link-quiet's
  `a_stock_start_crosses_the_link_or_fails_with_its_numbers`
  (`temper/crates/temper-service/src/tests/link_room.rs`) to the fit:
  its calls row crosses on both profiles, its newest answers whole and
  the rest cut, as the numbers give.
