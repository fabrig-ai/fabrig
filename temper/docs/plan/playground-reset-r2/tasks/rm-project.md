+++
id = "rm-project"
part = "forge-remove"
kind = "feature"
needs = ["rm-domain"]
files = [
  "temper/crates/temper-engine-domain-forge/src/project_removed.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/project_removed.rs",
  "temper/crates/temper-engine-domain/src/tests/project_removed.rs",
]
effort = "high"
minutes = 60
+++

## Goal

temper's forge connector hears jig's `connector::Ask::ProjectRemoved
{ project }` and, in the same decision, erases what it keeps for the
project: its repository rows (its adoptions, on any forge, a removed
forge's included) and every row keyed by or naming one of them, and any
row a task of the project left. One way: nothing is answered. This is
where a removed forge's repositories finally go (draft 9's second
default, as `rm-domain` keeps them until then).

Acceptance (new tests):

- `temper-engine-domain-forge`, `src/tests/project_removed.rs`:
  - `a_removed_project_erases_its_repositories_and_their_rows`: two
    projects, each with an adopted repository with a head, a pull, CI,
    a subscription and a client row; `Event::ProjectRemoved` of one
    erases (one `Request::Erase` each) every row of that project's
    repository and leaves the other's untouched; `repository(..)` and
    `repository_tag(..)` no longer find it.
  - `a_removed_forges_repositories_go_with_their_project`: a forge
    removed (`rm-domain`), its record erased once its acts answered,
    its repository rows still held; the project's removal erases them.
  - `rows_a_task_of_the_project_left_are_erased`: a change, a names row
    and an outbox entry of a task of the project still held (as when
    jig's erasure told the connector nothing per task) are erased.
  - `a_restart_after_the_removal_restores_nothing_of_the_project`:
    restore from the rows left, `reconcile` reports no dangling row.
- `temper-engine-domain`, `src/tests/project_removed.rs`:
  `jigs_project_removed_reaches_the_forge_connector`: the root's
  connector-ask arm for `ProjectRemoved` (a no-op since `jig-acts`)
  becomes `forge::Event::ProjectRemoved { project }`.
- The two crates' tests and their dependents' pass unchanged in meaning.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 1 (the
  connectors told: one way, in the decision that erases the project's
  own rows) and 2.
- `temper/docs/design/draft/playground-reset.md`, 4 ("Its records
  erased"), 5 ("Then its own rows") and 9 (second default).
- `temper/docs/design/protocol/configuration.md`, 13.2 (setting kinds
  2, 4 and 5 of a project's scope).
- `temper/docs/design/protocol/store.md`, its families table (which rows
  name a repository or a task).
- The code: `temper-engine-domain-forge/src/boundary.rs` (`Stored`,
  `Key`), `domain.rs` (`Event::ForgetAdoption`'s handler, the maps each
  family restores into), `src/remove.rs` (`rm-domain`).

Assumed: jig-acts' `jig_domain_core::connector::Ask::ProjectRemoved {
project: u32 }` and its no-op arm in
`temper-engine-domain/src/route/forge_route.rs`; rm-domain's
`forge_ended` and `remove.rs`.

## Owns and may change

Owns the new module `src/project_removed.rs` of the forge crate and the
two test files in the header.

May change: the forge crate's `boundary.rs` (one variant,
`Event::ProjectRemoved { project: u32 }`, placed right after
`Event::ForgetAdoption`), `domain.rs` (its dispatch arm and whatever
small accessor the erase needs), `lib.rs` (the module); the root's
`route/forge_route.rs` (the `ProjectRemoved` arm only). Forced arms per
`notes/scope.md`. temper keeps no setting of kinds 2, 4 or 5 today
(`Stored` has no `Setting` family): erase none, and say so in a comment
naming configuration.md 13.2.

Runs beside `pg-start`, which rewrites the demo's parts of the same two
crates: keep to the lines named here.

## Steps

1. `src/project_removed.rs`:

   ```rust
   /// playground-reset.md, 5; contracts.md, 1.
   pub(crate) fn removed(d: &mut Domain, env: &Env<Limits>, project: u32, out: &mut Queue<Request>);
   ```

   - Collect the project's repositories: every `d.repositories` row whose
     `project` is it. For each, erase its row and every row whose key or
     value names that repository (walk `Stored`'s families once and list
     them in a comment: heads, pull states, CI, landed, client rows,
     releases, bindings, subscriptions and goal topics on it, its
     credential/identity table entry, names entries naming it), each
     with `Request::Erase` and removed from its map.
   - Erase every task-keyed row whose task belongs to the project
     (changes, issues, names, holds, entries and their flights,
     landings, receipts, settles), found by the row's repository's
     project or the task's own project field. If jig's whole-task
     erasure already told the connector (`Unname`/`Release` per task:
     read `jp-` code if merged) these are gone and the walk finds
     nothing; keep the walk either way.
   - Drop volatile state naming them (pending reads, watches): reuse
     the walk `retire_ended` does for an ended forge's repositories.
   - Then a removed forge (rm-domain) whose record is gone and whose
     repositories are all erased holds nothing more: nothing to do, but
     assert in a test that no row names it.
2. Dispatch in `domain.rs`; the root's arm pushes
   `Work::Forge(forge::Event::ProjectRemoved { project })`.
3. Tests as the acceptance names. Then, one at a time:
   - `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/project_removed::/)'`;
   - `heavy cargo nextest run -p temper-engine-domain-forge --lib`;
   - `heavy cargo nextest run -p temper-engine-domain --lib`;
   - `heavy cargo nextest run -p temper-engine-domain-world -p temper-engine-forge-world`;
   - `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-domain --all-targets -- -D warnings`;
   - `python3 -I -B scripts/old_forms.py` from `temper/`.
4. Commit and push.

## Report

The standard report, plus: the families the walk erases and any it
leaves (with why); whether jig's erasure told the connector per task
when you built it; any row a removed forge left that nothing erases.
