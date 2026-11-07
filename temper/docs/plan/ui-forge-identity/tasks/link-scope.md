+++
id = "link-scope"
part = "link"
kind = "feature"
needs = ["link-decide", "sole-owner"]
files = [
  "temper/crates/temper-engine-domain/src/route.rs",
  "temper/crates/temper-engine-domain/src/tests/resume.rs",
  "temper/docs/design/domain/root.status.md",
]
effort = "medium"
minutes = 35
+++

# The root hands the forge the deployment's sole owner at each start

## Goal

A forge's link to a box's person holds only while that person is the deployment's sole owner (`temper/docs/design/domain/forge.md`, 4.6). At each start, once the core's and the forge's records are loaded, the root hands the forge `Event::Owner { party: domain.core.people.sole_owner() }`, before the forge's restore decision (`temper/docs/design/domain/root.md`, 9.1). `link-decide` made the connector clear what that no longer holds.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 3), because the design already says what this task builds.

Acceptance:

1. In `temper-engine-domain/src/route.rs`, the `Work::ForgeRestored` arm, after `domain.forge.reconcile(..)` and before the `forge::Event::Restored` it delivers, delivers `forge::Event::Owner { party }` through `forge_event`, `party` being `domain.core.people.sole_owner()`.
2. A table test in `temper-engine-domain/src/tests/resume.rs` (a new one beside `a_project_live_task_and_forge_repository_open_together`, built on that test's restored store), its rows a restored owned Forgejo whose record holds `Link { user: 7, person: P }`, under a box, and the deployment's roster: `[P owner]` keeps the link after the start; `[P owner, Q owner]` and `[Q owner]` clear it; `[P owner, Q member]` keeps it. Read the link after the start by `domain.forge.link(forge)`. It bites: hand `None` always once, and see the keeping rows fail.
3. The entry *9.1: the root hands the forge no sole owner at a start.* is gone from `temper/docs/design/domain/root.status.md`.
4. Focused tests pass in `temper-engine-domain` and `temper-engine-domain-world`; clippy clean for `temper-engine-domain`.

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/contracts.md`, *The link*: it binds.
- `temper/docs/plan/ui-forge-identity/notes/common.md`.
- `temper/docs/design/domain/root.md`, 9.1, step 3; `temper/docs/design/domain/forge.md`, 4.6, *The person is the deployment's sole owner*; jig's `jig/docs/design/domain/people.md`, section 4, *The deployment's sole owner*.

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours:** a start test that counts the forge's events or requests at the link step, and now sees one more (`rg 'ForgeRestored|Event::Restored' temper/crates/temper-engine-domain/src/tests`): add it to what it expects, and name it in the report.

**Options, each with its default:**

- If `link-decide` cleared the links in `restored`'s decision rather than at `Owner` (its report says), the order here is the same: `Owner` first, then `Restored`.
- If the restored-store fixture can't hold a second owner's roster row, seed the roster through the people's `Stored::Roles` row the fixture restores, as `first_start`'s helpers write it. Default: whatever the fixture already offers for rows.

## Steps

1. The arm (acceptance 1).
2. The table test and its bite.
3. The status entry.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-domain --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-domain -p temper-engine-domain-world`. Commit ("temper-engine-domain: hand the forge the deployment's sole owner at each start"), push.

## Report

As the implementer's text says, with the bite's broken rule and failure.
