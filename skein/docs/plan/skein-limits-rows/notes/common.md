# skein-limits-rows: the rules every task names

The request is `skein/docs/plan/skein-limits-rows/inputs/request.md`: read *What's wanted* and *For every task of this plan*. They bind every task, and win over anything here that seems to differ.

## Measuring

Instruction counts on one test by name, through nextest, so the test finds its paths as it does in the gate. `heavy` runs any command. On x86-64:

```sh
mkdir -p target/skein-limits-rows
NEXTEST_DOUBLE_SPAWN=0 \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --tool=callgrind --callgrind-out-file=$PWD/target/skein-limits-rows/NAME.%p.cg" \
heavy cargo nextest run -p PKG --lib -E 'test(=FULL::TEST::NAME)'
callgrind_annotate --inclusive=yes target/skein-limits-rows/NAME.PID.cg | head -60
```

Use `--test BIN` in place of `--lib` for a test binary. Nextest lists tests through the runner too, so a run leaves several `.cg` files: the test's is the largest, and its `Collected` line is the test's count.

- **A profile under callgrind runs about fifty times slower.** The default profile's `global-timeout` is 150 s. A test that reaches it runs with `--profile fuzzy --ignore-default-filter` and the same filter, whose timeout is longer. That changes no setting.
- **If valgrind isn't on the builder,** that's a `Question:` and not a reason to estimate: the figures are in instructions.
- Before and after are the same command, alone, on the same builder, in the same kind of build. Test builds stay unoptimized.
- `target/skein-limits-rows/` is yours, and is gone before you push.

## Your figures go in `plan.md`

The request asks for before and after in `skein/docs/plan/skein-limits-rows/plan.md`. Each task fills its own subsection of *Before and after*, named for the task, and no other part of that file, so that merges don't meet. Quote no suite total: the gate reads that after landing.

## Departures from the implementer's text

- **You write `plan.md`,** your own subsection alone.
- **Design:** `docs/design/` stays closed but for the lines your spec names.

## What holds in every task

- **Only focused and fuzzy run.** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories, and no walk, render check or throwaway driver. A story in a slow tier is compiled (`--no-run`), not run.
- **No budget, timeout, deadline or build setting changes.** `tools/tiers/budgets.toml`, `.config/nextest.toml`, `tools/tiers/packages.toml` and every `[profile.*]` stay as they are.
- **No test changes suite.**
- **Nothing is lost.** Every assertion stays, as a row or as a test. No assertion weakens.
- **Stay out of** `skein-ui`, `skein-ui-web`, `skein-browser`, `skein-fake-person-browser` and their tests, which other plans are changing.
- **A failure you think is outside the change** runs once on the pristine base first. If the base passes and your branch fails, it's yours.
- **Decide, don't ask.** A small edit just outside your files is made and named in your report. A `Question:` is only for what truly blocks the task.
- **Old forms and citations:** `python3 -I -B skein/scripts/old_forms.py` as the implementer's text says, and `python3 -I -B tools/citations/citations.py --base origin/main` if you touch a Markdown file or move a file that design cites.

## Beside this plan

Other plans add tests to these crates all the time. Keep hunks small, don't reformat, and don't move code you don't need to change. If a merge meets another draft, keep their change, and say so in your report.
