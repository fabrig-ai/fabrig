+++
id = "document-pane"
part = "document-pane"
kind = "feature"
needs = ["chats-base", "skein-keys"]
files = [
  "temper/crates/temper-client-view/src/pages/document.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/session.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/document.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/files/chat.svg",
  "temper/crates/temper-client-shell-web/files/split.svg",
  "temper/crates/temper-client-shell-web/files/doc.svg",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 90
+++

# The document pane, on a configuring chat

## Goal

A configuring chat with a document offers Chat, Split and Draft and shows the document beside its conversation, the rail folding to its strip: `configure-chat-draft`'s pages `02.html` to `04.html`, but for the drawing and Start over, which is the next task. The pane is the one pane: the next tasks open it to every chat.

Acceptance, temper's `client/README.md`, 3.5, *A chat's document*: *The layout*, *The pane*, *The tint*, and `browser.md`, 2.5; as `notes/contracts.md`, *The look* and *In the terminal*. Every commit carries `Look-request: configure-chat-draft`.

- `pages/document.rs`: `seg(b, chat, words, bindings)`, the group named "The draft", `Mode`'s three, each icon, the chosen `aria-pressed`, `SEG_SPLIT` on Split, the next shown mode's button carrying `Shortcut { key: b'\\', Primary }`; `pane(b, root, chat, task, words, ..)`.
- **The session:** `MODE_*` on `.temper-session`, and the pane after `.temper-convo`. In Chat the browser's rule hides the pane, and the view doesn't draw it. A chat with no document has no group and no pane.
- **The pane:** `DOC`, `DOC_INNER`. Its line, `DOC_PATH`: at its left `docs/design/draft/NAME.md`, for now; at its right "edited 1 min ago", `TimeForm::Ago`, in lower case by the look. The sections: `document::sections`, each `Role::Slot`, `DOC_SEC`, `b.key(Key { owner, kind, a: hash of heading, b: nth of its namesakes })`, and `skein_ui::markdown::read(bytes, 1, b)`. The first section's first paragraph after its heading as `DOC_MUTED`, a paragraph of its own. Tables as the overlay's.
- **The tint:** `DOC_CHANGED` on the section `section_holding(body, first)` of `chat.newest_write()`; for a `First` write the first `##` section. Under it `DOC_NOTE`, "Added by T4 3 min ago" for a `First`, "Changed by T4 just now" for a `Later`, `TimeForm::Ago`, the time the document's `changed`.
- **The rail's strip:** in `frame.rs`, on a chat's address, in the browser alone: `RAIL_UNFOLD` after the brand, as *The look* says; its rules, with the opened rail's own, under the marker. `files/menu.svg` is `.temper-rail-unfold`'s background, `url(files/menu.svg)`.
- **Draft and phone:** the composer at the foot on white; the pane's scrollbar gutter on both edges; on a phone Chat and Draft alone, Split hidden, the session showing as Draft while `mode` stays Split.
- **The terminal:** as *In the terminal*: a region named its title after the conversation in Split or Draft. `chat.txt`'s golden changes by no more than a chat with a document gives, none, since no golden has one; add a fixture's print only if one is cheap.
- **Limits:** `markdown::read` asserts its input within `limits.text_bytes`: a section of a 64 KB body must pass. `temper-client-view/src/limits.rs` prices the page: raise its derived figures for a document's nodes and bytes, at their definitions, by `long_text`, so that a 64 KB body of short paragraphs never panics. A test holds it, `a_full_document_fits_the_views_limits`.
- **Gallery:** `configure-chat-document`, and `configure-chat-document-split` as far as this task draws it: a `Then` setting `Mode`, by `Act::Mode`; `worlds.rs` answers `c::Read::Notes` with a note for that chat; a bar's rows as `samples.rs`' states' `HOLDS` and `SPENDS` need.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chat-draft/inputs/request.md`: *The client*, 4 and 5, for what it says of the configuring chat.
- `temper/docs/design/draft/look-requests.md`, sections 3 to 5: this is steps 3 to 5 of its section 3 for this much of the request.
- `git show 727f3523ee`, whole: its message, its `overlay.css` and its pages `02.html` to `04.html`, the look to copy, the flow's rules and `figure` left out. The mock's `.seg`, `.session`, `.doc`, `.doc-*` for what the overlay leaves out.
- `temper/crates/temper-client-view/src/pages/configure.rs`, `chat.rs`, `session.rs`.

## Owns and may change

Owns, as `files` lists them. May also change `temper/crates/temper-client-view/src/pages/configure.rs`, `tests/chat.rs`, `tests/fixtures*`, `temper/crates/temper-client-shell-terminal/src/tests/**`, regenerated, and `temper/docs/design/client/journeys/**` prints your pages change.

Options, each pre-approved:

- **`.temper-turns`, `.temper-turn-you`, `-agent`, `-tool`:** the overlay's turns are `ui-chats`' `conversation-turns` and `tool-lines`: if your base has them (`chats-base`' report), take theirs. If not, leave the overlay's turn rules unlanded, and name each in the report: they're `ui-chats'`.
- **The focus:** `b.focus` is one per tree; this task asks for none.
- **A mode on an ended chat:** a closed chat with a document keeps the group, and has no composer, as the design says.
- **Keys:** `skein-keys` has landed: `\` is a shortcut. If the hint's words depend on a shell, keep them words for now: the next task adds them.
- **`MAIN`:** if `.temper-session`'s main can't drop `MAIN` by moving its rules, put `MODE_*` on a `Role::Slot` directly inside it, and write the rules on that; say so.
- **Below 1,000 pixels:** the overlay's rules as they are; a difference you must make, in the report.

## Steps

1. Tokens and rules, then `pages/document.rs`, then the session's mode, then the strip.
2. **Tests,** `tests/document.rs`: `no_document_no_group_no_pane`, `the_group_presses_its_mode`, `chat_draws_no_pane`, `split_draws_the_pane_after_the_conversation`, `sections_are_keyed_by_heading`, `a_change_keeps_the_other_sections_nodes`, `the_tint_follows_the_newest_write`, `a_first_write_tints_the_first_second_level_section`, `the_terminal_prints_the_document_after_the_conversation`, `a_full_document_fits_the_views_limits`.
3. Render beside `02.html`, `03.html` and `04.html` at 1440, 1000 and 420, as `notes/common.md` says.
4. `heavy cargo nextest run` of the client's packages but `temper-client-world`; reprint.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report: each overlay rule landed, or left with why.

## Answers

2026-11-05 (fabrig/ui). Ruling: a chat's document is the note with the lowest number in its own scope whose description doesn't begin with "work:". A note whose description begins "work:" is the agent's working note and isn't drawn: a configuring chat will keep its settings value in one.

2026-11-05 (fabrig/ui). Decided: (1) Yes, add a small ordinary Role::Group to skein-ui and its mappings (web: role=group; the terminal prints as a slot does), repairing its exhaustive consumers: hearth doesn't use skein-ui. (2) No viewport wiring: the shortcut keeps the three-step cycle; a phone has no such key in practice, and Split shown as Draft on one is accepted. Say so in the report.
