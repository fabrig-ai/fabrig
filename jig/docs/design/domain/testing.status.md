# testing: status

2026-11-05. Where jig's code lags or departs from [testing.md](testing.md).

- **Not yet:** 4: the stories of `me`'s `resolved_default` (engine.md,
  11.1). 7: `ops`'s worlds on jig's store component; they run
  `jig-fake-store`.
- **Departs:** 3: the brief and notes worlds do not check the heap at each
  iteration as section 2 requires. Their suite entry points install the
  counting allocator and their fuzzy exercises use seeds, but
  `jig/tests/brief/src/world.rs` and `jig/tests/notes/src/world.rs` step
  without the heap check. No reason is recorded.
