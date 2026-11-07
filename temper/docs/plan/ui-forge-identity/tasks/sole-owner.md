+++
id = "sole-owner"
part = "scope"
kind = "interface"
needs = []
files = [
  "jig/crates/jig-domain-people/src/domain.rs",
  "jig/crates/jig-domain-people/src/tests/projects.rs",
  "jig/docs/design/domain/people.status.md",
]
effort = "medium"
minutes = 25
+++

# jig's people answer the deployment's sole owner

## Goal

temper's forge connector will keep a link to a box's one person only while the deployment's roster holds one owner (`temper/docs/design/draft/forge-identity.md`, 9). jig's people have no read for that. Add one, generic: `Domain::sole_owner`, the party holding `Role::Owner` on the `DEPLOYMENT` roster when no other party holds it, else `None`.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 3), because the design already says what this task builds.

Acceptance:

1. `jig_domain_people::Domain::sole_owner(&self) -> Option<u64>` exists, `#[must_use]`, with a doc comment in the design's words, and answers as `jig/docs/design/domain/people.md`, section 4, *The deployment's sole owner* says.
2. One new table test in `jig/crates/jig-domain-people/src/tests/projects.rs` holds it, with the rows below, and bites (break the "no other owner" check once, see a row fail).
3. The words *4: the deployment's sole owner, no read.* are gone from the **Not yet** entry of `jig/docs/design/domain/people.status.md`, the rest of the entry unchanged.
4. `heavy cargo nextest run -p jig-domain-people --lib` passes; clippy clean for `jig-domain-people`.

## Inputs

- `temper/docs/plan/ui-forge-identity/notes/common.md`.
- `jig/docs/design/domain/people.md`, section 4, *The deployment's sole owner*: it binds.

## Owns and may change

**Owned:** the `files` above.

**Readers:** none yet; temper's root reads it in a later task of this plan (`link-scope`).

**Options, each with its default:**

- Gate the read as `project_members` does (`self.ready() || self.start_lookup()`), so a start's restore can read it and a failed start answers `None`. Default: call `self.project_members(crate::DEPLOYMENT)` and walk its holdings.
- A `Role::Policy { .. }` holding is never an owner. Two holdings of the same party as `Owner` can't occur (one role per project); don't handle it.

## Steps

1. The read, beside `project_members` in `jig/crates/jig-domain-people/src/domain.rs`:

   ```rust
   /// The party holding `owner` on the deployment's roster when no other party
   /// holds it; `None` with no owner, with several, or when the roster can't be
   /// read (people.md, 4). Allocates and changes nothing.
   #[must_use]
   pub fn sole_owner(&self) -> Option<u64> { ... }
   ```

   Every `match` on `Role` lists its variants (the strict subset); no `matches!`.
2. The table test, in `src/tests/projects.rs`, with the `Test` harness the file already uses, setting the roster by `Event::Roles { project: DEPLOYMENT, holdings }`:

   ```rust
   #[test]
   fn the_deployments_sole_owner_is_its_one_owner_or_none() {
       for (holdings, expected) in [
           (vec![], None),
           (vec![(PARTY, Role::Owner)], Some(PARTY)),
           (vec![(PARTY, Role::Owner), (OTHER, Role::Member)], Some(PARTY)),
           (vec![(PARTY, Role::Owner), (OTHER, Role::Owner)], None),
           (vec![(PARTY, Role::Maintainer)], None),
           (vec![(PARTY, Role::Policy { role: 9 })], None),
       ] { ... }
   }
   ```

   Add `const OTHER: u64 = 2;` beside `PARTY`. If the roster needs its parties known, sign them in first, as `settings_admission_is_the_deployment_owners_and_replays_by_digest` does with `test.signin`. A project roster holding an owner (not `DEPLOYMENT`) answering `None` is a seventh row if the harness makes it one line.
3. The status entry (acceptance 3).
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-people --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-people --lib`. Commit ("jig-domain-people: answer the deployment's sole owner, for a fact resting on one person"), push.

## Report

As the implementer's text says, with the rule broken for the bite and the failure seen.
