# Session 04: the tools' waits

`wait` follows a subject's watch: its digest (04.1), checked conditions (04.2),
delivery and timeout (04.3), then watch endings and the ended task's read (04.4).
It provides **the tools' waits**. Needs at start: **the tools' root**. Steps are
`temper-04.1` through `temper-04.4`; all are step code in `temper-client-tools`'s
`jig` module. Unit proofs connect through the next plan's client stories and
real-loop stories, once those drivers exist.

## Read first

- temper `docs/design/client/tools.md`: 4, 5.2, 5.4, 6.1 to 6.4, 8.3, 11.1 to 11.3, 12;
  `docs/design/domain/testing.md`: 2.4. Use client/tools.md at `fe0ec316`: answers 21 to 24
  are in 5.4 and 6.3; corrected 29 and 33 govern the surrounding output.
- jig `docs/design/client/domain/client.md`: 8.1 (Dropped is not ended), 8.3,
  19 (`facts`); `docs/design/client/domain/contract.md`: 5.1, 6, 7.1 to 7.3;
  `docs/design/domain/tasks.md`: 3 (what a task waits on covers its tree).
- Code in jig: `crates/jig-client/src/facts.rs`, `chat_status` and `reader`;
  `crates/jig-client-domain/src/{boundary,watches}.rs`, `Why` and `ended`;
  `src/contract/{reads,queries,documents,owner_inbox,owner_tree}.rs` below that crate;
  `crates/jig-domain-core/src/routing/watch_marks.rs`, `visibility_end`.
- This plan's `temper/{02-tools-table,03-tools-root}.md`;
  `findings/temper-area-1.md`, 3 and 4; the plan's own decisions 5 and 7.
  Read `decisions.md`'s design questions 21 to 34 whole.
- skein `docs/design/lib.md`, 6.2 (`skein_lib::sha256`);
  foundation `programming-model.md`, 3, 5.1, 9; `testing-strategy.md`, 2.1, 8, 10.

## What the design asks

- **5.4, the digest:** first 8 bytes of SHA-256, printed as 16 hex digits, of the
  watch subject's JSON document without `digest`. A read and a wait use one form:
  for inbox/tasks, the first window's rows in order and whether more follow,
  within the tools' 32 rows. A read's cursor prints outside that form.
- **5.4, which reads:** `me`, unfiltered `inbox`, `tasks` with watch arguments,
  `task N`, `run N`; `task N --tree --run` gives three separate digests.
  `--activity`'s lines, `chat`, `activity` and `turn` have none. A subject takes
  only what its watch takes: inbox no flags; tasks neither `--after` nor `--ended`.
- **5.4, conditions:** me: waiting; inbox: waiting/updates; task: ended,
  waiting-on-me, held, replied, or a live phase; tree/run: ended alone.
  Tasks has no condition. Terminal phase words are usage errors, exit 1,
  naming `--until ended`; temper 02.3 parses and checks them.
- **5.4, answers:** since compares the first delivery too; without since or
  until, wait for a change after the first delivery. Until may meet on the first
  delivery; both require a differing digest and a met condition. Deliveries
  coalesce. Ended may be shown by a delivery or the subject's watch end;
  `task N` then answers as a read of the ended task, saying how it ended.
- **5.4, 6.3:** met exits 0; timeout or subject end with the condition unmet
  exits 3, printing the current or last document and digest. Subject no longer
  visible exits 2. Sign-out or a client failure exits 1. Stopping ends no watch:
  jig reopens it; a link down past the next plan's host deadline exits 1.
- **6.1, 6.2, 11.1, 12:** text ends the part with `digest: task <hex>`;
  JSON is one snake-case object. Test each condition's hits and misses and
  digest stability. Timeout defaults to 300 s, capped at 3600 s.

## Choices this plan makes

- **One owner of syntax.** Import temper 02.3's `jig::subject::{Subject, Condition}`;
  `subject::{condition, takes, phase, phase_word}` stay there. `--since` already
  supplies `[u8; 8]`. Take `limits::{WAIT, WAIT_MOST}` through `jig::Limits`'s
  `wait` and `wait_most`; neither parsing nor new timeout constants belong here.
- **One form per subject.** Extend temper 03's rendering with a borrowed `Part`
  (`src/jig/render.rs`, 04.1; a subject's value, not 03's `Parts`, the owner words).
  Its JSON reads the supplied contract value and limits alone, never the clock
  or `me`; “you” is text only. Add `digest` last after hashing the part without
  it. Keep the result envelope (`met`, `why`) and read cursor outside the hash.
- **Lists.** Normalize inbox read groups and both inbox delivery forms into the
  watch's first window: needs before updates, each in its supplied order, at
  most 32 rows across both. Normalize tasks from the live rows, in query order.
  Keep a `more` bit from the continuation/remaining rows; exclude cursor bytes
  and watch-only counts from the digested form. Conditions still read those
  counts from the delivery. A filtered inbox or a tasks read with after/ended
  prints no digest. This fixes answer 21's form, not a question for the run.
- **Facts.** `waiting-on-me` reads task N's `waits_on`, which already covers its
  tree, and live `me` for party/role membership. No second task watch for a tree.
  `replied` calls `jig_client::facts::chat_status`, accepting Parked,
  WaitingForYou, WaitingFor, Held, Closing or Ended, whatever the task's label.
- **Endings.** Ended phases are Ended/Done/Failed/Cancelled in a task's chip or
  a tree's root. A run has no terminal document fact: its watch end meets ended.
  For `task N --until ended`, a matching delivery or `Why::SubjectEnded` reads
  `Read::Task` to render
  its final facts; never print its last running document as a successful ended
  answer. Keep that read under the call's prefix and the next plan's host read deadline.
- **Timers and ownership.** Arm the runner's own `skein_lib::Deadlines` from
  `env.now`; expose it through root `fire` and `next_deadline`. Process inputs
  before timers: a meeting delivery wins in that iteration. Borrow documents
  through their handles, never across steps. One shared Runner asker, call
  numbers in handles' high 32 bits as temper 03 supplies; no asker per wait.
- **Result.** Use temper 03's `jig::Printed`: subject part with digest, then JSON
  `met` and, if unmet, `why` (`timeout` or `ended`); text adds `not met: ...`.
  An end before any delivery prints the subject and that it ended, with no
  invented document/digest; task ended still reads its final document. Refusals
  use `not_allowed`/`unknown` and exit 2; failures use exit 1 and the host's prefix.
  Reads/waits use the next plan's host's empty memory store: no kept output or store
  metadata is added here; corrected 29 and 33 remain temper 03's and that plan's output contract.

## Rules that bind it

- Nothing under `src/jig/` names temper or a `crate::` item outside `crate::jig`;
  temper 02's boundary test applies. Limits and application words arrive as data.
- Step code reads no clock and performs no I/O. No design, STATUS or profile
  edits; no optimized tests. No forge, fake forge, fake LLM or provider files.
- Extend the one fixtures module, `src/tests/fixtures.rs` (temper 02, 03),
  using shipped limits (`fixtures::limits()`, a function returning
  `limits::tools()`, which is no `const fn`); these tests are in
  `src/tests/jig/`. A fixture this file uses that neither temper 02 nor 03
  adds (`PARTY` and `me`, unless 03's harness has them, as its sketches
  use `PARTY`) joins the module here. Unit tests take under 20 ms, with virtual time for
  deadlines; the next plan's stories carry them.

## Increments

### 04.1 Digests, and the reads that print one
Scope: client/tools.md 5.4's common subject form, digest and eligible reads; three
separate digests for `task N --tree --run`.
Needs: nothing beyond the session's start.
Files: new `crates/temper-client-tools/src/jig/digest.rs`,
`src/tests/jig/digest.rs`; `src/jig.rs`, `src/tests/jig.rs`; temper 03's
`src/jig/render/{document,lists}.rs`, `src/jig/render.rs` (`Part`), `src/jig/runner.rs`;
`src/tests/fixtures.rs` and read goldens in `src/tests/golden/`.
Shape:
- `Digest([u8; 8])`, `Digest::of(json)` via `skein_lib::sha256::digest`, `hex`,
  and `from_bytes` for the table's checked since value; no parse method.
- `digest::part(&Part, &crate::jig::Limits)` uses the existing JSON renderer
  without a digest field. `Part<'a>`, new in `src/jig/render.rs` (sketch below),
  borrows Me, Inbox, Tasks, Task, Tree or Run facts, with `rows()` and `more()`.
- The read's list form is assembled from its rows as Choices says. Preserve
  owner row identity, ordering and supplied fields; no re-sort by tool name,
  no cursor in JSON rows used for hashing. Retain read continuations outside it.
- An inbox read that O's first page cuts before updates (31 needs, 2 updates)
  prints O's page and digests the watch's first window all the same (ruling
  137): its needs, then the first rows of the updates' page it read, to 32
  rows, and whether more follow, through `digest::part` and the one JSON
  renderer (client/tools.md 5.4, "The read makes that form from its own rows"). Its
  printed rows are then fewer than the digested; no second renderer, no other
  form.
- All eligible parts gain a digest; pages and filtered non-subject reads do not.
Test shape: `src/tests/jig/digest.rs`, on the shared document fixtures and
`fixtures::limits()`. Add builders for inbox needs/updates and a tasks page/window pair.
```rust
#[test]
fn a_read_and_a_watch_window_digest_the_same_rows() {
    let (read, watched) = fixtures::inbox_first_window(33);
    assert_eq!(digest::part(&read, &fixtures::limits()),
               digest::part(&watched, &fixtures::limits()));
    assert_eq!(read.rows(), 32);
    assert!(read.more());
}
```
The helpers extend `src/tests/fixtures.rs`; the reference includes both
inbox groups and an overflow, with the expected row identities/order checked
against the supplied window, not recomputed by the implementation.
Others, each under 20 ms:
- `a_digest_moves_with_the_document_and_only_then` (including row order/more);
- `the_printed_part_without_digest_is_the_bytes_digested` (16 lower-case hex;
  a delivery, and a read whose page is the window);
- `an_inbox_read_cut_before_updates_digests_the_watchs_first_window` (31
  needs, 2 updates: 31 rows printed, the digest a wait's first delivery's, 32
  rows and more);
- `one_delivery_under_two_clocks_and_two_mes_has_one_digest`;
- `list_cursors_and_watch_counts_are_outside_the_digest` (both inbox forms,
  tasks, 31/32/33 rows, a full window with no more and with more);
- `task_with_tree_and_run_prints_three_digests_each_over_its_part`;
- `pages_and_non_watch_arguments_print_no_digest`, including tasks after/ended
  and each inbox flag; supported task query filters still print one.
Proof: unit, `cargo nextest run -p temper-client-tools -E
'test(a_read_and_a_watch_window_digest_the_same_rows)'` (under 1 s), shows
identical first-window bytes despite page/watch envelopes. This plan builds no
tools world; 04.3 connects the digest to a wait, then the next plan's stories
connect the caller.
Don't: digest the engine's mark, revision, cursor, wait envelope or clock;
add another renderer or parser.
Size: at most 220 production lines; tests and goldens aside.
Removes: nothing.

### 04.2 The checked conditions' meaning
Scope: client/tools.md 5.4's conditions as facts over live documents, with `me`
where the condition says “you”. The table's checked values are the inputs.
Needs: nothing beyond the session's start; may land before 04.1.
Files: new `crates/temper-client-tools/src/jig/conditions.rs`,
`src/tests/jig/conditions.rs`; `src/jig.rs`, `src/tests/jig.rs`, `src/tests/fixtures.rs`.
Shape:
- `conditions::holds(&Condition, &Facts) -> Option<bool>`; None while a
  required document is not live. Import `Condition`, never redefine it.
- Me waiting: `Me.needs_total > 0`; inbox waiting/updates use `Inbox.needs`
  (any count above zero) and `updates.value`, or `OwnerInbox.cached_needs`
  and `cached_updates.value`, above zero.
- Task ended/held/live phase use `Task.chip.phase`; tree ended uses
  `OwnerTree.root` or the `Tree` node with `chip.task == tree.task`.
  Run ended waits for 04.4's watch end.
- `waits_on_me(waits_on, project, me)` follows client.md 19's party/role
  rule; `facts::reader` is private. Reuse `chat_status` for replied.
  `waits_on_me` copies `reader`'s role numbers (owner 0, maintainer 1,
  member 2, observer 3, a policy role its own) over `WaitsOn::Party`'s
  `whom`; `chat_status`'s `WaitingForYou` would not serve, as it puts
  held, closing and ended first, and client/tools.md's `waiting-on-me` reads what
  the task waits on. Name the copy in the commit body.
- `holds` reads a task condition from `Latest::Jig(Delivery::Task(task))`,
  its `waits_on`, `project` and `chip`, and gives None for another form.
  Tree has no waiting-on-me predicate and needs no root-task handle.
Test shape: `src/tests/jig/conditions.rs`, shared fixtures extended with a
chat's RunStage/OwnerChipWork, an ended chip's `ended_at`, and party/role waits.
`fixtures::chat_at(task, stage, work)` returns the delivered task as a
`Latest`; temper 03's `fixtures::chat(task, acts)` is another builder.
```rust
#[test]
fn replied_holds_once_a_chat_is_parked_and_not_while_it_writes() {
    let me = fixtures::me(fixtures::PARTY);
    let writing = fixtures::chat_at(41, RunStage::Running { attempt: 1 }, Some(OwnerChipWork::Writing));
    let parked = fixtures::chat_at(41, RunStage::Idle, None);
    assert_eq!(conditions::holds(&Condition::Replied, &Facts { subject: &writing, me: Some(&me) }), Some(false));
    assert_eq!(conditions::holds(&Condition::Replied, &Facts { subject: &parked, me: Some(&me) }), Some(true));
}
```
Others, each under 20 ms: `each_live_phase_holds_only_for_its_chip_phase`,
`replied_accepts_each_settled_chat_status_and_misses_the_others`,
`waiting_on_me_uses_the_party_and_its_project_role` (a descendant's question
and proposal in the root task's supplied waits_on; a different project misses),
`ended_and_held_follow_the_task_or_tree_root_phase` (all four terminal phases,
both tree forms), `waiting_and_updates_use_counts_in_both_inbox_forms`,
`a_condition_waits_for_each_document_it_reads_to_be_live`.
Proof: unit, `cargo nextest run -p temper-client-tools -E
'test(replied_holds_once_a_chat_is_parked_and_not_while_it_writes)'` (under 1 s).
The predicate is pure; 04.3 connects it to the caller, and the next plan's
stories prove the chat's reply through its worlds.
Don't: parse conditions or phase words again; edit the table/help; copy
`chat_status`, or infer a role from the chip.
Size: at most 180 production lines.
Removes: nothing.

### 04.3 The wait, answered or timed out
Scope: client/tools.md 5.4's checked wait call; deliveries, the answer rule below,
its timeout and cleanup. Met exits 0, timeout exits 3 with the document/digest.
Needs: the common digest (temper 04.1); checked conditions' meaning (temper 04.2).
Files: new `crates/temper-client-tools/src/jig/wait.rs`, `src/tests/jig/wait.rs`;
`src/jig/{runner,render}.rs`, `src/jig.rs`, `src/root.rs`;
`src/tests/jig.rs`, `src/tests/fixtures.rs` and wait goldens.
Shape:
- `Wait` holds the subject handle, optional dependency on live `me`, checked
  since/until, first digest and its deadline key. Use the root's shared me
  where available; release only this call's handles at the answer.
- Open jig's `Subject::{Me, Inbox, Tasks, Task, Tree, Run}` from the table's
  subject: window `limits.rows`, no continuation; Run turn None, offset 0.
  Resolve task queries as temper 03's reads do. A tree opens no task watch.
- Compare live documents on `Told::Changed`, and run text on `Told::Text`.
  A run's delivery brings both: each compares the subject's digest of its
  current document, so the second finds the first's digest and answers
  nothing.
  A live me change may re-evaluate a task condition; the digest remains the
  subject's. Behind/Opening are not deliveries. Keep no queue of old deliveries.
- Arm the checked timeout from `env.now`; default/cap are already 02's
  `WAIT`/`WAIT_MOST`. Fire after inputs. Borrow the current/last document
  before releasing handles; render met/why outside its digest.
Test shape: `src/tests/jig/wait.rs` (the waiting; 02.3's syntax tests are in
`src/tests/subject.rs`), temper 03's signed-in root harness: `submit`, `deliver`, `answer`, extended for virtual time and handle inspection.
`deliver`'s subject is temper's `jig::subject::Subject`, the table's
(`Task(u64)`), not jig's `contract::Subject`; where 03's `deliver` takes
another form, use 03's, and say so in the commit body.
```rust
#[test]
fn a_wait_until_replied_answers_at_the_parked_delivery_with_its_digest() {
    let mut root = fixtures::signed_in(fixtures::PARTY);
    let call = root.submit(&["wait", "task", "41", "--until", "replied", "--json"]);
    root.deliver(call, Subject::Task(41), fixtures::chat_at(41, RunStage::Idle, None));
    let answer = root.answer(call).expect("met on the first parked delivery");
    assert_eq!(answer.status, 0);
    assert!(answer.stdout.ends_with(b",\"met\":true}\n"));
}
```
Others, each under 20 ms: `a_read_and_a_wait_print_one_digest` (all six subjects,
inbox/tasks beyond the first window; since holds while unchanged),
`since_compares_the_first_delivery_and_neither_waits_for_a_change`,
`since_and_until_need_both_on_the_same_delivery`,
`timeout_exits_3_with_the_last_document_and_digest`,
`a_me_change_rechecks_the_task_without_changing_its_digest`,
`a_meeting_delivery_wins_over_timeout_in_the_same_iteration`;
check run text, coalesced changes, no answer while behind, and cleanup too.
Proof: unit, `cargo nextest run -p temper-client-tools -E
'test(a_read_and_a_wait_print_one_digest)'` (under 1 s), shows a read's digest
holding a wait until its document changes. This plan builds no tools world;
the next plan's stories (a reply, a concurrent close) connect it.
Don't: revalidate argv, duplicate timeout constants, hold a document across
steps, take a lock or touch the small store (the next plan's host owns them).
Size: at most 260 production lines.
Removes: nothing.

### 04.4 A watch's end, and the task's final read
Scope: client/tools.md 5.4/6.3's watch ends and final task document.
Needs: the wait's deliveries and timeout (temper 04.3).
Files: `crates/temper-client-tools/src/jig/{wait,runner}.rs`;
`src/tests/jig/wait.rs`, `src/tests/fixtures.rs` and goldens.
Shape:
- `Why::SubjectEnded` meets an Ended condition; any other condition or none
  exits 3 with last document/digest and `not met: ended`. `--since` does not
  turn a subject's terminal end into another change delivery to await.
- For a task waiting for ended, a delivery satisfying the answer rule or that watch end
  enters `Finishing { read }`: cancel the wait timeout, issue `Read::Task`
  under a fresh local handle and await its terminal before printing. An ended
  delivery without a matching condition remains subject to 04.3's rule.
- Render the returned task with 04.1's digest and met true; the task says how
  it ended. Read refusals/failures follow temper 03: engine refusal 2,
  client limit/transport/sign-out failure 1. An unheard final read is bounded
  by the next plan's host link/read deadline, not another successful watch-end answer.
- Other `Why` values: NotAllowed/Unknown exit 2; SignedOut/NotServed/Limit
  exit 1. Sign-out of a required me ends the call too. One terminal result,
  releasing all call handles and timers, including the final read's handle.
- Stopping/Dropped/Busy are not Told::Ended: jig reopens the watch. Behind
  preserves the wait and last document; only the next plan's host's expired link deadline
  makes a continuously unavailable engine exit 1. There is no engine-stop exit.
Test shape: extend `src/tests/jig/wait.rs`; `end` injects `Up::WatchEnded`,
`serve` answers the final read. No new harness or test kind.
- `a_task_watch_ending_reads_how_the_task_ended_before_answering` (no success
  before read; all terminal phases; also an already-ended first delivery).
- `a_run_or_tree_ending_meets_ended` (with/without a preceding delivery);
- `a_subject_ending_under_another_condition_exits_3_with_its_last_digest`;
- `a_subject_no_longer_visible_exits_2` (Unknown/NotAllowed);
- `sign_out_and_client_limits_exit_1` (including the final read);
- `a_stopping_engine_leaves_the_wait_behind_and_reopening_can_answer`;
- `an_answered_wait_releases_all_its_handles_and_timer` (late terminals ignored).
Each under 20 ms. Assert final-read text/JSON equals the ended task read's part,
with no running phase substituted, and assert no extra task watch for tree.
Proof: unit, `cargo nextest run -p temper-client-tools -E
'test(a_task_watch_ending_reads_how_the_task_ended_before_answering)'` (under 1 s).
This plan builds no tools world; the next plan's stories (a closed chat, a
concurrent close) prove the final read against the engine, and its host proves
exit 1 at the host-owned link deadline with virtual time, never a 30 s
real-loop sleep.
Don't: synthesize the ended task from its running chip; answer Stopping,
Dropped or Busy; classify a client failure as exit 2; add a tree's task watch.
Size: at most 200 production lines.
Removes: nothing.

## Interface sketches

Each function that returns a value is `#[must_use]` (clippy's
`must_use_candidate`). The path comments name the files and are not
their text: no file under `src/jig/` names temper (`tests::boundary`).

```rust
// crates/temper-client-tools/src/jig/digest.rs (04.1)
/// First eight SHA-256 bytes of the common subject form, owned by the caller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Digest([u8; 8]);
impl Digest {
    pub fn of(json: &[u8]) -> Self;
    pub fn hex(self) -> [u8; 16];
    /// Already checked by the table; no second parser.
    pub fn from_bytes(bytes: [u8; 8]) -> Self;
}
/// Part is the renderer's borrowed subject form; the digest excludes its envelope.
/// `jig::Limits`, the table's, carries the rows the digest reads.
pub fn part(part: &Part<'_>, limits: &crate::jig::Limits) -> Digest;
// crates/temper-client-tools/src/jig/render.rs (04.1); not 03's `Parts`, which words owner parts
/// A subject's contract value, borrowed from the delivery or the read (inbox: either form).
pub enum Part<'a> {
    Me(&'a Me), Inbox(&'a Inbox), OwnerInbox(&'a OwnerInbox), InboxPage(&'a InboxPage),
    OwnerInboxPage(&'a OwnerInboxPage), Tasks(&'a Tasks), Task(&'a Task), Tree(&'a Tree),
    TaskPage(&'a TaskPage), OwnerTree(&'a OwnerTree), Run(&'a RunText), RunState(&'a RunState),
}
impl Part<'_> { pub fn rows(&self) -> usize; pub fn more(&self) -> bool; }
// crates/temper-client-tools/src/jig/conditions.rs (04.2)
use crate::jig::subject::Condition; // the table's checked type
/// Live subject and optional live me, borrowed within this step; no root-task watch.
pub struct Facts<'a> { pub subject: &'a Latest, pub me: Option<&'a Me> }
pub fn holds(condition: &Condition, facts: &Facts<'_>) -> Option<bool>;
/// The supplied party or a role me says it holds in project (client.md 19).
pub fn waits_on_me(waits_on: &WaitsOn, project: u32, me: &Me) -> bool;
```

The answer rule for live deliveries (04.3); first is the first live subject
digest. Terminal watch ends take 04.4's rule. No comparison uses the engine mark.

```rust
// crates/temper-client-tools/src/jig/wait.rs (04.3)
fn answers(&self, digest: Digest, holds: Option<bool>) -> bool {
    let differs = match self.since.or(self.first) {
        Some(old) => old != digest,
        None => false,
    };
    match (&self.until, self.since) {
        (Some(_), Some(_)) => differs && holds == Some(true),
        (Some(_), None) => holds == Some(true),
        (None, _) => differs,
    }
}
```

The runner owns Wait per call number, emits temper 03's `Out::Printed` once,
and releases its handles at completion. A task end may first finish its read;
root `fire`/`next_deadline` include the wait while the next plan's host owns link deadlines.

## Questions to raise when reached

- A unit test past 20 ms: report its measured time and allocation at
  `Client::new`; profile and make it cheaper without changing the build profile.

## Done when

Each in `~/src/rust/temper`, against fetched `origin/main`; proofs run at their
increments, not again here. These read production properties, never test names.

- `git show origin/main:crates/temper-client-tools/src/jig/digest.rs | grep -q 'sha256'`
- `git grep -q -F 'Condition::WaitingOnMe' origin/main -- crates/temper-client-tools/src/jig/conditions.rs`
- `git grep -q -F 'chat_status' origin/main -- crates/temper-client-tools/src/jig/conditions.rs`
- `git grep -q -F 'Why::SubjectEnded' origin/main -- crates/temper-client-tools/src/jig/wait.rs`
- `git grep -q -F 'Read::Task' origin/main -- crates/temper-client-tools/src/jig/wait.rs crates/temper-client-tools/src/jig/runner.rs`
- `git grep -q -F 'Deadlines' origin/main -- crates/temper-client-tools/src/jig/wait.rs crates/temper-client-tools/src/jig/runner.rs`
- `! git grep -q -E 'fn (parse|condition|phase|phase_word)\(' origin/main -- crates/temper-client-tools/src/jig/conditions.rs`
- `test -z "$(git grep -n -i temper origin/main -- crates/temper-client-tools/src/jig/digest.rs crates/temper-client-tools/src/jig/conditions.rs crates/temper-client-tools/src/jig/wait.rs)"`
