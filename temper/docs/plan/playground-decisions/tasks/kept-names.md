+++
id = "kept-names"
part = "kept-names"
kind = "feature"
# The same crate as playground-restart-resets (temper-engine-domain-forge), which goes first.
needs = ["playground-restart-resets"]
files = [
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/removal.rs",
  "temper/crates/temper-engine-domain-forge/src/adoption.rs",
  "temper/crates/temper-engine-domain-forge/src/reconcile.rs",
  "temper/crates/temper-engine-domain-forge/src/restore.rs",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/removal.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/reconcile.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/restore_fit.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs",
  "temper/crates/temper-engine-domain/src/route.rs",
  "temper/tests/engine/domain/tests/suite/removal.rs",
  "temper/tests/engine/domain/tests/suite/reconcile.rs",
  "temper/docs/design/domain/forge.status.md",
  "temper/docs/design/protocol/store.status.md",
]
effort = "high"
minutes = 150
+++

# A removed forge's repositories' names, kept while a kept task names them

## Goal

The user decided (`inputs/request.md`, decision 2) that a removed forge's repository rows stay
while any kept task, live or ended and not yet erased, names them, so that an ended chat that
worked on one still reads its name until the chat is erased. Today the forge connector erases a
task's `Names` row at its release (`domain.rs`, `continue_release` calls `unnamed`), so the name
goes when the chat ends. This task makes the release keep the row, reduced to its repositories,
until `forge::Event::Unnamed { task }` says the task is erased, and maps jig's `Ask::Unname` to
that event in temper's root (`notes/contracts.md`, C1). jig sending `Ask::Unname` at an erasure is
`unname-at-erasure`, which lands after this task; until then a project's deletion clears kept
rows by its release (below), and a task forgotten once read keeps its row up to the bound.

Acceptance:

- At a task's release its watches, holds and subscriptions go as today, and its `Names` row is
  saved again under `Key::Names(task)` as a kept row: one `Name { forge, repository, what:
  What::Repository }` per repository its names lie in, in name order. `Domain::names(task)` no
  longer answers it; `Domain::kept_names(task)` does.
- A kept row watches nothing, `Rename` and `Names` for its task change nothing, and
  `erase_removed_repositories` counts its repositories as named: a removed forge's repository's
  row stays while a live or kept row names it.
- `Event::Unnamed { task }` erases the task's kept row (or a live row with its watches, as today)
  and then settles every ended forge (`removal::unnamed`).
- At most `Limits::repositories` kept rows, in their own map, never in `Limits::tasks`' room. A
  release past the bound first erases, of the kept rows and the released task's, the oldest (by
  task number) each of whose repositories a task numbered after it, live or kept, names too.
- A project's release (`adoption::release`) takes its repositories out of every kept row (a row
  left empty is erased, another saved again) and settles every ended forge.
- At a start, the root's link step splits the restored `Names` rows: a task `core.tasks` holds
  is live, any other is kept. More than `tasks` live rows or `repositories` kept ones is a
  restore fit failure. Reconcile accepts a kept row whatever forges it names; every name of a
  live or kept row still needs its repository's row.
- temper's root maps `Ask::Unname { task }` for the forge connector to `forge::Event::Unnamed {
  task }`.
- In the engine's world, an ended chat on a removed forge reads its repository's name across a
  start until its project's deletion, after which no `Names` row of its tasks remains and a start
  links.
- The status entries below are gone. The tests below pass.

## Inputs

- `temper/docs/plan/playground-decisions/notes/common.md`;
  `temper/docs/plan/playground-decisions/notes/contracts.md`, C1.
- `temper/docs/design/domain/forge.md`, 4.1: "its records erased", "A task's names, kept until it
  is erased", "Their bound", "A project deleted"; 5.1, the first bullet; 16, the second
  paragraph (the `Names` rows at a start).
- `temper/docs/design/protocol/store.md`, section 2, family 4 `Names`.
- `jig/docs/design/domain/connectors.md`, 3.4 (what `Unname` means; read only).

## Owns and may change

In `temper-engine-domain-forge` (`src/`):

- `domain.rs`:
  - A field `kept: Map<u64, Box<[Name]>>` with capacity `l.repositories`. Put it after
    `releases` (line ~169) and its constructor line after `releases`' (~576). Do not touch the
    `names` field's line (~145) or the constructor block at ~583: the `reference-m8` branches
    edit both.
  - `names`' capacity (`Map::with_capacity(l.tasks)`, ~553) becomes `l.tasks + l.repositories`
    (checked), so a start can restore both kinds before the link splits them. The live bound
    stays `names()`' explicit `d.names.len() == env.limits.tasks` check.
  - `pub fn kept_names(&self, task: u64) -> Option<&[Name]>`, beside `names` (~836), doc citing
    `temper/docs/design/domain/forge.md, 4.1`.
  - `pub fn link_names(&mut self, tasks: &jig_domain_tasks::Domain, limits: &Limits)`: moves
    each `names` row whose task `tasks.task(task)` is `None` into `kept`. If `kept` would pass
    `repositories`, or the rows left live pass `tasks`, it records the first failure in
    `restore_fit` as
    `RestoreFit { family: Some(RecordKind::Names), set: RestoreSet::Rows(RecordKind::Names), .. }`
    (held, additional and limit as `restore.rs`'s `room` gives them) and moves nothing more. It
    does nothing when a fit failure is already held.
  - The release: `continue_release` (~3664) keeps the row instead of erasing it. Give `unnamed`
    a way to keep (a `keep: bool` parameter, say) and change only its tail after
    `skein_forge_client::step(..)` (`if kept(&d.client_out) && d.names.remove(&task)...`). Leave
    its watch-building loop as it is: the `reference-m8` branches insert tier watches there.
    Keeping a row: reduce it to its repositories; if `kept` is full, apply the bound's rule; then
    `Save { record: Stored::Names { task, resources } }` (or `Erase` of `Key::Names(task)` when the
    rule takes the released row itself), then `crate::removal::unnamed(d, out)` as today.
  - The dispatch (~1249-1255): `Event::Names` for a task in `kept` changes nothing and emits
    nothing. `Event::Rename` stays as it is (it reads `d.names` alone). `Event::Unnamed { task }`:
    a kept row is removed, `Erase { key: Key::Names(task) }` emitted, then
    `crate::removal::unnamed(d, out)`. Any other row goes through `unnamed` as today.
  - `erase_removed_repositories` (~5224): the `named` scan reads `d.kept` beside `d.names`.
  - `restored_link_key` (~5494, `RecordKind::Names`): finds a kept row's key too.
  - Restore (~4949): unchanged (rows go into `names`; the link splits them).
- `adoption.rs`, `release` (~538): after erasing the project's repositories, remove those
  repositories' names from every kept row. A row left empty is erased (`Erase Key::Names`), a
  changed one saved again. Then call `crate::removal::unnamed(d, out)` before
  `ProjectReleased`.
- `removal.rs`: `unnamed`'s doc may say "a task's erased or released names". Nothing else
  unless the compiler asks.
- `reconcile.rs`:
  - `restore_references` for `Stored::Names` (~168): only each name's repository (`self.name`);
    the task need goes.
  - `capacity` (~62): the `Names` term sized for `tasks + repositories` rows, each
    `resources_per_task` names.
  - `names_forge` goes if nothing reads it after `restored_link_key`'s change.
  - The comment at ~315 cites `temper/docs/design/domain/forge.md, 4.1` alone (section 20's
    question is gone).
  - Leave line ~72-74 (`capacity`'s tail) and the `RecordKind` enum's end alone: the
    `reference-m8` branches edit them.
- `restore.rs`: `derived` (~289) stays on `d.names`, which holds live rows once linked. The
  `Names` room (~201) follows `names`' capacity.
- `boundary.rs` docs only: `Stored::Names` (~936) "A live task's resources, or a kept task's
  repositories (temper/docs/design/domain/forge.md, 4.1)". `Event::Rename` (~1297) "a released
  task's kept row stays as it is". `Event::Unnamed` (~1302) "The task is erased: drop what the
  connector keeps of its names".

In `temper-engine-domain`:

- `src/route/forge_route.rs`, `effect_ask` (~3889): take `Ask::Unname { task }` out of the
  `unreachable!("core effect handoff")` arm into its own arm: `if number ==
  domain.config.forge_connector { domain.work.push(Work::Forge(forge::Event::Unnamed { task })) }`.
  `Ask::Names` stays where it is.
- `src/route.rs`, `Work::ForgeRestored` (~374): call
  `domain.forge.link_names(&domain.core.tasks, &env.limits.forge)` first, before `fit_restored`.

Reader tests (each changed only as said):

- `temper-engine-domain-forge/src/tests/reconcile.rs`:
  - `owner_rows` (~115) drops its `Names` row: a kept task's row needs no live owner. A new
    test, `a_kept_tasks_names_resolve_without_their_task`, takes its place.
  - `a_mixed_resource_set_still_requires_its_live_owner` (~315) asserts the opposite now. Rename
    it `a_kept_row_resolves_whatever_forges_it_names` and assert `resolved(.., false)`.
  - `production_link_asserts_a_missing_task` (~421) uses a `Stored::Call` of `TASK` in place of
    the `Names` row (`restore_fit.rs`'s `row(RecordKind::Call, ..)` builds one). Expected:
    `RecordKind::Call`, its `Key::Call`, `Need::Task { task: TASK, retired: None }`.
  - `retained_reconciliation_references_fit_their_heap_ceiling` (~256): if its figure or row
    count moves with `capacity`, refit it as it documents.
- `temper-engine-domain-forge/src/tests/restore_fit.rs`:
  - `owned_tables_fit_at_limit_replace_and_refuse_one_more` (~134): `(K::Names, LIMITS.tasks +
    LIMITS.repositories)`. Do not touch line ~129: `reference-m8` edits it.
  - `missing_links_scan_retained_keys_after_lower_insertions_and_replacements` (~636): a `Names`
    row's unmade link is now its repository. Give `unlinked_row`'s `Names` a name on an absent
    repository and set its expected row as the scan gives it. If that duplicates `Hold`'s case,
    drop `Names` from the list instead and say so.
- `temper/tests/engine/domain/tests/suite/reconcile.rs`, `a_missing_task_asserts_before_the_switch`
  (~51): a `forge::Stored::Call` of `ABSENT_TASK`; expected `MissingLink { kind:
  RecordKind::Call, key: forge::Key::Call(..), need: forge::Need::Task { task: ABSENT_TASK,
  retired: None } }`. If the forge's `Call` payload types are not public, use
  `forge::Stored::Release` and say so.
- Tests that send `Unnamed` for a live task as its release keep passing unchanged:
  `tests/release.rs:9-40`, `tests.rs:643`, `route/forge_route/tests.rs:748`, the forge world's
  `fuzzy/simulation.rs:49-66` and the protocol world's `protocol.rs:968-978`. Change none of them.
  If one fails, the `Unnamed` live-row path changed: fix the code, not the test.

Pre-approved options:

- The bound's rule may be computed by a scan of `names` and `kept` at the release (both bounded).
- If a memory or heap-ceiling test's figure moves because of `kept`, refit it as that test
  documents and say so.

Not this task's: jig sending `Ask::Unname` (`unname-at-erasure`), and `people::Ask::RemoveProject`'s
`grace` field (`remove-project-grace`). Your story reuses `erase_chat` and writes no
`RemoveProject` literal of its own; if the trunk you start from has that field and `erase_chat`'s
literal (~277) lacks it, add `grace: None` there and nothing else, and say so.

## Steps

1. Forge, unit tests first, in `src/tests/removal.rs` beside
   `a_repository_named_by_a_kept_task_stays_until_that_task_is_erased` (~362), using its
   helpers (`kept`, `remove`, `adopted`, `branch`, `REPO`):

   ```rust
   #[test]
   fn a_released_task_keeps_its_repositories_until_it_is_erased() {
       // Names { task: 7, [branch()] }; Release task 7 (as a_removed_changes_open_pull_... drives
       // SettleEffects then Release): Save Names { 7, [REPO's Repository name] } is sent, no
       // Erase of Key::Names(7); d.names(7) == None; d.kept_names(7) == Some(&[repository name]).
       // remove forge 1, SecretsErased, BoxRemoved: the repository's row and the record stay.
       // Rename { 7, [branch()] } and Names { 7, [branch()] } change nothing (no output).
       // Unnamed { 7 }: Erase Key::Names(7), Erase Key::Repository(REPO), the record erased.
   }
   #[test]
   fn past_the_bound_the_oldest_covered_kept_row_goes_first() {
       // LIMITS.repositories == 2 kept rows. Tasks 7, 8, 9 each name branch() and are released
       // in turn: at 9's release task 7's row is erased (8 and 9 name REPO after it), 8's and
       // 9's stay. A row nobody later covers is never the one erased.
   }
   #[test]
   fn a_projects_release_takes_its_repositories_out_of_kept_rows() {
       // Task 7 released (kept row on REPO); forge 1 removed and whole but for its repository;
       // ReleaseProject { project: 5 }: Erase Key::Names(7), the repository erased, the record
       // erased, then ProjectReleased.
   }
   ```

   Then the restore split, in `src/tests/reconcile.rs`, with its `empty_tasks()`:

   ```rust
   #[test]
   fn a_start_keeps_the_names_of_tasks_jig_does_not_hold() {
       // REPO adopted; restore Names rows of two absent tasks on it; link_names(&empty_tasks(), ..):
       // names() answers neither, kept_names() both; fit_restored Ok; reconcile does not panic.
       // LIMITS.repositories + 1 such rows: link_names records the Rows(Names) fit failure, and
       // fit_restored returns it.
   }
   ```

   A live task's row staying live: if a `jig_domain_tasks::Domain` holding one live task is
   simple to build here (restoring its live row as the core does), assert it in this test.
   Otherwise the engine world's restart stories cover it (a live change's watches across a
   start); say which.

   `heavy cargo nextest run -p temper-engine-domain-forge --lib`.
2. The root: the `Ask::Unname` arm and the link call. A test beside
   `a_change_unnamed_after_its_pull_swap_was_queued_stays_unnamed` (route/forge_route/tests.rs
   ~748; not at the file's end, where `reference-m8` appends):
   `an_erased_tasks_unname_reaches_the_forge_as_unnamed` calls `effect_ask` with
   `Ask::Unname { task: 7 }` for the forge connector and finds `Work::Forge(forge::Event::Unnamed
   { task: 7 })` on `domain.work`. `heavy cargo nextest run -p temper-engine-domain --lib -E
   'test(unname)'`.
3. The engine's world, `temper/tests/engine/domain/tests/suite/removal.rs`, after
   `a_removed_forges_number_is_never_given_again`, with this file's helpers (`fixture`,
   `application`, `removed`, `end_change`, `stop_cancelled_runs`, `crash`, `erase_chat`,
   `assert_whole`, `forge_rows`, `session`). Factor the panel watch out of
   `assert_chat_keeps_name` into `assert_name_reads(d, change)` if you need it, keeping that
   helper's assertions as they are:

   ```rust
   #[test]
   fn an_ended_chat_reads_its_removed_forges_repository_until_its_project_is_deleted() {
       let (mut d, chat, change) = fixture();
       application(&mut d, 201, forge::Ask::RemoveForge { forge: FORGE }, false);
       d.pump();
       removed(&mut d, forge::Event::SecretsErased { forge: FORGE });
       removed(&mut d, forge::Event::BoxRemoved { forge: FORGE });
       end_change(&mut d, change);
       // The person closes the chat: people::Ask::Cancel { project: 1, task: chat, reason,
       // grace: None } as erase_chat sends its ask, then stop_cancelled_runs; the chat is
       // ended (Key::Ended), not erased.
       // The change's panel reads "org/repo"; the change's Names row is kept and names the
       // repository alone (What::Repository); the chat's is kept too unless the bound took it.
       crash(&mut d, 71);
       // Still reads "org/repo" after the start.
       erase_chat(&mut d, chat);
       assert_whole(&d);
       // No forge::Stored::Names of chat or change remains.
       crash(&mut d, 7);
   }
   ```

   Run each ask's reply as this file's helpers already do. If closing the chat in this fixture
   needs another step (a grace's fire, a run's answer), drive it as `erase_chat` does and say so.
   `heavy cargo nextest run -p temper-engine-domain-world --test suite -E 'test(removal)'`.
4. Status, as `notes/common.md` says:
   - `temper/docs/design/domain/forge.status.md`, **Not yet**: remove the whole entry `4.1, 5.1,
     16: a kept task's names: ...` (from `  - 4.1, 5.1, 16:` through
     `(\`temper-engine-domain-forge/src/reconcile.rs\`).`). Change nothing else but the date.
   - `temper/docs/design/protocol/store.status.md`: from the **Not yet** line, remove `family 4
     \`Names\` holds live tasks' rows alone, a task's row erased at its release
     (\`../domain/forge.md\`, 4.1); ` and nothing else, so the line reads `- **Not yet:** 2:
     family 20 \`Setting\` ...`.
   - The citations check (`notes/common.md`).
5. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`. This changes
   what `Event::Unnamed`, a release and a restored `Names` row mean, so run the focused tests of
   `temper-engine-domain-forge`, `temper-engine-domain`, `temper-engine-domain-world`,
   `temper-engine-forge-world`, `temper-engine-protocol-forge-world` and `temper-service`
   (`--lib`). Commit the code and tests, then the status text; push.

## Report

As the implementer's text says. Also: how you built the bound's rule, any reader test whose
change went beyond the list above, and whether the chat's close in the world story needed more
than the sketch.
