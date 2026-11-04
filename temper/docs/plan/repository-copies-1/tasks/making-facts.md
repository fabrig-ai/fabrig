+++
id = "making-facts"
part = "making-client"
kind = "feature"
needs = ["making-wire"]
files = ["temper/crates/temper-client-domain/src/forge/family.rs", "temper/crates/temper-client-domain/src/forge/forges.rs", "temper/crates/temper-client-domain/src/forge/making.rs", "temper/crates/temper-client-domain/src/tests/making.rs"]
effort = "medium"
minutes = 30
+++

# The client's facts of a making

## Goal

The client reads a forge's rows of repositories being made and can form the request. No page shows them.

Acceptance:

- `forges::Row` gains `makings: Box<[Making]>`, and the watch fills them, in a new `tests/making.rs`:
  `a_rows_makings_are_delivered_in_order`.
- `making.rs`: `pub fn ask(forge, org, name, from, into) -> Result<w::MakeRepository, Refused>`, checking
  the client's half with `source::{org, repository, parse}`, and giving the address `source::address`'s
  form; `pub fn copy_target(source) -> (org, name)`, the source's owner and name. Tests:
  `a_copys_target_is_its_sources_owner_and_name`, `an_owner_that_is_no_org_needs_as`.
- `pub fn phrase(row: &Making) -> Box<[u8]>`: "Copying ORG/NAME from HOST", "Making ORG/NAME",
  "ORG/NAME failed at STEP: WORDS". `the_row_says_what_it_is`.
- `heavy cargo nextest run -p temper-client-domain` passes; `heavy cargo check --tests -p temper-client-tools
  -p temper-client-view -p temper-client-world` passes.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, T3 and T4.
- `temper/docs/design/client/README.md`, 4.6; `temper/docs/design/draft/repository-copies.md`, 5.2 to 5.4.

## Owns and may change

- The files of the header. Stay off `forge.rs`, `forge/me.rs`, `route.rs` (`to-begin-store`) and `pages/`
  (`look-round-3`): `forge.rs` takes its `mod making;` line alone.
- `forges::Row` literals, the field alone: `temper-client-domain/src/tests/{forges,connect,github_app}.rs`,
  `temper-client-tools/src/tests/forges.rs`.
- Pre-approved: `Making` here is the client's own struct of `w::RepositoryMaking`'s fields; if the name
  collides in `forge.rs`, `RepositoryMaking`.

## Steps

1. The row. 2. `making.rs`. 3. The tests, each biting: leave the owner's check out once. 4. The
implementer's checks; commit and push.

## Report

As the implementer's text asks.
