+++
id = "domain-home"
part = "domain-home"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-domain/src/pages.rs", "temper/crates/temper-client-domain/src/pages/choice.rs", "temper/crates/temper-client-domain/src/pages/terminal.rs", "temper/crates/temper-client-domain/src/tests/pages/chats.rs", "temper/crates/temper-client-domain/src/tests/pages/choice.rs", "temper/crates/temper-client-domain/src/tests/pages/terminal.rs", "temper/docs/design/client/README.status.md"]
effort = "medium"
minutes = 100
+++

# tui-launchpad: Home's composer asks Ask temper

## Goal

On `Address::Home { project }` the client root's composer, which no browser draws and the
TUI's launchpad does, asks Ask temper as `Ask { project }`'s does: its choice refreshed
against template 5, `Act::StartChat` starting a new Ask temper chat through
`Chats::ask_chat`, and, where the look is the terminal's, a send's command words read as
the terminal's. The browser's Home sends neither `ChatWords` nor `StartChat` and draws no
picker, so nothing the browser does changes.

Acceptance, as `temper/docs/design/client/README.md`, 3.5, *Home*, *In the TUI* says and
`temper/docs/plan/tui-launchpad/notes/contracts.md`, section 5 spells:

- `pages.rs`'s `start_chat`: on `Home { project }`, `chats.ask_chat(env, me,
  Some(project), chats.words.clone(), &pages.choice, &mut pages.serial, out)`, as on `Ask`;
  its answer opens the chat with `Request::Show { address: Chat { task } }` (no replace),
  as from `Ask`.
- `pages/choice.rs`'s refresh: with no task and the chats' page open, `Home { .. }` refreshes
  against `chats::ASKING_TEMPLATE`, as `Ask { .. }` does.
- `pages/terminal.rs`'s `terminal_send`: where `look.terminal`, a send on `Home { .. }`
  reads command words (a named command runs, other "/" words note `Unknown`), as on the
  pages `draws` names. `draws` keeps its answers.
- Rows, each a failing test first:
  - `src/tests/pages/chats.rs`, `configure_and_new_chat_start_their_template_and_open_the_chat`'s
    table: a row for `Home { project }` with `configuring_me()`: `ChatWords` then
    `StartChat` sends one create of template 5 with the words, and its answer shows the
    chat; and in `starters_outside_configure_do_nothing` (or beside it) that `Starter` on
    Home still does nothing.
  - `src/tests/pages/choice.rs`: on Home with `configuring_me()`, the choice's context is
    template 5's, its closed choice template 5's `resolved_default`; on `NewChat` it stays
    template 1's.
  - `src/tests/pages/terminal.rs`, `a_command_runs_the_pages_own_act`'s table: on Home with
    the terminal look, `/help` sent whole notes `Help` and sends nothing; `/nope` notes
    `Unknown`; words with a space are sent as words (a create); with the look not the
    terminal's, `/help` on Home is sent as words.
- `temper/docs/design/client/README.status.md`: delete the **Not yet** entry "3.5,
  *Home*, *In the TUI*: Home's composer asks no one: ..." (its bullet's lines alone, not
  the blank lines around it), a stated departure from the implementer's text.

## Inputs

- `temper/docs/design/client/README.md`: 3.5, *Home* (*In the TUI*), *Ask temper's page*
  (its composer and start), *A chat's terminal* (*Its look, kept per browser*,
  *Commands*).
- `temper/docs/plan/tui-launchpad/notes/common.md`;
  `temper/docs/plan/tui-launchpad/notes/contracts.md`, sections 0 and 5.

## Owns and may change

- Owns the files in `files`, the Home hunks of the three source files alone: `ui-ask-drop`,
  `look-round-19` and `ui-account-again` change `temper-client-domain` beside this plan
  (`notes/common.md`); keep each hunk to the lines Home needs, and touch no other file of
  the crate. `pages/home.rs` and `pages/chats.rs` are not yours (another draft's).
- No new public item; `draws` keeps its signature and answers. A private helper in
  `terminal.rs` for "a send here reads command words" is yours.
- Readers: `terminal_send` and `start_chat` are private; `choice`'s refresh is read through
  `Pages::choice()` by `temper-client-view`'s picker, whose tests must pass unchanged
  (the browser's Home draws no picker).
- If `ask_chat` refuses Home's words for a reason `Ask`'s don't meet, stop and say so: the
  start is `Ask`'s, unchanged.

## Steps

1. The rows; `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-client-domain --lib -E
   'test(/pages::chats::/) | test(/pages::choice::/) | test(/pages::terminal::/)'`; see
   them fail.
2. The three hunks; the status entry.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-domain --all-targets -- -D warnings`; `TEMPER_BUNDLE=none heavy cargo
   nextest run -p temper-client-domain -p temper-client-view -p temper-client-tui`;
   `TEMPER_BUNDLE=none heavy cargo check -p temper-client-shell-web -p
   temper-client-shell-tui -p temper-client-world --tests`.
4. Commit with the trailer `Look-request: tui-launchpad`, push.

## Report

As the implementer's text says; each row's failure seen.
