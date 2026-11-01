+++
id = "cl-forges"
part = "client"
kind = "feature"
needs = ["cl-play-domain", "fr-wire"]
files = ["temper/crates/temper-client-domain/src/forge/forges.rs", "temper/crates/temper-client-domain/src/forge/listed.rs", "temper/crates/temper-client-domain/src/forge/remove.rs", "temper/crates/temper-client-domain/src/forge/form.rs", "temper/crates/temper-client-domain/src/forge.rs", "temper/crates/temper-client-domain/src/tests/forges.rs", "temper/crates/temper-client-domain/src/tests/remove.rs", "temper/crates/temper-client-view/src/forge/**", "temper/crates/temper-client-view/src/tests/forges.rs", "temper/crates/temper-client-view/src/tests/forge_page.rs", "temper/tests/client/tests/suite/forges.rs", "temper/tests/client/src/front/forges.rs"]
effort = "high"
minutes = 110
+++

## Goal

The Forges page lists the forge connector's records and offers Remove on
a forge's row, confirmed with words naming what goes and what stays on
the forge (draft `temper/docs/design/draft/playground-reset.md`, 3, "Its
client", and 4, "On the forge itself" and "Its client"). A fresh
deployment lists no GitHub or Forgejo slot until one is chosen; while the
playground runs, its forge's row is listed with nothing to add.

Acceptance:

- `forge::listed::personal`: every row whose mode is not `Mode::Fake`,
  whatever its connection (the records are the person's choices; the
  document's untouched `NotConnected`/`NotSetUp` slots are gone with
  `fr-wire`). `listed::rows` and `dot` keep their order and colours.
- Forges page (`temper-client-view/src/forge/forges.rs`): the person's
  rows as today; the playground's row (from `listed::playground`, after
  `cl-play-domain`) among the rows, folded as theirs are, with no access
  line and no Remove; the separate "Playground" heading and section go.
  While the playground runs, neither Add forge nor the empty list's
  Connect a forge is drawn.
- Remove (`forge::form::Act::RemoveForge { forge }`, then
  `Act::ConfirmRemoveForge { forge }` and `Act::CancelRemoveForge`): the
  first opens the row's confirmation (one at a time, held in a new
  `forge/remove.rs` state on `forge::Side`: `Confirming { forge }`,
  `Sent { forge, request }`, `Refused { forge, why }`); the confirm sends
  `fr-wire`'s `RemoveForge { forge }` once; its `removing` answer clears
  it (the row leaves with the Forges watch's next delivery);
  `remove_refused` is shown on the row; `unknown` clears it. Only an
  owner of the deployment sees Remove (the owners' act, draft 4).
- The confirmation's words, in `temper-client-view/src/forge/words.rs`
  (add forge words there, not in `src/words.rs`, so `cl-play-view`'s
  edits there merge cleanly), naming the forge by its row's name as the
  row's summary does:
  - every kind: "Remove FORGE? Its repositories leave their projects and
    its changes are cancelled. Chats that worked on it stay." with
    Remove and Cancel;
  - an owned Forgejo (`Mode::Owned`): "temper stops this Forgejo and
    deletes its directory, its repositories with it. Forgejo's binaries
    stay in the cache.";
  - a Forgejo elsewhere (`Mode::Managed`, `Mode::Connected`): "The forge
    itself is not changed: its bot account and webhooks stay there, for
    you to delete.";
  - GitHub: "The GitHub App stays yours: delete it in its settings on
    GitHub." with a link to the App's settings (the row's `web` address
    joined with `/settings/apps`, or text alone with no `web`).
- Tests: domain (`src/tests/remove.rs`, new, registered as the crate's
  tests modules are): `remove_asks_to_confirm_then_sends_remove_forge_once`,
  `a_removing_answer_clears_the_confirmation`,
  `a_refusal_is_kept_on_its_row`, `only_an_owner_is_offered_remove`;
  `src/tests/forges.rs`: `every_kept_forge_is_listed_whatever_its_connection`.
  View (`src/tests/forges.rs`): `a_fresh_deployment_lists_no_forge_and_offers_add`,
  `the_playground_forge_is_a_row_with_nothing_to_add`,
  `remove_confirmation_names_what_goes_and_what_stays_for_each_kind`,
  `the_github_confirmation_links_to_the_apps_settings`. Client world
  (`tests/suite/forges.rs`): `removing_a_forge_confirms_then_its_row_leaves`,
  its front's `RemoveForge` answers scripted in `src/front/forges.rs`
  (new; registered in `src/front.rs` by one `mod` line).
- The old view tests on the Playground section and untouched slots
  (`the_playground_forge_shows_while_the_forge_half_holds`,
  `the_playground_forge_has_no_access_line`, the setup-strip cases left by
  `cl-play-domain`) deleted or moved to these facts.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 3),
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md` (section 6,
  the wire: `RemoveForge`, `Removing`, `RemoveRefused`),
  `temper/docs/plan/playground-reset-r5/notes/playground.md` (section 6).
- The draft, 3 ("Its client") and 4. `temper/docs/design/client/forge.md`,
  5.4 (the Forges page).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s3-playground-today.md`,
  section 3, "Forges selection" and "Forges page".

## Owns and may change

- Owns the files in the header. In `forge.rs` and `forge/form.rs`, only
  the `RemoveForge` acts, their dispatch and `remove`'s state on `Side`.
- May change: `temper/tests/client/src/front.rs` (one `mod` line and a
  dispatch arm sending `RemoveForge` to `front/forges.rs`);
  `temper-client-view/src/tests/fixtures.rs` (add a fixture function at
  its end only; `cl-play-view` edits the rest of it at the same time).
- Not yours: `src/words.rs` and every view file outside `src/forge/**`
  (`cl-play-view`), the welcome (`cl-welcome`), `forge/connect.rs`
  (the Connect page and welcome's Forge step, `cl-welcome`).
- If the Forges watch still delivers a removing forge's row, hide a row
  whose number is in `Sent` once `removing` answered, and say so.

## Steps

1. Domain: listing, `forge/remove.rs`, acts; domain tests; `heavy cargo
   nextest run -p temper-client-domain --lib -E 'test(/remove|forges/)'`.
2. View: page, words, confirmation; `heavy cargo nextest run -p
   temper-client-view --lib -E 'test(/forges|forge_page/)'`.
3. Client world; `heavy cargo nextest run -p temper-client-world --test
   suite -E 'test(/forges/)'`.
4. `cargo fmt`; `heavy cargo clippy -p temper-client-domain -p
   temper-client-view -p temper-client-world --all-targets -- -D
   warnings`; the focused tests of the three. Old-forms from `temper/`.
   Commit and push.

## Report

As `notes/common.md` says.
