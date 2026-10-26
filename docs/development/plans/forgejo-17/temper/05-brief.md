# Session 05: the repair's brief names the failed step and its log's end

temper 04.1 changes `Read::Job`'s answer. It now carries the failed
step, `step: Option<Step>`, and the end of that step's log, or the end
of the job's log when no step failed. The forge connector's CI section
still writes that log under `Job log tail:`, and ignores the step
(`crates/temper-engine-domain-forge/src/brief.rs:306-322`). In 05.1 the
section names the step by number and name, then gives the end of its
log. When no step failed, it says so and gives the end of the job's
log. A truncated log is marked before the bytes kept. A read that fails
and a status with no pinned job keep their lines. The engine's domain
world tests the section on the fake forge's jobs, which have steps since
temper 03.1: a failed workflow step, and a cancelled job with no failed
step. The system world's failed-CI story asserts the step's line and the
log's end. It is the one story that runs the engine, the worker's
smith-hosted agents and the fake forge together. The section's head and
check lines, the share of the budget for the log, and the CI cut stay as
they are.

## Read first

- `docs/design/domain/forge.md`, sections 2 ("Brief sections"), 8.4 ("A
  repair", its second paragraph), 18 (the story "CI failing, repaired")
  and 20.1 ("CI and its failures").
- `docs/design/forge.md`, 3.2: the row `Read::Job`, and the note "A
  failed job's log is its failed step's end".
- `docs/design/domain/engine.md`, 9 (the sentences "what does not fit is
  cut in its kind's order and says how much" and "A section is loaded or
  read as it is cut, asking for no more than it keeps").
- `docs/design/domain/testing.md`, 2.1 (system worlds).
- The plan: `decisions.md` 6, 9 and 10; `code-organization.md` 2 ("The
  answer", "The fake's failing step"), 3 (the row "the brief's words for
  the failed step") and 6; `findings/fake-forge-and-brief.md`, sections
  4 to 6; `findings/answer.md`, 1 (the user `brief.rs:308`) and 3.
- This plan's `temper/03-fake-jobs.md`, 03.1's Shape: the fake's three
  steps, each step's log, `Failing`, and how a context's `Failing` is
  seeded. Also `temper/04-job-read.md`, 04.1's Shape: `api::Step`,
  `Answer::Job`, and `tests/engine/forge/client/src/translate.rs`'s job
  answer.
- skein's `docs/foundation/programming-model.md`, 3 and 6.1 (by number
  only); skein's `docs/foundation/testing-strategy.md`, 2.2 and 2.3 (by
  number only).
- Code: `crates/temper-engine-domain-forge/src/brief.rs:61-91` (`append`,
  `brief_part`), `:298-322` (the `Checks` and `Job` arms of
  `brief_answer`), `:375-407` (`next_failed_job`, the share),
  `:536-553` (`decimal`); `crates/temper-engine-domain-forge/src/held.rs:114-185`
  (`render` and `edge`: CI cut from its start); `crates/temper-engine-domain-forge/src/domain.rs:695-705`
  (`max_job_bytes`).

## What the design asks

- **The CI section** (domain/forge.md, 8.4): "For each failed check, the
  brief carries its context, description and link, then the step that
  failed, by name, and the end of that step's log, where a step tells
  its failure; the job's whole log's end when no step failed. The log is
  read at the job attempt that failed on this head, within the brief's
  byte budget, and its truncation is said; a read that fails is said in
  its place (section 20.1; `docs/design/forge.md`, 3.2)."
- **Brief sections** (domain/forge.md, 2): "CI's failures on its head,
  each with its failed step and the end of that step's output".
- **What the answer gives** (forge.md, 3.2, `Read::Job`): "the failed
  step, its `number` and `name`, none when no step failed; the last
  `max_bytes` bytes of that step's log, or of the job's whole log when
  no step failed; `truncated` when `Content-Range` names bytes before
  them".
- **Cutting** (domain/engine.md, 9): "what does not fit is cut in its
  kind's order, at its gather as in the brief, and says how much" (the
  gather: design question 4, answered 2026-10-16). The CI kind is cut
  from its start (`held.rs`, `render`, `tail`), so the log's end is the
  last thing it loses; at the base the gather cuts every section from
  its end (`append`, `brief.rs:61-69`; `brief_part`, `:79-91`), and
  05.1 makes it cut a CI section from its start.
- **A step's name** (forge.md, 3.3): "a job step's name [...] a longer
  one is carried empty, and the step is named by its number alone"
  (design question 1).
- **The world** (domain/forge.md, 18) names "CI failing, repaired" among
  the connector world's stories. That story is
  `tests/engine/forge/tests/simulation.rs:674`
  (`a_failed_ci_head_is_repaired_then_checked_again`), and it gathers no
  brief. The brief's words are held by the engine's domain world and by
  the system world (decisions.md, 10).

## Choices this plan makes

- **The words.** The design fixes what the section carries, not its
  words. 05.1 writes these lines, as byte literals, after a failed
  check's `Failed check:`, `Description:` and `Link:` lines:

  | Case | Lines |
  |---|---|
  | a step failed | `Failed step: <number> <name>\n` (`Failed step: <number>\n` when the name is empty), then `End of its log:\n` |
  | no step failed | `No step failed. End of the job's log:\n` |
  | the log truncated (either case) | `[log truncated before this]\n`, after the line above, before the log |
  | the log | its bytes, then `\n` |
  | the read failed, or answered another head | `[job log could not be read]\n` (unchanged) |
  | the status names no job at this head | `[no pinned job log for this head]\n` (unchanged) |

  `<number>` is the step's `number` in decimal, as Forgejo numbers it
  (0 is `Set up job`). `<name>` is the step's name as the answer gives
  it: its bytes are written as they came, a line break included. The
  section's head line, its closing `Repair the failed check.\n` and the
  check's three lines do not change. `Job log tail:` and `[job log
  truncated]` go, with no other words in their place.
- **The share stays.** The log is asked for at `bytes / checks / 2`, at
  least 1, at most `max_job_bytes = min(answer_bytes, ci_budget / 5)`
  (`brief.rs:394-401`, `domain.rs:700-703`). In the engine's domain
  world that is 76 bytes: `read_bytes` 512, CI budget 384, the client's
  `answer_bytes` 4,096 (`tests/engine/forge/src/world.rs:15`). The
  section with the new words is then 352 bytes, assuming 03.1 names
  `build`'s workflow step `Run build` (9 bytes). That is within the CI
  budget of 384, so it is not cut, and it shows both the step's line and
  `[log truncated before this]`, since the step's log, 88 bytes, is
  longer than the 76 kept. A name of up to 41 bytes keeps the section
  uncut. With no failed step the section is 349 bytes.
- **The system world takes the engine domain world's brief limits.** Its
  `configured_policy` uses walking's brief limits
  (`tests/engine/domain/src/walking.rs:142-159`: `read_bytes` 256, CI
  128, `brief_bytes` 384), which equal the shipped profile's
  (`crates/temper-service/src/limits.rs:212-243`, "from the root
  world"). Under those limits the new section is 301 bytes and passes
  the 256 bytes the gather holds. With the gather's cut of a CI section
  from its start (below), the gather keeps its last 256 bytes, and the
  CI cut to 128 (`held.rs`, from the start) its last 128: the log's end
  and the closing line, but not the step's line, which the story
  asserts. 05.1 therefore sets the system world's `read_bytes` 512,
  CI 384 and `brief_bytes` 768, as `tests/engine/domain/tests/forge_routes.rs:103-107`
  sets them for its change stories, beside walking's values, with the
  reason. These figures are computed from the code, not run.
- **The gather cuts a CI section from its start** (domain/engine.md, 9;
  design question 4). The CI section's `BriefWords` keeps its end: past
  its room, `append` drops its first byte for each byte it adds, and
  `brief_part` keeps its last `bytes`; each byte dropped is counted in
  `left`, as today, so `held.rs`'s render says how much was cut. Every
  other section keeps its start, as at the base.
- **The shipped profile's brief limits do not change** (design question
  4): its numbers wait for the first deployments' measurements
  (configuration.md, section 11, "Profiles' numbers"). Under them a
  failed check's section keeps its log's end, and may lose its step's
  line.

## Rules that bind it

- **A file over 800 lines at the base grows by at most 40 lines in one
  increment** (code-organization.md, 6).
  `tests/engine/domain/tests/forge_routes.rs` has 2,886: `git diff
  --numstat origin/main...HEAD -- tests/engine/domain/tests/forge_routes.rs`
  prints added and removed lines, added minus removed is at most 40, and
  the landing's report gives the two numbers. 05.1 stays within it by
  moving the two brief tests' shared setup into one helper.
- **No expected value computed by the code under test; a fixture's
  literal is read by hand** (code-organization.md, 5). The fake's step
  name and log lines in the tests below are temper 03.1's, copied from
  `testing/temper-fake-forge-domain/src/` by hand. Where 03.1 landed
  other words than this file assumes, the test takes 03.1's, and its
  name and assertions stay.
- **Limits in tests come from the shipped profile's builders; a test
  that needs another value names it beside the profile's, with the
  reason** (code-organization.md, 4; development.md, 5.6). That covers
  the system world's three brief limits above.

## Increments

### 05.1 The CI section names the failed step and the end of its log

Scope: in the CI section of a repair's brief, each failed check whose
job was read now has the failed step's line and the end of its log,
or, when no step failed, a line saying so and the end of the job's log.
A truncated log is marked before the bytes kept. A failed read and an
unpinned status keep their lines (domain/forge.md, 8.4 and 2, "Brief
sections"). The CI section's gather keeps its end, as its kind's cut
does (domain/engine.md, 9). The engine's domain world's brief stories
cover a failed workflow step and a cancelled job. The system world's
failed-CI story asserts the step's line and the log's end.
Needs: nothing beyond the session's start, which is temper 04.1:
`Answer::Job`'s `step`, and `translate.rs`'s job answer carrying it.
Through 04.1 it also needs temper 03.1: the fake's three steps, a log
for each, and `Failing::NoStep`.
Files:
- `crates/temper-engine-domain-forge/src/brief.rs` (553 lines):
  `brief_answer`'s `BriefStage::Job` arm (`:306-322` at the base; 04.1
  binds `step` there and ignores it); `BriefWords` (`:29-39`), `append`
  (`:61-69`), `brief_part` (`:79-91`) and where the CI section's words
  are made. Tests: `crates/temper-engine-domain-forge/src/tests.rs`.
- `tests/engine/domain/tests/forge_routes.rs` (2,886 lines, at most 40
  added net): the two brief tests (`:1177-1294`), rewritten on helpers
  placed beside `failed_ci_with_delayed_brief` (`:1296`), and one new
  test. This file is shared with client-api's temper sessions (README,
  Rules): a rebase conflict is resolved before merging.
- `tests/system/src/forge/stories.rs` (326 lines): the CI assertion of
  `a_smith_change_with_failed_ci_is_repaired_reviewed_and_landed`
  (`:256-262`).
- `tests/system/src/forge.rs` (502 lines): `World::configured_policy`'s
  `with_change` branch (`:70-103`), three brief limits. They reach each
  story that `start_change_world` (`tests/system/src/forge/stories.rs:62-63`)
  serves: beside the failed-CI story (`:209-210`),
  `the_smith_delegate_input_creates_the_producer` (`:152-153`) and
  `a_small_fix_made_in_a_smith_chat_lands_through_the_fake_forge`
  (`:176-177`), which this increment does not edit; they run unchanged
  in the gate.
- The literals' users, found by `git grep -n -F -e 'Job log tail' -e
  '[job log truncated]' -e 'failing check build 3' -- crates tests
  testing`: at the base, `brief.rs:309,313`, `forge_routes.rs:1236,1237`
  and `stories.rs:259`. The fake's own `failing check` lines,
  `testing/temper-fake-forge-domain/src/actions.rs:200-210` and
  `src/tests/actions.rs:226-228` (`WORKFLOW_LOG`; since temper 03.1), and `testing/temper-fake-forge-protocol/src/tests/http.rs:243`, which needs no change, are 03.1's. `[job log could not be read]`
  (`brief.rs:316`, `forge_routes.rs:1288`) and `[no pinned job log for
  this head]` (`brief.rs:403`) are kept. The agent's `job` tool and its
  `[log truncated]` (`crates/temper-engine-smith/src/answers.rs:527-535`)
  belong to temper 04 and are not changed here.
Shape:
- `temper_engine_domain_forge::brief::brief_answer`, the arm `answer if
  fetch.stage == BriefStage::Job`. For `Ok(client::api::Answer::Job {
  attempt, step, log, truncated }) if attempt.head == fetch.head.0`, it
  appends, through `append` and in this order:
  - with `Some(client::api::Step { number, name })`: `b"Failed step: "`,
    `decimal(u64::from(number))`, then `b" "` and `name` when `name` is
    not empty, then `b"\nEnd of its log:\n"`;
  - with `None`: `b"No step failed. End of the job's log:\n"`;
  - `b"[log truncated before this]\n"` when `truncated`;
  - `log`, then `b"\n"`.

  Any other answer or error appends `b"[job log could not be read]\n"`,
  as at the base. After the arm, `next_failed_job` and the closing
  `Repair the failed check.\n` are unchanged.
- The gather's cut (design question 4): `BriefWords` gains `tail:
  bool`, true for the CI section's words and false for every other
  section's. With `tail`, `append` at a full list drops the list's first
  byte for each byte it adds, counting it in `left`; `brief_part` keeps
  the last `bytes` of the words, the bytes before them counted in
  `left`. Without it both are as at the base.
- In the engine's domain world, the section reads as follows. `<head>`
  is the head's first 8 bytes in hex. `Run build` and the log lines are
  the assumed 03.1 literals: the workflow step's log is today's four
  lines, `failing check build 0` to `3`, 88 bytes, of which the last 76
  are kept.

  ```
  Repair the failed check at change head <head>. Push a new head and wait for checks there.
  Failed check: build
  Description: CI failed
  Link: /org/repo/actions/runs/<run>/jobs/0
  Failed step: 1 Run build
  End of its log:
  [log truncated before this]
  k build 0
  failing check build 1
  failing check build 2
  failing check build 3

  Repair the failed check.
  ```
- `tests/system/src/forge.rs`, `configured_policy`, `with_change`:
  `limits.brief.read_bytes = 512`, `limits.brief.budgets.ci = 384`,
  `limits.brief.brief_bytes = 768`, under one comment giving the reason.
  The comment says that the CI section holds its head, the check's
  three lines, the failed step's line and a 76-byte log end (352 bytes),
  as the engine domain world's change stories size it
  (`forge_routes.rs:103-107`), and that walking's 128 and 256 lose the
  log's end. The limits apply to the three stories of
  `start_change_world` (Files); the two this increment does not edit
  run unchanged in the gate.
Test shape:
- `crates/temper-engine-domain-forge/src/tests.rs`, one test of the
  gather's cut, under 20 ms, `BriefWords`, `append` and `brief_part`
  made `pub(crate)` where they are not:
  `a_ci_sections_words_keep_their_end_and_others_their_start`, two rows
  on words of room 8 given `abcdefghij`: with `tail`, `cdefghij` and
  `left` 2, and `brief_part` at 4 bytes `ghij` with `left` 6; without,
  `abcdefgh` and `left` 2, and `brief_part` at 4 bytes `abcd` with
  `left` 6. A test at the base that asserts a CI section's first bytes
  past its room takes its last bytes, and the landing's report names
  it.
- `tests/engine/domain/tests/forge_routes.rs`, on `change_world(false,
  0, None)` (`:633`) and `World::external` (`:363-390`),
  `World::tick` (`:495`) and `fail_job_reads` (`:414-425`). The limits
  are `configured_policy`'s `with_change` values (`:103-107`) and the
  client's `answer_bytes` 4,096 (`tests/engine/forge/src/world.rs:15`),
  as at the base. The two base tests' shared setup moves into
  `failed_ci_repair`. The base's first test is renamed and keeps its
  setup and its workspace assertion. The second keeps its name and its
  expected values. `failed_ci_with_delayed_brief` (`:1296`) stays as it
  is: it sets `slow_brief_reads` before the producer ends, and its
  repair is not assigned within the helper's 200 ticks.
- Seeding `Failing::NoStep`: through what 03.1 lands. This file assumes
  a call on a built fake, `fake::set_failing(&mut world.fake,
  b"org/repo", b"build", raw::Failing::NoStep)`, after the name and
  shape of `fake::set_reachable`
  (`testing/temper-fake-forge-domain/src/scenario.rs:88`). The test
  calls it after `change_world` and before the head is pushed. If 03.1
  seeds `Failing` only through `raw::Checks`, the test seeds it as
  03.1's own tests do, within the 40-line rule. If that breaks the rule,
  it is a question. The file also assumes that a context seeded
  `NoStep` still fails CI: its status is `Failed` by the CI user, and
  its job's steps are `success`, `cancelled`, `cancelled`.
- The helper's `Failed` status for `build`, written by user 1 as the
  base's two tests write it (`forge_routes.rs:1190-1197`, `:1253-1260`),
  stays. It is no stand-in: the fake's CI writes its own status on the
  head too (`change_world(false, 0, None)`, `passes` 0), as CI user 3
  (`tests/engine/forge/src/world.rs:599`), and 03.1's job read rests on
  that one. The Don't below forbids only a status written in place of
  the fake CI's own.
- The helpers and the reference test, in full:

  ```rust
  /// temper 03.1's fake, read from `testing/temper-fake-forge-domain` by
  /// hand: the context `build`'s workflow step, number 1, and its log's last line.
  const BUILD_STEP: &[u8] = b"Failed step: 1 Run build\nEnd of its log:\n[log truncated before this]\n";
  const BUILD_LOG_END: &[u8] = b"failing check build 3\n";
  /// temper 03.1's fake: the last line of `build`'s job log with no failed
  /// step, its `Complete job` step's slice under `Failing::NoStep`, read by hand.
  const JOB_LOG_END: &[u8] = b"job build cancelled\n";

  /// Push the producer's head, fail `build` on it, end the producer, and
  /// step until the repair is assigned.
  fn failed_ci_repair(world: &mut World, producer: &engine::Assignment, branch: &[u8]) -> engine::Assignment {
      assert!(matches!(
          world.external(2, raw::Op::Git(raw::Git::Create { branch: Box::from(branch), commit: 1 })),
          raw::Answer::Branch(raw::Created::Created)
      ));
      let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", branch, b"file", b"first", 1)
          .expect("producer pushed its branch");
      world.external(
          1,
          raw::Op::Write(raw::Write::Status {
              commit: head,
              context: Box::from(&b"build"[..]),
              state: raw::Check::Failed,
          }),
      );
      let event = world.protocol.answer(
          Token::new(7),
          producer.task,
          producer.attempt,
          5,
          tasks::End::Finished {
              result: tasks::TaskResult::Change {
                  connector: 0,
                  kind: 2,
                  resource: u64::from(forge_world::REPO.repository),
                  words: Box::from(&b"pushed"[..]),
              },
              cancel_delegates: false,
          },
          engine::Parts::none(),
      );
      world.send(event);
      for _ in 0..200 {
          world.tick();
          if world.assigned.len() >= 3 {
              break;
          }
      }
      world.assigned.get(2).expect("failed CI assigned a repair").clone()
  }

  /// The words of a repair's CI section.
  fn ci_words(repair: &engine::Assignment) -> &[u8] {
      repair
          .sections
          .iter()
          .find_map(|section| match &section.body {
              engine::BriefBody::Text(words)
                  if section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Ci) =>
              {
                  Some(words.as_ref())
              }
              engine::BriefBody::Text(_) | engine::BriefBody::Missing(_) => None,
          })
          .unwrap_or_else(|| panic!("no CI section: {:?}", repair.sections))
  }

  fn contains(words: &[u8], part: &[u8]) -> bool {
      words.windows(part.len()).any(|window| window == part)
  }

  #[test]
  fn a_failed_ci_repair_brief_names_the_failed_step_and_its_logs_end() {
      let (mut world, _chat, producer, branch) = change_world(false, 0, None);
      world.external(
          1,
          raw::Op::Write(raw::Write::Status { commit: 1, context: Box::from(&b"build"[..]), state: raw::Check::Passed }),
      );
      let repair = failed_ci_repair(&mut world, &producer, &branch);
      assert!(
          matches!(repair.workspace.repositories[0].start, engine::ForgeStart::Branch(ref name) if name.as_ref() == branch.as_ref())
      );
      let words = ci_words(&repair);
      assert!(contains(words, BUILD_STEP), "{}", String::from_utf8_lossy(words));
      assert!(contains(words, BUILD_LOG_END), "{}", String::from_utf8_lossy(words));
  }
  ```

  The three constants are 03.1's literals (`03-fake-jobs.md`, "Choices
  this plan makes", the table of steps and slices): with `NoStep`, 04.1's
  `translate.rs` answers `build`'s three slices joined, whose last line
  is the completion step's `job build cancelled\n`. Where 03.1 landed
  other words, the author replaces `Run build`, `failing check
  build 3` and `job build cancelled` with what it landed, read by hand.
  `1` and the words around them are 05.1's.
- Tests, each under 0.5 s (a focused world; the base's two took 0.04 s
  each from their built binary):
  - `a_failed_ci_repair_brief_names_the_failed_step_and_its_logs_end`
    (above; it was `a_failed_ci_repair_brief_contains_the_bounded_job_log`);
  - `a_failed_ci_with_no_failed_step_briefs_the_end_of_the_jobs_log`: it
    seeds `build` `NoStep`, then calls `failed_ci_repair`. Its CI words
    contain `No step failed. End of the job's log:\n[log truncated before
    this]\n` and `JOB_LOG_END`, and do not contain `Failed step: `;
  - `an_unreadable_failed_job_log_is_named_and_repair_still_runs`: it
    sets `world.fail_job_reads = true`, then calls `failed_ci_repair`.
    Its CI words contain `[job log could not be read]`, `Description:`
    and `Link:`, as at the base.
- `tests/system/src/forge/stories.rs`: the CI assertion of
  `a_smith_change_with_failed_ci_is_repaired_reviewed_and_landed`
  (`:256-262`) checks for `b"Failed step: 1 Run build\nEnd of its
  log:\n[log truncated before this]\n"` (03.1's name, as above) and
  `b"failing check build 3\n"` in the repair's CI section, in place of
  `failing check build 3` alone. The rest of the story is unchanged. It
  takes the three brief limits above, which give the same numbers as
  the domain world: `max_bytes` 76, a 352-byte section, uncut. If a
  bound of the system world (smith's `run_bytes`, the journal's) refuses
  the larger brief, that is a question, with the refusal. No other limit
  moves to make room.
Proof: the system world, once:
`cargo nextest run -p temper-system-world -E
'test(a_smith_change_with_failed_ci_is_repaired_reviewed_and_landed)'`.
This is the joint layer: the real engine and worker, smith's agents
hosted by the worker, and the fake forge, joined by the protocol
layer's translation. The fake's CI fails `build`'s workflow step on the
producer's head. The repair the engine assigns has a CI section with
`Failed step: 1 Run build`, `End of its log:`, `[log truncated before
this]` and the log's last line `failing check build 3`. The
smith-hosted repair then pushes, and the change lands with `repairs ==
1`. The test runs in under 0.5 s; it took 0.08 s at the base, from its
built binary. Nothing higher can fail CI: the process tier has no runner
(decisions.md, 10), and the one-process world never fails CI
(`findings/fake-forge-and-brief.md`, 6).
Don't: change the share (`brief.rs:394-401`), `max_job_bytes`
(`domain.rs:700-703`), the cut of sections other than CI's at the
gather, or the CI cut in the brief (`held.rs`); change the shipped
profile's brief limits (`crates/temper-service/src/limits.rs:212-243`)
or walking's (`tests/engine/domain/src/walking.rs:142-159`); write the
job's other steps or any step's status into the brief; read a second
log or the job again from the brief; answer `Answer::Job` by hand in a
world, as `fail_job_reads` answers its error: the step comes from the
fake through `translate.rs`; write a CI status in a test in place of
the fake CI's own (the helper's `Failed` by user 1, as the base's tests
write it, stays).
Size: at most 50 lines of production code: 25 more than planned, for
the gather's cut of a CI section (design question 4).
Removes: the words `Job log tail:` and `[job log truncated]`; the test
name `a_failed_ci_repair_brief_contains_the_bounded_job_log` (renamed);
the two base tests' repeated setup (moved into `failed_ci_repair`).

## Interface sketches

None. 05.1 adds no type. It reads temper 04.1's `client::api::Step {
number: u32, name: Box<[u8]> }` and `client::api::Answer::Job { attempt,
step: Option<Step>, log, truncated }` (code-organization.md, 2), and, in
its tests, temper 03.1's `raw::Failing::NoStep`. The words it defines
are the byte literals of its Shape (code-organization.md, 3: "the
brief's words for the failed step").

## Questions to raise when reached

- **The shipped profile's CI section cannot hold the step's line and its
  log's end** (design question 4): answered on 2026-10-16. The gather
  cuts a CI section from its start, as the brief does (Choices; Shape,
  "The gather's cut"), so the log's end is the last thing lost; the
  shipped profile's numbers do not change.
- **03.1's `NoStep` job does not fail CI, or its log is not served.**
  `a_failed_ci_with_no_failed_step_briefs_the_end_of_the_jobs_log`
  needs a `Failed` status by the CI user on a job whose steps name no
  failure. forge.md, 3.2 reads such a job "whole, its end the same way".
  If 03.1's fake gives that context no failed status, or no log (a
  `MissingJob`), raise it with 03.1's status and answer. Do not write
  a status by hand in place of the fake CI's own; the helper's `Failed`
  by user 1 stays, as the base's tests write it.

## Done when

Each command runs once, after 05.1 has landed, in `~/src/rust/temper`
after `git fetch origin`.

1. The step landed: `c=$(git log origin/main --all-match
   --grep='^Plan: forgejo-17$' --grep='^Step: temper-05.1$' --format=%H
   -1); echo $c` prints a commit id.
2. The old words are gone: `git grep -n -F -e 'Job log tail' -e '[job
   log truncated]' origin/main -- crates tests` prints nothing.
3. The new words are written once each: `for w in 'Failed step: ' 'End
   of its log:' "No step failed. End of the job's log:" '[log truncated
   before this]' '[job log could not be read]' '[no pinned job log for
   this head]'; do git grep -c -F "$w" origin/main --
   crates/temper-engine-domain-forge/src/brief.rs; done` prints six
   lines, each ending `:1`.
4. The system story asserts the step: `git show
   origin/main:tests/system/src/forge/stories.rs | grep -c -F 'Failed
   step: 1 '` prints 1.
5. The 40-line rule (code-organization.md, 6), checked before landing,
   in 05.1's worktree: `git diff --numstat origin/main...HEAD --
   tests/engine/domain/tests/forge_routes.rs` prints added and removed
   lines, with added minus removed at most 40; the landing's report
   gives the two numbers, and this item reads them there.
