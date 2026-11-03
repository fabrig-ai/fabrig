+++
id = "goal-forge-branch"
part = "goal-forge"
kind = "feature"
needs = ["goal-jig-surface", "change-requires", "change-held-release", "change-by-rule", "change-batch"]
files = ["temper/docs/design/domain/forge.status.md", "temper/crates/temper-engine-domain-forge/src/resources.rs", "temper/crates/temper-engine-domain-forge/src/items.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/goal_branch.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/tests/goal_branch.rs", "temper/crates/temper-engine-domain-forge/src/tests.rs", "temper/crates/temper-engine-domain/src/route/asks.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/holds.rs"]
effort = "high"
minutes = 110
+++

# goal-forge-branch: the forge holds a goal's own branch, makes it at a taking, keeps it

## Goal

temper's forge connector treats a goal's own branch g, `<prefix><root>/g<task>` (contract
C1), as the goal's: an exclusive hold that refuses; made at the home's default branch's tip
as the goal's hold is taken, as the taking's effect (jig's `connectors.md`, 4.7); a making
that fails for good holds the goal, saying why; kept as the goal closes, whatever way it
ends. Naming g from a template's symbol, and refusing a change from outside the goal's
subtree, are `goal-forge-symbol`'s.

Acceptance:

- **Recognised:** `items.rs` gains `pub(crate) fn goal_branch(name: &Name, prefix: &[u8]) ->
  Option<(u64, u64)>`, the root and the task of a branch `<prefix><root>/g<task>` (decimal,
  no leading zeros, nothing after), `None` for any other name; unit tests for `c`, `r`, `s`,
  `l` and `g` names and malformed ones.
- **Its hold refuses:** `resources.rs` gains `pub const GOAL_BRANCH_HOLD: HoldKind =
  HoldKind::Exclusive { wait: false }`, and `resource_facts` gives it, `Access::Owned`, for a
  goal branch under the repository's prefix; every other own branch keeps `BRANCH_HOLD`.
- **Made at the taking:** `Event::Hold` gains `entry: u64` (the root passes
  `jig_domain_core::Ask::Hold`'s `entry` in `temper-engine-domain/src/route/asks.rs`). When
  the held resource is a goal branch, `hold` (in `domain.rs`, or a new `goal_branch.rs` it
  calls) reads the branch; found, nothing more is done; missing, it reads the repository's
  default branch's tip (`settings.default_branch`) and creates the branch there as an outbox
  effect of the task keyed by `entry` (`skein_forge::api::Write::CreateBranch { branch,
  commit }`, kept with `keep_entry` as a release's effect is, its condition
  `Condition::None`). Any other resource's taking does nothing new.
- **A making that fails for good holds the goal:** the forge emits a new `Request::MakeFailed
  { task: u64, resource: Name, reason: <the forge client's failure, as the release's failure
  carries it> }`; the root (`forge_route.rs`, where `Request::Taken` and
  `Request::LostReadFailed` are handled) holds the task with `holds::hold_as(domain, task,
  tasks::Hold::EffectFailed, &words)`, the words from a new `holds::make_words` naming the
  branch, the repository and what the forge answered (in the style of `holds::lost_words`).
- **Read only once made:** a change step reading its base (`StepStage::Base`) whose base is a
  goal branch with its making still in flight waits for the making's outcome, then steps
  again, instead of emitting `Request::Refused`.
- **Restart:** a held goal branch is a live resource and is read afresh as any (forge.md,
  section 16); one found missing while its goal holds it and no making is in flight is made
  again as at a taking, under a fresh entry from the restored outbox's numbering if the
  forge has one, else held as a making that failed (say which in the report).
- **Kept at release:** `continue_release` releases a goal branch's hold row without
  `Write::DeleteBranch` and without `Request::Retained`, whatever the ending.
- Tests in `src/tests/goal_branch.rs` (a new module of `src/tests.rs`, in the style of
  `tests.rs`'s `a_hold_and_writer_slot_are_exclusive_until_answered` and
  `a_release_waits_for_its_lost_writers_reread_even_after_restart`):
  `a_goal_branch_hold_is_exclusive_and_refusing`,
  `taking_a_missing_goal_branch_makes_it_at_the_default_branchs_tip`,
  `taking_an_existing_goal_branch_makes_nothing`,
  `a_goal_branch_making_that_fails_for_good_says_why`,
  `a_change_into_a_goal_branch_being_made_reads_it_once_made`,
  `a_closing_goal_keeps_its_branch_whatever_way_it_ended`, and a restart case,
  `a_goal_branch_found_missing_after_restart_is_made_again`.
- In `temper/docs/design/domain/forge.status.md`, the **Not yet** sub-bullet "2, 11, 11.1: a
  goal's own branch held by its goal: ..." goes; the "2, 11.1: the symbol ..." sub-bullet is
  `goal-forge-symbol`'s, and the "11, 11.1: ... deleted ... once a change has landed it"
  sub-bullet stays.

## Inputs

- `temper/docs/design/domain/forge.md`, section 2 ("Holds"), section 11 (the `g` branch;
  "Landing branches are never made by a push"; "Deleted as their tasks close") and 11.1
  (the bullets "Held by its goal for its whole life" and "Landed whole").
- jig's `jig/docs/design/domain/connectors.md`, 3.3 and 4.7; `jig/docs/design/domain/
  tasks.md`, 6.2 ("A taking has its effects") and 6.5.
- `temper/docs/design/domain/forge.status.md`: the sub-bullet named above.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C1 and C3.

## Owns and may change

- Owns the files in the header (in `forge.status.md`, the one sub-bullet above). This plan's
  change family changes `temper-engine-domain-forge/src/domain.rs` (its step stages, for a
  landing branch's rule) and `forge_route.rs` too: start from their merged work
  (`change-requires`, `change-held-release`, `change-by-rule`, `change-batch`: the lead
  orders this task after them so one task writes each file at a time), keep your hunks to holds, takings, releases and
  the base read's wait, and if `StepStage::Base` has moved, apply the wait where the base is
  read now. `temper-engine-domain-forge` and `temper-engine-domain`'s forge routes are
  changed on plan reference-m8's trunk: keep each hunk to the goal branch.
- **Readers:** every match of the forge's `Event::Hold` and `Request` in temper (the root's
  `asks.rs` and `forge_route.rs`, the forge's tests and `temper/tests/engine/forge/**`
  fixtures that build `Event::Hold`), each given the field or the arm.
- **Options:** if the default branch's tip is already in the working set (`branch_head`), use
  it rather than a read, and say so. If a making needs a stored row of its own to survive a
  restart (a new record in `temper-protocol-store`), stop and ask: that is a format change
  this spec does not name. If the base read in `StepStage::Base` cannot wait without a new
  pending state, refuse the step as a resource wait (`Request::Taken`-style words naming the
  goal branch being made) and say so.
- **Not yours:** resolving the symbol and refusing a change from outside the goal's subtree
  (`goal-forge-symbol`), the jig core.

## Steps

1. Read `domain.rs`'s `hold` (near line 1883), `continue_release` and `enqueue_release`
   (near 3580 and 3750), `StepStage::Base` (near 2664), and the root's handling of
   `forge::Request::Taken` (`forge_route.rs`, near 3203) and `holds.rs`.
2. Build the pieces in the order above, each with its test:
   `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/tests::goal_branch::/)'`.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-forge -p temper-engine-domain --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-engine-domain-forge -p temper-engine-domain`;
   `heavy cargo nextest run -p temper-engine-forge-world`.
4. Commit ("forge: a goal's own branch, held refusing, made at its taking, kept"), push.

## Report

How a making is keyed and read, what a restart does, the tests and counts, and the options
taken.
