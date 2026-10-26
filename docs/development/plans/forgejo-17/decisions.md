# forgejo-17: decisions

2026-10-16. The decisions the plan rests on, then the run's rulings
(appended by the supervisor), then what the user may veto.

## The plan's decisions

1. **Scope: the rows `STATUS.md` names "the next plan".** `forge.md`,
   sections 1, 10 and 13 (the pin on Forgejo v17.0 and forgejo-runner
   v13.2, the throwaway Forgejo and runner, the conformance's
   transcripts and facts), 3.2 (`Read::Job`'s failed step and its log's
   end; the comparisons' `files` and `verification`) and 9 (the fake
   forge's jobs with steps); `domain/forge.md`, 8.4 (the repair's brief
   with the failed step and its log's end); `git.md`, section 9
   (`tools/git-transcripts`' records and facts on v17.0). The agent's
   `job` tool shows the failed step too (`domain/forge.md`, section 2,
   "Tools": "a failed job's output, its failed step's end"): it renders
   the same answer. Not in scope: webhooks (`forge.md`, section 5),
   GitHub (section 11), and `draft/ci-on-forgejo-17.md` (re-runs, a
   stall's reasons, the agents' step and search tools).
2. **The pin is Forgejo 17.0.0 and forgejo-runner 13.2.0,** the newest
   of each on 2026-10-16 (`findings/`, the spike's section 1):
   - `forgejo-17.0.0-linux-amd64`, SHA-256
     `5a3b07e124ed70b4e46ee3939515f003bd3cc85352257a9a4607a527251042ca`;
   - `forgejo-runner-13.2.0-linux-amd64`, SHA-256
     `fadaec897f5e6641c363f87ecaf75f866f99c81b7097ec6b433e4c48123fcad1`;

   both at `~/src/rust/tmp/temper/forgejo/{v17.0.0,runner-v13.2.0}/` on
   x1, each with its `.sha256`. A later patch released during the run
   does not move the pin: moving it is a constant, the transcripts and
   the facts (`forge.md`, section 10), for a later change.
3. **The run's machine is x1,** beside client-api's run on carbon-forge
   (the user, 2026-10-16): x1 holds both binaries, and Forgejo v16's
   support ends on 2026-10-29. The two runs share temper's `main`;
   Changes are rebased before merging.
4. **The store's version does not move in this plan** (the user,
   2026-10-16). `JobAnswer` changes in place in the schema
   `origin/main` holds at the increment's landing: `store-v1.schema` at
   the base, `store-v2.schema` once client-api's temper 04 has moved it.
   `STORE_VERSION` stays as `main` has it. client-api moves it once
   (its README, "Format versions"), and that one move covers both
   plans' changes: no temper store exists outside the tests' goldens,
   and client-api's checks (`STORE_VERSION: u32 = 1` on `main`) keep
   reading as its plan says. `docs/design/store/README.md` moves the
   version "once per plan", and says how plans that run at once share
   the move (design question 3): "the plan whose README names it moves
   the version, and the others change their records in place in the
   schema `main` holds, before the move or after it".
5. **One constant pair pins both binaries,** in
   `testing/temper-throwaway-forgejo/src/binary.rs`: `VERSION`
   (`"17.0.0"`) and `RUNNER_VERSION` (`"13.2.0"`), each checked exactly
   against the binary's first `--version` line, which the tool, the
   throwaway instance and the process tier share (`forge.md`, section 10:
   "a constant the throwaway Forgejo and the tool share"). Today the
   runner's check is a substring of its own in the tool
   (`findings/tools.md`, 1).
6. **The answer names the failed step as a value of its own:**
   `Answer::Job { attempt, step: Option<Step>, log, truncated }`, with
   `Step { number: u32, name: Box<[u8]> }` in the forge client's
   `api`. `step` is none when no step failed. The job's other steps are
   not kept: the design reads only the failed one (`forge.md`, 3.2).
7. **The failed step is found by the component,** from
   `actions/jobs/<job>`'s steps: the first whose `status` is `failure`
   (`forge.md`, 3.2, "A failed job's log is its failed step's end").
   Nothing above the component reads the steps.
8. **The log's end is a suffix range, and a `200` is read through,**
   keeping its last `max_bytes` bytes as it streams (`forge.md`, 3.2).
   The component's range becomes a choice of two, the head (`bytes=0-N`,
   a file's) and the end (`bytes=-N`, a log's); `truncated` is
   `Content-Range`'s first byte above 0, or bytes dropped from a `200`.
9. **The fake forge's jobs have three steps,** `Set up job`, one
   workflow step and `Complete job`, each with a log slice of its own.
   Where a job fails is seeded per context: the workflow step (the
   default), or no step (a cancelled job: its workflow and completion
   steps `cancelled`). The completion step reports `failure` whenever a
   step failed, as Forgejo's does (`forge.md`, section 9). The fake
   models only what v17 was seen to do or what the design names: a
   set-up failure and a completion failing alone are left to the
   component's step tests, on documents, where the rule "the first
   whose `status` is `failure`" is decided.
10. **The proofs.** The real side is reached at three places: the two
    recording tools, on the throwaway Forgejo and runner (sessions 01
    and 02); the conformance's component mode, the real component on
    the throwaway Forgejo (`forge.md`, section 10), for `Read::Job` and
    the comparisons; and the process tier (`scripts/proof-one-process.sh`),
    the real binary on v17, once, for the pin. The joint world of the
    forge component (`tests/engine/protocol-forge`, the component on
    the fake's HTTP layer) holds `Read::Job`'s story against the fake;
    the engine's domain world and the system world hold the brief's.
    The process tier has no runner and fails no CI: it proves the pin,
    not the job read.
11. **The plan's sessions are mostly in a line:** the pin, then the
    documents, then the fake's jobs and the component's read, then the
    brief. Only git's transcripts run beside the documents. It is a
    small plan; nothing is gained by cutting the meeting point finer.
12. **No repin:** temper is the last repository of the workspace's
    order, and the plan changes no other repository.
13. **Kept small.** `forge.md`, section 13's "that a job waiting for a
    runner has no steps" is not checked here: nothing this plan builds
    reads a waiting job, and the check needs the conformance to hold its
    runner across its run; it goes with the stall's reasons, which read
    pending jobs (`later.md`). The design questions found while planning
    went to the designer before the run (`~/src/rust/tmp/temper/forgejo-17-plan/design-questions.md`,
    not versioned), and the designer answered all six on 2026-10-16.
    The session files cite each as "design question <N>", in that
    file's numbering:
    1. a step's name past `string_bytes` is carried empty, the step
       named by its number (forge.md 3.3; temper 02.1, 04.1, 05.1);
    2. a job's log is read within `answer_bytes` less its step's name,
       as a file's is less its path (forge.md 3.3; temper 04.1);
    3. plans that run at once share one move of `STORE_VERSION`
       (store/README.md, section 4; decision 4);
    4. the gather cuts a section in its kind's order, a CI section from
       its start; the shipped profile's numbers do not change
       (domain/engine.md, 9; temper 05.1);
    5. a log read through is bounded by `body_bytes_in` like any body,
       and past it malformed, `InvalidAnswer` (forge.md 3.2, 3.3; temper
       04.2, as planned);
    6. a job's steps go up as rows as they stream, and the component
       keeps the failed one, so no bound counts them (forge.md 3.2;
       temper 02.1, 03.1, 04.1).

## Rulings

(The supervisor appends the run's rulings here.)

101. **temper 01.1's process tier without the two temper open tests, until the print bounds are fixed** (plan defect, temper 01's block of 2026-10-16T14:11:10Z).
     An interim ruling, while the designer answers the question of
     2026-10-16T14:19:38Z. The panic is on origin/main, not in 01.1:
     `crates/temper-client-shell-terminal/src/terminal.rs:72`,
     `Print::new(view_limits).expect("print bounds")`, came with f6f01026f
     (client-api's temper 01.3), and temper's own view limits put
     skein-ui-text's worst-case print at 257,411,441,911 bytes, past the u32
     `Print::new` takes. 01.1's branch changes no client crate and no
     `Cargo.lock`. Every `temper open` panics so, on any Forgejo.

     1. temper 01.1 lands with its process tier run without the two `temper
        open` tests (`temper_open_signs_in_prints_the_inbox_and_exits_on_q`,
        `temper_open_adopts_a_repository_by_commands`), as `temper/01-pin.md`,
        01.1's Proof, item 2, now says:

        ```sh
        TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 sh -c 'export NEXTEST_DOUBLE_SPAWN=0; cargo build -p temper --bin temper && cargo nextest run -p temper --profile process --no-fail-fast -E "not test(/^temper_open_/)"'
        ```

        It exits 0, and its log's nextest `Summary` line reads `4 tests run:
        4 passed`. Run it once; the landing report names its log. Nothing
        else is wanted for it.
     2. The fix is not 01.1's: no change to `crates/temper-client-*`,
        `crates/temper/src/open.rs` or `Cargo.lock` in session 01. It comes
        as a supplement step once the designer has answered; that step's
        proof is the full `scripts/proof-one-process.sh`, `6 tests run: 6
        passed`, and README's "Done when" item 9 is unchanged.
     3. If any of the four tests fails, or the run panics elsewhere, block
        again with the log.

102. **temper 02's Shapes against main** (ready check of temper-02.1).
     temper/02-documents.md corrected against origin/main b6a69057 and
     temper 01.1's and 01.2's landings (two ready checks, directly;
     with these corrections applied, 02.1's code passed fmt, clippy
     `-D warnings`, nextest 1257 passed, fuzzy 46 passed):
     - Choices: steps in a listing's rows are read and dropped, not skipped.
     - 02.1's Files: `decode/walk.rs:363-367` (the `Read::Body` cap), and the
       `d::Row` matches `actions.rs:233-242`, `contents.rs:207-217`,
       `reads.rs:388`, `:624-629`, `:708-716`, `writes.rs:196-207`.
     - 02.1's Shape: a job's end answers its row only when the root is
       `Jobs` (the base answers one always, `client/actions.rs:78-80`);
       `Row::Step` in `contents.rs` (`InvalidAnswer`), `writes.rs` (nothing),
       `long` (`false`). Its Don't no longer forbids `Row::Step`.
     - 02.1's throwaway test: `.expect("UTF-8 request")` for `.unwrap()`,
       which the crate's `unwrap_used` denies (`Cargo.toml:20`).
     - The recorder: `fixture.rs:80`, `cases.rs:24`, `runner.rs:169` take
       `Option<Range>`; `cases.rs:201`, `runner.rs:104` pass `Range::Head(3)`.
     - 02.2: the fake's flag check goes in a helper outside `answers::read`
       (96 of clippy's 100 lines at the base); the forgejo test writes its
       two targets as byte literals (`format!` is disallowed).
     A further copy of a corrected name is corrected the same way, and listed
     in the commit body.

103. **02.1's Size at most 370 lines** (plan defect, temper 02's block of 2026-10-16T14:44:58Z).
     02.1's Size is at most 370 lines, not 290, with no change of scope and
     no split. `temper/02-documents.md` said "Size: at most 290 lines
     (documents 45, the decoder 50, the server 30, the request and the
     encoder 10, the exhaustive arms 50, the recorder 15, the conformance
     90)"; the parts are a guide, and the total is the rule. It is counted
     once before landing by code-organization.md, section 6's command, less
     the three doc comment lines steer 1 corrects; the landing report gives
     the number. Past 370, block again with the count by file. Unchanged:
     02.1's Files, Shape, Don't and Proof; 02.2's Size, at most 70 lines;
     the 800-line file rule, which no file of 02.1 meets (the largest it
     grows, `document.rs`, has 340 lines at the base).

104. **The print's room: skein-00.s1 and temper-01.s1** (design question, temper 01's block of 2026-10-16T14:11:10Z).
     The designer's answer, skein `054c06e1` and temper `551b1d2d`; the two supplement steps are skein-00.s1 and temper-01.s1:


     Skein's, not a cap (skein `054c06e1`, temper `551b1d2d`): temper's view
     limits stay, and skein's text shell stops reserving their worst case.
     `ui-text.md`, 3: "`worst_case` is what the host counts in its memory
     [...], never what the shell reserves [...] Each print's bytes are
     reserved as it is made, by `worst_case`'s formula over that tree
     alone"; past a u32 the page prints `NAME: page too long to print`; 4:
     the output in pieces of 64 KiB, or the longest line with a node's text
     and 64 bytes. temper's `client.md`, 5.4: "a lower cap would be a guess,
     which skein-ui's builder turns into a panic on the page past it"; 10: a
     unit test makes the terminal from `limits::native()`'s limits.

     forgejo-17 lands it in two supplement steps (your names; the skein
     one's `Step:` starts `skein-`, which client-api's floor query greps):
     1. **The skein step**, needing nothing: `ui-text.md`, 3, 4 and 8, in
        `crates/skein-ui-text/src/render.rs` (`Print::new` reserves no text;
        `render` reserves by the tree's own room) and `terminal.rs`
        (`Terminal::new`'s room, not `worst_case`'s; a print's rest waits as
        the print and an offset). API unchanged: `Print.text: List<u8>`,
        `worst_case`'s `(u32, u64)`, both constructors. Proof: skein's gate,
      . Don't: name a downstream repository. Size: at most 80
        production lines.
     2. **temper-01.s1**, after it: first `cargo update -p skein-lib
        --precise <its landing>`, `Cargo.lock` alone, none when temper's
        skein pin already holds it (`merge-base --is-ancestor`); then the
        unit test `the_native_limits_make_the_terminal`
        (`crates/temper-client-shell-terminal/src/tests.rs`): `Loop::new`
        with `limits::view(&limits::native())`'s tree limits returns, its
        `worst_case()` `Some`. No other temper code. Proof: the full
        `scripts/proof-one-process.sh` on v17, `6 tests run: 6 passed`,
        ending ruling 101. For this step only, it replaces decisions.md 12
        and "This plan changes no dependency": a `Cargo.lock` conflict takes
        `origin/main`'s and moves again (development.md, 5.7).

     client-api never moves temper's skein back (README, "Pins", "The
     print's room": each move "goes to the later of its target and the
     floor"); its 03.1 needs the skein step and keeps the test on
     `Terminal::new`, its only check under its ruling 111. Its run reads
     this in its files alone: a one-line push to the user naming temper
     `551b1d2d` reaches its supervisor sooner. Unchanged: "Done when", 9.

105. **temper 03's Shapes against main** (ready check of temper-03.1).
     temper/03-fake-jobs.md corrected against origin/main 3ed817888 (a ready
     check, : with these corrections, 03.1's types, `op`
     without `limits`, the domain's reference test and both HTTP tests
     passed clippy, and 474 tests of the fake forge, the forge worlds, the
     domain, system, conformance and joint worlds passed, fuzzy 34) and 02.1's
     landing 1b40922a1:
     - Files: the fake protocol's `src/tests/routes.rs:1`, whose fixtures'
       `limits` import goes once `op` loses `limits`; and
       `tests/engine/forge/client/tests/simulation.rs:495-505`, which passes
       `Answer::Job` with the workflow step's log in place of `Answer::File`.
     - translate's arm is `byte_answer(asked, workflow_log(&steps))`, with
       `fn workflow_log(steps: &[forge::Step]) -> &[u8]`, a loop, beside
       `byte_answer`: `answer` is 92 of clippy's 100 lines.
     - `bounded::answer` (`response.rs:593`) is at 100 lines: the job is
       priced in a new `Budget::steps(&[a::Step])` beside `files`, and the
       `Runs` and `Jobs` arms fold into a new `Budget::rows(len)`.
     - translate's callers include `tests/engine/forge/src/world.rs:506,520`
       (the same file through `#[path]`); the `416` grep prints three lines.
     - The fake protocol's line numbers are 3ed817888's, before 02.1; the
       file names 02.1's twelve one-line insertion points.
     Noted, unchanged: translate's read by job id drops the attempt, so a
     stale attempt reads the latest attempt's step log where the base
     answered `MissingJob`; no world test reads a stale attempt, and temper
     04.1 replaces the arm. A further copy of a corrected name is corrected
     the same way, and listed in the commit body.

106. **temper 04's Shapes against main** (ready check of temper-04.1).
     temper/04-job-read.md corrected against origin/main 49f589b32 (a ready
     check, : the sketch and 04.1's contents and smith reference
     tests passed clippy `-D warnings` with these corrections) and 02.2's
     landing df37f0329:
     - The sketch drops `#![expect(clippy::unused_self, ...)]`, never met.
     - 04.1's reference test names each row's log line whole, with `type
       StepRow`; `job(statuses)` pushes into a `skein_lib::List<u8>`
       (`format!` is disallowed in step crates).
     - `a_jobs_verified_log_uses_range_and_content_range_reports_more`'s
       request check (`src/tests/contents.rs:472`) reads `&step=1`.
     - translate: its callers include `tests/engine/forge/src/world.rs:506,520`;
       the arm calls `fn job_answer(asked, job, steps)` beside `byte_answer`
       (`answer` is 92 of clippy's 100 lines), which replaces 03.1's
       `workflow_log`, and first asserts the fake's job is the attempt asked:
       the read by id drops the attempt, and the component answers
       `MissingJob` for a stale one, which no world reads.
     - Line numbers: `actions.rs` is 371 lines since 02.1 (the run :270-284,
       the listing :285-309, `log` :357-370); `joint.rs:70-78`, called from
       `Joint::cutting` (:47); `runner.rs:68`; connection tests :217 and
       :285 pass a range, and `contains`/`raw_request()` are helpers of
       `src/tests/connection.rs`.
     A further copy of a corrected name is corrected the same way, and listed
     in the commit body.

107. **temper 04's Shapes after 03.1** (ready check of temper-04.1).
     temper/04-job-read.md re-checked after temper 03.1's landing 00f6dc640,
     for the parts that need it (translate's arm, the client-world tests, the
     joint test; directly, with a stand-in for 04.1's client change:
     workspace clippy `-D warnings` passed, 370 of 371 tests passed, the one
     failure the joint reference test that 04.1's component work makes pass,
     fuzzy 30 of 30):
     - translate's arm is `forge::Answer::Job { job, steps } =>
       job_answer(asked, &job, &steps)`; `byte_answer`'s `Read::Job` arm is
       at `translate.rs:508-517` since 03.1.
     - `simulation.rs` has 836 lines; its translate test is at `:495-514`.
     - `Joint::failing(seed)` takes its other arguments as `Joint::cutting(seed,
       0, no faults)` gives them; a `match` with a `None` arm is `if let ...
       else`.
     The `assert_eq!` on the attempt holds in every world: no world sets
     `reruns` and uses translate.

108. **temper 05's Shapes against main** (ready check of temper-05.1).
     temper/05-brief.md checked against origin/main 988ec48a5 (a ready check,
     : the sketch passed clippy `-D warnings`; on a stand-in for
     04.1, 275 tests passed, fuzzy 8 of 8, the domain world's CI section 352
     bytes as the Shape says): the fake's own `failing check` lines are at
     `testing/temper-fake-forge-domain/src/actions.rs:200-210` and
     `src/tests/actions.rs:226-228` since temper 03.1, beside
     `testing/temper-fake-forge-protocol/src/tests/http.rs:243`, which needs
     no change. Notes: skein's `List` removes no first item, so the CI cut
     shifts bytes with `get_mut`; the unit test needs `BriefWords`'s `bytes`,
     `left` and `tail` `pub(crate)`.

109. **Remove plan downstream's worktrees for the release** (process, the release's block of 2026-10-16T16:01:31Z).
     Remove the three worktrees `wt/smith-downstream`, `wt/jig-downstream`
     and `wt/temper-downstream` with their `Cargo.lock` changes: they are
     `plan downstream`'s type-check worktrees (skein's `scripts/dev/plan`,
     `downstream()`, `dir=$run/wt/$repo-downstream`), detached, and their one
     change moves each lock's skein source to skein's `main` for the check;
     nothing lands from them. Remove each with `git -C <its repository>
     worktree remove --force <path>`, then run `plan release` again. That
     `plan downstream` leaves them, and so blocks `plan release`, is a
     proposal for the retrospective, not a rule of this plan.

## For the user

- **The store's version (decision 4).** `docs/design/store/README.md`
  moves `STORE_VERSION` "once per plan, at the first landing that
  changes an encoding". With client-api running at once, this plan
  leaves the move to client-api and changes `JobAnswer` in place, as
  the user decided; the designer worded the README to match (design
  question 3, decision 13).
- **The pin (decision 2):** 17.0.0 and 13.2.0, not "the newest patch at
  the run": the transcripts and facts name one binary each.
