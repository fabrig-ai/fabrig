# projection-full-outbox: rules every task keeps

- **The request** is `temper/docs/plan/projection-full-outbox/inputs/request.md`:
  a deployment's engine spun inside one `Engine::step` because a goal's
  issue projection that met a full forge outbox asked to be projected
  again at once, and the root did so in the same step, forever. Read it
  for the why; your spec and the design sections it names are what you
  build.
- **No figure changes.** The outbox's bound (`entries`, 8 in
  `temper-service/src/limits.rs`), the issue policy's `interval` and every
  other limit stay as configured, in code and in test fixtures.
- **No store change.** The waiting projection's due time is the volatile
  timer the forge already keeps (`projection_due`), which a restart
  re-arms; the step's mark (`notes/contracts.md`) is volatile too. No
  schema, record or golden moves.
- **Only the focused and fuzzy suites run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a browser, process or
  live tier story, a walk, a render check or a throwaway driver.
- **Tests stay cheap and fail first:** write the test, see it fail for the
  rule's reason, then the fix. A new case of one behaviour is a row of the
  table test your spec names. No test reaches a real host. Budgets and
  timeouts are the user's, never raised.
- **No hanging test:** a test that would loop forever without the fix is
  not run before the fix. Each spec's failing test fails on an assertion.
- **Every build and test through `heavy`,** with `TEMPER_BUNDLE=none` (the
  machines' cargo config sets it). `cargo fmt` builds nothing and runs
  directly.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once alone on the pristine base. If the base passes and the branch
  fails, it's the branch's.
- **Your status entry is yours.** Each spec names one **Not yet** entry in
  `temper/docs/design/domain/forge.status.md` that it removes, its lines
  alone; this departs from the implementer's text, which closes
  `docs/design/` to you. Nothing else under `docs/design/` changes.
- **Decide, don't ask.** A small edit just outside the file list is made
  and named in the report. A `Question:` is only for what truly blocks the
  task.
