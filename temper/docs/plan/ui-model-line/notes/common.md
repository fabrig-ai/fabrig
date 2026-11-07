# ui-model-line: for every task

The request: `temper/docs/plan/ui-model-line/inputs/request.md`. Its section *For every task of this plan* holds for you, and wins over your spec where they differ. The design is written and on the trunk.

- **Only focused and fuzzy run.** No browser, walk, render check, `tools/tiers/run`, `scripts/browser.sh` or `again`. This plan changes no wire form, so no journey's recording goes stale; a recording that seems to is a question.
- **Tests stay cheap:** a new case of one behaviour is a row of a table test. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate.
- **Goldens and prints** a task makes stale are that task's to regenerate with their own tool, never by hand: `SKEIN_CODEGEN_WRITE=1` for the terminal's goldens, `heavy cargo run -p temper-client-journeys` for the journeys' prints. The gallery commits no prints, and this plan adds it no sample. A print whose diff isn't this plan's words is a red, not stale.
- **`temper.css` isn't expected to change:** the bar's `.temper-byline` rule already has the muted form.
- **Each task removes its own `Not yet` entry** from the status file its spec names, the one edit under `docs/design/` it may make.
