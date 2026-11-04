# Spot review of debt-4's skein-kv and jig D028 changes (2026-11-04, at 8317051e57)

Read-only reviewer, using the user's quality criteria. Read from source; nothing was run, so the
mutant claims and line savings come from reading. Line numbers are at `8317051e57`.

## Part A: skein-kv

Verdict: clearly better than before and D017 is met. State and data are one value, module-wide
`#![expect]`s went from 5 to 0, and `Serving`, `Repair`, `Compaction` and `Queued` are good primitives.
Size is only partly earned: 1,193 lines became 1,670, against the task's 1,250 cap. About 200 lines
are removable without loss.

1. **Risk: `Queued::charges` can grow without bound.** `skein-kv/src/map.rs:33`, `:41`; `store.rs:1150`.
   `clear` runs only when the queue is empty at `batch_synced`. Under sustained load every batch has a
   successor, so it never runs. Erased keys stay as `None` charges, each holding a copy of the key, and
   no budget counts them, so put-and-erase churn on distinct keys grows the heap until traffic pauses.
   Fix: tag each charge with the number of the last commit that touched it; add
   `Queued::applied(ops, number)` in `batch_synced`, dropping keys tagged at or below `number` (a miss
   then falls back to the map, which is correct by then). About +10 lines; `if queue.is_empty()` goes.
2. **Bug, pre-existing: an in-flight snapshot's file handle is never closed.** `store.rs:324`, reached
   from `:854`, `:861`, `:880`. A log write, sync or rollover-close failure during compaction reaches
   `recover`, which sets `compaction = Idle` and drops `Header`, `Chunk`, `Trailer`, `Sync` or
   `Close { file }` with no `Close`. It breaks 5.4's rule on handles and kv.md 4's "settles its I/O".
   Fix: `Compaction::file(&self) -> Option<Token>`, shared with `failed`'s or-pattern at
   `compaction.rs:342`; close it before recovering; add an `io_error` mid-snapshot test asserting
   `open_files() == 0`. About +12 lines.
3. Four copies of the "everything but Ready" list (`store.rs:574`, `:700`, `:1251`, `:1281`). One
   exhaustive `Phase::ready(&self) -> Option<Log>`. About -50.
4. One `#[expect]` costs about 40 lines (`store.rs:46`): rustfmt breaks every `Phase` variant. Put it
   on `enum Phase`, or make `bytes` a `Box<[u8]>`, which also removes the expects at `:402`, `:1021`,
   `:1054`. About -40.
5. `Queued::check` is a 45-line near-copy of `KvMap::check`, and `commit_disk` checks twice (`map.rs:61`,
   `:173`; `store.rs:618-633`). `KvMap::check` delegates to `Queued::new().check(self, ops, limits)`;
   use `has_later` in `apply`; in `commit_disk`, check, then the byte test, then a reserve taking the
   projected result. About -43. The seeded test stays.
6. Forwarding handlers (`store.rs:941`, `:1017`, `:1105`, `:1176`; `compaction.rs:279-329`, `:331`).
   `compaction::follow` is exactly `start_batch`: delete it. Pass `step` to `recover`, `next_log` and
   `start_recovery`, and the four store wrappers go. Leave the eight relabelling handlers (`closed`,
   `renamed` and so on) alone: 5.4 asks for one handler per cell.
7. 17 identical three-line `unreachable!` arms in `up` (`store.rs:781-898`). One `fn wrong_answer() -> !`.
   About -34.
8. Recovery leftovers: `snapshot_read` and `log_read` share an `if match` plus `.expect`, and four sites
   build the same `ReadAt` (a `read_end` and a `read_next` helper, about -20); `log_opened` takes `ReadAt`
   but `at` is always 0; `snapshot_read` uses `next` for two meanings; `compaction.rs:96` sends a
   `Remove` from a cell that cannot happen (should be `unreachable!`); the dedup at `compaction.rs:226`
   exists only because `finish_recovery` keeps the tail name.

## Part B: jig scripted engine (D028)

Verdict: the fake's production code is clearer and 44 lines smaller. `Phase::class`, the single
`count`, and removing `inbox_order` and `closing.rs` are right. The stories add 354 net and are the weak
part: two of the five new stories cannot fail on the rule they name.

1. **Gap: the `Changed` story cannot see `Changed`.** `jig/tests/client/src/stories/shared/b_goals.rs:44`,
   `:72`; `jig/testing/jig-client-scripted-engine/src/queries.rs:71`. Goal 1 is the last changed, so
   the expected `[1, 2]` equals the number tie-break: replacing the `Changed` arm with `Ordering::Equal`
   stays green. Fix: seed goal 2 with a later `changed`, assert `[2, 1]` before the move and `[1, 2]`
   after. About +8.
2. **Gap: the delegate record story pins nothing specific.** `b_held.rs:62-64`, `:205-206`. `REQUESTER`,
   `PARENT` and the decider are all 1, so the old `requester: by` passes, and so does recording the
   parent task's number. Yet it costs 150 lines, and `testing.status.md` has no entry saying the rule is
   held only by a unit test on the fake. Fix: make the parent task number differ from the person (seed
   a chat first, or run as party 3) and record the `by` gap in the status file. If that costs too much,
   delete the story and `Facts::DelegateEscalation` (about -165), keep the unit test, and record that.
3. `b_inbox::scripted` is a false loop (`b_inbox.rs:64-148`): two helpers, `need` and `update`, plus two
   flat lists. About -25.
4. Fixture boilerplate (`b_inbox.rs`, `b_held.rs:69-147`, `b_scenarios.rs:46`): a general
   `install(front, key, value)` and `inbox_source(front, party, group, rows)`. About -35, and -30 more if
   old sites adopt them. Move `b_held`'s inline `use`s and mid-file constants to the top.
5. `b_config::setup` gained an `#[expect(clippy::too_many_lines)]` (`b_config.rs:6`, `:94`, `:112-117`):
   extract `scripts(facts)`, with `b_inbox::scripts()` replacing the two public script functions, and put
   the limit overrides in the `InboxOrder` case. About -8; `CHAT_SIGN_INS` duplicates a literal at `:69`.
6. `requests.rs:313-316` re-derives what the match above knew: the `waiting` match computes standing
   directly as `Option<bool>`, mirroring the core's `escalation_standing`.
7. `Phase::Ended.class()` exists to satisfy a grep (`queries.rs:31`, `:44`): write `PhaseClass::Ended`.
8. Tree-read copies (`b_conversations.rs:140`, `:170`, `:230`): a `chat_order(world, client)` helper
   beside `goal_order`, about -14. `a_person_s_chats_are_their_own` asserts its result twice: one goes.
