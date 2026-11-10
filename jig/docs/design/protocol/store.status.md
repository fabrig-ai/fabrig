# store: status

2026-11-10. Where jig's code lags or departs from [store.md](store.md).

Section 5.1's tasks-v9 terminal includes a bounded failed-activation reason.
Its schema, generated codec, goldens and layout fingerprint change together
under section 5's pre-release rule. The carried-terminal round trip retains
the cause alongside its class.

Section 5.1's tasks-v9 failed ends retain every optional invalid-result
kind through the store codec. The generated layout fingerprint and golden
bytes agree with the schema; no gap is known in this field.

Section 5 has no known gap in stored charter parts, task reasons and
claimed offered kinds, or `later` in task, people and template choices.
Their codecs and bounded restore checks carry these fields in the
existing family versions, under the pre-release format rule.

Sections 4.1 and 7.2 have no known gap in settings key order:
`Setting` orders Application, ProjectPolicy and Model as their stored
tags 4, 5 and 6 do. Stored bytes are unchanged. The cursor and key
tables cover these boundaries, and the key generator includes settings.
The consuming application's startup table walks an application setting
and a model setting one row at a time on open and after a crash.
This proves the order defect; the reported cancellation's cursor and
range are unknown, so it does not establish that incident's exact state.

Sections 4.4, 7.2 and 15.9 have no known gap in facts' key tag 5, their
bounded codec, restoration after the accounts and sources, or removal
with the account. The accounts family changes in place under section 5.

Section 4.4 has no known implementation gap in account list records,
model first-read times or the provider-list revision decider. The list
record and its catalogue decision share the commit and restart boundary.

Sections 5 and 5.1 have no known gap in loading model-setting names before
family restoration and taking catalogue updates before their commit, so a
person's stored model retains its number across restart.
Section 14's seven whole generation, patch, check and frame sweeps are
under `fuzzy::`, with the recovery bodies shared by their focused cuts.
The focused stray-cleanup cut's mismatch with `../domain/testing.md`, 9
is recorded in `../domain/testing.status.md`; it does not remove a fuzzy
cut or a recovery check. Whole-suite budget proof remains open there.

Section 14's core world on the simulated disk has no known gap in a
decision's commit and the restart after it: `jig-test-store-files`
runs the core world's store on the simulated disk without patch sets
and kills it at a chosen file operation, counted from a commit's
submission or from the store's open
(`a_kill_before_the_first_operation_loses_the_commit_and_past_the_last_keeps_it`).
Killed at each operation of a decision making a task and the person
task linking a person, and of the decisions saving a person and a task
naming it, then again during the restart and in the next decision, the
start opens, never refused, with the last acknowledged decision or that
and one whole uncertain one (`jig/tests/core`'s
`faults::a_kill_at_each_file_operation_of_a_decision_starts_from_a_whole_one`
and its fuzzy restart sweep). The store's world's random crash table
alternates a person's commit and a task's naming it, killed twice in a
row (`fuzzy::crashes::random_commits_and_crashes_keep_the_referees_promises`).

Section 8.3 has no known gap in a window's turn past its page's room.
A `Window` read answers it alone and cut, as `Event::Turns` with its
fields, its transcript empty, `cut` its transcript's length, and `next`
past its frame or none at the window's end. A page already holding a
turn ends before it. A cut opening turn is still checked as the
window's opening turn, and no window read answers `Unread` for a turn's
size. A window that starts at another turn, and a frame unread or
refused, still answer `Unread`
(`jig-protocol-store`'s `window_pages_cut_oversized_turns_alone_and_advance`,
`a_nonempty_prefix_continues_before_a_turn_that_is_cut` and
`direct_reads_without_room_are_unread_except_windows_cut_with_byte_room_zero`).
The room is the whole frame's charge (8.6). A current frame past it is
cut from its head, its body never read. A frame behind is read whole,
patched, then cut from the patched record
(`frame_patches.rs`'s `patched_frame_sizes_bound_resumes_turns_pages_and_tails`;
`jig/tests/store`'s
`upgrades::frames::patched_frames_past_bounds_match_current_boundaries`
and `upgrades::upgrade_edges`, whose `native_pages_without_room_end_as_unread`
keeps its name while its window reads expect the cut).

- **Not yet:** 7.3, 11.1: retention: no horizon shrinks an ended task to its summary or drops its transcript; a page of history, spend or notes cut before its first row. 10.5, 14: rehearsing and going back: `jig/crates/jig-protocol-store` has no `Request::Rehearse`, `Request::Back`, `Event::Back` or `BackRefusal`, and `jig/tests/store` no story of them. 14: the reference binary killed around each phase of a turn's commit (`jig/testing/jig-test/tests/process.rs` kills and restarts it once); `jig/tests/ops` and `jig/testing/jig-conformance` run `jig-fake-store`, where no world fakes the store.
- **Departs:** 14: the core's world on the simulated disk kills no
  snapshot racing commits and no group of several commits in flight:
  those rows run in the store's world
  (`jig/tests/store`'s `crashes::snapshots_racing_linked_commits_and_several_commits_in_flight_keep_whole_prefixes`),
  a person and a task naming it queued below the component and swept
  over every file operation, with and without a snapshot's map chunk
  pending, whose referee checks a whole prefix and the task's requester
  present, not the start's link pass, because the plan
  `store-crash-safe`'s cut judged the core world to reach neither
  cheaply: a snapshot waits for the log past its bound (skein's
  `kv.md`, 5), and a group for commits queued below the component.
- **Departs:** 14: `jig/tests/store/src/model.rs` reads a copy as a settings revision's and panics on any other row, because the core copies only that one, so no story reaches it; `jig/crates/jig-protocol-store/src/tests/commits.rs` holds a copy of another row and section 6's two rules of a copy.
