+++
id = "making-decide"
part = "making-engine"
kind = "feature"
needs = ["making-wire", "making-family"]
files = ["temper/crates/temper-engine-domain-forge/src/making.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/adoption.rs", "temper/crates/temper-engine-domain-forge/src/removal.rs", "temper/crates/temper-engine-domain-forge/src/make.rs", "temper/crates/temper-engine-domain-forge/src/tests/making.rs"]
effort = "high"
minutes = 70
+++

# The connector decides a making

## Goal

The forge connector takes `MakeRepository`, holds its row through its states and hears jig's adoption of
it, as `temper/docs/design/domain/forge.md`, 4.5 says (contracts T1, T5, T6). Nothing runs a plan yet.

Acceptance, in `src/tests/making.rs`, each named for its rule:

- `only_an_owner_may_make`; `a_forge_not_kept_or_ended_is_no_such_forge`;
  `github_and_a_connected_forgejo_are_not_offered`, and owned and managed are;
  `a_name_or_address_breaking_the_grammar_is_invalid`; `a_home_in_a_project_with_one_is_home_held`;
  `past_makings_is_full`.
- `a_kept_making_writes_a_waiting_row`; `a_second_request_is_answered_as_the_first`;
  `a_failed_row_asked_again_is_replaced_keeping_its_mark_for_the_same_address_alone`.
- `the_mark_is_saved_and_told_after`: `MakingCopying` saves `copying` and asks `MakingCopied`.
- `a_stop_at_taken_clears_the_mark`.
- `made_with_no_into_is_erased`; `made_with_into_is_adopting`.
- `making_adopt_asks_the_act_once`, with T6's key, and `the_key_is_the_same_after_a_restart`.
- `making_asked_adopts_with_no_locate`: `MakingAsked` puts the adoption in `adoptions` at its permission
  read, so section 4's reads and refusals hold.
- `adopted_is_erased`; `refused_is_failed_at_adoption_and_the_repository_stays`.
- `a_removal_erases_the_forges_rows_in_its_decision`.
- `rows_read_in_the_forges_document_in_key_order`.
- `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-forge-world` pass;
  `heavy cargo check --tests -p temper-engine-domain -p temper-service` passes.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, T1, T4, T5, T6.
- `temper/docs/design/domain/forge.md`, 4 and 4.5.
- `temper/docs/design/draft/repository-copies.md`, 5.
- jig's `jig/docs/design/domain/people.md`, 5.3, "`Adopt`".

## Owns and may change

- The files of the header.
- **The dispatch** is `domain.rs`, about 1214, where `Ask::SetUp` goes to `make::setup`, which never gets
  `role`: `MakeRepository`'s arm passes `role` and checks `Owner` itself, as `connect.rs`, 120 does.
- **The grammar:** `temper_client_protocol_wire::source::{parse, address, org, repository}`. `setup_login`
  and `setup_repository` (`domain.rs`, about 6025) may become one-line calls of `org` and `repository`,
  keeping their pinning comment; `make.rs`'s callers do not change.
- **The outcome's durability:** `adoption.rs`, 199, keeps `ApplicationOutcome`, and
  `application_answer` (`domain.rs`, about 392) maps every kept outcome to `Answered::Adopt`: it maps the
  two new ones to `Answered::Make`.
- **`host`** is the `Ask`'s, kept in `MakeInto`: `temper-service`'s `to_root` fills it, as it does `Adopt`'s.
  Here, a test's literal.
- **Reader files, each an arm alone:** `temper-engine-domain/src/{boundary,decision,admission,domain}.rs`,
  `temper-engine-domain/src/route/{applications,forge_route,playground}.rs`,
  `temper-service/src/{family,route}.rs`. A `Request` the root does not yet route goes in its existing
  or-pattern of what it does nothing with; `making-root` routes them.
- `removal.rs`, `begin`, 32: the row's `Request::Erase`, beside `first`'s clearing.
- `temper/tests/engine/forge`: an arm in `src/world.rs` if a match needs one.
- Pre-approved:
  - `adoptions` is bounded by `forge.adoptions`; `MakingAsked` past it answers jig `Again`, as a party's is.
  - `MakingAct` carries the `Adoption` of `forge.md`, 4.5: role `Owned`, `ci` true, no checks, prefix
    `temper/`, `landing` the row's branch, `owner` and `name` the row's.
  - A `MakingStopped` for a key with no row is ignored: its forge was removed.

## Steps

1. The ask's admission. 2. The states and events. 3. `making_adopt`, `MakingAsked` and `adopted`'s branch
for an act, which goes to `MakingActed`'s caller rather than to an application. 4. Removal. 5. The tests,
each biting: drop the role check, and keep the mark across a new address, each once. 6. The implementer's
checks; commit and push.

## Report

As the implementer's text asks; how an act's adoption is told from a party's in `adopted`.
