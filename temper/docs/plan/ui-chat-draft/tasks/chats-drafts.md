+++
id = "chats-drafts"
part = "chats-drafts"
kind = "feature"
needs = ["goal-sheet", "notes-list"]
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/pages/chats.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/address.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chats.rs",
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/chats.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 75
+++

# The Chats page under Drafts

## Goal

The Chats page says which chats write what, and which goals they started, and can filter to them, with one read as it opens.

Acceptance, temper's `client/README.md`, 3.5, *The Chats page*:

- `ChatsFilter::Drafts`, `?state=drafts`, between Live and Closed, while `drafts_offered`: the rows whose chat has a document, in the page, reading nothing more. A printed address reads back; on a project where it isn't offered, `Drafts` shows as All does.
- **The read:** `Read::Notes { scope: NoteScope::Tasks { project }, description: None, after }` as the page opens, its pages to the bound of the page's rows, by a handle of `Chats`' own, on `Asking`'s retries. `chats.documents` holds, per task, its lowest-numbered note's description. A note written later is seen on the next open: no live read here.
- **A row's top line, left:** "T4", then " · writes away-mode.md" from `document::slug`; then " · started goal TITLE", a link's title from the goal's chip, by one `Subject::Chips { tasks }` watch of the rows' newest `GoalFrom` facts, closed on leave; a configuring chat's " · Configuring" stays, before its "writes". The `::after` of the row's own link stays what covers it; a link to the goal needs its `z-index`, as `chats-page` left that rule.
- **The lede grows:** while `drafts_offered`, "A chat answers a question or changes the code, and can write a draft beside it. A draft becomes a goal for bigger work."
- **Gallery:** `chats-drafts`.
- `browser.status.md`'s entry on `?state=drafts` goes, and `README.status.md`'s first line loses the Chats page's Drafts.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`, *The project's documents*.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The client*, 8.
- The mock's `chatsPage`.

## Owns and may change

Owns, as `files` lists them. May also change `temper/crates/temper-client-view/src/{root,frame}.rs`, `pages/{home,session}.rs` where a `ChatsFilter` is named, in a match alone; `temper/crates/temper-client-shell-terminal/src/tests/**`, regenerated; `temper/docs/design/client/browser.status.md`, `README.status.md`; `temper/docs/design/client/journeys/**`, prints your pages change.

Options, each pre-approved:

- **A read refused or lost:** no row says "writes"; the filter holds none. A transient end retries.
- **The rows of a chat in another's name:** the page is the person's chats, so `Tasks` may hold others' documents; `documents` is read by the rows' own numbers. The others are ignored.
- **Pages past the bound:** "to the bound of its rows": stop at `rows` notes, and a chat past it shows no "writes": the report names it.
- **A link in a line in the terminal:** `skein-ui-text` flattens a link in a paragraph, so a goal's title is a slot, as the rest of the line is.

## Steps

1. The filter, its address and its readers.
2. The read and the facts; tests: `reads_tasks_as_it_opens`, `pages_follow_to_the_rows_bound`, `drafts_keeps_rows_with_a_document`, `the_filter_round_trips`, `drafts_isnt_offered_with_no_tracked_template`, `watches_the_goals_chips_and_closes_them`.
3. The view: `a_row_says_what_it_writes`, `a_row_says_which_goal_it_started`, `the_lede_grows_with_the_offer`, `a_configuring_chat_keeps_its_tag`. Each bites.
4. `heavy cargo nextest run` of the client's packages; goldens; reprint.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.

## Answers

2026-11-05 (fabrig/ui). Limit goal links to one rows window, no jig change: watch the goals of the newest rows' chats across both lists; a row past that shows no "started goal" line and stays otherwise correct. Say so in the report and the design's status, and finish.
