# ignored-tiers: rules every task keeps

- **The request** is `temper/docs/plan/ignored-tiers/inputs/request.md`:
  tests run by hand that need no person join a tier, and the ones that
  still need a person say why. Read it for the why. Build what your spec
  and the design sections it names say.
- **Where a test goes** (`skein/docs/foundation/testing-strategy.md`, 8
  and 8.1): what it checks decides its tier, never its cost. A test that
  starts the binary and its processes is `process-temper`'s. A world's
  test stays in the suite its crate's binary is in already (the focused
  suite, profile `default`). `live-temper` takes no test that needs no
  real model. Each test is selected by one profile.
- **The selection contract** (`.config/nextest.toml`; `b0-offline` writes it,
  and no other task edits that file):
  - `[profile.default]`'s temper block gains, below
    `& not binary(/^(process|live|live_github)$/)`:
    `& not (binary(=proof_live) & test(/^rung_b0s_/))` and
    `& not (binary(=live_github_managed) & test(=managed_ready_client))`.
  - `[profile.process-temper]`'s `default-filter` becomes
    `"(binary(=process) | (binary(=proof_live) & test(/^rung_b0s_/)) | (binary(=live_github_managed) & test(=managed_ready_client))) & (package(/^temper(-|$)/) | package(=git-transcripts))"`.
  - Nothing else in the file changes: no profile's timeout, threads or
    other filter. The other tests in `proof_live` (`support::forge_land`'s
    unit tests) and in `live_github_managed` (`relay_managed`'s six unit
    tests) stay in the focused suite, where they are now.
- **Only focused and fuzzy run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile's run, `scripts/browser.sh`, a process or live
  story, `scripts/proof-live.sh` or `scripts/proof-live-github-managed.sh`.
  A process-tier test is written and compiled, not run. Show that it
  compiles (`heavy cargo check -p temper --tests`) and that its profile
  selects it, by listing:
  `heavy cargo nextest list -p temper --profile process-temper` shows it, and
  `heavy cargo nextest list -p temper` (the focused profile) does not. A
  list builds but runs nothing, so it is allowed. Quote both lists'
  lines for your tests in the report. The after-landing run proves the
  test, and a red it raises comes back as a repair.
- **A focused test that drops its `#[ignore]` runs,** alone first
  (`heavy cargo nextest run -p PKG --test BIN -E 'test(=PATH)'`, count 1,
  `PATH` the test's full name with its modules, as `nextest list` shows it),
  then with its package. Report its time from nextest. A test over its
  guide (`skein/docs/foundation/testing-strategy.md`, 8: 1 s for a
  focused world test) is examined for its cause (8.1, question 2) and
  named in the report. It is never moved or ignored again for its cost.
- **No figure in `tools/tiers/budgets.toml` changes,** and no timeout,
  slow period or thread count in `.config/nextest.toml`.
- **Old forms:** `python3 -I -B temper/scripts/old_forms.py` once before
  pushing. A raised count is yours to fix.
- **Status lines:** the design is written (`## Inputs` names it). A task
  edits only the status entries its spec names, in the words the spec
  gives. That departs from the implementer's text, which closes
  `docs/design/` to you: those entries are your task's.
- **Decide, don't ask.** A small edit just outside your file list is
  made, and named in the report. A `Question:` is only for what truly
  blocks your acceptance.
- **Before calling a focused or fuzzy failure "outside the change,"** run
  it once on the pristine base. If it passes there and fails on your
  branch, it is your branch's.
- **Citations:** after committing, run
  `python3 -I -B tools/citations/citations.py --base origin/main` from the
  repository root. It reads HEAD, not the working tree.
