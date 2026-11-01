+++
id = "cl-play-view"
part = "client"
kind = "feature"
needs = ["cl-play-domain"]
files = ["temper/crates/temper-client-view/src/frame.rs", "temper/crates/temper-client-view/src/root.rs", "temper/crates/temper-client-view/src/choice.rs", "temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/pages/**", "temper/crates/temper-client-view/src/accounts/**", "temper/crates/temper-client-view/src/tests/{demo,demo_start,playground,frame,choice,chat,chats,task,change,connect_model,accounts,inbox}.rs", "temper/crates/temper-client-view/src/tests/fixtures.rs", "temper/tests/client/tests/suite/{demo_start,playground,welcome}.rs", "temper/tests/client/tests/qa_all/**"]
effort = "high"
minutes = 110
+++

## Goal

Draw the playground's pages (draft
`temper/docs/design/draft/playground-reset.md`, 6.1 and 6.2) on
`cl-play-domain`'s domain: one banner line on every page but the
welcome, "This is the playground: a scripted model and in-memory forge.
Use the wizard to [set up temper]", the bracketed words a link sending
`EndPlayground`; "Leaving the playground…" while the reset runs; the
setup strip and the demo banner's Connect links gone; Accounts listing
the Scripted model with nothing to add while the playground runs.

Acceptance:

- `frame.rs`: `demo_banner` becomes `playground_banner`: drawn while
  `root.playground()` is some, on every address but `Address::Welcome`
  (Accounts, ConnectModel, ConnectForge and Forges included now), after
  the accounts' banner and before the page and its chatbar, as today. One
  paragraph: the words `Word::PlaygroundBanner` ("This is the playground:
  a scripted model and in-memory forge. Use the wizard to") then a link
  `Word::SetUpTemper` ("set up temper") whose act is
  `pages::Act::LeavePlayground`. While `root.forge().leaving()`, the
  region holds `Word::LeavingPlayground` ("Leaving the playground…")
  alone, no link. The setup strip's drawing, `setup_underway` and the
  `Word::Setup*` words, `DemoBoth`/`DemoModel`/`DemoForge` and the
  banner's `ConnectModel`/`ConnectForge` links go.
- Accounts (`pages/accounts.rs`, `pages/accounts/rows.rs`): while the
  playground runs, the Scripted model's row is listed among the accounts
  (its own "Playground" folded section goes) and the page offers no Add
  or connect link; once it does not run, as today without the demo's
  provider exclusions. Model choices (`pages/connect_model.rs`,
  `choice.rs`) exclude the playground's provider while it runs, and
  nothing once it does not (no retired provider).
- Pages reading the playground's project (chats' start and suggestions,
  chat context, task header, inbox, project buttons) read
  `playground().project`; a chat in that project names it "Playground" as
  today.
- Words: in `words.rs`, the playground's words replace the demo's in the
  place the `Demo*` group holds today (`Word::Demo*` renamed
  `Word::Playground*` where they stay); add new words only in that
  group, so a parallel task adding forge words elsewhere merges cleanly.
- View tests: `src/tests/demo.rs` and `demo_start.rs` become
  `src/tests/playground.rs` with, at least,
  `the_playground_banner_is_one_line_with_its_set_up_temper_link`,
  `the_banner_shows_on_accounts_and_forges_but_not_the_welcome`,
  `set_up_temper_sends_end_playground_and_leaving_replaces_the_line`,
  `accounts_list_the_scripted_model_and_offer_nothing_to_add`,
  `no_setup_strip_or_connect_link_is_drawn`; the frame, choice, chat,
  chats, task, change, connect_model tests on the demo moved to the new
  facts; the halves' cases deleted.
- Client world (`temper/tests/client`): `tests/suite/demo_start.rs`
  becomes `tests/suite/playground.rs`:
  `try_the_playground_starts_it_and_opens_its_project`,
  `set_up_temper_leaves_the_playground_and_opens_the_forge_step`,
  `a_resetting_read_shows_leaving_until_the_reset_answers`; the
  `welcome.rs` suite and the `qa_all` cases on the halves, the strip and
  retirement deleted or moved to these facts. Every test `cl-play-domain`
  listed under `## Left` passes or is deleted with a reason.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md`,
  `temper/docs/plan/playground-reset-r5/notes/playground.md` (section 6).
- The draft, 6.1 and 6.2. `temper/docs/design/client/README.md`, 5.1
  (the banners) and 5.6 (the playground's pages, today's).
- `cl-play-domain`'s report (in its branch's last commit message or the
  plan's reports, if fab gives it) for the tests it left.
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s3-playground-today.md`,
  section 3, "Banner and setup strip" to "Remove-account action", and
  section 5's `temper-client-view` and `temper-client-world` lines.

## Owns and may change

- Owns: the files in the header: in `temper-client-view`, everything
  but `src/forge/**` and `src/pages/welcome.rs`'s Forge step (the welcome
  page's playground card may follow the domain's `playground_offered()`;
  nothing else of the welcome); the client world's `suite/demo_start.rs`
  (renamed), `suite/welcome.rs`'s playground cases, `qa_all/**` cases on
  the demo.
- Not yours: `temper-client-view/src/forge/**` and `src/tests/forges.rs`,
  `connect_forge.rs` (task `cl-forges` runs beside you there; it removes
  the Forges page's Playground section), the welcome's states and Forge
  step (`cl-welcome`), the journeys (`cl-journeys`): a journey test
  (`temper-client-journeys`) failing because the banner changed is listed
  under `## Left`.
- If `temper-client-view/src/forge/**` still calls a frame helper you
  remove (`setup_underway`), keep the helper's signature for it and say so.

## Steps

1. Banner, Leaving, words, looks; Accounts; the pages' project reads.
2. View tests; `heavy cargo nextest run -p temper-client-view --lib -E
   'test(/playground|frame|choice|accounts/)'`, then the crate's lib once.
3. Client world suites; `heavy cargo nextest run -p temper-client-world
   --test suite -E 'test(/playground|welcome/)'`, then the `suite`
   binary once.
4. `cargo fmt`; `heavy cargo clippy -p temper-client-view -p
   temper-client-world --all-targets -- -D warnings`; `heavy cargo check
   -p temper-client-gallery --tests -p temper-client-shell-web --tests`;
   the focused tests of `temper-client-view`, `temper-client-world`,
   `temper-client-gallery`. Old-forms from `temper/`. Commit and push.

## Report

As `notes/common.md` says.
