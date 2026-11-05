+++
id = "new-chat-address"
part = "new-chat-address"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/domain.rs",
  "temper/crates/temper-client-domain/src/departure.rs",
  "temper/crates/temper-client-domain/src/pages/connect_model.rs",
  "temper/crates/temper-client-domain/src/pages/switcher.rs",
  "temper/crates/temper-client-domain/src/tests/**",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/choice.rs",
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/pages/inbox.rs",
  "temper/crates/temper-client-view/src/pages/connect_model.rs",
  "temper/crates/temper-client-view/src/tests/**",
  "temper/crates/temper-client-shell-terminal/src/**",
  "temper/tools/gallery/src/samples.rs",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 75
+++

# The new chat's address

## Goal

A new chat gets an address of its own, `NewChat` at `/chats/new`, and `Chats`' `start` goes with `?at=start`.
Every "+ New chat" goes there. This task moves the address and its readers: `NewChat` still draws what
`Chats { start: true }` drew, and `new-chat-page` draws it anew.

Acceptance:

- `Address` as `notes/contracts.md`, *Addresses*. `/chats/new` reads and prints; `/chats/new?x=1` is replaced
  with `/chats/new`; `/chats?at=start` is replaced with `/chats`, and `/chats?state=live&at=start` with
  `/chats?state=live`; `/chats/7` is still the chat; `/chats/newer` is no page, as any such path.
- The rail's "+ New chat", the Inbox's, Home's "New chat", To begin's third step and `ConnectModel`'s Ready's
  "Start a chat" go to `NewChat`. Links that say "Chats" stay `Chats`; Welcome's whole state and
  `Act::OpenProject` stay `Chats`.
- The domain's `Chats` state keeps its fields. `NewChat` opens it with the page's watches, so that
  `Act::StartChat`, `Act::ChatWords`, `Act::Suggest` and `ready_to_start` work unchanged. A move between
  `Chats` and `NewChat` keeps the state: the words typed on `NewChat` are there after going to `Chats` and back.
- The rail and tab bar's "Chats" is current on `NewChat`. The switcher treats `NewChat` as it treats `Chats`.
- For now the view draws `pages::chats::main` at `NewChat`, its text area focused as `start: true` focused it,
  and `playground_start` answers at both addresses.
- `browser.status.md`: `2.3` leaves its **Not yet**.

## Inputs

- `temper/docs/plan/ui-chats/notes/common.md`, `notes/contracts.md`.
- `temper/docs/plan/ui-chats/inputs/request.md`, section 2, its first paragraph.
- `temper/docs/design/client/browser.md`, 2.3; `temper/docs/design/client/README.md`, 3.5's table.

## Owns and may change

Owns, as `files` lists them.

The survey: `start: false` and `start: true` appear in about 70 literals. Production readers:
`temper-client-domain/src/{domain.rs:164,pages.rs:516,pages.rs:658,pages.rs:684,pages.rs:764,route.rs:374}`,
`temper-client-view/src/{frame.rs:389,514,615,702,pages/chats.rs:91,pages/home.rs:331,505,619,pages/inbox.rs:135,pages/connect_model.rs:581}`.
Every `match` on `Address` takes the new arm: `departure.rs`, `pages.rs`, `pages/connect_model.rs`, `route.rs`,
`pages/switcher.rs`, the view's `choice.rs`, `frame.rs`, `root.rs`, and `temper-client-shell-terminal/src/client.rs`.

May also change, in any file of temper, the literals alone
(`Address::Chats { filter: ChatsFilter::All, start: false }` to `Address::Chats { filter: ChatsFilter::All }`,
`start: true` to `Address::NewChat`, and `/chats?at=start` in a test to `/chats/new`):
`temper/crates/temper-client-view/src/tests/**`, `temper/crates/temper-client-shell-terminal/src/tests/**`,
`temper/tools/gallery/src/**`, `temper/tools/journeys/src/**`, `temper/tests/client/**`,
`temper/tests/client-browser/**`, `temper/crates/temper-client-shell-web/src/**`.

Options, each pre-approved:

- `address_project(NewChat)`: none, as `Chats`, so that it reads the open project.
- `Act::Project` while on `NewChat`: as on `Chats`.
- A test that pressed "+ New chat" and asserts it stays on the Chats page: it moves with the link.

## Steps

1. The address, its codec and its tests (`src/tests/` of the domain, as `Chats`' are): each path above, both ways.
2. `went` and the arms. The links.
3. The literals, mechanically. Leave nothing that says `at=start`, but in `/chats?at=start`'s own test.
4. **Tests:** the move that keeps the words, in the domain's tests; the rail's and inbox's hrefs, which exist
   (`src/tests/rail.rs`, `src/tests/inbox.rs`), to `/chats/new`; Home's and `ConnectModel`'s acts. Each new one bites.
5. **Reprint** the journeys: the rail's href is in every print (`notes/common.md`, *Generated files*). A recording
   goes stale only if its exchange changes; say if one does.
6. **Check,** `notes/common.md`: all its packages, clippy, push. This changes what `Chats` means to its readers:
   run `temper-client-world` whole.

## Report

As `notes/common.md`, *Your report*.
