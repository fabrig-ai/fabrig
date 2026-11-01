+++
id = "rm-release-project"
part = "forge-removal"
kind = "feature"
needs = ["fr-surface", "jig-acts"]
files = [
  "temper/crates/temper-engine-domain-forge/src/release.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/release.rs",
]
effort = "high"
minutes = 45
+++

## Goal

The forge connector's half of jig's `ReleaseProject` (contracts,
section 4): once jig has erased a deleted project's tasks, the
connector erases its `Repository` rows of that project, ended or not,
and every row of its own that names one of those repositories or the
project, then answers `ProjectReleased`. A restart that cuts it sends
the ask again; the second answer finds nothing left and answers at once.

Acceptance:

- `release::release(d, env, request, project, out)` (the body of
  `fr-surface`'s stub): in one decision, erases (`Request::Erase`) and
  drops from the connector's tables each `Repository` row whose
  `project` is the project, and with each the rows that link to it
  (those whose reconcile need is `Need::Repository(it)`: `Ci`, `BranchHead`
  and `PullState` names on it, `Subscription` topics on it, `Hold` and
  `Names` entries naming it, `Receipt` rows of the project, `Landing`
  rows of the project, `Application` outcomes naming it), so that a
  later start's reconcile finds no missing link; then
  `Request::ProjectReleased { request, project }`.
- A project with no rows answers at once; a second ask after a restart
  answers at once.
- Rows of tasks still live are not this ask's: jig sends it only once
  the project's tasks are erased (contracts 4); if one is still there,
  leave the rows that name a live task and say so in a debug assertion's
  message (no panic in release).
- The tests below pass, and the connector's reconcile tests.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 4,
  D3).
- The draft, sections 4 ("its repositories' rows once no kept task
  names them") and 5 ("its adoptions released, each connector told").
- `temper-engine-domain-forge/src/reconcile.rs` (each row's needs: the
  list of what links to a repository) and `restore.rs`.
- The merged `fr-surface` and `jig-acts` reports.

## Owns and may change

New test files are declared from the module they test, at its end
(`#[cfg(test)] #[path = "tests/NAME.rs"] mod tests;`, the path relative to
that module, as `temper-protocol-store/src/records.rs` declares `holds`),
never in a crate's `src/tests.rs`: tasks running at once would all edit it.
Where this spec says "its `mod` line in `tests.rs`", read that.

Owned: `release.rs` and the new `tests/release.rs` (its `mod` line in
`tests.rs`).

Neighbours: `domain.rs`: making a private table `pub(crate)` if
`release.rs` must reach it (visibility only).

Pre-approved options:

- If the client keeps working rows (`Stored::Client`) for a released
  repository and offers no per-repository forget, leave them (no task
  watches them, so the client's keep drops them) and say so.

## Steps

1. `release.rs` (module comment cites contracts 4 and draft 5): the
   body, with a helper per family.
2. **Tests**, `tests/release.rs`: `a_projects_repository_rows_are_erased_then_released`,
   `rows_linking_to_a_released_repository_go_with_it` (restore a
   repository with a CI row, a branch head and a receipt, release,
   then feed the remaining rows to a fresh domain's restore and
   reconcile: no missing link),
   `another_projects_rows_stay`,
   `a_second_release_answers_at_once`.
   Narrow: `-p temper-engine-domain-forge --lib -E 'test(release::) |
   test(reconcile::)'`.
3. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge
   --all-targets -- -D warnings`; the crate's focused tests; `heavy
   cargo nextest run -p temper-engine-domain --lib -E
   'test(release_project)'`. Commit and push.

## Report

fab's sections, and `## Interfaces`: `release::release` and the
families it erases.
