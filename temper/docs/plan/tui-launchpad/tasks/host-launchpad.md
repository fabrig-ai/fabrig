+++
id = "host-launchpad"
part = "host-launchpad"
kind = "feature"
needs = ["tui-surface", "domain-home"]
files = ["temper/crates/temper-client-shell-tui/src/host.rs", "temper/crates/temper-client-shell-tui/src/tests.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 210
+++

# tui-launchpad: the host goes to the launchpad, follows the root and acts on the TUI's events

## Goal

The TUI's host (`temper-client-shell-tui`'s `host::Host`) gives the client root the
terminal's look at start, moves it to the launchpad, `Home { project }`, once `me` names a
project, follows every page the root asks for as the browser's host does (a chat started,
a chat chosen), draws in the theme the root's look names, and turns the TUI's new events
into the root's acts: the box's words, a send that asks Ask temper or messages a chat, the
commands (`/exit` closing, `/new` to the launchpad), a list's choice, the chat's question
set aside or answered in words, and Esc. Its seam keeps its shape.

Acceptance, as `temper/docs/design/client/README.md`, 6.13 (*A key*, *The frame*, *Its
pages*, *The look*, *The TUI's events*, *The seam keeps its shape*) and 6.16 (*Closing*)
say, and `temper/docs/plan/tui-launchpad/notes/contracts.md`, section 4 spells:

- `Start`, `Report`, `Duties`, `Settings` and `run.rs` unchanged.
- **The look:** right after `Start`'s `Event::Start`, `Event::LookKept { look: Look {
  terminal: true, theme, font: Font::DejaVu } }`, `theme` by `Start.scheme`; after each
  report the theme drawn is `tui::theme` of the root's `pages().look().theme`.
- **Its pages:** after each report's routing, once: the root at its start's address, the
  link live and `me` naming a project gives `Went(Home { project })` of `me`'s first; a
  `Duty::Show { address, .. }` (in `route`, today dropped) is followed in the same report
  by `Went` to it, `Home` or `Chat` as they are, any other address to `Home` of its
  project or of the root's; `Command { command: New }` gives `Went(Home)` of the root's
  project. Each `Went` tells `Told::Went`. `KeepLook`, `Screen` and the other notices stay
  dropped.
- **The events,** 6.13's table, every row: `Typed` signed out as today, else
  `Act::ChatWords` off a chat and `Act::MessageWords` on one; `Send` by the box's words:
  words beginning a command of `tui::offered(root, view.words())` press it as `Command`;
  other words beginning with "/" with no white space give `Act::Terminal(Dismiss)`, tell
  `Told::Unknown { words: Some(..) }` and `Told::Cleared`; on a chat whose
  `shared::question(root, view.words().view())` stands, `needs::Act::OpenReply { object }`,
  `Act::CardWords { object, words }`, `needs::Act::SendReply { object }`, then
  `Act::MessageWords` with no words and `Told::Cleared`; with no project nothing; else
  `Act::StartChat` off a chat, `Act::Send` on one, then `Told::Cleared`. `Command`: `Exit`
  closes (the duty `close`), never reaching the root; `New` as above; any other
  `Act::Terminal(terminal::Act::Run { command })` and `Told::Cleared`. `Act { act }`: the
  act. `Aside { object }`: `Told::Aside { object: Some(object) }`. `Dismiss`:
  `Act::Terminal(terminal::Act::Dismiss)` and `Told::Cleared`. `Told::Unknown { words:
  None }` at the next `Send` or `Went`; `Told::Aside { object: None }` once the root's
  question is none or another.
- **Keys:** Escape the key reader makes nothing of gives `tui::escape(&self.view)`'s event;
  `event_of` says the same. `Told::Size { columns }` at `Start` and each `Resize`.
- Rows, each a failing test first, in `src/tests.rs` (the host's world: reports given by
  the test, the wire's answers encoded as its `reply` helpers encode theirs), two table
  tests: `the_host_goes_to_the_launchpad_and_follows_the_root`: a start then `me` with two
  projects goes to `Home` of the first, once; `me` with none stays at the start's address,
  and goes when a project appears; the look is the terminal's in the start's scheme;
  `/theme` pressed draws the next frame in the other theme; a chat's start answered
  (`Show(Chat)`) is followed in that report; `Show(Ask { project })` goes to `Home`;
  `/new` on a chat goes to `Home`; `/exit` gives `close` and sends nothing;
  `each_event_becomes_its_acts`: `Typed` on the launchpad writes Home's composer words
  and on a chat the chat's; `Send` on the launchpad with words sends one create of
  template 5; `Send` of `/zz` sends nothing and draws "Unknown command /zz. /help lists
  them."; `Send` with no project sends nothing; `Command { Help }` draws the help note;
  Escape on the commands' list closes it and clears the box; `Aside` then `Send` on a
  waiting chat answers its question in words (the reply's exchange sent); a choice's
  `Act` answers it; `Told::Size` reaches the TUI. Prefer rows of these two tables over
  new tests.
- **The journeys:** the root's new reads (the look, Home's watches) make the committed
  `sign-in-approved` stale: leave it under `## Left` for the task `again`. The journeys'
  committed-prints test (`temper-client-journeys`' `the_committed_journeys_are_their_print`)
  is then red until `again` lands, which is expected: don't chase it. The journey
  module's own tests (`journey/tests.rs`, `journey/redact/tests.rs`, `journey/walk/tests.rs`)
  record and replay in one build and must pass; no record kind is added (`tui/journeys.md`,
  3.2 and 3.3).
- `temper/docs/design/client/README.status.md`: delete the **Not yet** entry "6.13, *Its
  pages*, *The look* and *The TUI's events*, and 6.16, *Closing*: ..." (its bullet's lines
  alone), a stated departure from the implementer's text.

## Inputs

- `temper/docs/design/client/README.md`: 3.5, *Home* (*In the TUI*), *A chat's terminal*
  (*Commands*, *A goal's question*); 6.13 (all of it); 6.14, *The box*, *The commands*,
  *The chat's question*; 6.16, *Closing*.
- `temper/docs/design/client/tui/journeys.md`, 3.2 and 3.3.
- `temper/docs/plan/tui-launchpad/notes/common.md`;
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, sections 0, 3, 4 and 5.

## Owns and may change

- Owns `host.rs` and `src/tests.rs` of `temper-client-shell-tui`. The view tasks run
  beside you in `temper-client-tui` and repair their own oracles in this crate's journey
  tests (`journey/tests.rs`'s closing `q`, `journey/walk/tests.rs`'s "Connected to"): keep
  off `journey/**`, `run.rs`, `limits.rs` and `tests/suite/**`.
- Readers of the host: `run.rs`, `temper/tests/client/src/tui.rs` and
  `temper/tools/journeys` pass reports and read duties and frames; nothing of the seam
  changes, so they need no edit. Run the joint world's TUI stories, `TEMPER_BUNDLE=none
  heavy cargo nextest run -p temper-client-world -E 'test(/tui::/)'`: they must stay
  green (they assert words of the screen the view tasks own; if one goes red on yours,
  find why before anything).
- `view.words()` is `temper_client_tui::View::words`; `shared::question` takes the view's
  `temper_client_view::words::Words`, which `tui::words::Words::view()` lends.
- If the root asks a `Show` for an address with no project and no project is known, stay
  where the root is and say so.
- `## Left` for `again`: `sign-in-approved`'s recording, stale by the start's new reads.

## Steps

1. The rows; `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-shell-tui --lib -E
   'test(/tests::/)'`; see them fail.
2. The look and the theme; the pages; the events; Escape and the `Told`s.
3. The status entry.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-shell-tui --all-targets -- -D warnings`; `TEMPER_BUNDLE=none heavy cargo
   nextest run -p temper-client-shell-tui -p temper-client-tui`; the joint world's TUI
   stories as above; `TEMPER_BUNDLE=none heavy cargo check -p temper -p
   temper-client-journeys --tests`.
5. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; each row's failure seen; the requests a fresh start now
sends after sign-in, for `again`.

## Left

- For `again`: `temper/docs/design/client/tui/journeys/sign-in-approved/` recorded again;
  the committed-prints test red until then.
