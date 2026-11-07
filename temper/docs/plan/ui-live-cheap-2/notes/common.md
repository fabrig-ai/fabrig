# ui-live-cheap-2: what every task holds to

- **Focused and fuzzy alone.** Never `tools/tiers/run`, a `<tier>-<tree>` profile,
  `scripts/browser.sh`, a walk, a render check or a throwaway driver. The live story
  (`temper/crates/temper/tests/live.rs`) is compiled, not run: no worker holds its login,
  and after-landing runs it hourly while it is red. Say so in the report.
- **Tests stay cheap.** temper's focused and fuzzy suites go to 30 and 120 CPU seconds in
  the gate, and the gate rejects a plan that adds cost. A new case of one behaviour is a
  row of the existing table test the spec names; a new test is only the one the spec
  sketches, in the cheapest world that runs both real sides. Report its time as nextest
  prints it.
- **The failing test first.** Write the spec's test against the code as it stands, run it,
  see it fail for the reason the spec gives, quote the failure in the report, then build.
- **A new test must bite** (the implementer's text): break the rule once by hand, see the
  test fail for the rule's reason, restore.
- **jig is landed and stays as it is,** but where a spec pre-approves a jig edit for a
  stated finding. No task changes a stored format, the client wire or a refusal
  vocabulary.
- **`docs/design/` stays closed** but for the status sentence a spec gives word for word,
  as a stated departure from the implementer's text: the design is written, and its
  status must say what is built when the branch lands.
- **A test's oracle or helper that your change's correct behaviour trips is yours to
  repair in the same task,** keeping what it checks for everything else, and named in
  the report.
- **Before calling a failure "outside the change",** run it once on the pristine base. If
  the base passes and your branch fails, it's yours.
