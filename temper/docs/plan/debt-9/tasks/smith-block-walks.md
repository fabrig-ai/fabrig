+++
id = "smith-block-walks"
part = "smith-block-walks"
kind = "feature"
needs = []
files = ["smith/crates/smith-domain-session/src/**", "smith/crates/smith-domain/src/**", "smith/crates/smith-protocol-llm/src/**", "smith/crates/smith-protocol-translate/src/**", "smith/crates/smith-local-domain/src/person.rs", "smith/crates/smith-local-shell/src/front/**", "smith/tests/**", "temper/tests/system/tests/suite/composed.rs", "smith/docs/development/debt/D005-block-walks.md", "smith/docs/development/debt/README.md"]
effort = "high"
minutes = 180
+++

# smith: pay D005, a tool call is its own type inside `Block`

## Goal

Pay `smith/docs/development/debt/D005-block-walks.md`. `smith_domain_session::llm::Block::ToolCall` holds a
`ToolCall` struct. `Block::tool_call()` and `Block::text()`, each matching every variant, sit beside the enum,
and the session's walks use them. A new field on a call touches the struct, and a new block kind breaks those
accessors rather than every walk. No behaviour changes.

Acceptance:

- The item's measure reads 0.
- That alone proves nothing, as the shape change empties it. So also: in
  `rg -n 'Block::ToolCall\(' smith/crates/smith-domain-session/src -g '!*test*'`, no line has a field pattern inside
  the parentheses (`Block::ToolCall(ToolCall {`), but those that build a block; the rest bind a whole call in an
  exhaustive match of several kinds. Your report gives that list.
- `unreachable!` is gone from the text extractions (`rg -n 'is text' smith/crates/smith-domain-session/src -g '!*test*'`).
- The session's production lines fall: your report gives `git diff --shortstat` for
  `smith/crates/smith-domain-session/src`, excluding tests.
- The item's file and its README row are gone.

## Inputs

- `temper/docs/plan/debt-9/notes/common.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `skein/docs/foundation/programming-model.md`, 10.2 (`let .. else` on `Option`).
- `smith/docs/development/debt/D005-block-walks.md`.

## Owns and may change

- Owns: `smith-domain-session/src/**`.
- This public shape's break is the item's own, so it is approved, for `Block::ToolCall` alone. Its readers, each
  edited only for the pattern's or the literal's shape: the protocol crates' translators
  (`smith-protocol-llm`, `smith-protocol-translate`, `smith-protocol-program`'s `backend/`), `smith-domain`
  (`peer.rs`), `smith-local-domain/src/person.rs`, `smith-local-shell/src/front/`, the tests under `smith/tests/`,
  `temper/tests/system/tests/suite/composed.rs`. 76 files hold 238 sites at base; `heavy cargo check --tests`
  finds the rest. `smith-protocol-program` is another task's at the same time: edit there only the lines that
  name `Block::ToolCall`.
- **Other enums are not this one:** `smith_domain::llm::Said::ToolCall` (it stays a struct variant),
  `smith_events::Block`, `skein_llm::types::Block`, `jig_client_domain`'s `Block` and `smith_transcript`'s
  generated `Block` don't change, and no wire or stored form moves. The translators map fields by name.
- An assertion that compares a block by value only changes the literal's shape. Anything more is a stop and ask.

## Steps

1. In `llm.rs`:

   ```rust
   /// A tool call in a message. `id` is the provider's name for this call, which its result echoes; `name` and
   /// `input` are what the LLM wrote, the input a JSON object; `call` is what the protocol layer decoded from them.
   #[derive(Clone, PartialEq, Eq, Hash, Debug)]
   pub struct ToolCall {
       pub id: Box<[u8]>,
       pub name: Box<[u8]>,
       /// Provider arguments, or canonical `{}` for an oversized or cut call; counted with their enclosing
       /// message against session ownership limits.
       pub input: Box<[u8]>,
       /// Full decoded classification: ...  (the variant's text, moved)
       pub call: Decoded,
       pub replay: Option<Replay>,
   }

   pub enum Block {
       // ...
       /// The LLM asks for a tool to run.
       ToolCall(ToolCall),
       // ...
   }

   impl Block {
       /// The call this block holds, if it is one.
       #[must_use]
       pub const fn tool_call(&self) -> Option<&ToolCall> { /* every variant */ }

       /// The text this block holds, if it is a text block.
       #[must_use]
       pub fn text(&self) -> Option<&[u8]> { /* every variant */ }
   }
   ```

   `smith_domain::llm`'s `pub use` gains `ToolCall`. Add `tool_call_mut` only if a walk writes a call's field.
2. Sweep the shape: `Block::ToolCall { id, .. }` → `Block::ToolCall(ToolCall { id, .. })`, compile-driven.
3. The session's walks, one commit per file (`session.rs`, `session/restore.rs`, `batching.rs`, `compaction.rs`,
   `history.rs`, `builtin.rs`, `accounting.rs`):
   - a walk that wants only calls takes `let Some(call) = block.tool_call() else { .. }` or
     `content.iter().filter_map(Block::tool_call)`, then matches `call.call`;
   - a walk that wants only text uses `block.text()`. Where the walk knows the variant,
     `.expect("summary instruction is text")` replaces the `match` and its `unreachable!`;
   - a match that means several kinds stays exhaustive and binds `Block::ToolCall(call)`, reading `call.id`.
     A match of every variant is the point of the rule, so don't force it through an accessor.
4. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p smith-domain-session
   -p smith-domain -p smith-protocol-llm -p smith-protocol-translate -p smith-protocol-program --all-targets --
   -D warnings`; `heavy cargo nextest run` on those five and `smith-agent-service`; the worlds:
   `smith/tests/session`, `smith/tests/protocol-llm`, `smith/tests/agent`, `smith/tests/run` (package names from
   their `Cargo.toml`); `heavy cargo check --tests` on every other crate you edited.
5. Delete the item and its row; the citations check. Commit and push.

## Report

The common note's items. Add one unit test of the two accessors beside the enum (each variant, what each returns).
Break: make `tool_call` answer `None` for the call variant and name the failure. Say which walks stayed as matches
and why, and give the measure's count before and after.

## Answers

2026-11-04 (agents/loop, for fabrig/debt). **Allow the single `Block::ToolCall` literal in `temper/tests/client/tests/live_text.rs:74`.** It's one repair caused by this task's change.
