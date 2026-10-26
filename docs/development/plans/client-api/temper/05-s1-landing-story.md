# Supplement temper-05.s1: the process tier's landing story passes again

Added by ruling 347 (the request of 2026-10-18, for
forge-setup's step 11.1 and its Done when 4). Read first: temper's
`AGENTS.md`, `docs/development/workflow.md`; this plan's README, "Rules
of this plan" and "Pins"; temper's `docs/design/domain/testing.md`, 2.6.

Scope: `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart`
(`crates/temper/tests/process.rs:68-89`) passes again on `main`. It
passes at temper `95a172cc5` and fails at `01e467afc`. forge-setup's
bisect (first parent, one run per commit) names three
of this run's landings:
1. `3f8b3df6f` (temper-05.1, "Mount jig clients on temper's own HTTP
   server"): the first run never lands; person step 4 misses
   `support::ANSWER` ("Process answer", `support/mod.rs:343`), and the
   fake LLM has 0 queries. `main` fails here now.
2. `e461dfd8d` (temper-04.3): "native signing failed: Refused"
   (`support/person.rs:133`), in 3 to 4 s.
3. `fe8fcdf6b` (temper-03.5, "Route temper children through the shared
   jig client"): the first run lands, and the restart run
   (`person::found`) misses "Reply message:" in "Task 2:" at native
   person step 1 (`support/person.rs:272`, raised at `support/mod.rs:419`).

Needs: temper-04.s1 (ruling 352): its repin takes jig `38ae1ba17`, which
carries authorities whole; at temper's pin, jig `e0c7b18bd`, the fixture's
grants are cut and the story's first act is not offered. The story has
carried no ignore since 04.4.

Shape: one commit for each cause, in the order the story meets them on
`main`. The bisect's commits say where to look, and the focused log
decides. Each repair goes where the log names it: temper's production
code, or the process tier's support. Adapt it as jig's reference does the
same thing (README, "Pins"). A person step or a find changes only where
the design changed what the page shows, with that section quoted in the
commit body.

Test shape: ruling 229. Reproduce each failure point first in a focused
test, at the lowest layer that shows it, and log
its cause there. The repair changes only what that log names. The
focused test stays as the step's test.

Proof: with `export
TEMPER_FORGEJO=$HOME/src/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64`
and `export
TEMPER_FORGEJO_RUNNER=$HOME/src/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64`
(carbon-forge's paths for forge-setup's `~/src/rust/tmp/…`), run the
story alone: `sh -c 'cargo build -p temper --bin temper &&
NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper --profile process -E
"test(a_persons_change_lands_on_the_forge_and_is_found_after_a_restart)"'`,
with ruling 229 for the count. Then run the whole tier once, `scripts/proof-one-process.sh`, as forge-setup's README, Done when 4 runs
it (within 180 s). Any other test of the tier that fails goes in the report
with its first failure line, and is not repaired here.

Size: about 150 production lines. Past 300, it asks.

Don't: change the story's assertions, the fake LLM's scripts, a referee's
check or a timeout to pass; mark the story ignored; use another Forgejo
or runner; change jig. A cause in jig stops the step, with the log's
cause line.
