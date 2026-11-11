# process-tier: status

2026-11-10. Where the tree's code lags or departs from [process-tier.md](process-tier.md).

- **Not yet:** the whole document: the hangup/rebind and Linux killed-box stories in
  `bin/temper-box/tests/suite/box_lifetime.rs` are registered in the
  process binary and compiled, but have no runtime result in that tier
  from `limits-heavy`. Both passed in focused tests before moving whole.
  Their original CPU medians transfer about 15.9 CPU s to a tier whose
  current margin is unknown; neither a current tier total nor its fit
  within 540 CPU s is established by the instruction account.

- **Departs:** the whole document: `temper-fixtures/src/ports.rs`, `ephemeral_first`,
  uses 49152 when `/proc/sys/net/ipv4/ip_local_port_range` is absent,
  rather than reading macOS's configured range. The plan chose its
  default because macOS has no Linux range file. A lower configured
  boundary can overlap the chosen ports; reading it remains later (11).

`checks.md`, sections 3 and 4 have the owned-Forgejo box's linked-person stories:
inventory and owner-role acceptance at one approval, refusal at two, clearing
on a second owner after restart, then a change waiting under Needs you,
another project's party reading none, and the person's review landing it.
The joined client story draws the card and files-page link and checks its
removal after the review lands the change. The connector removes review
facts on `Committed` for `Write::Merge`. The no-link cases keep the stock
permission mapping, owner-role refusal after admin adoption and fresh
maintainer-role acceptance. These are scripted worlds on fake Forgejo.

- **Not yet:** the whole document: the supplied inventory has no result for any of
  `ui-restart-red`'s five repaired process stories
  (`../protocol/setup/testing.status.md`, 4). The stock-purse regression and
  late-handover tests are present, and the started-cut fixture uses
  init's no-sign-in configuration while it holds the engine; none is a
  recorded pass of the five stories.

`process-tier.md`'s *The fake box keeps two walks* has no known gap in its
stories: the box keeps `here_key` and the browser tab, and each lower
story the bullet names is present in its world, holding its walk's keeps
on that world's clock. Home's plan brief and its restart story, Away's
sequence, the chat's goal report and the usable account are the joint
world's; the held issuer sockets and both documents are the real loop's;
both copy stories are the one-process world's; the logged turn-away is
the fake's own process story.

- **Not yet:** the whole document: no passing whole run is recorded for the three-test
  profile with the one fixed-callback walk serialized. Its proof belongs
  to the next after-landing run on main (11), per the request
  `process-threads` of 2026-11-06; the profile and its callback group are
  configured in `.config/nextest.toml`.
  the whole document: the tier's CPU against its 540 s, the 360 raised by half by the
  user's word of 11-08, for now, to be revisited: the seven whole runs in
  `test-speed`'s figures at `b23901eab7`, at differing loads, read 287.1
  to 433.7 s, four of them over the 360 (367.8 to 433.7 s), and they hold
  no run with that plan's parts together. The request
  `walks-lower` put the box's twelve walks at about 81 of a run's 340 CPU s
  by 2026-11-06's medians; ten have gone to lower worlds, and no run
  records the tier without them, nor the two that stay, which a plan
  compiles and does not run (11). The budget is the user's.
  the whole document: `here_key`'s first chat runs on the stock charter, whose 10 s wait
  holds its single worker (`../domain/agent/charter.md`, 5) as it holds a person's; the
  browser tab starts no chat. No figure says what that wait costs the
  walk now.
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
  the whole document: the
  playground's stories, in and out and a reset the box's death cut, and
  GitHub gone on after a restart, as `../protocol/setup/README.status.md` records
  them.
  the whole document: Forgejo signing in through temper: the story's connect gives no
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
- **Not yet:** the whole document: the managed sitting's driver lags its test.
  `scripts/proof-live-github-managed.py`'s `configure` still renders
  `policies` from `projects`, and its `connect_fake` runs
  `forge connect 1 github` and makes no project, where
  `managed_sitting.rs` renders none, runs `forge connect github` and
  makes the project `Sitting`. The test's client
  role has had no run under the driver since the test took its own
  sitting, so whether the driver's `fake` run, and its `live` run's
  engine, still reach the adoption is unknown until the owner's next
  dry run (11).
