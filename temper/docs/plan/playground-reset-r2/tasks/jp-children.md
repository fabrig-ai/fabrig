+++
id = "jp-children"
part = "project-delete"
kind = "feature"
needs = ["jig-acts"]
files = [
  "jig/crates/jig-domain-accounts/src/**",
  "jig/crates/jig-domain-notes/src/**",
  "jig/tests/accounts/tests/suite/drop_project.rs",
  "jig/tests/notes/tests/suite/drop_project.rs",
]
effort = "medium"
minutes = 45
+++

## Goal

Give the accounts and notes children what a project's deletion asks of
them (playground-reset.md, 5, "Then its own rows"): accounts drop the
project from every account's uses (jig's `domain/accounts.md`, 4.2), and
notes let go of every cached index of a scope naming the project, whose
rows the core erases from the store (`jp-delete`). Each is one new event.

Acceptance:

- `jig-accounts-world`, new `tests/suite/drop_project.rs`:
  - `dropping_a_project_removes_it_from_every_accounts_uses`: three
    accounts, uses `[Project(1), Party(4)]`, `[Project(1)]`, `[Project(2)]`,
    and one with default uses (`None`): after `Event::DropProject {
    project: 1 }`, the first's uses are `[Party(4)]`, the second's `[]`,
    the others unchanged; one `Save` of the account record for each
    changed account, none for the others.
  - `an_account_with_no_use_left_serves_no_project`: the second account
    (uses `[]`) is selected for no payer (`selection.rs`'s rule as it
    stands for an explicit empty list; if an explicit empty list there
    means something else, keep the account's uses `[]` and say what the
    selection does in the report).
  - `the_drop_survives_a_restart`: restored from the saved records, the
    uses are as after the drop.
- `jig-notes-world`, new `tests/suite/drop_project.rs`:
  - `dropping_a_project_forgets_its_cached_indexes`: indexes of
    `Project { 1 }`, `Goal { 1, 9 }`, `Resources { 1, .. }` and
    `Project { 2 }` cached; after `Event::DropProject { project: 1 }`, the
    next index of each of project 1's scopes loads from the store again
    (a `Request::Load`), and project 2's is served from the cache.
  - `a_drop_while_busy_is_kept_for_after`: with a call in flight, the
    drop is applied once the call completes, not refused (notes serve one
    call at a time: `busy()`), so the core never has to send it again.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md`.
- `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/design/draft/playground-reset.md`, section 5.
- `jig/docs/design/domain/accounts.md`, 4.2 (an account's uses).
- `jig/docs/design/domain/notes.md` (its scopes and cache; rg `Scope`).

## Owns and may change

Owns the two crates' sources and the two new test files. May change, as
`notes/scope.md` says: each world's `tests/suite/main.rs` (the `mod`
line), and every exhaustive match the new `Event` variants force (the
core's `routing/children.rs` and `routing/notes.rs` route these events
and nothing else matches on them; an arm there, if forced, does nothing
new).

Not yours: deciding when to drop (`jp-delete`); erasing notes' rows
(the core does, from `Scope::Notes { project }` pages).

## Steps

1. Accounts (`boundary.rs`): `Event::DropProject { project: u32 }`, doc
   citing `temper/docs/design/draft/playground-reset.md, 5`. The handler
   (a new `drop_project` beside `changes.rs`'s `SetUses`): for each
   account in number order whose `uses` is `Some` and holds
   `Use::Project(project)`, remove it and push `Request::Save { record:
   ordering::account_record(domain, number) }`. No refusal: an account
   closing or not ready is changed all the same (the project is gone for
   it either way). It emits nothing else.
2. Notes (`boundary.rs`): `Event::DropProject { project: u32 }`. The
   handler removes from `indexes` every cached scope whose project is
   `project` (`Scope::Project`, `Goal` and `Resources` naming it). While
   `busy()`, hold the project (a bounded field, `Limits`'s scope count
   as its bound, or one `Option<u32>` replaced by a list if more than one
   may wait) and apply it when the call ends. It emits no write.
3. Tests: the acceptance's. Then, one at a time:
   - `heavy cargo nextest run -p jig-accounts-world --test suite -E 'test(/drop_project/)'`;
   - `heavy cargo nextest run -p jig-notes-world --test suite -E 'test(/drop_project/)'`
     (binary names from each world's `Cargo.toml`);
   - `heavy cargo nextest run -p jig-domain-accounts -p jig-domain-notes -p jig-accounts-world -p jig-notes-world`;
   - `heavy cargo nextest run -p jig-domain-core -p jig-core-world`;
   - `cargo fmt`; clippy over the packages you edited; old forms from `jig/`.
4. Commit and push.

## Report

The standard report, plus what an explicit empty use list means to
selection, as you found it.
