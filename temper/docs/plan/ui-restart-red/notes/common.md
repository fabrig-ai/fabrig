# ui-restart-red: what every task holds

From the request (`temper/docs/plan/ui-restart-red/inputs/request.md`), which each task obeys:

- **Only focused and fuzzy run in a plan, with one exception.** Never `tools/tiers/run`, a browser or live tier, a whole process tier, or a tier's CPU figure. The exception is the request's own: this plan repairs a red, so the five stories its table names may each run by name, alone, through `heavy`, and the report gives each one's result. No other process or live story runs. `temper::live`'s stories are read, never run.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a golden) is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that a change makes stale are its task's to regenerate; don't park on them.
- **A help page** a change grows past its bound is that task's to shorten, losing no facts, and never raises the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report. No assertion of the five stories is weakened, least of all that no token file remains.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
- No `Look-request` trailer.

Beside the request:

- **A focused test for each cause, shown red without the fix:** the report names the rule broken and the failure seen.
- **Budgets and timeouts are the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1): none is raised here, and a story's deadlines stay as they are.
- **Security:** never read a handover's or any token file's bytes; check one by `stat` at most (`AGENTS.md`, 3).
