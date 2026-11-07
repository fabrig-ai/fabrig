+++
id = "world-helpers"
part = "world-helpers"
kind = "feature"
needs = ["counting", "wire-trace"]
files = [
  "skein/testing/**",
  "skein/docs/development/world-costs.md",
  "skein/docs/plan/skein-world-costs/figures/world-helpers.md",
]
effort = "medium"
minutes = 90
+++

# The same pattern elsewhere in skein's world helpers, and the account

## Goal

With `Counting` and `Wire` done, profile one median test each of `skein-forge-machine-world`, `skein-llm-world` and `skein-tls-world`. Fix what a profile shows in skein's world helpers, largest first, and stop when the next cause's gain is under about 1% of a test. Then write the account the request asks for: before and after, in instruction counts, with each cause and whether it's fixed. No fix may be the right result here: then only the account lands.

Acceptance:

- `figures/world-helpers.md` holds the three tests' readings on your base, each cause you found with its share and whether it's in scope, and a reading after each fix.
- No cause in scope is left with a gain of about 1% of its test or more, or the account says why it stayed.
- `skein/docs/development/world-costs.md` holds the account below, and the six tests' final readings, made on your tip.
- Each helper you changed keeps what it did: the same `Rng` draws in the same order, the same text where a failure prints it, the same counts. No assertion weakens, and no test leaves its suite.
- Any new test bites.
- `heavy cargo nextest run -p fabrig-boundary --test outward` passes, since the account is outside the plan's directory, and `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing new.

## Inputs

- `skein/docs/plan/skein-world-costs/inputs/request.md`, items 3 and 4.
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1.
- `skein/docs/plan/skein-world-costs/figures/counting.md`: the six tests' names and their readings on the pristine base.
- `skein/docs/plan/skein-world-costs/figures/wire-trace.md`.
- `skein/docs/plan/skein-world-costs/notes/common.md`, `skein/docs/plan/skein-world-costs/notes/measuring.md`.

## Owns and may change

Owns the helpers under `skein/testing/`, the account and the figures.

**What's in scope.** A cause is this task's if its code is in a crate under `skein/testing/`. Anything else gets a line with its share, and stays as it is:

| Where the cause is | Whose it is |
|---|---|
| `skein/tests/<world>/`: a test's own fixture | another plan's |
| `skein/crates/*`: the code under test | not this plan's; `skein-json`'s string paths are another plan's |
| A process's start, before a test begins | another plan's |
| What `Counting` and `Wire` still cost | the first two tasks'; the account gives its share |

**Which pattern to look for.** The first two tasks' pattern: a line formatted on a path that passes, a thread-local reached many times per call, a check of what's already known, a byte at a time where a slice would do. Take what the profile shows, not this list.

**Readers.** Before you change a public item of a helper, `rg` its readers in every tree. You may change them in any tree, each only in a type path or a method's name, as `wire-trace` did: a form that keeps the old method names, so that call sites stay, is the default. Name each in your report.

Options, each decided:

- **A fix whose readers need more than that:** leave it, and give it a line in the account with its share and what it would take.
- **A cause on the line,** at about 1%: fix it if it's a few lines; otherwise a line in the account.
- **Fixing a cause moves others' shares:** take the next cause from a new reading, not from the first.
- **A median test that spends nearly all its time in the code under test:** that's a finding. Say so, and read it no further.
- **A fix in `skein/testing/skein-world/src/stream.rs` or `skein/testing/skein-heap/src/lib.rs`:** yours, if a new reading shows one. It's the same rule, and `skein/tests/heap`'s tests still don't change.

## Steps

### 1. The readings

Read the three tests on your base, as `notes/measuring.md` says, with the names `figures/counting.md` gives. List each test's causes, largest first, by inclusive count, in `figures/world-helpers.md`, with whose each one is.

### 2. Largest first

One cause, one commit, which says what the profile showed. Read again after each, and write it down. Stop by the rule above. After each fix: the narrowest test that holds it, then the focused tests of the helper's own package.

### 3. The account

`skein/docs/development/world-costs.md`. Read the six tests on your tip first. The before column is `figures/counting.md`'s before arm. It holds, in this order:

```markdown
# What every world pays to count its heap and keep its wire's trace

<date -u>, base <short sha>, after <short sha>. Instruction counts under callgrind, each test by its name, unoptimized as the suites run (`skein/docs/foundation/testing-strategy.md`, 8).

## The tests

| Package | Test | Profile | Before | After | Change |
|---|---|---|---:|---:|---:|

## The causes

| Cause | Where | Test | Share before | Share after | Fixed |
|---|---|---|---:|---:|---|

## What the code can't reach

- What `Counting` still costs, by cause and share. A `[profile.dev.package.skein-heap]` override would reach it: the user's to sign off under `skein/docs/foundation/testing-strategy.md`, 8, with these numbers. None is added.
- Each cause in a test's own fixture or in the code under test, with its share and its test.
```

Every test and every cause you found has a row or a line, fixed or not, and so do the request's two: `Counting`'s `alloc` and `dealloc`, and `alloc::fmt::format` from `Wire::answer` and `Wire::take`. The figures carry the detail and retire with the plan, so the account holds every number a reader needs. `tests/boundary/tests/outward.rs` reads it: it names no machine, tool or session of whoever runs this development, and a command in it is plain `cargo` or `valgrind`.

### 4. Checks

Once:

- `cargo fmt`
- `python3 -I -B skein/scripts/old_forms.py`, and the same for each other tree you touched
- `heavy cargo clippy -p PKG ... --all-targets -- -D warnings`, naming each package you changed
- `heavy cargo nextest run -p skein-forge-machine-world -p skein-llm-world -p skein-tls-world`, with each package you changed, and the focused tests of each direct dependent of what you changed, up to the ten nearest; the trunk's end check runs the rest (`notes/common.md`)
- A change of meaning in a helper, such as a value it returns or an order it keeps, runs the tests of every package that reads it.
- For a public item whose signature changed, `heavy cargo check --tests`, naming each package that reads it.
- `heavy cargo nextest run -p fabrig-boundary --test outward`
- `python3 -I -B tools/citations/citations.py --base origin/main`

Delete `target/world-costs/`, commit and push.

## Report

The six tests' totals before and after. Each cause with its share, and what became of it. What's left that the code can't reach, with its share, for the user's sign-off. The three tests you read, and any that spent its time in the code under test. Each reader you changed.
