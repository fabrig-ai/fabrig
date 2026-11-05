+++
id = "note-tool"
part = "note-tool"
kind = "feature"
needs = ["note-range"]
files = [
  "temper/crates/temper-engine-smith/src/notes.rs",
  "temper/crates/temper-engine-smith/src/calls.rs",
  "temper/crates/temper-engine-smith/src/answers.rs",
  "temper/crates/temper-engine-smith/src/tests.rs",
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/translate.rs",
  "temper/crates/temper-engine-domain/src/route/tools.rs",
  "temper/crates/temper-service/src/stock/chat.txt",
  "temper/crates/temper-service/src/stock/configuring.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/tools.rs",
  "temper/tests/engine/domain/tests/suite/root_routes.rs",
  "temper/docs/help/reference/host-tools.md",
]
effort = "high"
minutes = 60
+++

# temper's run writes a note by lines, and its instructions say to

## Goal

jig's `note-range` reaches temper's runs: the tool's schema and decoder, the answer's words, the route, and the stock instructions, so that a
chat changes its document by lines and builds a long one in parts.

Acceptance:

- `temper-engine-smith`: `note`'s `Spec` and `notes.rs` by `notes/contracts.md`, *The tool*; `calls.rs` decodes the range form to
  `engine::Tool::NoteLines`. The `Spec`, in these words: its description `Write a note in a scope for later runs to recall, or revise one,
  whole or by a line range, naming the revision you read.`; `body`, `The note's text, or the text that replaces the range.`; `recalled`, `To
  revise a note: the revision you read, from your brief, recall or your last write's answer.`; `first`, `To change lines, with name, recalled
  and remove: the 1-based first line replaced; the note's lines + 1 to append.`, integer, minimum 1; `remove`, `With first: how many lines go;
  0 to insert.`, integer, minimum 0. `required` stays as it is.
- `answers.rs`' `note_written`: a later write closes with ` · L lines`, as the contract's *The words*. `note-range` wrote the refusals' words:
  pin them here.
- `temper-engine-domain`: `Tool::NoteLines` as the contract has it, `route/tools.rs` to `core::NamedAction::NoteLines`, each of
  `translate.rs`' exhaustive lists beside `Tool::Note`. `checked_call` refuses a rejected `Note` or `NoteLines` with `notes::new_refusal`.
- **`chat.txt`'s and `configuring.txt`'s document rule,** from `Write the whole Markdown body each time:` up to and including `If a write is too
  large, shorten it.`, keeping their order so that the sentences on what to say and on Mermaid close it, is these words, in both:

  > Write its first version whole, in Markdown: # TITLE, then the muted line _Draft, DATE, with the user._, one ## section per topic, and open
  > questions under ## Open questions. Then change it by lines: revise the same note, naming the note and the revision you read from your
  > brief, recall or your last write's answer, with first the 1-based line to change from, remove how many lines go, 0 to insert, and body
  > the text that takes their place. To append, first is the note's lines + 1 and remove 0; each write's answer ends with the note's lines.
  > Never rewrite it whole past a few KB, and build a long document in parts: a first version of a few KB, then appends of a few KB each. A
  > refused note says the revision or the line count to use: call again with it. Say in one sentence what changed; do not repeat the document
  > in your reply. Mermaid is not drawn, so do not draw diagrams.

- `temper/docs/help/reference/host-tools.md` is what `the_page_host_tools_is_what_the_declarations_render` (`temper/crates/temper/src/tests/commands.rs`) prints it should be.
- `temper/docs/design/domain/agent.status.md`, 8.3's entry becomes `8.3: a document built in parts is proved on no script.`

## Inputs

- `jig/docs/plan/note-parts/notes/common.md`, `jig/docs/plan/note-parts/notes/contracts.md`: *The tool*, *The words*, *temper's engine*.
- `temper/docs/design/domain/agent.md`, 8.3, *A chat's document*; `temper/docs/design/protocol/configuration.md`, 13.8, *One call's payload*.
- jig's `jig/docs/design/domain/engine.md`, section 10.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper/tests/fake_backends/walk/chat_document.rs`: `SECOND` takes its closing ` · 36 lines`.
- `temper/tests/engine/domain/tests/suite/**`, `temper/crates/temper-engine-domain/src/**` tests, and any `Tool` match the compiler shows, each only for the new arm.

Options, each pre-approved:

- **A stock text's size:** if a test bounds a charter's instructions and the longer rule passes it, first cut your own words, keeping the phrases
  `assert_document_rule` holds; if a derived bound must still rise, raise it and say so. A budget or a timeout is not one of these.
- **Real-loop's scripts** (`temper/tests/real-loop/src/case/`) read first writes' words, `· 1 lines`, which don't change. If one reads a later
  write's, change its literal.
- **The client's `note_write`** (`ui-chat-draft`'s `chat-state`, `temper/crates/temper-client-domain/src/pages/chat/document.rs`) reads a later
  write's words. If it is on your base, it takes the closing ` · L lines` and its test says so; if not, leave it: `plan.md` names it.

## Steps

1. The decoder, its `Spec` and its tests in `temper-engine-smith/src/tests.rs`, beside its `note` sample: `a_line_range_decodes_to_note_lines`,
   `a_line_range_needs_its_revision`, `first_and_remove_come_together`, `first_zero_is_out_of_range`,
   `a_note_without_first_decodes_as_today`.
2. The words, in `answers.rs`' tests: `note_answers_give_later_writes_changes_and_first_line` with the lines; a new
   `note_refusals_say_what_to_do_next`, for `Moved`, `PastEnd` and `PastBound`. `temper-service/src/tests/tools.rs`' literal follows.
3. The tool through the engine, with `root_routes.rs`, on `new_note`'s shape: `a_runs_line_range_writes_the_whole_body`,
   `a_runs_stale_line_range_is_answered_with_the_revision`, `a_body_past_its_bound_is_rejected_saying_so`.
4. The stock words; `temper-service/src/tests/stock.rs`, `assert_document_rule`'s phrases: `Write its first version whole`, `Then change it by
   lines`, `To append, first is the note's lines + 1 and remove 0`, `Never rewrite it whole past a few KB`, `build a long document in parts`, `A
   refused note says the revision or the line count to use`, beside what stays. The phrases that go leave with them.
5. `heavy cargo nextest run -p temper-engine-smith -p temper-engine-domain -p temper-service`; the engine domain's world, `rg -n '^name'
   temper/tests/engine/domain/Cargo.toml` naming it; `heavy cargo nextest run -p temper --lib`, whose `tests/commands.rs` holds `host-tools.md`;
   `heavy cargo nextest run -p temper --test process -E 'test(a_chat_document_is_written_and_revised_on_the_fake_box)'`; `heavy cargo check --tests -p
   temper-real-loop-world`.
6. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report. Say what a later write's words now print, and every reader of them you changed.

## Answers

2026-11-05 (fabrig/ui). Addition (fabrig/qa, finding-330; inputs/request.md): add join: true beside first/remove: the text continues the line before first, with no newline, under the same refusals, since a JSON string can't span lines and a settings value's line can exceed a call. If note-range landed without it, add it to jig's machine, core and tests here too; jig's files are yours for this. Schema words and stock instructions say when to use it.
