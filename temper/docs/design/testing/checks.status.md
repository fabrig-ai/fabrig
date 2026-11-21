# checks: status

2026-11-20. Where the tree's code lags or departs from [checks.md](checks.md).

- **Not yet:** 6.1: no adapter is built: `temper-engine-inspect` and
  `temper-worker-inspect` don't exist, and no world calls the shared
  checks. 6.2: no temper world keeps a fault ledger, draws from the
  seed's streams or sweeps with configured faults and exclusions, and no
  check holds the 3.3 failure table.

- **Departs:** 1: thirteen named real-loop stories remain in the fuzzy
  suite: ten scripted stories cut to what only the real loop shows
  (the reference case's 4.1 with 4.3, 4.6, 5.1, 5.2, 5.4, 6.4, 6.6, 6.7
  and 6.8, and the kept-delegate landing), the program backend, and the
  B0/B1 recorded transcript twins.
  They remain there because the request for `limits-worlds` kept surviving
  stories in place, and the request for room in the budget
  (`../draft/failure-worlds.md`, 8) moved their other assertions down
  without choosing their suite. Being past a guide is not a reason to
  choose that suite (`skein/docs/foundation/testing-strategy.md`, 8.1).
  [`limits-worlds.md`](../../development/limits-worlds.md) gives every
  survivor's coverage and CPU as it stood, and `reference-case.md` each
  step's holders now. Their placement is the user's to
  decide; no exception or cost-driven move resolves this departure.

Section 1's budgets by layer start at the CI builder's gate figures of
11-16 (`tools/tiers/budgets.md`, 6); where the code doesn't yet sum or judge
them is `tools/tiers/budgets.status.md`'s.

- **Not yet:** 1: the client world's agent page read through Older over long lines is past the system world's 0.6 s reference, about 1.23 s by the gate's CPU on 11-18, once its cases shared one world and client. About 45 per cent is the engine's advance under the domain world's driver, and 12 to 17 per cent is the client's view builds. These measurements are the user's to decide on (`skein/docs/foundation/testing-strategy.md`, 8.1).
- **Not yet:** 1: the engine's `upgrade --check` naming the record a start refuses is past the end-to-end 2 s reference, 2.99 s by the gate's CPU on 11-18. About 77 per cent is the store's four registry openings, its layout derives about 60 per cent. It belongs to its tier (`skein/docs/foundation/testing-strategy.md`, 2.9), and neither invocation can be dropped.

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

- **Not yet:** 4: the box's approval-identity and approval-wait stories
  have no GitHub row: their fixtures offer Forgejo alone. Setup and
  connector tables cover GitHub's identity evidence and link decisions,
  but not the composed story the table asks for. A real human GitHub
  review counted (the draft's section 10) has no new live story or supplied
  result; it needs the user's review credential. These proofs remain
  `README.md`, section 7's later work, along with the real owned-Forgejo walk and
  rendered comparison. The joint-world card replaces the browser step
  by the plan's explicit decision.
- **Not yet:** 3: the joined story checks the card gone after landing,
  without a separate observation between merge admission and landing.
  It therefore does not independently hold that timing in the joint world,
  although the connector drops the review facts at admission (3's
  *A change waiting on a review on the forge*).

Sections 4 and 5.3 have no gap in scripted note-backed proposals and single-revision collector evidence.

`macos.md`, section 3 lists the killed-box lifetime test as Linux-only, matching its
parent-death mechanism. The combined default-runner browser and terminal
proof is not established by the supplied inventory (`../protocol/setup/box.status.md`, 1 and 2, and `../protocol/setup/testing.status.md`, 4
); the existing real-box sign-in story still awaits its
after-landing execution (`../protocol/openid-provider.status.md`, 9).

- **Not yet:**  the remaining GitHub real-loop stories remain `real-loop.status.md` (Not yet).
  the twins' playground documents remain `reference-case.status.md` (Not yet, the whole document).
   the ignored cluster process story remains `process-tier.status.md` (Not yet, the whole document).
   the playground/reset and restarted-GitHub stories remain `../protocol/setup/testing.status.md`, 4.
   the Forgejo web-connect and host proofs remain `process-tier.status.md` (Not yet, the whole document).
   the later live ladder rungs remain `live-tier.status.md` (Not yet).
   the live GitHub pull-read proof remains `live-tier.status.md` (Not yet).
   the live story's terminal-failure stop remains `live-tier.status.md` (Not yet).  the probe App's read-only record remains `live-tier.status.md` (Not yet).
   the owned-forge live landing pair remains `live-tier.status.md` (Not yet).
   the live held-record return's proof remains `live-tier.status.md` (Not yet).
   the fake shell subset remains `fakes.status.md`, 3. 4: of
  temper's kinds' stories, the forge's world's (`../domain/forge/README.md`'s status,
  18); the domain world's project's rule refused to another project's
  owner, a project made after the first start covered by the
  deployment's rule, the two requirements removed from the rules and the
  resume fixture restarting with changed values
  (`tests/engine/domain/tests/suite/settings/kinds.rs` tells the rest);
  the fuzzy sweep's changes of landing rules
  (`tests/engine/forge`'s `suite::fuzzy::simulation` makes none); and
  the real loop's smoke of a gate added to a project's rule from the
  client. 4: the client's How work runs, a proposal's card and its
  review, in their step and view tests and joined with the engine,
  since the pages are not built (`../client/README.status.md`).
- **Departs:** 5.1: stage one's golden holds a row that neither the
  reference's table (`../draft/reference-code-production.md`, section
  14) nor this section's rows have, "the goal template's work": two
  `equal` obligations, `design` and `close_out` true, for the
  reference's template 4's work (`../protocol/configuration/reference.md`),
  its subject `template-goal` number 4, not the `goal-template` role, so
  a proposal whose goal template takes another number is judged at
  template 4's work there, the stock's neither, and fails both. The plan
  `ui-p3f-design` added it to keep the golden the profile's values
  (5); the check binds no kind by a template's role
  (`temper/crates/temper-stage-one-tool/src/check.rs`, `subject_key`), and nothing
  tells Ask temper to propose it (`../domain/agent/agents.status.md`, 1).
   the live scripts' scratch-path departure remains `live-tier.status.md` (Departs, the whole document).
   the managed script's machine alias remains `live-tier.status.md` (Departs, the whole document).
   the neutral-owner exceptions remain `live-tier.status.md` (Departs, the whole document).
   the offline startup helper gap remains `live-tier.status.md` (Departs, the whole document).
   the held-record return's mechanism remains `live-tier.status.md` (Departs, the whole document).
