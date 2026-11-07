# jig-world-cost-2: what every task holds to

The request is `jig/docs/plan/jig-world-cost-2/inputs/request.md`; its `Limits` and `For every task of this plan` bind every task. In short:

- **No story loses what it checks.** No budget, no build setting and no bound changes.
- **Only the focused and fuzzy suites run:** never `tools/tiers/run`, a browser, process or live tier, or a `<tier>-<tree>` profile, whatever a test's name suggests.
- **Nothing is measured.** No timer, no profile, no CPU figure, in a commit or out of one. Your acceptance is never a number of seconds, and your report claims none: the queue's record is read after landing.
- **A task that proves harder than it reads lands short.** Do what holds, say what doesn't in the report, and report `Done:`. The other tasks don't wait on its saving, only on its merge.
- **Decide, don't ask.** A small edit your task needs just outside its list (a re-export, a `mod` line, a design line) is made and named in the report. `Question:` is only for what truly blocks the task.
- **A helper your change leaves unused goes in the same task.**
- **Before you call a focused or fuzzy failure "outside the change",** run it once on the pristine base. If the base passes and your branch fails, it's yours.
- **A test moves between tiers only under `store-sweeps`' rule:** what it checks decides its place, never its cost.
- **"Every story ends with a cold restart" stays** (`jig/docs/design/domain/engine.md`, 15), and no story opts out.

## Stay out

- `Cargo.toml`'s profiles, `tools/tiers/budgets.toml`, `.config/nextest.toml`, how test binaries link.
- `skein/testing/skein-heap`, `skein/testing/skein-world/src/stream.rs`: just repaired.
- `jig/docs/development/debt/D034-world-replay-cost.md`'s row feed: not asked for.

## Running

A focused test, by name:

```
heavy cargo nextest run -p jig-conformance --lib -E 'test(=referee::deletions::tests::NAME)'
```

The fuzzy profile selects a `fuzzy::` test by name:

```
heavy cargo nextest run -p jig-store-world --profile fuzzy -E 'test(=fuzzy::patching::NAME)'
```

Check the count each time: nextest exits 4 on a run that selected nothing.
