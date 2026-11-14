# panics: nothing catches a panic

Draft, 2026-11-13. What fabrig does with a panic, from the user's question that day: whether to forbid catching panics, since Cranelift can't unwind and they prefer error handling or an abort. They then asked that the tests that catch panics become panic-less too, without a growing set of tests that start a process to see an abort, and accepted the rule's words in skein's `programming-model.md` as section 3.2 gives them. It reads main at `6962c8fcc7`, and takes one review round's findings.

## 1. In one page

- **Nothing outside tests catches a panic today** (section 2). Release aborts, and a binary built to unwind aborts before anything unwinds. Tests catch: `catch_unwind` in 131 files, and 463 `#[should_panic]` attributes in 149. 123 tests have been ignored since 2026-11-12 because Cranelift, which the builders use for dev builds, can't unwind.
- **The rule** (section 3). A panic is our own code's bug, and it aborts. Whatever anything outside the process can cause is an error, the store's contents included. Nothing catches a panic, tests included, and no test triggers an assertion. A check whose job is to fail returns its finding. These are two sentences and a wider clause in `programming-model.md`, 10.4, a row's words in 10.1, and a limit's two sides in `testing-strategy.md`, 10, narrowed to limits that refuse.
- **No new test crosses a process boundary** (sections 4 and 6). Of the 123, about 40 test a check that fails, which returns its finding instead; about 25 test a limit's tightness, which compares what a build used with its price; about 45 trigger an assertion, and go, or become refusals where the cause is outside the process; about 10 test cleanup after a failure. Most become in-process tests. The few that must see a process end stay in the suites that already cross one.
- **What a failed test leaves outside its process is cleaned from outside it** (section 5), once the process has ended, however it ended. Four things have a stake: the live tier's held login, a person's account, and three kits' real processes, a throwaway Forgejo, a forge runner daemon and smith's live Claude process, each stopped today only by a `Drop`.
- **The count stays low and falls** (sections 6 and 7). No helper expects a panic or an abort. A gate check counts `catch_unwind`, `resume_unwind` and `#[should_panic]` per file, the count only falls, and once a pattern reaches none, the check refuses any. clippy's `disallowed-methods` bans the first two as well, in every `clippy.toml` the gate reads.
- **Nothing here depends on the backend** (section 8): once it lands, LLVM and Cranelift run the same tests, and production, built on stable with `--release`, aborts as it does now.

## 2. What main holds

- **Production code catches nothing.** No `catch_unwind` or `resume_unwind` is outside test code. The one in a `src/domain/` path, `jig/crates/jig-domain-people/src/domain/start.rs`, is in its `#[cfg(test)]` module. Four `set_hook` calls run on a panic, and none catches it: `skein-fail-stop`'s, which reports and aborts, and three in web code, where the abort is a trap: skein's playground worker and temper's playground page, which report the failure, and `skein-ui-web`'s restart, which reloads the page or says the application failed.
- **Release aborts.** The root `Cargo.toml`'s `release` and `web` profiles say `panic = "abort"`. The dev profile unwinds, as the test harness needs, and `skein_fail_stop::install()`, first in every `main`, makes a dev binary abort before anything unwinds (`skein/docs/foundation/programming-model.md`, 2.1).
- **Tests catch.** `catch_unwind` and `resume_unwind` are counted as lines naming them, `use` lines included; `#[should_panic]` as attributes:

| Tree | `catch_unwind` lines | `resume_unwind` lines | `#[should_panic]` | Ignored for Cranelift |
|---|---|---|---|---|
| skein | 53, in 30 files | 2 | 285, in 71 files | 49 |
| jig | 95, in 66 files | 1 | 91, in 38 files | 48 |
| temper | 57, in 32 files | 6 | 31, in 19 files | 24 |
| smith | 4, in 2 files | 1 | 30, in 8 files | 1 |
| hearth | 1, in 1 file | 0 | 25, in 12 files | 1 |
| `tools/` | 0 | 1 | 1, in 1 file | 0 |

- **Cranelift.** Its trial failed 151 of the focused suite's 34,696 tests: it doesn't catch panics, and it lacks an SSE2 intrinsic a dependency uses (`crate-graph.md`, 2). The 123 tests that catch a panic, or need cleanup to run while one unwinds, carry `#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]`. The builders compile dev builds with Cranelift since 2026-11-13 (`gate-compile.md`, 2), by their environment, not the repository's toolchain file. No `#[should_panic]` test is among the ignored, which suggests the test harness's own catch, compiled by LLVM with the standard library, still catches a test's panic there, its frames' cleanup skipped. That is a reading, not a run.
- **How the suites run.** The gate's focused and fuzzy suites run each test binary's selected tests in one process, one at a time (`tools/tiers/one-process`; `.config/gate/README.md`; `tools/tiers/design.md`, 11). The slow tiers run each test in a process of its own, under nextest.

## 3. The rule

### 3.1 What a panic is

- **A panic is our own code's bug:** an assertion that something can't happen, which happened. Past it, the process can't trust its memory, so it aborts, and a supervisor restarts it from what is durable, the path a power cut or a kill takes anyway (`skein/docs/foundation/programming-model.md`, 2.1). Catching one adds a second recovery that nothing tests, running on state the panic left half-changed. Since release aborts, that recovery would run in development alone.
- **Whatever anything outside the process can cause is an error:** a peer, configuration, or the store's contents. The last is how jig's restart already refuses a link it can't make, "never a panic" (`jig/docs/design/domain/engine/restart.md`, from the user's request `store-crash-safe` of 2026-11-09): a panic on what the store holds is a crash at every restart, where a refusal starts the deployment failed and says why. Some of jig's design still panics on a link, and follows (the status file).
- **An abort ends every session the process serves,** and an input that trips a bug trips it again after the restart. That is release's behaviour today, which this draft doesn't change. What a supervisor does with a process that keeps failing on one input is section 10's.

### 3.2 The words in skein's foundation

Accepted by the user, 2026-11-13. `skein/docs/foundation/programming-model.md`, 10.4's bullet on assertions becomes:

> - **Assertions are fail-stop** and are for the service's own bugs, never for anything from outside the process: a peer, configuration or the store's contents. `expect` names the invariant it relies on. Nothing catches a panic, tests included, and no test triggers an assertion: a test sees what an assertion guards through the outcomes around it. A check whose job is to fail, such as a referee or a judge, returns its finding, and only its caller panics.

10.1's row "Fail-stop" adds "nothing catches a panic" to what holds it, and to what checks it clippy's `disallowed_methods` and the gate's count (section 7). 2.1's "Panics abort" stays as it is.

The rule reaches one line of `skein/docs/foundation/testing-strategy.md`, which asks for a bound tested "from both sides" (1) and "one past it, which does not" (10). One past a limit an assertion guards is that assertion. 10's bullet becomes, and 1's "a bound tested from both sides" follows it:

> - **A bound's two sides.** A limit that refuses is tested at the bound, which holds, and one past it, which is refused, each in a test or a row of its own. A limit an assertion guards is tested at the bound, with what was used against its price: one past it is the assertion, which no test triggers (`programming-model.md`, 10.4).

### 3.3 What it asks of a change

- **An assertion needs no test of its own.** A change that adds one adds no behaviour, so nothing fails without it. What it guards is tested through what the code returns and does around it. A request for a test that fails without the change asks nothing of an assertion, so nobody writes a test that starts a process to see one fire.
- **A cause from outside the process is a refusal,** tested in-process like any outcome.
- **A check whose job is to fail,** a referee, a judge, a kit's comparison or a table, returns its finding as a value. The world, the loop or the test that called it panics on it, once, at its top. The check's own tests read the finding.
- **A crash a test build makes on purpose,** to test what restarts or reports around it, is no assertion: the fail-stop hook's fixture, and the browser stories' address at which a test build's module panics at start (`skein/docs/design/ui/web/testing.md`, 2). Each tests a process boundary, and stays (section 6).

## 4. Tests without catching

The 123 ignored tests, by what each catches. The split is a script's, by each test's name and what its catch wraps, and a review sampled 34 of them: it's rough, and each milestone reads its own tests again.

| Kind | About | Examples | Becomes |
|---|---|---|---|
| A check that fails | 40 | the referees in `jig/tests/views`; io's judge; skein's world and simulator naming the seed and the trace; deadlines failing with their figures; `temper-fake-git`'s replay order | The check returns its finding, and the test reads it. The text a failure prints is built by a function the test calls. |
| A limit's tightness | 25 | `jig-client-view`'s fragments at their exact size and one under; the fake peers' record caps; `hearth-fake-machine`'s held bytes | The test builds at the exact price and compares what the build used with it. One under is an assertion, untested. |
| An assertion | 45 | a start begun twice; a stale continuation; a builder's role misuse; a constructor given limits its validation refuses; a row restored a second time | It goes, keeping any half that checks an outcome. Where the cause is outside the process (a charter's model effort, a connector's kinds, the store's contents), it becomes a refusal, and its test checks the refusal. |
| Cleanup after a failure | 10 | skein-world's unwinding run ending its descendants; a held login given back past a panic; the one-process mode's starved pass | Section 5, and the boundary tests below. |

- **Each converted test loses its ignore,** and runs under Cranelift as under LLVM.
- **Tables report their first failure.** A table that runs on after a failing case only by catching it stops at that case instead, having named it.
  - `skein-rows`' `check` stops running each row under `catch_unwind` (`skein/docs/design/testing/README.md`, 6). It names a row before running it, so the first failing row ends the table, its name in the test's output. Its own tests of `panics`, of every row running after a failure and of a failure naming each failing row go with that behaviour, as do its two tests of a malformed table, which trigger assertions. A test that the failing row is named stays.
  - jig's bundle of cases (`jig/tests/core/tests/support/bundle.rs`) does the same: the first failing case ends it, its seed written before it runs.
  - `skein_rows::panics` goes, and so does the fake peers' `assert_panic`.
- **Referees and judges return their findings.** The world or the loop that drives them panics at its top with the finding, the seed and the figures, as it prints them now.
- **A harness that catches to add context,** a replay naming its seed or a story naming its step, writes that context before it runs. The runner prints a failed test's output.
- **A check that cleans up after itself,** as `skein-codegen`'s patch check removes its scratch crate, cleans up before it returns its finding, and its test sees the scratch gone after an ordinary return. A crash's leftover is the test's scratch, which goes with it.
- **The one-process mode's starved pass** (`skein/docs/design/testing/README.md`, 5; `tools/tiers/design.md`, 11.5 and 11.6) ends its process with exit 0 after its line, as it does under nextest, and the runner starts a new process for the names left, instead of unwinding with `resume_unwind`. The runner does the same after any process that ends before its names do, the test it was running failed. skein-world's tests of the starved pass and of a starved referee's teardown (`skein/tests/world/tests/suite/deadline.rs`, `real.rs`) watch a child process end, and stay across that boundary.
- **The 463 `#[should_panic]` attributes are read by the same kinds,** a tree at a time. They pass today, so they come after the 123, and the count holds them meanwhile (section 7).

## 5. What a failed test leaves outside its process

- **It's cleaned from outside the process,** once the process has ended, however it ended: a panic, an abort, or the runner's kill at a timeout. A guard's `Drop` may clean it on an ordinary return, or while a panic unwinds, never alone. skein's kits already hold this for their fixtures: no fixture outlives its test, a runner's kill among the endings, and the real loop's cleanup never depends on unwinding after the report (`skein/docs/design/testing/README.md`, 5).
- **A process a test starts dies with it,** started dying with its parent (`exec_dying_with_parent`, `skein/docs/design/runtime/shell.md`, 3) or in a group its runner ends, and a store it writes lives in the test's scratch.
- **Three kits' processes don't, yet,** each started with an ordinary `spawn` and stopped only by a `Drop`, so a test that ends without unwinding leaves it running:
  - `skein-throwaway-forgejo`'s real Forgejo and its store (`skein/testing/skein-throwaway-forgejo/src/instance.rs`);
  - forge conformance's registered runner daemon (`skein/tests/forge/conformance/src/runner.rs`);
  - smith's live agent process, a signed-in Claude (`smith/tests/agent-process/tests/live/program.rs`).
  Each starts dying with its parent, and its store goes with the test's scratch.
- **A test of an unwinding path goes** where a test of the process ending covers the same thing: an unwinding supervised run ending its descendants, an abandonment removing its cgroup while unwinding, survivors reported once while unwinding, stderr's overflow unwinding without an abort. Each process-ending path keeps its test.
- **The live tier's held login** (`temper/docs/design/testing/live-tier.md`) has the largest stake: a person's signed-in account, lent to the owned-forge landing story and to the ladder's rungs.
  - **Today, three things return it.** A rung's `run_walk` (`temper/testing/temper-box-kit/src/forge_land/rung.rs`) catches a panic in the walk, runs the store's `give-back` while the rung's box is up, stops the box, then resumes the panic. The landing story's `Lent` (`temper/testing/temper-box-kit/src/held_login.rs`) settles what is still owed in its `Drop` on any ending that unwinds, starting a box over the store when none is up. `temper/scripts/proof-live.sh`'s exit trap runs `give-back`, on a signal before it stops the test runner, then checks the record by `stat`, printing `LOGIN NOT BACK` and keeping the store when it isn't back.
  - **The gap:** the first two need unwinding. Where a test build doesn't unwind, under Cranelift or with `panic = "abort"`, the launcher alone remains, and `give-back` needs a box up. With none, the launcher prints `LOGIN NOT BACK` and keeps the store, and a person returns the record by hand.
  - **The launcher takes it all.** Once the test process has ended, stopping and reaping it first when a signal ends the run, it runs `give-back` with the story's box where one is still up. Where none is, it starts a box over the store for that alone, signs a new client in by the box's link, installs the recovery hook for that client with later imports still excluded, and runs `give-back`, as `Lent` does now. That box stops on any ending. Then `Lent` goes, and so does `run_walk`'s catch: a panic in a sitting ends the test, and the launcher returns the record either way.
  - **Until then,** a held-login story builds where its tests unwind: with LLVM.

## 6. Few tests across a process boundary

- **No helper expects a panic or an abort.** `skein_rows::panics` and `assert_panic` go (section 4), and nothing replaces them: a helper is what a test copies.
- **The tests that cross a process boundary are the ones that test it:** the fail-stop hook's fixture (`skein/tests/fail-stop`); the fork kit's cuts; the browser stories that make a test build panic to see its page restart (`skein/docs/design/ui/web/testing.md`, 2; `temper/docs/design/client/web/testing.md`); skein-world's tests of a child's end and of the one-process runner's starved pass; the kits' cleanup when a test's process ends; and the live launcher's return. Each exists today in some form, and this draft adds no suite. A test that must see a process end goes through those kits, and through their suites.
- **A design that adds an assertion says which outcome around it is tested,** citing `programming-model.md`, 10.4, rather than asking for a test of the assertion.
- **Speed:** the tests that become in-process run as fast as they do now, those that stay across a boundary cross one they already cross, and each assertion test that goes is time saved.

## 7. The checks

- **A count that only falls,** from the start: per file, of `catch_unwind`, `resume_unwind` and `#[should_panic]`, in every Rust file git tracks, `tools/`' fixtures included, which clippy never compiles. It reads a list under `tools/lints/`. A file whose count grows, or a new file with any, is a finding. A change that lowers a count lowers the list in the same change. Once a pattern counts none anywhere, the check refuses any, and keeps refusing. The gate runs it beside `lints.py` (`.config/gate/README.md`; `tools/lints/design.md`, 5).
- **Then clippy's ban too.** Once `catch_unwind` and `resume_unwind` count none, `disallowed-methods` lists them, with one reason citing `programming-model.md`, 10.4, in every `clippy.toml` the gate's clippy reads: the managed variants (`tools/lints/design.md`, 4), smith's own copies and the root `tests/clippy.toml`. The gate's clippy runs on every target, test modules and test crates included (`.config/gate/README.md`). No profile allows a `disallowed_*` lint, and an exception is a scoped `#[expect]` with its reason (`tools/lints/design.md`, 6). `#[should_panic]`, which clippy can't ban, is the count's alone.
- **Review keeps what neither sees:** a thread's panic read through its `join`, or an assertion tested by some other route.

## 8. The toolchain

- **Nothing here depends on the backend.** Once the 123 are converted, they run under Cranelift as under LLVM, and nothing waits for Cranelift to unwind. Its other gap, the SSE2 intrinsic (`crate-graph.md`, 2), isn't this draft's.
- **Production and releases build on stable with `--release`** (the user, 2026-11-13): `panic = "abort"`, as now. Nothing runs on a panic there but the hook's report, which is the point of fail-stop, and why no cleanup that matters may hang on a `Drop` during one.

## 9. Who carries what

- **temper/dev,** by default: the rule's words, the checks, the conversions in every tree, the held login's return and the kits' processes (`panics.execution.md`).
- **temper/tiers:** the one-process runner's starved pass and its going on after a process ends, in `tools/tiers/`.
- **Whoever runs the builders:** section 10's exception, read again.
- **No priority mark:** panics aren't one of v0.1.0's critical pieces. The milestone of what a failed test leaves outside its process goes first.

## 10. Later and open questions

- **A supervisor and a crash on one input.** A process restarted from what is durable meets the same input again. What a supervisor does with a process that keeps failing, backing off and reporting the input, is the runtime's to design, as hearth does for its own (`hearth/docs/design/protocol/standalone.md`, 4), and isn't drafted here.
- **Test builds that abort.** Nightly's `-Zpanic-abort-tests` would build tests as release runs, a panic ending its own test's process. After this draft nothing depends on unwinding, and the one-process runner goes on after a process ends, so it becomes possible; nobody has asked for it.
- **The builders' exception.** The builders compile an engine built for a deployment with LLVM, by a note in their setup that Cranelift code can't catch panics. Nothing of fabrig's outside tests catches one, and a dev binary aborts before unwinding, so the reason doesn't hold as written. Whether a dependency or the SSE2 intrinsic needs it is for whoever runs the builders to check, and any change to how dev.fabrig.ai is built waits for the user's word on its redeploy.
- **Open: the implementers' charters.** They ask for a test that fails without the change. They're independent of this repository's rules, so whether they cite section 3.3's exemption for an assertion is the user's to decide.
