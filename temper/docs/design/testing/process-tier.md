# The process tier

`bin/temper-box/tests/process.rs`, after jig's reference binary's (jig's
`protocol/service.md`, section 8): temper as it ships, as processes.

- **Only what processes show** (skein/docs/foundation/testing-strategy.md, 2.9; the user,
  10-23, for every story): a path through `main` (its arguments,
  configuration, startup checks, printed lines, exit statuses and
  signals), or an integration point only processes have: real git, the
  owned Forgejo and its runner, the binary killed and started again
  (what a restart keeps is `README.md`, 2.5's), the cluster's processes. A behaviour
  a lower tier shows is tested there and not again here: a finding met
  here is reproduced in the lowest tier that shows it
  (skein/docs/foundation/testing-strategy.md, 9), and a story is added here only where its
  Musts depend on processes. A variant grows an existing story, never a
  copy of its setup.

- **One printed link, counted.** A story that holds a box to one sign-in
  link counts the lines of its stdout and never compares their bytes, which
  would write the link's secret into the log. Each line must be a complete
  link ending in a line feed; any other or unfinished line fails too.
  Its panic carries the box's stderr with every line holding `secret=`
  filtered out, so that a second
  link says why it came.

- **Focused wiring,** one short test in the ordinary `suite` binary:
  a shipped `temper-box serve` on a scratch configuration signs in a native
  client, answers one chat through the fake model, drains, and exits 0.
  It does not replay a whole first-run, welcome, settings or account story.
  The shared harness's 22 step checks are registered only by this binary.

- **Lower worlds own the behaviours.** Settings proposals and placed
  holds are jig's core and temper's client-tool tests; two parked tasks
  keeping revision R until the owner's advance is `tests/system`'s
  `running_revision`. Held reviewers' readable budget and owner holds
  survive cold restart in the engine domain's `reviews::retier` table;
  the pre-call reviewer has attempt zero before and after that restart.
  Producer failure and the chat's explanation are `producer_failure`'s;
  task-exhaustion review holds are `reviews`'s. Long questions and closed
  conversations use the client world's cut-question and open-chat stories,
  with the whole goal question and landing in the real-loop fuzzy twin.
  File moves, interrupted writes and lost replies use the CLI machine
  world's `moves` tests; HTTP routes, listener refusal and handover-file
  bounds use the service and host's step tests. The `temper-box` package has
  no QA binary or QA-named test group.

- **Account transfer interrupted by a signal,** one short process story:
  import an account on a fake issuer, interrupt an export while its write
  is pending, stop and restart the binary, and read it usable. It needs
  the OS signal and real binary restart, so remains in this profile;
  the restore and file-handshake decisions are the lower worlds'. It
  needs no Forgejo, change, CI or reviewer.

- **Owned variants share their setup.** Runner cancellation contains the
  main-advanced and mid-proof/restarted turn-landing checks. The owned
  welcome includes program login and key admission. One setup-cut story
  walks fetch, preparation, run and started cuts using one scratch and
  cache, makes one final setup, then restarts across its removal cut. Its
  started cut runs on a box with no sign-in, as init writes one without
  `openssl`: it holds the engine, and the group's source step needs the
  engine to answer its discovery (`../protocol/openid-provider.md`, 9).
  The slot-host story still belongs here: its loop starts the shipped
  owned-slot subprocess and real Forgejo. Its referee polls the marker
  promptly instead of waiting for the fixture shell's 30-second sleep.
  The plan-face landing remains separate as the offline proof of the
  shared live driver's model face (`live-tier.md`).

- **The playground and the set up, in and out,** each a story of its
  own (../protocol/setup/testing.md, 4): a first run into the playground and
  out by its reset, the store then read empty and the welcome at its
  Forge step; a reset the box's death cut, ended whole at the next
  start; a set up killed at each of its steps, found unfinished and
  started over, Forgejo's binaries taken from the cache; and GitHub
  gone on after a restart, on `temper box --fake-backends`' fake
  GitHub. They need the box's processes and their deaths, so they stay
  in this profile.

- **The box's lifetime with its steps running,** two stories of their own
  (../protocol/setup/testing.md, 4): a hangup stopping a box whose playground
  steps run, nothing left, its log's last line `Stopped`, and the same
  configuration binding again; and, on Linux, a box killed taking those
  steps with it. They need the box's processes and their deaths, so they
  run in this profile. A box started ignoring hangups asks for no step: it
  is one path through `main`, and stays in the `suite` binary with the
  other paths of a box's first run
  (`bin/temper-box/tests/suite/box_listeners.rs`).

- **A box restarted with a newer build** (`../protocol/setup/box.md`, 1;
  `../draft/landed-qa.md`, 10), one story on `temper box
  --fake-backends` with its owned Forgejo: the box started from a `bin/`
  of its own, holding copies of `temper-box` and `fake-claude` and a
  bundle's directory; a repository adopted and a task made; the box
  stopped, exit 0, nothing left; each file of `bin/` replaced by a new
  copy, the bundle's page given a mark; the box started again on the same
  directories. ctl signs in with its new link and reads the forge, the
  repository and the task as they were; every process of the box's tree,
  read once the link is printed and again while a run started after the
  restart is live, runs a file of the new `bin/`, by device and inode,
  and none of the old; the served page holds the mark. It needs the
  binaries' files and the processes, so it is this profile's.

- **The upgrade's commands,** through the real binary, whose patch sets
  are empty until the first release: `temper-box versions`' lines; `temper-box
  upgrade --check` over a stopped deployment's copy, saying the restores
  pass, nothing patched, the workers may stay, the copy left as it was;
  `temper-box upgrade --back` refused, with no previous generation, in its
  words; and a start's exit and words when the store refuses. The drill
  with a binary that patches, stop, start the new one, go back, with an
  owned Forgejo's migration, waits for a release to pair with
  (`../draft/upgrades.md`, section 13); the one-process world runs its
  steps on fixture sets until then (`README.md`, 2.5).

- **What runs:**
  - the real `temper-box` binary: `temper-box forge prepare`, `temper-box forge run`
    for Forgejo and for its runner, then `temper-box serve`, on a scratch
    directory, with a configuration the test writes from the owned
    example (../protocol/setup/init.md, 1);
  - skein's fake LLM peer, a process of its own on loopback, over TLS
    with a test root the agent's configuration trusts, scripted for the
    stories, accepting a key the test makes; the owned example's LLM
    providers' endpoints are written at its address, and Claude Code's
    path at smith's fake `claude` (`fakes.md`, 1);
  - an owned Forgejo and its runner (below);
  - real git, the machine's, at the absolute path configuration gives it
    (../protocol/configuration/README.md, section 3; ../protocol/git.md,
    section 3);
  - the person: skein's fake person in front of temper's client, over
    jig's native transport (../client/README.md, section 2); and, in a
    test of its own, `temper-box open` itself with stdin and stdout piped
    (../client/line.md and ../client/testing.md).
- **Its Forgejo is owned,** provisioned by the setup's commands
  (../protocol/setup/testing.md, 4): the configuration's one forge is owned
  (../protocol/setup/init.md), its binaries those `TEMPER_TEST_FORGEJO` and
  `TEMPER_TEST_FORGEJO_RUNNER` name, which the commands check against temper's
  pins (../protocol/setup/owned.md, 1); its address, the clients' listener and
  the webhooks' listener are loopback addresses of the test's own. The
  test runs the commands itself, in the order `temper box` would
  (../protocol/setup/box.md, 1), refuses to run without either variable, naming
  it, and stops every process it started after. At `temper-box serve`'s start
  the setup host's `Reconcile` plan makes temper's user, its tokens, its id, the org's team and its hook; the test, as the
  person, makes the repository in the setup's org through Forgejo's API,
  with Actions and a workflow whose job runs the repository's check on the
  runner. Never an existing Forgejo.
- **CI on, webhooks on.** The repository is adopted with "It runs CI"
  checked, and the project's landing rule asks for CI and for no gate or
  approval (`"ci": true`, `"gates": []`, `"approvals": []`;
  ../protocol/configuration/engine.md): the change lands once its job,
  run by the owned runner, has passed on its head. The webhooks' listener
  is configured, and the org's hook reaches it.
- **The story,** one test:
  1. the person signs in by the link the binary prints, and adds the
     test's key as the deployment's on the accounts page
     (../client/README.md, 3.6), which shows its account usable (`real-loop.md`);
  2. a chat is answered;
  3. a repository is adopted through temper's client, as above;
  4. a goal is given whose change, its CI passed, lands on the owned
     Forgejo, pushed by real git;
  5. the binary crashes: its whole tree is killed at once, as a
     supervisor ends a crashed service's (skein's `runtime/shell.md`, 12), and
     once nothing of it runs, it is started again on the same directory;
  6. the person finds the work, and the change's branch on the forge is
     at the run's commit.

  The person reads the chat that gives the goal, finds the goal on the
  board and its plan on its page, and finds the same after the restart.
  The forge's state is the story's proof of the change, read through
  Forgejo's API.
  This same story raises the keep's polling intervals and the change's
  stall past its patience, with a slow CI job, so it also proves that
  deliveries carry the job's end (../protocol/setup/testing.md, 4).

- **The build it runs in.** The tier runs the binary its build made,
  with or without its browser (`TEMPER_INTERNAL_BUNDLE`; ../client/web/README.md,
  2.6), and passes in both: where a start with `clients.bundle` `null`
  shows the browser (init's line for no browser, the page at `/`), the
  test expects what the binary holds, as `temper_box_shell::bundle::OWN` says,
  never one build's answer alone.
- **Its endings.** The landing story's crash
  proves what a crash of every process keeps on disk, the page cache
  surviving it, not what a power loss does, which the one-process world
  draws (`README.md`, 2.5). Every end outside a named crash, cancellation or setup cut, is a deployment's healthy stop
  (skein's `runtime/shell.md`, 12): the test sends `Terminate`, the engine
  drains (`../domain/root.md`, 8.2), the service closes what it started, and the
  binary exits 0 with none of its tree running. A process left at that
  exit is temper's failure, its shutdown's; one left by the crash's kill
  is skein's test kit's (skein's `examples.md`, 6). Neither is relaxed.
- **The terminal,** a second test, on a scratch directory of its own:
  `temper-box open DIRECTORY LINK` with stdin and stdout piped signs in by
  the link `temper-box serve` printed, prints the inbox, adopts a repository
  as above by commands (the form's fields written with `n words`, then
  Adopt pressed with `n`), and exits 0 on `q` after a drained close. Its
  referee reads the printed pages (the inbox and the adoption's
  outcome) and the exit status.
- **The agents' tools,** `temper ctl` against `temper-box serve`, on a
  scratch directory of their own: ../client/ctl/testing.md, 4. A call cut
  by `SIGINT` or `SIGTERM` sends nothing more, prints nothing more,
  leaves DIRECTORY's kept requests as they are, and exits 130 or 143
  (../client/ctl/README.md, 6.2).
- **`temper box`,** a fifth test, on a scratch directory of its own
  (../protocol/setup/testing.md, 4): `temper box` on the owned example prints the
  sign-in link on its stdout; Forgejo killed is restarted by the box, with
  the engine still serving; a stop leaves no process the box started.
- **The first owned repository,** one story on a fresh `temper box`:
  setup makes and adopts `work/hello`; the owner's CSV task pushes a
  change with real git. The owned runner first fails because the release
  note is absent. Its output reaches the repair's brief; the repair adds
  that note, CI passes at the repaired head, and a blocking agent review
  approves that same head before landing. A second box finds the same
  task, head and merge without replaying setup or making another task.
  Its fresh admin handover is consumed and revokes an independent older
  admin oracle. No credential is borrowed or printed. The fixture sets
  the stock charters' parking intervals to one millisecond: parking time
  is tested with simulated clocks, not by sleeping through it here.

- **Forgejo signing in through temper,** a test of its own at the end of
  `bin/temper-box/tests/process.rs`, on `support/forgejo_sign_in.rs`
  (../protocol/openid-provider.md, 8): an owned Forgejo prepared by temper's
  commands with no group's file (../protocol/setup/box.md, 4), with 9.4's
  settings in its `app.ini`, beside `temper-box serve` with jig's provider
  and a bundle; the key and the client's secret made by `temper-box
  forge sign-in` in the test's scratch directory, and the temper source
  added by the command it prints. A scripted client with a cookie jar
  signs in to temper by the printed link (the member of Google's
  provider, `README.md`, 2.3), opens `/user/oauth2/temper` on Forgejo, follows
  temper's authorization (its `Authorize` sent by jig's codec, as the
  page sends it) back to Forgejo's callback, and Forgejo's API, by its
  admin, then names the member's account by their login, their address
  and its external id, `sub` (`3:sam@example.com`, Google's provider and
  the address), which only Google's provider gives, so the link
  provider's identity cannot pass. The forge is then connected as its
  admin by the handover file, as `temper help server`'s walk does, and
  the page answers with its forge's meta (../protocol/openid-provider.md, 7).
  It aims at 2 s, as a world's story
  does (`checks.md`, 1), and takes about 3 s whole: about 2.5 s is the owned
  Forgejo's preparation and start, as every owned-Forgejo story pays,
  and about 0.5 s is temper's part; being past its aim, it waits on the
  user's call with these figures. It shares no start: nextest runs each
  test in a process of its own.
- **The throwaway Forgejo** is the one in skein's `testing/fakes/forge.md`,
  section 6, never an existing Forgejo: temper's `temper-git-transcripts`
  records real git against it (../protocol/git.md, section 9). temper's
  users give it the binaries `TEMPER_TEST_FORGEJO` and
  `TEMPER_TEST_FORGEJO_RUNNER` name, and refuse to run without
  `TEMPER_TEST_FORGEJO` (and, for a story with CI, `TEMPER_TEST_FORGEJO_RUNNER`),
  naming it.
- **Its referee:** the person's script finishes within its deadline; the
  binary does not exit before the test stops or crashes it, or its
  stderr is shown; a stopped binary exits 0; the forge's state, read
  through Forgejo's API, shows the change; nothing the person saw before
  the crash is lost after it; no process the binary or the test started
  outlives it: none runs at a stopped binary's exit, and none of the
  crashed tree when the binary starts again.
- **Credentials:** none borrowed. The fake LLM peer accepts a key the test
  makes. The person adds it from the client once signed in (`real-loop.md`),
  as the deployment's, and the agents' tools' test with `temper ctl
  account add-key` (../client/ctl/testing.md, 4); it is then kept in jig's
  secrets' store, and no file the configuration names holds an LLM account
  (../protocol/credentials.md, section 3). The forge's API token, its user
  and the git identity's token are records the setup host's plans write
  in jig's secrets' store, and the admin's token the preparation's
  handover, `admin_token`, which the setup host takes once
  (../protocol/setup/README.md, sections 3 and 5).
- **A person's change on the owned forge, signed in with ChatGPT,** the offline proof of the live driver (`bin/temper-box/tests/support/forge_land.rs`): "A
  Forgejo set up from the client" (above), its set-up, first repository
  and landing reused, carried on with ChatGPT's sign-in as above, a
  chat on the plan, and a restart from a fresh client. The module writes
  the story once, its model side a face: the fake issuer and the fake
  LLM peer here, ChatGPT itself in the live tier (`live-tier.md`), which reuses
  every step and check but the face's:
  - *the box and its forge:* `temper-box init --box` into three empty private
    directories, the sign-in's and the LLM endpoint's addresses swapped
    for the fakes', then `temper box` on them, which reads that
    document; the owned forge set up from the client and `work/hello` made and
    adopted into project 1, by that story's steps;
  - *the change:* the owner connects ChatGPT (`account connect 1`,
    `account list --until settled`) and opens a chat on gpt-6-luna in
    project 1 asking for one README line, unique to the run. The
    chat's agent calls `change`, naming `work/hello`, and the root
    delegates the change procedure on it (../domain/agent/tools.md, 1; ../domain/forge/change/README.md
    ); the coder edits on the plan's grant, its
    pull's starter workflow passes on the owned runner, the change
    lands with no review, since init's landing rule names no gate, and
    the chat's reply, its turn's text with no `finish` (../domain/agent/charter.md, 5),
    follows the procedure's result. The peer observes a bearer on every
    request, the coder's too;
  - *the restart:* the box stops, exit 0, with nothing left, and a
    second box starts on the same directories. A fresh client root,
    signed in by the new printed link, reads the same chat and its
    reply, and the change's task ended landed with the same pull and
    merge commit, and no task beyond the chat, its change and its
    producer. Close ends the chat with its closing turn, and no task of
    the project is left live;
  - *its referee:* temper's reads, with no credential, give the
    landing, its pull, its merge commit and its checks at the pull's
    head, in both boxes alike, so nothing was pushed or merged twice;
    the forge's own state is checked as that story checks it. The
    peer's requests number the script's, each answered, the closing
    turn's included.
- **The cluster,** its own tests (../protocol/cluster.md, 6), on
  temper's fakes as processes, never on Forgejo:
  `bin/temper-box/tests/fixtures/cluster/`, the process tier's cluster
  fixture (../protocol/cluster.md, 5), read by
  `bin/temper-box/tests/support/cluster.rs`, which starts the fake forge,
  the fake LLM, the engine alone and two workers alone as processes,
  each in a cgroup of its own, awaits their ready lines and stops them
  in reverse. Its stories: a task on each worker, on the box's binary
  and on the deployment's binaries (../protocol/cluster.md, 6); a
  worker lost, killed mid-run; a worker hung, stopped past the grace and
  continued; a worker started later, and one restarted; and a mixed
  document refused, exit 1. Each story's referee is the tier's: what the
  engine records, read through the client (the forge's state, from the
  fake forge's API, once
  it joins: ../protocol/cluster.md, 5), and no process of the cluster
  outliving it. Without delegated cgroups the cluster's tests fail at
  their start, saying why (`README.md`, section 6). These remain in the process
  profile because SIGKILL/SIGSTOP, parent death, delegated cgroups and
  process exit order are their Musts. The case library's `fakes.md`, 1/`fakes.md`, 3
  placement, `fakes.md`, 4 pace and `fakes.md`, 5 account choices run only in the real-loop
  fuzzy stories (`reference-case.md`).
- **Two drivers' offline proofs** (the user's request `ignored-tiers`,
  2026-11-08). Each test sets up what its script sets up, and the
  script runs the test:
  - *B0's,* `bin/temper-box/tests/proof_live.rs`: the rung's setup through
    its last read before the account's import, on the fake model with no
    account, and its whole walk on its twin, the driver the live rung
    runs (`live-tier.md`). Real Forgejo, as the stories above.
  - *The managed GitHub sitting's Ready,*
    `bin/temper-box/tests/live_github_managed.rs`'s `managed_ready_client`:
    the sitting's fake half as `scripts/proof-live-github-managed.sh fake`
    sets it up. That is the fake GitHub with the sitting's accounts and
    its two repositories, the sitting's relay in its fake mode, and the
    engine alone on `tests/support/github/engine.json`. Ready, the web
    client's domain, view and transport run natively, signs in by the
    printed link. Connect makes the App by its manifest and installs it
    on the one repository, and Ready adopts that repository. The
    sitting's driver runs the same test as its client, naming its mode
    in `TEMPER_TEST_MANAGED_CLIENT` and giving the link on stdin. The test
    ends as the driver's fake mode does: the engine and the fake GitHub
    stopped, the relay's scan finds no secret bytes in its log or its
    record. The push, the planted refusals, the cleanup, `fake-wide`'s
    refusal, and the App key, JWT and installation token the driver
    reads for its push and adds to that scan, stay the driver's.

  The profile selects these by name beside the `process` binary
  (`.config/nextest.toml`), so each binary's other tests stay in the
  suites that hold them.

- **Outside the gate suites:** nextest profile `process-temper`, run
  once the binary and the fake `claude` beside it are built. The 6-minute aim is the starting point for
  the process tier's CPU budget, the user's to set from measurements,
  separate from its wall hard limit. Both follow skein/docs/foundation/testing-strategy.md,
  8.1 and its three-question repair rule: a story moves only if what it
  checks belongs in another tier.

  The suite CPU budget is 540 CPU seconds, the 6-minute aim raised by
  half by the user's word of 11-08, for now, to be revisited; its wall
  hard limit belongs to `tools/tiers/budgets.toml`. Harness waits use
  `Patience` or skein-world's `Deadline`; product `ctl --timeout` is not
  their bound. Starved waits pass with their figures after cleanup
  (skein/docs/foundation/testing-strategy.md, 2.10). The owned preparation command watches its
  child with a deadline; the ctl referee reports overdue work to the
  world's starvation judge.

- **Three at a time:** the profile runs three tests at once, a story being
  one thread and one loop as ever, and the tier's budgets stand. A story
  shares its machine. A listener it holds from its bind to its last use
  may be on the kernel's port. A port it lets go of and listens on again,
  as a retained deployment's across a stop, is `temper_fixtures::ports`'
  (`support::free_addresses`, `support::own_listener_files`), and a fake
  box's are the story's too, through `TEMPER_INTERNAL_FAKE_BOX_PORTS`
  (`../protocol/setup/box.md`, 5). Such a port is:
  - **below the kernel's ephemeral range,** as
    `/proc/sys/net/ipv4/ip_local_port_range` reads, so that no outgoing
    socket and no bind of port 0 lands on it while nothing listens there;
  - **in a block its process holds,** by a listener on the block's first
    port from its first draw until it ends, so that no other test process
    on the machine draws it, of any run, tier or checkout, and an ended
    process's blocks are free at once, however it ended.

  The chooser starts at port 10000 and claims blocks of 64, keeping the
  first port's listener for the process's life. It skips occupied ports
  and those its caller excludes, and never hands out a port twice. Fewer
  than two blocks below the ephemeral range, or no free block, fails
  with the range named. The callback port 1455 remains outside these
  blocks and keeps its named test group.

  A story that can't share one runs in a named nextest test group, as
  `temper-real-loop` does, the reason beside the group.

- **The fake box keeps two walks** (the user, 2026-11-01): fake-based
  coverage on `temper box --fake-backends` is one short path through
  `main` on its fakes, not a story per feature
  (`bin/temper-box/tests/fake_backends/walk`):
  - `here_key::forgejo_here_and_a_key_reach_a_first_chat_on_the_fake_box`,
    the owned welcome walk (`README.md`, section 2, above), the one short end-to-end
    test;
  - `sign_in::a_chatgpt_browser_tab_on_the_fake_box_leaves_ready_able_to_create`,
    the plan's path: the box starts the fake issuer as a process, and the
    `--fake-backends` journeys stand on it.

  What any other walk checked is the world's that owns it, by what it
  checks, never by its cost (skein/docs/foundation/testing-strategy.md, 8.1), and none is the
  browser tier's (`../client/web/testing.md`). A walk's quiet wait,
  a sitting, a poll or a hold, is the lower world's clock there, never a
  wait. Each story below is the walk's lower version, keeping what the
  walk proved with that world's real parts; a keep a world can't hold
  without a stub stays on the box with its reason in the status:
  - **Home's brief on a person's plan,** in the joint world (`checks.md`, 3),
    `tests/client`'s `suite::status::a_plans_project_has_its_brief_on_a_return_past_the_sitting`:
    the person's ChatGPT plan is the only account and no deployment key
    is kept; Home before a landing and within the sitting after it shows
    no brief; past the sitting, the status run the plan pays for is
    admitted and its brief shown; the plan is usable after. It runs on
    the stock's budgets, whose period funds the brief beyond the owner's
    pool (`../domain/forge/attention.md`, 2; `../protocol/configuration/stock.md`), and fails
    without that funding.
  - **Home's landing count across restarts,** `suite::status::a_restart_keeps_the_rows`:
    a real landing counted, Skip pressed before it, and after the
    engine's restart a fresh client reads the count and the skip kept.
  - **Away,** `suite::away::a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off`,
    in this order: Away on from Home, with its manual end; a goal decides
    a question; Decided carries on, the goal's next call its note; Home's
    Overturn asks the goal again, which shows "Still needs you"; Away off
    and the goal's next question waits for the person, answered from
    Home.
  - **A chat's goal and its report,** `suite::task::a_chats_goal_shows_its_report_from_its_goals_row`:
    the chat's run starts a goal, the goal's run writes its report and
    finishes, and the person goes from Home's Goals link to the Goals
    row and from it to the report.
  - **A sign-in's usable model,** `suite::connect_model::a_held_welcome_sign_in_moves_on_once_when_the_watch_makes_its_account`:
    the account the sign-in makes reads usable from the engine, where
    before it read not.
  - **The fake issuer's held sockets,** in the real loop (`real-loop.md`),
    `tests/real-loop`'s `suite::sign_in::a_sign_in_beside_a_browsers_idle_issuer_sockets_completes_and_past_the_cap_is_turned_away`,
    on the issuer `temper_fake::oauth::Document::peer` makes: a browser's
    held authorize socket and an idle spare don't block the engine's
    sign-in, and past `CONNECTIONS` the issuer turns a connection away;
    and in this package's `suite::fake::the_oauth_fake_logs_each_connection_it_turns_away_at_its_cap`,
    beside the fake's other process stories, `temper-box fake oauth` logging
    each one. Each fails without its half of the fix.
  - **A long document built in parts,** in the real loop,
    `suite::notes::a_long_document_is_built_in_parts_and_revised_by_a_fresh_run`:
    a hosted agent's bounded `note` calls build it past one call's size,
    its stale and past-the-end edits are refused and save nothing, and a
    fresh run finds it in its brief and revises it by lines. Beside it,
    `suite::notes::a_chats_document_is_revised_whole_after_a_follow_up`:
    the person's follow-up message reaches the chat's run, which
    rewrites its note whole.
  - **Copies made and adopted,** in the one-process world (`README.md`, 2.5), whose
    service runs the setup's copies (`../domain/forge/adoption.md`, 5), as the joint world
    does not: `suite::ready_copies::readys_two_copies_are_made_in_order_and_adopted_with_the_first_as_home`,
    the person's press of Ready sending both copies in one client pass,
    made one at a time in that order and adopted, the first as home; and
    `suite::copy::a_copy_beside_a_repository_joins_its_project_keeping_the_sources_workflow`,
    `repository copy` into a project that holds a repository: the copy
    joins it owned and not home, one commit above its source with the
    starter workflow, the source's own workflow kept.

- **A walk wakes on what it waits for:** a fake-box walk waits on an
  event where one exists, such as ctl's `--until`, and where only the
  forge's poll brings the news, it runs on short paces
  (`forge.client.poll_ms` and its fellows), so that no walk waits out a
  5 s poll. A wait a person meets too is a finding of the product's,
  repaired there (skein/docs/foundation/testing-strategy.md, 9).
