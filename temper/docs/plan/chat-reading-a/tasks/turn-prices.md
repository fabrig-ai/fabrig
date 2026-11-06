+++
id = "turn-prices"
part = "turn-budget"
kind = "feature"
needs = ["live-whole", "chat-committed", "turn-order"]
files = [
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-shell-web/src/limits.rs",
  "temper/crates/temper-client-shell-terminal/src/limits.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "jig/crates/jig-client-domain/src/contract/turn.rs",
  "jig/crates/jig-protocol-transcript/src/tests.rs",
  "temper/tests/client/src/fixtures.rs",
  "temper/tests/client/src/front.rs",
  "temper/tests/client/tests/suite/live_text.rs",
  "temper/tests/client/tests/suite/agent_turn.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 90
+++

# temper's clients hold a turn of 131,072 bytes, and the view prices it

## Goal

Raise temper's two values to `long_text`: a committed turn's budget (`turn_text`) and a turn being written (`thread_limits`' `live`). The view's bounds can't take that as they stand: `Limits::of` prices the tree's text as all its nodes at the longest node's text, which overflows a `u32` once a node may hold 131,072 bytes, and the shells refuse to start. Reprice it, and prove the whole path in the client world.

Acceptance:

- `turn_text` is `LONG_TEXT` in `temper-client-shell-web` and `temper-client-shell-terminal`, and their `TURN_TEXT` constants are gone. `pages::chat::thread_limits` sets `live` to `long_text`. `temper-client-tools` keeps `turn_text: TEXT` and `live: text`, and `the_tools_preset_is_the_terminals_but_its_rows` still holds.
- `temper_client_view::Limits::of` returns `Some` at both shells' limits, `valid` holds, and the terminal's `render_bound`, `Terminal::operations` and `Terminal::new` still hold. Tests there exist (`temper-client-shell-terminal/src/tests.rs`, `src/tests/render.rs`): they stay green unchanged.
- At a small domain's limits a full window, each of its `LINES` lines a turn of `turn_text` bytes of the Markdown that makes the most nodes and text, builds without the builder's panic. One line short of the bound on each side isn't what this proves: it proves `Limits::of` holds what a window can make.
- At the shells' limits, a chat's page with one turn of 131,072 bytes of ordinary Markdown builds and prints.
- In the client world, in the order the request gives:
  - a live answer past today's 512-byte window arrives whole, its start and its end both shown before the turn commits;
  - a live answer past the engine's 65,536 retained bytes, followed from its start, arrives whole;
  - a committed turn whose results are large keeps every call's name, and nothing on the page says "omitted";
  - a committed turn holding an opaque block draws no "Content in".
- The **Not yet** line beginning `5.4:` is gone from `temper/docs/design/client/README.status.md`.

## Inputs

- `temper/docs/design/client/README.md`, 5.4.
- jig's `client/domain/client.md` at `jig/docs/design/client/domain/client.md`, section 14, on `turn_text` and `long_text`.
- `skein/docs/design/ui.md`, section 10, *bounded*, and section 15.
- `temper/docs/plan/chat-reading-a/notes/common.md`.
- `temper/docs/plan/chat-reading-a/notes/contracts.md`, *The temper values*.
- `temper/docs/plan/chat-reading-a/plan.md`, *The turn budget*: your report confirms its figures or corrects them.

## Owns and may change

Owns the two values and the view's bounds, as `files` lists them.

May also change, these edits alone:

- `temper/crates/temper-client-view/src/tests/**`, `temper/crates/temper-client-domain/src/tests/**`, `temper/crates/temper-client-shell-terminal/src/tests/**`: a test that pins 2,048 or a live line's 512: its expected value. A test that pins a cut at those bounds keeps its rule at its own small limits.
- `temper/crates/temper-client-view/src/pages/chat.rs`: `provisional` stops cutting the message to `text_limit`, reading all of it with `skein_ui::markdown::read`, and `markdown` goes with its last reader. `conversation_with_times` binds `let (rows, _) = limits;`, its signature unchanged. Nothing else there: the who slot, "Writing...", `read_growing`, the end and the argument's removal are `chat-live`'s.
- `temper/tools/gallery/src/worlds.rs`: compiles as it is. Change it only if a gallery test proves a cut there is needed, and say so.

Leave `temper/testing/temper-fixtures/src/first_run.rs`: its `received` cuts nothing at 131,072 bytes.

Options, each pre-approved:

- **The pricing's shape,** a sketch binding on its meaning, not its names:

  ```rust
  // Each node outside a window's turns holds at most `text`, which no longer takes turn_text.
  let line = structure.max(markdown + 8).max(plain).max(activity); // a line's nodes, its turn's Markdown aside
  let turn = receiving.turn_text.max(receiving.long_text);         // a turn committed or being written
  let structural = /* today's sum, with `turns` at `line` */;
  let text_bytes = structural * text
      + LINES * max_text_bytes(turn)                               // what the reader may make of a turn
      + max_text_bytes(receiving.long_text)                        // a document
      + text * 24 + rows * builder_name;
  let nodes = structural + LINES * (max_nodes(turn) + 3) + document; // + 3: the end's mark
  ```

  Every `checked_` stays. If `structural * text` leaves too little room in a `u32`, stop and report the figures: that is a `Question:`, since it bounds what a page may hold.
- **A disclosure's name:** `fold_bytes` stops reading `turn_text`: at most `rows` names a message, each at most `CALL_NAME` bytes, so it stays below `text` at the shells' limits. `pub const CALL_NAME: u32 = 64;` goes in `jig-client-domain`'s `contract/turn.rs`, and a test in `jig-protocol-transcript/src/tests.rs` holds it equal to `smith::CEILINGS.call_name`, as nothing above that crate links smith.
- **The world's limits:** `fixtures::domain_limits()` stays. Add `fixtures::world_with_whole_turns(seed)`, its `long_text` and `turn_text` at 131,072 and whatever those need (`document_bytes`, `turn_bytes`, `requests.bytes`, the engine's turn bytes as `with_long_turns` sets them).
- **A long reply's pieces:** `reply_in_pieces` takes `&'static` pieces, so a piece is a `static [u8; N]` of a repeated word.
- **The large results' turn:** a fixture beside `large_turn_body`, building `session_turn`'s messages with `smith_domain_session::llm` blocks: three calls, and three results of 64 KB each, so the three weigh more than the budget. `rows: 4` holds them.
- **The heap:** `temper-client-view` has no meter, and `temper-client-shell-web/tests/suite/memory.rs` has. If the host can be given a window of whole turns cheaply, keep a test there in that file's idiom, its ceiling the peak plus 25%, rounded up to two figures. If not, measure with throwaway code and give the figures in your report, adding no feed to the host for it.

## Steps

1. **Red first.** The world's four stories in `temper/tests/client/tests/suite/live_text.rs` (the opaque one may sit in `agent_turn.rs`, beside `large_turn_with_opaque`'s readers):

   ```rust
   #[test]
   fn a_live_answer_past_a_fields_window_arrives_whole()
   #[test]
   fn a_live_answer_past_the_engines_retained_bytes_arrives_whole()
   #[test]
   fn a_committed_turn_with_large_results_keeps_every_calls_name()
   #[test]
   fn a_committed_turn_draws_no_opaque_block()
   ```

   And `Limits::of` at the raised value, which is `None` before the repricing: `temper-client-view/src/tests/`, in a file beside its limits' readers, with the full-window test.
2. **Reprice,** then raise the values. Comments in `limits.rs` that speak of a cut status or of `thread::live_turn` at text bytes say what's true now.
3. **A change of meaning:** run `heavy cargo nextest run -p temper-client-view -p temper-client-domain -p temper-client-shell-terminal -p temper-client-shell-web -p temper-client-tools -p temper-client-world -p temper-client-gallery -p jig-protocol-transcript`.
4. **The ceilings:** `temper/tests` heap ceilings shouldn't move, as the engine holds what it held. One that does is named in your report, with its measurement before and after.
5. Fmt, clippy on the packages you changed, commit and push.

## Report

As the implementer's text says, and: `Limits::of`'s `nodes`, `text_bytes` and `patches` before and after at both shells' limits; the heap's figures for a full window; and whether `plan.md`'s table holds.

## Answers

2026-11-06. **The two live stories can't pass while `provisional` cuts the message to `text_limit`, in a file outside this task: widen the task, or move them to `chat-live`?** Widen. The stories stay here and the acceptance is unchanged, as raising `live` proves nothing while the page cuts it, and they are the request's own order of proof. "Owns and may change" names the one hunk in `pages/chat.rs`, and `files` lists it. `Limits::of`, as repriced, already holds a line of `long_text`, so the cut guards nothing here. `a_long_provisional_text_says_nothing_of_what_was_cut` stays as it is. Start from the pushed `d3514d9555`, and finish what stopped with the question: the unfiltered run of step 3, the heap's figures, the content mutations of the committed stories, and the `5.4:` status line.
