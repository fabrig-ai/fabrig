+++
id = "terminal-frame"
part = "terminal-frame"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/terminal.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/terminal.rs",
  "temper/crates/temper-client-domain/src/tests/pages.rs",
  "temper/crates/temper-client-view/src/pages/terminal.rs",
  "temper/crates/temper-client-view/src/pages/terminal/**",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/new_chat.rs",
  "temper/crates/temper-client-view/src/pages/configure.rs",
  "temper/crates/temper-client-view/src/pages/session.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/terminal.rs",
  "temper/crates/temper-client-view/src/tests.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/files/fonts/**",
  "temper/tools/gallery/**",
  "temper/docs/design/client/journeys/chat-draft/*.html",
  "temper/docs/design/client/journeys/welcome-playground/*.html",
]
effort = "high"
minutes = 180
+++

# A chat's terminal: its look, ">_", its frame and its type

## Goal

Land the frame of the look requests `chat-terminal` and `chat-terminal-font`: a button ">_" on every chat's bar in a browser turns the chat's pages into the terminal, one top line with "Draft" and "Exit", the conversation in the terminal's look, a running line, the box and a quiet line, in DejaVu Sans Mono at 11pt on a dark ground. Its commands, its full screen and kept record, its draft's look and its cards are later tasks'; this one makes the frame they draw into, to `notes/contracts.md`, 4 and 5.

Acceptance, by the tests below. No browser runs, here or in any step:

1. `pages::terminal` in the domain holds `Look`, `Theme`, `Font`, `Act` and `draws` as contract 4 has them; `Pages::look` starts at the default; `Act::Terminal(Enter)` and `(Leave)` set and unset `terminal`, `(Theme)` and `(Font)` step to the other, and nothing else of the page changes.
2. In a browser, while `look().terminal`, `Chat`, `NewChat` and `Configure` draw the terminal: no rail, no banner and no tab bar; the main styled `TERMINAL`, with `TERMINAL_LIGHT` or `TERMINAL_PLEX` by the look; its top line, conversation, running line, box and quiet line as `temper/docs/design/client/README.md`, 3.5, *A chat's terminal*, *The top line* to *The box*, say. Every other page draws as before, and the terminal shell's prints are untouched.
3. ">_", named "Terminal, full screen", shows in a browser's bar on every chat, `NewChat` and `Configure`, after the mode group or "+ Draft" and before "Start goal…" or "Start over", bound to `Act::Terminal(Enter)`; not in the terminal shell.
4. The top line: "✦", the project's name, "/", the level-1 heading named as the bar's heading is and showing what the browser's bar shows, the chat's number; "Draft" pressed while the document shows, toggling Chat and Split with `Ctrl+\`, or "+ Draft" with no document where the bar offers it, or none; "Exit", bound to `Act::Terminal(Leave)`. None of the bar's other controls draws in the terminal.
5. The running line draws above the box while the browser's look would draw working dots, or the chat waits on its delegates or an account, with the bar's status words and no clock; never otherwise.
6. The box is the page's composer with "❯" and no "Send"; the quiet line under it says "/ for commands", or "paused · a message resumes it" on a chat the person paused, then the picker as the composer draws it today, then the bar's own spend words. A chat that has ended draws neither box nor quiet line here (`terminal-commands` gives it its box for commands).
7. The terminal's rules are in `temper.css` under `/* ==== ui: chats ==== */`, from `chat-terminal-font`'s `terminal.css` as `temper/docs/design/client/browser.md`, 2.5, *A chat's terminal*, says, but for the draft pane's look (`terminal-draft`'s) and the list's (`terminal-commands`'); DejaVu Sans Mono 400 and 700 are in `files/fonts/` with their `@font-face` rules beside the others and their lines in `files/fonts/README.txt`.
8. The gallery holds `chat-terminal` and `chat-terminal-working` (`browser.md`, 7.2), each with its `HOLDS` row; the recorded journeys whose bar gains ">_" are reprinted.

## Inputs

- `temper/docs/plan/look-round-16/notes/common.md`: the standing rules, the requests and how to read them.
- `temper/docs/plan/look-round-16/notes/contracts.md`, 4 and 5: binding.
- `temper/docs/design/client/README.md`, 3.5: *A chat's terminal*, *Its look, kept per browser* to *The box*, and *Where it draws*; *A chat's bar*, *A chat's document* (*Its mode*, *The bar*, *The layout*), *The work's account and model* (the picker).
- `temper/docs/design/client/browser.md`, 2.5, *A chat's terminal*; 7.2, its `chat-terminal-*` rows.
- `temper/docs/design/draft/look-requests.md`, 3: the landing.
- The requests `chat-terminal` and `chat-terminal-font` (`notes/common.md`): their messages, `terminal.css` and pages 01, 02 and 06.

## Owns and may change

Owns the header's files. In `temper.css`, only the new rules and the `@font-face` lines; in `pages/chat.rs`, `new_chat.rs`, `configure.rs`, `session.rs` and `root.rs`, only the branch to the terminal and the call to `to_terminal`.

Reader edits allowed:

- `temper/crates/temper-client-view/src/words.rs`: none by default; the terminal's words go in `pages/terminal/words.rs`, made as `cards/words.rs` makes the cards' (`CardWord`, `say_card`). If the phrasebook's ids or bounds make a second table unworkable, add them to `words.rs`'s `Word` at its end and say so.
- A test of `temper-client-view` that counts or lists a chat bar's controls in the browser (`tests/chat.rs`'s `the_chat_bar_contains_identity_spend_and_acts`, `configuring_chats_keep_the_settings_layout_and_controls_in_both_shells`, `every_chat_bar_orders_its_byline_status_allowed_acts_and_spend_and_links_its_path`; `tests/document.rs`'s `no_document_no_group_on_a_chat`): ">_" among the expected rows, in its place.
- An exhaustive `match` on `pages::Act` elsewhere in temper (`temper-client-view/src/frame.rs`'s decoding, the gallery's driver): the arm for `Act::Terminal`.
- `temper/crates/temper-client-shell-web/tests/suite/*.rs`: none should change; the asset-path check already reads every `url(files/…)`.
- Prints: `chat-draft` and `welcome-playground` reprinted by `temper-client-journeys` with no arguments; no recording goes stale, since the walk presses nothing new.

Options, each decided:

- **The fonts:** copy `fonts/dejavu-sans-mono-400.woff2` and `-700.woff2` from the request `chat-terminal-font`'s commit (`git show COMMIT:temper/docs/design/client/journeys/chat-terminal/fonts/NAME > …`), byte for byte, into `files/fonts/`; `files/fonts/LICENSE.txt` already is DejaVu's. `README.txt` gains the request's `fonts/README.txt` line, naming the two files.
- **The rules:** the request's `.tt` rules, each re-selected onto temper's tokens (`browser.md`, 2.5): `.tt` is `#app > .temper-terminal`, or `.temper-terminal` where the frame's rules need no more weight, and a part the page's own tokens draw (a turn, a fold, the composer's area) is reached under `.temper-terminal` by its token's class. Where the view's markup differs from the page's, keep the declaration and say so in the account row (`terminal-close` writes the account from your report).
- **The speakers' names and the times:** hidden by the terminal's rules; the tree keeps them.
- **The heading's text:** what the browser's bar shows for it (`session::Heading::Chat`, its browser words), computed once in `chat.rs` and passed to both. If `look-round-15`'s `short_title` is on the trunk when you start, the browser's words are its short form; otherwise the title.
- **Working:** `pages::chats::working` as the browser's dots read it, and the waits as `choice::above_composer` draws them; the status words by `pages::chats::status_words`.
- **A phone:** the request's 760 px rules: the project, its "/" and the number go; nothing else differs in the tree.
- **NewChat and Configure:** their own composer, placeholder and starters, under the terminal's top line with "New chat" or the configuring chat's heading; they have no number.

Leave alone:

- `pages/document.rs` and the pane's look (`terminal-draft`'s), the commands and the list (`terminal-commands`'), the shell's host and full screen (`terminal-browser`'s), the cards (`terminal-cards`'), `words.rs` but as above, `temper-client-shell-terminal`, `temper-client-tools`.
- Drafts beside this one change `words.rs`, `session.rs`, `pages/chat.rs`, `frame.rs`, `root.rs`, `look.rs`, `temper.css` and the gallery (`look-round-15/chat-title`, `ui-wait-words/wait-words`, `ui-forge-identity/approval-card`): keep your edits in those files to the lines this task needs, so the merges stay small, and put the rest in the new files.

## Steps

### 1. The domain

`temper-client-domain/src/pages/terminal.rs` as contract 4; `pages.rs` gains `pub mod terminal;`, a `look` field on `Pages`, `Act::Terminal`, its arm in the page's act dispatch and `Pages::look`.

### 2. The view

- `pages/terminal.rs` and `pages/terminal/words.rs` as contract 5: `shown`, `style`, `top`, `running`, `under` (the quiet line alone here) and `notes` (drawing nothing yet).
- `pages/chat.rs`, `main`: where `terminal::shown`, open the main with the terminal's tokens, draw `terminal::top` in the bar's place, the conversation as today, `terminal::running`, the composer with `terminal::under` in the foot row's place and the pane as today; otherwise as today, plus `session::to_terminal` after the mode group or "+ Draft". `new_chat.rs` and `configure.rs` the same.
- `root.rs`: no rail, no banner and no tab bar while `terminal::shown`.
- `look.rs`: the tokens of contract 5, numbered after the last, their names at their indexes.

### 3. The look

`temper.css`, under the chats marker: the request's frame, top line, conversation, running line, box and quiet line rules on the tokens, dark and light grounds, DejaVu Sans Mono and Plex, the 760 px block. The `@font-face` rules beside the others.

### 4. The tests

- **New, domain:** `temper-client-domain/src/tests/pages/terminal.rs`, `the_look_moves_by_its_acts`: a table of `(look before, act, look after)`, every act from the default and from `{ terminal: true, theme: Light, font: Plex }`, and that the address, the mode and the composer's words are unchanged by each.
- **New, view:** `temper-client-view/src/tests/terminal.rs`:
  - `the_terminal_draws_a_chats_pages_in_the_browser_alone`: rows of (page: `Chat`, `NewChat`, `Configure`, `Chats`, `Home`; shell; `terminal` on or off) and what draws: `TERMINAL` on the main or not, the rail or not, ">_" in the bar or not.
  - `the_terminal_says_where_it_is_and_what_runs`: rows of a chat at rest, working, waiting on a delegate, paused, ended, with a document shown and hidden, with none and "+ Draft" offered, with none and not offered: the top line's words and buttons, "Draft"'s pressed state and binding, the running line's words or its absence, the quiet line's hint, and no "Send".
- **Rows of what exists:** the bar tests named under *Reader edits* gain ">_".
- **The gallery:** `samples.rs` gains `chat-terminal` (the `chat-draft-chat` world, ">_" pressed) and `chat-terminal-working` (a chat whose run is live, ">_" pressed); `names.txt` in its sorted place; `tests.rs`'s `HOLDS` a row each naming the top line's heading, "Exit" and the quiet line's hint.

Break each rule once by hand (`draws`' addresses, the rail's absence, the running line's condition, "Draft"'s pressed state, the paused hint), run the test, see it fail for that reason, restore, and name each in the report.

### 5. The checks

One command at a time:

- `heavy cargo nextest run -p temper-client-domain --lib -E 'test(=NAME)'` and `heavy cargo nextest run -p temper-client-view --lib -E 'test(=NAME)'` for each test you added or changed.
- `cargo fmt`, then `python3 -I -B temper/scripts/old_forms.py`.
- `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-gallery --all-targets -- -D warnings`, and `heavy cargo clippy --target wasm32-unknown-unknown -p temper-client-domain -p temper-client-view -p temper-client-shell-web -- -D warnings`.
- `heavy cargo run -p temper-client-journeys` to reprint, then `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-tools -p temper-client-gallery -p temper-client-journeys -p temper-client-world`.
- `python3 -I -B tools/citations/citations.py --base origin/main`.

### 6. Commits

- **Trailers:** the font's commits (the files, `@font-face`, the type's rules) `Look-request: chat-terminal-font`; every other `Look-request: chat-terminal`. One line above the other trailers.
- Push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report, and, for the account `terminal-close` writes:

- each of `terminal.css`'s rules you landed, with its selector there and here, and each you left (the list's, the pane's), and why;
- each page change of pages 01, 02 and 06 landed, or how it differs;
- each rule broken and the failure seen;
- each reader file you edited beyond the header's, and each print regenerated;
- where the terminal's words live, and why if not in their own table.

## Rebase (2026-11-07, step merge, foreman look-round-16/terminal-frame/foreman-1)

Rebase onto the landing trunk, combine phrase and gallery counts, and regenerate journey prints.
The merge conflicts are the branch’s: `words.rs`, gallery `samples.rs`, and `chat-draft/19.html` and `20.html`.
Main’s `a9ef0e974e` adds model-list phrases and four gallery samples; `9781e892a3` refreshes the conflicting prints. These changes reached the landing trunk through `255838cd83`.
In `temper/crates/temper-client-view/src/words.rs`, preserve main’s model vocabulary and the terminal phrase-table extension.
Combine their bounds: `PHRASES = 2013`, `SEGMENTS = 3500`, `ENGLISH_BYTES = 49200`; retain main’s `ALL` length of 1338.
Keep terminal words in `pages/terminal/words.rs`; their 5000-range IDs do not overlap main’s new model IDs.
In `temper/tools/gallery/src/samples.rs`, preserve all model additions and both terminal samples and their setup; combine `N` to 311.
Retain both sets of rows in the automatically merged gallery `names.txt`, `tests.rs` and `worlds.rs`.
Regenerate journey prints from the combined code with `heavy cargo run -p temper-client-journeys`, without arguments; resolve `19.html` and `20.html` from that output.
Report every regenerated print.
Verify `tests::accounts::english_phrase_declarations_match_the_vocabulary`, gallery `names_are_the_promised_ones` and `each_state_holds_what_its_name_says`, and the terminal tests through heavy, then complete the spec’s permitted checks.
