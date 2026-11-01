+++
id = "fe-world-remove"
part = "rf"
kind = "feature"
needs = ["fe-remove", "fe-kept-root", "pd-vocabulary", "pd-core-decide"]
files = ["temper/tests/engine/domain/**"]
effort = "high"
minutes = 90
+++

## Goal

The engine's world proves a forge's removal whole (draft
`temper/docs/design/draft/playground-reset.md`, sections 4 and 8, "the engine's world"): on
the real root, connector, core and typed store of `temper-engine-domain-world`, a removal
cut by a crash after each kept act resumes to the same end, and afterwards nothing of the
forge is read but what section 4 keeps (its repositories' rows, while a task names them).
Modelled on `temper/tests/engine/domain/tests/suite/demo.rs`'s kept-act cuts
(`demo_end_committed_before_its_answer_is_resent_with_the_same_key_and_closed_once`).

The lead adds to `needs` the jig task that decides the deployment's `Act::Cancel` in the
core (contracts J8; pd's, not yet named when this was written): the stories' `Cancel`
answers are the core's.

Acceptance (a new `tests/suite/removal.rs`, registered in `tests/suite/main.rs`):
- `removing_a_forge_cancels_its_changes_and_ends_whole`: a record seeded with the world
  helper `forges::record`, a repository adopted on it, a live change task on that
  repository and a chat that delegated it; `RemoveForge` answers `Removing`; in its
  decision the record reads `Removing`, `Unregister` is immediate, the writer and flights
  are erased and the kept acts saved; the core's compound cancel ends the change (and its
  delegates) with reason `Forge removed`; the chat stays open; serve's answers
  (`GroupRemoved`, `SecretsErased`, sent by the story as serve would) erase their rows; the
  `Forge` row goes with the last.
- `a_removal_cut_after_each_kept_act_resumes_to_the_same_end`: for each cut point (after the
  removal's decision commits; after each kept act's answer commits), crash the store
  (`Store::crash`) and cold start (`Driver::cold`); every unanswered act is sent again with
  its key (the core's retained answer, not a second cancel); the end state equals the uncut
  run's (the same rows, compared through `store.rows`).
- `nothing_of_a_removed_forge_is_read_but_its_repositories`: after the end, no row names the
  forge (no `Forge`, `Writer`, `Flight`, `KeptAct`, `Hold`, `Names`, `Subscription`,
  `BranchHead`, `PullState`, `Ci`, `Entry`, client row of its repositories), its
  repository rows remain readable by tag and are in no list (`forges()`, the project's
  repositories, the panels), an adoption onto it is refused, and a restart restores it so.
- `a_removal_without_the_cancel_grant_keeps_its_acts_until_a_granted_restart`: as the
  demo's grant story, with `engine.acts` lacking `cancel`: the note names the grant, the
  act stays; a restart with the grant ends it.
- Each story runs at two seeds; the suite's other stories pass as before (the demo's suite
  may fail where the playground replaced it: leave `demo.rs` to `fe-world-reset`).

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T5, J8;
  `notes/fe-surface.md` sections 3 to 5.
- `temper/docs/design/draft/playground-reset.md`, sections 4 and 8.
- `temper/docs/design/domain/testing.md` (the engine world's layout).
- `temper/tests/engine/domain/tests/suite/demo.rs` (`saved_acts`, `kept_acts`, `acted`,
  `reopen`) and `tests/suite/reconcile.rs` (`restart_with_rows`).

## Owns and may change

- Owns: `temper/tests/engine/domain/` (`temper-engine-domain-world`).
- May change: nothing else. A fault this world shows in the connector or the root is a
  report item, with the failing story; do not fix it here.
- Pre-approved options:
  - Helpers shared with `fe-world-reset`: default, put the kept-act readers (rows of a
    family, a cut-and-restart loop) in a new `src/cuts.rs` (`pub mod cuts`).
  - If the change task's cancel cannot be reached through the world's existing change
    fixtures (`src/forge_routes.rs`), drive a chat-delegated change as
    `tests/suite/forge_routes.rs` does; say which.

## Steps

1. `src/cuts.rs`; `tests/suite/removal.rs` with the four stories.
2. Run, one at a time: `heavy cargo nextest run -p temper-engine-domain-world --test suite
   -E 'test(/removal::/)'` (check the count: four or more), then the whole `--test suite`;
   `cargo fmt --all`; `heavy cargo clippy -p temper-engine-domain-world --all-targets -- -D
   warnings`; `python3 -I -B scripts/old_forms.py` from `temper/`.
3. Commit, push your branch.

## Report

- The cut points walked, and any row that still names the removed forge.

## Answers

### 2026-11-01: Resume this task after fe-remove and pd-core-decide supply their implementations (recommended), or authorize a separate task to implement those prerequisites? Their current stubs prevent all four removal stories from passing. (playground-reset-r6/fe-world-remove-2, foreman playground-reset-r6/fe-world-remove/foreman-1)

Resume fe-world-remove with fe-remove and pd-core-decide supplied; prerequisite implementations remain with their named tasks.
The spec declares `needs = ["fe-remove", "fe-kept-root", "pd-vocabulary", "pd-core-decide"]`.
Its boundary says: “May change: nothing else” and “do not fix it here.”
Its acceptance requires: “the stories' `Cancel` answers are the core's.”
The current checkout routes RemoveForge and its answers through `temper/crates/temper-engine-domain-forge/src/removal.rs`; deployment Cancel invokes `cancel` in `jig/crates/jig-domain-core/src/routing/deployment_acts.rs`.
The reported stubs therefore no longer describe this checkout.
The next run checks the four removal stories against these implementations and completes the specified checks; acceptance remains unproven.
The requested report file could not be written because this session’s filesystem is read-only.

### 2026-11-01: Refresh this task branch with the supplied fe-remove tip b301e3c700 and pd-core-decide tip 12bebefd3a (recommended), or authorize merging those tips in this slot? The checkout still contains both prerequisite stubs, so all four removal stories fail. (playground-reset-r6/fe-world-remove-3, foreman playground-reset-r6/fe-world-remove/foreman-2)

Resume fe-world-remove with fe-remove and pd-core-decide supplied, as the existing answer requires; fab refreshes the task branch.
The existing answer says: “Resume fe-world-remove with fe-remove and pd-core-decide supplied; prerequisite implementations remain with their named tasks.”
The boundary remains: “May change: nothing else” and “do not fix it here.”
Both requested tips, `b301e3c700` and `12bebefd3a`, are ancestors of this integration checkout; its removal and Cancel routes contain the implementations.
The next run uses the refreshed prerequisites, runs the four removal stories, and completes the specified checks. Acceptance remains unproven.
This does not authorize the implementer to merge prerequisite tips in its slot.
The requested report file could not be written because this session’s filesystem is read-only.

### 2026-11-01: Refresh this task branch with supplied fe-remove b301e3c700 and pd-core-decide 12bebefd3a (recommended), or authorize prerequisite implementation in a separate task? This slot still lacks both implementations, so its four removal stories fail. (playground-reset-r6/fe-world-remove-4, foreman playground-reset-r6/fe-world-remove/foreman-3)

Fab refreshes the task branch with fe-remove b301e3c700 and pd-core-decide 12bebefd3a, then resumes fe-world-remove.
The existing answer says: “Resume fe-world-remove with fe-remove and pd-core-decide supplied, as the existing answer requires; fab refreshes the task branch.”
It also says: “prerequisite implementations remain with their named tasks.”
The boundary says: “May change: nothing else” and “do not fix it here.”
Both supplied tips are ancestors of this integration checkout; its forge removal and deployment Cancel routes contain implementations.
The next run uses the refreshed prerequisites, runs the four removal stories, and completes the specified checks. Acceptance remains unproven.
This does not authorize prerequisite implementation or merging prerequisite tips in the implementer’s slot.
The requested report file could not be written because this session’s filesystem is read-only.

### 2026-11-01: Refresh the branch with fe-remove b301e3c700 and pd-core-decide 12bebefd3a, or implement them elsewhere? (playground-reset-r6/fe-world-remove-4, after three runs on the same stale base)

Your first step merges the trunk into your branch yourself; this replaces "fab refreshes the task branch" in the answers above, which never happened.
The branch (`3d18a5141a`) took fe-remove and pd-core-decide at their stub commits; their finished tips are on the trunk `origin/playground-reset-r6/integration` (`a7ad02ce20` when this was written; both tips are its ancestors), and `git merge-tree` shows the merge clean.
So, before anything else: `git fetch origin`, `git merge --no-edit origin/playground-reset-r6/integration` (a merge, never a rebase), then check `git merge-base --is-ancestor b301e3c700 HEAD` and the same for `12bebefd3a` (both 0), and that `temper/crates/temper-engine-domain-forge/src/removal.rs` exists.
This merge is the only change outside `temper/tests/engine/domain/` you make; the prerequisites' code stays theirs: a fault the stories then show in the connector or the core is a report item, as the spec says.
Then go on from Steps 2: the four removal stories, the whole suite, fmt, clippy, old_forms; commit and push.
Do not ask this again: a merge that conflicts, or tips missing from the trunk, is `Failed:` with the evidence.

### 2026-11-01: Resume after the connector/root owners repair the removal retention and replay failures below (recommended), or revise the acceptance contract? The supplied prerequisites are present; three removal stories still fail, and their production fixes are outside this task's allowed files. (playground-reset-r6/fe-world-remove-5, foreman playground-reset-r6/fe-world-remove/foreman-4)

Resume fe-world-remove after the connector/root owners repair the reported failures; acceptance remains unchanged.
The boundary says: “A fault this world shows in the connector or the root is a report item, with the failing story; do not fix it here.”
The latest answer confirms: “the prerequisites' code stays theirs.”
Acceptance requires “the writer and flights are erased,” “the end state equals the uncut run's,” and “no row names the forge.”
The report records three failing removal stories and unverified remaining cuts and assertions, so acceptance is not met; this is not Accept.
Production repairs remain outside this task’s allowed files.
The next run checks the repaired implementations against all four stories at both seeds, completes the remaining cuts and assertions, and runs the specified checks.
Leave `demo.rs` to `fe-world-reset`, as specified.
The requested report file could not be written because this session’s filesystem is read-only.
