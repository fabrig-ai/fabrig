# reference-m2: contracts

The shapes that cross this plan's tasks, as its binding calls. A task builds its half to this
note and needs no other task for it, except where its spec's `needs` says so. Each entry
names the design that holds it in full; the design wins on detail, this note on the names
and words. Entries are added as batches are released; a released entry never changes.

## C1. `change`'s `branch` member

Design: `temper/docs/design/domain/agent.md`, 4.3.1 (`branch`).

- JSON: `"branch": "mine"`, its only value, beside `words`, `repository`, `name`,
  `landing` and `after`, in a single change's object and in each object of `changes`.
- Rust: `temper_engine_domain::ChangeBranch`, in `temper-engine-domain/src/boundary.rs`
  beside `ChangeLanding`:

  ```rust
  /// A change's branch choice (domain/agent.md, 4.3.1).
  #[derive(Clone, Copy, PartialEq, Eq, Debug)]
  pub enum ChangeBranch {
      /// Its requester's held branch: the goal branch it holds in the change's repository.
      Mine,
  }
  ```

  and `ChangeCall.branch: Option<ChangeBranch>`, after `landing`; `ChangeCall::words`
  sets it `None`.
- The decoder (`temper-engine-smith/src/calls.rs`, `fn change`): absent, `None`; the string
  `mine`, `Some(ChangeBranch::Mine)`; any other string, empty included,
  `Problem::Range(b"branch")`; a value that is not a string, the problem `text(b"branch")`
  gives.
- The plan's `change` declaration (`PLAN_CHANGE`) gains, after `landing`:
  `"branch":{"type":"string","enum":["mine"],"description":"mine: your own goal branch, landed whole as it stands, with no coder; left out, a branch of the change's own."}`.
  The stock declaration (a chat's, without the control family) does not.
- A plan's member: `change_batch` (`temper-engine-domain/src/route/forge_route.rs`) counts
  `branch.is_some()` in `plan`, and the refusal's words become exactly
  `change's landing, branch, name, after and changes are a plan's: this task's authority lacks the control family (amend, cancel, release).`

## C2. g's deletion as its goal closes

Design: `temper/docs/design/domain/forge.md`, section 11 and 11.1;
`temper/docs/design/domain/after-landing.md`, section 6.

- **Landed it** means: the forge holds a change row (`ChangeRow`) in g's repository whose
  `branch` is g's name and whose change state has landed (`Landed`, `Red`, `Cleared` or
  `Uncleared`), and whose landed head, the head it merged at, equals g's last known tip
  (`Domain::branch_head` of g). Equal heads mean g's tip is in the landing branch through
  that merge; a later move of g, never landed, keeps it.
- **On the goal's release** (`continue_release`, the goal branch arm): landed, the release
  deletes g as it deletes any branch (`enqueue_release` with `Write::DeleteBranch`); not
  landed, or with no tip known, it keeps g as today. Whatever the goal's ending (done,
  failed, cancelled), the rule is the same.
- No stored field, no new read, no new effect.

## C3. The procedure's fact: the change's branch is its requester's

Design: `temper/docs/design/domain/forge.md`, 8.1 and 8.5.

- `temper_engine_domain_forge_change::Facts.requester_branch: bool`, after `writer_taken`:

  ```rust
  /// Its branch is its requester's held branch (domain/forge.md, 8.1): a move with no
  /// drift and no writer in the slot is its new head, checked again (8.5).
  pub requester_branch: bool,
  ```

- **The procedure** (task `mine-procedure`): with it true, a fresh head other than the
  change's own, at opening, checking, gating, queued, first or ready, with no drift and the
  writer slot free, is the change's new head, taken as a repair's pushed head is; never
  `BranchMoved`. False, as today.
- **The forge** (task `mine-forge`) sets it in `change_facts`: true when the forge's hold of
  the change's branch (`Domain::hold` of its name) is another task's than the change's own;
  false when the change holds it or no hold is known. A change on its requester's held
  branch never holds it (C4), so this is the change of parameter 11.

## C4. The root's `mine`: a change on its requester's held branch

Design: `temper/docs/design/domain/agent.md`, 4.3.1; `temper/docs/design/domain/forge.md`, 8.1.

- **Resolved** in `change_batch` as `change_landing`'s `Goal` arm resolves `"goal"`: the name
  `tree_branch(repository, root, b'g', task, None, name_bytes)`, `task` the calling run's
  task, accepted when `domain.forge.hold(&name)` is held by that task.
- **Refused** (`ChangeRefused::Words`, the whole call, as `"goal"`'s refusal), in this order,
  with exactly these words (`OWNER/NAME` the repository's name as `repository_name` gives it,
  `N` a task number in decimal):
  1. `landing` is `Goal` and `branch` is `Mine`:
     `branch mine with landing goal: a branch cannot land into itself.`
  2. the task holds no goal branch in the repository:
     `branch mine: this task holds no goal branch in OWNER/NAME.`
  3. a live change is on that branch already, made by an earlier call (a forge change row
     in that repository whose `branch` is g's name, whose task jig still has live):
     `branch mine: change task N is on it already.`; or an earlier change of the same call
     names `branch` too: `branch mine: an earlier change of this call is on it.`
- **The delegate:** as any change's (`chat_change`), its spec's parameters 1 (the
  repository) and 2 (the landing branch) as today, then `Bytes { name: 3, value: <g's name> }`
  (the branch, the parameter a handed branch uses) and `Number { name: 11, value: 1 }`, new:
  **parameter 11, the branch is its requester's held branch**. No other parameter, grant or
  authority differs from a change's with its own branch.
- **No hold:** `implicit_task_holdings` gives no write holding for a change carrying
  parameter 11 (jig would hand g down from its goal); `start_change` accepts parameter 11
  and emits no `forge::Event::Hold` for the branch, still naming branch and base
  (`forge::Event::Names`). Every other match over a change's parameters in `forge_route.rs`
  that refuses unknown numbers accepts 11.
- **No producer:** the branch has a head (g is made at its base's tip), so the procedure
  goes from producing to opening at once, as for a handed branch.

## C5. The forge's side of a change on its requester's held branch

Design: `temper/docs/design/domain/forge.md`, 8.1 and 8.5.

- **Drift:** a drift the forge finds on a held branch (`drift`, `Request::Drift`) is its
  holder's, as today, and also each live change row's whose `branch` is that name and whose
  task is not the holder: its `ChangeRow.drift` is set as the holder's would be, and a
  `Request::Drift` names its task too, which the root routes to its procedure as for any
  change (`forge_route.rs`), so it holds for the drift.
- **Its own update:** a pending update or prepare entry of such a change counts for the
  branch as the holder's own does (`updating_entry`), so the update it asked for is never
  drift.
- **Its delegates' writes:** `workspace_item`'s holder for its repair and its resolution is
  the branch's holder (the forge's hold of `row.branch`), so `run_workspace` admits their
  push under the requester's hold; a change that holds its branch is as today.
- **Facts:** C3's `requester_branch`, as C3 says.
