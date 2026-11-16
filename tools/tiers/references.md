# Each test's reference

Provisional, 2026-11-16. How the gate holds each test of the focused and fuzzy suites to its per-test reference: which tests run alone and how, what the base adds, how a test is judged, and the lines that let a test pass its reference with a reason. Written from `temper/docs/design/draft/test-budgets.md`, 5 and 6. The references by layer are `tools/tiers/budgets.md`, 6's; the ledger's form is `tools/tiers/budgets.md`, 7's; the one-process runner is `tools/tiers/design.md`, 11's.

## 1. In one page

- **A reference for each test** (`tools/tiers/budgets.md`, 6): its layer's per-test figure, its tree's own where it sets one, or a line with a reason (section 2).
- **Only a run of its own measures a test** (`tools/tiers/design.md`, 11): a shared process's CPU can't be divided among its tests. So a test is measured where it matters: run alone, in a process of its own, its CPU that process's less its binary's run selecting no test (section 4).
- **Which tests run alone** (section 3): with main's ledger, a new test, one the ledger lacks, and a flagged one, whose wall in the shared run is past twice its figure at main and at least 50 ms over it. Without a ledger nothing runs alone: every run but the gate's stays as it is.
- **After the suite** (section 4): in the same run, under its wall, once the suite's processes have ended; their CPU counts in no layer.
- **At the base** (section 5): a flagged test runs alone at the candidate's base too, through `tools/tiers/one-process --alone`, from the base's checkout, as the gate's runner reruns a failure there.
- **The judgement** (section 6): a new test within its reference; an existing flagged one within the larger of its reference and half again its CPU at main. Past it is red at the gate and a warning elsewhere, as a layer over its budget is.
- **In the ledger** (`tools/tiers/budgets.md`, 7): each test's CPU alone, its last three figures, beside its walls, so a flagged test has a figure at main where the base gives none.

## 2. The reasons' lines

A test may pass its reference by a line in `tools/tiers/budgets.toml`, a table of its profile's `reasons` array (`temper/docs/design/draft/test-budgets.md`, 6):

```toml
[[gate.default.reasons]]
binary = "temper-system::suite"
test = "suite::chat::a_whole_chat_lands"
cpu_s = 3
why = "it runs a whole chat, its agents, git and ctl, as checks.md, 1 allows a whole-chat story"
```

- `binary`, a binary id as nextest names it (`tools/tiers/layers.md`, 3), and `test`, the test's name in it, each a nonempty string;
- `cpu_s`, the CPU the test may take alone, in seconds, a finite positive number, never a boolean;
- `why`, a nonempty string: a sentence for the change's review, which judges it by what the test checks. The gate checks only the figure.

`budgets()` (`tools/tiers/design.md`, 10.2) refuses (`ValueError`) a line with any other key or a value outside the form, and two lines of a profile naming the same binary and test. It checks neither that the binary nor that the test exists: a line for a test that is gone holds nothing.

- **A line stands for the test's reference,** in both of section 6's rules: a new test within the line's figure, an existing one within the larger of it and half again its CPU at main. Past it, it is red again.
- **A line never raises a budget:** the test's CPU in the suite still counts in its layer (`tools/tiers/budgets.md`, 4).
- **Who writes one:** the change that adds a test past its reference, or grows one past its limit, with its reason; the review reads the reason against the tree's testing document. A guide above its layer's reference in a testing document (`temper/docs/design/testing/checks.md`, 1, a whole-chat story's 3 s) is such a test's figure, and its line is written when the test is new or grows.

## 3. Which tests run alone

A one-process run given main's ledger (`tools/tiers/budgets.md`, 7) picks them once the suite's processes have ended, from the tests whose outcome in the suite is `passed` (`tools/tiers/design.md`, 11.6). A test that failed, crashed, starved or was cancelled runs alone for no reference: its failure, or the run's starvation, is its verdict.

- **New:** the ledger's `binaries` lacks its binary, or its binary's `tests` lacks its name.
- **Flagged:** its wall in the suite is more than twice its figure at main and at least 0.05 s more than it, its figure at main the median of the ledger's walls for it, as `tools/tiers/budgets.md`, 8 takes it: a wall of 0.130 s against a median of 0.060 s flags it; 0.110 s against 0.060 s doesn't, not being past twice; nor does 0.060 s against 0.020 s, past twice but only 0.040 s over.

A run without a ledger runs nothing alone, and every test is neither: the delivery check, a builder's run and anyone's own stay as they were. A run its cancellation stopped before the suite's processes ended runs nothing alone either (section 4).

## 4. A test's run alone

- **A reference process** runs one test: `tools/tiers/design.md`, 11.4's process with one name, the test's, in a process of its own, with the same `cwd`, environment and test groups as its binary's processes in the suite.
- **An empty process** runs its binary selecting no test: the same command with the one name `tiers-one-process: no test`, which no Rust test's path can be, so libtest lists the binary's tests, runs none and exits 0. One for each binary with any test run alone.
- **A test's CPU alone** is its reference process's user plus system time less its binary's empty process's, at least 0, to the millisecond: what the test costs beyond its binary's start. It is null where the reference process did not pass, where the empty process did not exit 0, or where either has no record.
- **After the suite:** the processes start once the suite's last process (its fresh and alone diagnostics included) has ended, on the same lanes and under the same bounds: the profile's `global-timeout` from the first process's start, and `wall_hard_s`. Their order and how they share the lanes are the code's; each keeps its test's groups' `max-threads`. A process the cancellation stops leaves its test unmeasured, and the run is starved as any run its wall stopped is (`tools/tiers/design.md`, 11.7).
- **Outside every sum:** a reference or empty process counts in no CPU figure of the run: neither the suite's, nor a layer's, nor its binary's figure at the head, nor the ledger's `cpu_s` (`tools/tiers/budgets.md`, 4, 7 and 8). The suite's figures stay what the suite cost, whatever ran alone after it. Their walls are kept for no estimate (`tools/tiers/design.md`, 11.8's kept times), and a test's case in `junit.xml` stays its suite run's.
- **What the run records,** in `results.json` (`tools/tiers/design.md`, 11.8):
  - each process with a record, `kind` `reference` or `empty` among the others (`tests`, `fresh`, `alone`);
  - `references`, one entry for each test picked, ordered by binary id, then name: `binary_id`, `test_name`; `why`, `new` or `flagged` (`base` in section 5's run); `outcome`, its reference process's as `tools/tiers/design.md`, 11.6 gives one (`passed`, `failed`, `crashed`, `starved`, `cancelled`), `unobserved` where it never started, or `missing` (section 5); `process` and `empty_process`, the indexes of its reference process and of its binary's empty process in `processes`, null where none has a record; and `cpu_s`, its CPU alone, or null. An entry may hold more keys of the code's; these are the contract. A run that picks nothing, or runs without a ledger, has an empty list.
  - `command-output.txt` holds a line for each entry as it settles, its figure or its outcome; its text is the code's.
- **`run`** (`tools/tiers/design.md`, 11.1) takes `ledger=None`, main's ledger as `ledger()` reads it, which `execute_one_process` passes it as it passes it to `summary`. `--again` runs nothing alone.

## 5. At the base

A flagged test is run alone at the head and at the base, as the gate already reruns a failure at the base, and judged by the two figures (`temper/docs/design/draft/test-budgets.md`, 6). The head's figure is its run's (section 4). The base's comes from a run at the base's checkout, which the gate's runner, outside this repository, makes:

- **`tools/tiers/one-process --alone HEAD_DIR [--out DIR] [NEXTEST_ARGS...]`,** from the root of a checkout of the candidate's base, `HEAD_DIR` the head's run directory. It reads `HEAD_DIR/results.json`: its `profile`, and its `references` entries whose `why` is `flagged`.
  - With none, it writes its files with no process and builds nothing.
  - Otherwise it lists as `tools/tiers/design.md`, 11.2 does, with `--profile PROFILE`, a `-p` for each flagged binary's package (a binary id's part before its first `::`, `tools/tiers/layers.md`, 3) and `NEXTEST_ARGS`, which build: behind `TIERS_WRAPPER`, holding `TIERS_HOLD`, as one-process does (`tools/tiers/design.md`, 11.9). It runs each flagged test the listing names for its binary in a reference process, and each such binary's empty process, as section 4 runs them, under the profile's `global-timeout` in `.config/nextest.toml`.
  - It writes `DIR`'s `plan.json`, `run.json`, `results.json` and `command-output.txt` in 11.8's forms: `references` holds an entry for each flagged test, `why` `base`, and `outcome` `missing` with `cpu_s` null where the base's listing lacks its binary or its name; `tests` and `unobserved` are empty. It writes no `junit.xml`, `ledger.json` or kept times. `DIR` defaults as one-process's does and is printed first.
  - It exits 0 once its files are written, whatever its tests' outcomes, and 2 for an invalid invocation: a `HEAD_DIR` without `results.json`, or whose `results.json` has no `references`; a `DIR` not fresh; `--budgets`, `--walls`, `--ledger`, `--gate` or `--again` beside it.
- **The judgement with the base** is `tools/tiers/cpu-summary HEAD_DIR --ledger LEDGER --base DIR --gate` (section 6), its exit status the gate's verdict with the base's figures.
- **Why a command of its own** and not a head run that builds the base itself: a run builds one checkout, behind one build command, into one target; the base is another checkout, which the runner already keeps and builds to rerun a failure there.

## 6. The judgement

`summary(directory, limits=None, pair=None, expected=None, gate=False, ledger=None, base=None)` (`tools/tiers/design.md`, 10.2) judges each of the head run's `references` entries (`why` `new` or `flagged`). `base`, a directory section 5's run wrote, or none, gives its entries' `cpu_s` by binary id and name; a `base` whose `results.json` has no `references` is refused (`ValueError`), and for a run other than one-process it is refused as `ledger` is.

- **A test's reference** is its layer's, as its binary's row has it (`tools/tiers/budgets.md`, 4: its layer table's `reference_s`, else its profile's `references` entry, `checks` included), or null for an unplaced binary or a layer with none. **Its line** is its profile's `reasons` line for its binary and name, or none. **Its figure** is its entry's `cpu_s`.
- **Its figure at main,** for a flagged test: the base's `cpu_s` for it, where `base` gives one that isn't null; else the median of the ledger's `alone` figures for it (`tools/tiers/budgets.md`, 7), where it has any; else none. A new test has none.
- **Its limit,** with `R` its line's `cpu_s` where it has a line, else its reference:
  - a new test: `R`;
  - a flagged test with a figure at main `M`: the larger of `R` and `1.5 × M`;
  - a flagged test with no figure at main: its line's `cpu_s` where it has a line; otherwise none, since one already past its reference keeps its place, which only its figure at main can show (the warning below).
  
  Where `R` is null (no line, and no reference), the test has no limit and isn't judged.
- **Over** where its figure and its limit aren't null and the figure is more than the limit. An example in temper's focused step tests, reference 0.04 s: a new test at 0.050 s is over, and within with a line of 0.1 s; a flagged one at 0.120 s whose base gives 0.100 s is within its limit of 0.150 s, and at 0.160 s over it.
- **Its line in the verdict:** `test over its reference: BINARY_ID TEST_NAME: FIGURE CPU s alone, limit LIMIT s`, the figure and the limit with three decimals; the code may continue it with what set the limit (new, the reference, its line, its figure at main and whence). It goes in `over_budget` with `gate`, so the verdict is `over budget` and the exit 1, and in `warnings` without, as a layer over its budget does (`tools/tiers/budgets.md`, 4).
- **Warnings, never red:**
  - `test not measured alone: BINARY_ID TEST_NAME: OUTCOME`, for an entry whose `cpu_s` is null;
  - `test flagged without its figure at main: BINARY_ID TEST_NAME: FIGURE CPU s alone, reference REFERENCE s`, for a flagged test with no line and no figure at main whose figure is past its reference. Once the runner passes `--base`, the base gives it.
  
  The text after each line's second colon is the code's.
- **The rows:** the result's `references` holds one row for each judged entry, ordered as `results.json`'s: `binary_id`, `test_name`, `tree`, `layer`, `why`, `wall_s` (its wall in the suite), `main_wall_s` (the ledger's median, null where new), `cpu_s`, `main_cpu_s`, `main_from` (`base`, `ledger` or null), `reference_s`, `line_s`, `limit_s` and `over`. Empty without entries. `print_summary` prints a line for each row; its text is the code's.
- **`tools/tiers/cpu-summary DIR ... [--base DIR]`** gives the directory as `base`; one-process takes no `--base`, since the base's run follows the head's.

## 7. Later

- **A test that fails alone** after passing in the suite is a warning, `test not measured alone`; whether it is red, a test that depends on an earlier one, is left until one is seen.
- **A cap on the tests run alone:** none. A change that adds or renames many tests runs each alone once, a process's start and the test, shared over the lanes; a run's wall bears it, and the gate's wall limit holds it as it holds the suite.

## 8. Open questions

None.
