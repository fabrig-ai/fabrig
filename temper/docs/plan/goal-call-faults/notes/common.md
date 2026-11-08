# goal-call-faults: rules every task keeps

- **The request** is `temper/docs/plan/goal-call-faults/inputs/request.md`:
  three faults a goal's coordinator met in its host calls on a real
  deployment. Read it for the why; your spec and the design sections it
  names are what you build. The evidence is outside the repository: the
  request's words are all there is.
- **The failing row first:** your first commit adds the row (or rows) your
  spec names to the existing table it names, and you see it fail for the
  fault's reason before you change production code. The report quotes
  the failure seen.
- **Tests stay cheap:** a new case of one behaviour is a row of an
  existing table test. No test reaches a real host or a real LLM; the
  worlds' fakes and scripted agents stand in.
- **Only the focused and fuzzy suites run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a browser, process or live
  tier story, nor a walk, render check or throwaway driver. A slow-tier
  test whose expected output your change makes stale is updated, compiled
  (`heavy cargo check -p PKG --tests`) and named in the report.
- **Budgets, timeouts and figures are the user's:** no task adds or
  changes a configured figure, a charter's words, a model or an effort.
- **No stored format and no protocol changes:** no wire schema, no store
  record, no journal entry changes shape. A fix that would need one is a
  `Question:`, never a quiet departure.
- **jig never names temper;** a mechanism a jig fix needs stays in jig.
- **Not yours:** `docs/design/` (the design this plan wrote is the
  contract; what the code taught goes in the report), the drafts, and
  `docs/help/` but a generated page your change makes stale, regenerated
  by its own tool.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once on the pristine base. If main passes and the branch fails, it is
  the branch's.
