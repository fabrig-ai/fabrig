# checks: status

2026-11-14. Where the tree's code lags or departs from [checks.md](checks.md).


- **Departs:** 1: temper's measured focused request-world sum is
  81.892959 CPU s against 30, and the real-loop focused median
  is 31.769481 CPU s against its standing 15 s ceiling.
  The focused measurements, their load and accounting limits, retained
  causes and next decisions are in
  [`limits-worlds.md`](../../development/limits-worlds.md). The whole-temper
  focused suite total remains unknown; package sums include warm nextest
  overhead. Neither the focused 30 nor the 15 s ceiling has been raised.

- **Departs:** 1: eighteen named real-loop stories remain in the fuzzy
  suite: sixteen scripted stories (fourteen reference-case stories,
  kept-delegate landing and the program backend) and the B0/B1 recorded
  transcript twins.
  They remain there because the request for `limits-worlds` kept surviving
  stories in place. Being past a guide is not a reason to
  choose that suite (`skein/docs/foundation/testing-strategy.md`, 8.1).
  [`limits-worlds.md`](../../development/limits-worlds.md) gives every
  survivor's unique coverage, CPU and profiled instructions, and names
  the closest lower-story candidates. Their placement is the user's to
  decide; no exception or cost-driven move resolves this departure.

Section 1's fuzzy suite has no known gap against its 180 CPU s, the
strategy's 120 raised by half by the user's word of 11-08, for now, to be
revisited (`README.md`, section 7): the whole-temper fuzzy median is 125.579032 CPU s
in three warm serial runs including the driver (119.288299–127.664356
CPU s), all 81 tests passing in each, though two runs and the median
exceed the 120. The request's roughly 15 CPU s reduction and reliable
headroom, which `README.md`, section 7 keeps, are not achieved. The gate's own CPU
result is unknown from these serial samples, whose driver and feature
unions differ from the request's gate baseline.
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

`README.md`, section 2.8 lists the killed-box lifetime test as Linux-only, matching its
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
- **Not yet:** the whole document: the engine replay's digest and text
  comparisons panic on a difference, and their test catches.
  [in flight: change 2867]
