# Session 01: jig on skein's and smith's main

jig moves its pins of skein and smith together onto their mains when this plan started: skein `b4c6241` and smith `a277e922`.
smith `29fa97c9`, jig's pin today, does not build on skein's main (`findings/base-repin.md`, "The finding that shapes the fix").
01.1 is the repin. It repairs J1 to J7 at the least change that keeps behaviour, and refreshes the transcript's goldens from
smith's version 5. A turn that holds a job's outcome reads as one jig cannot decode until 01.2. 01.2 then gives jobs and a call
not run the words of the client's design (`client.md`, 10). 01.3 lets jig's test configuration take smith's shipped memory.
The repin was rehearsed at jig `f50c3f63` on debian-16gb-hel1-1. Measured there: 137 changed lines besides `Cargo.lock`, 65 of
them production; the focused suite (4270 tests, 5.8 s), the fuzzy suite (111, 7.6 s), the process tier (2, 0.4 s), clippy and
fmt passed. The browser tier was not run. Three increments, in order; each lands alone. The downstream repository's tools plan
repins onto 01.1's landing.

## Read first

- **Plan:** `findings/base-repin.md`, "Commands" and "jig, config B (skein + smith), errors by group" (J1 to J7).
- **jig:**
  - `docs/design/client/domain/client.md` 10, the item "A job is a call's outcome";
  - `docs/design/protocol/README.md` 11, "What jig needs from smith" (transcript v5, already written);
  - `docs/design/STATUS.md`, the item "smith's parity and wall-clock changes, for jig's next repin of smith";
  - `docs/development/tech-debt.md` 11, the item "Four copies of smith's goldens";
  - `crates/jig-protocol-transcript/src/summary.rs` (`validate_returned`, `summarize`, `Builder::outcome`, `Builder::command`)
    and `src/lib.rs` (`decode`, `result_text`, the module comment on call numbers);
  - `crates/jig-client-view/src/turns/call.rs` (`call`: "Call failed", "Call completed" or "Call in progress", then the
    summary).
- **smith** at `a277e922`:
  - `docs/design/protocol/transcript.md` 2 (a call's outcomes, "not run, with why") and 4 ("No job is resumed");
  - `docs/design/domain/tools.md` 5 (a job: reading it, stopping it, its bound);
  - `docs/design/shell.md` 6.3, the row `memory`;
  - commit `b415f737`, smith's own repin onto skein's new variants: new arms, with locks kept impossible where nothing asks
    for one;
  - commit `99356013`, the shipped memory raised to `2_214_592_512`.
- **skein:** `docs/foundation/development.md` 5.3 (transitional code names the step that removes it), 5.6 (limits come from
  the shipped profile) and 5.7 (repins); 3.3 (the status file is the designer's).

## What the design asks

- **client.md 10:**

  > **A job is a call's outcome,** worded like any other call's. A call that answered `running` reads "started as job N",
  > never "is running": a job ends with its activation (smith's `protocol/transcript.md`, section 4). A `job` call names its
  > job by number and what it answered. `JobsFull` and `UnknownJob` read as invalid calls. A call not run reads with its why:
  > stopped, outside the choice, or after a failed write. The turn's count of jobs is not shown.
- **STATUS.md, the smith item:** the transcript's version 5 and its not-run why, which `jig-protocol-transcript` follows
  where it matches `Returned::NotRun`; the event vocabulary's version 3 and `Purpose::Ordinary { guard }`; the shipped
  profiles' larger `memory`, which jig's test configurations follow. "The downstream repository's tools plan moves jig's pin of
  smith to smith's `main` (transcript version 5, jobs); the crate it changes is `jig-protocol-transcript`."
- **protocol/README.md 11** already names transcript v5: no increment edits it.
- **smith's shell.md 6.3, `memory`:** 2,112 MiB, "as large as smith's work needs [...] and never smaller than the derived
  worst case". development.md 5.6: a test takes the shipped profile's limits, and names another value only with its reason.
- **development.md 5.7:** the repin moves skein and smith together; it changes the pins, required items, moved names and
  paths, and the tests an upstream change of behaviour broke, adapted as upstream adapted its own; at most about 200 lines
  besides `Cargo.lock`, and no change of design. Its Proof is the gate's.

## Choices this plan makes

- **01.1 only compiles and keeps behaviour.** The words for jobs are a change of design for 5.7, so they are 01.2's. Until
  01.2, `validate_returned` refuses a job's outcome (`Outcome::Running`, `Outcome::Job`) and a job's problem
  (`CallProblem::JobsFull`, `UnknownJob`) as `Undecodable::Unknown`. Such a turn reads as one jig cannot decode, as a turn
  naming an unknown variant does. No half-built words are shown (5.3). `summarize`'s and `Builder::outcome`'s new arms are
  `unreachable!` with their reason, which jig's clippy allows. A call not run keeps reading "Not run" (J5).
- **The version check names smith's constant.** `decode` compares the record's version with `smith::Turn::FORMAT_VERSION`,
  not a literal. The finding missed this site: at `3` it refused every v5 turn as `Version { found: 5 }`.
- **The goldens follow smith.** `tests/data/record_turn_full.bin` and `record_turn_smallest.bin` become smith's
  `crates/smith-transcript/golden/v5/` at `a277e922` (146 and 42 bytes). The tests' literals derived from their sizes follow:
  145, 146, 147 and `FULL_TURN`. Version bytes `[0, 3]` in tests that expect an unreadable record become `[0, 5]`. That is
  smith's own adaptation, to its version, in its v5 goldens. smith's charter golden v2 is the same at both pins.
- **01.2's words** (the summary's, under the view's "Call completed" or "Call failed"):

  | smith's answer | summary | failed |
  |---|---|---|
  | `Outcome::Running` | `Started as job N; output bytes B; dropped D` | no |
  | `Outcome::Job`, end `Running` | `Job N answered running; output bytes B; dropped D` | no |
  | `Outcome::Job`, end `Ended` | `Job N ended: exit C` (or `signal S`, `timed out`) `; output bytes B; dropped D` | as a command's end |
  | `Outcome::Job`, end `Stopped` | `Job N stopped; output bytes B; dropped D` | no: its call asked it |
  | `Outcome::Job`, end `Cancelled` | `Job N cancelled; output bytes B; dropped D` | yes |
  | `CallProblem::JobsFull` | `Invalid call: jobs full, at most N at once` | yes |
  | `CallProblem::UnknownJob` | `Invalid call: unknown job N` | yes |
  | `NotRunWhy::Stopped` | `Not run: stopped` | yes |
  | `NotRunWhy::OutsideChoice` | `Not run: outside the tool choice` | yes |
  | `NotRunWhy::AfterFailedWrite` | `Not run: after a failed write` | yes |

  "Output bytes" is the head's and the tail's lengths, as `Builder::command` counts them. Two things are not shown.
  - The allowed calls' names: a summary is built within 256 bytes of fixed words and numbers (`Builder::put`).
  - The failed write's position: smith counts it among the completion's calls, while jig numbers a call by its first block's
    position in the turn (`lib.rs`, module comment), so the two numbers would disagree on one page.
- **`Turn.jobs` is not shown:** `contract::Turn` gains no field. One unit test shows that two turns differing only in `jobs`
  decode the same.
- **01.3 takes the shipped memory by omission.** `agent_configuration` stops declaring `memory`, so that smith's shipped
  `standard` value applies (2,112 MiB at `a277e922`), and keeps declaring `tool_payload` and `read_window`. It is not
  restated as a number: 5.6 asks a test to name a value only when it needs another. It also passes at 2 GiB on the new pins
  (measured), so it is not part of the repin. The STATUS item also cites `tests/store/src/root.rs`. Its `queued_bytes:
  2_147_483_648` is skein-kv's queue bound, not smith's memory, and stays.
- **STATUS.md is not this session's** (3.3). The designer closes the smith item after the plan.

## Rules that bind it

- **Pins** (development.md 5.7). Each pin is named with `cargo update -p <package> --precise <sha>`. After
  01.1, `grep -o 'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' Cargo.lock | sort -u` prints two lines:
  - `skein.git#b4c6241aefea0e434b6e1128754168d02bae0706`;
  - `smith.git#a277e9226273bf71c845db98b62a59b9ca4c9bfd`.

  A conflict in `Cargo.lock` is never merged by hand: take `origin/main`'s, then run Shape step 1 again. jig 02.1 moves skein
  past `b4c6241` later in this plan. jig 02 to 05 do not move smith; `jig-repin` does.
- **A repin's bounds** (5.7). A break past 01.1's Shape, past about 200 lines, or needing a decision is a question. Then the
  step lands nothing, and the gap goes to jig's STATUS.md, under "Not planned", and to the plan's `later.md`.
- **Tests keep their names and expected values** in 01.1, but for those adapted under 5.7, which Shape names. The landing's
  body names smith `a277e922`'s golden v5 as the rule they follow.
- **jig's files never name the downstream repository.** Commit bodies and comments say "the downstream repository's tools
  plan".

## Increments

### 01.1 jig on skein `b4c6241` and smith `a277e922`
Scope: the two pins; J1 to J7 of the finding, with the sites it missed; the goldens and the literals that follow their sizes;
tech-debt.md 11's facts on the copies.
Needs: nothing.
Files (the rehearsal's 30, `Cargo.lock` a shared hotspot):
- `Cargo.lock`;
- J1: `crates/jig-protocol-store/src/directory.rs` (`made`, `opened`), `src/transcripts/cut.rs`, `read.rs`, `write.rs`
  (`arrival` in each);
- J2, J3: `crates/jig-protocol-store/src/tests/fixtures/store.rs`, `src/tests/transcripts.rs`;
  `testing/jig-test-service/src/accounts.rs` (`file_owner`); `testing/jig-test-store-files/src/files.rs` (`Files::answer`);
- J4 to J7: `crates/jig-protocol-transcript/src/lib.rs`, `src/summary.rs`, `src/tests.rs`, `tests/data/` (the two goldens and
  `README.md`); `testing/jig-test-worker/src/tests/fixtures.rs`; `testing/jig-test-service/src/tests/fixtures.rs`;
  `tests/client/src/joint/` (`fixture.rs`, `front.rs`, `session.rs`, `deployment_story.rs`, `historical_story.rs`,
  `role_stories.rs`, `wire/activity.rs`, `wire/turns.rs`); `tests/client/src/stories/unread_turn.rs`;
  `tests/ops/src/conformance/scripts.rs`;
- the literals: `testing/jig-client-test-records/src/activity_reads.rs` (`FULL_TURN`);
  `crates/jig-client-protocol-link/src/tests/fixtures/turns.rs` (`unread_pages`), `src/tests/pages.rs`;
- `docs/development/tech-debt.md` 11.

None is the fake forge, a forge client or component, the fake LLM or a provider.
Shape, in this order:
1. The pins, from jig's root: `cargo update -p skein-io --precise b4c6241aefea0e434b6e1128754168d02bae0706`, then
   `cargo update -p smith-domain --precise a277e9226273bf71c845db98b62a59b9ca4c9bfd`, then skein's command once more:
   smith's update moves the skein packages only smith's crates use to skein's `main`. Check the Pins rule's two lines; a
   second revision still there after that is a question.
2. J1, skein's `file::Event::{Locked, Busy}`: `| file::Event::Locked { .. } | file::Event::Busy { .. }` joins each exhaustive
   ignore arm. J2, `file::Request::{ListPage, Lock}`: they join the store tests' arms and `Files::answer`'s panic arm, and
   `file_owner`'s owner arm as `ListPage { owner, .. } | Lock { owner, .. }`. Nothing in jig takes a lock (smith `b415f737`).
   J3, `Rename { from_root, to_root, .. }`: `Files::answer` passes `opened(from_root)` and `opened(to_root)` to
   `machine.rename`.
3. J4: `smith_transcript::v3` becomes `v5` at its 13 sites, `tests/ops`'s `use smith_transcript::v3 as transcript` included
   (the finding missed it). `decode` checks `smith::Turn::FORMAT_VERSION`.
4. J5: `smith::Returned::NotRun(_)`, in `validate_returned` and `result_text`, and "Not run" in `summarize`. J6, as Choices
   says: in `validate_returned`, `Outcome::{Running, Job}` and, through `invalid.problem()`, `CallProblem::{JobsFull,
   UnknownJob}` return `Err(Undecodable::Unknown)`; `summarize`'s and `Builder::outcome`'s arms are `unreachable!("validate
   refuses job …")`. Each such arm is commented "removed by jig-01.2". J7: `jobs: 0` in the four literals and `tests.rs`'s
   `source`.
5. The goldens: `git -C ~/src/rust/smith show a277e922:crates/smith-transcript/golden/v5/record_turn_full.bin`, and the
   smallest, over `tests/data/`'s two files. `tests/data/README.md` names `a277e922` and `golden/v5/`.
6. The tests adapted under 5.7, nothing else:
   - `tests.rs`: its lengths 142 and 38 become 146 and 42; its `TooLarge` record's version byte becomes 5;
   - `FULL_TURN` becomes the v5 full golden's 146 bytes;
   - `limits()` in `joint/wire/turns.rs` and `codec()` in `joint/wire/activity.rs`: 146, with the literals beside them
     (145 too large, 147 trailing);
   - `pages.rs`, `a_turn_past_its_admission_bound_is_carried_as_too_large`: `turn_limits(32, 145)`;
   - `unread_pages` and `unread_turn.rs`: `[0, 3]` and `[0, 3, 255]` become `[0, 5]` and `[0, 5, 255]`.
7. tech-debt.md 11's item on the copies: 146 and 42 bytes, the pin `a277e922`, `golden/v5/`, and the README naming the pin.
   Its point, that nothing checks the copies, stays.
8. `cargo check --workspace --all-targets`, then the gate's suites. A failure past step 6 follows the 5.7 rule.

Test shape: no test is added. `tests.rs`'s case for a call not run builds its value with `NotRunWhy::Stopped`; its expected
`Not run`, failed, is unchanged.
Proof: process, `scripts/check.sh`, the gate. It shows jig building, linting and passing its focused, fuzzy and browser
tiers on the new pins. Bound: the gate's own; the rehearsal's focused suite took 5.8 s and its fuzzy suite 7.6 s.
Don't:
- hand-edit `Cargo.lock` or generated code;
- take, answer or count a lock, or give `Busy` a meaning;
- show anything for a job, a not-run why or `Turn.jobs` (01.2's);
- change `testing/jig-test/tests/support/mod.rs` (01.3's);
- edit `docs/design/`.
Size: at most 200 lines besides `Cargo.lock` and the two goldens; the rehearsal's was 137, 65 of them production, and step 7
adds about 6.
Removes:
- jig's pins at the session's start (at `f50c3f63`: skein `0f093bb7`, smith `29fa97c9`);
- `smith_transcript::v3`, the version literal `3` in `decode`, and the v3 goldens.

### 01.2 Jobs as call outcomes, and a call not run with its why
Scope: client.md 10's words for jobs and for a call not run, as Choices' table gives them; `Turn.jobs` not shown.
Needs: nothing beyond 01.1.
Files: `crates/jig-protocol-transcript/src/summary.rs`, `src/tests.rs`; `tests/client/src/joint/stories.rs` (one `mod`
line) and a new `tests/client/src/joint/stories/job_stories.rs`.
Shape:
1. `validate_returned` accepts `Outcome::{Running, Job}` and `CallProblem::{JobsFull, UnknownJob}`, as it accepts the other
   outcomes and problems, with no check of its own. 01.1's refusals and `unreachable!` arms go.
2. `Builder::command`'s exit and output are split into two helpers, `exit` and `output`. `command` keeps its words, and a
   job's end uses the same two.
3. `Builder::outcome` gains `Running` and `Job`; `summarize` gains the two problems and the three whys, each as Choices'
   table says, through `put` and `number`.
Test shape: unit tests in `src/tests.rs`, with its helpers `result`, `assert_result` and `exited`.
- `jobs_read_as_call_outcomes`: a command answered running; a job read running, ended (exit 0, exit 1, a signal, timed
  out), stopped and cancelled; each with its summary and failed flag.
- `job_problems_read_as_invalid_calls`.
- `a_call_not_run_reads_its_why`: the three whys.
- `a_turns_count_of_jobs_is_not_shown`: one turn decoded with `jobs: 0` and with `jobs: 3` gives equal values.
Ceiling: under 10 ms each.
The joint story, `a_job_reads_as_its_calls_outcome`, uses `super::world` with a turn built in the story's file, on
`smith_transcript::v5` as `fixture::turn` builds one. The assistant's message calls `job` and `shell`. The next message
answers `job` with job 1's end at exit 0 and `shell` with `running` as job 2. That is two messages of two blocks each, within
the joint world's receiving limits. The person signs in and starts a conversation, as the joint stories' first one does
(`stories.rs`, `signing_in_and_starting_a_chat_on_the_engine`). In the `shell` call's region they then see "Call
completed" and "Started as job 2", and in the `job` call's region "Job 1 ended: exit 0". The joint world cuts a summary at 32
bytes, and both prefixes fit within it. Ceiling: under 1 s of wall time.
Proof: joint, `cargo nextest run -p jig-client-world -E 'test(=joint::stories::job_stories::a_job_reads_as_its_calls_outcome)'`,
once. It shows a job's start and a job's end reaching the person's page as call outcomes, through the engine, the worker
and the client. Bound: 1 s besides the build.
Don't:
- word a call as "is running", or show `Turn.jobs`;
- show the allowed calls' names or smith's call position;
- add a field or variant to `jig_client::contract` or `Undecodable`;
- change `Builder::command`'s words or another outcome's;
- touch the view's phrases.
Size: at most 80 production lines in `summary.rs` (`+60 −10` expected); tests aside.
Removes: 01.1's refusal of job outcomes and problems in `validate_returned`, and its `unreachable!` arms in `summarize` and
`Builder::outcome`.

### 01.3 jig's test configuration on smith's shipped memory
Scope: the agent configuration of `jig-test`'s process and live tiers takes smith's shipped `memory`.
Needs: nothing beyond 01.1.
Files: `testing/jig-test/tests/support/mod.rs` (`agent_configuration`).
Shape: `"memory":2_147_483_648_u64,` leaves the `declared` object; `tool_payload` and `read_window` stay.
Test shape: none added. The process test that runs the shipped agent on this configuration is the measure.
Proof: process, `cargo nextest run -p jig-test --profile process -E 'test(=a_killed_jig_test_restarts_and_the_work_is_found)'`,
once. It shows `jig-test`'s agent passing smith's startup checks at the shipped memory, answering a chat, and the work found
after a restart. The process tier is not in jig's gate. Bound: 2 s besides the build (the rehearsal: 0.4 s for the tier's two
tests).
Don't: declare `memory` as a number; change another declared quantity, the policy or the endpoints; touch
`tests/store/src/root.rs`.
Size: 1 line.
Removes: the declared 2 GiB, below smith's shipped 2,112 MiB.

## Interface sketches

None crosses a crate boundary: `contract::Turn`, `contract::Call` and `Undecodable` keep their shapes. Inside
`summary.rs`, 01.2's `Builder` reads:

```rust
impl Builder {
    fn outcome(&mut self, outcome: &smith::Outcome);   // gains Running and Job
    fn command(&mut self, end: &smith::CommandEnd);    // "Command ended: " + exit + output, as today
    fn job(&mut self, read: &smith::JobRead);          // "Job N " + its end + output
    fn exit(&mut self, exit: &smith::Exit);            // "exit C" | "signal S" | "timed out"; failed unless exit 0
    fn output(&mut self, head: usize, last: usize, dropped: u64); // "; output bytes B; dropped D"
}
```

## Questions to raise when reached

- **01.1: a test that fails for a reason Shape step 6 does not cover**, or a repair past about 200 lines. Raise it with the
  test, its output and the smith or skein commit it points at. The rehearsal found none, but the browser tier was not run.
- **01.1: a third revision in `Cargo.lock`** after skein's command has run again, or smith `a277e922` resolving a skein other than `b4c6241`.
- **01.2: the joint world shows no call's region for a turn** (no joint story yet sees a call outcome). Raise it before
  proving at the world layer instead.

## Done when

Each in `~/src/rust/jig`, after `git fetch` there and in `~/src/rust/skein` and `~/src/rust/smith`:
- `pin=$(git show origin/main:Cargo.lock | grep -o 'git+https://github.com/forgetic/smith.git#[0-9a-f]*' | sort -u | cut -d'#' -f2) && test "$(printf '%s\n' "$pin" | wc -l)" = 1 && git -C ~/src/rust/smith merge-base --is-ancestor a277e9226273bf71c845db98b62a59b9ca4c9bfd "$pin"`
- `pin=$(git show origin/main:Cargo.lock | grep -o 'git+https://github.com/forgetic/skein.git#[0-9a-f]*' | sort -u | cut -d'#' -f2) && test "$(printf '%s\n' "$pin" | wc -l)" = 1 && git -C ~/src/rust/skein merge-base --is-ancestor b4c6241aefea0e434b6e1128754168d02bae0706 "$pin"`
- `git show origin/main:crates/jig-protocol-transcript/src/lib.rs | grep -q 'smith::Turn::FORMAT_VERSION'`
- `test -z "$(git grep -n -E 'smith_transcript::v[0-4]([^0-9]|$)' origin/main -- crates testing tests)"`
- `git show origin/main:crates/jig-protocol-transcript/tests/data/README.md | grep -q 'golden/v5/'`
- `git show origin/main:docs/development/tech-debt.md | grep -q 'smith-transcript/golden/v5/' && ! git show origin/main:docs/development/tech-debt.md | grep -q 'smith-transcript/golden/v3/'`
- `git show origin/main:crates/jig-protocol-transcript/src/summary.rs | grep -q 'Started as job ' && ! git show origin/main:crates/jig-protocol-transcript/src/summary.rs | grep -qi 'is running'`
- `git show origin/main:crates/jig-protocol-transcript/src/summary.rs | grep -q 'Not run: after a failed write' && ! git show origin/main:crates/jig-protocol-transcript/src/summary.rs | grep -q 'validate refuses job'`
- `! git show origin/main:testing/jig-test/tests/support/mod.rs | grep -q '"memory"'`
- 01.1 to 01.3 have landed, each with its gate, and 01.2's and 01.3's proofs passed once.
