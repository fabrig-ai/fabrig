# ui-ways-rules: rules every task follows

- **Read first:** fabrig's `AGENTS.md`, `temper/AGENTS.md`, your spec, and
  `temper/docs/plan/ui-ways-rules/notes/contracts.md`, which binds the
  shapes that cross tasks. Where a spec and the contracts note differ, the
  note wins; say so in the report.
- **The request:** `temper/docs/plan/ui-ways-rules/inputs/request.md`, read
  whole (it is short). Its `For every task of this plan` list binds every
  task; the lines below repeat what bites here.
- **Only focused and fuzzy run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, a browser, process or live tier, or
  `scripts/browser.sh`. A test of those tiers your change must touch is
  edited and compiled (`heavy cargo check -p PKG --tests`), never run;
  say so in the report. No walk, render check, screenshot or throwaway
  driver: the plan is complete when it lands.
- **Tests stay cheap.** A new case of a behaviour is a row of an existing
  table test where one fits; a new test only where none does, beside its
  siblings' fixture. temper's focused and fuzzy suites have 30 and 120 CPU
  seconds in the gate; a fuzzy test of this plan runs at most 64 seeds.
- **Fakes, never the real thing:** the forge's own test fixtures
  (`temper/crates/temper-engine-domain-forge/src/tests/fixtures.rs`), the
  forge world (`temper/tests/engine/forge`, package
  `temper-engine-forge-world`), the engine's domain world
  (`temper/tests/engine/domain`, package `temper-engine-domain-world`), the
  service's test fixtures (`temper/crates/temper-service/src/tests/fixtures.rs`)
  and the client world (`temper/tests/client`, package
  `temper-client-world`).
- **Design files are the cut's,** with one departure each spec states: the
  task deletes the **Not yet** entries it closes from the status files it
  names, and touches no other design text.
- **Help pages:** `temper/docs/help/` is closed to an implementer but where
  a spec names a page, as a departure; a page your change pushes past its
  byte bound is condensed, losing no fact, never its bound raised.
- **Generated files your change makes stale** (goldens, prints,
  `temper/docs/help/reference/*`, journey recordings, counted baselines)
  are yours to regenerate with their own tool, in the same task, and to
  name in the report. `temper-client-journeys again` is generation, not a
  walk: run it when a recording goes stale (the journey recordings hold the
  settings inventory of the stock, so a change of the stock's defaults
  makes them stale); on "a key mark caught a moving page" retry it at most
  three times, each on a fresh fake box.
- **A test's oracle or helper that the correct new behaviour trips** is
  yours to repair in the same task, keeping what it checks for everything
  else, named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once on the pristine base. If the base passes and your branch fails,
  it is yours.
- **Decide, don't ask.** A stored format, a protocol or a public interface
  may change freely: no compatibility. A small edit just outside your
  files (a re-export, a `mod` line, a doc comment, a golden) is made and
  named in the report. A `Question:` is only for what truly blocks your
  acceptance, and for charter 5's words (the contracts note, section 3).
- **Not to touch:** `temper/crates/temper-client-shell-web/temper.css`,
  `temper/docs/design/**` but the status entries your spec names, and
  every other task's owned files below.

| Task | Owns |
|---|---|
| `forge-rules` | the forge's judging, matching and holding of landing rules; the root's approval coverage and its run `settings` read of kind 2 |
| `stock-roles` | charter 5, template 4 and the delegation in `temper-service`'s stock, and every fixture and count that pins the stock's charters, templates and delegation |
| `field-words` | the settings form's words for a landing rule's pattern, its reference page, and `temper/docs/help/landing-rule.md` |
| `project-stock` | the binary's forge stock (kind 2's rule 1 for every project in place of kind 1's rule 1) and the tests and worlds that assumed kind 1's |
| `reference-rules` | `temper/docs/examples/reference/project.json` and `README.md`, and the reference's tests of them |
