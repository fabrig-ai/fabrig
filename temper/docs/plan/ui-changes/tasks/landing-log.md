+++
id = "landing-log"
part = "landing-log"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-protocol-store/**",
  "temper/crates/temper-engine-domain-forge/src/landing_log.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/landing_log.rs",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/limits.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/reconcile.rs",
  "temper/crates/temper-engine-domain-forge/src/restore.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/restore_fit.rs",
  "temper/crates/temper-service/src/limits.rs",
  "temper/crates/temper-service/src/limits/names.rs",
  "temper/docs/help/reference/limits.md",
]
effort = "high"
minutes = 100
+++

# `LandingLog`, family 30, a small version

## Goal

Every adopted repository keeps a ring of its last `landings_kept` landings, written in the merge's own
decision: the commit, the branch, the engine's wall time, and each change's task, pull and title, so
that a landing's title outlives its change's close. `changes-view` reads it. Moves from outside temper,
trailers and tier verdicts stay out.

Acceptance:

- `Stored::LandingLog` is one row per repository, saved in the decision of a merge's outcome.
- A change that sees its pull merged (someone else's merge, the forge's own merge queue) is logged the
  same, in the decision of the step that moved it to `Landed`.
- A commit logged already gains the change, within `landing_members`, and makes no second landing.
- The ring drops its oldest past `landings_kept`, which `small` and `standard` set to 16.
- A title is fitted to `LOGGED_TITLE_BYTES` at a character's boundary, with `crate::text::fit`.
- The log is erased wherever its repository is, and restored at the restart step; a restored log naming
  no repository is what a restored `Verification` naming none is.
- The tests below pass, and bite.

## Inputs

- `temper/docs/plan/ui-changes/notes/common.md`; `temper/docs/plan/ui-changes/notes/contracts.md`, *The store*
  and *The limits*.
- `temper/docs/design/domain/after-landing.md`, 10, *The landing log* and *Limits*.
- `temper/docs/design/protocol/store.md`, section 2, byte 30, and section 4.
- **The model:** family 29 (`Verification`) for a repository's row; `git show 0dfd2cdb22 --stat` for every
  file a new family touches.

## Owns and may change

Owns `files`. May also change:

- **Arms and lines a new family needs,** each edited only for that:
  `temper/crates/temper-fake/src/check.rs`; `temper/tests/codecs/tests/suite/fuzzy/store.rs`;
  `temper/tests/engine/forge/src/world.rs`; `temper/tests/engine/protocol-forge/src/protocol.rs`;
  `temper/crates/temper-engine-domain-forge/src/tests.rs` (its `mod`);
  `temper/crates/temper-service/src/tests/startup.rs` (a limit's literal);
  `temper/crates/temper-protocol-store/src/tests/*`, owned already.
- **The limit's name:** `temper/docs/help/reference/limits.md`, `temper/docs/examples/reference/*` where
  they list the forge's limits, and any test holding the names to them.
- **Design status, its closed lines alone:** `temper/docs/design/protocol/store.status.md` (family 30 goes
  from *Not yet*, but its trailers and verdicts); `temper/docs/design/domain/after-landing.status.md`, 10
  (no family and no `landings_kept` go; no trailers, moves from outside or verdicts in the log stay, with
  `trailers` and `trailer_bytes`); `temper/docs/design/STATUS.md`'s line on the landing log.

Pre-approved:

- If the store's largest-value check (`protocol/store.md`, 4, *Sizes*) trips on the new row, raise nothing:
  check the bounds handed it come from the forge's limits, not the schema's ceilings; else say so in the
  report and shrink `LoggedLanding.branch` to the bound the limits give.
- A merge's outcome for a change in release writes the log all the same: the merge happened. `verification::landed`
  is skipped there; the log is not.
- A `pull: None` is written as read: a merge with no pull number is still logged.

## Steps

1. **Store:** the schema and regeneration (`notes/common.md`, *Codecs*); `keys.rs` (`LANDING_LOG`, byte 30,
   both ways); `records.rs`, `records/landing_log.rs`; `src/tests/{keys,records,fixtures}.rs`. Limits made
   from the forge's: `landings_kept`, `landing_members`, `name_bytes`.
2. **Domain:** `boundary.rs`, `limits.rs` (with `valid`), `landing_log.rs`:

   ```rust
   /// Log `task`'s landing at `commit`, in the decision that saw it.
   pub(crate) fn landed(d: &mut Domain, now: Wall, commit: Commit, task: u64, out: &mut Queue<Request>);
   pub(crate) fn erased(d: &mut Domain, repository: Repository, out: &mut Queue<Request>);
   pub(crate) fn row_bytes(row: &LandingLog) -> Option<u64>;
   ```

   Call `landed` beside `crate::verification::landed` in `domain.rs`'s `Made::Merged` arm, outside its
   `releases` guard, and in `step_change` where a row's `landed_commit` goes from `None` to `Some`. Call
   `erased` where `Key::Repository` is erased: `Event::ForgetAdoption` and `erase_removed_repositories`.
   Add the family to `reconcile.rs` (`RecordKind`, as `Verification`), `restore.rs`, `lib.rs` (`key`,
   `row_bytes`) and `domain.rs`'s restore and removal's keys. **Size:** `Limits.landings_kept` bounds the
   row; a bigger restored ring is refused as a too-big `Verification` is.
3. **Service:** `forge.landings_kept` in `limits.rs` (16) and `limits/names.rs`; the reference.
4. **Tests,** `src/tests/landing_log.rs`, driving the domain's events as `tests/verification.rs` does:
   - `a_merge_logs_its_change_with_its_title`: the saved row's commit, branch, wall, task, pull, title.
   - `the_log_outlives_the_changes_close`: after the change's end and its row's release, `landing_log`
     still says the title.
   - `a_pull_seen_merged_is_logged`.
   - `the_ring_keeps_the_last_landings_kept`: the seventeenth drops the first, and saves one row.
   - `a_commit_logged_already_gains_the_change`, then a change past `landing_members` is dropped.
   - `a_long_title_is_fitted_at_a_character`.
   - `its_repositorys_removal_erases_the_log`, by both paths.
   - `restore_fit.rs`: the family's row in its tables, and its limit's refusal.
5. **Check:** the narrow tests, then `heavy cargo nextest run -p temper-protocol-store -p
   temper-engine-domain-forge`, `-p temper-codecs-world --test suite -E 'test(drift::)'`, `-p temper-service
   --lib`, `-p temper-docs`; `heavy cargo check -p temper-fake -p temper-engine-forge-world -p
   temper-engine-domain-world -p temper-engine-protocol-forge-world --tests`; fmt, old forms, clippy; the citations
   (`python3 -I -B tools/citations/citations.py --base origin/main`).
6. Commit and push.

## Report

As the implementer's text has it, with the row's worst size in bytes under `small`, and the rule each new
test bit on.
