+++
id = "chat-state"
part = "chat-state"
kind = "feature"
needs = ["chat-stock", "person-words"]
files = [
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/pages/chat/document.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat_document.rs",
  "temper/crates/temper-client-domain/src/tests/pages.rs",
  "temper/tests/client/tests/suite/chat_draft.rs",
  "temper/tests/client/tests/suite/main.rs",
]
effort = "high"
minutes = 85
+++

# A chat's document in the domain: its state and its live reading

## Goal

The chat page holds its document and its mode, reads the document live with no new watch, and says what each `note` call did. This task draws nothing.

Acceptance, as `notes/contracts.md`, *The chat page's state*, and temper's `client/README.md`, 3.5, *A chat's document*: its *Read live*, *Its mode*, the tool line's words:

- `pages/chat/document.rs`: `Mode`, `Document`, `NoteChange`, `NoteWrite`, `note_write`, `slug`, `Section`, `sections`, `section_holding`, `drafts_offered`.
- `Chat`'s new fields; `Chat::new` takes the opening mode, today always `Mode::Chat`, and its handles move: `document`, `activity`, `draft_read`, `next = first + 3`.
- **Live reading:** `Read::Notes { scope: NoteScope::Task { project, task }, description: None, after: None }` by a second `Asking`, on `activity.rs`' retries:
  - once the task's document has arrived, at `told`'s two arms that see it first (`Told::Changed` at the document handle, `Told::Page` for the final task), `project` the task's;
  - again when the thread shows a `note` call whose number is past `notes_seen`, and when `run_changed` sees the run's call move off `note`;
  - one read out at a time; `wanted` asks once more on its answer;
  - the answer's lowest-numbered `Note` of that scope replaces `draft`, its `changed` its `changed`, its description `title`, its `body.text` `body`; an empty page, or a refused or lost read, leaves `draft` as it was. A transient end retries as `activity_page` does.
  `fire`, `next_deadline` and `leave` hold both reads.
- `Chat::newest_write`. `note_write` reads the words `chat-stock` gives, with `written` required so that `NoteRecalled`'s `Note N revision R:` never parses, and a refusal's neither.
- `Act::Mode { mode }` sets `mode`; `Act::ShowSection { line }` moves to `Split` unless in `Split` or `Draft`, sets `shown`, moves `shown_count`. Both join `chat.rs`' and `pages.rs`' exhaustive lists.
- **A world proves words and read together,** `temper/tests/client/tests/suite/chat_draft.rs`: a front replying with a `note` call (`CallAnswer::NoteWritten`, rendered by `temper-engine-smith`'s `answers`, so that a change of those words breaks this test), then `a_note_call_reads_the_document`, `a_second_note_call_replaces_it`, `a_refused_read_leaves_it`, `a_chat_opened_again_starts_in_chat`.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The client*, 1, 2 and 3.
- `temper/docs/design/client/README.md`, 3.5.
- `temper/crates/temper-client-domain/src/pages/activity.rs`, `Asking`; `pages/delegation.rs`, `results`, the precedent for parsing words; `pages/chat.rs`, `delegations`, `run_changed`, `told`.
- The mock's `slug`, `temper/docs/design/client/greenfield/app.js`, line 100.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-domain/src/lib.rs`, a re-export; `src/tests.rs`, a `mod` line.
- `temper/tools/gallery/src/worlds.rs`, its `c::Read::Notes` arm answering `None` as today, compiled; `temper/tests/client/src/**` where a front answers `Read::Notes`.
- Each `Chat::new` caller, the opening mode.

Options, each pre-approved:

- **`slug` over non-ASCII:** the mock's rule is ASCII `[a-z0-9]`, so other bytes become `-`, and a `slice(0, 28)` on bytes then is on `-` or ASCII, never inside a character. Keep it.
- **A heading in a fence:** `sections` ignores a `#` line inside a ``` fence.
- **A body with no heading:** one section, line 1, an empty heading.
- **`me` absent:** `drafts_offered` isn't called: a view's gate is `pages.me(client)`'s.
- **A note read by a party with no role:** read as refused, `draft` stays `None`.

## Steps

1. `document.rs`, pure, with tests in `tests/pages/chat_document.rs`: `a_first_write_parses`, `a_later_write_parses`, `a_recall_never_parses`, `a_refusal_never_parses`, `slug_is_the_mocks`, `slug_cuts_at_28`, `an_empty_title_slugs_untitled`, `sections_split_at_each_heading`, `a_fence_hides_a_heading`, `section_holding_finds_a_line`, `a_line_past_the_end_is_the_last_section`, `drafts_offered_by_a_tracked_template`, `not_by_a_conversation`.
2. The read and its triggers, tests in `tests/pages/chat.rs`: `reads_once_the_task_arrives`, `a_new_note_call_reads_again`, `one_read_out_wanted_once_more`, `a_transient_end_retries_on_activitys_waits`, `off_a_call_to_note_reads_again`, `an_empty_answer_leaves_the_document`, `the_lowest_numbered_is_the_document`.
3. Each test bites: remove `wanted`'s second ask, remove the " written" requirement, and see their tests fail.
4. `heavy cargo nextest run -p temper-client-domain`; the client world's `chat_draft` and `chat` suites (`temper/tests/client/Cargo.toml`'s name); `heavy cargo check --tests -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal`.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
