# Session 09: the protocol world

Move the five existing fake-protocol tests into skein (09.1). The additional owner, stories and
invariants (09.2–09.4) move to the GitHub draft, section17 (ruling189). Existing coverage remains
with temper until its preserved move/deletion steps transfer it. Needs at start:
skein 08.1's Joint/fixtures, skein 07.1's client-world LIMITS/translation, temper 03.1's users on the
component, temper 05.1's users on the fakes and skein 06.2's transcripts/deliveries, landed on origin/main.

## Read first

- skein `docs/design/forge-connection.md` 4.1, 6–8, 10; `forge-client.md` 2.1, 8, 10; `fake-forge.md` 4, 8; `forge.md`
  3.7, 5.4.
- skein `docs/foundation/programming-model.md` 4.2–4.4, 5.2–5.4, 6.3, 7, 9, 10.2; `testing-strategy.md`
  2.4–2.5, 3, 4.4, 6–8, 10.
- README, code-organization.md 1, 3, 5–8, decisions.md 11, 18, 22, 23, 25 and Design questions36; later.md;
  findings/{survey-03-connection,survey-06-tests,trace-connection,
  trace-fakes,base-skein-03,base-skein-07,base-skein-08}.md. The last is this session's checked inventory;
  earlier drafts' 32-test partition is obsolete.

## What the design asks

- Connection 6, 8: authenticated hints after responses; each phase's deadline; Close drops outstanding work,
  then closed once and last after io settles.
- Connection 10: client/component stories and invariants over the fake's bytes; skein08 owns the
  machine world's retained plaintext/TLS byte faults; new independent measurement is deferred.
- Fake 4, 8: its existing HTTP/domain boundary, observations and tally; testing-strategy 8: preserve moved
  coverage and understand slow tests.

## Choices and rules that bind it

Copy temper's origin/main at each move's start, naming the source commit. Files and counts below are **at
the base**; preserve additional tests landed before copying. No rewrite of a moved part; only the
home-forced changes listed in Shape. Count file pairs by code-organization 6. A moved file over 800 lines is
split by its next changing increment, never by the move. No `temper` in skein code, comments, fixtures or
values; use `kit`, `kit/0.0.0`, `b"kit"`; no serde, alias or re-export of a moved type. Each world is a hand-listed
Cargo.toml member. No optimized test builds, raised budgets or filters hiding moved tests. New units <20 ms,
focused world tests <0.5 s; new focused work aims at ≤1 s per world, fuzzy ≤5 s. Decision 11 preserves the
four inherited 7.481–7.709 s sweeps in 08.1. All Sizes here include tests. The additional cost invariant is deferred with09.4; no new protocol
owner/story/invariant test runs in this plan.

## Increments

### 09.1 The five movable fake-protocol tests
Scope: decision 22's component/fake portion, at tests/forge/protocol.
Needs: the session's start capabilities, including skein 08.1's Joint/fixtures and skein 07.1's
  translate::commit (used by both job tests).
Files: temper tests/engine/protocol-forge/tests/fake_protocol.rs -> skein
  tests/forge/protocol/tests/fake_protocol.rs; new Cargo.toml/src/lib.rs; skein Cargo.toml (explicit
  member/dependencies), Cargo.lock. Manifest dependencies: skein-forge, skein-lib,
  skein-forge-machine-world (path ../machine), skein-forge-client-world (path ../client),
  skein-fake-forge-domain (raw operations for direct), skein-fake-forge-protocol (Route).
Shape: set `license.workspace = true` in the new Cargo.toml's `[package]`, beside its other
  workspace-inherited fields (README, Workspace license inheritance).
  Package skein-forge-protocol-world; import Joint directly from skein_forge_machine_world::joint, api
  from skein-forge, and commit from skein_forge_client_world::translate. No copied driver or translation.
  Retain Joint::new/failing(seed: u64) -> Self, call(&mut self,u64,Repository,Op), answered(&self,u64) ->
  Option<&Answered>, skein_forge_machine_world::joint::REPO ({forge:1,repository:2}),
  and Joint::direct (BUILD_STEP_NAME/BUILD_STEP_LOG stay private consts of the moved fake_protocol.rs); never fixtures::REPO or the client world's REPO.
Test shape: 5 moved tests, names unchanged, no new test:
  the_component_starts_and_lists_items_through_the_fakes_bytes;
  pull_for_answers_the_open_pull_when_the_lookup_names_a_closed_one;
  opening_with_nothing_to_merge_writes_no_pull_request;
  a_failed_job_read_names_its_failed_step_and_reads_its_log;
  a_failed_jobs_log_end_comes_through_a_suffix_range. Preserve fixture limits and assertions: first list
  cost2/1 item/4 requests; lookup cost4; empty pull NothingToMerge/cost4/no write; jobs cost3, step1/"Run
  build", suffix b"build 3\n". Recorded base rerun: 5/0.010 s (findings/base-skein-08.md).
Proof:
joint; `cargo nextest run -p skein-forge-protocol-world`: 5 passed, each <0.5 s; base form --test fake_protocol, 5/0.010 s lighter-load rerun. This proves the moved HTTP/domain joint; no socket behavior changes.
Don't: move or delete these 10 temper-policy tests:
```text
a_change_produced_opened_checked_queued_and_landed
a_created_issue_with_its_answer_lost_is_found_after_restart
a_merge_with_its_answer_lost_is_found_after_restart_without_merging_twice
a_failed_ci_head_is_repaired_then_checked_again
an_external_move_of_a_held_branch_is_reported_as_drift
idle_forge_call_cost_does_not_follow_old_issue_history
honest_costs_equal_the_requests_the_fake_served
a_busy_pass_lists_at_most_changed_items_over_the_page_plus_one
the_api_token_reaches_no_record_request_or_fact
stories_through_cut_streams_answer_each_call_once
```
  Temper retains protocol.rs, writer_settings/protocol_shared/goal_view and its owner-valued
  settings/shared/token_absent fixtures, plus engine-forge's LIMITS/fake_config for its other worlds. It
  uses skein's Pipe/Joint and shared fake_limits; temper 07 deletes only moved files/tests and repairs
  imports/manifests, preserving protocol.rs's access to 08.1's public seams.
Size: about 113 lines moved, 40 new including tests: 30 manifest/module lines, 10 import/path edits.
Removes: nothing in temper; its owner-policy world remains after temper 07.

### 09.2 The owner, token maps and restart — dropped

Dropped by the scope cut, ruling 189 (2026-10-20). Moved to the GitHub draft, section 17 (ruling 189).
No implementation or proof of this increment runs in forge-kit; its earlier specification remains
in git history. Every retained move and its inherited coverage still runs.

### 09.3 The protocol stories — dropped

Dropped by the scope cut, ruling 189 (2026-10-20). Moved to the GitHub draft, section 17 (ruling 189).
No implementation or proof of this increment runs in forge-kit; its earlier specification remains
in git history. Every retained move and its inherited coverage still runs.

### 09.4 The three protocol invariants — dropped

Dropped by the scope cut, ruling 189 (2026-10-20). Moved to the GitHub draft, section 17 (ruling 189).
No implementation or proof of this increment runs in forge-kit; its earlier specification remains
in git history. Every retained move and its inherited coverage still runs.

## Questions to raise when reached

A new sweep over its time aim: report the failing case, time and cause, never raise a budget. Decision18
allows +1.5 s default/+6 s fuzzy for the whole plan; 08+09's new bounds are 2 s/10 s, exceeding them by
0.5 s/4 s before moves. Report actual sums alongside 08's inherited max7.709 s/summed30.369 s four sweeps
and README's whole-suite measurements. Answer36 is settled, not a pending design question.

## Done when

From skein, after retained09.1. Preserve the five moved tests and their assertions; no new protocol
focused or fuzzy test is required here. Report moved timings separately against the existing whole-plan
aims. The owner/stories/invariants are to the GitHub draft, section 17 (ruling 189).

- `cargo nextest run -p skein-forge-protocol-world` prints 5 passed, each <0.5s.
- No fuzzy run is required for this five-test moved package.
- `cargo clippy -p skein-forge-protocol-world --all-targets -- -D warnings` prints Finished without warnings.
- `rg -n '^    "tests/forge/protocol",$' Cargo.toml` prints exactly one member line.
- `rg -n -i temper tests/forge/protocol` prints nothing (rg exit1); README Done when5 covers the whole kit.
