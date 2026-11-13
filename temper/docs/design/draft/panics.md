# panics: nothing catches a panic

Draft, 2026-11-13. What fabrig does with a panic, from the user's question that day: whether to forbid catching panics, since Cranelift can't unwind and they prefer error handling or an abort. They then asked that the tests that catch panics become panic-less too, without a growing set of tests that start a process to see an abort, and accepted the rule's words in skein's `programming-model.md` as section 3.2 gives them. It reads main at `6962c8fcc7`.

## 1. In one page

- **Nothing outside tests catches a panic today** (section 2). Release aborts, and a binary built to unwind aborts before anything unwinds. Tests catch: `catch_unwind` in 131 files and `#[should_panic]` in 150. 123 tests have been ignored since 2026-11-12 because Cranelift, which the builders use for dev builds, can't unwind.
- **The rule** (section 3). A panic is our own code's bug, and it aborts. Whatever anything outside the process can cause is an error, the store's contents included. Nothing catches a panic, tests included, and no test triggers an assertion. A check whose job is to fail returns its finding. These are two sentences and a wider clause in `programming-model.md`, 10.4, and a row's words in 10.1.
- **No test starts a process to see an abort** (section 4). The 123 become in-process tests, by kind: about 40 test a check that fails, which returns its finding instead; about 25 test a limit's tightness, which compares what a build used with its price; about 45 trigger an assertion, and go, or become refusals where the cause is outside the process; about 10 test cleanup after a failure, which section 5 takes.
- **Cleanup after a failed test happens outside its process** (section 5), once the process has ended, however it ended. The live tier's held login is the one with a stake: its last case still needs unwinding, and the launcher takes it.
- **The count stays low and falls** (sections 6 and 7). No helper expects a panic or an abort. A gate check counts `catch_unwind`, `resume_unwind` and `#[should_panic]` per file, and the count only falls. Once the first two reach none, clippy's `disallowed-methods` bans them; `#[should_panic]` follows when its count does.
- **Nothing here depends on the backend** (section 8): once it lands, LLVM and Cranelift run the same tests, and production, built on stable with `--release`, aborts as it does now.

## 2. What main holds

- **Production code catches nothing.** No `catch_unwind` or `resume_unwind` is outside test code. The one in a `src/domain/` path, `jig/crates/jig-domain-people/src/domain/start.rs`, is in its `#[cfg(test)]` module. Four `set_hook` calls report a panic and catch none: `skein-fail-stop`'s, which aborts, and three in web code, skein's playground worker, `skein-ui-web`'s restart and temper's playground page, where the abort is a trap.
- **Release aborts.** The root `Cargo.toml`'s `release` and `web` profiles say `panic = "abort"`. The dev profile unwinds, as the test harness needs, and `skein_fail_stop::install()`, first in every `main`, makes a dev binary abort before anything unwinds (`skein/docs/foundation/programming-model.md`, 2.1).
- **Tests catch,** counted as lines naming each, `use` lines included, so roughly:

| Tree | `catch_unwind` | `resume_unwind` | `#[should_panic]` | Ignored for Cranelift |
|---|---|---|---|---|
| skein | 53, in 30 files | 2 | 286, in 72 files | 49 |
| jig | 95, in 66 files | 1 | 95, in 38 files | 48 |
| temper | 57, in 32 files | 6 | 31, in 19 files | 24 |
| smith | 4, in 2 files | 1 | 30, in 8 files | 1 |
| hearth | 1, in 1 file | 0 | 25, in 12 files | 1 |
| `tools/` | 0 | 1 | 1 | 0 |

- **Cranelift.** Its trial failed 151 of the focused suite's 34,696 tests: it doesn't catch panics, and it lacks an SSE2 intrinsic a dependency uses (`crate-graph.md`, 2). The 123 tests that catch a panic, or need cleanup to run while one unwinds, carry `#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]`. The builders compile dev builds with Cranelift since 2026-11-13 (`gate-compile.md`, 2), by their environment, not the repository's toolchain file. `#[should_panic]` still passes there, since the test harness's own catch is compiled by LLVM with the standard library: none of the 468 is among the ignored, which is the draft's reading, not a run. nextest runs the suites, each test a process of its own (`.config/nextest.toml`), so a test that aborts ends itself alone.

## 3. The rule

### 3.1 What a panic is

- **A panic is our own code's bug:** an assertion that something can't happen, which happened. Past it, the process can't trust its memory, so it aborts, and a supervisor restarts it from what is durable, the path a power cut or a kill takes anyway (`skein/docs/foundation/programming-model.md`, 2.1). Catching one adds a second recovery that nothing tests, running on state the panic left half-changed. Since release aborts, that recovery would run in development alone.
- **Whatever anything outside the process can cause is an error:** a peer, configuration, or the store's contents. The last is how jig's restart already refuses a link it can't make, "never a panic" (`jig/docs/design/domain/engine/restart.md`, from the user's request `store-crash-safe` of 2026-11-09): a panic on what the store holds is a crash at every restart, where a refusal starts the deployment failed and says why.
- **An abort ends every session the process serves,** and an input that trips a bug trips it again after the restart. That is release's behaviour today, which this draft doesn't change. What a supervisor does with a process that keeps failing on one input is section 10's.

### 3.2 The words in `programming-model.md`

Accepted by the user, 2026-11-13. 10.4's bullet on assertions becomes:

> - **Assertions are fail-stop** and are for the service's own bugs, never for anything from outside the process: a peer, configuration or the store's contents. `expect` names the invariant it relies on. Nothing catches a panic, tests included, and no test triggers an assertion: a test sees what an assertion guards through the outcomes around it. A check whose job is to fail, such as a referee or a judge, returns its finding, and only its caller panics.

10.1's row "Fail-stop" adds "nothing catches a panic" to what holds it, and to what checks it clippy's `disallowed_methods` and the gate's count of `#[should_panic]` (section 7). 2.1's "Panics abort" stays as it is.

### 3.3 What it asks of a change

- **An assertion needs no test of its own.** A change that adds one adds no behaviour, so nothing fails without it. What it guards is tested through what the code returns and does around it. A request for a test that fails without the change asks nothing of an assertion, so nobody writes a test that starts a process to see one fire.
- **A cause from outside the process is a refusal,** tested in-process like any outcome.
- **A check whose job is to fail,** a referee, a judge, a kit's comparison or a table, returns its finding as a value. The world, the loop or the test that called it panics on it, once, at its top. The check's own tests read the finding.

## 4. Tests without catching

The 123 ignored tests, by what each catches. The split is a script's, by each test's name and what its catch wraps, so it's rough: each milestone reads its own tests again.

| Kind | About | Examples | Becomes |
|---|---|---|---|
| A check that fails | 40 | the referees in `jig/tests/views`; io's judge; skein's world and simulator naming the seed and the trace; deadlines failing with their figures; `skein-rows`' own tests; `temper-fake-git`'s replay order | The check returns its finding, and the test reads it. The text a failure prints is built by a function the test calls. |
| A limit's tightness | 25 | `jig-client-view`'s fragments at their exact size and one under; the fake peers' record caps; `hearth-fake-machine`'s held bytes | The test builds at the exact price and compares what the build used with it. One under is an assertion, untested. |
| An assertion | 45 | a start begun twice; a stale continuation; a builder's role misuse; a constructor given limits its validation refuses; a row restored a second time | It goes, keeping any half that checks an outcome. Where the cause is outside the process (a charter's model effort, a connector's kinds, the store's contents), it becomes a refusal, and its test checks the refusal. |
| Cleanup after a failure | 10 | skein-world's unwinding run ending its descendants; a held login given back past a panic; the one-process mode's starved pass | Section 5. |

- **Each converted test loses its ignore,** and runs under Cranelift as under LLVM.
- **The 468 `#[should_panic]` tests are read by the same kinds,** a tree at a time. They pass today, so they come after the 123, and the count holds them meanwhile (section 7).
- **The kits change with their tests:**
  - `skein-rows`' `check` stops running each row under `catch_unwind` (`skein/docs/design/testing/README.md`, 6). It names a row before running it, so the first failing row ends the table, its name in the test's output. `skein_rows::panics` goes, and so does the fake peers' `assert_panic`.
  - Referees and judges return their findings. The world or the loop that drives them panics at its top with the finding, the seed and the figures, as it prints them now.
  - A harness that catches to add context, a table naming its failing case or a replay naming its seed, writes that context before it runs. nextest prints a failed test's output.
  - The one-process mode's starved pass (`skein/docs/design/testing/README.md`, 5; `tools/tiers/design.md`, 11) ends its process with exit 0 after its line, as it does under nextest, and the mode's runner starts a new process for the names left, instead of unwinding with `resume_unwind`.

## 5. Cleanup after a failed test

- **What a failed test owes is settled outside its process,** once the process has ended, however it ended: a panic, an abort, or the runner's kill at a timeout. A guard's `Drop` may settle it on an ordinary return, never alone. skein's kits already hold this for fixtures: no fixture outlives its test, a runner's kill among the endings, and the real loop's cleanup never depends on unwinding after the report (`skein/docs/design/testing/README.md`, 5).
- **A test of an unwinding path goes** where a test of the process ending covers the same thing: an unwinding supervised run ending its descendants, an abandonment removing its cgroup while unwinding, survivors reported once while unwinding, stderr's overflow unwinding without an abort. Each process-ending path keeps its test.
- **The live tier's held login** (`temper/docs/design/testing/live-tier.md`) is the one with a stake: a login lent from a person's account to the owned-forge landing story.
  - **Today,** `Lent` (`temper/testing/temper-box-kit/src/held_login.rs`) settles it in its `Drop`, on every ending. `temper/scripts/proof-live.sh`'s exit trap also runs the store's `give-back` once the tests have ended, then checks the record by `stat`, printing `LOGIN NOT BACK` and keeping the store when it isn't back.
  - **The gap:** `give-back` needs the story's box up. When a panic has stopped the box, only `Lent`'s `Drop` starts one over the store to give the record back. Where a test build doesn't unwind, under Cranelift or with `panic = "abort"`, that `Drop` never runs: the launcher prints `LOGIN NOT BACK` and keeps the store, and a person returns the record by hand.
  - **The launcher takes `Lent`'s last case.** When `give-back` fails with no box up, it starts a box over the store for that alone, signs a new client in by the box's link, installs the recovery hook for that client with later imports still excluded, and runs `give-back`, as `Lent` does now. That box stops on any ending. Then `Lent` goes, and so does the catch in `run_walk` (`temper/testing/temper-box-kit/src/forge_land/rung.rs`): a panic in a sitting ends the test, and the launcher returns the record either way.
  - **Until then,** a held-login story builds where its tests unwind: with LLVM.

## 6. Few tests across a process boundary

- **No helper expects a panic or an abort.** `skein_rows::panics` and `assert_panic` go (section 4), and nothing replaces them: a helper is what a test copies.
- **The tests that cross a process boundary are the ones that test it:** the fail-stop hook's fixture (`skein/tests/fail-stop`), the fork kit's cuts, the kits' cleanup when a test's process ends, the live launcher. They exist today, and this draft adds none. A test that must see a process end goes through those kits, and through their suites.
- **A design that adds an assertion says which outcome around it is tested,** citing `programming-model.md`, 10.4, rather than asking for a test of the assertion.
- **Speed:** every converted test runs in-process, as fast as it does now, and each assertion test that goes is time saved.

## 7. The checks

- **A count that only falls,** from the start: per file, of `catch_unwind`, `resume_unwind` and `#[should_panic]`, against a list the check reads, under `tools/lints/`. A file whose count grows, or a new file with any, is a finding. A change that lowers a count lowers the list in the same change. The gate runs it, as it runs `lints.py` (`tools/lints/design.md`, 5).
- **Then a ban.** Once `catch_unwind` and `resume_unwind` count none, every `clippy.toml` variant's `disallowed-methods` lists them (`tools/lints/design.md`, 4), with one reason citing `programming-model.md`, 10.4, and the count drops them. No profile allows a `disallowed_*` lint, and an exception is a scoped `#[expect]` with its reason (`tools/lints/design.md`, 6). `#[should_panic]`, which clippy can't ban, keeps its count until none, and then the check refuses any.
- **Review keeps what neither sees:** a thread's panic read through its `join`, or an assertion tested by some other route.

## 8. The toolchain

- **Nothing here depends on the backend.** Once the 123 are converted, they run under Cranelift as under LLVM, and nothing waits for Cranelift to unwind. Its other gap, the SSE2 intrinsic (`crate-graph.md`, 2), isn't this draft's.
- **Production and releases build on stable with `--release`** (the user, 2026-11-13): `panic = "abort"`, as now. Nothing runs on a panic there but the hook's report, which is the point of fail-stop, and why no cleanup that matters may hang on a `Drop` during one.

## 9. Who carries what

- **temper/dev,** by default: the rule's words, the checks, the conversions in every tree, and the held login's return (`panics.execution.md`).
- **temper/tiers:** the one-process mode's starved pass, in `tools/tiers/`.
- **Whoever runs the builders:** section 10's exception, read again.
- **No priority mark:** panics aren't one of v0.1.0's critical pieces. The held login's milestone goes first.

## 10. Later and open questions

- **A supervisor and a crash on one input.** A process restarted from what is durable meets the same input again. What a supervisor does with a process that keeps failing, backing off and reporting the input, is the runtime's to design, as hearth does for its own (`hearth/docs/design/protocol/standalone.md`, 4), and isn't drafted here.
- **Test builds that abort.** Nightly's `-Zpanic-abort-tests` would build tests as release runs, a panic ending its own test's process under nextest. After this draft nothing depends on unwinding, so it becomes possible; nobody has asked for it.
- **The builders' exception.** The builders compile an engine built for a deployment with LLVM, by a note in their setup that Cranelift code can't catch panics. Nothing of fabrig's outside tests catches one, and a dev binary aborts before unwinding, so the reason doesn't hold as written. Whether a dependency or the SSE2 intrinsic needs it is for whoever runs the builders to check, and any change to how dev.fabrig.ai is built waits for the user's word on its redeploy.
- **Open: the implementers' charters.** They ask for a test that fails without the change. They're independent of this repository's rules, so whether they cite section 3.3's exemption for an assertion is the user's to decide.
