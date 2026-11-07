# hearth-apt-304: what every task holds to

- **Focused and fuzzy alone (`AGENTS.md`, 5; the user, 11-06).** Never `tools/tiers/run`, a `<tier>-<tree>` profile, `scripts/browser.sh`, a browser, process, live or conformance tier, one of their tests (`conformance_*`, `recorded_*` binaries), or a tier's CPU figure. This overrides anything in a spec.
- **Decide, don't ask.** A `Question:` is only for what truly blocks the task; the spec's options settle the rest.
- **Recordings are captured, never written by hand** (`hearth/docs/design/domain/testing.md`, 4.5). No file under `hearth/testing/recordings/` is edited, regenerated or reformatted, and `1.5.7-apt3.0.3`'s recordings and expectations stay as they are.
- **No code, doc, fake or test change is expected.** A focused or fuzzy test that trips over a second version directory has its assumption repaired, keeping what it checks for each version, and the repair is named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If the base passes and your branch fails, it's yours.
