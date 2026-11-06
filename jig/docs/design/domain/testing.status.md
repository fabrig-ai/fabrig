# testing: status

2026-11-06. Where jig's code lags or departs from [testing.md](testing.md).

- **Not yet:** 4: the stories of `me`'s `resolved_default` (engine.md,
  11.1). 7: `ops`'s worlds on jig's store component; they run
  `jig-fake-store`.
- **Departs:** 5: `jig_conformance::core` answers the core's accounts and
  builds its store view and snapshot, but an adapter still writes the core's
  arms in `deliver`, `poll`, `inbound`, `cold` and `observed` among its own,
  because decoding reads `jig-protocol-charter`, which links smith, and the
  first period and `Drained` differ between adopters in events the referee
  sees.
- **Departs:** 3: the brief and notes worlds do not check the heap at each
  iteration as section 2 requires. Their suite entry points install the
  counting allocator and their fuzzy exercises use seeds, but
  `jig/tests/brief/src/world.rs` and `jig/tests/notes/src/world.rs` step
  without the heap check. No reason is recorded.
