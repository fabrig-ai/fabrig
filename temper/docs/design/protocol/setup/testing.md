# Testing the setup

The tiers, their budgets, the fakes and the layout are `../../testing/README.md`'s;
this section names what checks this document's contract, and what it
relies on.

## 1. Step tests

- **Forgejo's and the runner's files:** `app.ini` and the runner's
  configuration for each setup, as golden text.
- **The preparation's decisions:** a first preparation; one at the same
  versions; an upgrade, dumped before migrated; a newer recorded version
  refused; a runner added, under its lock, and one removed; another
  binary refused; a missing `admin_token` made again, under a fresh name;
  a preparation cut short made again from an empty layout; Forgejo's
  files with no `state` refused; a later preparation taking Forgejo's
  lock alone when the runner configuration is unchanged; a changed
  capacity at the same versions rewrites the runner's YAML while keeping
  its registration and secrets; `register` run only where the row may
  not exist (`owned.md`, 3), so that what a runner declared stays over a later
  preparation, an upgrade and a runner added again; a box's later
  preparation pointing its source at the current address, and going on
  whatever either command answers, with neither command at a first
  preparation or without `owned.sign_in`. The shell checks
  that a live runner's
  lock permits identical YAML to remain and refuses changed YAML until
  the runner stops.
- **The specs:** each command's arguments and environment, and no secret
  of the preparation's in any argument.
- **The plans' decisions** from answers are skein's (skein's
  `forge/setup/README.md`, section 5). What is temper's is the mapping of
  `reconcile.md`, in temper-forge-setup: each member of a managed setup and
  of an owned one becomes its value, the word `temper` and the names
  `temper-api` and `temper-git` among them, and each `Place` the kit's
  check refuses is named by its path.
- **The configuration:** each bound of `init.md`'s machine part
  refused, naming its path; a forge's setup entry made from the machine's
  part by its kind; the kinds' steps with and without each part.
- **The playground's fakes,** in temper-fake: each bound of
  `application.demo` refused, naming its path; the fakes' configurations
  and seeds from the default document; the fake kind's group.
- **The box's machine:** the order; a restart after its once steps; the
  backoff and the cap; the stop's order, and `temper-box serve`'s second
  signal; a link step started on its long step's first line, given it
  alone, once per box, and given a restart's first line when the long
  step exited before writing one; a stop, a give up, a group's stop and its
  failure withdrawing a spawn not yet asked for, once, a second termination
  ordering no second withdraw; a spawn asked for waited on in its place,
  its late child stopped, or closed at once after a second termination,
  never readied, continued, recorded or restarted, and its late failure
  ending it as a stop's.
- **The box's lifetime:** in skein-shell, a `SIGHUP` read as `Hangup` when
  the service asks, and left alone when ignored at the start. In
  temper-box, a spawn's arguments with and without the box's id; a hangup
  starting the stop's order; a hangup during a stop closing nothing; a
  second termination after a hangup closing them. On the shipped binary,
  in `bin/temper-box/tests/suite/box_listeners/lifetime.rs`, a box started
  ignoring hangups runs on: one focused path through `main`. The running
  steps' process boundaries are checked in 13.4.
- **The fetch's machine** (`owned.md`, 6), in temper-fetch, and in
  temper-forge-setup where a forge's setup gives its pins, given its URLs
  and digests: each pin's URL and digest for each architecture, and
  another architecture refused; another status or a redirect refused;
  a body past `setup.fetch_bytes` refused, and one past its size; the
  lines in order for an answer with a size and for one without. With
  its host, temper-fetch-shell's, in temper-forge-setup-shell's tests,
  which hold its local file server over TLS, its root skein's test root named
  by `SSL_CERT_FILE`: a wrong
  digest, its part file removed and its refusal naming the URL and both
  digests; the right one, made 0o755 and renamed; a file already there
  hashed, nothing asked when it matches, and a mismatch refused, naming
  its path, the file left; a part file left by a fetch cut short
  written again; the lines reaching the host's writer in order, with a
  size and without, none for a file already there.
- **Claude Code's pin and fetch** (`owned.md`, 6): in temper-fetch, each
  platform's digest and size against `owned.md`, 6's table, and each pin's URL. At
  its host, temper-fetch-shell's, in the same suite on the same file
  server: a missing build
  fetched once, then found with no request; a wrong digest refused and nothing left; a file
  of another size refused and left; two commands at once making one
  request. No test reaches `downloads.claude.ai` (`README.md`, section 9).
- **Claude Code's offer and launcher** (`owned.md`, 6, `init.md`, 2, 3), on the shipped
  binary with an empty environment, in `bin/temper-box/tests/suite/startup.rs`:
  a fresh box's Model step reads ChatGPT's and Claude's cards, its
  program's path `"fetched"`; `temper-box program-login` and the launcher's
  own link reach a build placed in the cache, its arguments unchanged,
  `DISABLE_AUTOUPDATER=1` in its environment and stdout the build's
  alone; `temper-box serve`, a worker alone and `temper-box program-login` write
  both links for a fetched program, and neither for a found `claude`.
  `temper-agent`'s own link reaches the build the same way, in its
  package's suite (`../binaries.md`, 3); the substitution's cases are
  temper-launcher's.
- **A fetch's lines** (`box.md`, 2), in temper-box: each relayed as a `fetched`
  line for the group the box is starting, in its log and on no terminal,
  with `--echo steps` and in a failure's tail too; one from a group
  started already named and dropped, one it cannot read named; one left
  in the pipe after the step's exit going no further; the longest line
  within 64 bytes; and a stalled pipe keeping the latest alone, every
  reply still written.
- **The box's asked groups and its pipe** (`box.md`, 1), in temper-box: a
  dormant group running nothing at the box's start; `start FORGE`
  running the group's steps in order, a `step` line for each, then the
  record written and `started FORGE`; a step's failure answering `failed
  FORGE WHY`, the group dormant again and the others running on; a
  second `start` while one runs answered busy, a re-ask of the starting
  forge answered by its own lines, an ask while stopping, a record not
  written, a forge with no group `not asked`, and a started one
  `started`; a kind's group bound to the forge it is asked for, its
  file and its record written; at a later start, a group with its
  record run in place as an always group, one without it dormant, and a
  second started group of one kind not started, named; `remove FORGE`
  stopping the group, running its remover and removing its file and
  record, answered `removed FORGE`, on the real ring too, and a removal
  that fails answered `failed`; a line past `box.line_bytes`
  refused; `temper-box serve` restarted with a pipe afresh, an answer meant
  for the exited one dropped; a restart held unsent and then a stop
  launching nothing; a held spawn's late start and late failure during a
  box's stop, and a late start during a group's stop, the box ending with
  nothing open.
- **The box's terminal and logs** (`box.md`, 2), in temper-box, its clock the
  test's: given one pipe for its stdout and its stderr, serve's link
  written in pieces between other steps' lines reads whole there; the
  sign-in block's exact bytes before any step's spawn and nothing else on
  either stream, and a later link's blank line and link; each link's bytes
  on a terminal and on a pipe, the widest whole; a step's lines in
  its log and not on the terminal, and on both with `--echo steps`; no file
  holding a link; the secret on serve's first spawn's stdin alone and in no
  spawn's arguments or environment, and a restart's spawn with no stdin;
  `box.log`'s lines for a first run, its logs' place, a group's start,
  `started` and `stopped`, by its label, and a stop, with `Stopped` its
  last; the terminal's lines for a group's `failed`, a restart and a give
  up; a failure's tail, its lines and its log's path; a log moved to `.1` at `box.log_bytes`, and the last run's
  moved to `logs.previous/` at a start; a log not written named once,
  its step's lines then on the terminal.
  The link/ordering cases include separate held write acknowledgements,
  full output and policy queues with acknowledgement progress, first/fresh
  and restarted-serve blocks, UTC midnight rollover, bounded EOF fragments,
  exact/over-bound URLs with recovery, and the legacy first-line handoff.
  One default real-loop story runs the real box Service and synthetic
  children on one real ring, making split writes to a shared stdout/stderr
  pipe with interposed diagnostics. It observes complete ordered blocks,
  then asks for stop and drains all owned resources; it uses no backend,
  download or sleep. Exhaustive saturation cases stay in simulated tests.
  Pure policy tests pin group identities and structured failure reasons,
  exact retry delays, window reset and give up at decision time. Formatter
  tests use fixed wall values and supplied labels, including labels whose
  bytes cannot be inferred from arguments. Host tests pin labelled group
  start, started, failed/retry and stopped against unchanged numeric pipe
  replies, intentional-stop exits, repeated asks, failure teardown ordering,
  and no duplicate final give up. A failed requested group leaves serve
  usable. Held output acknowledgements preserve facts' source wall values
  and the complete sign-in block.
- **The box's TUI in front** (`box.md`, 6): in temper-box, the box's lines' bytes,
  their bound and their cut at a UTF-8 boundary, and the lines a fact and a
  terminal's line give; on the simulated host, the front spawned before any
  step with its descriptors, its arguments and environment, the local
  link's URL alone on its fd 3, both secrets on serve's first stdin,
  nothing on the terminal while it runs, the block's words and the
  terminal's lines in `box.log` and no link there, its lines on fd 4 and
  their bound, its log and tail, its end as the stop, the stop and the give
  up with it, and the terminal's lines after it; in temper, the front's
  condition and `--no-tui`; and one focused story in the `suite` binary, a
  fresh box on a pseudo-terminal whose front signs in and whose Ctrl+C
  stops the box, which exits 0 with no link in any log.
- **The setup host's handover at runtime** (`reconcile.md`, 1), in temper-service's
  step tests: at `started`, `TakeOver` when the handover is present, its
  record kept before removal, then `Reconcile` and the connect, the row
  "Connecting" between; a stop or failed removal leaving it "Set up
  failed", named by its step; `failed` naming the box's step and
  pointing to its output for the rest; with no pipe, `SetUp` of an
  owned forge refused *not under the box* (`owned.md`, 5).
- **The setup host's close** (`reconcile.md`, 1), in the existing temper-service
  setup step fixture: notes full and never popped, host outputs full
  before close is requested, and `Closed` behind the active notes,
  store, forge and events gates. The close request is retained, host
  `Close` is sent once when its bounded room permits, and only the
  actual host/socket terminal events and io settlement complete it.
  Events and counts establish the order, with no sleeps, new harness,
  unbounded queue or timeout change.
- **The playground's start and reset** (`playground.md`), in the engine's domain
  world (`tests/engine/domain`) and the forge connector's tests: no
  row, none; a start making the fake forge's record and the row,
  starting, and asking for its group in one decision, the box link
  holding it while another group starts; once the forge connects, the
  account added unless the deployment holds one of the provider, the
  value left unused otherwise, the project made, the account's uses
  naming it alone, and `playground/widget` adopted; the answer,
  running, once all are done, and start failed at the first that fails,
  the forge not connected by `setup.deadline_ms` among them; a second
  start while running answered the same; the same key answered from
  memory, and unknown after a restart; a start refused *not offered*
  while a real forge is kept, or while the row resets; a start a
  restart cut short reset at the next start; the reset deleting the
  project, then removing the forge, then the account, then erasing the
  row; an act not granted kept, named with its grant, and sent again at
  the next start; a reset cut after each act ending whole at the next
  start; and in temper-service's step tests, a start carrying
  `llm.key`'s bytes.
- **The fake's start-time adoption** (`playground.md`), in temper-service's step
  tests: after handover and reconcile, one `Adopt` for each seeded
  repository not already restored, in order, with the deployment's
  listener when configured;
  the admin token and webhooks' secret answered from records, and the
  connect's access built from records; a stop named by its step.
  `temper-init`'s step tests check the document's listener, and the
  binary's crate tests the five ports distinct.

## 2. Reconcile's world

The world is skein's (`forge/setup/README.md`, section 5): the plans on the
fake forge's protocol layer, serving the setup's routes, and the host
component's own waits and bounds. What is temper's is the engine's
routing of the setup host, checked in `temper-service`'s own tests
(`../forge/README.md`, section 9) and by the process tier (4):

- a start's handover taken, its file removed only after `TakeOver`'s
  `Done`, and left when it stops; a handover readable by others
  refused, naming it;
- each write answered `Written` only once kept, so a restart between
  finds what a browser was told;
- a read served from the run's pasted value before the forge's record;
  a pasted value whose digest differs refused, "the pasted value
  changed";
- one writing plan per forge and one `Reachable` beside it, a second of
  either refused busy, and a request past `setup.runs` refused busy;
- a start's reconcile that stops leaving that forge not connected,
  named with its step and no secret, and the others served;
- records made for another kind or address leaving the forge not
  connected, named, nothing erased; an `api_token`, `git_token`, `user`
  or `webhook_secret` file in a forge's directory left unread.

## 3. What the setup relies on

Forgejo's and the runner's behaviours at the pinned versions (`owned.md`, 1), each
recorded by skein's conformance (`skein/docs/design/testing/fakes/forge.md`, section 7) on the
throwaway Forgejo and runner, and named here by its fact in the
conformance's `facts.txt`. What reconcile relies on, and the deliveries,
are skein's: `skein/docs/design/forge/setup/forgejo.md`, section 2 and `../forge/README.md`, section 8. What
the preparation relies on stays here:

- **Forgejo's API takes the person's tokens** once the person need not
  change their password (`api-token-of-a-user-whose-password-must-change-is-refused`,
  403 "You must change your password", recorded at 17.0.1; hence
  `--must-change-password=false`), and after a later change of it: a
  token is the person's until it is revoked
  (`api-token-survives-a-password-change`);
- **Forgejo's command line** (`owned.md`, 3):
  - `admin user create --random-password` prints `generated random
    password is '<password>'` on its first line
    (`cli-user-create-prints-the-random-password-first`), and refuses a
    user made twice (`cli-user-create-again-is-refused-naming-the-user`);
  - `admin user generate-access-token --raw --scopes` prints the token
    alone (`cli-token-raw-prints-the-token-alone`), limited to the scopes
    it names, comma-joined (`cli-token-scopes-limit-what-the-token-reads`,
    `cli-token-admin-scopes-comma-joined-are-taken-under-two-suffixes`),
    and refuses a second token of one name
    (`cli-token-name-used-twice-is-refused`);
  - `migrate` runs on a fresh work path, and again on a migrated one
    (`cli-migrate-runs-on-a-fresh-work-path`,
    `cli-migrate-runs-again-on-a-migrated-work-path`);
  - `dump` needs the work path's repositories directory and its
    `--tempdir` (`cli-dump-needs-the-repositories-directory`,
    `cli-dump-needs-its-tempdir-to-exist`), adds `.zip` to a name without
    it (`cli-dump-appends-zip-to-a-file-name-without-it`), and overwrites
    a file of its name (`cli-dump-over-an-existing-file-overwrites-it`);
  - a user made to change its password is refused the API until it has
    (`cli-user-made-to-change-its-password-is-refused-the-api-until-cleared`).
- **The runner** (`owned.md`, 3):
  - `forgejo-cli actions register --secret-file` takes exactly 40
    hexadecimal digits with no line feed
    (`runner-register-refuses-a-secret-of-another-length`,
    `runner-register-refuses-a-secret-that-is-not-hexadecimal`), and run
    again with the same secret keeps one global runner
    (`runner-register-again-keeps-one-global-runner`);
  - the runner's `create-runner-file` takes its secret only as an
    argument
    (`runner-create-runner-file-takes-its-secret-only-as-an-argument`);
  - a connection of the runner's `server.connections` has, as its `uuid`,
    the secret's first sixteen characters in hexadecimal, and as its
    token the secret
    (`runner-uuid-is-the-secrets-first-sixteen-characters-in-hex`,
    `runner-token-is-the-secret`), which a file may hold (`token_url:
    file:`; `runner-connection-with-a-token-file-comes-online`); a runner
    started on that connection alone takes the instance's jobs
    (`runner-connection-alone-takes-the-jobs`), a job's `runs-on: kit-host`
    over a row registered raw as `kit-host:host`, since it declares at its
    start. `register` run again writes the row's labels over
    (`runner-register-again-keeps-one-global-runner`), which is why a
    preparation runs it only where the row may not exist (`owned.md`, 3).

## 4. The process tier

The tier, its budget and its referee are `../../testing/README.md`'s (2.6). What
it checks of this document:

- **The box's lifetime with running steps** (`box.md`, 1, 2), in
  `bin/temper-box/tests/suite/box_lifetime.rs`: a hangup stops a box whose
  playground steps run, nothing left, its log's last line `Stopped`,
  and the same retained configuration binds again; and, on Linux, a
  killed box takes its running steps with it. These checks need real
  processes and their deaths; 1 holds the stop decisions below them.
- **Its directories:** every command a test runs is given the three
  directories' flags, each a scratch directory of the test's own; the
  owned example is copied as a configuration directory (`init.md`, 1). No test
  resolves a default under `$HOME`.
- **Owned in place of the test's support:** the test runs `temper-box forge
  prepare`, `run forgejo` and `run runner` itself, with no group's file
  (`box.md`, 4), then `temper-box serve`,
  whose setup host takes the handover and reconciles (`owned.md`, 5), with the
  binaries `TEMPER_TEST_FORGEJO` and `TEMPER_TEST_FORGEJO_RUNNER` name, and
  webhooks on. Its repository runs Actions, so its landing rule asks for
  CI.
- **The hinted story:** the same story with the keep's `poll`,
  `poll_max` and `slow` and the change's `stall` raised past the story's
  budget (../configuration/limits.md; `../../domain/forge/change/repairs.md`), and its
  CI's job outlasting `settle` from the change's push, so that it lands
  within its budget only if the job's end is delivered and the reads
  that follow it (`../../domain/forge/README.md`, section 6, "Checked"); and with no conflict resolved,
  since its changes never conflict. With no webhooks, the same story
  misses its budget.
- **`temper box`,** a test of its own on a scratch directory, with no
  `--echo`, its terminal the sign-in block alone and each step's lines in its
  log; a story that reads a step's lines runs it with `--echo steps`: the sign-in
  link on its stdout, and whole on one pipe given as both its stdout and
  its stderr, as a terminal is (`box.md`, 2); a port taken refused with nothing on
  its stdout; the engine's link file holding its local link alone, never
  the printed link's, and that secret on no stdout, stderr or log
  (`../../client/tui/README.md`, 2); the first link opened at once, before
  serve listens, waiting and then signing in, the link file left as it
  was; a killed serve's fresh link, with the socket held
  across the restart; Forgejo killed and restarted, with the engine still
  serving; a stop leaving no process it started.
- **A first run, into the playground and out,** a test of its own:
  `temper box` on three empty scratch directories writes the
  configuration (`init.md`, 2), says so in its own log (init's line for no browser
  among its lines only from a binary built without one: the test checks the build
  it runs in, `../../client/web/README.md`, 2.6), and prints one sign-in link,
  which signs the owner in, with no forge kept and every group dormant.
  The owner starts the playground with `temper ctl playground start`
  (`../../client/ctl/areas.md`), answered running once its fake forge is
  connected and `playground/widget` adopted into Demo, the project its
  start made (`playground.md`); then resets it with `temper ctl playground end`,
  answered reset. The store then reads empty, as a first run's: no
  forge listed, no account of the deployment's, no project, and no
  `playground-N`, `box/groups/N` or `box/started/N`
  left; and a native client reloaded opens the welcome at its Forge
  step (`../../client/pages/README.md`).
- **A reset the box's death cut,** a test of its own: a first run's box,
  the playground started, then reset while the box is held before its
  `removed`, after the project's deletion is committed, and the box
  killed with the reset unanswered. The next box on the same
  directories, with no client asking, ends the reset whole at its start
  (`playground.md`): the store reads empty as above.
- **Two first runs, one after the other,** a test of its own: a first
  run's box, signed in, then a second box on three other empty scratch
  directories while the first runs. The second's init finds the first's
  ports held and writes the ports the kernel assigns (`init.md`, 2), and the
  second signs in, starts its playground and answers a suggested request
  on its own fakes while the first, which starts none, holds the
  defaults. No first-run story takes a lock
  on the default ports: each box holds its own (`box.md`, 1), so the stories
  run beside each other as any others do.
- **A set up cut short, started over,** a test of its own: under the
  box, with Forgejo's binaries placed in the cache as below, the
  owner's set up of an owned Forgejo is killed with the box at each of
  its steps, its fetch, its preparation, its Forgejo's run, and after
  its `started` record with the engine held before it hears it. That
  started cut alone uses init's configuration without sign-in, since
  its source step would need discovery from the held engine
  (`../../testing/process-tier.md`). Sign-in is restored before the final
  setup and removal cut. Each
  next box finds the forge kept and its first repository not made, and
  its removal, `temper ctl forge remove N`, answers removed and leaves
  no `forge-N`, `box/groups/N` or `box/started/N`. A set up after the
  four makes forge 5, since numbers are never reused, connects it and
  makes its first repository; the cache's binaries are the ones placed,
  unchanged (`owned.md`, 7). Then a removal the box's death cut, the box held
  after the removal was answered and killed: the next box ends it whole
  at its start, no forge listed and nothing of it left.
- **GitHub gone on after a restart,** a test of its own, on `temper box
  --fake-backends`' fake GitHub (`box.md`, 5): GitHub's set up stopped after
  its App's records are kept, the box restarted, and the forge found
  unfinished, then gone on, to the App's installation, with no Start
  over, and connected (`owned.md`, 7).
- **A first run set up for real,** a test of its own: `temper box` on
  three empty scratch directories, with the binaries `TEMPER_TEST_FORGEJO`
  and `TEMPER_TEST_FORGEJO_RUNNER` name placed where the fetch looks,
  `<cache>/forgejo/VERSION/forgejo` and
  `<cache>/forgejo-runner/RUNNER_VERSION/forgejo-runner`, so that the
  fetch hashes them against the pins and fetches nothing (`owned.md`, 6): no
  mirror, no TLS and no call off the machine. The owner asks "Set up a
  Forgejo here" through the client, `temper ctl DIRECTORY forge
  setup`, answered setting up; the new forge's row, forge 1, the first
  made, followed by `forge list --until settled`, goes through "Setting
  up" and "Connecting" to "Connected as temper", with no restart, and
  its first repository made, `work/hello`; the call adopts it into the
  new project `hello`, project 1, the first made (`owned.md`, 7). The
  workflow's push has run on the owned runner: before the owner's first
  task, the runner's output, which the box, run with `--echo steps`,
  gives on its stderr as `runner-1: ` lines, says it took a task on `work/hello`, main's
  push's, since no pull exists yet. It shows the run taken, not its
  verdict: no read of temper's says CI at a repository's `main`, and
  the story reads no credential. The owner's first task
  in the project, on the fake LLM, changes `hello`: its worker clones
  it, its pull's run of the starter workflow passes, and it lands; the
  test reads it all through temper's own reads, with no credential:
  the set up's answer names `work/hello` adopted, owned, on forge 1;
  the task, given in project 1, runs on and lands into `work/hello`;
  the change's checks at its pull's head are the starter workflow's
  run, passed; and the task ends landed. The
  handover file is gone, and
  `forge-1` has no `api_token`, `git_token`, `user` or
  `webhook_secret` file. The service's step world shows the git value
  and its generation (../forge/README.md, section 9, the box's link). A second
  box on the same directories starts the group by its record, before
  `temper-box serve` unless the box signs its person in through that forge; then
  serve starts first, to answer the source's discovery (../openid-provider.md, 9). Preparation makes a new handover; serve takes it over,
  its file gone again, and connects forge 1 from its records (`owned.md`, 5), and the task is found
  landed, its CI at its head unknown until a next step reads it
  (`../../domain/forge/change/README.md`, 5).
- **A Forgejo connected as its admin, on this host:** on a real Forgejo
  with its API on loopback, each repository adopted has one hook at
  `http://<listener>/hooks/<number>` (`reconcile.md`, 4), and has it again, with its
  collaborator, after a restart with both removed by hand (reconcile's
  step 6). Its delivery is not this story's. With a hook that delivers
  nothing, a step test shows the forge staying connected with nothing
  ended, and the forge's world wakes a watched change on its pull's CI
  with no delivery heard, by polling (`README.md`, section 6).
- **The first-run behaviours** past these use their lower owners: the
  stock scripts' data tests, the real-loop playground stories, and the
  client and engine setup/reset worlds. The process tier has one short
  fake wiring test; it does not replay the playground before the owned
  repository story (`../../testing/process-tier.md`).
