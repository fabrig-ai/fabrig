# forge-setup: decisions

The planner's decisions, each with its reason. Rulings and answers of the
run are appended after "For the user".

1. **The box first, on polling; webhooks join it.** Sessions are numbered
   in setup.md 14's order and start by availability. The box needs
   prepare, run and reconcile (temper 07 to 10, 13), and none of them
   needs webhooks, while the webhooks' listener waits for client-api's
   temper 05.1 (temper-service's `server.rs`, about 10-18 13Z) and
   temper's skein move. So `temper box` and the process tier on an owned
   Forgejo with CI (temper 10.2) land on polling, and webhooks reach them
   in temper 11. The throwaway Forgejo's hook (setup.md 14, item 3)
   serves the conformance's delivery facts (temper 04.1) alone: by the
   time the engine has a listener, the process tier runs on owned.
2. **temper's skein moves once, in temper 05.1,** after skein 01.2 and
   after the client-api ruling lands (temper's skein may be
   ahead of client-api's floor; its repins check `git merge-base
   --is-ancestor`; none moves skein back). The review chose this on
   2026-10-16; the ruling is client-api's ruling 154 (`c36858db7`), on
   `main` at the plan's base, so 05.1 waits for skein 01.2 and temper
   04.2 alone.
3. **No repin but temper's.** skein 01 adds `skein_lib::sha256` and moves
   three private copies onto it, keeping every public item. smith is
   left alone (smith/wall-clock runs on hetzner); jig's pins are
   client-api's to move; hearth's paused M1 plan takes skein at its next
   repin, and its SHA-256 step is this plan's skein 01.
4. **One pin, in temper-forge-forgejo** (setup.md 6.1; protocol.md 5):
   `VERSION`, `RUNNER_VERSION` and their checks move there from
   `testing/temper-throwaway-forgejo/src/binary.rs` (temper 03.1); the
   throwaway re-exports them. forgejo-17's rule that only the test crate
   names a version ends with this move.
5. **The shipped profile's new limits** (configuration.md 7 names them
   and gives no value): `setup.orgs` 8, `setup.deadline_ms` 120000,
   `setup.backoff_ms` 100, `setup.backoff_max_ms` 5000; `box.backoff_ms`
   500, `box.backoff_max_ms` 30000, `box.restarts` 5, `box.window_ms`
   300000, `box.grace_ms` 10000; `forge_component.hooks` 4,
   `hook_head_bytes` 8192, `hook_body_bytes` 1048576 (a push of many
   commits; the body streams, so it costs no memory), `hook_deadline_ms`
   10000, `hook_secret_bytes` 128 (reconcile draws 32 bytes, 64
   hexadecimal digits); `forge.client.settle_ms` 30000; the profile's
   sockets 48, so that `small` holds a document with both listeners
   (designer's answer 30). Each satisfies configuration.md 7's
   cross-checks. A test that needs another value
   names it beside the profile's (development.md 5.6).
6. **How the setup is checked** (the designer's answer of 2026-10-16;
   setup.md 8): temper-service hands `application.setup` and the
   `setup.`/`box.` overrides on undecoded, with the forges' and
   identities' members; the binary calls temper-forge-setup's and
   temper-box's checks for every command, `temper serve` included.
   `temper forge …`, `temper box` and `temper sign-in` read no private
   file to check.
7. **The lock and the exec** (the designer's answer; setup.md 6.4): one
   module of temper-forge-setup-shell, std's `File::try_lock` and
   `CommandExt::exec`, the lock handed to the exec'd process as its
   stdin. The module holds the lock and the exec alone; the
   preparation's commands run on skein-io's processes in the host's
   loop (the designer's answer 32). The boundary test that keeps
   `std::process` and file locks in that module lands with the module's
   first use (temper 08.1).
8. **The `state` file** is a skein-json document with a `version` member
   (1), the versions prepared, whether the first preparation ended, and
   Forgejo's secrets. Pre-release: a reader refuses another version.
9. **The box finds `temper`** by `std::env::current_exe()`, and gives
   each step the configuration's path made absolute against its own
   working directory.
10. **The upgrade comes last** (temper 15.1): a box at the pinned
    versions never meets it.
11. **Facts are named by group:** `setup-…` (admin routes, tokens, orgs,
    teams, hooks), `cli-…` (Forgejo's command line), `runner-…`,
    `delivery-…`, so "Done when" counts each group.
12. **The process tier aims at 180 s whole** (domain/testing.md 2.6;
    setup.md 13.4), CI, webhooks, the box and the upgrade included. If it
    goes past, the slowest story becomes a test of its own, or the aim
    moves, by a ruling for the user.
13. **The suites' aims for the whole run:** at the base (temper
    0ff45022, x1) the default suite takes 4.43 s and the fuzzy one
    24.22 s (medians of three, `findings/measurements.md`); the run adds
    at most 1.0 s to the first and 3.0 s to the second. No session has a
    share. skein's suites take what skein 01 adds, at most 0.5 s to its
    default suite.
14. **The owned example is a file,** `docs/examples/owned.json`, the
    whole document, with its non-secret files in `docs/examples/owned/`
    (the designer's answer 12), which the guide and the tests read; the
    guide is `docs/guide/box.md`.
15. **A hint's key is not used to skip a read.** setup.md 10 has each
    echo cost the keep one read, which the keep does today
    (`keep.rs:638-641`); the domain drops what it already knows after
    that read.
16. **No live service.** The Forgejo and runner are the pinned local
    binaries; the process tier's LLM is skein's fake peer; no agent
    signs in. The guide's `temper sign-in` is the person's, and its
    check stops before it.

17. **Session 07 is split, and the upgrade is a session of its own.**
    The drafts gave temper 07 589 lines, past development.md 5.2's 400;
    `forges[].user` as a file (drafted as 10.1) must come before the
    first command, since setup.md 8 requires it for a set-up forge. So:
    temper 07 is the crate, Forgejo's files, the configuration (07.3:
    `forges[].user` as a file, `application.setup`) and the preparation's
    machine (07.4, which waits for temper 03.3, the configuration filling
    that wait); temper 08 is `temper forge prepare`, `run` and the
    process tier's Forgejo; temper 15 is the upgrade, which nothing waits
    on.
18. **temper 06 waits for temper 04.1,** so that the fake's hooks follow
    the order and the mergeability the conformance recorded, not an
    expectation (forge.md 9: the fake answers what the facts record).
19. **The fake's deliveries are not sent to the hook reconcile made in a
    world:** temper 11.1 proves Forgejo's delivery to reconcile's org
    hook at the process layer, and the fake's world tests the listener
    with the hook its scenario names.
20. **The process tier's person** gets a token made on the admin's routes
    (designer's answer 14), as the bot does; nothing signs in with a
    password through the API.
21. **No live service, in the example either:** temper 14's test and
    its guide's script hand in an LLM account for skein's fake peer, as
    the process tier does; the guide names the borrowed-login way for
    people (designer's answer 13).
22. **The increments whose Size is the process tier's support** (temper
    08.3, 10.2, 11.1, 11.2, 13.2, 14.1) count that support's lines
    (`code-organization.md`, 6), since the plan's count leaves `tests`
    aside.

23. **temper 10 starts after temper 08.3,** so that its proofs read the
    owned Forgejo through the throwaway's `attach` (08.3), not through a
    second HTTP helper of their own.
24. **`box.` overrides are refused as unknown** until temper 13.1 wires
    temper-box's checks into the binary's; no document of the plan
    names one before.
25. **The private org's CI clones with the job's token:** the runner is
    offline, so a workflow runs `git` itself, not `actions/checkout`;
    temper 03.3 records the fact that a job's token clones its private
    repository.
26. **The protocol world's late, reordered and lost deliveries wait**
    (forge.md 9, 10; `later.md`): the fake sends each delivery on time;
    the connector world's hinted stories (temper 06) and the process
    tier's hinted story (temper 11.2) carry hints with polling as the
    fallback.
27. **The base's lint idioms bind the sketches** (`findings/base-runs.md`):
    `#[expect(clippy::large_enum_variant, reason = …)]` on an enum that
    carries a `Document`, never a `Box`; no `Default` derive in step code
    (`User`'s literals name its two new fields).

## Rulings

101. **04.1's Size at 300** (plan defect, temper 04's block of 2026-10-16T22:10:01Z).
     Ruling 101, a plan defect: 04.1's Size of 250 under-counted the scenario the session file itself prescribes
     (the listener, eleven steps, two probes, 21 facts and their roll-up, "Facts, 21, each named `delivery-` …" in 04.1's
     Shape). 04.1's Size is now 300, the plan's largest (development.md, 5.2: "Every Size is at most about 300 lines"),
     so its tolerance is 600 by the implementation rules ("An increment lands within twice its Size without asking").

     - 04.1 lands at its count of 523, plus what the recording's proof needs fixed, while the count by
       code-organization.md 6's command stays at most 600.
     - Past 600, stop on a question again, with the count by file.
     - Unchanged: 04.2's Size (220, codec); the rule against trimming a doc comment, a test or the formatting to meet a
       Size; the proof, the recording whole, before landing.

102. **07.1's Size at 300; sizes within 2.6 times** (plan defect, temper 07's block of 2026-10-16T22:11:00Z).
     Ruling 102, a plan defect: 07.1's Size of 280 under-counted its own Shape (sketches 1-3, `problem::{Problems,
     lines}`, `limits::{apply, check}`, `check::members`' refusals of setup.md 8, `Step` and `steps`). Accept, do not split:
     07.1 is written, its 17 tests pass, and 07.2-07.4, temper 08, 09 and 12 wait on it.

     - 07.1's Size is now 300, the plan's largest (development.md, 5.2: "Every Size is at most about 300 lines").
     - The same block twice (04.1, ruling 101; 07.1) becomes a standing rule, in the README's "Rules of this plan":
       "An increment of this plan lands within 2.6 times its Size without asking, a Size marked "(codec)" within 3 times;
       its report gives the count by code-organization.md 6's command, by file. Past that, it stops on a question, as
       the implementation rules says." By it, 07.1 lands at its 724 and what its review needs, up to 780.
     - Unchanged: never trim a doc comment, a test or the formatting to meet a Size; a split
       increment's parts count together; 07.2-07.4's Sizes.
     - For the retrospective: with Sizes at most about 300 and increments landing at about twice them (development.md,
       5.2), a tolerance of twice stops about half of them; this plan's two size blocks came in its first 11 minutes.

103. **04.1's deletion by forge.md 5; cancelled, interim** (design gap and plan question, temper 04's block of 2026-10-16T22:13:42Z).
     Ruling 103, interim on question 1, final on question 3 of temper/04-deliveries.md, "Questions to raise when
     reached".

     - **Question 3, the branch deletion: a plan question, answered by the design.** forge.md 5's table, row `push`:
       "of a branch, `Change::BranchMoved { branch, head, actor }`, … ; of a branch deleted, `Change::Branch(branch)`".
       So, as question 3 says, 04.2's document gains `Delivery::deleted()` (a push whose `after` is all zero), and the
       deleted branch's hint comes from that push. 05.2's Shape already reads it so (05-webhooks.md:168-169: "`Push`,
       `BranchMoved { … }`, or `Branch(branch)` when `after` is all zero"). `branch-deletion-sends-no-push` failing is
       allowed: 04's "Done when" counts "2, or 3 with branch-deletion-sends-no-push".
     - **Question 1, the cancelled job's delivery: a design gap, sent to the designer** (`question.md`, 22:14:42Z). Until
       the answer, go on as question 1 says: "A delivery not had within the Size is left unrecorded (no fact;
       `SUBSCRIBED` unchanged; the coverage test names it)". No web driver. The answer may change 04.2's coverage test
       and `SUBSCRIBED`; it reaches you as a later ruling.
     - **Question 2, the pull-ordering pair:** go on with your investigation. The recording rule holds (README, "The
       recording tools": once more after each fix aimed at a cause the output names; the third failure with the same
       symptom is a question). If both facts of the pair still fail then, block with session 04's question 2.
     - Unchanged: 04.1 lands only with `holds webhook-deliveries` and 04's "Done when" counts.

104. **04.1's cancelled job end, by the API's cancel** (designer's answer, temper 04's block of 2026-10-16T22:13:42Z).
     Ruling 104: the designer's answer, landed in temper fbe1ceaed, replaces ruling 103's interim on question 1:

     temper 04.1, the cancelled job's delivery: **record it through the API, with no web driver.** Landed in temper
     fbe1ceaed (forge.md 5 and 13; plans/forge-setup/temper/04-deliveries.md).

     The premise that no API route cancels a run does not hold. Forgejo v17's API description
     (`tmp/forgejo-17-swagger.json:6556`, `CancelActionRun`) says: `POST /repos/{owner}/{repo}/actions/runs/{run_id}/cancel`,
     "Pending or running jobs of the run are cancelled. A run that has already finished ... is left unchanged. In both cases
     the endpoint responds with HTTP 204." forge.md 13 now lists it: "A run is cancelled, its waiting and running jobs with
     it, by `POST actions/runs/{run_id}/cancel`: `204`, ...; temper never cancels one."

     - **04.1, step 10:** `deliveries.yaml` gains job `held`, `runs-on: temper-held`, which no runner serves; once
       `workflow_job_success`, `_failure` and `_skipped` are had, O cancels the run (`POST R/actions/runs/{job.run_id}/cancel`,
       `204`): `workflow_job_cancelled` (`held`).
     - **Facts, 22:** `job-ends-row` leaves the cancelled end aside; a new fact, `cancelled-job-end-is-delivered` (may fail),
       checks it as `job-ends-row` checks the others. "Done when" counts 22, and lets that fact fail.
     - **04.2:** `SUBSCRIBED` unchanged. `every_subscribed_event_has_a_recorded_delivery_the_decoder_knows` names
       `workflow_job_cancelled` as the one unrecorded event only while `delivery-cancelled-job-end-is-delivered` fails.
     - **If the cancel delivers nothing,** forge.md 5 now rules: "An event the tool records no delivery of stays as the
       sources name it, its fields those of the events whose payload it shares, and the tool names it unrecorded: a hint only
       brings a read sooner (`domain/forge.md`, 5.2), so a row that is wrong costs a poll's wait."
     - forge.md 13's to-check list gains "a job's cancelled end, from cancelling the run of a job no runner takes".
     - 05 (the component's hint for a job's end) is unchanged. Nothing is deferred: no `later.md` row.

105. **06's Shapes against main** (ready check of temper-06.1).
     Ruling 105, the ready check of temper-06.1 (against skein a1252d40, temper eab91b13), corrections to
     temper/06-settle.md: `translate::hint` serves the forge world and the conformance world, the client world keeps
     ignoring hooks (`client/src/world.rs:125,467,645`); Files add fake `tests/next.rs`, whose `:32,33` read the
     harness's hooks by tuple position; `Config` has six literals, named, not five; the updated tests add
     `observations_and_webhooks_follow_branches_pull_requests_and_the_wiki` (`tests.rs:1720`) and the totals at
     `tests.rs:1431-1433`; `wiki.rs:45,71` call `Hook::wiki` now and push the observation alone once `Wiki` goes;
     the helpers moved to `changes.rs` add `change_step_with_release` (`simulation.rs:324`), `change_step_task` is
     `pub`, and `protocol.rs:63-66` keeps its copy. Sketches A to E compiled under clippy; no suite was reached. A further
     user of a reshaped `Hook` or `Change` found while working is updated, and named in the commit body.

106. **12's Shapes against main** (ready check of temper-12.1).
     Ruling 106, the ready check of temper-12.1 (against skein a1252d40, temper eab91b13), corrections to
     temper/12-box-machine.md: jig-boundary is at the rev `Cargo.lock` locks (755e02e then, not 2900e9b); 12.2's
     `io_limits` gives `skein_io::Limits`'s five other fields (`sends` 1, `accepts` 1, `backlog` 1, `close_timeout` 5 s,
     `retry` 10 ms, as the terminal's `limits.rs:90-94`); the harness's root is `Sim::root(Handle::new(opened.raw()))`,
     `Sim::now` is `sim.rs:191`, and each ask goes to `skein_fake_machine::step` alone, never `serve`, the spawn's
     environment recorded from its `Ask::Spawn` first (`step` drops it, `face.rs:19`). The host in the `no_std` crate
     under the subset's lints is the design's: setup.md 596-598, "its host (ordinary Rust) on skein-io's processes; …
     one crate, under the subset's lints, its host written within the subset". Sketches 1-3 and the boundary test
     compiled under clippy; the boundary test passed, 1 of 1. A further stale pin or limit found while working is
     refreshed, and named in the commit body.

107. **08's Shapes against main** (ready check of temper-08.1).
     Ruling 107, the ready check of temper-08.1 (against skein a1252d40, temper eab91b13), corrections to
     temper/08-run.md: each shell test file declares `pub mod support;` (as `crates/temper/tests/configuration.rs:1`;
     private, clippy finds unused helpers dead), in 08's sketch and 10's (10-reconcile.md, sketch E); `Programs::marker`
     fills `descriptors` from `/proc/<pid>/fd` once `/proc/<pid>/comm` reads `sleep` (a script's own `/proc/$$/fd`
     also lists its script and a pipe); 08.3's bound on the tier whole is 42.4 s plus the ceilings of the tests added to
     the `process` binary since the base (08.1's 30 s, 08.2's 60 s, each other named in the report), not 90 s; the
     signals are adopted by an `Io` (`Io::adopt_signals`, `layer.rs:198-206`), not a `Layer`; `support/live.rs:140`
     spawns git, and the tier starts `temper` through skein-world's `Binary`; `crates/temper/Cargo.toml:69-84`. Sketch 5
     compiled under clippy; the default profile of the throwaway, the conformance tool, git-transcripts and temper passed,
     37 of 37. A further private `mod support;` in a test crate whose helpers some files leave unused is made `pub`, and
     named in the commit body.

108. **09's Shapes against main** (ready check of temper-09.1).
     Ruling 108, the ready check of temper-09.1 (against skein a1252d40, temper eab91b13), corrections to
     temper/09-reconcile-machine.md: 09.1 needs 01.1 for `User`'s `is_admin` and `full_name` (01-users-tokens.md:93) and
     01.2 for `UserOf` and `CreateUser`; 09.3's unit tests are in `crates/temper-forge-setup/src/tests/reconcile.rs`,
     now in its Files; 09.2's Files add the fake's `src/tests.rs` (`mod setup;`), `src/tests/fixtures.rs` (`ask_as`),
     `src/tests/setup.rs`, and `tally.rs:122`'s `expect` message, stale once the store's tokens count; the world reads a
     document of its own, not `TWO` (one org per entry there); sketch B's `find`, `token_user` and `scopes` are
     `#[must_use]` (clippy's `must_use_candidate`). Sketch A and the 09.1 reference test compiled under clippy; no suite
     ran. A further sketch function that clippy asks to mark `#[must_use]` is marked, and named in the commit body.

109. **02's Shapes against main** (ready check of temper-02.1).
     Ruling 109, the ready check of temper-02.1 (against skein a1252d40, temper eab91b13), corrections to
     temper/02-orgs-hooks.md: 02.1 needs 01.2's `pub(crate)` `Recorder` and `send` (01-users-tokens.md:162), and makes
     `w` (`cases.rs:33`) and the fields `orgs.rs` reads `pub(crate)` where 01.2 left them private; the sketch's `access`
     and `access_text` are `#[must_use]`; 02.1's Files add the exhaustive lists the check's compile found:
     `server/writes.rs:316-394,398-439`, `client/writes.rs:38-124,171`, `client/start.rs:135`, `server/items.rs:252`,
     protocol-forge `actions.rs:232-242`, the fake's `routes.rs:235` (:321), and `tests/codecs/tests/forgejo_{files,
     items,reviews,start,writes}.rs` (:153, :146, :112, :70, :169); "Done when" 3's fuzzy run passes, 2 tests, not 14.
     The sketched types and arms compiled (`cargo check --workspace --all-targets`); the default profile of the four
     crates passed 217 of 218, the one failure the `orgs` reference test with no encoder written; the fuzzy run passed,
     2 of 2. A further exhaustive match the compiler finds is given its arm, and named in the commit body.

110. **04.1's proof retried once** (process, temper 04's block of 2026-10-16T22:32:06Z).
     Ruling 110, process: retry the whole prescribed recording proof once, with no code change. The README's
     "The recording tools" lets a tool run "once more after each fix aimed at a cause their output names"; this failure
     names no cause in the increment (the `--component` phase's existing pull fixture, which the first whole proof passed),
     so one retry without a fix is allowed here, once.

     - If the retry passes: amend the Proof trailer with it, and land.
     - If the same `provisioning /api/v1/repos/fixture/specimen/pulls: HTTP 500` comes back: do not retry again. Make the
       fixture's provisioning error keep the response body and keep the instance's log on failure (a fix aimed at the
       cause's diagnosis, within 04.1's Files: `src/fixture.rs`), run once more, and if it fails a third time, block with
       the body and the log's lines.
     - Unchanged: the recording rule's "third failure with the same symptom is a question".

111. **04.1's pull pair; the fake always checks** (designer's answer, temper 04's block of 2026-10-16T22:36:21Z).
     Ruling 111: the designer's answer to session 04's question 2, landed in temper edcd1954b:

     temper 04.1, a pull's mergeability at its push's delivery varies: **the design bears either order. The facts record
     whether Forgejo was ever seen checking, and the fake always checks.** Landed in temper edcd1954b (forge.md 9 and 13;
     plans/forge-setup/temper/04-deliveries.md, 06-settle.md). domain/forge.md is unchanged.

     - **`settle` (06.2) needs nothing more.** domain/forge.md 5.2: "within `settle` of a move of its head or base (…), a
       pull request that reads not mergeable is read again at `hinted`, doubling, until it reads mergeable or `settle` has
       passed", and "Until `settle` has passed since the move, not mergeable is not known".
     - **The pull pair (04.1's Facts):** `pull-is-settled-when-its-push-is-delivered` holds when every probe B (steps 7 and
       8) read the pull mergeable. `pull-is-checking-when-its-push-is-delivered` holds when one probe at least read it not
       mergeable, and each such pull then read mergeable within 30 s (the small profile's `settle`). It is read again once
       the delivery is answered, at 1 s, doubling. Each fact's line keeps probe B's reads. Exactly one holds on every run.
       If Forgejo takes longer than `settle`, both fail, and that is a new question. "Done when" is unchanged.
     - **forge.md 13 now asks** "whether a pull request's mergeability is ever still checked when the push that moved its
       head or base is delivered (the order varies between recordings, so one read checking is enough), and that it settles
       within `settle` of the delivery".
     - **The fake (06.1) always checks,** whichever pull fact holds. forge.md 9, "Forgejo's quirks, kept": "a push that
       moves a pull request's head or base starts its check, which reads it not mergeable until it ends, and is delivered
       while it runs, the order Forgejo sometimes takes (section 13)". `git::moved` starts each `on_branch` pull's check
       for `CHECK` (2 s). 06.1's ready check greps `delivery-job-end` alone (two lines); the job order still follows its
       pair.
     - Session 04's question 2 is marked answered. Nothing is deferred: no `later.md` row.

112. **05's Shapes against main** (ready check of temper-05.1).
     Ruling 112, the ready check of temper-05.1 (against skein a1252d40, temper eab91b13), corrections to
     temper/05-webhooks.md: 05.1's reference test names `domain::api::Change::Push` (`api.rs:720`), not `raw::`;
     `server::down` (`server.rs:132`, its arm :140) and `Service::target` (`service.rs:94`) keep ignoring a `Hook`;
     05.2's MAC in hexadecimal calls `marker::hex` (one nibble, `marker.rs:53`) on `byte / 16` and `byte % 16` of each
     byte, as `marker.rs:23` does; 05.2's Files add the conformance's `component/hosted.rs:255` (`kit::Event`) and `:87`
     (`Limits`), and take from 05.3 `forge.rs`'s `problems` (:197, :212) and `indexed` (:257) arms for the new
     `Unusable` and `Stacked`, without which temper-service stops compiling at 05.2; the new stop is `stop::hooks`, not
     `stop::listener` (`Stop::Listener`, `lib.rs:49`, is the clients'); 05.3's renamed test is
     `an_unknown_missing_or_reserved_member_is_named_by_its_path` (`startup.rs:33`); `NotBuilt` is at `problems.rs:171`,
     not :169. 05.1 and 05.2 corrected passed workspace clippy `-D warnings`; default suites 89 passed (05.1) and 298
     passed (05.2), fuzzy 9 passed each. 05.3 was not checked against code. A further exhaustive match the compiler
     finds is given its arm in the increment that adds the variant, and named in the commit body.

113. **12.2 at up to 850; 106's corrections on main** (plan defect, temper 12's block of 2026-10-16T23:00:52Z).
     Ruling 113, a plan defect: 12.2's Size of 220 under-counted the host its Shape gives (the passes, a child's
     ownership by generation, a line buffer per pipe, the queued sinks, the descriptors' teardown with the inherited root).
     Accept, do not split: the sinks and lines (171 lines) serve the host alone, and a split would land the same code in
     two pieces.

     - 12.2 lands at up to 850 production lines by code-organization.md 6's command, past the README's "Sizes" (2.6 times,
       572). Past 850, stop on a question again, with the count by file; a split of `lines.rs` and `sink.rs` into a
       supplement step is then the default.
     - The session's "Done when" count is now at most 1324 (12.1's 474 landed, and 12.2's 850), not 1000.
     - Ruling 106's corrections to 12-box-machine.md reached `main` only now (temper d8bb411bc; they were dropped by the
       plan's landing tool): read 12.2's Shape there before going on. They bear on the host: `io_limits` gives
       `skein_io::Limits`'s five other fields (`sends` 1, `accepts` 1, `backlog` 1, `close_timeout` 5 s, `retry` 10 ms);
       the harness's root is `Sim::root(Handle::new(opened.raw()))`; each ask goes to `skein_fake_machine::step` alone,
       never `serve`, the spawn's environment recorded from its `Ask::Spawn` first.
     - Unchanged: never trim a doc comment, a test or the formatting to meet a Size; 12.2's tests
       and their ceilings.

114. **The component fixture's HTTP 500** (process, temper 01's block of 2026-10-16T23:03:16Z).
     Ruling 114, process. The same block twice (04.1's of 22:32:06Z, ruling 110; 01.2's) becomes a standing rule, in
     the README's "Rules of this plan":

     "The conformance tool's `Fixture::new` met `provisioning /api/v1/repos/fixture/specimen/pulls: HTTP 500` in 04.1's
     and 01.2's recordings, in the `--component` phase, after passing before. temper 01.2 makes `Fixture::api`
     (`tools/forgejo-conformance/src/fixture.rs:84`) keep the answer's body, at most 200 bytes, in its error, and
     `Fixture::pull` (:98) try an HTTP 500 again, at most 3 times, 1 s apart, each try's status and body on stderr. Until
     that lands, a recording that fails so is run once more, unchanged, without asking; a second such failure is a
     question, with the body."

     - temper 01.2: make that fix (a fix aimed at the cause, README "The recording tools"), in its own commit within
       01.2's landing, then record once, whole, as 01.2's Proof says. Its lines count in 01.2's Size.
     - Unchanged: the recorded cases and facts (the retry is the fixture's provisioning, never a recorded exchange); the
       recording rule's "third failure with the same symptom is a question".

115. **06.1 and 06.2 land together** (plan defect, temper 06's block of 2026-10-16T23:33:36Z).
     Ruling 115, a plan defect in the order of 06's increments: 06.1's Shape makes the fake always check ("`git::moved`
     … starts the check of each `on_branch` pull … for `CHECK` (2 s)", ruling 111), and until 06.2's `settle` the
     connector takes a pull not mergeable at once as conflicted, which the conformance world's
     `a_ci_repair_recovers_its_focused_commit_cuts` meets. So 06.1 and 06.2 land together.

     - One merge, two commits: 06.1's, then 06.2's, each with its own `Step:` trailer; the gate runs once, at the tip.
       06.1's commit need not pass the gate alone.
     - Each keeps its own Size and its own proof; the report gives both counts and both proofs.
     - Do not weaken the story or its fixture, and do not defer the phase.
     - Unchanged: 06.2's Shape and tests; the README's "Sizes".

116. **02.1 at up to 1100, 02.2 at 600** (plan defect, temper 02's block of 2026-10-16T23:47:27Z).
     Ruling 116, a plan defect: 02.1's Size of 250 (codec) under-counted seven request variants with both codec sides,
     two tables and a 21-capture recorder, and the exhaustive arms they reach, which ruling 109 found to be more than the
     session file named (`server/writes.rs`, `client/writes.rs`, `client/start.rs`, `server/items.rs`, the fake's lists).

     - 02.1 lands at up to 1100 production lines by code-organization.md 6's command, past the README's "Sizes" (3 times a
       codec Size, 750). Past 1100, stop on a question again, with the count by file.
     - 02.2 (four hook variants, the same lists) lands at up to 600, not 450: its arms meet the same lists as 02.1's.
     - Unchanged: never trim a doc comment, a test or the formatting to meet a Size; 02's proofs.

117. **A literal a third time goes to the fixtures** (steer of the check of 2026-10-16T23:49Z).
     Ruling 117, process, from the check of 2026-10-16T23:49Z: the same finding twice (the `User` literal in
     temper-forge-forgejo's tests, the first check's steer; `Delivery::Ref { .. }`, `repository: 7` and the pushed head in
     04.2's `src/tests/delivery.rs`) becomes a line of code-organization.md 5's "Don't" list: "writes a document or request
     literal in a third test of a crate without a constructor for it in that crate's `tests/fixtures.rs`". delivery.rs is
     converted when an increment next touches it. Unchanged: the plan's tables of literals written from the recordings
     (04.2's `delivery_cases`); the fake's and the codec world's own fixtures.

118. **09.1's existing bot: never edited, its email stays** (designer's answer, temper 09's block of 2026-10-16T23:55:50Z).
     The designer's answer, landed in temper 231582c05 ("Reconcile never edits an existing bot: its email stays"):

     temper 09.1, an existing bot's email: **reconcile does not repair it. A bot that passes step 2's checks is never edited.**
     Landed in temper 231582c05 (setup.md 1, 5 and 8; plans/forge-setup/temper/09-reconcile-machine.md). The supervisor's
     reading and the session's interim rule hold, and become the design's.

     - **setup.md 5, step 2 now adds:** "A bot that passes is never edited: its email is the setup's `bot.email` when
       reconcile makes it, and stays as it is after. A change to `bot.email` reaches only a bot made after it; an existing
       bot's email is the person's to change, on Forgejo. temper reads no mail of the bot's, and its commits name the
       identity's `author` (configuration.md, section 3), not this email."
     - **setup.md 1:** "the bot user, made when missing and otherwise checked, never edited; its two tokens, ... each made
       when missing, repaired when changed and left alone otherwise."
     - **setup.md 8's table:** `bot`'s bounds read "a Forgejo login; an email, the bot's when reconcile makes it (section 5,
       step 2)".
     - **The code already agrees:** `temper_forge_forgejo::document::User` holds `id`, `login`, `is_admin` and `full_name`, no
       email (document.rs:35-40), and 09.1 landed with "Existing bot email is left untouched" (3f835db41). Nothing changes in
       09.1 to 09.4, temper 01 (no `PATCH /admin/users/{login}`) or the fake's `Account`.
     - Session 09's question is marked answered. Nothing is deferred: no `later.md` row.

119. **02.1's nameless team edit keeps the name** (plan defect, temper 02's block of 2026-10-17T00:10:55Z).
     Ruling 119, a plan defect: 02.1's Choices said "an edit always carries `name`, which v17 requires", and its facts
     named `setup-team-edit-without-a-name-is-422`. Neither the design nor the findings say so (`git grep` below); the
     pinned Forgejo answered the nameless edit `PATCH /api/v1/teams/4` `{"permission":"write"}` with 200 and kept the name
     `writer`. The fact records what Forgejo does.

     - The fact is `setup-team-edit-without-a-name-keeps-the-name` (capture `setup-team-edit-nameless`, unchanged): it holds
       when the answer is 200 and the answer's `name` is the team's name before the edit (`writer`). It replaces
       `setup-team-edit-without-a-name-is-422`.
     - 02.1's proof stands as written: the record command shows 18 new `holds`, no `fails`, and 21 new directories.
     - Unchanged: `TeamSpec` is one value, and every make and edit sends its `name`; the encoder, the cases and the other
       17 facts.

120. **15's Shapes against main** (ready check of temper-15.1).
     Ruling 120, the ready check of temper-15.1 (skein f981746a, temper 92bdbd429; 08.1 and 08.2 not landed, their
     names checked against `08-run.md`). Amends `temper/15-upgrade.md`:

     - Files gains `src/tests.rs` (`mod upgrade;`) and a `File::DumpPartial` row in `src/tests/files.rs:43-56`, and the docs
       of `Request::Lock` (`prepare.rs:58`), `Failure::Recorded` (`:89-91`) and `File::RunnerLock` (`place.rs:34`).
     - 07.4's `a_recorded_version_other_than_the_pin_is_refused` has one row, Forgejo `another-version`, and no runner row to
       move: renamed, it gets two rows, a runner in the setup and not in `state` and the reverse, both `Failure::Recorded`.
     - Shape: the two `Course` values are fields of `Prepare`, which has three `bool`s (a fourth trips
       `struct_excessive_bools`); the runner's lock's `Done` goes on to `Layout` in an upgrade, not `begin()`; a Forgejo-only
       upgrade takes Forgejo's lock alone and goes on beside a running runner (the old text said nothing is dumped beside one).
     - The reference test's sketch: `let … else` in place of `assert!(matches!(…))` (`clippy.toml` disallows `matches!`), and
       its nested `use` wrapped as rustfmt prints it.
     - Questions: two go to the designer (run's words for a runner-presence mismatch; setup.md 6.3's preamble against its
       item 3); until answered, 15.1 leaves 08.2's words for a presence mismatch as they land.
     - Unchanged: Size 180, the tests' names and Done when's count of 11; sketch 1's `todo!()` bodies are the
       author's to write.

121. **01.3's recording made byte-stable; 720 lines** (plan question, temper 01's block of 2026-10-17T00:17:56Z).
     Ruling 121, a plan question the session file asked for (01-users-tokens.md:385: "**Byte-stability:** a recording
     at the base rewrote 34 transcripts. If two recordings of the group differ, raise it."). Two recordings differ in five
     new responses: four token answers whose masked `created_at` aliases an earlier timestamp by second, and
     `setup-token-git-discovery`, whose pkt-line object ids (a 4-digit length before 40 hexadecimal digits, so runs of 44 or
     48) pass `Mask::bytes`, which recognizes runs of 40 or 64 (`mask.rs:90`).

     - The recording is fixed, as the session proposes: an object-id mask that knows git's pkt-lines (the id after the
       4-digit length, masked by the existing first-appearance rule, the length kept), and each captured token made in a
       second of its own, so its `created_at` aliases no earlier timestamp. The instability is not accepted.
     - What counts: after the fix, two recordings of the group in a row (the first one's transcripts committed) leave
       `git status --short tools/forgejo-conformance/transcripts tools/forgejo-conformance/facts.txt` empty. Run that
       check once, and quote its output in the report; nothing else is wanted.
     - Size: 01.3 lands within 720 production lines by code-organization.md 6's command (the README's "Sizes" gives a codec
       Size of 190 three times, 570), since the fix lives in `tools/forgejo-conformance/src/mask.rs` and `tokens.rs`, which
       that command counts. Past 720, stop on a question again, with the count by file.
     - A transcript or fact that the new mask changes beyond these five is refreshed in the same landing, with each test or
       fixture that reads a changed id; list each in the commit body. None is expected: no transcript on main holds a
       pkt-line object id (grep below).
     - Unchanged: the 25 setup and 40 component facts; the masks' first-appearance rule and their 40- and 64-digit runs;
       never trim a doc comment, a test or the formatting to meet a Size.

122. **10's Shapes against main** (ready check of temper-10.1).
     Ruling 122, the ready check of temper-10.1 (skein f981746a, temper 92bdbd429; 08.1, 08.2, 08.3, 09.2, 09.3 and
     05.3 not landed, their names checked against their session files). Amends `temper/10-reconcile.md`:

     - Line references moved on main: `sign_in.rs:94` (Small, its `.driver`), `:102`, `:139`, `:164-217`; `lib.rs` `USAGE` :23,
       `address()` :197; `instance.rs:195`; throwaway `lib.rs:84, :98` (`Outgoing`, `get`); `client.rs:36, :98, :160`
       (`temper_forge_forgejo::client`, not skein-http); the decode loop is `drive`, `tests/fixtures.rs:67`; `mod.rs:83, :463`.
     - 10.2's rule (`support/forgejo.rs:132`) flips `"ci":false` to `true` and keeps the rest: `up_to_date` is required
       (`landing.rs:74`), which the old text dropped.
     - `Settings.hook_secret_bytes` needs 05.3's `forge_component.hook_secret_bytes`, not landed and not in Needs: as Choice 8
       does for `listener`, `forge.rs` gives it `token_bytes` until 05.3 lands, and whichever of 05.3 and 10.1 lands second sets
       it from `hook_secret_bytes`. 10.1 does not wait for 05.3.
     - Unchanged: the `secrets/forge-1.*` paths of `support/configuration.rs:39` and temper-service's fixtures are other
       tests' own documents, not the tier's; 10.2's Shape 3 changes only `forgejo::configuration`. Sizes, tests and proofs.

123. **15.1's runner added or removed; the runner's lock** (designer's answer, ready check of temper-15.1).
     Ruling 123, the designer's answer to the ready check of temper-15.1's question of 2026-10-17T00:15:04Z, landed in temper 13e35d82e (setup.md 6.3, 6.4, 13.1; temper/15-upgrade.md, 07-prepare.md). It replaces ruling 120's rename of 07.4's test, and ruling 120's interim rule on 08.2's words. The answer's decision:


     temper 15.1, the runner's presence and the runner's lock: **a later preparation converges a runner added or removed,
     and `run` names `prepare` for it; the runner's lock is taken whenever the preparation writes the runner's files.**
     Landed in temper 13e35d82e (setup.md 6.3, 6.4, 13.1; plans/forge-setup/temper/15-upgrade.md, 07-prepare.md).

     - **1, setup.md 6.3 item 4 (new):** "A runner added or removed: the setup and `state` disagree on whether there is one,
       and `app.ini`'s `[actions]` follows the setup (6.2). A runner added is prepared under the runner's lock too, as a
       first preparation prepares it: its secret drawn into `runner/secret` when that file is missing, and kept otherwise;
       registered; its connection written. A runner removed is recorded as none, and its files and its registration are
       left, unused, so that a runner added again registers as the same one. A runner's recorded version is compared with
       its pin only while the setup has one. Either may come with an upgrade, in one preparation."
     - **1, setup.md 6.4:** `run` checks "that `state` records a preparation at the pinned versions, with a runner exactly
       when the setup has one; refuses otherwise, naming what to run: `temper forge prepare` for a forge not prepared, a
       preparation that did not end, an older version, or a runner added or removed (6.3); no command of this temper for a
       newer version, or one it cannot read."
     - **2, setup.md 6.3's preamble:** "the runner's too when it writes the runner's files: at a first preparation, and for
       a runner older than its pin (item 3) or added (item 4)". 6.4: "the runner's too when it writes the runner's files
       (6.3)". A runner removed writes none, so takes Forgejo's lock alone; a Forgejo-only upgrade too (ruling 120 stands).
     - **temper 15.1 builds it** (15-upgrade.md, amended): `Found` gains `runner_secret: bool`; the runner's field is an enum
       of the machine's own (a `Course`, added, removed or none); added: `Lock(RunnerLock)` first, then after `app.ini`
       `Draw` and `Write RunnerSecret` (when missing), `Register`, `RunnerConfiguration`, `State`; removed: no request of its
       own. `admit_state` checks presence both ways before the versions: `Refusal::Runner { recorded: bool }`, worded
       `prepared with a runner, and the setup has none` / `prepared without a runner, and the setup has one`, each then
       `: run temper forge prepare CONFIG.json N`. `Failure::Recorded` and its words go, and 07.4's
       `a_recorded_version_other_than_the_pin_is_refused` with it (ruling 120's rename is replaced).
     - **Tests:** `a_runner_added_to_a_prepared_forge_is_registered_under_its_lock` (rows: secret missing, present),
       `a_runner_removed_writes_no_runner_file_and_records_none`, `the_run_names_prepare_for_a_runner_added_or_removed`
       (binary), and two rows in 08.2's `the_run_refuses_a_forge_not_prepared_at_the_pins`. Done when 2 counts 14, from 11.
       Size 180 to 230. No process test of its own: 08.1's runs `Register` on the real binaries.
     - 08.2 is unchanged: it lands its words as planned, and 15.1 splits them. 07's question 1 is marked answered. Nothing is
       deferred: no `later.md` row.

124. **Byte-stable recordings, a standing rule; 02.1 at 1160** (plan defect, temper 02's block of 2026-10-17T00:24:09Z).
     Ruling 124, a plan defect, the same block as ruling 121's (01.3), so a standing rule now: the README's
     "Byte-stable recordings (rulings 121 and 124)". 02.1's recording after ruling 119 changed 11 setup responses
     (`setup-org-{issue-by-member,made-by-write-admin-alone,made,read,route-orgs-made}`,
     `setup-team-{edit,list-new-org,list-page-1,list,made,members}`): masked `created` timestamps alias by second, and
     Forgejo orders the deprecated `units` array at random.

     - Fixed, as the session proposes: each org and team creation whose answer carries a timestamp is made in a second of
       its own, and the mask sorts the `units` array's strings. Both fixes are in the README's rule; the second is in the
       shared `tools/forgejo-conformance/src/mask.rs`, outside 02.1's Files, which this ruling adds. No transcript on main
       holds a `units` array (grep below), so only 02.1's captures change.
     - 01.3 changes `mask.rs` too (ruling 121: pkt-line ids). 02.1 lands after 01.3, as it already waits for 01.3's
       `decode_refusal`: rebase on it, keep both fixes, and record again.
     - What counts: the README rule's check, once: after a second recording in a row, `git status --short
       tools/forgejo-conformance/transcripts tools/forgejo-conformance/facts.txt` prints nothing; quote it in the report.
     - Size: 02.1 lands within 1160 production lines by code-organization.md 6's command (ruling 116 gave 1100); past 1160,
       stop on a question again, with the count by file.
     - Unchanged: the 18 facts; the mask's first-appearance rule; ruling 116's 600 for 02.2.

125. **Size tolerance 3.5, codec 4.5** (steer of the check of 2026-10-17T00:28Z).
     Ruling 125, process, from the check of 2026-10-17T00:28Z: the Size tolerance is 3.5 times a Size and 4.5 times a
     "(codec)" Size (README "Sizes", which ruling 102 set at 2.6 and 3). Five Size stops (rulings 101, 102, 113, 116, 121)
     were all granted, at 2.2 to 4.4 times; each ended a session's turn. A ruling's own number (116: 02.1 at 1100, 02.2 at
     600; 121: 01.3 at 720; 124: 02.1 at 1160) stands where it is larger. Unchanged: the count by file in each report; the
     stop past the tolerance; never trim a doc comment, a test or the formatting to meet a Size.

126. **Ruling 125 applies to increments in progress** (process, temper 05's block of 2026-10-17T00:46:54Z).
     Ruling 126, process: ruling 125's tolerance (the README's "Sizes": "within 3.5 times its Size without asking, a
     Size marked \"(codec)\" within 4.5 times") applies to every increment not yet landed, those in progress included; the
     steer's "from their next increment" was the supervisor's slip. 05.2 (Size 280, not codec) lands at 764 production
     lines, within 980.

     - Unchanged: the count by file in each report; the stop past the tolerance; never trim a doc comment, a test or the
       formatting to meet a Size.

127. **02.1's team edit reads back four units** (plan defect, temper 02's block of 2026-10-17T00:52:53Z).
     Ruling 127, a plan defect: 02.1's Choices say "Units go in `units_map` …, each of the four at its access" and "The
     tool's literals are the encoder's output byte for byte", while the fact said "actions omitted so `none`". The encoder
     sends `"repo.actions":"none"`, and Forgejo answers it so. The fact follows the encoder:

     - `setup-team-edit-replaces-units-and-flags` holds when the `setup-team-edit` answer's `includes_all_repositories` is
       false and its `units_map` is the four units as the edit sent them: code `write`, issues `write`, pulls `read`, actions
       `none`. It checks the explicit `none`; an absent key no longer counts.
     - Unchanged: the four-unit `TeamSpec` and the codec's replay; the fact's name; the other 17 facts; ruling 124's
       check (two recordings in a row, `git status --short` empty) and Size of 1160.

128. **15.1 against 08's landings; JWT_SECRET's encoding** (re-check of temper-15.1).
     Ruling 128, the re-check of temper-15.1 after 08.1 and 08.2 landed (temper 7f3908ae8; a scratch worktree ran the
     corrections: clippy clean, 128/128 default, Done when 2's 14/14, and both process tests 2/2). Amends
     `temper/15-upgrade.md`:

     - **`JWT_SECRET`** (a defect of 07.4's draw, `prepare.rs:230`): 64 hexadecimal digits, which Forgejo decodes as base64
       in the URL alphabet to 48 bytes, not its 32; the first Forgejo command after each `app.ini` then logs
       `JWT_SECRET … invalid base64 decoded length: 48 … creating new key` on stdout and rewrites `app.ini`, and `ran`'s
       `first_line` takes that line as `Register`'s uuid (the runner test failed so) or, on a later preparation with
       `admin_token` missing, as the token. The draw becomes `skein_lib::base64::encode(Alphabet::Url, ..)` of its 32 bytes;
       a third preparation in 08.1's `forge_prepare_makes_an_owned_forge_then_remakes_nothing`, `admin_token` removed, checks
       the token and `app.ini` byte-equal. No `Migrate` before `Register` (the check's workaround is not taken).
     - 08.2 split `the_run_refuses_a_forge_not_prepared_at_the_pins` into one test per refusal (`tests/run.rs:7-48`): the
       runner rows go into those; a new test for `Refusal::Runner { recorded: true }`.
     - Files: `Command::Dump` in `tests/spec.rs:99-111`'s list; `runner_secret` in `fixtures.rs:152, :156-165`'s `Found`.
     - A `state` without a runner is made inline (`record_versions` keeps `None`), each row on its own `Scratch`;
       `Found.runner_secret` rides in the runner's enum, not a fourth `bool`.
     - Line references: `prepare.rs:90-91`, `place.rs:35-36`, `tests/support/mod.rs:6,44-71,518`.
     - Unchanged: Size 230, Done when's counts, the process tests' names.

129. **temper-15.s1: JWT_SECRET as Forgejo reads it** (plan defect, temper 15's block of 2026-10-17T01:16:08Z).
     Ruling 129, a plan defect: 15.1 landed (temper a2b4c07bc) as ruling 128 reached it. What ruling 128 asks that 15.1
     did not do is step **temper-15.s1**, `temper/15.s1-jwt-secret.md`, in session temper 15: `JWT_SECRET` drawn as base64
     in the URL alphabet of its 32 bytes, `Register` read by `first_line` again, and a third preparation in 08.1's
     `forge_prepare_makes_an_owned_forge_then_remakes_nothing` that removes `admin_token` and checks the token and `app.ini`.

     - Not asked any more: ruling 128's `Added { secret: bool }` and its inline runner-less `state`. 15.1 keeps the secret
       in `Prepare.runner_secret: Option<Secret>` (`prepare.rs:147`), not a fourth `bool`, and passed the gate so.
     - Unchanged: 15.1 as landed; Done when's counts; 15's Size of 230 is 15.1's.

130. **02.2 at up to 900** (plan defect, temper 02's block of 2026-10-17T01:27:20Z).
     Ruling 130, a plan defect: 02.2's Size of 150 (codec) under-counts the four hook variants with both codec sides,
     their exhaustive arms, twelve captures and facts and the recorder, as 02.1's did (ruling 116; 02.1 landed at 1153 of
     250). 02.2 lands at up to 900 production lines by code-organization.md 6's command, the proof's fixes included. Past
     900, stop on a question again, with the count by file. No split: 09.3 waits on 02.2, and the run's path is serial.

     - Unchanged: never trim a doc comment, a test or the formatting to meet a Size; the README's "Byte-stable
       recordings" check (two recordings in a row, `git status --short` empty); 02.2's proofs; the steer of 01:26Z.

131. **02.2's narrowed hook keeps its workflow_job events** (plan defect, temper 02's block of 2026-10-17T01:31:26Z).
     Ruling 131, a plan defect: the plan expected a hook PATCH to replace its events. The pinned Forgejo, patched from
     the full set to `{"events":["push"],"active":false}`, answers 200 with `active` false and `events`
     `["push","workflow_job_cancelled","workflow_job_failure","workflow_job_skipped","workflow_job_success"]`: it keeps the
     four `workflow_job_*` subscriptions. The fact records what Forgejo does:

     - `setup-hook-patch-narrows-events-but-keeps-the-workflow-job-ones` (capture `setup-hook-patched-narrow`, unchanged)
       holds when the answer is 200, `active` is false, and `events` is `push` and those four. It replaces
       `setup-hook-patch-changes-events-and-active`.
     - Production is unaffected: reconcile always patches the full set, which holds the four, so a repaired hook reads back
       as sent.
     - Unchanged: the fake forge, which replaces a hook's events whole (no request of temper's narrows a hook, so no test
       reaches the difference); the other eleven hook facts; the probe's request; ruling 130's 900.

132. **09.3 retries once past the descriptor test** (process, temper 09's block of 2026-10-17T01:50:52Z).
     Ruling 132, process, interim: `temper::forge::the_new_process_holds_no_descriptor_but_the_standard_three` (08.2's
     test, `crates/temper/tests/forge.rs:184`) saw `[0, 1, 2, 3]` in 09.3's gate; candidate 49d225783 touches no process or
     descriptor code, and the gates of the landings since 08.2 passed it. The supervisor has the failure diagnosed by a
     sub-agent; its fix, in the test or in production, becomes a step of its own, not 09.3's.

     - 09.3 runs the gate once more on candidate 49d225783, unchanged. If the same test fails again, report it with both gate
       logs and wait: do not change the test, the fixture or the process code, and do not retry a third time.
     - Unchanged: the implementation rules for every other failure; 09.3's Size (875) and proofs.

133. **temper-08.s1: the process tests wait for what they read** (diagnosis of 09.3's gate failure).
     Ruling 133, a plan defect in 08.1's and 08.2's process tests: step **temper-08.s1**,
     `temper/08.s1-tests-wait-for-what-they-read.md`, fixes three races in the tests' observation that fail 1 to 5% of gate
     runs (09.3's gate, ruling 132): `Programs::marker` reads `/proc/<pid>/fd` while `sleep`'s loader holds a transient fd
     3; `Programs::hung` reads the pid file before it is written; `forge.rs:112` reads a killed sleep's stat once. No
     production descriptor leaks (the lock is close-on-exec and dup2'd to 0). It needs nothing that has not landed; any
     free slot starts it now, in a fresh session. Until it lands, a gate that fails on one of these three tests alone is
     run once more, unchanged, without asking; a second failure is a question, with both logs.

134. **10.1 and 13.1 against main** (re-check of temper-10.1, ready check of temper-13.1).
     Ruling 134, the re-check of temper-10.1 after 08.x and 09.3 landed, and the ready check of temper-13.1, both at
     temper 4764383ae (scratch worktrees: clippy clean, default suites passed, fuzzy 0 tests in reach). Amends
     `temper/10-reconcile.md` (Choice 8; 10.1's "Corrections against main") and `temper/13-box.md` (Needs; 13.1's
     "Corrections against main"):

     - 10.1: `Settings.hook_secret_bytes` from `forge_component.hook_secret_bytes`, landed with 05.2 (replaces ruling 122's
       interim); the directory test also overrides `setup.backoff_max_ms` and writes a private `admin_token`; `USAGE`'s
       three test copies and `forge.rs:146`'s usage; the shell's `Refused::words()`, prefixed once; `owned.rs` already
       has `run` and `Running::start`: a free name, `server.log`, `Option<Child>`, the port from `start`; the service's
       `Profile`, `Clock` not `Instant`, bounded accepts; nine line references.
     - 13.1: needs 10.2 at start too; Files gain `crates/temper/src/forge.rs` (the check's result); the box's values are
       `temper-box/src/limits.rs:40-48`; four paths and twelve line references.
     - Unchanged: Sizes, tests, proofs and Done when.

135. **10.1's remainder: 05.3 wires the hook bound** (plan defect, temper 10's block of 2026-10-17T02:16:37Z).
     Ruling 135, a plan defect: 10.1 landed before ruling 134 reached it. No supplement step:

     - `Settings.hook_secret_bytes`: 05.3 sets it from `profile.forge_component.hook_secret_bytes` at
       `crates/temper/src/forge.rs:255`, beside `listener`, as Choice 8 already gives 05.3 the wiring that lands second.
       Until then reconcile reads `webhook_secret` within `token_bytes` (512), which holds its 64 digits.
     - `Refused::words()` is dropped: 10.1's command removes the shell's prefix before its own `line()`, so the prefix
       prints once, which was the point.
     - Unchanged: 10.1 as landed; 10.2 goes on.

136. **13's box filter counts 13** (plan defect, hourly check of 2026-10-17T03:36Z).
     Ruling 136, a plan defect in `temper/13-box.md`, Done when 2: `test(/box/)` reads `8 tests run: 8 passed`, and 13.1
     landed 13 tests the filter selects under the default profile, all of them 13.1's (cf4cc9e4c): `src/tests.rs`'s three
     `box_with…`/`box_without…` and `tests/startup.rs`'s ten `…box…` refusals. The process tier's `temper_box_*` and
     `temper_open_…inbox…` are outside the default profile (`.config/nextest.toml:8`, `not binary(/^(process|live)$/)`).

     - Done when 2 now reads `13 tests run: 13 passed`; README Done when's "at least" applies if a later session adds one.
     - Unchanged: the filter, 13's other items, and the landed tests.

137. **The author runs the renamed Proof** (process, the coordinator's block of 2026-10-17T03:50:37Z).
     Ruling 137, process: the steer's sentence was wrong; the coordinator checks reports with git only
     (development.md, section 2.3: "It checks reports with git only, ... it reads no code").

     - The author of 05.3 or 14.1 runs its increment's Proof with the new name, and its report gives the command run
       and its last line, with "old name -> new name" for each test renamed.
     - The coordinator checks each new name with git only, in the session's worktree: `git grep -c '<new name>' <commit>`
       prints a count of at least 1. A report that renames a test but gives no Proof command for the new name is asked for
       it once, as an incomplete report.
     - Unchanged: the rest of the steer's point 1 (names of about eight words; `owned_example` kept in 14.1's names; its Pass
       on text, word for word) and the coordinator's other report checks.

138. **05.3's Shapes against main** (ready check of temper-05.3).
     Ruling 138, the ready check of temper-05.3 at temper 62ca62ade and skein 867095d1, after client-api's temper 05.1
     (3f8b3df6f) and 05.2 (3c0936cd0) and design commit 405924109. Amends `temper/05-webhooks.md` (05.3's "Corrections against
     main", which take precedence over its Files, Shape and Test shape):

     - 05.3 adds the setup's view and the wiring, 07.x, 08.1 and 10.1 being on main; only `Named.webhooks` is new.
     - Files gain `support/owned.rs:20-22`, `src/tests/fixtures/forge.rs:134`, `temper-forge-setup/src/tests/fixtures.rs:28,38`
       and `fixtures::files()`'s capacity (`fixtures/startup.rs:41`); `"forges"` is in 11 files, not 8.
     - `application::parse` (99 lines) gives up its workers loop first; `if let` in place of `manual_map`.
     - An in-process `application.webhooks.listener` is refused by the address reader (`invalid form`), as the clients'
       listener's is: configuration.md's row, "a socket address on loopback ..., as the clients' listener's in standalone".
     - The reference test's `Post`, `PING`, `SECRET` and `Exit` are named; about 40 line references are moved.
     - The check's scratch worktree, with 05.3 sketched in it (about 219 lines of 250): clippy `-D warnings` and fmt pass;
       default 1745 passed, fuzzy 50 passed, Done when 6's filter 54 passed, the process test 204/403/404 passed. The 9 process
       tests that need `TEMPER_FORGEJO` were not run there.
     - Unchanged: Size (250), the tests, the Proof and Done when.

139. **11's and 14's Shapes against main** (ready checks of temper-11.1 and temper-14.1).
     Ruling 139, the ready checks of temper-11.1 and temper-14.1 at temper 64f610ae6 and skein 867095d1, before 05.3
     landed (5ee2677e9). Amends `temper/11-hinted.md` and `temper/14-example.md` (each's "Corrections against main", which
     take precedence over their Files, Shapes and Test shapes): sketches' duplicate imports, stale line references (about 45),
     users the Files miss (the box's tests and `live.rs` through `tier_configuration`; the `setup` member's copies), T2's
     `user.file` line, the `llm` address that text replacement misses, T4's four assertions.

     - **The process tier fails on main:** `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` passes at
       d4fe894a9 (51.8 s) and fails at 64f610ae6 (person step 4 sees no "Process answer" within 20 s; the fake LLM gets no
       query). No forge-setup code landed between them; the supervisor has it bisected and routes the fix. Interim: 11.1
       goes on with its own work. A proof of 11.1 that runs the landing story and fails as above is reported with its log, and
       waits: do not change the story, the person or the engine for it.
     - **14.1 waits for the designer:** with `small`, three authority overrides at 8 and the 10 grants `forgejo.rs:156-165`
       writes, the start refuses `engine.rules.ceiling.grants`, `projects[0].policy.ceiling.grants`,
       `projects[0].policy.projections` and `roles[0].authority.grants`, each past its bound; design commit 405924109 moved
       configuration.md 7's authorities. Which limits the example states is the question of this ruling's time.
     - **11.2's control** landed without deliveries in the check (paces at 600 s, at d4fe894a9): 11-hinted.md's Q2 fires
       before 11.2 starts; it goes to the designer with the wait named.
     - Unchanged: Sizes, tests, Proofs and Done when.

140. **Two standing rules for tests** (hourly check of 2026-10-18T11:19Z).
     Ruling 140, two standing rules in the README's "Rules of this plan", from the hourly check of 2026-10-18T11:19Z,
     each the third sighting of its finding (development.md, section 14.4): one behaviour per test with a literal
     expectation; named sizes, and both sides of a bound. They bind temper 11 and 14 from their next increment; landed tests
     that no increment touches stay. The steer of this time passes them on, with the process changes of skein b30cc95 and
     b1d2276 that this run takes.

141. **11.1 against 05.3** (re-check of temper-11.1).
     Ruling 141, the re-check of temper-11.1 after 05.3 landed (5ee2677e9), at temper 01e467afc. Amends `temper/11-hinted.md` (its "Corrections against main"): the `webhooks.rs` sketch's two duplicate imports; `support/mod.rs`'s references moved by one line, `limits.rs:252`, `forge.rs:116`; `owned::document` takes no webhooks input, so `forgejo::configuration` sets them after it, or `owned::Names` gains a field. With these, in a scratch worktree: clippy `-D warnings` passes; the default 54 of 54; the process tier 16 of 17, `a_push_is_delivered_signed_with_the_secret_reconcile_drew` in 4.9 s; the landing story fails as ruling 139 says, with 11.1's changes and without them.

142. **14.1's example limits, the designer's answer** (design gap, ready check of temper-14.1).
     Ruling 142, the designer's answer to the question of 2026-10-18T09:22Z (temper 14.1, the owned example's authority limits): temper a975025ad (configuration.md 7 and 10; 14-example.md's "Corrections after the designer's answer"). T3 (report item 5, the plan's call): `example_document` keeps the file's `charter` 2.


     1. **The example keeps the owner's 10 grants and raises small's bounds to hold them.** configuration.md 10 now says:
        "the rules' ceiling, the project's ceiling, its role's authority and its projections each hold one grant on the org,
        `acme`, for each kind a change and its goal's issue use … 10, as the process tier's owner holds them". So the grants
        are the ones `support/forgejo.rs:156-165` writes (kinds 1, 2, 3, 4, 7, 8, 10, 11, 12, 13). None is cut.
     2. **Its `engine.limits` is `small` with 13 overrides, and no others.** configuration.md 10's skeleton:
        ```
        "authority.segments": 8, "tasks.hold_segments": 8, "tasks.authority_segments": 8,
        "documents.rows": 2, "documents.lines": 2, "documents.text_bytes": 128,
        "views.watchers": 16, "views.watchers_per_party": 16,
        "authority.grants": 12, "tasks.authority_grants": 12, "notes.scopes": 15,
        "tasks.authority_bytes": 512, "tasks.batch": 1
        ```
        configuration.md 7 gives each override's reason. The two that matter here: `notes.scopes` 15 is needed "since jig's
        core admits an authority's grants only with 3 notes' scopes more". `tasks.authority_bytes` 512 and `tasks.batch` 1
        are needed because "the bounds above widen a task's stored row past the small profile's store, whose bounds no
        override changes". The old text, "the example admits at most the small profile's 8", is replaced by "the most the
        process tier and the owned example admit in a list, their `authority.grants` and `tasks.authority_grants` of 12".
     3. **Measured.** The probe ran the tier's engine document with these limits and the 10 grants on temper dbc5b4da7 (no
        code changed through d09271b57). `temper::prepare_in` returned `Ok`. With grants 10 and `notes.scopes` 13 it also
        returned `Ok`. Without `tasks.batch` 1 the start is refused with `engine.limits: past its bound` (jig `sizes::check`:
        `Value { needs: 4758343, has: 1303199 }`). With the design's old three overrides and 4 grants, the store also refuses
        it (needs 2,097,799). So the old example could not start whatever its grants were.
     4. **T4 `the_owned_example_fits_the_native_client`** (14-example.md, the new "Corrections after the designer's answer"
        block, which takes precedence over ruling 139's T4 line). It works on `owned::EXAMPLE`'s `engine`, parsed as
        `tests/configuration.rs:9-15` does, and checks three things:
        (1) the four assertions of `:16-33`: rows, lines, paths and text;
        (2) `assert_eq!` of the JSON's `engine.limits` with configuration.md 10's literal;
        (3) the authority's entries. If client-api's temper-04.s1 is on `main` when 14.1 starts, use its three assertions
        over the example. Otherwise assert that `authority.grants` and `tasks.authority_grants` are each at most 12, the
        literal from client.md 5.4. The commit body says which.
        Size: +10.
     5. **Report item 5 (charter) is the plan's call, not design.** The design text: the skeleton has
        `"charters": [{"number": 1, ...}, {"number": 2, ...}]` and the owned block has `"charter": 2`. The whole example
        therefore starts with charter 2. Only the tier's `owned::application` sets 1.

143. **14.1's startup reruns** (plan defect, temper-14.1's block of 2026-10-18T12:00:49Z).
     Ruling 143, a plan defect in temper 14.1's Shape 1. Yes: rerun the focused startup tests until T3's start passes.

     - Shape 1 said: "`memory_bytes` and each other number are the least the start's checks accept (one run reports every
       problem: at most three reruns)". Its premise is wrong on `main`: configuration.md 2 reads the file "in this order"
       (stages 1 to 5), and `crates/temper-service/src/startup.rs` ends each stage with `if !problems.is_empty() { return
       Err(problems); }`, so one run reports one stage's problems. "Every problem found is reported" holds within a stage.
     - The parenthesis now reads: "take each from the check that refuses it, and rerun the focused tests until T3's start
       passes (each stage of configuration.md 2's "Reading" reports its problems and stops, `startup.rs`'s `return
       Err(problems)`; ruling 143)". The three-rerun limit is gone; no count replaces it.
     - Rule: each number is the least its check accepts, derived from that check's source calculation, as you did for
       `checkout.name_bytes` 539 and `largest_output`; never a round number above it. If one field is refused again after you
       set its derived least, block with both refusals.
     - Unchanged: the Test shape's ceilings (T1 20 ms, T2 0.5 s, T3 1 s, T4 50 ms), the designer's `engine.limits` literal
       (ruling 142), `application.limits` as `small` with the fewest overrides T3 finds, each named in the landing report;
       Size (151 of 140 support lines is within the README's 3.5 times), Proof and Done when. The startup code's stages stay
       as they are: 14.1 changes no `startup.rs`.

144. **11.2's hinted story, the designer's answer** (design gap, ready check of temper-11.2).
     Ruling 144, the designer's answer to Q2 of temper 11.2: temper 3a323b3b2 (setup.md 10 and 13.4, domain/testing.md 2.6, `temper/11-hinted.md`, `later.md`).


     1. **The hinted story proves one wait: CI on a head, row 3.** setup.md 10, "Checked", now says: "the process tier's
        hinted story (13.4) checks the table's third row, CI on a head: the one wait of its landing that the forge ends. Its
        other waits end on temper's own writes, its push, its pull request and its merge, whose deliveries are echoes."
        Rows 1 and 2, when someone else moves them, are checked from hints in the worlds, like the other rows.
     2. **The control also raises `stall`, and the job outlasts `settle`.** "Within its budget, only the job's end delivery
        brings a read of its CI": the keep's `poll`, `poll_max` and `slow`; "so is the change's `stall`, whose end reads its
        pull request afresh"; "its job outlasts `settle` from its change's push, so the settle window's reads end before CI
        does". Without the slow job, a pull read not mergeable after it opens starts the settle window's reads (at `hinted`,
        doubling, until `settle`). One of them can see CI passed, and the control lands again.
     3. **11.2, amended in place** (11-hinted.md, 11.2's "Amended by the designer's answer to Q2"):
        - `raise_paces` sets four names to `RAISED_MS`. The fourth is `forge.change_policy.stall_ms` (names.rs:120; the
          profile's 30 s, temper-service `src/limits.rs`:154).
        - `repository` (support/forgejo.rs:353) writes `check.sh` as `test ! -e slow || sleep 60`, then
          `test -s README.md`.
        - In the hinted case, `landing` calls 11.1's `forgejo.push("slow")` after `Throwaway::start`, before `temper serve`.
          The landed story never pushes it, so its job is unchanged.
        - 60 s is twice `settle`. The window ends `settle` after the change enters `Checking`, a few seconds after the push
          that starts the job.
        - Unchanged: the control, its command and its expected `chat did not` line. If it lands, raise Q2 again.
        - Don't now also lists `settle_ms`, shortening the 60 s, and pushing `slow` in the landed story.
        - Done when counts 4 names, and greps the `check.sh` line once. Size: about 18.
     4. **Not a limit's sufficiency:** this answer measured nothing. The control, run by hand once, is the measurement
        (development.md 4.5).
     5. **Deferred, one later.md row, for the supervisor to place.** It was found while checking the answer against the code:
        - A change names its branch and base alone (temper-engine-domain `route/forge_route.rs`:1582). So the keep never
          reads its pull's CI on `poll` to `poll_max`, as domain/forge.md 5.1 and 5.2 say it should.
        - With no hint, CI's end is read only at the change's `stall`.
        - The shipped `stall` is 30 s, "from the engine forge world fixture" (`src/limits.rs`:154, :323). It holds as Stalled
          any change whose CI runs longer, hooks or not: a held change waits for a release.
        - The tier's 2 s CI never meets this. A box whose repository's CI takes minutes does.


     **The supervisor places item 5's row** (plan scope, development.md 10.4): deferred to temper's next plan, before the
     dogfood run (14.2's steps 4 and 8), and not a step of this run. 14.2's walk commits `check.sh` as `test -s README.md`
     (14-example.md, 14.2, Shape 1), and the tier's CI takes about 2 s, so no story or check of this plan meets a 30 s
     `stall`. `later.md`'s row now says so. Unchanged: 11.2's Size (about 18), its control and Done when as the amendment
     gives them; 11.1's block (ruling 139) and 14.1.

145. **14.1's application and worker limits, interim** (design gap, temper-14.1's block of 2026-10-18T12:10:04Z).
     Ruling 145, an interim ruling on temper 14.1's block of 12:10Z: which `application.limits` and `workers[0].limits`
     the owned example states is a design gap, sent to the designer in `question.md` at 12:1xZ. configuration.md 10's
     skeleton gives both as `{"profile": "small", "overrides": {}}`, and on `main` `small` does not fit smith's fixed agent
     bounds (the report's four refusals); whether the example lowers the forge's bounds, `small` changes, or smith's bounds
     widen is the design's (configuration.md 7, "the limits against each other and the worst case").

     Meanwhile 14.1 goes on with the work that does not rest on the answer:

     - the two `temper_box_*` tests in `process.rs`, the webhook setup, `owned::application` and `tier_configuration`, as
       Shape 3 and Files say; T1, T2 and T4 as they pass now;
     - to reach the later stages of the start, state provisionally in `owned.json` the tier's eight overrides,
       `crates/temper/tests/support/configuration.rs:44-45` (application: `checkout.name_bytes` 256,
       `checkout.repositories` 2, `checkout.message_bytes` 4096, `forge.name_bytes` 64, `forge.resources_per_task` 2,
       `git_component.stdout_bytes` 2048) and `:24-25` (the worker: `hub.accounts` 4, `hub.event_bytes` 598); run T3 and the
       box tests on them, and report whether the start passes, or the next refusal with its log.

     Rule: nothing of 14.1 lands before the designer's answer reaches you as an unblock; after the Proof and clippy, wait for
     it. The answer's numbers replace the provisional ones. Change no `startup.rs`, `worker_settings.rs` or smith code.
     Unchanged: ruling 143 (each number the least its check accepts; a field refused twice is a block), ruling 142's
     `engine.limits`, the Test shape's ceilings, Size, Proof and Done when.

146. **14.1's application and worker limits, the designer's answer** (design gap, temper-14.1's block of 2026-10-18T12:10:04Z).
     Ruling 146, the designer's answer to temper 14.1's question of 12:1xZ: temper 6545cb474 (configuration.md 7 and 10, STATUS.md, `temper/14-example.md`, `later.md`). It replaces ruling 145's interim; 14.1 may land once its Proof passes.


     1. **`small` gives.** The example and the skeleton keep `"limits": {"profile": "small", "overrides": {}}` for
        `application`. configuration.md 7 now says "`small` starts as it stands, with no override, against the agent temper
        runs". The constants change in `crates/temper-service/src/limits.rs`: `checkout.repositories` 3 to 2,
        `forge.resources_per_task` 4 to 2, `checkout.message_bytes` 256 to 4096, and `forge.name_bytes` 256 to 64, which
        "leaves 122 bytes of a `git` base within the checkout's 256". `checkout.name_bytes` 256 and
        `git_component.stdout_bytes` 1024 stay. Neither smith's bounds nor the example's application overrides change.
     2. **`workers[0].limits` is `{"profile": "small", "overrides": {"hub.accounts": 4, "hub.event_bytes": 598}}`**
        (configuration.md 10's skeleton). configuration.md 7 says these are "the two of the hub's that jig's small, sized for
        any agent, sets past smith's agent". On `hub.event_bytes` it says: "`hub.event_bytes` is the agent's `message_bytes`
        less 2, and no other value starts".
     3. **`owned/agent.json`'s `profile` is the process tier's**, `support/configuration.rs:5-7`, written out
        (configuration.md 10, "Its agent's configuration"). Its messages are 600 bytes, so 598.
     4. **Two temper-service tests rest on small's old forge values.** In `tests/checks.rs:43` and
        `tests/fixtures/startup.rs:29`, each override list gains `"forge.name_bytes":256,"forge.resources_per_task":4`.
     5. **Measured** (temper f6cb29f5c: the tier's document with the example's 13 engine overrides, `git`
        `http://127.0.0.1:3000`, the tier's agent). With the four constants changed, `{}` and the worker's two:
        `temper::prepare_in` returned `Ok`. One past each bound was refused: `hub.event_bytes` 597 and 599, `hub.accounts` 5,
        `checkout.message_bytes` 4095, `checkout.repositories` 3, `checkout.name_bytes` 257, `forge.resources_per_task` 3
        and `forge.name_bytes` 115 (114 starts). Then `cargo nextest run -p temper-service -p temper` ran 260 tests,
        260 passed (258 before item 4's two lines).
     6. **14.1's scope:** 14-example.md's new "Corrections after the designer's answer of 2026-10-18T12:25:44Z" block. It
        takes precedence over ruling 145's provisional eight and over the Test shape's "the fewest overrides T3 finds". The
        tier's own overrides stay. Ruling 145's "Change no `startup.rs`, `worker_settings.rs` or smith code" stays.
        `limits.rs` is in scope. Size: about +8 lines. README's Done when is unchanged.

147. **14.2's person password and web forms** (plan defect, temper-14.2's block of 2026-10-18T12:45:45Z).
     Ruling 147, two plan defects in temper 14.2.

     1. **`person_password` holds Forgejo's whole line, a defect of 07's prepare; it is step temper-14.s1**, in session
        temper 14, before 14.2 goes on: `temper/14.s1-person-password.md`. setup.md 3's table says the file is "the person's
        first password on the owned Forgejo"; `prepare.rs`'s `CreatePerson` arm writes `first_line` of Forgejo's
        `generated random password is '<password>'`. The file keeps the bytes between the line's first and last `'`, as
        `tools/forgejo-conformance/src/cli.rs`'s `password_of` reads them, and a line without them ends in
        `Failure::Unread`. As you proposed; the guide never extracts the password itself.
     2. **The web forms post the fields their page holds.** 14.2's Shape 3 said each is "posted with the `_csrf` its page
        holds"; the pinned Forgejo's `/user/login` holds `user_name`, `password` and `remember`, and no `_csrf` (your probe,
        `logs/20261018T124234-45.log`). It now says: "each posted with the fields its page holds, its `_csrf` when it holds
        one (the pinned `/user/login` holds none; ruling 147)". The password-change form posts its `_csrf` if its page
        holds one. No setting of Forgejo changes; the landing report names each page that held no `_csrf`, one line.

     Unchanged: 14.2's Files, checks (a) to (e), Size and Proof; 14.1 as landed. Refresh any further copy of the CLI's
     password line in a fixture that a suite shows failing, and list it in the commit body.

148. **Two standing rules: a setup written once, a shared support function's users run** (hourly check of 2026-10-18T12:37Z).
     Ruling 148, two standing rules in the README's "Rules of this plan", from the hourly check of 2026-10-18T12:37Z
     (development.md, section 14.4): a setup written once (the second sighting of 11:34Z's point 7: 14.1's
     `the_owned_example_fits_the_native_client` repeats its sibling's parse and four assertions; `owned_example` and
     `boxed_example` repeat the lay and the write of `config.json`); and a shared support function changed, its users run
     (14.1 rewrote `forgejo::configuration` and dropped the tier's `root.brief.gather_ms` 5000, running only the box's
     stories). They bind temper 11 and 14 from their next increment. The steer of this check passes them on. Not a gate
     check (skein b1d2276, point 6): temper has no `scripts/check.sh` yet (the workspace's AGENTS.md), and three steps are
     left; a proposal for the retrospective.

149. **14.s1 lands past the known red landing story** (plan defect, temper-14.s1's block of 2026-10-18T12:59:00Z).
     Ruling 149, a plan defect in ruling 148's rule: 14.s1 lands now; it does not wait for client-api's fix.

     - Ruling 139's wait ("A proof of 11.1 that runs the landing story and fails as above is reported with its log, and
       waits") is 11.1's: its Proof is the landing story. 14.s1's Proof is `cargo nextest run -p temper-forge-setup`,
       which passed (98). Ruling 148's rule said "a story red on `main` for a known cause stays red, with its cause named";
       it now reads: "runs the process tier once before landing, with `--no-fail-fast`; a story red on `main` for a known
       cause (ruling 139's landing story) stays red, its cause named in the commit body, and the increment lands without
       waiting for the fix (ruling 149)".
     - Your run stopped at the first failure: `logs/20261018T125712-45.log` ends `1/16 tests run: 0 passed, 1 failed`, so
       the other 15 stories did not run. Run the same command once more with `--no-fail-fast` added. What counts: 15 of
       16 pass, and `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` fails as ruling 139 names it (person
       step 4 sees no "Process answer"; the fake LLM gets no query). Then land, with one line in the commit body naming the
       red story and its cause (client-api's temper-04.s1 and 05.s1, not on `main`). Any other failure is a block, with its
       log. Nothing else is wanted.
     - Unchanged: 14.s1's Files, Shape, Size and Proof; ruling 147's point 2 for 14.2; ruling 139's wait for 11.1.

150. **14.2's script runs** (plan defect, temper-14.2's block of 2026-10-18T13:09:00Z).
     Ruling 150, a plan defect in 14.2's Proof: three runs were too few for a script that meets the pinned Forgejo's
     pages for the first time. Each of your three failed on a cause of its own (the version's build suffix; no `_csrf` on
     `/user/login`; `/user/change_password` redirecting to `/user/settings/change_password`), each found by a focused probe
     and handled in `follow.sh`.

     - The Bound now reads: "at most six runs, each after the last one's cause, a new one, was found by a focused probe and
       handled in the script (ruling 150); a failure with a cause seen before, or a fault in temper or the example
       rather than the script, is a question." So you have three more full runs, each under the 420 s bound.
     - A fault in `docs/examples/*` or the guide is fixed there, as 14.2's Files say; one in temper's code is a block.
     - Unchanged: checks (a) to (h), the 420 s wall, the script's own timeouts, ruling 147's forms rule (the pages that held
       no `_csrf` are named in the landing report, one line each), 14.2's Files, Size and Don't.

151. **14.2's CI branch and runs** (plan defect, temper-14.2's block of 2026-10-18T13:16:26Z).
     Ruling 151, two plan defects in 14.2.

     1. **Check (e) left a branch that check (f)'s adoption refuses.** (e) pushes "a new branch `temper/guide`"; adoption
        refuses a branch under the deployment's prefix that it does not own (`temper-engine-domain-forge`'s `adoption.rs`,
        the `Branches` stage: `branch.starts_with(&pending.request.prefix) && !owns_branch(..)` → `Error::Refused`). (e) now
        ends: "and the branch is then deleted (`DELETE /api/v1/repos/acme/widget/branches/temper/guide`, 204), since
        adoption refuses a branch under the deployment's prefix that it does not own (…; ruling 151)": as your script
        now does.
     2. **The Bound is ten runs.** It now reads "at most ten runs, each after the last one's cause, a new one, was found by
        a focused probe and handled in the script (rulings 150 and 151)"; the rest of it stands ("a failure with a cause
        seen before, or a fault in temper or the example rather than the script, is a question"). You have four more full
        runs, each under 420 s. Before each, `cargo build -p temper --bin temper` once, as the script now checks.

     Unchanged: checks (a) to (d) and (f) to (h), the guide's text (the person's walk pushes no `temper/` branch), 14.2's
     Files, Size and Don't; ruling 147's forms rule, and the landing report's lines for the pages with no `_csrf`.

152. **11.1 against main after client-api's fix** (re-check, temper-11.1's block of 2026-10-18T11:20:58Z).
     Ruling 152, the re-check of temper 11 against `main` after client-api's fix of the landing story (its temper-04.s1
     and temper-05.s1; fe4978f22 and acc7496e2), at temper fe4978f22, with the process tier run once.

     1. **11.1 goes on, restarted from `main`** in a fresh thread from `sessions/temper-11.md` (11:34Z's point 4). Ruling
        139's wait is over: `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` passes on `main` (22.4 s).
     2. **What 14.1 already did** (7f5ce036f): `support::owned::application` holds the example's `webhooks` listener and
        `forge-1/webhook_secret`, `Owned::tier()` picks the address, and `forgejo::configuration` keeps them. So:
        - `Forge::Throwaway` gains no `webhooks` field; `owned::tier_configuration` stays as it is; `forgejo::configuration`
          sets no `application.webhooks`. Ruling 141's "after `owned::document`" text is void.
        - `Throwaway::start` adds `directory: PathBuf` and `webhooks: Option<SocketAddr>`, read from the scratch's
          `config.json`, `application.webhooks.listener.socket` (`None` when `application.webhooks` is `null`, as 11.2's
          control writes it).
        - `Throwaway::start`'s users are `process.rs`'s `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart`,
          its adopt story, `a_native_chat_reaches_the_model` (new), and `live.rs`'s; ruling 139's "Users gain the box's
          tests … which call `tier_configuration`" is stale: they call `boxed_example`.
     3. **From the branch `forge-setup/temper-11-1` (2edde3617), keep** its test, `Delivery` and `receive` (with the `Read`
        import), and `forgejo.rs`'s fields and three methods; **drop** its `mod.rs`, `owned.rs` and `configuration` hunks.
        The re-check applied exactly that on `main`: clippy `-D warnings` clean, and
        `a_push_is_delivered_signed_with_the_secret_reconcile_drew` passed in 4.99 s. Start from `main`, and take those
        parts from the branch with `git checkout 2edde3617 -- <file>` and an edit, not a rebase.
     4. **Put back the tier's `root.brief.gather_ms` 5000** in `forgejo::configuration` (the steer of 12:49Z, point 1).
     5. **Size:** the corrected 11.1 adds about 92 support lines against its Size of 30 (within the README's 3.5 times);
        11.2 about 22 against "about 18".
     6. **Red on `main`, not 11.1's:** at fe4978f22, `temper_serve_answers_a_signed_delivery_204`,
        `temper_serve_answers_an_unknown_forge_404` ("the referee failed the real loop: webhook delivery or drain did not
        finish", at 30 s) and `temper_serve_prints_its_link_and_drains_on_a_signal` ("signal sent: true … exit: None").
        All three passed at 13:03Z on 14.s1's run (`logs/20261018T130310-45.log`), at 0.04 s each; between that base and
        fe4978f22 only client-api's code landed. They go to client-api's owner,. 11.1 runs the process
        tier once with `--no-fail-fast` before landing (ruling 148's rule, as ruling 149 amends it): these three may stay
        red, each named in the commit body with this cause; any other red story is a block.
     7. Ruling 148 said "temper has no `scripts/check.sh` yet"; it has had one since 796d50279, and every landing runs it.
        The gate check of b1d2276's point 6 is still not added: two steps are left.

     Unchanged: 11.1's Proof and Done when; 11.2 as ruling 144 gives it; the steers' points in force (12:49Z's 1-3,
     11:34Z's 9).

153. **11.1 waits for the drain regression's fix** (interim, temper-11.1's block of 2026-10-18T14:01:33Z).
     Ruling 153, an interim ruling: 11.1 waits for the fix of one regression on `main`, routed to client-api through the
     review (14:1xZ).

     - **The cause, bisected:** client-api's 3262df8f9 (its temper-04.s1, "Finish held HTTP calls once their clients
       component closes"). At it, `temper_serve_answers_a_signed_delivery_204` and `…_an_unknown_forge_404` fail on every
       run; `temper_serve_prints_its_link_and_drains_on_a_signal` passes 2 of 4. The same commit with `crates/temper-service`
       as it was at 63f6174dc passes all three. Your unsigned 403's timeout is the same symptom ("webhook delivery or drain
       did not finish"). Your landing story's failure at the restart (saved reply missing) comes with 11.1's tier webhooks
       on, and is read as the same cause until the fix shows otherwise.
     - **Wait:** keep 2866288c8 on `forge-setup/temper-11-1-restart` as it stands, and land nothing. Do not change
       temper-service, the person, the engine or the stories for it. When the fix is on `main`, the supervisor's unblock
       says so: then rebase 2866288c8 on `main`, run the process tier once with `--no-fail-fast`, and land if every story
       passes. A story still red then, with its log, is a block.
     - 11.2 waits behind 11.1, as before.

     Unchanged: ruling 152's items 1 to 5 and 7 (what you did is as it asks: `Throwaway::start` keeps the optional address,
     `gather_ms` is back); 11.1's Size (94 of the 3.5 times 30); the steers' points in force.

154. **11.1 after the drain fix** (unblock, temper-11.1's block of 2026-10-18T14:01:33Z).
     Ruling 154: the drain regression is fixed on `main`; 11.1 goes on as ruling 153 says for this case.

     - client-api's temper-05.s2, 46796e8df ("Keep the accounts root close runnable during drain", 14:42Z), fixes 3262df8f9's
       regression. Its body: "The six-story process proof passes, including the landing story's restart and four signal
       drains … The whole process tier passes all 17 tests in 64.565 seconds with both Forgejo exports and no fail-fast".
     - Rebase 2866288c8 on `main` (it merges cleanly onto eb8abb6aa), run the process tier once with `--no-fail-fast`,
       and land if every story passes. A story still red then is a block, with its log: none may stay red any more (ruling
       152's item 6 and ruling 153's list end here). Then 11.2, as ruling 144 gives it.

     Unchanged: ruling 152's items 1 to 5 and 7; 11.1's Size (94 of the 3.5 times 30); the steers' points in force.

155. **The Done when's six misses** (Done when run of 2026-10-18T16:29:31Z).
     Ruling 155: six of the seven FAILs of the Done when run of 2026-10-18T16:29:31Z (done.md) are misses, each
     recorded here with its reason and the command that shows the property on `main` now; no step follows from them.
     The seventh, the gate (temper 01.7, 03.7, 06.5, 10.1, 07.1, 14.2, 15.4), went to the designer at 16:33Z
     (`question.md`: the owned example's `memory_bytes` against a person's directory); its step follows the answer.

     1. **README 7, temper's default suite: the aim is missed.** README's Rules: "the run adds at most 1.0 s to the
        first". Median of three at temper 58c5321c1, `--no-fail-fast` (the default run stops at the gate's failure): 5.991 s
        for 1816 tests, against 3.928 s for 1306 at eab91b13d (+2.06 s). client-api's commits since the base are inside
        that difference and are not separable by item 7's command. The three slowest default tests:
        `an_adopted_repository_gets_its_change_pushed_through_git` 1.555 s,
        `held_and_decision_commit_cuts_restore_paged_real_children_and_exact_scripts` 0.861 s,
        `terminal_restart_replays_complete_state_with_delayed_store_and_page_terminals` 0.437 s. For the user; the README
        says the run is done either way. The fuzzy suite's aim is met (21.977 s against 24.745 s); skein's too (+0.30 s).
     2. **temper 01.6: a miss of the command, not of the property.** The command reads the plan's history: its three
        lines are temper-02.1's (faadff494, `let actual = forgejo::decode_answer(…)` and two `actual.…` assertions), and
        temper-02.2 (3e4c7f039) removed them 25 minutes later. No later commit can make it pass. Now: `git log origin/main
        --grep='^Plan: forge-setup$' -p --format= -- '*.rs' | grep -E '^[-+]' | grep -iE
        'actual|genuine|legacy|compat|workaround'` prints those three lines once with `+` and once with `-`.
     3. **temper 03.1: a miss of the text.** The third line is a test vector, `crates/temper-forge-setup/src/tests/upgrade.rs`'s
        `the_upgrade_names_its_dump_by_both_versions` (`upgrade::backup_name(b"16.0.5", b"17.0.0")`, temper-15.1), a literal
        expectation as ruling 140 asks; the pin is named only in `pin.rs`'s `VERSION` and `RUNNER_VERSION`. Now: the item's
        `grep` with `| grep -v '/tests/'` prints the two `pin.rs` lines.
     4. **temper 04, the transcripts' diff: a miss of the text.** It was written for 04's own landings; temper 01 and 02 add
        the `setup-*` captures their own Done when counts. None of the base's transcripts changed: with `B=eab91b13d`,
        `git diff $B origin/main --stat --diff-filter=MDR -- tools/forgejo-conformance/transcripts` prints nothing, and
        `--diff-filter=A` lists 120 files, each under a `setup-*` directory.
     5. **temper 08.4b: a miss of the text.** temper-15.1 renamed three of the run's step tests (for example
        `the_run_refuses_another_forgejo_binary`); its filter now finds one. Now: the README 7 run's default suite
        (`logs/donewhen-20261018/out/x7-temper.out`) ran temper-forge-setup's `tests::run` among 1815 passed, with
        `serve_prepares_once_on_the_owned_example` its only failure.
     6. **temper 10.3d: a miss of the text.** temper-14.1 (7f5ce036f) moved the tier's application to the owned example's,
        so the CI rule left `support/forgejo.rs`. Now: `git grep -c '"ci":true\|"ci": true' origin/main --
        crates/temper/tests/support docs/examples/owned.json` prints `docs/examples/owned.json:1` alone.

     Rule: a Done when item that a later step changes is shown by the command that reads the property on `main`, named as
     above; the item's own text stays as the plan wrote it. Unchanged: the README's Done when, the other items' PASS, and
     the release's order: "done when ran" waits for the gate's step.

156. **14.s2, the owned example's memory_bytes, the designer's answer** (design gap, the Done when run of 2026-10-18T16:29:31Z).
     Ruling 156, the designer's answer of 2026-10-18T16:48:11Z: temper 13154c9dd (configuration.md 7 and 10, STATUS.md, the plan's temper/14-example.md and later.md). It adds step `temper-14.s2` (temper/14.s2-memory-bounds.md), the last before "done when ran".


     1. **The example carries room for what a person's copy changes, up to each one's bound. The worst cases stay as they
        are.** configuration.md 7 now says: "each start passes with each string its service's worst case counts, and a
        person's copy changes, at its bound". The engine counts "the forge's `api_token` and its `git` base, a byte for each
        of theirs: the token at `small`'s `forge_component.token_bytes`, 512, … and the base at an owned forge's longest, …
        `http://127.255.255.254:65535`, 28 bytes". The worker counts "its agent's `program` and the configuration's path
        among its `arguments`, two bytes for each of theirs with the example's one slot: each at Linux's `PATH_MAX`, 4,096
        bytes". configuration.md 10 adds: "never its least for one directory".
     2. **The numbers, measured on temper 0601176ab: `engine.memory_bytes` 78,669,036,604,974 and `workers[0].memory_bytes`
        1,989,738,112,203.** main states 478 and 15,990 bytes fewer. With the strings at their bounds, these two start. One
        less each is refused "the worst case passes memory_bytes by 1 bytes", from scratch directories of 78 and 83 bytes.
        So the numbers no longer depend on the checkout.
     3. **The step (`temper-14.s2`; 14-example.md's new "Corrections after the designer's answer of 2026-10-18T16:47:23Z"
        block):**
        - T3 `serve_prepares_once_on_the_owned_example` lays the example at those bounds:
          - `Owned { forgejo: "127.255.255.254:65535" … }`;
          - `workers[0].agent.program` and `arguments[1]` each `"/"` followed by 4,095 `p`s;
          - `forge-1/api_token` of 512 bytes.
        - Then each number is raised by the N in T3's refusal, and no more.
        - Size: about +10 lines of test and the two numbers. No production code changes. T2, T4 and the box's tests are
          unchanged.
     4. **After the step, in this run and in client-api's:** a landing that raises a worst case raises that number by the
        N in T3's refusal. A landing that lowers one leaves the number. This replaces ruling 143's "least" for `memory_bytes`
        only; it stays for every other number. client-api holds its edits until the step lands (its ruling 377). Its
        rulings 354, 356 and 363 then read "exact" as "at T3's bounds".
     5. **Deferred** (`later.md`, STATUS.md): `small`'s worst cases are about 71.5 TiB for the engine and 1.8 TiB for the
        worker. So `memory_bytes` states no machine's memory, and the start's check bounds nothing a person has.

## For the user

- 1: the box lands on polling before webhooks reach it.
- 5: the limits' values.
- 10: the upgrade after the box.
- 14: the example's and the guide's places.
- 15: echoes cost a read each; no skip by key.
- `sha2` stays in temper-engine-domain-forge-issues (`later.md`).
- 17: the upgrade in a session of its own, last.
- 12: the process tier's 180 s, and a ruling if it goes past.
- 19: no fake-world story of deliveries to reconcile's hook (`later.md`).
- 26: the protocol world's late, reordered and lost deliveries wait
  (`later.md`).
- 20: the process tier's support reads `admin_token` (to attach and to
  make the person's token), which setup.md 9 reserves to reconcile in
  production.

## Answers received

- **The review,** 2026-10-16: temper's skein moves in temper 05.1
  once client-api's ruling lands (decision 2; ruling 154); temper builds
  on skein's `main` at the base (`findings/skein-main-in-temper.md`).
- **The designer,** 2026-10-16, landed in temper `326cbded`, `0d67b373`,
  `50c7dd0d`, `91ad7ef1`, `b05af1f7` and skein `121daa43`, `04c1562e`: 45 answers,
  each now in the design, which the session files cite. Among them: the
  lock and the exec by std, alone in one module (setup.md 6.4); the
  admin token's fresh name and scopes, a cut-short first preparation
  begun again from an empty layout, a later preparation taking Forgejo's
  lock alone (6.3); the bot's tokens on the admin's routes, its marker
  in its full name, private orgs (5); reconcile on plaintext loopback
  alone (5, 15); settle's window as its own wait, in the change's
  procedure (domain/forge.md 5.2); the box's signals, text and restarts
  (7.2; configuration.md 7); the listener's answers (forge.md 5); an
  absent option written `null`, never missing (configuration.md 3, 10);
  SHA-256's sweep in the fuzzy suite (lib.md 10).
