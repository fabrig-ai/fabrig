# Testing scenarios and their referee

This holds testing scenarios and their referee, split from the [testing design](README.md).

## 1. Scenarios

### 1.1 Setting one up

A scenario (skein/docs/foundation/testing-strategy.md, section 7) is set up in the terms of
temper's fakes: each conversation's script for the LLM, the repositories
on the forge with their files and checks, the items and the people, and
the faults to inject. A scenario cues its scripts through content: a
repository's files say which script its run follows.

A scenario's limits are the shipped profile's, `small`, built by
`temper-service`'s limits module (../protocol/configuration/limits.md),
in every world, fixture and process-tier configuration: none is set by
hand. A scenario that needs another value changes it by name on the
profile's, with the reason beside it. A limit set by hand hides what the
shipped profile refuses, and the shipped numbers drift toward the
fixtures'.

### 1.2 The referee

The referee is the strategy's (skein/docs/foundation/testing-strategy.md, section 7). temper's
expectations are of this kind: safety, such as nothing landing on a
default branch without green CI, or no forge credential reaching an
agent; liveness, such as a pull request opened, with green CI, within
the hour. What temper adds:

- **It steps on what it observes,** and on its own timers, not on every
  iteration, so its work per iteration stays bounded. Its earliest
  deadline is covered by the shell's one ring timeout, as every layer's is
  (skein's `../foundation/programming-model.md`, section 9).
- **A failure in the real-loop, process or live tier** is reported with
  the trace and the files the run left, since the seed does not replay
  there.
- **Its checks return their findings,** as skein's do (skein's
  `testing/README.md`, 5; skein's `../foundation/programming-model.md`,
  10.4), and nothing in temper's tests catches a panic:
  - every referee and judge of temper's worlds implements `watch`, in
    every tier, and keeps what its `act` or `idle` finds for its next
    `watch`; the real loop fails with the finding after its forced
    cleanup, and a check's own test reads it through the world's
    `try_run`;
  - the one-process world's durable observer returns a current
    generation's row that doesn't decode as its finding, which its
    referee returns; an old generation's rows stay opaque until the
    switch (`tiers.md`, 1.5);
  - the real loop's checks of a call's answer within its bound, of a
    CLI timeout and of an overdue deadline with its diagnostic and
    figures, each a returning form beside the assertion a story calls
    at its top; a starved wait still passes as starved
    (`real-loop.md`);
  - the engine replay's digest and text comparisons (`checks.md`), the
    fake git program's replay order (`fakes.md`, 3), the browser
    stories' attribution (`../client/web/testing.md`, 3), the process
    tier's printed link and cluster members (`process-tier.md`), and
    the GitHub stories' secrets scans and teardown (`live-tier.md`);
  - the engine's domain world's cold restart oracle: its comparisons of
    the rows, the answer and the forges a restart reads back, and the
    erasing close, return the first difference, naming its key, and the
    world's close and the conformance world's violation carry it to
    their top; the walking world's delivery of a result its referee
    refuses, which the world keeps for its run;
  - the worker world's referee of turns (`checks.md`, 2): a replay that
    changes a turn's body or spend, or a turn past one never seen, is a
    finding the world keeps for its run;
  - the one-process world's observer of saved artifacts, whose scan for
    a newly issued token returns its finding through the observer's
    close (`tiers.md`, 1.5);
  - the real loop's `Until` step, which an observation that failed or
    was signalled ends with its finding, whatever its stdout
    (`real-loop.md`);
  - temper-box-kit's ctl checks that a child's prerequisite succeeded
    before its output is read or polled again, and that a refused
    adoption stops what waits on it, kept for its judge's `watch`; and
    the fake box's readiness, which returns the child's exit status and
    stderr when the child ends before it is ready;
  - the port chooser's refusal of an ephemeral range that leaves fewer
    than two blocks, naming the range (`process-tier.md`), which its
    caller panics on;
  - a harness that names a seed, a walk, a case or a story's step
    writes it to stderr before it runs it, and a table of cases stops
    at its first failing case, having named it. The runner prints a
    failed test's output.

  No temper test triggers an assertion: what an assertion guards is
  tested through the outcomes around it (skein's
  `../foundation/programming-model.md`, 10.4).

The referee is built on skein's world harness (`skein-world`; skein's
`testing/README.md`, section 5), as ordinary Rust: each world writes its
scenario's expectations, with the stimuli it may inject, and the
harness judges them.

Every engine world has a referee, the engine's own included, over what
the forge did and what the engine assigned. Its stimuli are an engine
restarting cold: at a drawn moment; at once as it posts an outcome, so
that it restarts while applying it; or at a moment a scenario chooses as
the forge shows it. The worker's worlds add a worker's channel dropping,
a worker vanishing and a worker told to shut down. The engine's referee
meets a story only when its item closes. The system world checks its
stories independently of the roots, against durable task and call
records and smith's own observations (`tests/system`). The conformance
world (`tests/conformance`) runs the engine root on jig's conformance
harness, under its outside referee. Its fuzzy sweep crashes each live
commit of a story once, every cut forked from the one baseline (jig's
`domain/testing.md`, 5). Its stories' and scenarios' heap
ceilings are three times the figure skein's ../foundation/testing-strategy.md,
section 6 gives (the measured peak plus a quarter, rounded up), the
user's, 2026-11-03: an engine running a workload of many goals holds
tens of MB, so a quarter's headroom over one story cost a repair at
each milestone's added state, while three times still catches a leak.
The worker's child domain worlds keep their checks inline.

**Beside the referee, the harness's invariants.** The referee never
reads a domain's state. What must hold of a component's frozen state, its
waits and its bounded tables, is a harness invariant: the world reads the
component through its inspection adapter at its driver's quiet point and
returns the shared checks' findings as its run's (skein's
`testing/inspection.md`; `checks.md`, 6.1). A world that injects faults
records each in its run's fault ledger, draws its workload, faults,
parties and scheduling from independent streams of its seed, and a sweep
over its seeds fails when a fault it configures never falls; its only
exclusions name ignored scenarios (skein's `testing/sweeps.md`;
`checks.md`, 6.2).

## 2. Open questions

- **Differential checks:** whether a scenario must end the same way at
  every tier it runs at, as a check that the layers below the domain
  decide nothing.
- **What the referee sees in the process tier:** the machine is real
  there and reports nothing, so an expectation about it, such as what an
  agent's environment holds, needs facts that say so, or is checked only
  in the tiers below. Default: checked in the one-process world.

## 3. Later

- **Deployment-world costs and surviving stories** (`real-loop.md`, `scenarios.md`, 1.2, `reference-case.md`, `checks.md`, 1):
  the measurements and retained causes are in
  `temper/docs/development/limits-worlds.md`. Decide the named survivors'
  placement by their unique coverage, with lower holders first where a
  behaviour belongs below; the closest candidates are case 5.5's depth
  refusal and note recall, and the inbox, effort and advisor checks in
  step 6.5 of `reference-case-milestones.md`.
  Recorded-model twins retain their independent transcript histories.
  Cost grants no exception, and moving a story to escape a budget raises
  it. Whole-suite focused CPU measurement and the gate's fuzzy accounting
  remain necessary; partial package sums and failed-prefix instruction
  profiles prove no suite pass.
  Further cause work needs bounded shared schema representations and
  prompt admission measurements without changing validation or refusal
  order. Engine-domain and one-process fuzzy profiles and their retained
  causes are in `temper/docs/development/limits-fuzzy.md`.
  Required engine steps, deadline queries, configuration admissions,
  restart/readback and token searches stay until a profile supports an
  equivalent algorithm. Real-loop matching and public setup candidates
  below roughly 2% of a story stay. Complete repair-story instruction
  totals need a successful profile; instrumentation failures establish
  none. Budgets, the real-loop ceiling and any hot-leaf build override
  remain the user's decisions.

- **Reference obligations for a goal that designs** (`checks-stage-one.md`, 1.1): fold
  `../draft/goal-as-designer.md`'s workflow into the reference, its charters
  and stage one's judge together. Goal-delivery stories use test charters;
  they do not establish the reference's roles, models, efforts or wakes.
  Default: keep the current reference obligations until that change.

- **jig's referee tests, repeated.** `tests/engine/domain/tests/suite/referee.rs`
  keeps tests of jig's referees that jig's own suites hold:
  `authority_referee_rejects_a_person_without_a_grant` observes what
  `rejects_an_observer_routed` does in `jig/tests/people/tests/suite/referee.rs`.
  Default: they stay, until a debt item names them.
- **A cached state graph.** Below the real loop a state is a pure function
  of its seed and path, so a crash sweep could fork at a prefix it now walks
  again. The slow tiers wait: a process story's restore has its meaning to
  prove, and the request `test-speed` of 2026-11-06 takes what needs no cache
  first.
