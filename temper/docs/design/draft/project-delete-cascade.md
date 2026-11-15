# Deleting a project takes its forge records with it

Draft, 2026-11-15, on the user's word. This is the root fix behind `deleted-repository-rows.md`, the stop-gap, which repairs stores that already carry the leftovers.

On staging, 11-15, `temper ctl project delete` removed a project's repository records but left other records naming the repository. The next start refused them as missing links, and the deployment couldn't start until the leftovers were cleared.

## 1. A project's release removes every record its repositories own

- **Today:**
  - Deleting a project closes and hides it, erases its tasks and project state in pages, then asks the connectors to release its adoptions (jig `jig/crates/jig-domain-core/src/admission/deletion.rs`).
  - The forge's `adoption::release` (`temper/crates/temper-engine-domain-forge/src/adoption.rs`) deletes each released `Repository` and the client's repository row, removes the repository from kept names, and erases project-scoped settings and facts. Its history is swept in pages afterwards (`history/sweep.rs`).
  - It leaves `BranchHead`, `PullState`, `Ci`, `Tier`, `Verification`, `LandingLog`, `Receipt` and repository-bearing retained application answers. It has no sweep of the repository's changes, issues, landings, holds, proposed effects, entries, flights, subscriptions or live watches, which their own lifecycles remove only under conditions.
  - `temper/docs/design/domain/forge/adoption.md`, 1, describes the release without that list.
- **Wanted:**
  - Releasing a repository removes every record that names it, through one shared helper that every path erasing a repository uses: its durable rows and their in-memory state, its watches and topics, and its pending work, with a change's parts removed together. A context another project still adopts stays, as do deployment-wide records and the deletion's own marks.
  - The repository's removal and its bounded child erasures go in one store decision where they fit. Where they don't, the release stays paged as the project's tasks and history already are. A restart part-way leaves no row whose repository is gone: the release resumes, or each page leaves the store consistent.
  - A release repeated after a restart is harmless.
  - Revise `adoption.md`, 1, and `temper/docs/design/protocol/store.md`, 2, with their status files, to list what a release removes and what it keeps on purpose.

## 2. Out

- Stores that already carry the leftovers: `deleted-repository-rows.md`.
- Deleting anything on the external forge. A release never touches it.

## 3. Tests

Cheap: focused and fuzzy suites only; walk = false.
- The forge's adoption tests (`temper/crates/temper-engine-domain-forge/src/tests/adoption.rs`): a release removes every owned family. Another project's repository, and a shared context, stay.
- A story in temper's engine domain tests: delete a project whose repository has pull, branch, CI, tier, verification, receipt, change and issue records. Then restart from the durable store: it starts, and no row names the repository. Repeat with a restart part-way through the release, and with a repeated release.

Run only the new and affected tests, by name.
