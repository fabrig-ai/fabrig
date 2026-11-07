# skein-limits-rows: how a fold is done

Binding for every fold task, beside `notes/common.md`. The rules are the request's (*A fold is honest or it isn't done*) and `skein/docs/foundation/testing-strategy.md`, 8 (*A test is a process*) and 10 (*One behaviour per test*, *Rows carry data*, *A bound's two sides*). What a fold saves is the binary's floor per test folded, not the test's own work.

## The kit

`skein-rows` (`skein/testing/skein-rows`, `skein/docs/design/testing.md`, 6), made by `rows-codec`:

```rust
#[track_caller]
pub fn check<R>(rows: impl IntoIterator<Item = (&'static str, R)>, body: impl Fn(R))
```

It runs `body` on every row, then panics at its caller's line naming each row that panicked, with its message. It has no dependencies, so a binary's floor doesn't grow. A crate takes it as a `[dev-dependencies]` line, `skein-rows.workspace = true`, which every fold task may add to the crates it owns.

## A table

```rust
#[test]
fn closing_a_ready_phase_discards_its_private_work() {
    skein_rows::check(
        [
            ("close_during_ready_full_discards_private_work", Stage::Full),
            ("close_during_ready_contents_discards_private_work", Stage::Contents),
        ],
        close_at,
    );
}
```

- **The table's name** says, in about eight words, what all its rows show.
- **A row's name is the old test's name, verbatim,** so a search for it still finds it. A row taken from an old test's inner case is `old_name: its label`.
- **A row carries data:** its inputs and its expected values, as a tuple, or a small `struct` beside the table when it has more than three fields, as the file's own tables do. A literal or named size, never computed by the code under test.
- **One body checks every row alike.** It never reads a row's name, and never branches on what kind of row it is. A row whose checks differ from its neighbours' isn't in that table. Passing a closure or a function as a row is the same thing as a branch, and is out.
- **A row's setup stays a row's.** A fixture a test built for itself is built in the body, per row. A `const` or immutable input may be shared where sharing it can't change what's shown.
- **A bound's two sides** may be two rows of one table, each with its expected value.

## What stays apart

- Cases that show different behaviours, whatever they cost.
- A test whose checks are not the same as its neighbours': an extra assertion on one row means it stays a test, unless every row can hold that assertion truly.
- A test that measures its process: heap ceilings and meters (`skein_heap`, `CallHeap`, `component_heap`), or anything reading process-wide or thread-local state an earlier row would disturb.
- `#[should_panic]`, a test that starts a thread or a process, and a test reading files that another row's run could change.
- Anything under `fuzzy` or named `fuzzy_*`. No test changes suite.

Tables already in these files stay as they are, though they stop at their first failure, unless a fold takes them in. Rewriting them saves no floor.

## The lists are the survey's reading

Each task's spec lists the groups a read-only survey found, with line ranges at commit `92d04ee863`. They may have moved. **A list is a reading of the code, not a mandate.** Read each group before you fold it:

- A listed group that fails a rule above stays apart, and gets a line under *Left apart*. Some will: a survey reads quickly, and an assertion only one row has is easily missed.
- A group the survey left apart that does pass every rule folds too. Check the groups around a listed one: a file with one fold often has another.
- Largest gain first: the binaries as the spec orders them, the largest tables first in each. Stop when no honest group is left. The request's estimate of about 1,000 tests isn't a target to reach by loosening a rule.

## A table stays within its guide

Section 8's per-test guides: 40 ms for a unit test, 0.2 s for an in-memory world test, 0.6 s for a joint world test, 1 s for a focused world test. Read a table's time from its `PASS` line (`--status-level pass`). If a table passes its guide, make it two. A table with RS256 signing in its rows gets a callgrind count of one row first, so you know what it will cost before you run it.

## Nothing is lost

- Every assertion stays, as a row's or a test's. No assertion weakens, and no row drops a check to fit the body.
- **Every test name your branch removes appears verbatim as a row's name.** Check it with a throwaway script over `git diff`, untracked and deleted before the push. Give the count in your report: names removed, names found as rows, and the two are equal.
- **A fold bites.** The implementer's text lets a move skip its bite, but here one table per file you changed gets one: change one row's expected value, see the table fail naming that row, see a later row's failure named in the same run if you break two, and restore. The report names the tables you broke.
- Before removing a name, `rg` it over `.config/`, `tools/` and `**/docs/**`. A name in `docs/development/` is a dated record and stays, since the row keeps the name. A name anywhere else is a real reader: update it, and say so in your report.

## What goes in `plan.md`

Your own subsection of *Before and after*, in this form. Counts come from `heavy cargo nextest list -p PKG`, before and after.

```markdown
### <task>

| Binary | Module | Tests before | Tests after | Tables | Rows | Floor saved, ms |
|---|---|---:|---:|---:|---:|---:|

Left apart:

- `module`, N tests: why, in one line.
```

Floor saved is (tests before less tests after) times the binary's floor in the request's table. A module with nothing folded still gets its row, and a line under *Left apart*; one line may cover a whole file or several files with one reason.

## Departures from the implementer's text

- **Test-only changes run less.** A branch that changes only tests and a `[dev-dependencies]` line runs the focused tests of the packages it changed and not their dependents, since no reader's code changed.
- **A fold's bite** is as above, rather than the implementer's text's skip for a move.
