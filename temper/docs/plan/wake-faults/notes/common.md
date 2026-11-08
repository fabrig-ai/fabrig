# wake-faults: rules every task keeps

- **The request** is `temper/docs/plan/wake-faults/inputs/request.md`.
  Read it for the why; your spec and the design sections it names are
  what you build. The evidence is outside this repository: the request's
  words are all there is.
- **The contracts** that cross tasks are
  `temper/docs/plan/wake-faults/notes/contracts.md`: names, types and
  signatures there are binding. A change one of them needs is a
  `Question:`, never a quiet departure.
- **The design is written.** This plan's cut wrote the sections your spec
  cites (jig's `docs/design/domain/tasks.md`, `engine.md`, `hosts.md`,
  temper's `docs/design/domain/worker.md`, the draft
  `temper/docs/design/draft/per-wake.md`, section 4). Each status file
  beside them carries a **Not yet** entry per gap, naming the code that
  lags. **Your spec names the entries that are yours: delete exactly
  those, and nothing else,** when your acceptance holds; if a status
  paragraph then says the section has no gap, leave it as it stands. That
  deletion is the one edit under `docs/design/` this plan's tasks make
  (a departure from the implementer's text, which closes `docs/design/`).
- **Failing test first, at the lowest tier that shows it.** Write the row,
  run it, see it fail for the fault's reason (say what you saw in the
  report), then fix. A new case of one behaviour is a row of an existing
  table test; a new story goes in a world below the real loop. No test
  reaches a real host or a real LLM.
- **Only the focused and fuzzy suites run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a walk, a render check or
  a throwaway driver. A slow-tier test whose expectation your change makes
  stale is updated, compiled (`heavy cargo check -p PKG --tests`) and named
  in the report.
- **No stored format or protocol change.** None of this plan's tasks
  changes a store schema, a wire schema, a golden or a generated codec. If
  your fix seems to need one, stop and ask.
- **Dependencies point one way:** jig never names temper; smith names
  neither.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once alone on the pristine base. If the base passes and the branch
  fails, it is the branch's.
- **`jig-domain-tasks` has two writers in this wave,** `finish-parks` and
  `covered-entries`, on disjoint lines: each spec names its own. Edit
  another task's lines only as your spec allows.
