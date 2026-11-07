# jig-world-cost: what every task holds to

The request is `jig/docs/plan/jig-world-cost/inputs/request.md`; its `Limits` and `For every task of this plan` bind every task. In short:

- **No story loses what it checks.** No budget, no build setting and no bound changes.
- **Only the focused and fuzzy suites run:** never `tools/tiers/run`, a browser, process or live tier, or a `<tier>-<tree>` profile, whatever a test's name suggests.
- **Nothing is measured.** No timer, no profile, no CPU figure, in a commit or out of one. Your acceptance is never a number of seconds, and your report claims none: the queue's record is read after landing.
- **An item that proves harder than it reads lands short.** Do what holds, say what doesn't in the report, and report `Done:`. The other tasks don't wait for this one.
- **Decide, don't ask.** A small edit your task needs just outside its list (a re-export, a `mod` line, a design line) is made and named in the report. `Question:` is only for what truly blocks the task.
- **A helper your change leaves unused goes in the same task.**
- **Before you call a focused or fuzzy failure "outside the change",** run it once on the pristine base. If the base passes and your branch fails, it's yours.
- **A test moves between tiers only under its task's rule:** what it checks decides its place, never its cost.

## Stay out

Another plan's, or not to be changed by any plan of this kind:

- `skein/testing/skein-heap`, `skein/testing/skein-world/src/stream.rs`
- `jig/tests/ops/src/joint`, `jig/testing/jig-test-service`, `jig/testing/jig-client-kit/src/joint`
- `Cargo.toml`'s profiles, `tools/tiers/budgets.toml`, `.config/nextest.toml`

## Running

The fuzzy profile selects a `fuzzy::` test by name:

```
heavy cargo nextest run -p jig-core-world --profile fuzzy -E 'test(=fuzzy::faults::NAME)'
```

Check the count each time: nextest exits 4 on a run that selected nothing.
