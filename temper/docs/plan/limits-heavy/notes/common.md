# limits-heavy: the rules every task names

The request is `temper/docs/plan/limits-heavy/inputs/request.md`: read its table, *What's wanted* and *For every task of this plan*. They bind every task, and win over anything here that seems to differ. `temper/docs/plan/limits-heavy/plan.md`, *Belonging*, holds the cut's reading of where each row belongs.

## The order of a task

1. **Belonging** is read already, in `plan.md`. A task moves only what its spec says it moves.
2. **Profile before changing anything.** Quote the count before; name the cause from the profile, not from the test's name. The spec's *Starting points* are guesses to check, never findings.
3. **Fix the cause where the profile puts it,** largest first. A cause in the code under test is fixed there, with its own focused test and no change of behaviour: a person's box pays it too. A cause in the test is fixed in the test.
4. **Stop** when the next cause's gain is under about 0.2 CPU s across the tests that pay it. To read a gain: a cause's share of a test's inclusive instructions, times that test's CPU s in the request's table, summed over the tests that pay it.
5. **Write the figures,** below, then the usual end of your run.

## Measuring

Instruction counts on one test by name, through nextest, so the test finds its paths as it does in the gate. `heavy` runs any command. On x86-64:

```sh
mkdir -p target/limits-heavy
NEXTEST_DOUBLE_SPAWN=0 \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --tool=callgrind --callgrind-out-file=$PWD/target/limits-heavy/NAME.%p.cg" \
heavy cargo nextest run -p PKG --lib -E 'test(=FULL::TEST::NAME)'
callgrind_annotate --inclusive=yes target/limits-heavy/NAME.PID.cg | head -60
```

Use `--test BIN` in place of `--lib` for a test binary. Nextest lists tests through the runner too, so a run leaves several `.cg` files: the test's is the largest, and its `Collected` line is the test's count. A test that starts processes adds `--trace-children=yes` and sums its files, and says so.

- **A profile under callgrind runs about fifty times slower.** The default profile's `global-timeout` is 150 s. A test that reaches it runs with `--profile fuzzy --ignore-default-filter` and the same filter, whose timeout is longer. That changes no setting.
- **A test whose real-clock patience trips under callgrind** (a real box, say) is measured instead by the median of three `/usr/bin/time -f '%U %S'` runs of it alone through nextest, with nextest's own cost from a run that selects a trivial test in the same binary taken off. The figures say which it is, since CPU seconds on these machines swing by a third between runs.
- Before and after are the same command, alone, on the same builder, in the same kind of build. Test builds stay unoptimized.
- `target/limits-heavy/` is yours, and is gone before you push.

## The figures

Every task writes `temper/docs/plan/limits-heavy/figures/<task>.md`, and only its own, so that merges don't meet. `heavy-close` builds `plan.md`'s table and the report from them, so the form binds:

```markdown
# <task>: figures

| Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---:|---:|---|---|
| `package::binary::full::test::name` | 1,234,567,890 | 234,567,890 | The cause, from the profile | The fix, or "stays: <why>" |

## Found and not fixed

- The cause, its instructions and tests, and what fixing it would take. Write "None" if there are none.

## Tests that went

- `old::name`: held by `holder::name`, which passed at <commit>. Write "None" if there are none.
```

One row for each test you touched or profiled. A row with no profile says so, and gives its count as `unknown`.

## Departures from the implementer's text

- **You write under `temper/docs/plan/limits-heavy/figures/`,** your own file alone.
- **Design:** `docs/design/` stays closed unless your spec names the line and how it changes.

## What holds in every task

- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories, and no walk, render check or throwaway driver. A story in a slow tier is compiled (`--no-run`), not run.
- **No budget, timeout, deadline or build setting changes.** `tools/tiers/budgets.toml`, `.config/nextest.toml`, `tools/tiers/packages.toml` and every `[profile.*]` stay as they are. A cost no test can avoid goes in the figures, and stays.
- **No test leaves its suite** but the two that `box-lifetime` moves.
- **Every bound stays.** A limit test holds a bound the product ships, or a bound its design derives; the test still reaches it, whole, and asserts what it asserted. Make the building cheap, never the bound small. A test that checks every order, row or commit still checks every one: no sampling.
- **Nothing is lost.** A test that goes names its holder in your figures, and that holder exists and passes first.
- **Rows.** Section 8's *A test is a process* lets tests that share one fixture become rows of one test, where the profile shows the fixture is the cost and each check costs little beside it. Then each row names itself when it fails, and every assertion stays. A test past its guide isn't merged with another to make room (`temper/docs/design/domain/testing.md`, 6.1): if the cost is the check's own, it stays a test of its own.
- **A cause in skein, smith or jig** is fixed there if small and clearly theirs; otherwise it goes under *Found and not fixed*.
- **A failure you think is outside the change** runs once on the pristine base first. If the base passes and your branch fails, it's yours.
- **Decide, don't ask.** A stored format or protocol may change freely, with no compatibility. A small edit just outside your files is made and named in your report. A `Question:` is only for what truly blocks the task.
- **Old forms and citations:** `python3 -I -B temper/scripts/old_forms.py` as the implementer's text says, and `python3 -I -B tools/citations/citations.py --base origin/main` if you touch a Markdown file or move a file that design cites.

## Beside this plan

Other drafts are working here. Keep hunks small, don't reformat, and don't move code you don't need to change. If a merge meets another draft, keep their change, and say so in your report.

- `limits-fixtures` is repairing what the small tests build first, in `temper-engine-domain-world`'s `forge_routes` and `root_routes`, `temper-service`'s `setup` and `tests::stock`, `temper-client-world`, `temper-client-view` and `temper::suite`'s `startup` and `forge`. A fixture they share with your row may be theirs: fix what's yours (your test's own loop, a per-case fixture) and leave a shared fixture's general cost under *Found and not fixed*, unless the profile puts more than 0.2 CPU s of your row on it.
- `limits-worlds` is repairing the real loop and the worlds on it. It shares an engine, a store, a fake forge and a fake LLM with some of your rows.
- `look-round-13` and `look-round-11` hold `temper/tools/gallery/**`, `temper/crates/temper-client-view/src/forge/**` and much of `temper-client-view/src/pages/**`.
- `model-catalogue-2` holds `temper/crates/temper/tests/support/**`, `temper/crates/temper-init/template/*`, `temper/crates/temper-fake/src/**`, `temper/crates/temper-service/src/startup*` and `temper-client-tools/src/jig/**`.
- `ui-model-next-turn` and `goal-delivery` hold parts of `jig-domain-tasks` and `temper-engine-domain-forge`.
