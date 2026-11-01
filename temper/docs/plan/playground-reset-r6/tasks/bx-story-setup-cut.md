+++
id = "bx-story-setup-cut"
part = "pg"
kind = "feature"
needs = ["bx-story-playground", "fe-remove", "fe-make", "sv-registry-setup"]
files = [
  "temper/crates/temper/tests/reset/**",
  "temper/crates/temper-service/src/setup.rs",
  "temper/crates/temper-service/src/route.rs",
  "temper/crates/temper-service/src/route/box_link.rs",
  "temper/crates/temper-service/src/tests/registry_setup.rs",
]
effort = "high"
minutes = 110
+++

## Goal

Draft section 8's third process story: an owned Forgejo's set up killed at each step,
found unfinished after the restart, and started over (draft 6.4): `forge remove N`, then
`forge make here` and a set up of the new number, which runs Forgejo from the cache's
binaries without fetching them again.

Acceptance:
- `tests/reset/setup_cut.rs` (a `mod` line in `tests/reset/mod.rs`), one test per cut,
  sharing one function:
  - `an_owned_set_up_killed_at_its_fetch_starts_over_from_the_cache`,
  - `an_owned_set_up_killed_at_its_preparation_starts_over_from_the_cache`,
  - `an_owned_set_up_killed_with_forgejo_running_starts_over_from_the_cache`,
  - `an_owned_set_up_killed_before_its_first_repository_starts_over_from_the_cache`.
  Each: a fresh scratch and `temper init`, the cache seeded with the pinned binaries
  (`support::forge_land::cache_forgejo_binaries`, as
  `a_first_run_plays_the_demo_then_sets_up_a_forgejo_here_with_no_restart` does), a ctl story
  with stderr echo (`support::boxed::echo_steps`): sign in; `forge make here` → N;
  `forge setup N` started without waiting (`Step::Start`); `KillBox` once the box's stderr
  shows the step (`Step::Stderr` on the group's `step N fetch`/`prepare`/`run` line as the
  box prints it, or for the last cut, the forge reading connected with its first repository
  not yet made); `StartBox`; `forge list --json` reads forge N unfinished (kept, not
  connected with its first repository); `forge remove N` → `removing`, then `forge list`
  until N is gone, and on disk no `box/groups/N`, `box/started/N` or `forge-N`; `forge make
  here` → M (M > N, numbers never reused); `forge setup M` until connected with its first
  repository; the cached binaries unchanged
  (`support::forge_land::assert_cached_binary_unchanged`) and no fetch of them (the fetch
  step's log names the cached digest, or no download reached the mirror); the box exits 0
  with nothing left.
- Run each alone:
  `heavy cargo nextest run -p temper --profile process-temper --test process -E 'test(=reset::setup_cut::NAME)'`.
  Each must fit the process profile's budget: measure and report each test's time.

## Inputs

- Draft `temper/docs/design/draft/playground-reset.md`, sections 4, 6.4 and 8.
- `temper/docs/design/protocol/setup.md`, 6.6 (the cache), 6.7 (the first repository), 7.1
  (the pipe's step lines).
- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T3, T4, T5.
- `temper/crates/temper/tests/support/forge_land.rs` (`cache_forgejo_binaries`,
  `forgejo_setup_steps`, `assert_cached_binary_unchanged`); bx-harness's report.

## Owns and may change

- Owns: `temper/crates/temper/tests/reset/setup_cut.rs` and its `mod` line.
- May change: `temper/crates/temper/tests/support/forges.rs` and `support/forge_land.rs`,
  adding helpers (no change to an existing one's behaviour).
- Options, pre-approved:
  - If a cut point is passed too fast to cut reliably (the fetch with a seeded cache), cut
    at the next line the box prints for that group and name the test for where it cut; say
    so.
  - If the four together pass the process tier's budget, keep the fetch and the running
    cuts in the process tier and move the other two to `qa_all` as `qa_` tests; report the
    times that decided it.

## Steps

1. The shared function and the four cases; `heavy cargo check -p temper --tests`.
2. Run each alone; read a failure's log before anything reruns.
3. `cargo fmt`; `heavy cargo clippy -p temper --all-targets`. Commit and push.

## Report

Each case's cut point as it landed, its time, and how the cache was shown unused.

## Answers

2026-11-01. Q: the box prints no `step N …` lines on stderr and the lifecycle makes the first
repository before it connects: have other tasks add cut points, or widen this one?

A: Neither; no box, service or ctl change. The pipe's step lines already reach the forge's
row: `forge list --json` reads `"state":"setting_up","step":"fetch"|"prepare"|"run"`
(`temper-client-tools/src/forges.rs`, the row's `state`; serve sets it from `step N …`,
`temper-service/src/tests/box_link.rs`). After `temper init` names no forge, forge N is the
list's only row, so each cut is a `Step::Until(forge list --json, BYTES)` then `KillBox`:
- **fetch:** `BYTES` = `"state":"setting_up"` (any step, the first read once the set up is
  asked). A seeded fetch is too fast to wait for `"step":"fetch"` alone, which could hang;
  this is the pre-approved fallback: report the step read at the kill (none, fetch or
  prepare).
- **prepare:** `"step":"prepare"` (it hashes two releases). If your runs miss it, the
  pre-approved fallback applies: report it, with the reads that missed it.
- **Forgejo running:** `"step":"run"`, then `Step::Stderr` on `forgejo N: ` (echo steps) so
  Forgejo has printed before the kill; N is known from `forge make here` (1 on a fresh
  init; assert it, as `Step::Stderr` takes static bytes).
- **before its first repository:** revised. Connected with the first repository not made
  is not a state the lifecycle reaches (Create comes before connect,
  `temper-service/src/setup/lifecycle.rs`), so cut on `"state":"connecting"` after the run
  step: reconcile, Origin and Create under way. Report the row's `first.state` at the kill
  (`making`, or `made` if Create won the race); either leaves the forge unfinished (draft
  6.3: not connected with its first repository adopted), so the rest of the case holds.
- **No fetch of the binaries:** `assert_cached_binary_unchanged` for both, and the box's
  stderr after `StartBox` has no line starting `fetch ` or containing `download`, as
  `assert_forgejo_setup_after_sign_in` checks (`support/forge_land.rs`); no mirror.

2026-11-01. Q: `forge make here` makes forge 1, but `forge setup 1` is refused `not offered`
(the row reads `not_connected`): owner's repair, or widen? A: Widen, in this task: its
acceptance cannot hold without it, and the fault is one condition. A record registered
after the restore gets its `Local` with `pending: false`, so nothing ever sets its
connection; `setup.rs::register` sets `NotSetUp` only for `record.playground`. Make that
condition the `Local`'s trigger instead: after the insert, when
`engine.setup.lifecycle.startup_restored()` and the trigger is `Trigger::StartDemo` or
`Trigger::StartAsked`, `lifecycle::state(.., Connection::NotSetUp)`, as `local_stopped`
reads a `StartAsked` forge not prepared. A managed entry's `Start` and a forge registered
at restore (pending, started by `start_pending`) are unchanged. Its lowest-tier check: in
`tests/registry_setup.rs`' `a_registered_owned_record_is_served_and_waits_for_its_group`,
assert the `Connection { forge: 1, state: NotSetUp }` event as the fake record's test does,
and that `forge.offer(1)` is `Offered` (the fixture is under the box) or, if the fixture is
not, `NotUnderTheBox`; run `heavy cargo nextest run -p temper-service --lib -E
'test(/tests::registry_setup::/)'`, then the whole `-p temper-service --lib`. No other
service, box or ctl change: a second blocker after this one is a question again. Then run
the four stories as the spec says.

2026-11-01. Q: `forge setup N` on an owned forge fails `no settings`: the root's `SetUp` asks the
box `start N` with no `box/groups/N` written; widen, or the setup owner's? A: Widen, in this
task, one arm: its acceptance cannot hold without it, and contracts T3 already says serve
writes `box/groups/N` before every `start N`; the `SetUp` arm in `route.rs` skips it while
`SetUpGroup` (`route/box_link.rs::set_up_group`) does it. In `route.rs`'s
`root::Request::SetUp` arm, keep `lifecycle::first`, then take the record's entry
(`engine.records.get(&forge)` and its `setup`, cloned) and call
`box_link::set_up_group(engine, forge, request, &entry)`; with no record or no entry, the
empty entry, which asks directly as today. Guard `set_up_group`'s
`group_starts.insert(..).expect(..)` for a second set up while the write is pending: if
`group_starts` already holds the forge, answer it `Busy` (`setup::lifecycle::set_up_refused`)
instead of panicking; that guard may go in `route/box_link.rs`, in this function only. Its
lowest-tier check, in `tests/registry_setup.rs` (no new test file): after
`a_registered_owned_record_is_served_and_waits_for_its_group`'s registration, route
`root::Request::SetUp { forge: 1, .. }` and assert `take_group_write()` names forge 1 with
`group_settings(OWNED, 1)`, the box link's `write` none until `group_written(1, Ok(()))`,
then `start 1\n`. Run `-p temper-service --lib -E 'test(/tests::registry_setup::/)'`, then
`tests::box_link::`, then the four stories. The `names::client_crates_carry_the_client_prefix`
failure (`temper-github-app`) is not this task's: report it under `## Left`. A third blocker
outside these files is a question again.


2026-11-01, after run 4. Q: four runs without landing; what is left? A: Only the merge. Run 4
met the acceptance (the four stories pass alone, within budget, pushed at `f0afa6877`), but
its branch merged a trunk from before `0544f90a5` (`mod reset_cut;`), so it still conflicts in
`tests/reset/mod.rs`. The next run does nothing else:
- Merge the trunk's tip into the branch (`git merge origin/playground-reset-r6/integration`,
  no rebase, no force-push) and resolve `reset/mod.rs` as the Rebase section below says.
- `sv-startup-refuse` was answered the same `Request::SetUp` → `box_link::set_up_group`
  repair in `route.rs`. If the trunk already holds it when you merge, keep the trunk's
  `route.rs` and `route/box_link.rs`, add this branch's `Busy` guard only if the trunk lacks
  one, and keep this branch's `registry_setup.rs` assertions where they still hold; one
  write and one `start N`, never two.
- Then `heavy cargo check -p temper -p temper-service --tests`; on a service conflict,
  `-p temper-service --lib -E 'test(/tests::registry_setup::/)'`; the four stories alone
  once each, and `reset::reset_cut`'s tests once alone to show the two modules coexist.
  Commit the merge, push the branch.
- The run-4 `## Left` items stay left: they are other tasks' or the trunk's end check's.
  No new work, measurement or report table beyond what this step reruns.

## Rebase (2026-11-01, step merge, foreman playground-reset-r6/bx-story-setup-cut/foreman-4)

resolve reset/mod.rs by retaining both reset_cut and setup_cut module declarations.
The merge failed in `temper/crates/temper/tests/reset/mod.rs`: both branches append a module after `mod playground;`.
Integration commit `0544f90a562c46c96f8c71188a5a8e9133000386` adds `mod reset_cut;`; this branch adds `mod setup_cut;`.
Resolve the conflict by keeping the module documentation followed by:
```rust
mod playground;
mod reset_cut;
mod setup_cut;
```
Both stories are required by `temper/docs/design/draft/playground-reset.md`, section 8.
No test-body or service change is needed for this refusal.
