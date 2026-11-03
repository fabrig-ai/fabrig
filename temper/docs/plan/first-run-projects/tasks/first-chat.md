+++
id = "first-chat"
part = "first-chat"
kind = "feature"
needs = ["no-repository"]
files = [
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/chats.rs",
  "temper/crates/temper-client-domain/src/acts.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chats.rs",
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/chats.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 60
+++

# The first chat: "Tell temper how you work"

## Goal

On the chats page of a project that is not the playground's and holds no chat yet, the offer
"Tell temper how you work" starts the person's first chat: the configuring agent's, template 3,
while `me` offers it for the project, else the stock chat, template 1, with the person's first
message "Help me tell temper how I work." (`temper/docs/design/client/README.md`, 3.5, "The first
chat"). After the welcome's Ready, which opens the chats page of the project it made, this is
the first chat; it names no repository.

Acceptance:

- A new `pages::Act::FirstChat` in `temper-client-domain`, pressed on the chats page, sends one
  `Create` keyed as the composer's start is, from `jig_client::shapes::chat_from(me, project, 3,
  words)` when it is offered, else `jig_client::shapes::chat(me, project, words)`, its words
  `FIRST_CHAT_WORDS`; the composer's text is left as it was; the chat's page opens once it is
  made, as a suggested request's does.
- The view draws the offer, a button named "Tell temper how you work", above the composer, only
  while the open project is not the playground's and the chats document shows no chat, live or
  ended, under "All"; disabled while the composer's start or the offer's is pending.
- The tests below pass; the status entry below is gone.

## Inputs

- `temper/docs/plan/first-run-projects/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5: "The first chat"; "A suggested request" (the same
  start); "The chats list" (when it says "No chats yet.").
- `temper/docs/design/protocol/configuration.md`, 13.4: template 3 (the configuring agent's), not
  yet among the stock settings (`temper/docs/design/protocol/configuration.status.md`): with
  today's settings the offer starts template 1.
- jig's `jig/crates/jig-client/src/shapes.rs`, `chat` and `chat_from` (read only).

## Owns and may change

- `temper-client-domain/src/pages.rs`: `Act::FirstChat` (no fields), dispatched in `Pages`' act
  handling beside `Act::StartChat`, to a `Chats` method. Every exhaustive match over `Act` gains
  its arm (compiler-forced, `acts.rs` and the pages' matches in `pages/chat.rs`, `pages/task.rs`
  among them: arms only).
- `temper-client-domain/src/pages/chats.rs`:

  ```rust
  /// The person's first message of the first chat (temper/docs/design/client/README.md, 3.5).
  pub const FIRST_CHAT_WORDS: &[u8] = b"Help me tell temper how I work.";
  /// The configuring agent's stock template (temper/docs/design/protocol/configuration.md, 13.4).
  const CONFIGURING_TEMPLATE: u32 = 3;
  ```

  and a `first_chat(...)` method on `Chats` beside `suggest`, sharing `send`'s keying, its pending
  start (`self.start`) and its refusal; `send` takes the template to fill (an argument or an
  enum, your choice) so that the composer and the suggestions keep template 1 exactly as now.
- `temper-client-view/src/pages/chats.rs`: the offer node in `main`, before the composer, bound to
  `Act::FirstChat` through the page's bindings as `Act::Suggest` is bound (line ~493); and
  `temper-client-view/src/words.rs`: `Word::FirstChat`, "Tell temper how you work", in the
  phrasebook's form (both words sets). The chats page's look is the look plan's
  (`notes/common.md`): add the one node and no style beyond the button's existing one; if the
  trunk's chats page already draws a pane where the project's repository shows (the
  `no-repository` task's node), put the offer before the composer and leave the pane as it is.
  If `frame.rs`'s `Bindings::decode` must map the press, edit that arm alone.
- Tests:
  - `temper-client-domain/src/tests/pages/chats.rs`: `the_first_chat_starts_the_stock_chat_while_no_configuring_template_is_offered`
    (a `me` whose project offers template 1 alone: the act sends one `Create` of template 1 with
    `FIRST_CHAT_WORDS`, and a second press while it is pending sends nothing);
    `the_first_chat_starts_the_configuring_template_once_offered` (a `me` whose project offers
    template 3 as a conversation template: the `Create` is template 3's); and
    `the_composer_still_starts_the_stock_chat` if no existing test pins the composer's template.
  - `temper-client-view/src/tests/chats.rs`: `a_project_with_no_chat_offers_the_first_chat`
    (pressing the button gives `Action::Pages(pages::Act::FirstChat)`),
    `a_project_with_a_chat_offers_no_first_chat`, and
    `the_playground_project_offers_no_first_chat`, in the fixtures' forms (`fixtures::root_at`,
    `fixtures::playground_root`).
  - A test that counts the chats page's buttons or nodes and now finds one more: update its count
    alone, and list it.
- Status, as `notes/common.md` says: `temper/docs/design/client/README.status.md`, **Not yet**:
  remove the sentence `3.5, The first chat: the offer "Tell temper how you work" on a chats page
  with no chat; no such act in \`temper-client-domain\`'s \`pages.rs\` and no offer in
  \`temper-client-view\`'s \`pages/chats.rs\`.`

## Steps

1. The domain's act and `first_chat`; `heavy cargo nextest run -p temper-client-domain --lib -E
   'test(chats)'`.
2. The view's offer and word; `heavy cargo nextest run -p temper-client-view --lib -E
   'test(chats)'`.
3. The status edit; the citations check.
4. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-client-domain -p
   temper-client-view --all-targets -- -D warnings`; the focused tests of `temper-client-domain`,
   `temper-client-view` and their direct dependents (`temper-client-shell-web`,
   `temper-client-shell-terminal`, `temper-client-world`); if the terminal's goldens of a chats
   page with no chat now print the offer, regenerate them as `notes/common.md` says. Commit the
   code and tests, then the status text; push.

## Report

As the implementer's text says, and which goldens or counts moved.
