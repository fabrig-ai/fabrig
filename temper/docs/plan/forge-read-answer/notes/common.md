# forge-read-answer: what every task holds to

- **Focused and fuzzy alone (`AGENTS.md`, 5; the user, 11-06).** Never `tools/tiers/run`, a `<tier>-<tree>` profile, `scripts/browser.sh`, a walk, a render check, a throwaway driver or a live run, even where a test's name invites it. The live stories `a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge` and its Claude Code twin are after-landing's to run; this plan changes nothing they compile.
- **Tests stay cheap.** A new case of one behaviour is a row of an existing table test (`skein/docs/foundation/testing-strategy.md`, 8); a new test is only one your spec sketches. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate: name your new test's time in the report. No test reaches a real host.
- **Failing test first.** Where your spec says so, commit the test red before the fix, so the branch's history shows it failing; say in the report what it failed on.
- **No stored format or protocol change.** `CallAnswer`, the store's records and the wire stay as they are: nothing here needs one. If your change seems to need one, it's a question.
- **`forge_route.rs` is shared.** Other plans write `temper/crates/temper-engine-domain/src/route/forge_route.rs` and `temper-engine-domain`. This plan's hunks stay in `route/forge_route/reads.rs`; `forge_route.rs` itself is not edited.
- **`docs/design/` stays closed** but for the status entry your spec names: the design sections are already written (`notes/contracts.md` cites them).
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair,** keeping what it checks for everything else, and named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If the base passes and your branch fails, it's yours.
