+++
id = "project-people"
part = "people"
kind = "feature"
needs = ["settings-reads"]
files = ["temper/crates/temper-engine-domain/src/route/host_route.rs", "temper/crates/temper-engine-domain/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-smith/src/settings.rs", "temper/crates/temper-engine-smith/src/tests/settings.rs", "temper/crates/temper-service/src/system.rs", "temper/tests/engine/domain/src/host_tool.rs", "temper/tests/engine/domain/tests/suite/configuring.rs", "temper/tests/system/src/composed.rs"]
effort = "high"
minutes = 80
+++

# The `settings` inventory names the project's people

## Goal

The configuring agent can't start "who approves" from the person, because no tool of its reads a project's members, their roles or their forge identities. Widen the answer it reads first rather than add a tool: an authorized inventory names the caller project's people and each owned forge's person. No brief section, no new tool, nothing in jig.

Acceptance:

- The inventory prints `notes/contracts.md`, 5's two blocks after `project repositories:`, each only with a row, and paginates as the rest does.
- A story in the engine's domain world holds: the chat's person is marked `you`; a member's login and forge user are named; a person seeded by adoption has no login part; temper's writer is marked and not counted; services are left out; another project's people are not named.
- A member's configuring chat reads the same lines as an owner's. A run without the settings family reads none of it: the core refuses it first.
- A setting, revision or history read carries neither block, and neither does a not-read answer.
- No line holds an email or a display name.
- `forge::Machine.owned_person` comes from `application.machine.owned.person`'s login; `forge people:` names it for an owned forge alone, once per host.

## Inputs

- `temper/docs/plan/configuring-words/notes/common.md`
- `temper/docs/plan/configuring-words/notes/contracts.md`, 5
- `temper/docs/design/domain/agent.md`, 8.1, *Its input*
- `temper/docs/design/domain/forge.md`, 8.7 (a person known by forge user id)

## Owns and may change

Owns the files in the header. Reader and neighbour files:

- `temper/tests/client/src/front/account_acts.rs`, `temper/tests/engine/domain/tests/suite/{first_repository,playground,reset}.rs`, and any other `forge::Machine { .. }` literal: the new field only.
- `temper/tests/client/src/front.rs`, `temper/tests/engine/domain/src/protocol.rs`: a `HostDelivery::Settings` pattern, only if the compiler asks.
- `settings_answer` and `settings_answer_bounded` in `temper-engine-smith/src/settings.rs` pass empty slices for the new arguments.

Options, pre-approved:

- `settings_repository_answer` takes the two new slices beside `repositories`. If a struct holding the three reads better at its call sites, use it and say so.
- A party's role number: `people::Role::number()`, as `route/kinds.rs` reads it.
- `requester`: `domain.core.tasks.task(flight.task)`'s `requester`; only `tasks::Party::Person` marks `you`. A delegate's run marks nobody.
- `forge_user`: only `IdentityKey { provider: 0, subject }` with an 8-byte subject, as `forge_route.rs`'s `human_forge_identity` reads it; `writer` is `domain.forge.is_writer` on a readable repository's forge. If `human_forge_identity` can share a helper, split one out in `forge_route.rs` and add that file to what you changed.
- An empty `people` block for a project with only services: no heading.

## Steps

1. `temper-engine-domain-forge`: `Machine.owned_person`; an accessor beside `configured`: `pub fn owned_person(&self, forge: u16) -> Option<&[u8]>`, `Some` when the forge's `mode` is `Owned` and the login isn't empty. `temper-service/src/system.rs`, near its line 1163, fills it from `machine.owned`.
2. `host_route.rs`: the two types, exported from `temper-engine-domain/src/lib.rs` beside `RepositoryIdentity`. `settings_people` and `settings_forge_people` on the shape of `settings_repositories`: only for an `Inventory` answer, from `domain.core.tasks.delegation(flight.task)`'s project, `domain.core.people.project_members`, `identity` and `kind`.
3. `temper-engine-smith/src/settings.rs`: `repository_rows` writes the two blocks. Readers `temper/tests/engine/domain/src/host_tool.rs` and `temper/tests/system/src/composed.rs` pass the new fields.
4. A test in `temper-engine-smith/src/tests/settings.rs` of each line's forms and of pagination across the blocks; stories in `temper/tests/engine/domain/tests/suite/configuring.rs`: `an_inventory_names_its_own_projects_people` and `a_members_chat_reads_the_same_people`.
5. For each test, break its rule once by hand (drop the project scope, drop the `you` mark), see it fail for that reason, restore. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain -p temper-engine-domain-forge -p temper-engine-smith -p temper-service -p temper-engine-domain-world --all-targets -- -D warnings`; `heavy cargo nextest run` on those packages; `heavy cargo check --tests` on `temper-engine-forge-world`, `temper-client-world` and the system world, naming them from their manifests. Commit and push.

## Report

Your text's report, with one whole inventory as a story prints it.
