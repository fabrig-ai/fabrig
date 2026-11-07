# testing: status

2026-11-07. Where jig's code lags or departs from [testing.md](testing.md).

- **Not yet:** 5: the `debt-25` request of 2026-11-06 has the core world's observer
  update from each commit's rows
  (`jig/docs/development/debt/D034-world-replay-cost.md`), but
  `jig_conformance::core`'s `durable_at` rebuilds its snapshot whole at each
  new commit, because a feed of rows alone would keep that rebuild and the
  referee's walk of every task, and that plan's cut left it out, by judgement,
  with its share of a suite's CPU measured for that plan's owner.
  7: `ops`'s worlds on jig's store component; they run `jig-fake-store`.
  7: the `debt-24` request of 2026-11-05 has each world name its parties,
  projects, tasks and connectors once, a bare number left only where it's the
  point, at most ten by the search in
  `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`: it reads 2, both
  explained there, but four files name a fixture of their world or kit again,
  `jig/tests/core/tests/suite/stories.rs`'s `PROJECT`,
  `jig/tests/accounts/tests/suite/terms.rs`'s `PARTY`,
  `jig-client-kit`'s `stories/shared/b_held.rs`'s `OWNER` and
  `jig/tests/engine-clients/tests/requests/full.rs`'s `PARTY`, so the item stays
  open, where the `debt-25` plan has it go, because they span more than the one
  file its last task could correct. `ask` is one constructor in `jig-core-kit`,
  where the `debt-24` plan's story has one definition per world: two core files
  keep a wrapper that returns the reply, and the conformance world two, a queue
  and a drain, because the `debt-25` plan keeps different verbs under one name.
  7: the `debt-25` request has its three jig items end with fewer test lines,
  and together they fell 181, but D035's own tasks rose 34, where that plan's
  story has about 200 down: the client's sweep rose 158 and the people's 61, and
  the core's fell 139; no reason is recorded.
- **Departs:** 5: `jig_conformance::core` answers the core's accounts and
  builds its store view and snapshot, but an adapter still writes the core's
  arms in `deliver`, `poll`, `inbound`, `cold` and `observed` among its own
  (`jig/docs/development/debt/D030-conformance-not-drop-in.md`), because
  decoding reads `jig-protocol-charter`, which links smith, and the first period
  and `Drained` differ between adopters in events the referee sees.
- **Departs:** 3: the brief and notes worlds do not check the heap at each
  iteration as section 2 requires. Their suite entry points install the
  counting allocator and their fuzzy exercises use seeds, but
  `jig/tests/brief/src/world.rs` and `jig/tests/notes/src/world.rs` step
  without the heap check. No reason is recorded.
