# testing: status

2026-11-06. Where jig's code lags or departs from [testing.md](testing.md).

- **Not yet:** 4: the stories of `me`'s `resolved_default` (engine.md,
  11.1). 7: `ops`'s worlds on jig's store component; they run
  `jig-fake-store`.
  7: the `debt-24` request of 2026-11-05 has each world name its parties,
  projects, tasks and connectors once, a bare number left only where it's the
  point, at most ten by the search in
  `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`: it reads 37, across
  nine packages, none yet explained. `PROJECT` stands in three files of
  `jig/tests/people`, two of `jig-client-scripted-engine` and a local of
  `jig/tests/core/tests/suite/stories.rs`. `ask` is one constructor in
  `jig-core-kit`, where that plan's story has one definition per world: two core
  files keep a wrapper that returns the reply, and the conformance world two, a
  queue and a drain.
- **Departs:** 3: the brief and notes worlds do not check the heap at each
  iteration as section 2 requires. Their suite entry points install the
  counting allocator and their fuzzy exercises use seeds, but
  `jig/tests/brief/src/world.rs` and `jig/tests/notes/src/world.rs` step
  without the heap check. No reason is recorded.
