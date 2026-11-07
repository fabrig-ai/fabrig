+++
id = "inventory-forges"
part = "reviewer"
kind = "feature"
needs = ["forge-subject"]
files = [
  "temper/crates/temper-engine-domain/src/route/host_route.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs",
  "temper/crates/temper-engine-smith/src/settings.rs",
  "temper/crates/temper-engine-smith/src/tests/settings.rs",
  "temper/tests/engine/domain/tests/suite/configuring.rs",
  "temper/docs/design/domain/agent.status.md",
]
effort = "medium"
minutes = 30
+++

# The inventory's line names each forge

## Goal

A user's number means nothing without its forge, so the configuring agent's inventory says which forge each number is on, and which forges of the project don't know the person. Its readers then go through the lookup `forge-subject` made, by way of `person_forges`.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 4), because the design already says what this task builds.

Acceptance:

1. `ProjectPerson` and `PersonForge` are as `notes/contracts.md` has them, and `settings_people` fills `forges` with `person_forges`.
2. The line is as the contract's table has it: `temper-engine-smith`'s table of people in `tests/settings.rs` holds it, with a person known on one of two forges among its rows, and its other rows on the new words.
3. `person_forges`' rows pass (Steps, 2).
4. The entry `temper/docs/design/domain/agent.status.md`, **Not yet**, 8.1, "a person's line in the inventory names one forge user", is gone.
5. Focused tests pass in `temper-engine-domain`, `temper-engine-smith`, `temper-engine-domain-world` and `temper-service`.

## Inputs

- `temper/docs/plan/ui-forge-reviewer/notes/contracts.md`, *The inventory's person* and *The lookup*: they bind.
- `temper/docs/plan/ui-forge-reviewer/notes/common.md`.
- `temper/docs/design/domain/agent.md`, 8.1, the sentences on the inventory's people.
- `temper/docs/design/domain/forge.md`, section 4 (*Who a forge's user is*).

## Owns and may change

**Owned:** the `files` above. `ProjectPerson` is read only in `route/host_route.rs` and `temper-engine-smith/src/settings.rs` and its tests; the `pub use` lines for `PersonForge` in `route.rs` and `lib.rs` of `temper-engine-domain` are yours.

**Reader pins, only their expected words:** `temper/tests/engine/domain/tests/suite/configuring.rs` pins `person 3: role 1, forge user 2`, `person 1 person: role 0, requester, forge user 7` and `person 4 member: role 2, requester, forge user 8`; each takes `forge HOST user N`, with HOST the world's repository host, as that inventory's repository line prints it. Its `forge identity unknown` pins stay.

**Options, each with its default:**

- A story's project with no repository: its people read `forge identity unknown`. That's true: change the pin, and say so.
- A second pin of these words that the survey missed: change its words, and say so.
- A help page or the charter seeming to need the new words: they don't. `configuring.txt` says "known forge users" and "an unknown forge identity cannot count for approvals", both still true. Leave them; other plans are writing that file.

**Not yours:** `temper/crates/temper-service/src/stock/configuring.txt`, `docs/help/`, `jig/**`.

## Steps

1. In `route/host_route.rs`: `PersonForge`, and `ProjectPerson` as the contract has it; `settings_people` loses its loop and its fields for `forges: super::forge_route::person_forges(domain, context.project, member.person)`. In `route/forge_route.rs`, `person_forges` walks `domain.forge.readable_repositories(project)`, takes each forge once with that repository's `host`, and fills `user` with `person_forge_user` and `writer` with `is_writer`. In `temper-engine-smith/src/settings.rs`, `project_people_rows` writes the contract's table.

2. **Tests.**
   - `forge_route/tests.rs`, one table beside the two `forge-subject` wrote, on its fixture, giving forge 3's repository another host, `other.example`: the party's key and what `person_forges` answers. `user_key(1, 7)`: `[forge.example user 7, other.example unknown]`. `user_key(3, 7)`: `[forge.example unknown, other.example user 7]`. With forge 1's writer being user 7, `user_key(1, 7)`: the first row marked `writer`. A key of provider 1: both unknown. And a second repository on forge 1 adds no row.
   - `temper-engine-smith/src/tests/settings.rs`: the `people` literals take `forges`, and a further person holds a known forge and an unknown one, in the expected text. Page-splitting stays as it is: it reads the same text.
   - The pins above. No new story.

3. Narrow tests after each change; then `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `python3 -I -B tools/citations/citations.py --base origin/main`, `heavy cargo clippy -p temper-engine-domain -p temper-engine-smith -p temper-engine-domain-world --all-targets -- -D warnings`, and the focused tests of acceptance 5. Commit and push.

## Report

As the implementer's text says, and: each rule broken by hand for a new row and how it failed; every option above you took.
