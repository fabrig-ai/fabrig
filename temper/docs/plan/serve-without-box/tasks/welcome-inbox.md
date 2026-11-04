+++
id = "welcome-inbox"
part = "pages"
kind = "feature"
needs = ["client-reads-deployment"]
files = ["temper/crates/temper-client-domain/src/domain.rs", "temper/crates/temper-client-domain/src/tests/pages/welcome.rs", "temper/crates/temper-client-view/src/frame.rs", "temper/crates/temper-client-view/src/pages/welcome.rs", "temper/crates/temper-client-view/src/tests/welcome.rs", "temper/tests/one-process/tests/suite/one_process.rs", "temper/tests/one-process/src/**", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 60
+++

# With no welcome, signing in opens the inbox

## Goal

On a deployment with no welcome, `Welcome` at any step moves on to the inbox, its address replaced,
with nothing of the welcome drawn first: the printed link's sign-in, a later one, `/welcome` typed
and each `/welcome?at=`. This is the plan's meeting point: one person, the real service, a
document with no welcome.

Acceptance:

- `temper-client-domain/src/domain.rs`, `welcome`: with `root.forge.deployment()` `Some` and
  `welcome` false, on any `Address::Welcome { .. }`, `Request::Show { address: Address::Inbox {
  filter: crate::address::Filter::ALL }, replace: true }`. The authorization gate at its top stays
  first; the store's state is not judged.
- `FirstRun` gains no variant.
- `frame.rs:301-308`, `pages/welcome.rs:122`: on `Welcome`, with the fact anything but
  `Some(welcome: true)`, the frame draws temper's name alone, and the main nothing: no box's line,
  no steps, no heading.
- Domain tests in `tests/pages/welcome.rs`: `no_welcome_moves_each_step_on_to_the_inbox` (Welcome,
  Forge, Model, Ready), `no_welcome_replaces_the_address`, `a_welcome_waits_for_the_read`,
  `no_welcome_offers_no_playground`.
- View tests in `tests/welcome.rs`: `before_the_read_the_frame_draws_temper_alone`, and that an
  answer of `welcome: true` draws what it draws today.
- One story in `temper-one-process-world`, beside
  `a_person_signs_in_by_the_printed_link_and_a_chat_is_answered`:
  `a_person_on_a_deployment_with_no_welcome_finds_the_inbox`, on `documents::world_document`'s
  document, the real service answering; the person goes to `Address::Welcome { step: Welcome }`
  and sees the inbox, and no word of the welcome's was ever drawn.
- `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p
  temper-client-shell-terminal -p temper-client-world -p temper-one-process-world -p
  temper-playground-world` pass.
- `README.status.md` loses "3.4, 3.5: `Welcome` reads no fact of the document: it shows wherever a
  person lands on it."

## Inputs

- `temper/docs/plan/serve-without-box/notes/common.md`; `notes/contracts.md`, K5.
- `temper/docs/design/client/README.md`, 3.4; 3.5 (The welcome, Whether it shows); 5.1.
- `temper/docs/design/client/browser.md`, 2.4 (The printed link).

## Owns and may change

- The files of the header.
- `temper-client-shell-web`'s `home: b"/welcome"` (`lib.rs:102`, `playground/web.rs:164`) stays:
  the guard sends it on (the draft, 4).
- **A one-process or browser story on a document with no welcome that wanted the welcome:**
  `temper-playground/src/documents.rs`'s `world_document` writes no member, so such a story now
  finds the inbox. If one needs the welcome, it gains `"welcome": true` through its `edit`, one
  line, listed in the report. Default: leave `world_document` bare.
- `temper/crates/temper/tests/support/**` run under `temper box` or on the box's document: no
  edit is expected. A process test on a document of its own that needed the welcome: the same
  one line.
- `forge::listed::setup` (`listed.rs:83`) and `pages.rs:1095` read `decision` and need no new arm.
- The chats page is `no-project-page`'s, the Forge choices `forge-choices`'.

## Steps

1. The guard, ahead of `*root.pages.address() != ..`:

   ```rust
   if let crate::Address::Welcome { .. } = root.pages.address()
       && let Some(deployment) = root.forge.deployment()
       && !deployment.welcome
   {
       out.push(Request::Show {
           address: crate::Address::Inbox { filter: crate::address::Filter::ALL },
           replace: true,
       });
       return;
   }
   ```

2. The view's two places.
3. The tests; the story; the implementer's checks; commit and push.

## Report

As the implementer's text asks; name any story given the welcome.
