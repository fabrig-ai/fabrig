+++
id = "pg-connect-kinds"
part = "playground"
kind = "feature"
needs = ["fr-setup", "fr-connect", "pg-start-over"]
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/forge/connect.rs",
  "temper/crates/temper-client-domain/src/forge/github.rs",
  "temper/crates/temper-client-domain/src/forge/admin.rs",
  "temper/crates/temper-client-domain/src/forge/forges.rs",
  "temper/crates/temper-client-domain/src/forge/form.rs",
  "temper/crates/temper-client-domain/src/tests/connect_kinds.rs",
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/forge/bindings.rs",
  "temper/crates/temper-client-view/src/tests/connect_kinds.rs",
]
effort = "high"
minutes = 60
+++

## Goal

The welcome's and Connect a forge's choices are by kind, not by kept
row (draft `temper/docs/design/draft/playground-reset.md`, 3, "Its
client": "Connect a forge and the welcome make one"; contracts,
section 5: "`SetUp` and `Connect` make the forge"), so a store that
keeps no real forge, a fresh `temper init`'s or one after a reset or a
start over, still offers GitHub and the owned Forgejo. Written by the
answerer (2026-11-01): `fr-setup` names this task as the reader of
`Forges.owned`, and the plan held no spec for it.

Acceptance:

- `address::Choice` gains `GitHub` and `Owned` (a forge to make, by
  kind), encoded `forge=github` and `forge=owned`, and, on Connect a
  forge only, `Forgejo` (a managed Forgejo to connect by its address),
  `forge=forgejo`; `Choice::Forge(n)` and `Elsewhere` stay, decoding
  as today.
- `connect::choices`: every row it offers today, as today; then
  `GitHub` when no GitHub row is offered; `Owned` when no owned row is
  listed and the `forges` subject's `owned` (`fr-setup`'s
  `Forges.owned`) is not `not_offered`; on Connect a forge, `Forgejo`;
  then `Elsewhere`.
- The kind choices draw the cards the rows' choices draw today, in the
  same words (GitHub's "Connect GitHub"; the owned Forgejo's "Run on
  this machine", its note by `Forges.owned` as `welcome_forgejo` reads
  a row's `offer` today); their acts send:
  - GitHub's manifest and App id: `w::Request::Connect` with
    `ConnectForge::Github`, its flow's back address naming no forge
    (`/welcome?at=forge`, or Connect a forge's address);
  - the owned Forgejo's run: `w::Request::SetUp`, which names no forge
    already (`family.rs`' `set_up`); `Act::SetUpAtOnce` and
    `Meaning::SetUpAtOnce` lose their forge number if the row's is not
    read;
  - a managed Forgejo's connect: `ConnectForge::Forgejo { address }`
    with the admin's token, the address from the admin form's new
    field.
- Once sent, the step follows the forge made: the newest row of that
  kind that was not listed when the request went out (the answer
  carries no forge number), and from then on as `personal` follows it.
- With no row at all, the welcome's Forge step draws GitHub and the
  owned Forgejo (when offered) and Elsewhere.
- The tests below pass, and every existing test of the packages
  changed.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `notes/contracts.md` (sections 5 and 6),
  `notes/progress-playground.md` (the second contract problem).
- The draft, 3 and 6.3; `temper/docs/design/client/forge.md`, 5.4;
  `temper/docs/design/client/README.md`, 3.5.
- The merged `fr-setup` (`Forges.owned`), `fr-connect` (the connector's
  make on `ConnectForge::Github`/`Forgejo`) and `pg-start-over`
  (`connect::unfinished`, which leaves offered rows to the choices)
  reports.

## Owns and may change

Owned: the `files` above (the two test files new).

Neighbours: every `match` on `Choice`, `Act::SetUpAtOnce` or
`Meaning::SetUpAtOnce` in the client crates, `temper/tests/client/`,
`temper/tests/one-process/`, `temper/tools/gallery/`: the new arms or
the dropped number only; `temper-client-domain/src/tests.rs`,
`temper-client-view/src/tests.rs`: one `mod` line each.

Pre-approved:

- if a kind's card is drawn by a function that takes a `Row` today,
  split it into the card's words and its act, the row's call kept;
- the welcome keeps leaving the managed Forgejo to Connect a forge
  (`choose`'s `Place::Welcome` rule).

A wire change (the answer carrying the made forge's number, say) is a
question.

## Steps

1. Domain: `Choice`'s new variants and their encoding; `choices`; the
   GitHub and admin acts on a kind choice; the step's following of the
   forge made.
2. View: the kind cards in `choose` and `welcome_forgejo`; bindings.
3. Tests:
   - `temper-client-domain/src/tests/connect_kinds.rs`:
     `kind_choices_round_trip_in_the_address`,
     `an_empty_store_offers_github_and_the_owned_forgejo`,
     `owned_is_not_offered_without_the_template`,
     `connect_github_on_its_kind_sends_a_new_github_forge`,
     `run_on_this_machine_on_its_kind_sends_set_up`,
     `the_step_follows_the_forge_made`;
   - `temper-client-view/src/tests/connect_kinds.rs`:
     `the_welcome_with_no_row_draws_github_and_run_on_this_machine`,
     `connect_a_forge_with_no_row_offers_a_forgejo_by_its_address`,
     `after_a_start_over_the_welcome_draws_the_kinds_again`.
   - Narrow runs per file.
4. Checks, once: `cargo fmt`; `heavy cargo clippy -p
   temper-client-domain -p temper-client-view --all-targets -- -D
   warnings`; focused tests of both; `heavy cargo check --tests` for
   `temper-client-tools`, `temper-client-shell-terminal`,
   `temper-client-shell-web`, `temper-client-world`,
   `temper-client-gallery`, `temper-one-process-world`, one at a time.
   Commit, push.

## Report

fab's sections; `## Interfaces`: `Choice`'s variants and encodings,
`choices`' order, the acts' new forms.

## Answers

2026-11-01. Q: may existing choice-list assertions in the domain tests
change to include the new kind choices? A: yes, option (1). The
acceptance's `choices` order makes the old expected slices untrue, so
those expectations follow it as neighbour edits. Change only the
expected slices: in `temper/crates/temper-client-domain/src/tests/connect.rs`,
`choose_offers_github_forgejo_here_and_elsewhere_by_offer` gains
`Forgejo` before `Elsewhere`, and in `src/tests/admin.rs`,
`only_the_configured_admin_form_has_a_card` gains `GitHub` and
`Forgejo` where the acceptance places them. Leave each test's name,
setup and other assertions as they are. Any other existing test whose
expected choices change by the same rule gets the same edit, and the
report's Changed names it. A test that fails for another reason is
still a question.
