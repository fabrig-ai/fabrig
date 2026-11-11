# fakes: status

2026-11-10. Where the tree's code lags or departs from [fakes.md](fakes.md).

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
  3: the shell subset;
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
- **Departs:**  the case library's purpose is retired: the reference
  case it was written from is replaced (`../draft/reference-case.md`), and
  its code awaits removal with its design.
  3: `temper-fake-git/src/exchange.rs`, `decode`, and its
  `src/tests/net.rs`, `decode`, each drive a tokenizer over one whole body
  in a `Held` by hand, with their own step bounds, where
  `skein/docs/design/data/json.md`, 5.3 gives one whole buffer to `parse` or a
  `Feed`, and the user's request of 11-04 asked that no crate under
  `temper/crates` do so. No reason is recorded, and no debt item holds
  them.
