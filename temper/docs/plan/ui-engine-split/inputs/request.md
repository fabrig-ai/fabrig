# ui-engine-split: the engine's own binary, and the service split

Milestone M2 of `temper/docs/design/draft/deployment-binaries.execution.md`: the engine, the service's split and `temper-engine`, with its web, `forge`, `versions`, `upgrade`, help and `--version`. The draft is `temper/docs/design/draft/deployment-binaries.md`, sections 4 and 9.2, the engine's half. The user's word, written by the draft's author; the director routed M2 here. Nothing in the draft is reopened. The user's development process stays as it is: no new CI and no build job.

## What it executes

- **`temper-service` splits** so that an engine-alone service composes without the worker's crates, cutting section 4's couplings on the engine's side:
  - `fit_brief` moves out of `temper-engine-smith` into a crate the engine and the worker both link;
  - the engine's limits read no worker domain or checkout crate;
  - `temper-protocol-records`' `forge` feature is on only where it's needed.

  The standalone service that composes both stays for the box and the tests, and the box's `temper` keeps every command it has.
- **A `temper-engine` package and binary:** `serve` (an engine-alone document), `forge …`, `versions`, `upgrade`, `help`, `internal forge-child`, and `--version`. It's the only binary to embed the browser bundle. It refuses a document with a worker member, naming `temper-worker` as the binary that would run it.
- **The engine's half of the boundary test** (section 4): `temper-engine`'s dependency closure, as `cargo tree` gives it, holds no worker domain and no smith agent. Any cut that costs more than it gives is named as that binary's exception, with its reason.
- **The design first:** the service's and setup's design sections this changes, with their status files, and the draft's status for M2.

The worker's package and `program-login` are M3's, built on this split by the owner of coding runs. Leave the worker's side composing as today, and name in `plan.md` what M3 takes from this split.

## Its tests

Focused and fuzzy only:
- `temper-engine`'s step tests: `--version`, its help, an unknown command refused, a worker's document refused naming `temper-worker`;
- the engine's dependency closure.

The draft's cluster story on separate binaries is written and compiled in M3 or M5, not here.

## Sources

- `temper/docs/design/draft/deployment-binaries.md`
- `temper/docs/design/draft/deployment-binaries.execution.md`


## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Tests stay cheap (temper/tiers, 11-06):** a new case of one behaviour is a row of an existing table test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost gets rejected there.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **`temper-client-journeys again` is generation, not a walk:** it regenerates what the gate's focused replay checks read, so a task whose change makes a recording stale runs it, though it drives a browser.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
