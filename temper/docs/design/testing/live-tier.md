# The live tier

Nothing is handed in: no input takes a record, no tool reads another
program's login, and a ChatGPT account comes only from a sign-in
(../protocol/credentials.md, section 3), on the owned example's
registration (../protocol/setup/init.md, 1). The one exception is a held
record, a person's signed-in ChatGPT account moved in by `temper ctl
account import` and back by `account export`, never copied, which the
owned-forge landing story and the ladder's rungs take when given one
(below). An Anthropic API key is the other: a private file whose path
the key landing story hands to `temper ctl account add-key`, never read,
printed or copied by the story (below).

`bin/temper-box/tests/live.rs`, after jig's (jig's `protocol/service.md`,
section 8; skein/docs/foundation/testing-strategy.md, 2.9).

- **What runs:** the process tier's binary, owned Forgejo, real git and
  person, with a real model: the live configuration, a configuration
  directory (../protocol/configuration/directories.md), is the owned
  example's (../protocol/setup/init.md, 1), its LLM providers'
  endpoints the providers' own and Claude Code's path the machine's
  `claude`. Each run is a fresh deployment, with no LLM account until
  the script adds one, in place of the process tier's key (`process-tier.md`, step
  1), with `temper ctl` signed in by a link the binary
  prints (../client/ctl/README.md, section 8); the script's argument says which
  account, Claude Code's login by default, since it needs nobody.
- **Claude Code, unattended** (the default). The live configuration's
  programs directory is `~/.local/state/temper/live/programs`. The
  developer runs `temper-box program-login --config LIVE live`, `LIVE` the
  live configuration's directory, once on each
  machine that runs the tier, and signs in as Claude Code asks
  (../protocol/credentials.md, section 3). Each run's deployment declares
  the program source `live` (`temper ctl DIRECTORY account declare 2
  claude_code live --key live-program`, ../client/ctl/areas.md, area
  5), then waits with `account list --until usable`: the name, not the
  account's number, ties the source to the login, which the worker finds
  signed in.
  Claude Code renews its own login; temper never reads it, and the run
  needs nobody. Its runs are on smith's program backend (../protocol/llm.md,
  section 2).
- **ChatGPT, signed in as the run starts.** `scripts/proof-live.sh`
  runs `temper ctl DIRECTORY account connect 1 --key live-sign-in`,
  prints its address, and runs `account list --until settled`, within the
  sign-in's deadline, while the developer approves it in a browser on
  the same machine, or through the port forwarded to it
  (../protocol/credentials.md, section 3). The account is the person's
  that `temper ctl` signed in, a plan's (jig's `domain/accounts.md`,
  3.2), in the run's deployment, and goes with it; its runs are on
  smith's direct backend.
- **A key, when neither is at hand:** `account add-key PROVIDER FILE
  --deployment --key live-key`, from a private file the script's
  argument names, then `account list --until usable`.
- **Nothing borrowed:** no other program's login is read (no
  `~/.codex`), no record or key is in a file the configuration names,
  and the script copies no credential; an account added for a run is in
  its deployment's store, removed with its scratch directory.
- **GitHub, by its own test:** the stories above have the owned
  Forgejo alone and never call github.com. github.com is reached only
  by the GitHub story below and by skein's opt-in conformance (skein's
  `testing/fakes/github.md`, section 2), each run by hand by the developer who
  holds the probe App's fixtures (`README.md`, section 2), never by an implementer.
- **The stories:** the process tier's chat, answered by the real model;
  and one small change, made by the model in a repository of the owned
  Forgejo, adopted as in `process-tier.md`, and pushed by real git. The story ends
  once the change's branch on the forge is at the run's commit; it does
  not wait for the change to land. A story also ends at its task's first
  terminal failure, which fails the test at once, naming it: it never
  waits out the run's 30 minutes after its task has ended. An account
  limited, refused or needing sign-in fails the test the same way,
  naming the account's state and its reset, since its task would wait
  for it (jig's `domain/authority.md`, 8.3).
- **The owned-forge landing story,**
  `a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge`, run by
  `scripts/proof-live.sh forge-land`: a fresh ChatGPT sign-in, init's
  shipped charters and real Responses on `gpt-6-luna`; the chat chooses
  `change`, a tagged README line lands on work/hello with CI green: its
  final nonempty line matches exactly, skipping only zero-length lines.
  Before discovering a change, an attempted idle/news chat with no live
  child, run wait or subscription and the shipped immediate wake policy
  fails promptly. Active work and final parked replies remain valid.
  The chat's one delegate is its change: a delegate whose executor is an
  agent, as a goal's is, fails the story as soon as it is found, naming
  the task and its charter, never watched as though it were the change.
  Where the chat made no successful `change`, the failure names the
  tools of the chat's committed calls, those that succeeded and those
  that failed, so a refused `change` before another call shows. A
  fresh client finds the same chat, pull and merge hash after the box
  restarts on its three directories, without another provider sign-in,
  push or merge. The change's page says "Passed" for CI at its head on
  both runs: a landed change keeps its last verdict at its head with its
  record through the restart (../domain/forge/change/README.md, 5), as `process-tier.md`'s restart stories
  read it. It closes the chat and drains the box; the sitting
  allows ten minutes for sign-in, forty minutes in total and 1–25
  completions, counted through ctl. Progress and final evidence reach
  the person through a private notice file, followed by the script and
  retained with the sign-in secret redacted. Its config, data, cache and
  client roots stay in its own store, `forge-land` under
  `TEMPER_TEST_LIVE_STORE` (by default `$HOME/src/rust/tmp/live-store`), on
  pass or failure. A
  rerun uses the kept sign-in without a person and stops when that
  sign-in is expired or refused. It admits the deployment alone and
  makes its own client roots, so a sitting that failed before its
  restart leaves a store a rerun can use. With `TEMPER_TEST_LIVE_ACCOUNT`
  naming a held ChatGPT record, a fresh store's first run moves it in by
  `temper ctl account import FILE --key connect-plan`, which removes
  FILE, and waits for it usable in place of the person's sign-in, as an
  account import does; the script refuses it with a kept store. Each box
  exports and verifies the record before stopping, including a successful
  first box whose result is checked before the restart. The second box
  imports the returned record with a distinct operation key. Hook installation
  and reopening import share the give-back's lock; a terminal give-back
  excludes later imports.
  Every ctl call of a sitting names one DIRECTORY, `client` beneath the
  run's root in the store (`first`, `second`, or tagged on a kept
  store): the import, the held record's give-back and the sitting's
  reads alike (../client/ctl/README.md, 6.1). The chat's ended delegates and
  the change's are read whole, a page at a time: `task list --project
  PROJECT --parent TASK --ended --json`, then the same with `--after`
  and the cursor the page printed (`ended.rows.after.value`), until a
  page prints none (../client/ctl/tools.md, 2); a page cut with no cursor,
  or past 32 pages in all, fails the story, naming it.
- **The owned-forge landing story on Claude Code,**
  `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge`, run
  by `scripts/proof-live.sh forge-land-claude`: the same story, every
  role on Claude Code on `claude-haiku-5-5` at `low` effort, on a held login used in place
  (../protocol/credentials.md, 3). The stock catalogue has no Haiku, so
  a fresh sitting takes it by name, as a person adds a Claude
  subscription's model on Settings' Models and then chooses it (jig's
  `domain/engine/settings.md`, 11), in two `settings apply`s. The first sets a
  model at the highest number its catalogue leaves free, with the figures
  of the provider's dearest stock model, `claude-fable-5-1`, marked
  estimated and the person's, its efforts with `low` its default; the
  second, against the revision the first made, has every charter the
  story selects name it. One change can't hold both: the clients'
  component refuses a change whose charter names a model the catalogue
  lacks before the core sees it, and a model the change itself sets is
  not in the catalogue yet. A focused test applies the fixture's own
  changes through ctl on the joined world (`checks.md`, 3), its engine holding the
  Claude deployment's stock catalogue. `TEMPER_TEST_LIVE_CLAUDE` names the
  login's resting path, a signed-in configuration directory; the box's
  programs directory is its parent, and the source is declared by the
  directory's own name. Nothing of it is read or copied, and once the
  box has stopped the script checks the path by `stat` alone: a private
  directory, still there. Whoever runs it holds the login for the run.
  Its store is `forge-land-claude` under `TEMPER_TEST_LIVE_STORE`.
- **The owned-forge landing story on an Anthropic key,**
  `a_person_asks_claude_on_an_api_key_to_land_a_change_on_the_owned_forge`,
  run by `scripts/proof-live.sh forge-land-claude-key`: the ChatGPT
  story's sitting, restart, referee, bounds and evidence, on a person's
  Anthropic API key through smith's direct backend
  (`../draft/claude-api.md`, sections 4 and 6), every role on
  `claude-haiku-5-5` at `low` effort. ChatGPT and Codex logins stay off
  automated tests (the user, 2026-11-12), so this story, not ChatGPT's,
  is the after-landing pair's direct one.
  - **Its key:** `TEMPER_TEST_LIVE_ANTHROPIC_KEY` names a Claude Console
    key's file: a regular file of the test's user, not a link, mode 0600
    and nonempty, admitted by `stat` alone. Unset, the story skips: it
    says why on stderr and passes, starting nothing. Set to anything
    else, it fails before any process starts, naming the variable and
    the rule broken, never the path.
  - **Adding it:** a fresh store's first sitting runs `temper ctl
    DIRECTORY account add-key 2 FILE --key live-key`, which makes the
    person's key-only account (`../client/ctl/areas.md`, area 5), then
    `account list --until usable`, and reads that the person holds an
    account of provider 2 whose key source is usable. ctl reads FILE and
    leaves it in place; the story and the script never open it, and what
    they print of the command says `FILE` for its path. temper keeps the
    key in the story's deployment, as any key added through `ctl`, in the
    store under `TEMPER_TEST_LIVE_STORE`: a second sitting, and a rerun
    on a kept store, add none and run on the key the store keeps. Nothing
    is lent, so the story has no return.
  - **Its configuration:** init's owned example as init writes it, both
    Anthropic endpoints and Claude Code's program included, but the
    `anthropic` provider without `keep` and `drop`, so that its accounts
    read no list (jig's `domain/accounts.md`, 3.7). A key's first read
    would add Anthropic's models at the catalogue's highest free numbers
    while the sitting adds Haiku there by name, and the two racing would
    stale the apply's revision or double the name. Its catalogue is the
    stock and Haiku, as on Claude Code.
  - **Its model:** the Claude Code story's two `settings apply`s,
    unchanged: the stock has no Haiku, so a fresh sitting adds it by
    name with `claude-fable-5-1`'s figures, `low` its default, then has
    charters 1 and 2 name it. The run takes the key's direct endpoint
    whatever endpoint the model's row names, since the endpoint is the
    chosen source's (`../draft/claude-api.md`, section 4).
  - **Its store** is `forge-land-claude-key` under
    `TEMPER_TEST_LIVE_STORE`.
- **Its referee:** the process tier's, without the restart; and the
  completions, counted from the engine's transcripts. Each completion is
  a turn of main's conversation on either backend: on the program
  backend, each assistant message the program writes (smith's
  `protocol/programs.md`, 5.5), so the count holds for Claude Code's
  runs as for direct ones.
- **Budget:** at most 25 completions and 30 minutes in all. The live
  configuration's charters bound each run's turns so the two stories
  together make at most 25 completions, which smith's agent holds on
  either backend (on the program backend with `--max-turns` behind its
  own count; smith's `protocol/programs.md`, 5.7), the script stops the
  run at 30 minutes, and the test fails when the transcripts count more
  than 25. Each run's summary, a failed run's included, records its
  completions and its wall time.
- **A chat's change lands on github.com, through a restart,** a test
  of its own, `bin/temper-box/tests/live_github.rs`, run by
  `scripts/proof-live-github.sh OWNER`, `OWNER` the org `fabrig-ai` and
  no other account, one repository a run: the process tier's story of `process-tier.md`
  ("a chat's change on connected GitHub, through a restart") on the
  real service, on the probe App of skein's conformance fixtures and
  its installation on `OWNER/temper-probe`, whose `ci` workflow is a
  real Actions run of about 45 s.
  - **What runs:** the engine alone and one worker, as in `process-tier.md`, the fake
    LLM (the model is not what it tests), the machine's git, and the
    test's relay. The forge is a GitHub forge connected with the probe
    App, its record's `read_only` set (../protocol/forge/engine.md, 1;
    ../protocol/setup/github.md, 7), its place `application.machine.github`'s
    (../protocol/configuration/engine.md): `api` and `remote` the
    relay's two loopback listeners, `web` the latter's origin; and the
    forge component's and the resources' polling their defaults but
    `log_poll`, 15 s.
  - **Offline startup admission:** `startup.rs` renders these same
    GitHub templates with synthetic ready captures and uses the live
    story's pure engine shaper for that owner, distinct
    API/git sockets and the live polling defaults. Actual engine and
    worker preparation must admit without a relay, credentials or
    processes. The shared landing helper for this and the fake process
    story clears only unused goal projection grants: both stories make a
    chat's change, no tracked goal. Task ceiling and role-authority
    grants, the enforced CI/up-to-date/ exact-head gate rule, metadata
    and all capacities stay, including `core.policy_bytes=8192`. The
    four-role stock policy measures 5,742 joined bytes; this fixture ceiling
    leaves headroom, while Standard and Small keep 16,384. The
    pure service startup regression keeps the overbroad seed's named
    refusal and the narrowed seed's admission. This is offline coverage,
    and establishes no live proof.
  - **The relay,** the test's, between temper and github.com: plaintext
    HTTP/1.1 from the engine and the worker's git on loopback, verified
    TLS to `api.github.com` and `github.com` by skein's TLS client on the
    machine's roots, as skein's conformance reaches GitHub, the
    Host rewritten, every request and answer framed and passed whole.
    It counts each HTTP request, the test's own included, across both
    lives, taking the slot before it forwards: past 380 it answers
    `429` and the test fails naming the cap; the last 20 are the
    teardown's, 400 in all. It forwards only the App's routes
    (`GET /app`, its hook's config, its delivery log, its installations,
    a mint for the run's installation alone), `DELETE
    /installation/token`, `POST /graphql` carrying skein's viewer or
    pull document alone (the pull's on the probe; never a mutation),
    `GET /user/ID`, `GET /repositories/ID` for the probe's id, learned
    from the story's first read, the probe repository's reads, its
    writes on temper's own routes (pulls, issues, comments, reviews,
    reviewers, statuses, merge and update), and git's three smart-HTTP
    paths of that
    repository; a ref written, by the API or a push's commands, is one
    under the run's prefix, never `main`; any other request is answered
    `403` and fails the test, naming it. Its journal holds each
    request's method, path and status, never a header or a body.
  - **The story:** the person signs in and adds the fake LLM's key;
    `forge connect 1 github-app APP FILE` with the fixtures' App and key
    file, `forge list --until connected:1`, then `--until settled`;
    `forge install 1` issues a state, asked again while setup answers
    "busy", at most 30 times, one a 2 s poll, each under a fresh key (a
    key's first answer is replayed, ../client/ctl/README.md, 5) and logged
    with its time, since the reconcile after a connect is the engine's,
    which `settled` does not cover (../client/ctl/tools.md, 4); and the
    test brings it to the engine's setup route with the installation's
    id, as GitHub's redirect would
    (`/forges/1/github/setup?installation_id=I&setup_action=install&state=S`),
    which accepts it; `repository adopt` of `OWNER/temper-probe` with
    the prefix `temper/live-<run>/`, `<run>` unique to the run; a chat
    asks for a change whose coder writes one new file, `probe/<run>.md`,
    and finishes; the story waits until the change task has ended landed
    and the chat has replied and closed; then the worker and the engine
    are restarted as in 2.6.
  - **Its Musts,** after the restart: the change task ended landed, its
    result the same pull request and merge commit as in the first life,
    and no task made after it; GitHub agrees, read in the person's
    role: the pull merged, its merge commit temper's and its head the
    change's head, and main's tree holds `probe/<run>.md` with the
    run's bytes; the head's `ci` check run, of `github-actions`,
    completed with success before the pull's `merged_at`; the relay saw
    one pull opened and one merge, and refused nothing; the forge
    reconnects as the App's `SLUG[bot]` and id, the installation
    accepted; and no process's output, ctl's or the test's, nor any
    file under the processes' directories but jig's secrets store,
    holds the App's key (PEM, DER, DER as base64), a token the relay
    saw minted, or the person's token.
  - **Its teardown,** whatever the story's end, through the relay: each
    branch under the run's prefix deleted (temper deletes its own at a
    task's release; one left is named), a pull of the run still open
    closed, and each token the relay saw minted revoked. The landed
    file stays on main, one a run. When the teardown did not finish,
    the script does the same with the person's token, within what is
    left of the 400.
  - **Credentials:** the fixtures' directory is the script's argument
    or its default; the App's key file goes to `forge connect` by path,
    and the person's token is read by the test and the script into
    memory alone, never printed, copied or written.
  - **Its budget:** at most 400 requests to github.com and 20 minutes
    a run; the summary records the requests, the wall time and the
    run's id.
  - **What it does not cover,** for the relay's sake: the engine's own
    TLS to github.com (temper speaks plaintext to the relay; skein's
    TLS client is live in skein's conformance), and authorities and
    landing rules on github.com's host, since the repository's
    resources are named by the relay's git host (../protocol/channel.md, section
    4). The model is the fake LLM's, the browser's visits to GitHub
    (making the App, installing it) are the fixtures', made once by
    hand, and a failing or stalled check is the process tier's on
    fakes.
- **The ladder's rungs,** a live story of their own each once its
  acceptance story (`../../development/acceptance.md`) passes on real
  backends (`../draft/v0.1.0.md`, 3): the acceptance story's steps as a
  test in `bin/temper-box/tests/live.rs`, run by
  `scripts/proof-live.sh RUNG`, `rung-a` to `rung-b2`. Its driver, the
  same code, first walks the whole story offline on the rung's twin
  (`real-loop.md`). B0's offline proofs need no person: its setup and its walk on
  the twin are the process tier's (`process-tier.md`, *Two drivers' offline proofs*),
  and its live configuration's admission is `suite`'s (`startup.rs`).
  `scripts/proof-live.sh rung-b0-offline` runs the first two before the
  live story, under profile `process-temper`, whose default filter
  selects them; it admits no live account. A driver's own fault is met there, never
  by spending a held login.
  - **What runs:** a fresh box as the help makes one, its owned Forgejo
    set up through ctl with the playground's copy
    (`../protocol/setup/owned.md`, 7); the model a held ChatGPT record,
    `TEMPER_TEST_LIVE_ACCOUNT`, moved in by `temper ctl account import` and
    moved back by `account export` over the same path before the box
    stops; on any ending that comes first, a pass, a failure or a
    signal, the launcher returns it once the test process has ended
    (the held record's return, below), and checks it by `stat` alone.
    Whoever runs it holds the login for the run.
  - **Its acts are the story's,** the person's words included, through
    `temper ctl`; the forge's by the box's person, its read and write
    header files (`../protocol/setup/box.md`, 3), as the story's person
    acts.
  - **Its Musts are the story's "What must hold",** read through temper
    and through Forgejo's API, each a comparison of the two accounts.
  - **Its bounds are the passing walk's,** as its report states them:
    each phase's time and the whole; a run past one fails, naming the
    phase. A bound is never raised: a slower run is a finding.
  - **When it runs:** before the release, and when a change touches its
    path, by the owner of that change, one live run at a time on the
    login. Never at the gate.
  - **Its other pairs** come as the rung's walks with GitHub and with
    Claude pass (`../draft/v0.1.0.md`, 2.2): the same story, its forge
    or its model named by the script's argument.
- **After every landing,** the owned-forge landing pair, needing no person:
  `a_person_asks_claude_on_an_api_key_to_land_a_change_on_the_owned_forge` on an
  Anthropic API key (`TEMPER_TEST_LIVE_ANTHROPIC_KEY`) and
  `a_person_asks_claude_code_to_land_a_change_on_the_owned_forge` on a held Claude
  Code login (`TEMPER_TEST_LIVE_CLAUDE`, with `claude` on the machine's `PATH`), both
  named inputs of the after-landing tier. The tier runs profile `live-temper`
  under `--run-ignored only`, whose default filter selects these two alone.
  Automatic live runs take the cheapest models, as the user requires:
  `claude-haiku-5-5` for both. ChatGPT and Codex logins stay off automated
  tests (the user, 2026-11-12): the ChatGPT landing story is the owner's,
  through `scripts/proof-live.sh forge-land` alone.
  - **One invocation, a store per story:** the runner sets one fresh
    `TEMPER_TEST_LIVE_STORE`; each story takes its own store beneath it, named by the
    script's argument for it, `forge-land-claude-key` and `forge-land-claude`, so
    neither reuses the other's deployment. Each store keeps the story's rules: fresh
    when absent or empty, kept when it holds a deployment.
  - **The runner's part,** as `scripts/proof-live.sh`'s: a private `TMPDIR`; after
    the invocation, whatever its end, the login checked by `stat` alone, and any
    process left under the stores stopped.
  - **What the stories need of the runner,** and nothing else:
    `TEMPER_TEST_LIVE_STORE`, an absolute path, made 0700 with its parents when
    absent; `TEMPER_TEST_LIVE_ANTHROPIC_KEY`, the key's file, a regular file of the
    runner's user, mode 0600, which stays where it is; `TEMPER_TEST_LIVE_CLAUDE`,
    the login's resting path, and `claude` on `PATH`; a private `TMPDIR`. With
    `TEMPER_TEST_LIVE_ANTHROPIC_KEY` unset the key story skips, saying so, and
    the tier runs Claude Code's alone. No directory above these
    needs a mode or a name: ctl opens DIRECTORY and FILE through their
    parents and checks DIRECTORY, its files and FILE alone
    (../client/ctl/README.md, 6.1).
  - **Its bounds** are each story's own, within the profile's 50 minutes for the
    pair. Never at the gate, and never run by an implementer.
- **The held record's return,** for each story that moves a held record
  in, the ChatGPT landing story and B0's rung: the launcher's, once the
  test process has ended, however it ended (`../draft/panics.md`, 5).
  Each box of a story still returns the record before it stops, as its
  own act (above); this return is for whatever ends a sitting first, a
  panic, an abort, a failed return or a signal. Nothing in the test
  process catches a panic for it, and a panic in a sitting ends the
  test. `scripts/proof-live.sh`'s exit trap, on every ending, a build's
  failure among them, in this order, by `stat` alone, the record never
  read:
  - **The test process ends first.** On a signal, the launcher stops the
    runner with TERM and reaps it, and the runner ends the test process
    before it exits. On any other ending the runner has already
    returned.
  - **The record at its lent path,** a private regular file with bytes
    in it: nothing is owed. The import has not run, or a box's own
    return has.
  - **Owed, the story's box still up:** the installed `give-back`, within
    its 60 seconds. The rung's box outlives its test process, started in
    a session of its own, so the launcher finds it up; the landing
    story's box outlives a test process that ended without unwinding.
  - **Still owed, a box of its own:** the launcher runs
    `bin/temper-box/tests/live.rs`'s
    `the_held_record_returns_on_a_box_of_its_own` in a process of its
    own, with `TEMPER_TEST_LIVE_STORE` naming the story's own store: a
    box started over the store for this alone, dying with that process;
    a new client, `return-client`, signed in by the box's link within
    240 seconds; the recovery hook installed for that client, later
    imports still excluded; then `give-back`, and the box stopped within
    30 seconds. The test is ignored, its reason naming the launcher, and
    no profile's default filter selects it.
  - **Then every process the sitting started stops,** the story's box and
    the return's among them: those whose working directory is in the
    run's scratch or the store and that were not there before the run.
  - **Still not back:** the notice `LOGIN NOT BACK`, naming the store
    alone, which stays for recovery, and the launcher exits 1. The same
    path holding anything else is reported, never overwritten.

  A hook's installation is refused, as an error, once a terminal
  `give-back` has run: a later sitting can't reopen the import. The
  rung's box dies with its test in the offline face, which has no
  launcher (`bin/temper-box/tests/proof_live.rs`). A held Claude Code login is used in place
  and has no return: the launcher checks it by `stat` once every process
  has stopped.
- **What needs a person,** seven tests, each ignored, its reason naming what
  only a person gives, and each run by its script. None runs on a scripted
  person alone: each needs a person's account, key or sign-in, or
  github.com.
  - `live.rs`'s `a_real_model_answers_a_chat_and_pushes_a_small_change`: a
    person's fresh ChatGPT sign-in (`scripts/proof-live.sh`). Its story, a real
    model's chat and pushed change, runs unattended in the landing pair, which
    carries it on to a landing and a restart; a held login for it as well
    would spend a real model's run on no check the pair lacks.
  - `rung_b0_a_person_agrees_a_plan_before_the_change_lands`: a held ChatGPT
    account, on the rung's own model, sol at high (`scripts/proof-live.sh
    rung-b0`). Its bounds are that model's passing walk, so the cheapest
    model would prove another walk.
  - The landing pair: an Anthropic API key and a held Claude Code login,
    the after-landing tier's inputs (above). The owner also runs each alone,
    through `scripts/proof-live.sh forge-land-claude-key` and
    `forge-land-claude`.
  - The ChatGPT landing story: a held ChatGPT account, or a person's sign-in,
    through `scripts/proof-live.sh forge-land` alone, never in the tier.
  - `live_github.rs`'s `a_chat_lands_a_change_on_github_com_through_a_restart`:
    github.com, on the probe App's fixtures the owner holds
    (`scripts/proof-live-github.sh`).
  - `live_github_managed.rs`'s `managed_relay`: github.com, where a person
    makes the App under their own account and installs it
    (`scripts/proof-live-github-managed.sh live`). The fake sitting starts
    the same relay in its fake mode, as its relay.
- **Harness step checks run once,** in `suite` (`tests/suite/support_tests.rs`): no
  live or process test repeats a focused one.
- **Outside the suites:** nextest profile `live-temper`, whose default filter is the
  landing pair above, run after every landing; `scripts/proof-live.sh` and
  `scripts/proof-live-github.sh` select their own story over it
  (`--ignore-default-filter` and their own filter), one at a time. Never at the gate.
