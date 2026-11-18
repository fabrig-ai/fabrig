# Each test's reference

Provisional, 2026-11-16. How the gate holds each test of the focused and fuzzy suites to its per-test reference: which tests run alone and how, what the base adds, how a test is judged, and the lines that let a test pass its reference with a reason. Written from `temper/docs/design/draft/test-budgets.md`, 5 and 6. The references by layer are `tools/tiers/budgets.md`, 6's; the ledger's form is `tools/tiers/budgets.md`, 7's; the one-process runner is `tools/tiers/design.md`, 11's.

## 1. In one page

- **A reference for each test** (`tools/tiers/budgets.md`, 6): its layer's per-test figure, its tree's own where it sets one, or a line with a reason, which the user writes, never a change (section 2).
- **Only a run of its own measures a test** (`tools/tiers/design.md`, 11): a shared process's CPU can't be divided among its tests. So a test is measured where it matters: run alone, in a process of its own, its CPU that process's less its binary's run selecting no test (section 4).
- **Which tests run alone** (section 3): with main's ledger, a new test, one the ledger lacks, and a flagged one, whose wall in the shared run is past twice its figure at main and at least 50 ms over it. Without a ledger nothing runs alone: every run but the gate's and a delivery check's with main's ledgers stays as it is.
- **After the suite** (section 4): in the same run, under its wall, once the suite's processes have ended; their CPU counts in no layer. A test whose figure passes its limit, main's ledger standing for the base, runs alone twice more, and its figure at the head is the lowest of its three; three figures past half again apart name it as swinging, a warning.
- **At the base** (section 5): a flagged test runs alone at the candidate's base too, three times, through `tools/tiers/one-process --alone`, from the base's checkout, as the gate's runner reruns a failure there; its figure there is the highest of its three.
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
- **Who writes one: the user,** never the change. A line is the user's figure, as a budget is (`temper/docs/design/draft/test-budgets.md`, 6; the user, 11-18): the gate refuses a change that adds a line or raises one's `cpu_s`, and passes one that lowers or drops it (`tools/tiers/budgets.md`, 10.1).
  - **A test past its reference,** new or grown past its limit, is repaired by the strategy's 8.1, as its red's last line says: placed in the tier it belongs to, or made cheaper; failing both, its goal takes its measurements to the user, who decides. A line the user gives lands by a change that touches `tools/tiers/budgets.toml` alone, written on their word, which the gate passes whatever it does there (`tools/tiers/budgets.md`, 10.1). Its `why` says what the test checks, for the review.
  - **A guide above its layer's reference** in a testing document (`temper/docs/design/testing/checks.md`, 1, a whole-chat story's 3 s) is a figure the user has given for such a test: its line is still the user's to write, asked for with the test's measurements when the test is new or grows.

## 3. Which tests run alone

A one-process run given main's ledger (`tools/tiers/budgets.md`, 7) picks them once the suite's processes have ended, from the tests whose outcome in the suite is `passed` (`tools/tiers/design.md`, 11.6). A test that failed, crashed, starved or was cancelled runs alone for no reference: its failure, or the run's starvation, is its verdict.

- **New:** the ledger's `binaries` lacks its binary, or its binary's `tests` lacks its name.
- **Flagged:** its wall in the suite is more than twice its figure at main and at least 0.05 s more than it, its figure at main the median of the ledger's walls for it, as `tools/tiers/budgets.md`, 8 takes it: a wall of 0.130 s against a median of 0.060 s flags it; 0.110 s against 0.060 s doesn't, not being past twice; nor does 0.060 s against 0.020 s, past twice but only 0.040 s over.

A run without a ledger runs nothing alone, and every test is neither: a builder's run, anyone's own and a delivery check without main's ledgers stay as they were. A delivery check with them runs its new and flagged tests alone as the gate does, and, building nothing at its base, judges a flagged test by the ledger's figures alone where it has them and otherwise warns (section 6; `tools/pre-pr/design.md`, 4). A run its cancellation stopped before the suite's processes ended runs nothing alone either (section 4).

## 4. A test's run alone

- **A reference process** runs one test: `tools/tiers/design.md`, 11.4's process with one name, the test's, in a process of its own, with the same `cwd`, environment and test groups as its binary's processes in the suite.
- **An empty process** runs its binary selecting no test: the same command with the one name `tiers-one-process: no test`, which no Rust test's path can be, so libtest lists the binary's tests, runs none and exits 0. One for each binary with a test in a round (below).
- **A run alone** is a test's reference process and its binary's empty process of the same round. **The test's CPU alone** in it, the run's figure, is its reference process's user plus system time less that empty process's, at least 0, to the millisecond: what the test costs beyond its binary's start. It is null where the reference process did not pass, where the empty process did not exit 0, or where either has no record.
- **Measured again before red** (`temper/docs/design/draft/test-budgets.md`, 6). One run alone is a sample: below a tenth of a second a test's CPU alone swings with its process past half again on the same code. So the runs alone go in rounds, each with empty processes of its own, so that each of a test's figures is a sample of both its processes:
  - **The first round** runs each picked test (section 3) once, and an empty process for each of their binaries.
  - **Its limit as it runs:** once the first round has ended, each test whose first figure isn't null is held to its limit as section 6 sets it, with main's ledger standing for the base, which hasn't run. With `R` its line's `cpu_s` where it has a line, else its reference (section 6): a new test's limit is `R`; a flagged test's the larger of `R` and `1.5 × M`, `M` the median of the ledger's `alone` figures for it (`tools/tiers/budgets.md`, 7); and a flagged test the ledger has no `alone` figure for is held to `R`, the lowest limit its base can set (section 5). Where `R` is null, it has no limit.
  - **Two rounds more:** each test whose first figure is more than its limit runs alone again in the second round and again in the third, each round with an empty process for each binary it runs. A test within its limit, or with none, runs once.
  - **Its figure at the head** is the lowest of its runs' figures that aren't null, or null where none has one. In temper's focused step tests, reference 0.04 s: a new test at 0.052 s runs twice more, at 0.047 and 0.031 s, and its figure is 0.031 s, within its reference; a flagged one whose ledger figures alone have a median of 0.020 s, so a limit of 0.040 s, runs once at 0.035 s.
  - **One rule for the limit:** the limit a round holds a first figure to is computed by the code that computes section 6's, in `tools/tiers/cpu.py`, which `tools/tiers/one_process.py` calls as it calls `summary`; the two differ only where the base has run, and for a flagged test the ledger has no `alone` figure for.
- **After the suite:** the first round starts once the suite's last process (its fresh and alone diagnostics included) has ended, and each later round once the round before it has ended, on the same lanes and under the same bounds: the profile's `global-timeout` from the first process's start, and `wall_hard_s`. A round's order and how it shares the lanes are the code's; each process keeps its test's groups' `max-threads`. A process the cancellation stops leaves its run unmeasured, and the run is starved as any run its wall stopped is (`tools/tiers/design.md`, 11.7); a round the cancellation came before never starts, and its runs are `unobserved`.
- **Outside every sum:** a reference or empty process counts in no CPU figure of the run: neither the suite's, nor a layer's, nor its binary's figure at the head, nor the ledger's `cpu` figures (`tools/tiers/budgets.md`, 4, 7 and 8). The suite's figures stay what the suite cost, whatever ran alone after it. Their walls are kept for no estimate (`tools/tiers/design.md`, 11.8's kept times), and a test's case in `junit.xml` stays its suite run's.
- **What the run records,** in `results.json` (`tools/tiers/design.md`, 11.8):
  - each process with a record, `kind` `reference` or `empty` among the others (`tests`, `fresh`, `alone`);
  - `references`, one entry for each test picked, ordered by binary id, then name:
    - `binary_id`, `test_name`; `why`, `new` or `flagged` (`base` in section 5's run);
    - `runs`, its runs alone in round order, one at the head where it ran once and three where it ran again, three at the base, none for a `missing` entry. Each is `outcome`, its reference process's as `tools/tiers/design.md`, 11.6 gives one (`passed`, `failed`, `crashed`, `starved`, `cancelled`), or `unobserved` where it never started; `process` and `empty_process`, the indexes of its reference process and of its round's empty process for the binary in `processes`, null where none has a record; and `cpu_s`, its figure, or null;
    - `cpu_s`, the test's figure: the lowest of its runs' figures at the head, the highest at the base (section 5), or null where no run has one;
    - `outcome`, `process` and `empty_process`, those of the run whose figure `cpu_s` takes, else of its first run, or `missing` and null where the base lacks the test (section 5);
    - `run_limit_s`, the limit the first round held its first figure to (above), or null where it had none, its first figure is null, or the run is the base's.
    
    An entry may hold more keys of the code's; these are the contract. A run that picks nothing, or runs without a ledger, has an empty list. `ledger.json` folds each entry's `cpu_s` as the test's figure alone (`tools/tiers/budgets.md`, 7): the lowest of its runs at the head, in the ledger's form as it was.
  - **A run's before this form:** an entry without `runs`, as a base's checkout older than this form writes, is read as one run of its own `outcome`, `process`, `empty_process` and `cpu_s`; nothing else reads `runs` (section 6).
  - `command-output.txt` holds a line for each run as it settles, its figure or its outcome, and, for a test run more than once, a line naming the figure it takes; their text is the code's.
- **`run`** (`tools/tiers/design.md`, 11.1) takes `ledger=None`, main's ledger as `ledger()` reads it, which `execute_one_process` passes it as it passes it to `summary`. `--again` runs nothing alone.

## 5. At the base

A flagged test is run alone at the head and at the base, as the gate already reruns a failure at the base, and judged by the two figures (`temper/docs/design/draft/test-budgets.md`, 6). The head's figure is its run's, the lowest of its runs there (section 4). The base's is the highest of three runs alone at the base's checkout, which the gate's runner, outside this repository, makes: the limit the base sets is held to its highest figure, so the base's noise counts for the test, never against it, as the head's does through its lowest.

- **`tools/tiers/one-process --alone HEAD_DIR [--out DIR] [NEXTEST_ARGS...]`,** from the root of a checkout of the candidate's base, `HEAD_DIR` the head's run directory. It reads `HEAD_DIR/results.json`: its `profile`, and its `references` entries whose `why` is `flagged`.
  - With none, it writes its files with no process and builds nothing.
  - Otherwise it finds each flagged binary's target in the base's `cargo metadata --no-deps --format-version 1`, which builds nothing: the member's target with `test` true whose id, as `tools/tiers/layers.md`, 3 makes it, is the binary's. It lists as `tools/tiers/design.md`, 11.2 does, once for each package with a target found, in name order: `--profile PROFILE`, `-p PACKAGE`, the flag of each of its found targets, and `NEXTEST_ARGS`. A target's flag is by its id: `--lib` for `PACKAGE`, `--bin TARGET` for `PACKAGE::bin/TARGET`, `--test TARGET` for `PACKAGE::TARGET`, `--example TARGET` and `--bench TARGET` for `PACKAGE::example/TARGET` and `PACKAGE::bench/TARGET`. With `temper-box::suite` and `temper-store` flagged, it lists `-p temper-box --test suite`, then `-p temper-store --lib`.
  - **A listing a package,** since cargo's target flags hold for every package one invocation names: each listing builds its targets and what they depend on alone, behind `TIERS_WRAPPER`, holding `TIERS_HOLD`, as one-process does (`tools/tiers/design.md`, 11.9), and the workspace's feature unification (`.cargo/config.toml`) gives each dependency the features a whole build gives it. The listings' binaries together are its plan. It runs each flagged test its binary's listing names alone in three rounds, as section 4 runs its rounds: each round a reference process for each such test and an empty process for each of their binaries, every test in all three, whatever its figures, each round once the one before it has ended, all under the profile's `global-timeout` in `.config/nextest.toml`. Where no flagged binary's target is found, it lists and builds nothing.
  - It writes `DIR`'s `plan.json`, `run.json`, `results.json` and `command-output.txt` in 11.8's forms: `references` holds an entry for each flagged test in section 4's form, `why` `base`, its three `runs`, its `cpu_s` the highest of their figures that aren't null, and its `run_limit_s` null; `outcome` `missing`, with no runs and `cpu_s` null, where the base lacks its binary's target or its binary's listing lacks its name; `tests` and `unobserved` are empty. A flagged test with figures 0.080, 0.100 and 0.090 s at the base has 0.100 s there. It writes no `junit.xml`, `ledger.json` or kept times. `DIR` defaults as one-process's does and is printed first.
  - It exits 0 once its files are written, whatever its tests' outcomes, and 2 for an invalid invocation: a `HEAD_DIR` without `results.json`, or whose `results.json` has no `references`; a `DIR` not fresh; `--budgets`, `--walls`, `--ledger`, `--gate` or `--again` beside it.
- **The judgement with the base** is `tools/tiers/cpu-summary HEAD_DIR --budgets tools/tiers/budgets.toml --ledger LEDGER --base DIR --gate` (section 6), its exit status the gate's verdict with the base's figures.
- **Why a command of its own** and not a head run that builds the base itself: a run builds one checkout, behind one build command, into one target; the base is another checkout, which the runner already keeps and builds to rerun a failure there.

## 6. The judgement

`summary(directory, limits=None, pair=None, expected=None, gate=False, ledger=None, base=None)` (`tools/tiers/design.md`, 10.2) judges each of the head run's `references` entries (`why` `new` or `flagged`). `base`, a directory section 5's run wrote, or none, gives its entries' `cpu_s` by binary id and name; a `base` whose `results.json` has no `references` is refused (`ValueError`), and for a run other than one-process it is refused as `ledger` is.

- **A test's reference** is its layer's, as its binary's row has it (`tools/tiers/budgets.md`, 4: its layer table's `reference_s`, else its profile's `references` entry, `checks` included), or null for an unplaced binary or a layer with none. **Its line** is its profile's `reasons` line for its binary and name, or none. **Its figure** is its entry's `cpu_s`: the lowest of its runs alone at the head (section 4).
- **Its figure at main,** for a flagged test: the base's `cpu_s` for it, the highest of its runs alone there (section 5), where `base` gives one that isn't null; else the median of the ledger's `alone` figures for it (`tools/tiers/budgets.md`, 7), where it has any; else none. A new test has none.
- **Its limit,** with `R` its line's `cpu_s` where it has a line, else its reference:
  - a new test: `R`;
  - a flagged test with a figure at main `M`: the larger of `R` and `1.5 × M`;
  - a flagged test with no figure at main: its line's `cpu_s` where it has a line; otherwise none, since one already past its reference keeps its place, which only its figure at main can show (the warning below).
  
  Where `R` is null (no line, and no reference), the test has no limit and isn't judged.
- **Over** where its figure and its limit aren't null and the figure is more than the limit. An example in temper's focused step tests, reference 0.04 s: a new test at 0.050 s is over, and within with a line of 0.1 s; a flagged one at 0.120 s whose base gives 0.100 s is within its limit of 0.150 s, and at 0.160 s over it. So a test is red only where the lowest of its runs at the head passes the limit the highest of its runs at the base sets, or its reference where it is new: a new test whose runs at the head give 0.052, 0.047 and 0.031 s has 0.031 s, within.
- **Its line in the verdict:** `test over its reference: BINARY_ID TEST_NAME: FIGURE CPU s alone, limit LIMIT s`, the figure and the limit with three decimals; the code may continue it with what set the limit (new, the reference, its line, its figure at main and whence). It goes in `over_budget` with `gate`, so the verdict is `over budget` and the exit 1, and in `warnings` without, as a layer over its budget does (`tools/tiers/budgets.md`, 4). In `over_budget` its entry ends with the repair line, on a line of its own (`tools/tiers/budgets.md`, 10.1), so the test's goal reads where its repair starts: the strategy, then 8.1's three questions, never a line of its own.
- **Warnings, never red:**
  - `test not measured alone: BINARY_ID TEST_NAME: OUTCOME`, for an entry whose `cpu_s` is null, and for each run at the head without a figure of an entry whose `cpu_s` isn't null, with that run's outcome (the code may name its round after it);
  - `test flagged without its figure at main: BINARY_ID TEST_NAME: FIGURE CPU s alone, reference REFERENCE s`, for a flagged test with no line and no figure at main whose figure is past its reference. Once the runner passes `--base`, the base gives it.
  - `test swings alone at the head: BINARY_ID TEST_NAME: A, B, C CPU s alone`, the figures of its three runs at the head in round order, each with three decimals, for an entry whose three runs at the head all have figures, the largest more than half again the smallest (`temper/docs/design/draft/test-budgets.md`, 6). They are compared in whole milliseconds, as they are kept: it swings where the largest's milliseconds times 2 are more than the smallest's times 3, so 0.052, 0.047 and 0.031 s swing (104 against 93) and 0.052, 0.047 and 0.040 s don't (104 against 120); a smallest of 0 makes any larger figure a swing. It goes in `warnings` with `gate` and without, after the entry's other lines, carries no repair line (`tools/tiers/budgets.md`, 10.1) and changes no verdict, row's `over` or exit status: the test is judged by its lowest figure all the same, and its swing, a break of the strategy's section 10, is for a person to repair at its cause, never a red. The base's runs name no swing: their highest only sets a limit.
  
  The text after each line's second colon is the code's.
- **The rows:** the result's `references` holds one row for each judged entry, ordered as `results.json`'s: `binary_id`, `test_name`, `tree`, `layer`, `why`, `wall_s` (its wall in the suite), `main_wall_s` (the ledger's median, null where new), `cpu_s`, `main_cpu_s`, `main_from` (`base`, `ledger` or null), `reference_s`, `line_s`, `limit_s` and `over`. Empty without entries. `print_summary` prints a line for each row; its text is the code's.
- **Each row's runs,** beside those keys: `alone`, the figures of its entry's runs at the head in round order, null for a run without one; `main_alone`, the figures of the base's runs for it in round order where `main_from` is `base`, else an empty list, a base entry without `runs` giving the list of its one `cpu_s` (section 4); and `swinging`, true where its `test swings alone at the head` line was written. `print_summary` may show the runs on the row's line.
- **`tools/tiers/cpu-summary DIR ... [--base DIR]`** gives the directory as `base`; one-process takes no `--base`, since the base's run follows the head's.

## 7. Later

- **A test that fails alone** after passing in the suite is a warning, `test not measured alone`; whether it is red, a test that depends on an earlier one, is left until one is seen.
- **A cap on the tests run alone:** none. A change that adds or renames many tests runs each alone once, a process's start and the test, shared over the lanes; a run's wall bears it, and the gate's wall limit holds it as it holds the suite.
- **A flagged test within its limit by the ledger** runs once at the head (section 4). Where its base's highest figure then sets a lower limit than the ledger's median did, it is judged by that one figure; whether such a test should run again at the head, after the base, is left until a red so judged is seen. Holding every flagged test to its reference as it runs, the lowest limit any base can set, would close it, at the cost of running again every flagged test past its reference, which the draft's word, the ledger standing for the base, doesn't ask.

## 8. Open questions

None.
