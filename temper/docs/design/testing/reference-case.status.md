# reference-case: status

2026-11-11. Where the tree's code lags or departs from [reference-case.md](reference-case.md).

- **Departs:** the whole document: the fuzzy story-placement departure remains `checks.status.md`, 1.

`checks.md`, section 1's fuzzy suite has no known gap against its 180 CPU s, the
strategy's 120 raised by half by the user's word of 11-08, for now, to be
revisited (`README.md`, section 7): the whole-temper fuzzy median is 125.579032 CPU s
in three warm serial runs including the driver (119.288299–127.664356
CPU s), all 81 tests passing in each, though two runs and the median
exceed the 120. The reduction, headroom and gate-CPU proofs remain `checks.status.md`, 1.
[`limits-fuzzy.md`](../../development/limits-fuzzy.md) records these runs,
their accounting limits, retained causes and next decisions.

`real-loop.md`'s real-loop goal-delivery story is present, alongside the six
engine domain stories and the checkout world's later-run cases. It uses
real smith and a worker with scripted model replies, fake git and the
fake forge; no real model or real forge is proved by it. `checks.md`, section 5.1's
reference obligations retain their current charter words; folding in the
goal-as-designer workflow remains in `README.md`, section 7.

`README.md`, section 2.1 has no known gap in the merge job's story:
`a_changes_merge_job_runs_on_its_worker` (`tests/system/tests/suite/merge_job.rs`)
runs a change's merge job in the composed world, both roots declaring the
kind as production does (`temper-service`'s `limits::job_kinds` and
`merge_jobs`) and the job crossing production's translation
(`assignment::job_down`, `worker_route::job_items` and `job_spec`). Its
five rows are a clean catch-up of main into a goal's branch, a settled
conflict, a conflict left for a resolution, a base absent from the cached
clone until fetched by its id, and `conflicts + 1` files settled; each reads
the forge's branch and its parents, the job's task ended and the change's
next step. Its git and forge are the fakes, its command the world's
(`tests/system/src/composed/merge_job.rs`); the worker's checkout bounds a
merge's listing as the git layer does (`../protocol/git.status.md`).

- **Not yet:**  the remaining GitHub real-loop stories remain `real-loop.status.md` (Not yet).
  the twins' documents among `temper-playground`'s stories: B0's and
  B1's are still the real loop's own fixtures
  (`tests/real-loop/tests/fixtures/b0`, `b1`).  B2's twin
  (`../../development/acceptance.md`, section 8).
  the whole document: the ignored cluster process story remains `process-tier.status.md` (Not yet, the whole document).
   the playground/reset and restarted-GitHub stories remain `../protocol/setup/testing.status.md`, 4.
   the Forgejo web-connect and host proofs remain `process-tier.status.md` (Not yet, the whole document).
   the later live ladder rungs remain `live-tier.status.md` (Not yet).
   the live GitHub pull-read proof remains `live-tier.status.md` (Not yet).
   the live story's terminal-failure stop remains `live-tier.status.md` (Not yet).  the probe App's read-only record remains `live-tier.status.md` (Not yet).
   the owned-forge live landing pair remains `live-tier.status.md` (Not yet).
   the live held-record guard proof remains `live-tier.status.md` (Not yet).
   the fake shell subset remains `fakes.status.md`, 3.  the remaining settings-kind stories remain `checks.status.md`, 4.  the remaining client-page stories remain `checks.status.md`, 4.
- **Departs:** the whole document: the case library's purpose is retired: the reference
  case it was written from is replaced (`../draft/reference-case.md`), and
  its code awaits removal with its design.
   the fake-git tokenizer departure remains `fakes.status.md`, 3.
