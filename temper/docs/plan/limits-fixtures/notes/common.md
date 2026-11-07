# limits-fixtures: rules every task names

The request is `temper/docs/plan/limits-fixtures/inputs/request.md`; its `For every task of this plan` binds every task, and this note is how to meet it. The rule is `skein/docs/foundation/testing-strategy.md`, 8 ("A test is a process", "What a test costs the suite", "A slow test is understood, never hidden") and 8.1, second question.

What each spec says about causes comes from reading the code on 2026-11-06. **No test ran for this cut, so none of it is a finding.** The profile decides. If it puts the cost elsewhere, follow the profile and say so in your report.

## What doesn't move

- **No build setting changes:** no `opt-level`, no `[profile.dev.package.*]` line, no `RUSTFLAGS`, linker or nextest setting. `.config/nextest.toml` and `tools/tiers/budgets.toml` don't change. No budget, timeout or deadline changes, and a test's product timeouts keep their values.
- **Every test stays in its suite and checks what it checked.** No test goes, none moves, no assertion weakens, no case or seed falls. Rows of table tests are another plan's. A test's body changes only where its own setup is the cause.
- **A fixture's value doesn't change.** A cheaper fixture gives its tests the same limits, the same rows and the same bytes. A golden, a recorded digest or an expected value that would have to change is a finding: read it, and take the repair out or say why.
- **The product behaves the same.** A product fix is welcome where the profile puts the cause, at the size the cause needs, with its own focused test: the same answers, bytes and refusals. A stored format or protocol may change freely if that's the cause, with no compatibility.
- **Fixture names and signatures stay.** Other plans add tests that call these fixtures all the time. A public fixture keeps its name and signature. If the cause needs another form, add it beside the old one and move callers only where they save something.
- **Move no code you needn't.** These fixture files are also changed by other plans (`temper/docs/plan/limits-fixtures/plan.md`, "Overlaps"), so keep the diff to what saves instructions: no tidying, no reordering.

## A process-wide cache saves little here

nextest runs each test in its own process, so a fixture built once per process is built once per test (strategy, 8). A `OnceLock` helps only a test that builds the same thing two or more times. It can't help a test that builds it once. So look for these, in this order:

1. **Built again in one test:** a fixture that calls a fixture that calls the first one again. Pass the value down, or build it once.
2. **Dearer than it needs to be:** a scan where a lookup would do, data copied or encoded again, a table sized past its use, work done for a case this test doesn't reach.
3. **A value that could be a constant,** with a test that holds the constant to what it replaces.

A `OnceLock` plus a deep `clone` per call is often no saving: check the clone's cost in the profile before you rely on it.

## How to profile

Instruction counts (`Ir`) are the figure this plan reports, since CPU seconds on these machines swing by a third between runs. Nextest's tracer runs one test with the environment `skein_scratch::paths` reads, without nextest's timeouts. The output path is absolute, because a test runs in its package's directory (`mkdir -p target/lf/cg` first):

```text
NEXTEST_DOUBLE_SPAWN=0 TEMPER_BUNDLE=none heavy cargo nextest run --profile default \
  -p PKG --test suite \
  --tracer "valgrind --tool=callgrind --callgrind-out-file=$PWD/target/lf/cg/NAME.%p.out" \
  -E 'test(=TEST_NAME)'
```

- Use `--lib` in place of `--test suite` for a library's tests (`temper-service`, `temper-client-view`).
- A test that starts processes also takes `--trace-children=yes`. Each `%p` file is one process, the first line of its `cmd:` names which, and the test's figure is their sum.
- One test at a time. Callgrind runs some fifty times slower.
- `callgrind_annotate --auto=no --threshold=99 --inclusive=yes FILE | head -60`, then `--inclusive=no`. `PROGRAM TOTALS` is the test's `Ir`. Skim: `head -60` is enough to see the shape.
- **Choosing the tests:** a focused run's times choose them, and aren't quoted: `TEMPER_BUNDLE=none heavy cargo nextest run -p PKG --test suite -E 'test(/^MODULE::/)' --status-level pass --final-status-level none 2>&1 | grep PASS | sort -t'[' -k2`. The first line is the module's cheapest test (its floor), and the middle line a median one.
- **What can't be measured is `unknown`,** with why: never a guess. A test that fails under callgrind because of a harness deadline or a syscall valgrind doesn't know isn't a finding about the test. Note it and take the next test along.
- **If `valgrind` isn't on the machine:** first line `Question:`, naming the machine. Nothing gets installed, and no other profiler stands in.
- `callgrind_annotate` is the only reader you need. Callgrind's output stays under `target/lf/`, and nothing from it is committed.

A floor can sit in the loader rather than in the test: relocating a large binary shows up as `_dl_relocate_object` and friends. No test can avoid that and no build setting may change, so it goes in your figures as a cause not fixed, with its share.

## When to stop

Largest cause first. **Stop when the next cause's gain is under about 1% of the binary** (the `CPU s` in the request's table, all rows of that binary): estimate it as the cause's share of `Ir`, times the CPU of the tests it reaches. What's left goes in your figures.

## Your figures

Each task writes `temper/docs/plan/limits-fixtures/figures/<your task>.md`, which it alone owns. `temper/docs/plan/limits-fixtures/tasks/close-figures.md` gathers them at the end, so a test you name must keep its name.

```markdown
# <task>: figures

Base <commit>, machine <hostname through heavy>, <date -u>.

| Row | Test | Floor or median | Ir before | Ir after | What was built before the first assertion |
|---|---|---|---:|---:|---|

## Causes

| Cause | Where | Ir per test, on which test | Fixed in | Or what fixing it would take |
|---|---|---:|---|---|
```

- **Before** is measured at your branch's start, before you change anything; **after**, at your last commit. Both use the same test by name.
- **A row of the request's table** that your task names gets a floor and a median, as your spec lists them.
- **A cause found and not fixed** gets a line, with what fixing it would take.
- **Each commit says what it saved:** the test's `Ir` before and after.

## The end of every task

Your implementer's text holds, and:

- From the root, `python3 -I -B tools/citations/citations.py --base origin/main` lists nothing of yours.
- `git status` shows no leftover: `target/lf/` is ignored, and no script stays.
- A change of what a shared item means runs its readers' tests: a product change runs the focused tests of every package that reads it, in every tree, as your spec names them.
