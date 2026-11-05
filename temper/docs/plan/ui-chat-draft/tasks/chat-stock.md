+++
id = "chat-stock"
part = "chat-stock"
kind = "feature"
needs = ["note-scope", "note-answer"]
files = [
  "temper/crates/temper-service/src/stock.rs",
  "temper/crates/temper-service/src/stock/chat.txt",
  "temper/crates/temper-service/src/stock/configuring.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
  "temper/crates/temper-service/src/tests/tools.rs",
  "temper/crates/temper-service/src/limits.rs",
  "temper/crates/temper-service/src/tests/limits.rs",
  "temper/crates/temper-engine-smith/src/answers.rs",
  "temper/tests/real-loop/src/case/oracle.rs",
  "temper/tests/real-loop/src/case/scripts.rs",
  "temper/docs/design/domain/agent.status.md",
  "temper/docs/design/protocol/configuration.status.md",
]
effort = "high"
minutes = 80
+++

# A chat keeps a document: its authority, its instructions, its answer's words

## Goal

A stock chat, and only a chat, may write its own note scope; it's told when and how to keep its document; and the tool's answer says what the client's line reads.

Acceptance, temper's `protocol/configuration.md`, 13.4, and `domain/agent.md`, 8.3, as `notes/contracts.md`, *The stock's words and scopes*:

- **Stock:** `CHAT_TOOLS` 1,927 (note 128, recall 256); `ceiling()`'s `notes` `Scopes(1)`, so rules, policy's ceiling and every role; `project_role`'s spend-0 branch clears it for the observer; `CONFIGURING_TOOLS` 1,920. A goal's `Scopes(0)` from `goal.rs` and a change's from `forge_route.rs` stand, and a person's goal has none by `shapes` (`note-scope`). A goal or an advisor calling `note` is answered as today.
- **The tests read those:** `tests/stock.rs`, 83 `stock_chats_can_delegate…`, with `a stock chat's authority holds notes and recall`; 102 `configure_reference_authority…`, whose "before" loop flips for 128 and 256 and whose last assertion flips; 249 `the_configuring_agent…`, 1,920 and the new length; `tests/stock_brief.rs`' `1|2|4|512|1024` and figures.
- **`answers.rs`:** the words the note gives: `Note N revision R written · L lines` when `revision == 1`, `Note N revision R written · +A −D · line F` otherwise. A pure helper and tests beside its neighbours'; `tests/tools.rs:129`'s pin. The real loop's readers (`oracle.rs`, 1307, 3463, 4060, 5020 exact, 3930 `strip_suffix`; `scripts.rs` cues) read the new first-write words; `rg -n 'revision 1 written'` for the others.
- **`chat.txt`:** after the chat's own paragraph, its instruction, in its words, as `inputs/request.md`, *Instructions*: when the person asks for a draft, or says "write this up", keep one document: `note` with scope goal, the project and your own task number, both from your brief's `[You are task T in project P]`, description its title, body Markdown beginning with `# TITLE` and a muted line "Draft, DATE, with the user.", whole each time, naming the note and the revision you read, one `##` section per topic, open questions under "## Open questions". Say in a sentence what changed; don't repeat it. Mermaid isn't drawn. If a write is too large, shorten it. A goal's and an advisor's lines stay; the paragraphs' "this paragraph alone is yours" is reworded so that the document's rule is the chat's and no one else's.
- **`configuring.txt`:** the same, its title "How PROJECT works", written once the person's way is understood, kept current as the conversation changes it. It delegates nothing and effects nothing, as today.
- **The host's room:** each charter still leaves at least 2,048 bytes (`brief_room`, `room.rs`); `MARGIN` 567 is what `chat.txt` may grow by, then `root.brief.brief_bytes` falls at its definition, with its readers: `limits.rs:397` and its comment, `tests/limits.rs:180,191`, `temper/docs/design/protocol/configuration.md`, and `temper/docs/help/configure.md`. Say the new figure.
- The status lines in `agent.status.md` and `configuration.status.md` go.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The engine*, *temper*: *Stock*, *`temper-engine-smith/src/answers.rs`*, *Instructions*.
- `temper/docs/design/protocol/configuration.md`, 13.4; `temper/docs/design/domain/agent.md`, 4.3.2, 8.1, 8.2, 8.3.

## Owns and may change

Owns, as `files` lists them.

May also change: `temper/docs/help/**` and `temper/docs/help/reference/*`, by their renderers, for a figure; `temper/docs/design/protocol/configuration.md`, a figure; `temper/crates/temper-service/src/tests/room.rs` if it pins one; `temper/tests/playground`'s kept log, by `SKEIN_CODEGEN_WRITE=1` on its `tests/suite/walk.rs`, if `me`'s bytes are in it; `temper/tests/client/**` that pin the charter's tools or `me`'s authority; `temper/crates/temper-init/src/tests/**` for the same.

Options, each pre-approved:

- **`temper-service`'s tests** that pin a role's `notes` as 0: follow, naming them.
- **A first write's words read by the real loop** at another revision: match what the loop writes.
- **`stock.rs`' doc comments** say what's true now, as `CHAT_TOOLS`' does.

## Steps

1. Stock, its tests; `heavy cargo nextest run -p temper-service --lib -E 'test(stock)'`.
2. The words; `heavy cargo nextest run -p temper-engine-smith`, and `-p temper-service --lib -E 'test(tools)'`.
3. The instructions, the room's figures; `-p temper-service --lib`.
4. A test that bites each: `a_goal_tool_goal_has_no_note_scope`, if `goal_tool.rs` doesn't hold it; `chat_txt_tells_a_chat_to_keep_one_document`; `configuring_txt_tells_it_how_project_works`.
5. `heavy cargo nextest run -p temper-real-loop`'s package, as `temper/tests/real-loop/Cargo.toml` names it, the cases whose oracle you changed; `-p temper-init`; the client world's `configuring_chat` suite.
6. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
