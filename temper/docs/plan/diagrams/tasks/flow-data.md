+++
id = "flow-data"
part = "flow"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-ui/src/flow.rs",
  "skein/crates/skein-ui/src/flow/read.rs",
  "skein/crates/skein-ui/src/flow/layout.rs",
  "skein/crates/skein-ui/src/lib.rs",
  "skein/crates/skein-ui/src/tests.rs",
  "skein/crates/skein-ui/src/tests/flow.rs",
  "skein/crates/skein-ui/src/tests/flow/read.rs",
  "skein/crates/skein-ui/src/tests/flow/layout.rs",
]
effort = "medium"
minutes = 25
+++

# The flow's surface: its owner, tokens, types and data

## Goal

Create `skein_ui::flow` with what its reader and its drawing code both build on: skein-ui's owner, the kind, the tokens and their names, and the data's encoding and decoding. No reading of Markdown and no drawing yet: `flow-read` and `flow-layout` each take one file this task leaves ready, so they don't meet.

Acceptance:

- Every name in `temper/docs/plan/diagrams/notes/contracts.md`, *`skein_ui::flow`*, but `draw`, exactly as it stands there. `draw` is `flow-layout`'s.
- `decode(encode(..))` gives back what went in. `decode` refuses every byte string `encode` can't write, without a panic: no node, an unknown direction, a flag bit set beyond the two, a name of no bytes, a length past the end, a caption flag with no caption, and a caption of no bytes.
- `encode` gives `None` past its bound, and for a name or caption past 65,535 bytes.
- `look::check(&[flow::NAMES])` is `None`.
- Step-crate rules hold (`AGENTS.md`, 2): no indexing, no `as`, no unchecked arithmetic, every `match` whole.

## Inputs

- `skein/docs/design/ui.md`, 3.3, *skein-ui's own*, and 10.1, *What it makes*, its data.
- `temper/docs/plan/diagrams/notes/contracts.md`, *`skein_ui::flow`* and *The data's layout*.
- `temper/docs/plan/diagrams/notes/common.md`.

## Owns and may change

Owns the files in the header. `lib.rs` gains only `pub mod flow;` and `tests.rs` only `mod flow;`.

Readers: nothing reads this module yet. `skein_lib::{List, Reader, Writer}` are what `scene.rs` and `jig/examples/ops/client-view/src/drawing.rs` use for such bytes; `Reader` (`skein/crates/skein-lib/src/reader.rs`) has `u8` and `bytes(n)`. Its `u16` is big-endian, so read a length's two bytes as `scene.rs`'s `read_coordinate` does.

## Steps

1. `flow.rs`, with a module comment citing `ui.md`, 3.3, 10.1 and 11.5:
   - the constants, `Direction`, `Node`, `encode`, `decode`, `Nodes`, `marks`, `scene_bytes`;
   - `pub(crate) mod read;` and `mod layout;`.
2. `flow/read.rs` and `flow/layout.rs`: each holds only its module comment for now, `//! Reads a fence into a flow (ui.md, 10.1).` and `//! Lays a flow out as a scene (ui.md, 11.5).`, so that the later tasks touch neither `flow.rs`'s `mod` lines nor each other's files.
3. `tests/flow.rs`: `mod layout; mod read;`, and `tests/flow/read.rs` and `tests/flow/layout.rs`, each a comment alone. Then two table tests in `tests/flow.rs`:
   - `a_flows_data_reads_back`: rows of direction and nodes, with and without captions, strong and not, each through `encode` and `decode`.
   - `data_encode_cannot_write_is_refused`: rows of bytes and the reason, each `decode` `None`.
   Beside them, `marks` and `scene_bytes` in rows (`marks(4)` is `Some(16)`, `scene_bytes(4, 100)` is `Some(200)`, and each is `None` where it overflows), `encode`'s two refusals, and `look::check`.
4. Narrow test: `heavy cargo nextest run -p skein-ui --lib -E 'test(flow)'`. Then the closing checks of your text. Commit and push.

## Report

The implementer's report, with what you broke by hand for each new test.
