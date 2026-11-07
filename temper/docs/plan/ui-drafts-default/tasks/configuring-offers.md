+++
id = "configuring-offers"
part = "configuring-offer"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/chat/sheet.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/tests/document.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
]
effort = "high"
minutes = 30
+++

# A configuring chat offers "+ Draft" whatever the project's `drafts` says

## Goal

The configuring chat offers "+ Draft" whatever kind 7's `drafts` says, with its other conditions as they are: no document
yet, not writing, no result, not ended. Two gates need it, the bar's and the one in the domain that drops the act. Another
chat keeps following `drafts`.

Acceptance:

- With `drafts` off, a configuring chat with no document draws "+ Draft", and pressing it starts the document (`chat.writing`,
  Split, the words sent).
- With `drafts` off, another chat draws none and the act is dropped, as now.
- A configuring chat with a document, ended, or with a result draws none.
- "Start goal…" stays off a configuring chat (`pages/chat_sheet.rs`, unchanged), and `Configure`, the page before its first
  message, draws no "+ Draft" (`pages/configure.rs`, unchanged).
- The tests below pass and bite; `temper-client-domain`, `temper-client-view` and `temper-client-world` focused suites are
  green.

## Inputs

- `temper/docs/plan/ui-drafts-default/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5, *A chat's document*: *Offered,* and the bar's line "With no document".

## Owns and may change

Owns, as `files` lists them.

May also change:

- `temper/docs/design/client/README.status.md`: remove the **Not yet** entry naming `pages::chat_offers_draft` (the note's
  departure).
- `temper/tools/gallery/src/tests.rs`: a `Holds` row of a configuring chat's sample that names "+ Draft" as absent, if there
  is one; none is known. A gallery sample's words change by themselves.

Options, each pre-approved:

- If `pages.rs` needs a path to `c::Task` other than the one below, use what it already imports.
- If the table below would be cleaner with a helper beside `plus`, add it; if `every_chat_omits_the_foot_when_ended_or_message_is_not_offered`
  is the wrong home for the ended rows, put them in the table in `document.rs` with a new `ProposalState` variant, and say so.

## Steps

1. In `temper/crates/temper-client-domain/src/pages.rs`, beside `drafts_offered`:

   ```rust
   /// Whether a chat offers "+ Draft": a configuring chat whatever its project's settings
   /// say, another while they offer drafts (client/README.md, 3.5).
   #[must_use]
   pub fn chat_offers_draft(root: &crate::Domain, task: &jig_client::contract::Task) -> bool {
       let configuring = match root.pages.me(&root.client) {
           Some(me) => chats::configuring(me, task.project, &task.chip.executor),
           None => false,
       };
       configuring || drafts_offered(root, task.project)
   }
   ```

2. `temper/crates/temper-client-domain/src/pages/chat/sheet.rs`, `step`: the `Act::AddDraft` gate reads
   `!pages::chat_offers_draft(root, task)` in place of `!pages::drafts_offered(root, task.project)`.

3. `temper/crates/temper-client-view/src/pages/chat.rs`, `main`: the bar's condition reads
   `temper_client_domain::pages::chat_offers_draft(root, task)` in place of `drafts_offered(root, task.project)`.

4. Tests. In `temper/crates/temper-client-view/src/tests/document.rs`, make `plus_draft_shows_when_the_setting_offers_it` a
   table over `(body, state, offered)`, each row in both shells, keeping its checks of the style and of the press's act where
   `offered`:

   ```rust
   for (body, state, offered) in [
       (None, ProposalState::DraftOffered, true),  // a chat, drafts on
       (None, ProposalState::Ordinary, false),     // a chat, drafts off
       (None, ProposalState::NotProposed, true),   // a configuring chat, drafts off
       (Some(BODY), ProposalState::NotProposed, false), // a configuring chat with its document
   ] {
       let mut root = document_root(body, Mode::Chat, &[], state);
       // plus(view(&root, shell).tree()) == offered
       // f::root_event(&mut root, Act::AddDraft): root.pages().chat().unwrap().writing == offered
   }
   ```

   `f::root_event` goes through the domain's gate, so that row holds `sheet.rs` as `plus` holds `chat.rs`. In
   `temper/crates/temper-client-view/src/tests/chat.rs`, in `every_chat_omits_the_foot_when_ended_or_message_is_not_offered`,
   on the `configuring` rows, also check that a button named "+ Draft" is found exactly when `!ended && !result`.

   Break each gate once by hand to see the row fail (`sheet.rs` and `chat.rs` apart), and restore.

5. `heavy cargo nextest run -p temper-client-view --lib -E 'test(plus_draft)'`, then the note's and the implementer's text's
   closing checks, naming `temper-client-domain`, `temper-client-view` and `temper-client-world`. Commit and push.

## Report

As the implementer's text has it, naming the rules broken and the failures seen, and any gallery sample whose words changed.
