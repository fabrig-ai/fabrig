+++
id = "configure-picker"
part = "configure-picker"
kind = "feature"
needs = ["picker-offer"]
files = [
  "temper/crates/temper-client-domain/src/pages/choice.rs",
  "temper/crates/temper-client-domain/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/pages/configure.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/tests/configure.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/tests/client/tests/suite/configuring_chat.rs",
]
effort = "high"
minutes = 90
+++

# The configuring chat gets the picker, and its starter names what is connected

## Goal

"It's not possible to select the model for the configuring agent." Give Configure temper, and a configuring chat once
started, the same chooser as any chat, in today's look. Its starter "Models by role" names only connected accounts,
and goes with a single account.

Acceptance:

1. On `Configure`, the line is template 3's `resolved_default`, with the chooser behind it: Opus 5.5 at high with Claude,
   Astra 6 at high with a ChatGPT plan alone, as `me` gives them. No chat exists yet.
2. A choice made there is the chat's: a starter or typed words send `Create` naming it; with no press, `Create` names none
   (`temper/tests/client/tests/suite/configuring_chat.rs`).
3. On a configuring chat, `choice::byline` as any chat's: its bar's line with the chooser behind it, and a press amends
   its next run. Its path stays through `Settings`.
4. "Models by role": with accounts A and B in the offer, title as now, line "A for reviews, B for the rest.", words "Use A
   for reviews and B for everything else."; with one account, or none, no such starter. The first two accounts met in
   the offer's order, each worded as the accounts page names its kind ("Claude", "my ChatGPT plan", "my OpenAI API key"),
   a number the accounts document doesn't hold by its provider's name (`tests/configure.rs`).

## Inputs

- `temper/docs/plan/model-catalogue-1/notes/common.md`, `temper/docs/plan/model-catalogue-1/notes/contracts.md`
  (*The client*).
- `temper/docs/design/client/README.md`, 3.5, *One layout for every chat* and the configuring layout's line, conversation
  and composer.
- `git show origin/look-requests:temper/docs/design/client/journeys/model-picker/05.html`, for reading only: the look is
  `fabrig/ui`'s, and this task adds no CSS.

## Owns and may change

- Owns its `files`. `chat.rs` and `words.rs` are shared: small hunks (`notes/common.md`).
- May change, each for its compile or words alone: `temper/crates/temper-client-view/src/tests/{chat,task}.rs`;
  `temper-client-domain/src/tests/pages/chats.rs`.
- `Word::ConfigureModelsCardLine` and `Word::ConfigureModels` gain two `Name` holes.
- The browser's `temper/tests/client-browser` world isn't yours; run it only if a test there fails at the trunk's end
  check, as a repair. Journeys' prints are `fabrig/ui`'s catch-up.
- `temper/docs/design/client/README.status.md`: removes what remains of this plan's **Not yet** on the configuring
  layout's line and "Models by role".
- Options, pre-approved:
  - `Configure`'s `pages.chats` is what `refresh` sees, as a new chat's is: tell them apart by `pages.address()`.
  - `configure_chat` takes the `Draft`, as `start` does, and passes `Some(choice)` to `send`.

## Steps

1. Domain: `refresh` sets `Context::Template { template: configuring_template(..), .. }` on `Address::Configure`;
   `configure_chat` carries the draft.
2. View: `configure::byline` calls `crate::choice::byline(b, root, None, w, bindings)`, and `chat.rs`'s bar drops its
   configuring branch. `configure.rs`'s starters: "Models by role" from the offer's accounts.
3. The tests, each rule broken once to see it bite.
4. Narrow tests, then the implementer's text's closing checks over `temper-client-domain`, `temper-client-view` and
   `temper-client-world`. Commit and push.

## Report

As the implementer's text says; each shared file you changed, by name.
