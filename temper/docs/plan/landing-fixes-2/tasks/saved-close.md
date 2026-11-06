+++
id = "saved-close"
part = "saved-close"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/saved_close.rs",
  "temper/crates/temper-engine-domain/src/route/asks.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/saved_close.rs",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 90
+++

# A task's saved work goes as its task closes

## Goal

The request's item 2. `temper/1/s10` and `s14` were left on the forge after their goal. `temper/docs/design/domain/forge.md`, section 11 says a task's saved work is deleted as its task closes. From reading, `temper/<root>/s<task>` is never a hold, and a close releases holds alone (`continue_release`), so nothing deletes it. Reproduce that first, and say if it's wrong.

The way in, which adds no stored field: as its task's release begins, each branch of the task's saved set becomes a hold of the closing task. `continue_release` then deletes it through the path a change's branch already takes. `ReleaseRow` is stored, and it stays as it is.

Acceptance:

- A task closing `Done` or `Cancelled` with a saved tag deletes `<prefix><root>/s<task>` in that repository, with `Write::DeleteBranch`, before `Released`.
- A branch already gone counts as deleted, and the close goes on (`skein/docs/design/forge-client.md`, 7: the client answers it made, so `release_outcome` needs nothing new).
- A task closing `Failed` that isn't its tree's root keeps the branch, as a hold moved to the root. It is deleted as the root closes. A failed root deletes at once.
- A task with no saved tag issues no `DeleteBranch` for an `s` branch, and takes no such hold. Closing costs it nothing.
- Asking twice takes one hold. A name another task holds is left alone.
- With the forge's holds full at the close, the branch stays and the close completes. It is never `ReleaseFailed` for this.
- A restart between the hold and the delete still deletes: the hold is stored, and the release row resumes as it does today.
- No stored form, wire form or `ReleaseRow` field changes. If this seems to need one, stop: that's a question.
- The `11:` entry is gone from `temper/docs/design/domain/forge.status.md`.

## Inputs

- `temper/docs/design/domain/forge.md`, section 11, **Deleted as their tasks close**, and the paragraph beginning `A task's saved work is named by the set`.
- `temper/docs/design/domain/worker.md`, section 5, **Saved work**.
- jig's `jig/docs/design/domain/tasks.md`, 6.5.
- `temper/docs/plan/landing-fixes-2/notes/common.md`.

## Owns and may change

Owns the files in the header. Both `saved_close.rs` are new.

Reader and neighbour files:

- `temper/crates/temper-engine-domain-forge/src/tests.rs` and `temper/tests/engine/domain/tests/suite/forge_routes.rs`: the `mod saved_close;` line.
- `temper/tests/engine/domain/tests/suite/forge_routes/saved_retry.rs`: `saved_repair_retry` and the helpers your story needs may become `pub(super)`. Nothing else there changes.
- A test whose exact effects at a close now hold a `DeleteBranch` for a saved branch, because it saved work and closed: that test's expected list gains the row, and nothing else. Say which ones in your report.
- A compiler-forced arm for the new `forge::Event` variant, in any temper file.

Other drafts: `resume-told/clean-merge` and `repair-cause` also delete lines from `forge.status.md`, each its own. `repair-cause` changes other functions of `route/forge_route.rs`. Stay out of `temper-worker-domain/src/workspace.rs` and `temper-service/src/translate.rs`, where `resume-told/saved-kept` is working.

Departs from the implementer's text: delete from `temper/docs/design/domain/forge.status.md` the `Not yet` entry beginning `11: a task's saved work is not deleted as it closes` and its continuation lines.

Sketches of the outer layer:

```rust
// temper-engine-domain-forge/src/boundary.rs, in Event
/// A closing task's saved work: each branch becomes its hold before the
/// release that follows (domain/forge.md, section 11).
SavedWork { task: u64, branches: Box<[Name]> },

// temper-engine-domain/src/route/forge_route.rs
/// A closing task's saved-work branches, from its record's saved set.
pub(crate) fn saved_branches(domain: &Domain, env: &Env<Limits>, task: u64, root: u64) -> Box<[forge::Name]>

// temper-engine-domain/src/route/asks.rs, core::Ask::Release, before the push of Event::Release
domain.work.push(Work::Forge(forge::Event::SavedWork { task, branches }));
```

`saved_branches` reads `domain.core.tasks.task(task)`'s `saved`, through `saved_tags` and `forge::untag`, then `domain.forge.repository`, `tree_branch(repository, root, b's', task, None, name_bytes)` and `branch_what`. A repository no longer adopted is left out. It pushes no event for an empty set.

Options and traps, each pre-approved:

- Don't call `hold()`: it starts a goal's branch and reports `Refused` to the task. Insert the `Hold { name, task, writer: None, drift: None }` directly, after `valid_name`.
- `save_hold` asks `read_afresh` for a hold with no writer. Don't use it here: nothing needs this branch's head, and the delete is next. Default: `emit(out, Request::Save { record: Stored::Hold(row), release: None, read_afresh: None })`.
- A retained hold reaches `tasks::Event::Retained`, whose hub finds no such holding and does nothing (`jig-domain-tasks/src/holds.rs`, `retained`). That is right: the forge's hold alone keeps it. If you find it holds the task instead, stop: that's a question.
- Read the other loops over `d.holds` in `domain.rs` (lines 1016, 2355, 2395, 3853, 4248, 5790, 6081, 6245) and `goal_branch.rs`. A hold with no writer and no known head should pass all of them. Report any that needs care.
- If an oracle or restore check in `temper/tests/engine/domain/src/oracle/` holds that every forge hold has its hub holding, stop: that's a question, since it would change this way in.
- No room in `d.holds`: leave that name out, as section 11 says.

## Steps

1. **Reproduce at the domain tier first.** In `forge_routes/saved_close.rs`, take `saved_repair_retry()`, finish the retry and land the change, then assert that the fake forge has no `temper/<root>/s<task>` and that the task closed. It fails today, on the branch still there, which proves the request's cause. Add `a_task_with_no_saved_work_deletes_no_saved_branch`.
2. **The forge's rules.** In `temper-engine-domain-forge/src/tests/saved_close.rs`, modelled on `tests/verification.rs`, `deleted_branch`, add:
   - `a_closing_tasks_saved_work_is_deleted`
   - `a_saved_branch_already_gone_counts_as_deleted`
   - `a_failed_tasks_saved_work_stays_with_its_root_until_it_closes`
   - `a_failed_roots_saved_work_is_deleted_at_once`
   - `a_second_asking_takes_no_second_hold`
   - `a_name_another_task_holds_is_left_alone`
   - `full_holds_leave_the_branch_and_the_close_completes`
3. **A restart:** take a stored hold and release row back through the existing restore path, and see the delete go out. `saved_retry.rs`'s world or `tests/verification.rs` may have the nearest form. Say which you used.
4. **Fix**, by the sketches.
5. Break each rule once: skip the `Failed` branch of retention, or the room check, and see its test fail. Restore.
6. **A change of meaning runs its readers' tests,** since every close with a saved tag now has a new effect: `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-domain-world -p temper-engine-forge-world -p temper-system-world`, then `heavy cargo nextest run -p temper --test suite -E 'test(forge_land)'`. Read each failure before you edit its expected list: a test that closes a saved task and expected no effect takes the row, and anything else is a red to report.
7. `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`. Commit and push.

## Report

The implementer's report, plus: whether the request's cause held; each test whose expected effects changed; what the other `d.holds` loops needed; and the size of one more `Stored::Hold` in a release's batch, if any bound came close.
