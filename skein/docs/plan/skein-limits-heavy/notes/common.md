# skein-limits-heavy: the rules every task names

The request is `skein/docs/plan/skein-limits-heavy/inputs/request.md`: read its table, *What's wanted* and *For every task of this plan*. They bind every task, and win over anything here that seems to differ. `skein/docs/plan/skein-limits-heavy/plan.md`, *Belonging*, holds the cut's reading of where each row belongs.

## The order of a task

1. **Belonging** is read already, in `plan.md`. A task moves only what its spec says it moves.
2. **Profile before changing anything.** For each row, the costliest test and a median one. Quote the count before; name the cause from the profile, not from the test's name. The spec's *Starting points* are guesses to check, never findings.
3. **Fix the cause where the profile puts it,** largest first. A cause in the code under test is fixed there if it's small and no behaviour changes, with its own focused test: a person pays that one too, so prefer it. A cause in the test is fixed in the test.
4. **Stop** when the next cause's gain is under about 0.05 CPU s across the tests that pay it. To read a gain: a cause's share of a test's inclusive instructions, times that test's CPU s in the request's table, summed over the tests that pay it.
5. **Write the figures,** below, then the usual end of your run.

## Measuring

Instruction counts on one test by name, through nextest, so the test finds its paths as it does in the gate. `heavy` runs any command. On x86-64:

```sh
mkdir -p target/skein-limits-heavy
NEXTEST_DOUBLE_SPAWN=0 \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --tool=callgrind --callgrind-out-file=$PWD/target/skein-limits-heavy/NAME.%p.cg" \
heavy cargo nextest run -p PKG --lib -E 'test(=FULL::TEST::NAME)'
callgrind_annotate --inclusive=yes target/skein-limits-heavy/NAME.PID.cg | head -60
```

Use `--test BIN` in place of `--lib` for a test binary. Nextest lists tests through the runner too, so a run leaves several `.cg` files: the test's is the largest, and its `Collected` line is the test's count. A test that starts processes adds `--trace-children=yes` and sums its files, and says so.

- **A profile under callgrind runs about fifty times slower.** The default profile's `global-timeout` is 150 s. A test that reaches it runs with `--profile fuzzy --ignore-default-filter` and the same filter, whose timeout is longer. That changes no setting.
- **If valgrind isn't on the builder,** that's a `Question:` and not a reason to estimate: the figures are in instructions.
- Before and after are the same command, alone, on the same builder, in the same kind of build. Test builds stay unoptimized.
- `target/skein-limits-heavy/` is yours, and is gone before you push.

## The figures

Every task writes `skein/docs/plan/skein-limits-heavy/figures/<task>.md`, and only its own, so that merges don't meet. `limits-close` builds `plan.md`'s table and the report from them, so the form binds:

```markdown
# <task>: figures

| Row | Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---|---:|---:|---|---|
| `binary` `module` | `full::test::name` | 1,234,567,890 | 234,567,890 | The cause, from the profile | The fix, or "stays: <why>" |

## Found and not fixed

- The cause, its instructions and tests, and what fixing it would take. Write "None" if there are none.

## Tests that went

- `old::name`: held by `holder::name`, which passed at <commit>. Write "None" if there are none.

## Counts

- `test::name`: the count before and after, and what the test asserts that shows the smaller one still does the same thing. Write "None" if there are none.
```

One row for each test you touched or profiled. A row with no profile says so, and gives its count as `unknown`.

## Departures from the implementer's text

- **You write under `skein/docs/plan/skein-limits-heavy/figures/`,** your own file alone.
- **Design:** `docs/design/` stays closed unless your spec names the line and how it changes.
- **A smaller count bites too.** The implementer's text asks a new test to bite. Here a test whose count falls is treated as new: break what it holds once by hand, see it fail with the smaller count, and restore. The figures' *Counts* line names what you broke and what you saw. Effort is medium; this step isn't optional.

## What holds in every task

- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories, and no walk, render check or throwaway driver. A story in a slow tier is compiled (`--no-run`), not run.
- **No budget, timeout, deadline or build setting changes.** `tools/tiers/budgets.toml`, `.config/nextest.toml`, `tools/tiers/packages.toml` and every `[profile.*]` stay as they are. A cost no test can avoid goes in the figures, and stays.
- **Every assertion stays as strict as it was.** A count falls only where the spec says, or where the *Counts* line shows by what the test asserts that the smaller one does the same thing: a ring still wraps, a ceiling's structure is still far larger than the limit it's held to.
- **No test leaves its suite** but those `plan.md`'s *Belonging* moves, and a moved test keeps every assertion.
- **Nothing is lost.** A test that goes names its holder in your figures, and that holder exists and passes first.
- **Rows.** Section 8's *A test is a process* lets tests that share one fixture become rows of one test, where the profile shows the fixture is the cost and each check costs little beside it. Then each row names itself when it fails, and every assertion stays. A test past its guide isn't merged with another to make room: if the cost is the check's own, it stays a test of its own. Rows of table tests are otherwise another plan's.
- **A helper's change stays in one task.** A helper several tests share is changed by the task that names it, whole, and no other.
- **A cause in a shared crate** is fixed there if small and clearly its own; otherwise it goes under *Found and not fixed*, with what fixing it would take. A branch that changes what a shared item means, not only its signature, also runs the tests of every package that reads it.
- **A failure you think is outside the change** runs once on the pristine base first. If the base passes and your branch fails, it's yours.
- **Decide, don't ask.** A small edit just outside your files is made and named in your report. A `Question:` is only for what truly blocks the task.
- **Old forms and citations:** `python3 -I -B skein/scripts/old_forms.py` as the implementer's text says, and `python3 -I -B tools/citations/citations.py --base origin/main` if you touch a Markdown file or move a file that design cites.

## Beside this plan

Other drafts are working here, and add tests to these crates all the time. Keep hunks small, don't reformat, and don't move code you don't need to change. If a merge meets another draft, keep their change, and say so in your report.

- **Leave alone, and report a cost in them under *Found and not fixed*:** `skein_heap::Counting` and `skein_world::stream::Wire` (`skein-world-costs`); `skein-json`'s product code (`json-strings`); what a process costs before its test starts (other plans).
- `limits-fixtures/view-words` holds `skein/crates/skein-ui/src/phrases.rs` and `skein/crates/skein-ui/src/tests/phrases.rs`.
- `limits-worlds/cuts-fork` holds `skein/testing/skein-fork/**`, and `limits-worlds/engine-causes` may change any of `skein/crates/**` for a cause its profile finds.
