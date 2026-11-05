+++
id = "configuring-chat"
part = "configuring-chat"
kind = "feature"
needs = ["configure-page"]
files = [
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/chats.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-shell-web/temper.css",
]
effort = "high"
minutes = 110
+++

# A configuring chat, in this layout and tagged

## Goal

A chat from template 3 keeps the page's layout once it is made, and is tagged in `Chats`. Every commit
carries `Look-request: configure-chat-in-settings`.

Acceptance, `temper/docs/design/client/README.md`, 3.5, *A configuring chat*:

- A chat is configuring by `temper_client_domain::pages::chats::configuring`, on its chip's executor.
  With no template 3 in `me`, or a template 1 chat, nothing changes, in any test.
- **`Chat { task }`** of a configuring chat: main `look::MAIN` and `look::SESSION`
  (`styles_per_node` is 2, so `look::CHAT` gives way); `pages/configure.rs`'s `top`, its heading the
  chat's number and title; then its status, spend, Stop and Close, in the top bar at the right; the
  conversation in `section.temper-convo > .temper-thread`, as the chat page draws it, delegates,
  proposals and all; the foot of `pages/configure.rs`'s layout, its text area "Message", "Send", the
  hint, a refusal; no composer's line, no chooser; an ended chat, no composer.
- **The rail and the tab bar:** "Settings" is current, not "Chats", once the task is delivered.
- **Chats:** the row's meta holds "Configuring", a tag beside its neighbours, in both shells, the
  word fixed, not `Template.name`.
- Any rule of `.temper-chat` a conversation needs also reaches `.temper-session`, by `:is(...)`, and
  nothing else moves: a plain chat's page renders as before at 1440 and 420.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5, *A configuring chat*; 5.3.
- `git show --format=%B -s 53ba7876f8`: its states.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/crates/temper-client-view/src/pages/configure.rs`: `top` and the foot, shared.
- `temper/crates/temper-client-view/src/{words.rs,limits.rs,look.rs}`, `src/tests/{fixtures.rs,look.rs,rail.rs}`.
- `temper/tools/gallery/src/{samples.rs,tests.rs,worlds.rs}`, `names.txt`: `configure-chat`,
  `configure-chats-tag`; `temper/docs/design/client/browser.md`, 7.2.
- `temper/docs/design/client/README.status.md`: *3.5, The first chat, The configuring chat's page*
  goes whole.

Options, each pre-approved:

- The chatbar's parts may be one function with two callers, or the top bar may take them as a closure:
  your call. A plain chat's markup doesn't change.
- `Close` on a phone: the top bar wraps as its rule says; say what you saw.

## Steps

1. `frame.rs`'s current entry, `pages/chat.rs`, `pages/chats.rs`.
2. **Tests,** `tests/chat.rs`, `tests/chats.rs`: each line above; a chat from template 1 unchanged.
   Each bites.
3. **Gallery,** render at 1440, 1000 and 420.
4. **Check,** `notes/common.md`: `heavy cargo nextest run -p temper-client-view -p
   temper-client-shell-terminal -p temper-client-gallery -p temper-client-journeys -p
   temper-client-world`, clippy, push.

## Report

As `notes/common.md`, Your report.
