+++
id = "bx-story-setup-cut"
part = "pg"
kind = "feature"
needs = ["bx-story-playground", "fe-remove", "fe-make", "sv-registry-setup"]
files = ["temper/crates/temper/tests/reset/**"]
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
