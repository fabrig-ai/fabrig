# delivery-checks: status

2026-11-11. Where the design lags or departs from [delivery-checks.md](delivery-checks.md).

- **Design lags:**
  - The draft's third ranked item: the coding charter's words still say
    to deliver the work it has before a question, not to `save` it. The
    charters' words are generated elsewhere, not in this plan.
  - The draft's third ranked item: an early start may merge a dependency's save, unchecked
    (`../domain/forge/change/questions.status.md`, 1).
- **Design departs:** the draft's first ranked item: the focused tests'
  `cargo nextest run`, default profile, is the front end alone, selecting
  and building; the tests run through `tools/tiers/one-process default`,
  each test binary's selected tests in one process, one at a time, and
  binaries in parallel (`tools/pre-pr/design.md`, 4;
  `skein/docs/foundation/testing-strategy.md`, 8). The user, 2026-11-11:
  every caller runs the focused and fuzzy suites so.
- **Design departs:** the draft's second ranked item: nothing is built or
  tested at the base. A lint or test failure is inherited only when the
  checked tree's `.temper/known-reds.md` lists it, and a build failure is
  always the delivery's; exceeded counts, citations and design sizes are
  still compared with the base (`tools/pre-pr/design.md`, 5). Base runs
  were what pushed a failing delivery past its check bound; the user,
  2026-11-11.
- **Decided:** 3, by the owner, 2026-11-09: work in progress before a
  question is saved without a delivery, by a new tool, `save`, which pushes
  the tree to the change's own branch with no checks, marked unchecked in
  its commit and its receipt; `deliver` and a `finish` that delivers run the
  full check, and only a producer's checked finish is merged into the goal's
  branch. smith's charter (version 4) and channel (version 6) change in
  place, before v0.1.0; temper's store and the worker wires do not
  (smith's `domain/run.md`, 8.5; jig's `domain/hosts/README.md`, section 6 and `domain/hosts/charters.md` and `domain/hosts/README.md`, section 7;
  `../domain/agent/tools.md` and `../domain/agent/README.md`, 5; `../domain/worker/checkouts.md`
  ; `../protocol/git.md`, 4.7).
