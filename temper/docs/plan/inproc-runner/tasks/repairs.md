+++
id = "repairs"
part = "repairs"
kind = "feature"
needs = ["starved", "runner", "summary"]
files = ["tools/tiers/one_process.py", "tools/tiers/test_one_process.py", "tools/tiers/fixtures/one-process/**", ".config/nextest.toml", "temper/docs/plan/inproc-runner/plan.md", "temper/docs/design/domain/testing.status.md"]
effort = "high"
minutes = 300
+++

# inproc-runner: every tree's focused and fuzzy suites through the one-process mode, green where nextest is green

## Goal

Prove the mode: run the focused (`default`) and fuzzy suites of every tree through
`tools/tiers/one-process`, and make them green wherever nextest is green, by repairing
at its cause each test that fails only when it shares its binary's process
(`skein/docs/foundation/testing-strategy.md`, 8, *A test leaves the process as it
found it*). A test the plan can't make clean runs in a process of its own by the
`process-of-its-own` override, named in `plan.md` with why; none is dropped or
ignored. And the mode's own end-to-end rows: the command over the runner's fixtures,
to its exit statuses.

Acceptance:

- `TIERS_WRAPPER=heavy tools/tiers/one-process default` and `TIERS_WRAPPER=heavy
  tools/tiers/one-process fuzzy`, from the slot's root, the whole workspace each:
  every failure in them is either repaired, or fails under nextest too on this branch
  and on the pristine base (`heavy cargo nextest run -p PKG --profile PROFILE -E
  'test(=NAME)'`), which is no failure of this plan's and goes under `## Left` with
  both logs. A starved or unobserved test is a warning, never red. No `may depend on an
  earlier test` label stands at the end.
- Each repair is at the cause, in the test or its fixture: the threads, children,
  files, working directory, environment, process-wide statics or allocated memory it
  left behind, so that the next test starts where it did. Its test still passes under
  nextest (`heavy cargo nextest run -p PKG -E 'test(=NAME)'`), and, where it was
  labelled `may depend on an earlier test`, the pair (the earlier test and it) passes
  in one process (`tools/tiers/one-process PROFILE -p PKG -E 'test(=EARLIER) |
  test(=NAME)'`).
- A test that can't be made clean in this task: `[test-groups]` gains
  `process-of-its-own = { max-threads = "num-cpus" }` and an override in each profile
  that selects it (`filter` naming it by `package(=..) & test(=..)`, `test-group =
  "process-of-its-own"`), each with a comment saying why (`tools/tiers/design.md`,
  11.3), and `plan.md` gains a section *In a process of its own* naming it, its
  package and why, as a stated departure from the implementer's text (`plan.md` is the
  cut's). With none, neither is written.
- End-to-end rows in `tools/tiers/test_one_process.py`, over the runner's fixture
  checkout, through `cpu.py`'s `execute_one_process` and once through
  `tools/tiers/one-process`: a crash 100; a hang past the global timeout 0, with its
  unobserved test in the summary; a starved pass 0, `verdict` `starved`; a failure that
  passes alone 100, with `may depend on an earlier test` in the printed summary; a split
  binary 0 with two processes of its binary in `processes`; an empty selection 4.
- Each fault of the mode itself the real suites show (a variable a test needs, a
  libtest line the runner misreads, a group or filter misread) repaired in
  `one_process.py` with a row in `test_one_process.py` that bites.
- `temper/docs/design/domain/testing.status.md`'s **Not yet** entry `6.1: no focused
  or fuzzy run of temper's binaries a binary to a process ...` deleted once temper's
  suites are green through the mode with its memory scenarios among them, as a stated
  departure from the implementer's text; if a memory scenario needed a repair for it,
  the report says which.
- Read and compare no CPU figure: the gate and after-landing read them once the gate
  switches.

## Inputs

- `skein/docs/foundation/testing-strategy.md`, 8 (*A binary is a process*) and 8.1.
- `tools/tiers/design.md`, 11 (all of it), 10.2 (`execute_one_process`, `summary`'s
  one-process result).
- `skein/docs/design/testing.md`, 5 (*The real loop classifies its patience*): a
  starved pass in one process.
- `temper/docs/design/domain/testing.md`, 6.1.
- `temper/docs/plan/inproc-runner/notes/contracts.md`, sections 1 to 3;
  `temper/docs/plan/inproc-runner/notes/common.md`.

## Owns and may change

- Owns the files in `files`, and, for a repair, any test file, test-world module or
  test fixture in any tree (`tests/`, `testing/`, a crate's `src/tests*`) whose test
  fails only in one process: the smallest edit at the cause, each named in the report
  with the test and what it left behind. Product code changes only where the cause is
  there (a process-wide static the product keeps), named in the report as such.
- Files other drafts change (`notes/common.md`, the last rule): a repair there is the
  smallest edit that fixes the cause, named in the report.
- `.config/nextest.toml` gains only the group and the overrides above: no filter,
  timeout, slow period, priority or other group changes.
- Run the suites once through the mode first, and read their failures before any
  repair: a failure many tests share (a variable, a working directory, the runner's
  reading) is one cause, repaired once, in `one_process.py` if it is the mode's.
- If the workspace's run passes a profile's `global-timeout` and leaves tests
  unobserved, run that profile again a tree at a time (`-E` the tree's package filter
  from the comment atop `.config/nextest.toml`) so that every test is observed once;
  budgets and timeouts stay the user's.
- A test that passes alone and fails beside another every time is a repair; one that
  fails now and then in one process and never under nextest is run five times alone
  through the mode before it is called a flake, and a flake goes under `## Left` with
  its logs.

## Steps

1. The end-to-end rows; see them fail where the code doesn't yet hold them.
2. `TIERS_WRAPPER=heavy tools/tiers/one-process default`, then `fuzzy`, each into a
   directory kept outside the worktree's tracked files (`--out`); read each
   directory's summary (`cpu-summary DIR`), the failures' output in
   `command-output.txt` by `grep`, never whole.
3. For each failure: under nextest alone; on the pristine base where nextest fails it
   too; else the cause, its repair, its tests (above).
4. The two suites again, whole, through the mode, until no failure of this plan's
   stands; the status entry.
5. `cargo fmt`; for each tree touched, `python3 -I -B <tree>/scripts/old_forms.py`;
   `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` naming every package
   changed; their focused tests under nextest as well (`heavy cargo nextest run -p
   ...`); `python3 -I -B tools/tiers/test_one_process.py` and
   `python3 -I -B tools/tiers/test_cpu.py`; `python3 -I -B
   tools/citations/citations.py --base origin/main`. Commit and push.

## Report

As the implementer's text gives it, with: each suite's run through the mode (tests,
passed, failed, starved, unobserved, wall), first and last; each repair (test, what it
left behind, the fix); each test put in a process of its own and why; each fault of
the mode and its row; what is `## Left`, with logs.
