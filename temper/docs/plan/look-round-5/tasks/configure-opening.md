+++
id = "configure-opening"
part = "configure-opening"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/pages/configure.rs",
  "temper/crates/temper-client-view/src/tests/configure.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 130
+++

# "Configure temper": its opening as a page

## Goal

`Configure { project }` is named "Configure temper" and its opening reads as a page, as Home's, with
the ways to begin as cards, in both shells. The conversation and the composer share one centred
column. Every commit carries `Look-request: configure-temper-name`.

Acceptance:

- **Every rule** the request's `overlay.css` diff adds or changes is in `temper.css`, selectors and
  declarations as the overlay has them, under `/* ==== ui: settings ==== */`, with the overlay's
  phone rules in an `@media (max-width: 760px)` there, under a comment naming the request.
- **The rules it replaces are gone from where they stand** (the block headed `configure-chat-in-settings`):
  in the base, `.temper-thread`, `.temper-starter`, `.temper-starter > p`, `.temper-starters`,
  `.temper-starters button` and its `:hover`, `.temper-session-foot`, `.temper-session-composer`,
  `.temper-session-hint`; in its `@media`, `.temper-thread`, `.temper-starter`,
  `.temper-session-foot`. The rest of the block stands.
- **Turns share the column,** which the message asks in words and the overlay can't draw, since its
  page has no turns: one rule more under the marker, `#app > .temper-session .temper-thread > * {
  max-width: 720px; margin-inline: auto; }`, or what the render shows does it. In the gallery's
  `configure-chat` at 1440 and 1000, the log's left and right edges equal the composer's.
- **The markup** is the request's page's `main`, roles, names, classes and order; the words,
  `temper/docs/design/client/README.md`, 3.5, *The configuring chat's page*. `Word::FirstChat` no
  longer names this page: it stays on the chats page's offer, and Home's `Word::HomeHow` stays.
- **The acts:** each card and the fourth starter is `Act::Starter { words }` with the words the
  design's table says it sends, disabled while `Chats.start` is out. Nothing else about starting
  changes.
- **The terminal** prints the path, the heading, the question, the lede, `What it sets:` with its
  "TERM: LINE" items, the note, `Ways to begin:` once, its three "[n] TITLE: LINE" cards, "[n] Not
  sure yet? Ask what temper can do.", the text area and "Send".
- `temper/docs/design/client/README.status.md`: the line *3.5, The configuring chat's page* goes.

## Inputs

- `temper/docs/plan/look-round-5/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5, *The first chat*, *The configuring chat's page*.
- `temper/docs/design/draft/look-requests.md`, 3.
- The request, as the note says.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/limits.rs`: a count derived from the words.
- `temper/crates/temper-client-view/src/tests/{look.rs,fixtures.rs}`, and any view test that reads a
  word or class this removes. `tests/chat.rs` keeps asserting `look::STARTERS` is off a configuring
  chat.
- `temper/docs/design/client/README.status.md`, that line alone.

Leave: `pages/home.rs`, `pages/chats.rs` (`configure-entry` holds Settings' entry; the offers keep
their words); `temper/tools/gallery/names.txt`, `samples.rs` and `worlds.rs`, since the states stand
and only their words change; `temper/tests/client/tests/suite/configuring_chat.rs`, whose
`Act::Starter` is unchanged.

Options, each pre-approved:

- The project's name in the question: as `top` finds it, `Word::Project` with no name.
- The joiner in the terminal's items and cards: an existing word with ": ", as `Word::HomeAskLine`
  is, if it fits; otherwise a new word.
- The extra rule's selector: if `*` catches something the render shows it mustn't, name the parts
  (the log, the cards, "Older", the ended line) and say so.
- A `<p>` the request draws with no `aria-label`: draw it so, by a helper without a name, unless
  skein-ui's tree needs one; say so if it does.

## Steps

1. **Sketch of the view:** in `pages/configure.rs`'s `main`, inside `look::STARTER`:

   ```rust
   text(b, Role::Heading { level: 2 }, &question, None);          // Word::ConfigureQuestion, Name
   paragraph(b, &lede, look::STARTER_LEDE);
   b.open(Role::List); b.name(&sets); b.style(look::STARTER_SETS); // "What it sets"
   // each item: Browser, two b.text leaves, the term then the line;
   //            Terminal, one leaf, "TERM: LINE".
   b.close();
   paragraph(b, &note, look::STARTER_NOTE);
   b.open(Role::Region); b.name(&ways); b.style(look::STARTER_WAYS);
   // Browser only: text(b, Role::Heading { level: 3 }, &ways, None);
   b.open(Role::List); b.style(look::STARTERS);                    // no name now
   // each card, Role::Item > Role::Button, bound to Act::Starter { words: sent }:
   //   Browser, no name and two b.text leaves, the title then the line;
   //   Terminal, one leaf, "TITLE: LINE".
   b.close();
   // Role::Button, its name its words, look::STARTER_UNSURE, Act::Starter { words: unsure_sent }.
   b.close();
   ```

   New classes: `look::STARTER_LEDE`, `STARTER_SETS`, `STARTER_NOTE`, `STARTER_WAYS`,
   `STARTER_UNSURE`, as `temper-starter-lede`, `-sets`, `-note`, `-ways`, `-unsure`.
   `Word::ConfigureAgent` goes. `ConfigureAlone`, `ConfigureDesign` and `ConfigureUnsure` stay as
   the words sent. `ConfigureWords` becomes "Describe how you work, or what to change". New words
   name the title, the question, the lede, the set's name, terms and lines, the note, each card's
   title and line, the third card's sent words and the fourth starter's words.
2. **The stylesheet,** by the acceptance and `notes/common.md`.
3. **Tests,** `tests/configure.rs`, each biting: the page's node order and classes against the
   request's in both shells (`look::THREAD` holds `look::STARTER`; `look::STARTER` holds the heading,
   the lede, the sets, the note and the ways; the ways hold the browser's heading, the list and the
   fourth starter); each of the four starters' sent words, by pressing it in both shells; all four
   disabled while the start is out; the question names the project; "What it sets" and each card as
   two leaves in the browser and one line in the terminal; the terminal's print, in its order;
   `Word::FirstChat`'s words on the chats page and Home's step unchanged, by their existing tests.
4. **Gallery,** `tools/gallery/src/tests.rs`: the four `configure*` states' heading "Configure
   temper", `Part::Named { role: Role::Region, name: "Ways to begin" }` and the text area's new name.
   Render `configure` and `configure-chat` at 1440, 1366, 1000 and 420, and 1366 by 680, with a
   throwaway script: the opening and the composer whole at 1366 by 680, the edges shared.
5. **Check,** by the implementer's text: `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-shell-terminal -p temper-client-gallery -p
   temper-client-journeys`; clippy; the citations; push.

## Report

As `notes/common.md`, Your report: the rules as a list, each state, and the render's edges.
