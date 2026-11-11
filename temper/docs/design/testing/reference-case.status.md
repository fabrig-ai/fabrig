# reference-case: status

2026-11-10. Where the tree's code lags or departs from [reference-case.md](reference-case.md).

- **Departs:** the whole document: eighteen named real-loop stories remain in the fuzzy
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

`checks.md`, section 1's fuzzy suite has no known gap against its 180 CPU s, the
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

- **Not yet:**  GitHub's real-loop stories 3, 4, 5, 7 and 8; 1, 2 and 6
  run (`tests/real-loop/tests/suite/github_land.rs`, `github.rs`). 
  the twins' documents among `temper-playground`'s stories: B0's and
  B1's are still the real loop's own fixtures
  (`tests/real-loop/tests/fixtures/b0`, `b1`).  B2's twin
  (`../../development/acceptance.md`, section 8).
  the whole document: the cluster's 4.6 process story is ignored: its fixture
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
  skein's fake machine runs no `sh` and no `rg`.  of
  temper's kinds' stories, the forge's world's (`../domain/forge/README.md`'s status,
  18); the domain world's project's rule refused to another project's
  owner, a project made after the first start covered by the
  deployment's rule, the two requirements removed from the rules and the
  resume fixture restarting with changed values
  (`tests/engine/domain/tests/suite/settings/kinds.rs` tells the rest);
  the fuzzy sweep's changes of landing rules
  (`tests/engine/forge`'s `suite::fuzzy::simulation` makes none); and
  the real loop's smoke of a gate added to a project's rule from the
  client.  the client's How work runs, a proposal's card and its
  review, in their step and view tests and joined with the engine,
  since the pages are not built (`../client/README.status.md`).
- **Departs:** the whole document: the case library's purpose is retired: the reference
  case it was written from is replaced (`../draft/reference-case.md`), and
  its code awaits removal with its design.
   `temper-fake-git/src/exchange.rs`, `decode`, and its
  `src/tests/net.rs`, `decode`, each drive a tokenizer over one whole body
  in a `Held` by hand, with their own step bounds, where
  `skein/docs/design/data/json.md`, 5.3 gives one whole buffer to `parse` or a
  `Feed`, and the user's request of 11-04 asked that no crate under
  `temper/crates` do so. No reason is recorded, and no debt item holds
  them.
