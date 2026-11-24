# testing-fakes: status

2026-11-24. Where the tree's code lags or departs from [testing-fakes.md](testing-fakes.md).

- **Not yet:** 1.1, 1.2: jig's record check, `jig_inspect::records`, and its calls wherever the shared checks run over a core [in flight: change 15645].
- **Not yet:**
  1: the core kit's `World` keeps one commit per run, rehearsed in
  `World::finish`, and its rehearsal pays for its store and start alone.
  [in flight: change 13093]
  1: the conformance harness keeps one commit per story, rehearsed in
  `Harness::finish`. [in flight: change 13095]
- **Not yet:**
  1.1: jig's inspection adapter, `jig-inspect`'s `tasks` and `core`, with
  the reads they take. [in flight: change 11665]
  1.1, 1.2: `jig_inspect::engine_clients`, and the engine-clients world's
  calls of the shared checks. [in flight: change 11678]
  1.2: the core kit's `World` calls the shared checks at its quiet point,
  with each check's own tests. [in flight: change 11676]
  1.3: the core world's scenarios of P43, S15 and P77, and P77's client
  part. [in flight: change 11668]
  1.3: the core world's scenarios of P76, P93, P98 and P82's residual.
  [in flight: change 11673]
  1.3: the engine-clients world's scenario of P117.
  [in flight: change 11674]
  1.3: the exclusions in `jig_core_kit::open` and
  `jig_engine_clients_kit::open`. [in flight: change 11679]

Section 1's core row names retain typed keys until a failure prints them;
shared setup avoids the repeated parse, unused view and whole-image copy,
duplicate server limits, rebuilt rules domain and second
restart-configuration copy.

- **Not yet:** 1: `ops`'s worlds on jig's store component; they run `jig-fake-store`.
  1: the `debt-24` request of 2026-11-05 has each world name its parties,
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
  1: the `debt-25` request has its three jig items end with fewer test lines,
  and together they fell 181, but D035's own tasks rose 34, where that plan's
  story has about 200 down: the client's sweep rose 158 and the people's 61, and
  the core's fell 139; no reason is recorded.

- **Not yet:**
  1: the step crates' case tables and the two named-people tables run
  each case under `catch_unwind` and collect the failing ones.
  [in flight: change 2853]
  1: the fuzzy sweeps, the core's scenarios and bundle of cases, the
  tasks kit's story and the host world catch a panic to add the seed,
  the choices or the trace's tail. [in flight: change 2854]
  1: the conformance observer panics on its referee's error and asserts
  in its own checks, and the conformance harness and world catch to add
  context. [in flight: change 2855]
  1: the views' referee asserts in its expectations, and the client's
  model panics on a patch that breaks it. [in flight: change 2856]
  1: jig's world referees implement `observe`, not `watch`.
  [in flight: change 2857]
  1: `jig-client-wire-world`'s and `jig-client-protocol-world`'s public
  sweeps write each case's whole input before they run it, not the case
  and its length. [in flight: change 3078]
- **Not yet:**
  1: the client's world asserts its referee's findings at once, with no
  returning entry points. [in flight: change 3115]
  1: the restart oracle asserts its comparisons, and the store files'
  readback panics on what it finds. [in flight: change 3114]
  1: the host's refusal referee and turn engine, and the notes' referee,
  assert. [in flight: change 3116]
  1: jig's domain step tests trigger assertions of start order, routing
  and their constructors' limits. [in flight: change 3110]
