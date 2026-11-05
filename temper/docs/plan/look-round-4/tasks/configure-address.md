+++
id = "configure-address"
part = "configure-address"
kind = "feature"
needs = ["to-begin-card"]
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/chats.rs",
  "temper/crates/temper-client-domain/src/tests/address.rs",
  "temper/crates/temper-client-domain/src/tests/pages/project_pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chats.rs",
  "temper/tests/client/tests/suite/configuring_chat.rs",
]
effort = "high"
minutes = 80
+++

# `Configure { project }`: its address and its start

## Goal

The domain's half of `configure-chat-in-settings`: the page's address and the chat's start from it,
with template 3 or template 1. No page is drawn yet, and no offer leads there: `configure-page` does
both. Every commit carries `Look-request: configure-chat-in-settings`.

Acceptance:

- `Address::Configure { project }` prints and parses `/projects/<project>/settings/configure`
  (`temper/docs/design/client/browser.md`, 2.3); a bad number or a longer path parses as today's
  neighbours' do.
- It opens its project as `Settings` does; switching project keeps the page; `address_project` and
  `conversation_project` answer `Some(project)`.
- `Pages::chats()` is `Some` there, and `Act::ChatWords` and `Act::StartChat` work on it. Any
  page other than `Configure` starts as today.
- `Act::Starter { words }` (`notes/contracts.md`) starts the chat with those words, on `Configure`
  alone; elsewhere it does nothing.
- **Either start:** template 3 while `jig_client::shapes::template(me, project, 3)` is `Some`, else
  template 1, as `first_chat` chooses today; one request, a second press sending nothing while
  `Chats.start` is out; its answer opens `Chat { task }` as today; refused, `Chats.refusal` or
  `outcome` holds it and the words stay.
- **The task's own models:** a start from `Configure` carries no choice, so the task takes its
  template's `llm`: `pages/chats.rs`'s copy of `choice.requested` doesn't reach it.
- `temper/tests/client/tests/suite/configuring_chat.rs`:
  `the_page_starts_the_configuring_chat`: through the real engine, from `Configure`, words typed and
  sent, the task runs charter 3 and the page opens; and the same from a starter.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/contracts.md` (The configuring chat's page).
- `temper/docs/design/client/README.md`, 3.5, *The first chat*, *The configuring chat's page*;
  `browser.md`, 2.3; `../protocol/configuration.md`, 13.4 from `temper/docs/design/client/`.
- `git show --format=%B -s 53ba7876f8`.

## Owns and may change

Owns, as `files` lists them.

May also change, compiler-forced arms only, each `Configure` treated as `Settings` is but where this
spec says otherwise:

- `temper/crates/temper-client-domain/src/{route.rs,departure.rs,acts.rs}`,
  `src/pages/{connect_model.rs,switcher.rs,chat.rs,task.rs}`.
- `temper/crates/temper-client-view/src/{root.rs,choice.rs,frame.rs}`, `src/pages/connect_model.rs`,
  `src/tests/{fixtures.rs,rail.rs}`; `temper/tools/gallery/src/samples.rs`.
- `temper/tests/client/src/fixtures.rs`: what its story needs to list template 3.

Options, each pre-approved:

- `root.rs`'s `if let` chain draws nothing for the address: leave it, `configure-page` adds the arm.
- `drops-connected` may land beside you, removing `switcher.rs`'s block: take its removal.
- `Act::FirstChat` and `FIRST_CHAT_WORDS` stay; `configure-page` removes them.

## Steps

1. `address.rs`, then the compiler's arms.
2. `pages.rs`, `pages/chats.rs`: `Configure` builds `Chats` as `Home` does, one private function
   choosing the template, shared with `first_chat`.
3. **Tests:** the round trip in `tests/address.rs`; `tests/pages/project_pages.rs`; in
   `tests/pages/chats.rs`, `the_configure_page_starts_template_three_or_one`,
   `a_starter_starts_once_with_its_words`, `a_refused_start_keeps_the_words`,
   `a_start_from_configure_keeps_its_templates_models`. Each bites.
4. **Check,** `notes/common.md`: `heavy cargo nextest run -p temper-client-domain -p
   temper-client-view -p temper-client-gallery -p temper-client-world`, clippy, push.

## Report

As `notes/common.md`, Your report.

## Answers

### 2026-11-05

- **May the next run repair the missing `MakeKept`/`MakeRefused` arms in
  `temper/crates/temper-client-domain/src/forge/dismissed.rs`, or wait for their owner?** Neither:
  the repair is on the trunk. `de94b716a1` (a dismissal refuses both outcomes) is on
  `origin/look-round-4/integration`, and this task's branch forked before it. First merge
  `origin/look-round-4/integration` into the branch, which merges clean with `d232fd3e66`, and leave
  `dismissed.rs` as the merge brings it: it stays outside this task's files. Then go on from the
  check that stopped; the acceptance stands. If the build stops on another match missing the same
  making outcomes, you may add its arms as these were added, under this request's trailer, and say
  so in your report; anything else stops the run with a `Question:`.
