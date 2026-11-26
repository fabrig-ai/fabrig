# testing: status

2026-11-26. Where jig's code lags or departs from [testing.md](testing.md).

- **Not yet:** 4: the core world's root holds the engine, and the core
  world's kit, the reference service and the worlds that step past the
  root drive it by the engine's functions. [in flight: change 19209]

Section 5's keyed durable-row evidence and exhaustive forked cuts are
present. Section 9's seven store sweeps run in fuzzy, sharing their
checks with the chosen focused cuts. Complete snapshots, startup and
cold readback remain the contract; section 10 holds further observation
and world-cost work.

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
  4: the core world's crash sweeps hold their prefix once, forking at each
  cut, and a disk world opens its disk store alone. [in flight: change 6765]
  4: the core world's deletion, agents-tools, decision-read and named-people
  families fork after their shared prefix. [in flight: change 6766]
  5: the conformance world's every-commit, deletion and snapshot sweeps hold
  their prefix once, and `Store::fork` copies the durable image.
  [in flight: change 6769]
  9: the core world's standing cost per test (construction, the observation
  per commit, the referee's scans, the closing oracle) is repaired.
  [in flight: change 6767]
  9: the core world's fuzzy sessions and settings tests read their evidence
  and configuration once. [in flight: change 6768]
  9: the store world's file-operation sweeps hold their settled prefix once.
  [in flight: change 6774]
  9: a store open takes no plan for an identical, shallow layout and finds
  its registry rows by key. [in flight: change 6775]
  9: the channel world's standing cost per world is repaired.
  [in flight: change 6776]
  9: the client-protocol world's standing cost per test is repaired.
  [in flight: change 6777]
  9: the client world's cost per world is repaired. [in flight: change 6778]
  9: the ops world's repeated work is repaired. [in flight: change 6779]
  9: the core world's stories are placed in `domain`, its disk cuts in
  `simulated` and its step tests in `step`, with their moves.
  [in flight: change 6788]
  9: the conformance world's stories are placed in `domain` and its HTTP
  settings carriage in `protocol`, with their moves. [in flight: change 6789]
  9: the client-protocol world's step tests are placed in `step`.
  [in flight: change 6790]
  9: the client world's scripted stories are placed in `domain`.
  [in flight: change 6791]
  9: the ops world's domain stories are placed in `domain`.
  [in flight: change 6792]
  9: the store and channel worlds' step tests are placed in `step`.
  [in flight: change 6793]
- **Not yet:** 9: the client-protocol world's byte and protocol stories run
  in its binary placed `simulated`, not `protocol`: a move into jig's
  `protocol` layer while main holds it over its budget is a change's red
  (`tools/tiers/budgets.md`, 8), so they move once main's figure is within
  it.

- **Not yet:** 9: the focused suite's CPU budgets are by layer (`tools/tiers/budgets.toml`),
  the CI builder's gate figures of 11-16 plus a tenth, by the user's word
  of 11-16, against an ideal of 30 s (`tools/tiers/budgets.md`, 6). The
  `jig-world-cost` and `jig-world-cost-2` requests leave further cost repairs;
  their whole-suite effect, and fuzzy's fit with the moved core and store
  sweeps and `Standard` drive, await the queue's after-landing record. The
  per-test shares in the latter request do not establish a saving or a
  whole-suite total. No budget or build setting changes are part of those
  repairs.
