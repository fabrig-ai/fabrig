+++
id = "design-session-web"
part = "drafter-web"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/chats.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chats.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-view/src/tests/document.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 75
+++

# A design session on the web

## Goal

On the web, the new chat with a draft starts a design session, template 6, where
`me` lists it, and the session reads as a chat with its draft in Split
(`temper/docs/design/client/README.md`, 3.5, *A new chat*, *With a draft*, and
*A chat's document*, *A design session*). The client knows a design session as
it knows an Ask temper chat: by the charter of its template in `me`. The rail's
"Goal" item already leads to `NewChat { draft: true }`; its words stay.

Acceptance:

1. **The names,** in `temper/crates/temper-client-domain/src/pages/chats.rs`,
   exactly as `temper/docs/plan/ui-drafter/notes/contracts.md`, *The client*, has
   them: `pub const DRAFTING_TEMPLATE: u32 = 6;` and `pub fn drafting(me: &Me,
   project: u32, executor: &c::ChipExecutor) -> bool`, built as `asking` is over
   `ASKING_TEMPLATE`. The terminal's `/draft` (the TUI owner's milestone) will
   start from them.
2. **The start:** `Chats::start` with `draft` sends `DRAFTING_TEMPLATE` where
   `jig_client::shapes::template(me, project, DRAFTING_TEMPLATE)` is some, and
   `jig_client::shapes::CHAT` otherwise. Its words don't change: the person's, a
   blank line and `WRITE_THIS_UP`, either way (the contract says why). It opens in
   `Mode::Split` either way, as today. A start without `draft` is unchanged.
3. **Opened again:** a design session opened from a list or a link opens in
   Split rather than Chat, until the person picks a mode on the page. Do it where
   `Chat` itself (`pages/chat.rs`) first holds both its task (`chat.task(client)`,
   its chip's executor) and `me`: once, while the mode is still the one it opened
   with and the person hasn't pressed a mode. **If** that would need
   `temper/crates/temper-client-domain/src/pages.rs` (`went`, or the act routing
   about line 1830), which another plan changes now, **then** leave reopening as
   today's, keep the README.status.md sentence "and one opened again starts in
   Chat" (acceptance 6), and say so in the report.
4. **The domain's rows,** in existing tests of
   `temper/crates/temper-client-domain/src/tests/pages/chats.rs`, no new test
   function:
   - a fixture `drafting_me()` beside `asking_me()` (about line 79): the stock chat
     template and a template 6 with `executor` `TemplateExecutor::Charter(12)`, and
     12 among `creating.within.charters`;
   - `ask_and_new_chat_start_their_template_and_open_the_chat` (about line 97): a
     row `(drafting_me(), 12, Address::NewChat { draft: true })`. Its words
     assertion reads per row: `My workflow`, then for the draft row a blank line and
     `Write this up as a draft beside us.`;
   - `the_chat_opens_in_split` (about line 943) becomes a table of two rows,
     `fixtures::chats::me()` and `drafting_me()`, each asserting the task's
     executor, charter 1 and 12, and `Mode::Split` with `writing`;
   - if acceptance 3 is built: a row, in the existing test of
     `tests/pages/chat.rs` that holds a chat's opening mode, for a design session
     opened by a link (Split) beside a chat opened by a link (Chat).
5. **The web drawing the session's draft in Split,** a row of the existing table
   test `split_draws_the_pane_after_the_conversation`
   (`temper/crates/temper-client-view/src/tests/document.rs`, about line 372): the
   drafter's muted line `_Draft, with the user._` on a root whose `me` lists a
   template 6 whose charter is the chat's own executor (built as `asking_me`
   there builds template 5; a `ProposalState` variant or a parameter of
   `document_root`, your choice), asserting what the other rows assert: the pane
   after the conversation, named its title, Split pressed, the muted paragraph's
   runs, and the path `docs/design/draft/...` with "edited just now", not Ask
   temper's line.
6. **The status:** in `temper/docs/design/client/README.status.md`, delete the
   bullet `**Not yet:** 3.5, *A new chat* and *A chat's document*: ...`; or, if
   acceptance 3 was left, keep only its last clause, "a design session opened
   again starts in Chat (`temper-client-domain/src/pages/chats.rs`)"
   (`notes/common.md`, *Status files*).

## Inputs

- `temper/docs/design/client/README.md`, 3.5: *A new chat*, its *With a draft*;
  *A chat's document*, its *A design session* and *Its mode*.
- `temper/docs/design/domain/agent.md`, 8.5, *Where one starts*.
- `temper/docs/plan/ui-drafter/notes/contracts.md`, *The client*.
- `temper/docs/plan/ui-drafter/notes/common.md`.

## Owns and may change

- Owns: `pages/chats.rs`'s new names and `start`; `pages/chat.rs`'s opening mode
  (acceptance 3); the test rows and fixtures named above; the one status bullet.
- Not yours: `pages.rs` (`notes/common.md`, *Not to touch*); the view's
  drawing code, which draws Split by the mode and needs no change (if acceptance
  5's row fails for a reason in the view, that is a question); the rail and its
  words; the stock (`drafter-stock`, in parallel): the tests build their own `me`.
- Readers (this cut's survey): `Chats::start` is reached from `Act::StartChat` on
  `NewChat`; the journeys `chat-draft` and its siblings start a draft on a fake
  box whose stock, once `drafter-stock` lands, lists template 6. The words being
  today's, the fake model's script holds. If a journey replay check in
  `temper-client-journeys` fails on this branch for the start's template, run
  `temper-client-journeys again` for it (`notes/common.md`) and name it.

## Steps

1. Read the inputs, `pages/chats.rs` (`start`, about line 335; `send`; `asking`,
   about line 598) and `pages/chat.rs`'s `Chat::new` and its mode acts.
2. Write the rows (acceptances 4 and 5), then the code; break each rule once
   (`start` sending `CHAT` always; `drafting` false; the reopening) and see its row
   fail for it; restore.
3. Narrow tests: `heavy cargo nextest run -p temper-client-domain --lib -E
   'test(=tests::pages::chats::ask_and_new_chat_start_their_template_and_open_the_chat)
   | test(=tests::pages::chats::the_chat_opens_in_split)'`, then `heavy cargo
   nextest run -p temper-client-view --lib -E
   'test(=tests::document::split_draws_the_pane_after_the_conversation)'`.
4. Focused tests of `-p temper-client-domain`, `-p temper-client-view`, `-p
   temper-client-tui`, `-p temper-client-world` and `-p temper-client-journeys`
   (`heavy cargo nextest run -p ...`, one at a time); `heavy cargo check -p
   temper-client-shell-web --target wasm32-unknown-unknown`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-client-domain -p temper-client-view --all-targets -- -D warnings`.
6. Commit (why, in the message) and push.

## Report

As the implementer's text says, and: whether acceptance 3 was built, and where;
the rows, with each rule broken and the failure seen; any journey regenerated.
