# checks: status

2026-11-10. Where the tree's code lags or departs from [checks.md](checks.md).

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

- **Not yet:**  GitHub's real-loop stories 3, 4, 5, 7 and 8; 1, 2 and 6
  run (`tests/real-loop/tests/suite/github_land.rs`, `github.rs`). 
  the twins' documents among `temper-playground`'s stories: B0's and
  B1's are still the real loop's own fixtures
  (`tests/real-loop/tests/fixtures/b0`, `b1`).  B2's twin
  (`../../development/acceptance.md`, section 8).
   the cluster's 4.6 process story is ignored: its fixture
  expects `engine.llm.models` in the startup configuration, which holds
  no model since models became settings (`../protocol/configuration/README.md`,
  section 2), so the story has not run since ws says it drains, and jig's
  hub, translation and fleet tests alone hold the drain (jig's
  `domain/engine/README.status.md`, its 8), where the request `ignored-misc` of
  2026-11-08 asked that it run by name and pass, a test whose design has
  moved being rewritten to it.
   the
  playground's stories, in and out and a reset the box's death cut, and
  GitHub gone on after a restart, as `../protocol/setup/README.status.md` records
  them.
   Forgejo signing in through temper: the story's connect gives no
  `--web`, its `redirect_uri` being at `git`, where `temper help server`'s
  gives one, and no test takes a connect's `--web` through to the page's
  meta: a `web` apart from `git` is shown on a kept record alone
  (`temper-service/src/tests/forge_sign_in.rs`). The bullet's figures are
  those from before its bundle and its connect; whether they still hold
  is unknown. No test reaches a host's units, a proxy or a
  redeploy, as the user's request of 2026-11-05 says, and its plan ran no
  walk, so the page's steps on a host stand on these stories alone.
   the ladder's live stories but B0's: `scripts/proof-live.sh` takes
  `rung-b0` alone, on ChatGPT with an owned Forgejo, and
  `bin/temper-box/tests/live.rs` holds no other rung.
   neither GitHub proof has shown it holds on `fabrig-ai` alone:
  `scripts/proof-live-github.sh` and `proof-live-github-managed.sh live`
  stay for the next live walk, the second with `TEMPER_TEST_MANAGED_PERSON`
  set. That sitting's relay admits GraphQL's `VIEWER` alone
  (`bin/temper-box/tests/support/relay_managed.rs`) where the story's
  admits `PULL` on the probe too (`bin/temper-box/tests/support/relay.rs`),
  and an open pull on an organization's repository reads through `PULL`
  (`skein/crates/skein-forge/src/github/plans/reads/pulls.rs`); whether
  a live sitting's engine reads the pull it opens is unknown.
   a live
  story ending at its task's first terminal failure.  the live GitHub
  story connects the probe App without `--read-only`, so its record's
  `read_only` is false (`bin/temper-box/tests/support/live_github.rs`).
   the owned-forge
  landing pair has not passed in the after-landing tier: neither story
  has passed live on its own store. The focused settings proof above
  establishes no real launch with
  `--model claude-haiku-5-5 --effort low` or live landing. That evidence
  remains after-landing work (11): the plan compiles the live story
  without running it, and no worker holds its login.
   the held record's guard is shown on the fake issuer and on stubs
  alone (`bin/temper-box/tests/process.rs`,
  `a_story_panicking_after_its_box_stopped_leaves_the_held_record_at_its_path`,
  and `bin/temper-box/tests/suite/startup.rs`). Its return with no box up
  holds through red-26's repair (660556eab1,
  `a_failed_terminal_return_allows_recovery_without_reopening_import`),
  and that red closed recovered. Nothing offline runs its lines in
  `bin/temper-box/tests/live.rs`, and whether a live store's box, with its
  owned Forgejo, answers the guard within its 240 s is unknown.
   the shell subset;
  skein's fake machine runs no `sh` and no `rg`. 4: of
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
  (`temper/tools/stage-one/src/check.rs`, `subject_key`), and nothing
  tells Ask temper to propose it (`../domain/agent/agents.status.md`, 1).
   `scripts/proof-live.sh`'s `TEMPER_TEST_LIVE_STORE` and
  `TEMPER_TEST_LIVE_KEEP`, `proof-live-github.sh`'s fixtures and
  `proof-live-github-managed.sh`'s target and evidence default under
  `$HOME/src/rust/tmp/`, a scratch layout of whoever runs fabrig's
  development; `live-tier.md` names the first, and
  `bin/temper-box/tests/suite/forge_land.rs` holds it. The user's request
  of 11-04 makes `AGENTS.md`, 1 absolute and says fabrig names nothing
  outside plan directories, but `tests/boundary/tests/outward.rs`
  matches the `~/` form alone, so these pass it. They stay because the
  scripts' behaviour held but for their build wrapper, and no step of
  that request moves them.
   `proof-live-github-managed.py`'s `ssh -L` mark names a machine
  alias of whoever runs fabrig's development, and the outward check
  matches that name only with digits after it. The same request left it
  in place.
   the managed sitting's `fake` and `fake-wide` runs,
  `managed_ready_client`'s fake sitting and `startup.rs`'s
  `the_live_github_configuration_passes_startup_admission` keep
  `fabrig-ai`, where the user's request of 11-04 gives tests on the fake
  forge a neutral owner, because the first are the live sitting's dry
  run, whose owner is a fixed name and never an input, and the second's
  shaper renders the template's `acme` to the owner, so `acme` would
  prove nothing. The relay's own tests take `acme` and `alice`.
   Offline startup admission: no shared helper clears goal projection
  grants. `bin/temper-box/tests/support/live_github.rs`'s
  `configure_engine` clears none, and `github_land.rs`'s, a separate
  function, removes the charters, templates, policies and rules for
  stock policy 1. No reason is recorded.
   the held record's guard takes the goal of the live-login-guard
  draft of 11-04, every exit path exports the record, and departs from
  its mechanism, by the designer's decision: it stands before the import,
  where the draft has it right after, because the import is inside the
  first sitting's referee and one whose reply was lost still owes the
  record; it starts a box of its own where none is up, where the draft
  has its `Drop` run with the box still up, because a panic through a
  sitting kills the box; and a `stat` of the record's path disarms it,
  where the draft has an explicit export, because a record back at its
  path owes nothing whichever return brought it.
