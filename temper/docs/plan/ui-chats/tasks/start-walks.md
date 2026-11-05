+++
id = "start-walks"
part = "start-walks"
kind = "migration"
needs = ["new-chat-page"]
files = [
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/**",
  "temper/tests/client/**",
  "temper/tests/client-browser/**",
  "temper/tests/one-process/**",
  "temper/tests/real-loop/**",
  "temper/crates/temper/tests/**",
  "temper/tools/stage-one/**",
  "temper/tools/gallery/src/tests.rs",
]
effort = "medium"
minutes = 70
+++

# Starting a chat walks through NewChat

## Goal

The composer leaves the Chats page, as the request's section 1 says, and everything that started a chat from it
starts it from `NewChat`. This task is the migration, so that `chats-page` can redraw the page freely.

Acceptance:

- `pages::chats::main` draws no text area, hint, "Start" or `crate::choice::composer`. The playground's start and
  suggestions stay on `Chats` until `new-chat-playground`: while the playground's start is drawn, its suggestions still
  work.
- Every walker that typed in "Start a chat" and pressed "Start chat" goes to `NewChat` first (by "+ New chat", or
  `/chats/new`), types in "What do you want done?" and presses "Send" or Enter. Each story keeps what it proves.
- `Word::StartChat`, `Word::DescribeChange`, `Word::ChatHint` and `Word::Start` go if nothing else reads them.
  Leave `ConnectModel`'s own "Start a chat" (`accounts/words.rs`) as it is.

## Inputs

- `temper/docs/plan/ui-chats/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chats/inputs/request.md`, sections 1 and 2.

## Owns and may change

Owns, as `files` lists them, for the walkers' lines alone, but `pages/chats.rs`.

The survey (`rg '"Start a chat"|Describe a change|"Start chat"|"Start"'` and `StartChat`): in `temper/tests/client/**`
`suite/stories.rs`, `sign_in.rs`, `sign_in_paste.rs`, `accounts_edges.rs`, `open_chat.rs`, `connect_model.rs`,
`connect_model_edges.rs`, `choice.rs`, `tests/live_text.rs`; `temper/tests/client-browser/src/lib.rs` (`start_chat`,
`start_playground_chat`) and `tests/browser_all/browser_pages.rs`; `temper/tests/one-process/src/{world,stories,
stories/playground,referee/agent}.rs`; `temper/tests/real-loop/{src/terminal.rs,tests/live_text.rs,
tests/suite/terminal.rs}`; `temper/crates/temper/tests/support/person.rs`; `temper/tools/gallery/src/tests.rs`;
`temper/crates/temper-client-view/src/tests/{chats,choice,welcome,frame,playground,playground_start}.rs`. `rg` again:
the list may have grown.

May also change: the terminal's goldens and the journeys' prints, as `notes/common.md`.

Options, each pre-approved:

- A helper shared by many tests changes once, in the helper. A test whose point was a composer on the Chats page itself
  moves to `tests/new_chat.rs` if `new-chat-page` lacks it, and goes if it does not.
- `welcome-playground`'s recording may go stale here; leave it to `playground-recording`, and say so.
- A browser walker needs no run here beyond the one below.

## Steps

1. `chats.rs`: the composer's block goes. `offers_first_chat` stays for `chats-page`.
2. The walkers: helpers first, then `rg` until nothing is left.
3. **Check,** `notes/common.md`: all its packages, then the worlds you changed:
   `heavy cargo nextest run -p temper-one-process-world`, and `heavy cargo check --tests -p temper-real-loop-world -p
   temper-client-browser-world`, whose slow tiers you don't run. Of the
   browser tier, run `heavy cargo nextest run --profile browser-temper -p temper-client-browser-world -E
   'test(=enter_starts_one_chat_keeping_a_shift_enter_break)'`. Clippy, push.

## Report

As `notes/common.md`, *Your report*, with each slow test you changed and did not run.
