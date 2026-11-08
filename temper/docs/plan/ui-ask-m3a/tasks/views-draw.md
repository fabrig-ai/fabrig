+++
id = "views-draw"
part = "views"
kind = "feature"
needs = ["client-views", "path-draw"]
files = ["temper/crates/temper-client-view/src/pages/chat.rs", "temper/crates/temper-client-view/src/pages/chat/**", "temper/crates/temper-client-view/src/pages/document.rs", "temper/crates/temper-client-view/src/views.rs", "temper/crates/temper-client-view/src/views/**", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/tests/views.rs", "temper/crates/temper-client-view/src/tests.rs", "temper/crates/temper-client-shell-web/temper.css", "temper/tools/gallery/**", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 180
+++

## Goal

Draw a reply's and a chat document's `view` blocks from the chat page's
view reads: a way, a path, a proposal's two paths and a charter's card,
in the browser as `paths`' drawing and in the terminal as its fallback,
never as code (`temper/docs/design/client/README.md`, 5.16).

Acceptance:

- A module `views` in `temper-client-view` draws one view block from
  `Chat::view(address)`: `Asking`, the line "Drawing ADDRESS…", muted;
  `Failed`, "Couldn't draw ADDRESS."; `Drawn`:
  - `Way`: a group named the way's name, its name strong, " · now" where
    `current`, its line in grey, then its path (`paths::path`, its label
    "The way a change goes");
  - `Path`: a group named "The way a change lands on BRANCH in
    OWNER/NAME" (or "on BRANCH" with no repository), the path;
  - `Proposal`: "Before" and its path, "After" and its path;
  - `Charter`: the card: `name` as its heading, `who` under it, then its
    facts as a list of terms, each label and value;
  - `NotDrawn`: its words.
  Each stop's mark is its `Actor` (`Bar` for a bar), its words its
  `words`.
- A committed turn's text and the chat document's text are drawn through
  `temper_client_domain::views::parts`: each `Text` part through skein's
  Markdown reader as today, each `View` part as above, in order. A turn's
  growing text is drawn as today.
- Words in `words.rs`; any style it adds in `look.rs` and its rules in
  `temper.css` under the chats marker.
- Tests (`src/tests/views.rs`): a table, one row a view state (asking,
  failed, each `View` variant, a turn with text, a view and text again),
  each in both shells: the tree's groups, names and drawing kind, and the
  terminal's print, a stop a line ("produce, an agent"); a `view` block
  inside a `text` fence still drawn as code.
- Gallery: samples `chat-view-way`, `chat-view-proposal` and
  `chat-view-charter` (an Ask temper chat whose reply holds the block), in
  `samples()`, sorted in `names.txt`, each with its `HOLDS` row (and a
  `SPENDS` row, as chat samples take), their state from the gallery's
  world answering the `View` read with a built answer; the gallery's
  checks pass.
- `temper/docs/design/client/README.status.md`: the sentence "5.16: a
  `view` block shows as code, the chat page reads no view, and no path is
  drawn." leaves its **Not yet**.

## Inputs

- `temper/docs/plan/ui-ask-m3a/notes/common.md`; `notes/contracts.md`,
  sections 3 and 5 (binding).
- `temper/docs/design/client/README.md`, 5.16; the look request's card
  (`git show c342a9f113:temper/docs/design/client/journeys/ask-temper-ways/04.html`,
  the section `temper-ways-zoom`) as the model of a charter's card.
- `temper-client-view/src/paths.rs` (task `path-draw`);
  `temper-client-domain/src/views.rs` and `Chat::view` (task
  `client-views`); `pages/chat.rs`'s text drawing (`read`, `read_growing`)
  and `pages/document.rs`.
- The gallery: `temper/tools/gallery/src/samples.rs`, `worlds.rs`,
  `tests.rs`, `names.txt`.

## Owns and may change

- **Owns:** `src/views.rs` (new) and its tests; the text drawing of a
  turn and of the chat document; the words and styles it adds; the
  gallery's three samples and their world.
- **Options, pre-approved:** if the browser host test of the shell
  (`temper-client-shell-web/src/host_tests.rs`) compares a chat page's
  drawing names, update it. If a print or golden of a chat page goes
  stale, regenerate it. If a journey recording goes stale, run
  `temper-client-journeys again`.

## Steps

1. `views`; the turn's and the document's text through `parts`.
2. Words, styles, CSS.
3. The table; break a rule once (draw a `View` part as code) to see its
   rows fail, and restore.
4. The gallery's samples.
5. The status sentence.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-client-view -p temper-client-gallery -p
   temper-client-shell-web --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-client-view`, `-p temper-client-gallery`, `-p
   temper-client-shell-web`, `-p temper-client-shell-terminal`. Commit
   and push.

## Report

As the implementer's text asks; name the gallery samples and any
regenerated print or golden.
