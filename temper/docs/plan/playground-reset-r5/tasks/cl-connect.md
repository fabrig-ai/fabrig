+++
id = "cl-connect"
part = "client"
kind = "feature"
needs = ["cl-forges", "fr-wire"]
files = ["temper/crates/temper-client-domain/src/address.rs", "temper/crates/temper-client-domain/src/forge/connect.rs", "temper/crates/temper-client-domain/src/forge/picker.rs", "temper/crates/temper-client-domain/src/forge/admin.rs", "temper/crates/temper-client-domain/src/forge/github.rs", "temper/crates/temper-client-domain/src/forge/forges.rs", "temper/crates/temper-client-domain/src/forge/family.rs", "temper/crates/temper-client-domain/src/tests/{address,connect,admin,github_app}.rs", "temper/crates/temper-client-view/src/forge/**", "temper/crates/temper-client-view/src/tests/connect_forge.rs", "temper/tests/client/src/front/forges.rs", "temper/tests/client/tests/suite/connect_forge.rs"]
effort = "high"
minutes = 110
+++

## Goal

Make a forge from the client (draft
`temper/docs/design/draft/playground-reset.md`, 3, "Its client": "Connect
a forge and the welcome make one"): the Connect page and the welcome's
Forge step offer the kinds the deployment can make, not the document's
slots, and send `SetUp`/`Connect` naming `ForgeChoice::New` for a forge
not yet kept, or `ForgeChoice::Kept` to go on with a kept one
(`fr-wire`). Until this task every sender sends `Kept` (fr-wire's forced
edits).

Acceptance:

- `temper_client_domain::Choice` (in `address.rs`) becomes `{ Owned,
  GitHub, Elsewhere, Forge(u16) }`: the first three a forge to make (an
  owned Forgejo here, a GitHub App, a Forgejo elsewhere by its address),
  `Forge(n)` a kept forge to go on with. Addresses print and parse them
  (`/connect-forge?forge=owned|github|elsewhere|N`, and the welcome's
  `?at=forge&forge=...` the same); an unknown value falls back to Choose,
  as today.
- `forge::connect::choices(rows, playground)` returns, in this order,
  `Owned`, `GitHub`, `Elsewhere` (never the playground's forge, never a
  fake), then nothing for kept rows: a kept forge is gone on with from
  its row (Forges, or the welcome's unfinished forge, task `cl-welcome`).
  An owned Forgejo is not offered once a kept row is `Mode::Owned` (one
  per machine: `MakeRefusal::Taken`); every choice stays offered
  otherwise, the deployment's answer saying when it cannot
  (`make_refused`, shown in the details as a refusal: words below).
- Senders: `forge::forges::set_up` / `set_up_at_once` (owned),
  `forge::github::send` / `manifest` (GitHub), `forge::admin::send` (a
  Forgejo elsewhere, its address the form's new field `Field::Address`)
  take a `ForgeChoice` (`New(NewForge::{Owned, GitHub { read_only },
  Forgejo { address }})` for `Choice::{Owned, GitHub, Elsewhere}`,
  `Kept(n)` for `Choice::Forge(n)`). After a `New` ask answers
  (`setting_up`, `connecting` or `link`), the session follows the row the
  Forges watch delivers next whose number it has not seen before the ask
  and whose kind and mode match: `connect::step` shows its progress as it
  does a kept forge's today, and the address is replaced by
  `Choice::Forge(n)`.
- `make_refused { full | taken | not_offered }` words, in
  `temper-client-view/src/forge/words.rs`: "temper keeps as many forges as
  it may: remove one first.", "This machine's Forgejo is already set up:
  go on with it from Forges.", "This deployment cannot set up this kind
  of forge."
- Tests: domain (`src/tests/connect.rs`, `src/tests/address.rs`):
  `choices_are_owned_github_and_elsewhere_with_no_kept_slots`,
  `a_kept_owned_forge_withdraws_the_owned_choice`,
  `a_new_choice_sends_new_and_follows_the_row_that_appears`,
  `a_kept_forge_sends_kept`, `choices_print_and_parse_in_addresses`;
  view (`src/tests/connect_forge.rs`): the choose page's three cards on a
  fresh deployment, each `make_refused` reason shown; client world
  (`tests/suite/connect_forge.rs`): its front (`src/front/forges.rs`)
  answers `New` asks by adding a row with the next number to the Forges
  watch; `owned_set_up_makes_a_row_and_follows_it`,
  `github_connect_makes_a_row_and_follows_it`. The existing connect-forge
  cases move from configured numbers to the choices.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md` (section 3),
  `temper/docs/plan/playground-reset-r5/notes/forge-records.md` (sections
  1, "the choice by kind and mode", and 6, the wire),
  `temper/docs/plan/playground-reset-r5/notes/playground.md` (section 6).
- `temper/docs/design/client/forge.md`, 5.4 and the Connect page's
  sections (`rg -n '^#' temper/docs/design/client/forge.md`).
- Survey `temper/docs/plan/playground-reset-r5/notes/surveys/s5-setup-welcome.md`,
  section 2 ("Forge choices/readers").

## Owns and may change

- Owns the files in the header; in `forge/forges.rs` only the senders
  (`set_up`, `set_up_at_once`, `retry_start`) and what they keep.
- May change, call sites only: every reader of `Choice` in
  `temper-client-domain`, `temper-client-view`, `temper-client-shell-web`,
  `temper/tools/gallery`, `temper/tests/client` (`rg 'Choice::'`).
- Not yours: the welcome page's states and its unfinished forge
  (`cl-welcome`, after you), `temper-client-tools` (`fr-ctl`).
- `NewGitHub.read_only`: `false`, unless the GitHub details already ask
  whether the App may write (then the person's answer); say which.
- If the Forges watch can deliver the new row before the ask's answer,
  follow it on whichever comes second; if two asks of the same kind are
  pending, follow by order of answer.

## Steps

1. `Choice`, addresses; `connect::choices`/`step`/`welcome_step`.
2. Senders and following; domain tests (`heavy cargo nextest run -p
   temper-client-domain --lib -E 'test(/connect|address|admin|github/)'`).
3. View and words; `heavy cargo nextest run -p temper-client-view --lib
   -E 'test(/connect_forge/)'`.
4. Client world; `heavy cargo nextest run -p temper-client-world --test
   suite -E 'test(/connect_forge/)'`.
5. `cargo fmt`; `heavy cargo clippy -p temper-client-domain -p
   temper-client-view -p temper-client-world --all-targets -- -D
   warnings`; `heavy cargo check --tests -p temper-client-shell-web -p
   temper-client-gallery`; the focused tests of the three. Old-forms
   from `temper/`. Commit and push.

## Report

As `notes/common.md` says.
