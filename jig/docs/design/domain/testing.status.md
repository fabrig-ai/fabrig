# testing: status

2026-11-14. Where jig's code lags or departs from [testing.md](testing.md).

Section 5's keyed durable-row evidence and exhaustive forked cuts are
present. Section 7's core row names retain typed keys until a failure
prints them; shared setup avoids the repeated parse, unused view and
whole-image copy, duplicate server limits, rebuilt rules domain and second
restart-configuration copy. Section 9's seven store sweeps run in fuzzy,
sharing their checks with the chosen focused cuts. Complete snapshots,
startup and cold readback remain the contract; section 10 holds further
observation and world-cost work.

- **Not yet:** 7: `ops`'s worlds on jig's store component; they run `jig-fake-store`.
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
  9: the focused suite's fit within 30 CPU s is unproved. The
  `jig-world-cost` and `jig-world-cost-2` requests leave further cost repairs;
  their whole-suite effect, and fuzzy's fit with the moved core and store
  sweeps and `Standard` drive, await the queue's after-landing record. The
  per-test shares in the latter request do not establish a saving or a
  whole-suite total. No budget or build setting changes are part of those
  repairs.
- **Departs:** 9: the focused stray-cleanup story selects its first cut
  with `Change::Make`, not the first `Change::Remove` the design names
  (`jig/tests/store/tests/suite/generations.rs`,
  `strays_cut_at_their_first_and_last_removal_are_gone_at_the_next_start`).
  Its second cut is the last removal, and fuzzy retains every cut with the
  same recovery checks. No reason for the first cut's substitution is
  recorded; the named removal remains required.
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

- **Not yet:** 9: core-world stories that opt out of the restart
  oracle with a named finding (`broken()` in `jig/tests/core`), most for
  a claim the start settles (`engine/README.md`, 6, step 3) changing a task's `chip`, `acts`
  or run `settings`, which the projection keeps durable, others for a
  frame's `needs` with a person proposal left pending and
  `Deployment.messages` advancing on a retried deadline; the core
  world's `world` and `direct` drivers run no oracle.

- **Not yet:**
  7: the step crates' case tables and the two named-people tables run
  each case under `catch_unwind` and collect the failing ones.
  [in flight: change 2853]
  7: the fuzzy sweeps, the core's scenarios and bundle of cases, the
  tasks kit's story and the host world catch a panic to add the seed,
  the choices or the trace's tail. [in flight: change 2854]
  7: the conformance observer panics on its referee's error and asserts
  in its own checks, and the conformance harness and world catch to add
  context. [in flight: change 2855]
  7: the views' referee asserts in its expectations, and the client's
  model panics on a patch that breaks it. [in flight: change 2856]
  7: jig's world referees implement `observe`, not `watch`.
  [in flight: change 2857]
