+++
id = "box-story"
part = "link"
kind = "feature"
needs = ["link-lookup", "link-scope", "link-words"]
files = [
  "temper/tests/engine/domain/tests/suite/configuring.rs",
  "temper/tests/engine/domain/tests/suite/configuring/fixture.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/approval_identity.rs",
  "temper/docs/design/domain/testing.status.md",
]
effort = "high"
minutes = 60
+++

# The engine's world: a box's person says it, and their rule is accepted

## Goal

The meeting point of the link part: through the root and the core on jig's store, a box's person known on its owned Forgejo by the forge's link asks for their own approval, and the configuring agent's rule is accepted with nothing added on the forge. This is steps 1 and 2 of `temper/docs/design/draft/forge-identity.md`, section 2; steps 3 to 5 (the change waits under Needs you, approved, landed) extend this story in the `approval` part. Today's story of a box's owner (`a_box_inventory_distinguishes_the_requester_from_the_forge_approver`) asserts the refusal; it becomes the story `temper/docs/design/domain/testing.md`, 6.4's row *a box's approval identities* now names.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 4), because the design already says what this task builds.

Acceptance:

1. The story, in `temper/tests/engine/domain/tests/suite/configuring.rs`, renamed `a_box_person_known_by_the_forges_link_approves_with_nothing_added`, on `box_approval_world()` under a box: the setup host's answer is sent to the root as `temper-service` sends it, `root::Event::Forge(Box::new(forge::Event::PersonOf { forge, answer: PersonAnswer::Is(user) }))` with the person's Forgejo user; then the inventory names person 1's forge account (`forge forge.example:3847 user N`, no "forge identity unknown"); the agent's owner-role rule with `people: 1` is accepted; one with `people: 2` is refused, naming the forge.
2. Rows of one table in that file (make the story a table of its worlds, or add a sibling table beside it, sharing `box_approval_world`), each a world at the domain world's usual cost:
   - the owned Forgejo linked: accepted (acceptance 1);
   - a GitHub forge whose App is the person's, if the world's fixture offers a GitHub forge: the same, with `Is(owner id)`; if it offers none, leave the row out and say so in the report;
   - the deployment given a second owner and restarted: the link cleared at the start, the rule refused naming the forge;
   - no link (today's world: the setup host sends nothing): the requester refused as before, and accepted as maintainer once a human collaborator is adopted as admin, today's `adopted_approval` flow kept whole.
3. `a_box_refuses_an_unfulfillable_person_approval_rule` (`forge_routes/approval_identity.rs`) keeps asserting the refusal for a box with no link, its words the per-forge words `link-lookup` gave; nothing else there changes.
4. The entry *6.4, the row a box's approval identities: ...* is gone from `temper/docs/design/domain/testing.status.md`.
5. `heavy cargo nextest run -p temper-engine-domain-world` passes, and its focused figure stays within the domain world's budget: name the story's seconds in the report.

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/contracts.md`, *The link*.
- `temper/docs/plan/ui-forge-identity/notes/common.md`.
- `temper/docs/design/domain/testing.md`, 6.4, the row *the engine's domain world, a box's approval identities*: it binds.
- `temper/docs/design/domain/forge.md`, 4.6 and 8.9; `temper/docs/design/domain/agent.md`, 8.1.

## Owns and may change

**Owned:** the `files` above. Only the box approval story, its helpers (`approval_context`, `refuse_owner_approval`, `adopted_approval`, `box_approval_world`, `approval_documentation`) and the fixture's box world in `configuring/fixture.rs`; the file's other stories stay.

**Readers and neighbours:** a helper another story in `configuring.rs` shares and this one changes: keep its old behaviour for that story (a parameter, or a second helper), and name it in the report.

**Options, each with its default:**

- **The world under a box:** default, the fixture's existing box flag; if the domain world's root config has no `under_box`, set `Config.under_box` in the fixture (`temper-engine-domain/src/translate.rs`), one line.
- **The second owner:** default, a second configured owner at the world's first start, or a deployment roster changed through the core's `SetRoles` ask, whichever the world already drives; then restart as `resume`'s stories do.
- **The configuring agent is scripted:** the script checks the context it's given (inventory, help, instructions) and the outcomes, not a model's judgement, as the story does today.

## Steps

1. The fixture's box world sends the `PersonOf`, the story's first row.
2. The other rows; `approval_identity.rs`'s assertion if its words moved.
3. The status entry.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain-world`. Commit ("temper: the engine's world has a box's person approve with nothing added"), push.

## Report

As the implementer's text says, with the story's seconds.
