# inproc-runner: focused and fuzzy run a test binary's tests in one process

The focused (`default`) and fuzzy suites run through nextest today, a process per test, each measured by `tools/tiers/cpu-wrapper.c`. A process costs some milliseconds before a test checks anything, and more for a large binary's loader, so across the trees' 32,000 focused tests starting processes is most of what the suites spend. A trial on five of temper's binaries, 5,602 tests, one build: 134.8 CPU s with a process each, 92.6 with one process each and the tests in sequence, and no test failed that passed alone.

**This plan builds a runner that runs each binary's selected tests in one process, binaries in parallel, as a mode of the collector beside the nextest one.** The landing gate loads `tools/tiers/cpu.py` from the tree under test (`tools/tiers/design.md`, 10.2) and switches its focused and fuzzy steps to the new mode afterwards, tree by tree, in a change of its own. So nothing the gate calls today changes behaviour here, nextest stays for every slow tier, and `cpu-wrapper.c` stays.

**The user's word on measuring:** CPU per process is enough for now, and fast tests come before precise diagnostics. So the runner measures each process's CPU exactly, and no test's CPU at all. An outer runner can't measure a test inside a shared process: the kernel's per-process times tick at 10 ms, and libtest reports a result after it has moved on.

## What's wanted

1. **Selecting, as today.** From the caller's nextest command, the runner lists with `cargo nextest list --message-format json` (which builds, under the caller's prefix as now) and takes each binary's path, working directory and the tests whose `filter-match` matches, keeping the profile's filter, ignored handling and nextest's result for an empty selection.
2. **Running.** Each binary's selected tests in one process: `BINARY --test-threads=1 --exact NAME...`, with libtest's results through `--logfile /dev/fd/N`, the write end of a pipe the runner holds. **No file is written per test, or during the run.** Libtest runs the names in its own order. The processes run under the caller's prefix (whatever precedes `cargo` in its command), once for the whole run, not once per process.
3. **Environment, resolved per binary:** what nextest gives a test today, from the listing and cargo's metadata: `CARGO_MANIFEST_DIR`, the package's `CARGO_BIN_EXE_*` (`skein/testing/skein-scratch` reads them), package variables, `OUT_DIR`, library paths. **Not `NEXTEST`** (item 7), and a variable of the runner's own, `TIERS_ONE_PROCESS=1`.
4. **Lanes.** As many as the machine's CPUs. Binaries start longest first, by the run before's process times if the runner kept them, else by test count; a binary longer than the total over the lanes is split into a few processes. A test group's `max-threads` bounds how many processes holding its tests run at once (`temper-real-loop`, 2); priorities order the start. The runner refuses a profile with a setup script or `test-threads` other than unset.
5. **CPU, per process only.** Each process's user and system time, its own and its waited-for descendants', from `wait4`'s rusage, is the only CPU figure, and a suite's CPU is their sum. No `/proc` sampling. A per-test record carries outcome and wall time, and its CPU is null, "not measured", never a share or a guess.
6. **What a caller reads**, as 10.2 and 10.3 give for a nextest run: `run.json`, `command-output.txt`, `junit.xml` in nextest's shape (one case per test, so a caller's existing JUnit reading holds), and `summary`'s keys with their meanings. In a one-process run, `tests`' `cpu_s`, `user_s` and `system_s` are null, and two keys are added: `processes`, one entry a process (binary ID, its tests, `cpu_s`, `user_s`, `system_s`, `wall_s`, exit status or signal), and `suite`'s `cpu_by`, `"process"` (`"test"` for a nextest run). Exit statuses as `execute`'s: 1 over a CPU budget, 100 when a failure stands, 0 when only starved, nextest's 4 for an empty selection. Limits as 7.3's, `cpu_suite_s` and `wall_hard_s`; a table with `cpu_test_s` is refused in this mode. New functions sit beside 10.2's, documented there, with the same arguments where they apply.
7. **Starved, in one process.** A starved pass ends the process with `std::process::exit(0)` when `NEXTEST` is set: `Starved::pass` in `skein/testing/skein-world/src/deadline.rs`, `real.rs`'s two deadline paths, and the real conformance's `forward_starved` (`skein/tests/conformance/real/tests/suite/signal_sender.rs`, and `signal_sender_fixture`'s patience). **Under `TIERS_ONE_PROCESS`, a starved pass ends only its test:** it cleans up as now, writes the same `skein-world: starved:` line, and unwinds rather than exiting, and the runner records that test starved: a warning, never red and never passed. Under nextest and outside both, it behaves as now. Every place that keys on `NEXTEST` for starvation does the same; `skein/tests/world/tests/suite/real.rs` removes `NEXTEST` from children, and removes the new variable too.
8. **A crash or a hang.** A process that ends without a result for the test it was running fails that test, with the process's status and output, and the runner starts a fresh process on that binary's tests not yet run. The profile's `global-timeout` holds as now; tests not reached are unobserved, never passed. No per-test bound is added: a new one would be a policy, the user's.
9. **A failure stays red.** The runner also runs it once more alone, in a fresh process, as a diagnostic: passing alone is labelled "may depend on an earlier test" in the summary and the output, with the tests that ran before it in that process, and is never turned into a pass.
10. **A test that fails only in one process leaves something behind.** Repair it at the cause, in the test or its fixture: threads, children, files, working directory, environment, process-wide statics or allocated memory, so that the next test starts where it did. One the plan can't make clean runs in a process of its own, by a nextest override the runner honours, named in `plan.md` with why; none is dropped or ignored.
11. **By hand:** `tools/tiers/one-process PROFILE [nextest filter arguments]` runs a profile so and prints the summary as `cpu-summary` does.
12. **Docs.** `tools/tiers/design.md` gains the mode's section, and 10.2 and 10.3 its functions and files. `skein/docs/foundation/testing-strategy.md`, 8, **A test is a process**, becomes, with the user's approval of the direction:

    > **A binary is a process.** The focused and fuzzy suites run each test binary's tests in one process, one at a time; binaries run in parallel. A process costs some milliseconds before a test checks anything, and more for a large binary, so starting one per test would spend most of a suite's budget on starting. **A test leaves the process as it found it:** its threads, descendants, files, working directory, environment and allocated memory, so that the next test starts where it did. The slow tiers run a test per process.

    With it, the sentences of 8 and 8.1 that assume a process per test in focused and fuzzy (a fixture built once per process, a test's cost to the suite, per-test CPU with its descendants), saying that a one-process run measures CPU per process; and `temper/docs/design/domain/testing.md`, 6.1's "each test runs in its own process, so a memory scenario and a sweep never share an allocator's counters".

## Limits

- **Prove it by running the focused and fuzzy suites through the new mode, every tree, green where nextest is green,** and by the mode's own tests beside `tools/tiers/test_cpu.py`, with fixtures for a crash, a hang past the global timeout, a starved pass, a failure that passes alone, and a split binary. Read and compare no CPU figure: the gate and after-landing read them once the gate switches.
- Nextest's path, its records and its results stay as they are, and 10.2's contract holds for it: a key may be added, none removed or changed in meaning.
- No budget, timeout, filter, tier or build setting changes.
- `walk = false`. Effort high. Four or five tasks: skein-world's starved report first, then the runner, its summary and command, the repairs item 10 finds, and the docs.

## Sources

Too large for a source, so read by section: `tools/tiers/design.md`, 7.1 to 7.3 and 10; `skein/docs/foundation/testing-strategy.md`, 8 and 8.1; `skein/testing/skein-world/src/real.rs`, the two deadline paths.

- `tools/tiers/cpu.py`
- `skein/testing/skein-world/src/deadline.rs`
- `skein/tests/conformance/real/tests/suite/signal_sender.rs`
- `.config/nextest.toml`

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories. No walk, render check or throwaway driver. Running the focused and fuzzy suites through the new mode is running focused and fuzzy.
- **No budget, timeout, deadline or build setting changes.** Test builds stay unoptimized.
- **Decide, don't ask.** A stored format or protocol may change freely, with no compatibility. A small edit just outside a task's file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
