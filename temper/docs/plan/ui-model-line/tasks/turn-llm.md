+++
id = "turn-llm"
part = "model-line"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-client-domain/src/contract/turn.rs",
  "jig/crates/jig-client-domain/src/contract/documents.rs",
  "jig/crates/jig-protocol-transcript/src/lib.rs",
  "jig/crates/jig-protocol-transcript/src/tests.rs",
  "jig/crates/jig-client/src/thread.rs",
  "jig/crates/jig-client/src/thread/model.rs",
  "jig/crates/jig-client/src/tests/thread.rs",
  "jig/testing/jig-client-scripted-engine/src/codec/turns.rs",
  "jig/testing/jig-client-scripted-engine/src/codec/tests.rs",
  "jig/docs/design/client/domain/client.status.md",
]
effort = "high"
minutes = 60
+++

# A decoded turn keeps its model and effort, and a thread knows the latest

## Goal

Smith's turn names the model its completion was made with and the effort it asked for, and its bytes reach the client whole, but `jig_protocol_transcript`'s `translate` drops both. Keep them on the client's `Turn`, and give `Thread` the newest, so that a page can say what a chat ran on. No stored format and no wire form changes.

Acceptance:

1. `Turn.llm` holds a decoded turn's model and effort; none where the turn names no model (an empty name). It stands outside the `turn_text` budget, and is never cut.
2. `Thread.latest_llm` is the `llm` of the newest decoded turn whenever the window reaches the live end. Reading older pages leaves it. A newest turn that doesn't decode leaves the turn before it as the answer. It is none with no turn read.
3. `Thread::caught_up()` is false from a task's document announcing a position until the thread has read it, and true with nothing announced. A read of older lines doesn't make it false.
4. A line's `weight()` counts `llm`'s bytes.
5. `jig-client-scripted-engine`'s codec writes `llm` into the turn's bytes, so its round trip holds a named model and effort.
6. jig's `client.status.md` has lost this plan's `Not yet` entry.

## Inputs

- `jig/docs/design/client/domain/client.md`, section 10: the bullets *Turns, decoded below the client's surface* and *What the conversation last ran on*.
- `smith/docs/design/protocol/transcript.md`, section 2: *model and effort*.
- `temper/docs/plan/ui-model-line/notes/contracts.md`: the binding names.
- `temper/docs/plan/ui-model-line/notes/common.md`.

## Owns and may change

**Owns:** the files in the header. **A departure from the implementer's text:** you remove this plan's entry from `jig/docs/design/client/domain/client.status.md`, and no other edit under `docs/design/`.

**Readers, one line each, a literal's new field `llm: None,`** (a `grep` for `read_through:` finds them; temper's lines are carried here, as the producer's branch of a required field):

- jig: `jig/tests/ops/src/client/stories.rs`, `jig/crates/jig-client/src/tests/fixtures/thread.rs`, `jig/crates/jig-client-protocol-link/src/tests/fixtures/turns.rs`, `jig/testing/jig-client-kit/src/stories.rs`, `jig/testing/jig-client-kit/src/scripted/stories.rs`, `jig/testing/jig-client-kit/src/stories/unread_turn.rs`, `jig/testing/jig-client-scripted-engine/src/tests.rs`, `jig/crates/jig-client-view/src/tests/tree/turns.rs`, `jig/crates/jig-client-view/src/tests/state/domain/conversations.rs`.
- temper: `temper/tools/gallery/src/worlds.rs`, `temper/crates/temper-client-view/src/tests/document.rs`, `.../tests/chat_sheet.rs`, `.../tests/delegation.rs`, `.../tests/fixtures/task.rs`, `.../tests/chat.rs`, `temper/crates/temper-client-domain/src/tests/fixtures/chat.rs`, `temper/crates/temper-client-tools/src/tests/fixtures/reads.rs`, `temper/crates/temper/tests/suite/forge_land.rs`, `temper/testing/temper-fixtures/src/first_run.rs`.

**Options, each decided:**

- If a test compares a turn decoded from real bytes with a literal, give that literal what the bytes name in place of `None` (`jig/tests/ops/src/client/stories.rs` may be one).
- If rows reach `Thread.lines` by another path than `accept` (where `fresh` meets the window), take `latest_llm` there too.
- If `loaded >= committed` doesn't hold as those fields' comments say, use what does, and say so in the report.
- Leave `temper-client-tools`' renders alone: `temper ctl` prints no model for a turn.

**Not yours:** any store or wire schema, `jig-domain-tasks`, `jig-domain-core`, `temper-client-view`'s code (`model-line`).

## Steps

### Implementation

```rust
// jig-protocol-transcript/src/lib.rs, in translate, beside usage
llm: llm(source),

/// The turn's model and effort, outside the text budget as its usage is (client/domain/client.md, 10).
fn llm(source: &smith::Turn) -> Option<ModelEffort>;
```

`Turn.llm` and `Thread.latest_llm` as `notes/contracts.md` has them. Take `latest_llm` in a private method, called where a page has been accepted: walk `self.lines.rows` newest first when `self.lines.edges.reaches(End::After)`, take the first `TurnRead::Decoded` whose `llm` is some, and leave the field as it is when there's none. `Thread::new` starts it at none.

`DecodedActivityLine::weight`: add `llm`'s bytes with `byte_weight`.

`jig-client-scripted-engine/src/codec/turns.rs`, `turn`: `model` and `effort` from `value.llm`, empty and none where it's none. Today they are `Box::new([])` and `None`.

### Tests

- `jig/crates/jig-protocol-transcript/src/tests.rs`: rows for a named model with an effort, a named model with none, and an empty name (`smith_smallest_golden_decodes_at_its_whole_length` has `llm: None`, the fullest golden what it names); and a turn so small a `turn_text` budget cuts its words keeps its `llm` whole.
- `jig/crates/jig-client/src/tests/thread.rs`, one table test, `the_latest_llm_is_the_newest_turn_at_the_live_end`: two turns, the newest wins; the newest undecodable, the one before; a window with no turn, none; reading older until the newest row goes, it stays. Beside `reading_older_lets_the_newest_go_and_reading_newer_the_oldest`, with its fixtures.
- The same file, `caught_up_follows_the_announced_position`: false after `document`, true after its page, still true during a read of older.
- A `weight` row where that file's window tests compare weights, if one fits; otherwise one assertion.
- `jig/testing/jig-client-scripted-engine/src/codec/tests.rs`, `committed_turns_use_the_transcript_and_client_codecs`: its turn names a model and effort.

Each must bite, as your text says.

### Finish

`cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-client-domain -p jig-protocol-transcript -p jig-client -p jig-client-scripted-engine --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-client-domain -p jig-protocol-transcript -p jig-client -p jig-client-protocol-link -p jig-client-scripted-engine -p jig-client-view -p jig-client-kit -p jig-client-world`; `heavy cargo check -p jig-ops-world -p temper-client-domain -p temper-client-view -p temper-client-tools -p temper-client-gallery -p temper-fixtures -p temper --tests`. Commit and push.

## Report

As your text says. Also: any literal you gave more than `llm: None`, and how long a turn's `llm` makes a line in a window weigh.
