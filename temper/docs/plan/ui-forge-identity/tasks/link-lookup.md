+++
id = "link-lookup"
part = "link"
kind = "feature"
needs = ["link-record"]
files = [
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs",
  "temper/crates/temper-engine-domain/src/route/kinds.rs",
  "temper/crates/temper-engine-domain/src/tests/kinds.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/approval_identity.rs",
  "temper/docs/design/domain/forge.status.md",
  "temper/docs/design/protocol/configuration.status.md",
]
effort = "high"
minutes = 75
+++

# The one lookup gains the link, and eligibility counts per forge

## Goal

The root's lookup of who a forge's user is answers the forge's link first, so a box's person is known on a forge with no adoption, and a seeded party the link shadows is known on none. Acceptance counts people per forge of each repository a rule covers, the refusal names that forge, and a rule covering the playground's fake forge is refused in its own words. The surface `notes/contracts.md`, *The lookup the other readers use* gives is binding: the `approval` part's read builds on `forge_human` and `eligible_people`.

This task departs from the implementer's text in one way: three status entries are yours to remove (acceptance 6), because the design already says what this task builds.

Acceptance:

1. `forge_person`, `person_forge_user` and `person_forges` answer as `temper/docs/design/domain/forge.md`, section 4, *Who a forge's user is* now says, signatures unchanged; `human_forge_identity` is replaced by `forge_human(domain, project, forge, person)`; `kinds.rs`'s `eligible_people(domain, project, forge, role)` is `pub(super)` and counts on that forge alone.
2. New rows of the existing tables in `temper/crates/temper-engine-domain/src/route/forge_route/tests.rs`, building the forge with a restored record whose `person` is `Some(Link { .. })`:
   - `a_review_counts_only_from_the_forge_it_was_made_on`: a review by the link's user counts as the link's person, with their role; a review by the same user number on another forge doesn't.
   - `human_forge_identity_is_known_on_its_own_forge_and_is_not_its_writer` (renamed `forge_human_is_known_on_its_forge_and_is_not_its_writer`, now per forge): the linked person counts on the link's forge and on no other; a seeded party whose key names the link's user doesn't count there (shadowed); the link's person with no role in the project doesn't count.
   - `person_forges_names_each_forge_once_with_its_own_user_and_writer`: the linked person's line names the link's user on that forge, unknown on another; the shadowed seed's names unknown there.
3. Acceptance per forge, as `forge.md`, 8.9 now says, new rows of `approval_rules_count_known_forge_people_with_the_required_role` (`temper/tests/engine/domain/tests/suite/forge_routes/approval_identity.rs`): a person known only on forge 1 doesn't make a rule over forge 2's repository satisfiable, refused naming forge 2's host; the linked person makes `people: 1` accepted and `people: 2` refused; a rule covering a fake forge's repository is refused in the fake forge's words, whatever the count.
4. The refusal's words are the design's: "project P role R on forge HOST requires N distinct known forge people; currently M eligible." then the remedy as today; the fake forge's: "project P: the playground's forge has no accounts, so no one can approve there. Name a branch pattern or a project that leaves its repository out, or remove the approval requirement." The existing assertions of the old words change to the new ones and nothing else (below).
5. The two new behaviours bite: drop the link branch in `forge_person` once and see a row of 2 fail; count on every forge once and see the per-forge row of 3 fail.
6. The entries *4: `forge_person` answers adoption's seeds alone, never a forge's link.* and *8.9: eligibility counts an account on any forge of the project, ...* (`temper/docs/design/domain/forge.status.md`), and *13: eligibility counts an account on any forge of the project, ...* (`temper/docs/design/protocol/configuration.status.md`) are gone, the other entries unchanged.
7. Focused tests pass in `temper-engine-domain` and `temper-engine-domain-world`; clippy clean for `temper-engine-domain`. This changes what a shared item means: run the focused tests of `temper-engine-smith` too (its inventory text reads `PersonForge`).

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/contracts.md`, *The lookup the other readers use*: it binds.
- `temper/docs/plan/ui-forge-identity/notes/common.md`, and its rule on `forge_route.rs`.
- `temper/docs/design/domain/forge.md`, section 4 (*People the forge knows*, *Who a forge's user is*), 4.6 (the link), 8.9 (*The root checks approval eligibility*, *The playground's fake forge*).
- `temper/docs/design/protocol/configuration.md`, section 13 (the refusal per forge).

## Owns and may change

**Owned:** the `files` above. In `forge_route.rs`, the four lookup functions and nothing else.

**Readers and neighbours,** each edited only as said:

- `temper/tests/engine/domain/tests/suite/configuring.rs`, `refuse_owner_approval`: its `contains` of the old words becomes the new words with the fixture's forge host (`forge.example:3847`, as `approval_context` names it). Nothing else in that file: the story is `box-story`'s.
- `temper/crates/temper-engine-domain/src/tests/kinds.rs`, `a_refusal_carries_the_forges_words`, and any other test asserting the old eligibility words (`rg 'distinct known forge people' temper`): the words only.
- `temper/crates/temper-engine-smith/src/settings.rs` and its tests read `PersonForge`, whose shape doesn't change: no edit expected; if a test's expected inventory moves because a fixture now has a link, stop and say so.

**Options, each with its default:**

- `forge_person` with a link naming the user whose person has no role in the project: answers `None`, never the seed (the seed is shadowed whatever its role). Default as said; the design's words hold it.
- The refusal's forge name: the covered repository's `host`, as `PersonForge.host` and the inventory's lines use it. Default: the first covered repository of that forge, in `writable_repositories`' order.
- Per-forge iteration in `approval_refusal`: default, for each project, the distinct forges of the writable repositories the rule covers (as `covers_project` finds them), in that order; the first forge short refuses. Rename or reshape `covers_project` freely inside `kinds.rs`.
- The fake forge: known by `domain.forge.configured(forge)`'s mode `Mode::Fake`. If the root can't read it there, use the forge domain's nearest public read and name it in the report.
- Adoption's skipping of the link's user is `link-decide`'s, in the forge domain's `human_collaborators`; leave `outputs`' seeding loop as it is.

## Steps

1. The lookup in `forge_route.rs`: `forge_person`'s link branch first (`domain.forge.link(forge)`), then `person_forge_user`, then `forge_human` replacing `human_forge_identity` at its one caller.
2. `kinds.rs`: `eligible_people` per forge, `approval_refusal` per covered forge, `approval_words` naming the forge, the fake forge's words.
3. The rows (acceptance 2, 3), the readers' words (above), the bites.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain -p temper-engine-domain-world -p temper-engine-smith`. Commit ("temper-engine-domain: the lookup answers the forge's link first, and eligibility counts per forge"), push.

## Report

As the implementer's text says, with each bite's broken rule and failure.
