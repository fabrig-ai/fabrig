# Owned: prepare and run

## 1. Its inputs and its binaries

- **Inputs:** the Forgejo binary, and the runner's, or no runner, each
  an absolute path or `"fetched"`, temper's pinned build in the cache
  (6); the address Forgejo listens on, a socket address on loopback;
  the person's login and email; the runner's labels, capacity and
  `PATH`; the forge's directory.
- **The binaries are checked** by every command that runs them: the
  first line of each one's version equal to temper's pin, Forgejo's and
  the runner's (`skein-forge`'s `VERSION` and `RUNNER_VERSION`, skein's
  `forge/README.md`, section 4.3), the versions skein's conformance checks
  (`skein/docs/design/testing/fakes/forge.md`, section 7). Another binary refuses the command,
  naming its path and what it printed. A fetched binary is checked so
  too, after its digest at its fetch; temper downloads nothing but by
  `temper-box forge fetch` (6).
- **One pin,** `skein-forge`'s, read by owned mode, the throwaway and
  the conformance alike, so the Forgejo temper runs is the Forgejo whose
  facts it checked (`testing.md`, 3): the versions, and beside them each fetched
  binary's SHA-256 for each platform and architecture temper fetches for
  (6). One version on both platforms: on macOS a bottle is that
  version and another build, so that the version holds there and the
  build the conformance checked, byte for byte, is Linux's.

## 2. Forgejo's settings

`app.ini`, written by every preparation from the setup and `state`,
private, never edited by hand:

- `RUN_USER` the process's user, `WORK_PATH` the work path;
- `[server]`: `HTTP_ADDR` and `HTTP_PORT` from the address, `ROOT_URL`
  `https://<public>/` with `DOMAIN` its host when `public` is given, and
  `http://<address>/` otherwise, `DISABLE_SSH`, `OFFLINE_MODE`;
- `[database]`: SQLite in the work path;
- `[repository]`: its root in the work path, `DEFAULT_BRANCH` `main`;
- `[security]`: `INSTALL_LOCK`, `SECRET_KEY` and `INTERNAL_TOKEN`;
  `[oauth2]`: `JWT_SECRET`; each drawn once, at the first preparation,
  and kept in `state`;
- `[service]`: `DISABLE_REGISTRATION`; `[mailer]`: off;
- `[session]`: `SESSION_LIFE_TIME` 3600, an hour, on every owned Forgejo
  (`../openid-provider.md`, 9);
- when the forge signs its people in through temper, named by
  `application.sign_in.forges` (`../openid-provider.md`, 7): `[oauth2_client]`
  `ENABLE_AUTO_REGISTRATION = true`, `USERNAME = preferred_username` and
  `ACCOUNT_LINKING = login`, and `[service]`
  `ALLOW_ONLY_EXTERNAL_REGISTRATION = true` (`../openid-provider.md`, 4). The
  source itself is not the preparation's: it is added by the command
  `temper-box forge sign-in` prints, as for a managed Forgejo (8);
- on a box, when `application.machine.owned.sign_in` is not `null`: the same, with
  `ACCOUNT_LINKING = auto` (`../openid-provider.md`, 9). Its source is its own step's (`box.md`, 1);
- `[webhook]`: `ALLOWED_HOST_LIST` the webhooks' listener's IP address,
  when one is configured, and Forgejo's default otherwise, which refuses
  loopback;
- `[actions]`: `ENABLED` when the setup has a runner;
- `[log]`: files in the work path;
- `[api]`: Forgejo's defaults, which the forge component reads at its
  start (skein's `forge/connection.md`, section 3).

## 3. The preparation

`temper-box forge prepare [DIRECTORIES] FORGE`, before Forgejo runs. It takes
Forgejo's lock (4), and the runner's too when it writes the runner's
files: at a first preparation, for changed settings (item 2), and for
a runner older than its pin (item 3) or added (item 4); checks the
binaries; and decides by
`state`:

1. **The first preparation:** the layout of `README.md`, section 3; `app.ini`;
   `forgejo migrate`; temper's admin, `temper-admin`, made as a site
   admin with a password Forgejo draws and nobody keeps, and its token
   made by `admin user generate-access-token` into `admin_token`, the
   handover (`README.md`, section 3), with the four scopes `write:admin`, `read:user`,
   `write:organization` and `write:repository` (skein's `forge/setup/forgejo.md`,
   1.3, step 1: the users, their tokens and the orgs; `GET /user`; the
   orgs' and their repositories' making and reading; the repositories'
   routes of `Adopt`, reconcile's step 6 and the picker's read), and no
   other, under a name that
   ends in a random suffix, since Forgejo refuses a second token of one
   name; the
   person's account, a site admin with a password Forgejo draws into
   `person_password`, private, never printed, never in an argument or a
   log, with no change forced at the first sign-in (`admin user create
   --random-password --must-change-password=false`; the user, 10-26), so
   that the person's tokens work at once; the person's
   two tokens, each `admin user generate-access-token --raw --scopes`
   for the person's login under a name ending in a random suffix,
   written whole into `person_read.header` and `person_write.header`,
   private, so that the person's agent reads the forge, or acts on it
   as the person, by a file's path and never a token on a command line
   (`../../client/ctl/testing.md`, 5); the runner's
   secret drawn into `runner/secret` (40 hexadecimal digits, no line
   feed), registered as a global runner by `forgejo-cli actions register
   --secret-file`, and the runner's connection written from it into the
   runner's configuration (`server.connections`), its token in a private
   file the connection names (`token_url: file:`); the work path's
   repositories directory, which `forgejo dump` needs. `state` records
   the first preparation's start before anything else, and its end last.
   A first preparation that `state` says began and did not end is cut
   short: the next one removes what it left (the work path, the runner's
   directory, `admin_token`, `person_password`, the two header files)
   and begins again from an
   empty layout, since Forgejo refuses a user made twice and never shows
   a drawn password again. A forge's directory that holds Forgejo's files
   with no `state` is refused, naming it: the preparation removes only
   what a first preparation it recorded left.
2. **A later preparation at the same versions, with a runner exactly
   when `state` records one:** `app.ini` written again, so a change of
   the setup reaches Forgejo; `admin_token` made again when missing,
   which it is once the engine has taken it (5), under a fresh suffix,
   for temper's admin, which Forgejo's command line cannot make twice:
   an admin a person deleted fails the preparation, naming it; the
   person's two header files made again, each alone, when missing, as
   on a forge prepared before they were (a person who revokes a token
   deletes its file, and the next start mints a new one). A configured
   runner keeps its row (below), and its configuration is rendered from
   the current setup, with the `uuid` its present configuration names.
   If the configuration is identical, its file is left untouched and no
   runner lock is needed, so a box can restart Forgejo while the runner
   runs. A changed or missing configuration is written whole under the
   runner's lock; a running runner refuses that preparation. The prepared
   versions and Forgejo's secrets are recorded unchanged. Stop the box,
   edit `CONFIG/temper.json` and restart with the same directories to
   apply changed capacity, labels or `PATH` without replacing secrets.
3. **An upgrade:** a recorded Forgejo older than the pin is dumped
   (`forgejo dump`) into `backups/forgejo-<old>-to-<new>.zip`, written as
   `backups/dump.partial.zip` (Forgejo adds `.zip` to a name without it)
   and renamed, never overwritten (a dump already there is not made
   again), then migrated, then recorded. A recorded runner older than its
   pin has its files prepared again, under the runner's lock too, with no
   dump, since the runner keeps no data. A recorded version newer than
   its pin, Forgejo's or the runner's, refuses: Forgejo's migrations only
   go forward.
4. **A runner added or removed:** the setup and `state` disagree on
   whether there is one, and `app.ini`'s `[actions]` follows the setup
   (2). A runner added is prepared under the runner's lock too, as a
   first preparation prepares it: its secret drawn into `runner/secret`
   when that file is missing, and kept otherwise; registered, unless it
   keeps its row (below); its connection written. A runner removed is
   recorded as none, and its files and its registration are left, unused,
   so that a runner added again is the same one. A runner's recorded
   version is compared with its pin only while the setup has one. Either
   may come with an upgrade, in one preparation.

- **The runner's row is the runner's once it is made.** `forgejo-cli
  actions register` writes the row over: its labels as given, raw
  (`NAME:host`; `runner-register-again-keeps-one-global-runner`), and no
  version. The runner declares over it at its start, and only then: its
  labels' names and its version, which a job's `runs-on: NAME` matches
  (`runner-connection-alone-takes-the-jobs`). So a preparation writing
  the row under a runner that has declared, here or on another host (no
  lock of this one sees that), leaves its jobs waiting with no error
  until it starts again. A preparation registers only where the row may
  not exist: a first one, one that draws a secret, and one whose
  `runner/config.yaml` is missing or names no `uuid`, since that file is
  written only after `register` has answered. Otherwise, at the same
  versions, for an upgrade of either binary and for a runner added again,
  it runs no `register`: changed labels reach the row when the runner
  starts on its new configuration. Forgejo's command line cannot read the
  row, so a row a person removed in the site administration is not
  noticed: removing `runner/config.yaml` before a start registers it
  again.
- **A box's source, last.** A later preparation, with
  `application.machine.owned.sign_in` set, ends by pointing the source
  already listed at temper's current address (`../openid-provider.md`, 9), after
  `state` is written, so that a move of temper's port reaches Forgejo
  before it loads the source. A first preparation has no source yet. No
  ending of these two commands fails the preparation, but a termination
  signal, as below.
- **Forgejo's command line only,** on the work path, never its web: under
  the lock no Forgejo runs, so nothing writes its database beside the
  preparation.
- **No secret in an argument.** A password is drawn by Forgejo (the user
  creation's random password) and read from its output; the runner's
  secret goes by file (`--secret-file`). The runner's `create-runner-file`
  takes its secret only as an argument, so the preparation never runs
  it: it writes the runner's connection itself, in the runner's
  configuration, with its token in a file (`testing.md`, 3).
- **Its commands run on skein-io's processes,** as the box's children
  do: each spawned with its arguments and the spec's environment, its
  output read through pipes within a bound, its exit status awaited.
- **Interrupted** by a termination signal while a command runs, it
  kills the command's process group and exits 1, printing `forge <n>:
  interrupted`. `state` is not marked ended, so the next preparation
  begins again from an empty layout, for a first one, or repeats, for a
  later one or an upgrade.
- **Its commands' output,** kept for a failure's message, is masked of
  every secret it holds, as the throwaway Forgejo masks its own.

## 4. Running

`temper-box forge run [DIRECTORIES] FORGE forgejo|runner`:

- **Checks** the binary, and that `state` records a preparation at the
  pinned versions, with a runner exactly when the setup has one;
  refuses otherwise, naming what to run: `temper-box forge prepare` for a
  forge not prepared, a preparation that did not end, an older version,
  or a runner added or removed (3); no command of this temper for a
  newer version, or one it cannot read.
- **Takes the lock,** an exclusive lock, taken without waiting, on a
  file of the forge's directory, Forgejo's or the runner's. Two Forgejos
  never share a work path, and no preparation runs under one: the
  preparation takes Forgejo's lock, and the runner's too when it writes
  the runner's files (3), and refuses, naming the forge, while one it
  needs is held.
- **Checks Forgejo's listener,** when the box gave it one (`box.md`, 1):
  descriptor 3, with `LISTEN_FDS=1`. It checks that the descriptor
  listens at the setup's `listen` (skein's `listener_at`, `skein/docs/design/runtime/shell.md`,
  6.6) before taking the lock, refusing otherwise, naming the descriptor
  and the address it found. Forgejo then takes the socket whose address
  its `app.ini` names, in place of binding it (its `modules/graceful`,
  which reads no `LISTEN_PID`). Given none, as under every other
  arrangement (`box.md`, 4), Forgejo binds its address itself.
- **Stays Forgejo's parent when given a listener.** Forgejo 17 sends
  its parent `SIGTERM` once when it starts with `LISTEN_FDS` set (its
  `KillParent`, which no setting turns off), so forge run doesn't exec,
  which would make the box that parent. It holds the lock for Forgejo's
  whole life, adopts the listener into skein-io, and spawns `temper-box
  internal forge-child --parent PID --signal kill -- forgejo web …`, PID
  its own, with the listener at the child's descriptor 3, the spec's
  environment ending in `LISTEN_FDS=1`, and the spec's directory.
  Forgejo's standard input is empty; its output and errors are relayed
  to forge run's own.
  - **The hidden `internal forge-child`** reads no document and takes no
    directories. It execs its program with what it was given, through
    skein's `exec_dying_with_parent` (skein's `runtime/shell.md`, 3) with the
    parent-death signal its caller names, Kill for forge run, so that a
    forge run killed takes Forgejo with it at once. Kill, not Terminate:
    a Forgejo still stopping gracefully when nothing waits for it would
    meet the restarted forge run's on the box's socket and its database
    (`bin/temper-box/tests/suite/startup.rs`,
    `killing_forge_run_alone_kills_its_forgejo_child`).
    Forgejo's graceful stop is forge run's forwarded Terminate. The box
    names Terminate for each of its steps (`box.md`, 1). A
    failure, the parent changed or the program missing, prints `temper-box
    internal forge-child: WHY` and exits 1.
  - **Its shutdowns:** one that io names as Forgejo's (skein's `runtime/io/README.md`,
    section 4) is ignored, every time; every other is forwarded to
    Forgejo as the same Terminate or Interrupt. Forgejo's exit ends
    forge run with Forgejo's code, or 128 and the signal's number, so
    the box supervises forge run as it would Forgejo, and restarts it
    when Forgejo is killed. The decisions are a pure machine in
    `temper-forge-setup`; the shell only carries them out.
  - **On Linux only.** On macOS a shutdown names no sender (kqueue's
    `EVFILT_SIGNAL`) and no parent-death signal exists, so the box gives
    Forgejo's step no listener and Forgejo binds its own port, as above.
- **Replaces itself** (`execve`) otherwise: for the runner, and for
  Forgejo given no listener. It becomes `forgejo web` or
  `forgejo-runner daemon`, with the spec's arguments and the spec's
  environment alone: cleared, then `HOME` in the forge's directory,
  `PATH`, and git's configuration isolated, as the throwaway Forgejo's
  is. The lock crosses the exec as the process's standard input, so the
  process holds it for its life, and the process a host supervises is
  then Forgejo's or the runner's own, with its signals and its exit
  status.
- **Outside skein's shell, in one module.** skein's shell spawns but
  never execs, and its locks never pass to a child (skein's
  `../foundation/programming-model.md`, 2.1; skein's `runtime/io/files.md`). So the lock and
  the exec are std's, `File::try_lock` and `CommandExt::exec`, with no
  `unsafe`, in one module of `temper-forge-setup-shell`, which the
  preparation's lock uses too, and which builds `internal
  forge-child`'s command for skein's parent-death exec; the
  preparation's commands and Forgejo's parent loop are not in it, and
  run on skein-io (3). It is where temper locks a file or uses
  `std::process` outside skein: a boundary test scans the production
  crates' sources (`crates/*/src`, test files aside) and fails the gate
  when either appears outside that module, but for what it declares by
  file, each for that use alone: `std::process::ExitCode` in the
  binaries' `main`s, `temper-box`'s, `temper-client`'s and `temper-agent`'s; `temper-box
  program-login`'s exec of smith's command (../credentials.md, section 3);
  Claude Code's fetch's lock, in `temper-fetch-shell`, and its
  launcher's exec and process id, in `temper-launcher` (6); and the
  process tier's fault, which aborts
  its own serve. Test crates (`testing/`,
  `tools/`) are not scanned. It moves onto skein when skein's shell
  offers both (`README.md`, section 9).
- **The runner's spec:** its configuration file (its connection,
  capacity, labels, timeouts, its jobs' directory, no cache), its `PATH`
  from the setup. Its jobs run as the user that runs it (`README.md`, section 9).

## 5. Then the engine's start, and the person

- **The handover is the engine's,** for an owned or fake forge, at
  `temper-box serve`'s start after the records' load and the root's restore,
  or at the box's `started FORGE` (`box.md`, 1). Whenever the handover file is
  present, even with a kept `AdminToken`, the setup host reads
  `admin_token` once by the service's io, as a private file beneath the
  forge's directory, within `records.value_bytes` (../configuration/README.md,
  section 5). It runs `TakeOver { forge, admin_token }` (skein's
  `forge/setup/forgejo.md`, 1.7): the token checked as a site admin's, kept as
  the forge's `AdminToken` record, and temper's admin's other tokens
  deleted. Only after `TakeOver`'s `Done` and the record kept does the
  host remove the file. It then runs `Reconcile { forge, adopted }` with
  the root's restored repositories (`reconcile.md`). A fake then runs its
  start-time adoption (`playground.md`), before `Request::Connect` with access built
  from the records (../forge/engine.md, 1, 2). The forge is
  connecting from the start or `started` until its `Connected`; its git
  value comes from its `GitToken` record (../forge/engine.md, 5).
- **A present handover is always taken over.** With an `AdminToken`
  record and no handover file, the host reconciles from the record. With
  the file present, `TakeOver` replaces the kept token and deletes the
  older tokens; taking the same token again is harmless. No owned or
  fake forge's `api_token`, `git_token`, `user` or `webhook_secret` file
  is written or read.
- **A box's restart, when it signs its person in** (`box.md`, 1): serve starts
  before the group, so its read at the start comes before preparation
  has made the new handover. That start's `Reconcile`, from the kept
  `AdminToken`, waits for the forge (skein's `forge/setup/forgejo.md`, 1.4), and
  Forgejo runs only after preparation has ended. So once that
  `Reconcile` is done, before the first repository or the connect, the
  host reads `admin_token` once more. A file there is taken over,
  removed and reconciled again, as above; with none, the forge goes on.
  The forge stays connecting throughout, so it reads connected only with
  the handover gone. This second read is once per start, for a forge
  that start restored, and a stop along the way is a stop as below.
- **A step that stops blocks that forge's connect.** A `TakeOver`,
  `Reconcile`, `Create` or `Adopt` stop, a handover read that fails, or a removal
  that fails leaves the forge not connected, named by the plan's step or file
  operation, never a secret; at `started`, it is set up failed. A
  `TakeOver` stop leaves the file; a failed removal blocks reconcile.
  The row says not connected after a start; at `started`, set up
  failed with its `why`, the step and a short reason within the wire's
  256 bytes, a file operation naming the file as "forge N's
  admin_token". The operator's note names the plan's step and its
  reason, and for a file operation the file's full path; never the
  file's bytes. The next start or ask takes that handover over again. The
  engine and every other forge run on. "Set up again"
  asks the box, which answers a started group `started` at once (`box.md`, 1),
  and the engine runs the same sequence again.
- **Then reconcile runs as managed's** (`reconcile.md`), and makes no org:
  the set up's org is its first repository's, made by `Create` (7).
- **Then the first repository,** at the box's `started`, when the set
  up asked for one (7): `Create`, before the forge's connect.
- **A forge not prepared,** an owned forge's record with neither a
  handover nor an `AdminToken` record, is not set up: no access and no
  connect; every other forge starts as it would. The deployment's page
  (`../../client/forge/deployment.md`):
  - **Under the box,** offers "Set up a Forgejo here" while no owned
    forge is kept and the machine's part names the box's owned Forgejo
    (`init.md`). Its card says CI jobs run here unisolated, Forgejo
    stops with `temper box`, and its files are kept in temper's data
    directory. `SetUp` makes the forge's record, owned, numbered next
    (../forge/engine.md, 2), with its first repository's intent (7), in its
    decision, and the routing asks the box for the forge's group (`box.md`, 1);
    at `started FORGE` the engine takes the handover, reconciles, makes
    the first repository (7) and connects; at `failed FORGE WHY` the
    page names the step and offers it again, "Set up again", on the
    same record. A box runs one owned Forgejo: a `SetUp` of a new one
    while one is kept is refused *not offered*, or *busy* while that one
    sets up.
  - **Without the box** (`box.md`, 4), offers none: an owned forge's group is
    the box's, and `SetUp` is refused *not under the box*.
- **The page's actions** are the engine domain's offers, from the same
  function that gives the set up's offer (`../../domain/forge/adoption.md`, 1).
  "Connect as its admin" makes a Forgejo elsewhere from the address the
  person gives (`reconcile.md`, 5).
- **The connected card** says "Connected as LOGIN", with its id on an
  Account line. Access is "Site admin, with temper's own token" for
  owned, "Site admin, with a token you gave" for managed, and "Its bot's
  tokens, which you gave" for connected. Owned's Runs line names Forgejo
  and its runner on this machine, their versions, and CI jobs running
  here as the person. Its First password line names the path to
  `person_password`, never its password, from set up on: the person's
  first sign-in, through temper too, links their account by it
  (`../openid-provider.md`, 4), and temper keeps no record of that sign-in by
  which to drop the line after it. A stopped owned
  forge says "Restart temper box: its preparation mends it". Owned and
  managed offer "New repository", with the set up's organization and
  repository fields (`README.md`, section 9).
- **The person** signs in on Forgejo's web as a site admin with
  `person_password`, and may change it when they like; makes
  repositories in their orgs, with Actions,
  and adopts them through temper's client. temper's admin is temper's; a
  person who deletes it makes the next start's reconcile and the next
  preparation stop, naming it.
- **The forge's place,** in its record (../forge/engine.md, 1), is the
  machine's owned Forgejo's (`init.md`): `api` its address, `transport`
  plaintext, `git` `http://<address>`.

## 6. Fetching

`temper-box forge fetch [DIRECTORIES] FORGE`, before an owned forge's
preparation, for a forge whose `forgejo` or `runner.binary` is
`"fetched"` (`init.md`): a once step of the box (`box.md`, 1), and a command of
its own for any other host.

- **What it fetches,** each binary its setup gives as `"fetched"`, at
  temper's pins (1), for the machine's architecture, `ARCH`:

  | Binary | From | Into |
  |---|---|---|
  | Forgejo | `https://codeberg.org/forgejo/forgejo/releases/download/vVERSION/forgejo-VERSION-linux-ARCH` | `<cache>/forgejo/VERSION/forgejo` |
  | the runner | `https://code.forgejo.org/forgejo/runner/releases/download/vRUNNER_VERSION/forgejo-runner-RUNNER_VERSION-linux-ARCH` | `<cache>/forgejo-runner/RUNNER_VERSION/forgejo-runner` |

  `ARCH` is `amd64` or `arm64`; another machine refuses, naming its
  architecture, and its binaries are given by path.
- **One request, one answer.** Each URL answers `200` with the binary,
  never a redirect: skein-http follows none, and any other status
  refuses, naming the URL and the status.
- **A mirror,** the owned entry's `mirror`, absent by default (`init.md`
  ): an `https` origin for each binary, in place of codeberg.org's and
  code.forgejo.org's, or of `ghcr.io` on macOS, its paths and digests the
  pins' still. A host that
  keeps its own copies names it; the process tier needs none, its
  binaries placed where the fetch looks (`testing.md`, 4).
- **TLS with the machine's roots,** read at the command's start as
  skein's shell reads them (skein's `runtime/shell.md`, 6.2): the bundle
  `SSL_CERT_FILE` names, or the first of the distributions' bundles that
  exists. skein-tls embeds none, so a machine with neither refuses the
  command.
- **The digests are pinned,** one SHA-256 per binary and architecture,
  in `skein-forge` beside `VERSION` and `RUNNER_VERSION` (skein's
  `forge/README.md`, 4.3), and required: on Linux a fetched binary is the build
  the conformance checked, byte for byte.
- **Streamed, never held:** the body's chunks are asked for one at a
  time, as the file takes them; each is hashed into the digest, which
  is incremental, and written at its offset into `<file>.part`, private,
  within `setup.fetch_bytes` (../configuration/limits.md). At the
  body's end, a digest equal to the pin makes the part file `0o755` and
  renames it over its name.
- **Its size is the answer's `Content-Length`,** the `200`'s that carries
  the body, so past a redirect on macOS. A body longer than it removes
  the part file and is refused, naming the URL, as one past
  `setup.fetch_bytes` is. An answer without one, or of none, has no
  size, and its progress is its bytes alone. skein-http delivers no byte
  past the length, so the refusal is the machine's own check.
- **It says how far it has gone on stdout,** a plain line a machine
  reads and a person running the command by hand reads too: `fetched
  BINARY DONE of TOTAL bytes`, or with no size `fetched BINARY DONE
  bytes`, `BINARY` being `forgejo` or `runner`, `DONE` the bytes written
  and `TOTAL` the size. One at the binary's start, one at each whole
  percent it passes, at most one for a chunk, and one at its end, once
  each; with no size, one at each MiB in place of the percents. A binary
  already there says nothing, and on macOS the bytes are the archive's,
  its extraction saying nothing. A line stdout will not take is dropped:
  no fetch fails for what it says.
- **A wrong digest** removes the part file and refuses, naming the URL,
  the pinned digest and the digest it got. Nothing is left under the
  binary's name.
- **A file already there is hashed,** never fetched again: its pin's
  digest is nothing to do; another is refused, naming its path, and
  never overwritten, so a binary a person put there stays theirs to
  remove.
- **A part file an interrupted fetch left** is written again from its
  start.
- **Then the preparation checks its version** as every command that
  runs it does (1); a digest is no version.
- **Its code:** the fetch's machine in `temper-fetch`, its
  requests, chunks, writes, digests and lines its inputs and outputs, and its
  host in `temper-fetch-shell`, on skein-io, skein-http and skein-tls,
  which a bottle's extraction is given to by its caller (`README.md`, section 7).
  Forgejo's pins and the bottle's extraction stay `temper-forge-setup`'s
  and `temper-forge-setup-shell`'s, which re-export the machine's items
  and the host's failure under their own paths, and `setup::Origin`. It holds no credential and writes no record
  (`README.md`, section 5). The platform is the binary's to say, as its shell was
  built for: the machine is given it.
- **On macOS** neither Forgejo nor its runner publishes a build, so the
  fetch takes Homebrew's bottles, each a keg in a tarball, at addresses
  fixed by the pins' digests. Homebrew is a URL and nothing more: no
  `brew`, no `/opt/homebrew`, no request to its formula API. `DIGEST` is
  the pin's archive digest, the bottle for macOS 15, which later
  systems run too:

  | Binary | From | The archive's member | Into |
  |---|---|---|---|
  | Forgejo | `https://ghcr.io/v2/homebrew/core/forgejo/blobs/sha256:DIGEST` | `forgejo/VERSION/bin/forgejo` | `<cache>/forgejo/VERSION/forgejo` |
  | the runner | `https://ghcr.io/v2/homebrew/core/forgejo-runner/blobs/sha256:DIGEST` | `forgejo-runner/RUNNER_VERSION/bin/forgejo-runner` | `<cache>/forgejo-runner/RUNNER_VERSION/forgejo-runner` |

  An Intel Mac is refused, naming its architecture. A file already
  there is held to the binary's digest. Three things differ from
  Linux's fetch:
  - **A token that is no credential:** the first request carries
    `Authorization: Bearer QQ==`, the anonymous value Homebrew sends, to
    `ghcr.io` alone.
  - **One redirect:** `ghcr.io` answers `307` with its storage's address.
    The fetch follows one redirect, to `https`, without the token;
    another, or one to anything else, refuses, naming the URL. Linux's
    fetch still takes none.
  - **An archive:** the body goes to `<file>.archive`, private, hashed
    as it is written, as the part file above, against the archive's
    digest, within `setup.fetch_bytes`. Only on a match is the one
    member, by its exact name, taken out by the machine's own
    `/usr/bin/tar` into `<file>.part`, within the same bound; its digest
    is checked against the binary's, it is made `0o755` and renamed over
    its name, and the archive removed. A wrong digest, a bound passed or
    a failed extraction removes both files and refuses, and an
    interrupted fetch starts again from nothing.
- **A bottle gone:** Homebrew promises nothing about old blobs. The
  fetch refuses, naming the URL and saying what answers it: a binary
  given by path, or the owned entry's `mirror`, which serves the same
  archive at the same `/v2/homebrew/core/…/blobs/sha256:DIGEST` path,
  under the same digests and the same one redirect, and is sent no
  token.
- **Claude Code,** a `programs.claude_code.path` of `"fetched"` (`init.md`, 2):
  temper's pinned build, by this section's machine and host, in what
  follows only where it differs from Forgejo's binaries. No step of the
  box and no `temper-box forge fetch` runs it: the first command of its
  source that needs it does (`../../draft/claude-provider.md`, 3), its
  sign-in, its status or a run, so a machine that never chooses Claude
  never downloads it. Declaring a source has each host report its login
  (jig's `domain/accounts.md`, 5), so the fetch may begin with a status,
  before any sign-in: declaring is choosing Claude.

  | Binary | From | Into |
  |---|---|---|
  | Claude Code | `https://downloads.claude.ai/claude-code-releases/VERSION/PLATFORM/claude` | `<cache>/claude-code/VERSION/claude` |

  - **The pin:** `VERSION` `2.1.311`, Anthropic's stable channel on
    2026-11-05, and for each `PLATFORM` a SHA-256 and a size, from that
    release's `manifest.json`, in `temper-fetch`'s `claude_code`
    module; a new pin is temper's release:

    | `PLATFORM` | SHA-256 | Bytes |
    |---|---|---|
    | `linux-x64` | `86dc1fcc69498159eaf15f446613e847010f9036bfbbdffd4c49350be8489ada` | 293,776,752 |
    | `linux-arm64` | `2e2496c9a8065df93acb0fca2d341f59a6b8777649b2e025ab8c9bcd5eef61f3` | 293,658,808 |
    | `linux-x64-musl` | `cd40dd327145ee47aa0deacd571801c1d782388f709d71242eee6b91de8ed11f` | 287,442,392 |
    | `linux-arm64-musl` | `172150e3f9ea07e8e2e2e29862d7d9a25f5c234d8e909e4063df237665bf09b9` | 285,822,152 |
    | `darwin-arm64` | `85d8c08707fcfd1e78ffa0204eefbc6ec9e4fe22f375fb2767ef85d957ed3161` | 276,185,504 |
    | `darwin-x64` | `d34e1cb82015b050471a97574aa6ea6f698ecf3a97a61c980961f43ad9f838f5` | 287,420,928 |

    The binary says its platform, as its shell was built for: its
    system, its architecture and, on Linux, whether it was built for
    musl. Another machine refuses, naming its architecture; a `claude`
    on `PATH` answers it, at the next `temper-box init`.
  - **One form everywhere:** the same URL shape on Linux and macOS, one
    request and one answer, no bottle and no mirror: a machine that
    cannot reach it uses its own `claude`.
  - **Its bound is the pin's size,** not `setup.fetch_bytes`: no
    document is at hand, and the size is known. A body past it removes
    the part file and refuses, naming the URL.
  - **Its launcher:** smith takes an absolute program path, and the
    worker's agent runs with neither temper's directories nor an
    environment. So every read of `agent.json` hands smith, in place of
    `"fetched"`, the launcher's path, `claude-code/claude` beside the
    programs directory (`<data>/claude-code/claude` as init writes it),
    a symbolic link to the worker's agent's program, its `agent` member's
    `program` (../configuration/README.md, section 4): the box's `temper-box`, or a
    deployment's `temper-agent`, and smith, jig and the agent never meet
    the word. A
    `temper-box` or a `temper-agent` (`../binaries.md`, 3) run by the name
    `claude`, as the playground's git is (`playground.md`), is the launcher: it takes
    the cache from `cache` beside it, a symbolic link to the cache
    directory,
    fetches the build if it is missing, and replaces itself with it,
    its arguments unchanged. `temper-box serve`, a worker alone and `temper-box
    program-login` each write both links at their start, when the path
    is `"fetched"`, in that directory, private: a link pointing elsewhere
    is replaced by a new one and a rename, and a right one left as it is,
    so that two starts at once are harmless and a person's own command
    never fetches by a stale name. The launcher, the placing of its
    links and the substitution are `temper-launcher`'s (`README.md`, section 7),
    which a worker's commands and the agent's binary both link and which
    checks the links' directory private through smith's
    `smith-agent-document`, never smith's agent. Claude Code's fetch
    beneath it is `temper-fetch-shell`'s, the fetch's host the forge's
    setup links too, which links no smith crate, so the engine links no
    launcher (`../binaries.md`, 1). The `claude` link names the agent's
    program, whichever binary places it, so that the program it runs is
    the one the worker's agents run (`../../draft/deployment-binaries.md`,
    section 4).
  - **Found by its size:** every command of the source passes through
    the launcher, so it hashes nothing it finds. A file at the pin's
    size runs; one of any other size is refused, naming its path, and
    never overwritten. The digest is checked when the build is fetched,
    byte for byte; a wrong one removes the part file, refuses, and
    leaves nothing under the binary's name.
  - **At most once at a time:** the launcher waits on an exclusive lock,
    `<cache>/claude-code/VERSION/claude.lock`, then looks again, so of
    two commands at once one fetches and the other finds it. A command
    killed at its own deadline, a status's or a run's, fails as that
    command, and leaves its part file to the next, which writes it again
    from its start; `temper-box program-login`, which has no deadline,
    finishes it.
  - **It says so on stderr,** stdout being the command's own: `fetching
    Claude Code`, then this section's lines, `BINARY` being
    `claude-code`. `temper-box program-login`'s terminal shows them.
  - **Its own updates off:** the launcher sets `DISABLE_AUTOUPDATER=1`
    in the build's environment, so the pin stays the pin. A `claude`
    found on `PATH` is the person's, and keeps its own.

## 7. The first repository

- **Asked with the set up, with no project.** `SetUp`'s `first` is one
  of `Empty { org, name }`, an organization (default `work`) and a
  repository (default `hello`); `Copies { addresses }`, at most 16, each
  as `../../draft/repository-copies.md`, 5.4, has it; or `Nothing`. It
  names no project: `Empty`'s repository is made unadopted, and its
  adoption is a later act's, into a project the person has
  (`../../draft/first-run-projects.md`, 4). `Copies` and `Nothing` make no
  repository at set up: the forge connects with no `Create`, and the
  copies wait for a project to go into (`../../client/pages/README.md`). The
  forge domain keeps it in the forge's record (../forge/engine.md, 1), the set
  up's intent: one per forge, written in the decision that admits
  `SetUp`, and replaced by the next `SetUp` it admits. `temper ctl forge
  setup` and a managed connect's `first` ask `Empty` alone.
- **Asked at a managed connect.** `ForgejoAdmin.first` carries the same
  organization and repository, with no project, and the person's runner label
  (default `docker` in the client and ctl), kept in the forge's record
  as `SetUp`'s are. After Reconcile and its
  `AdminOrigin` write are kept, Create makes the repository
  before the admin token is kept and the forge connects (`reconcile.md`, 5). A missing
  organization is owned by the admin Create has verified. On a forge
  served the listener Create adds the repository's hook, and the forge
  connects with its secret; on another host it adds none (`reconcile.md`, 5).
  The forge's row carries `admin`, the string form's admin-connect offer;
  `first.label`, the runner label; and `ended_stop`, an optional
  `AdminStop`: `NotAdmin`, `Refused { status, stage: SetupStage }`,
  `Unmarked` or `Other`. `SetupStage` names admin, bot, token, org,
  repository, file, app, installation or state. `ended` is the separate
  diagnostic; the client chooses its stop's words from `ended_stop`.
- **Made after reconcile.** At the box's `started FORGE`, or at a start
  while the record keeps an intent whose repository is not made, once
  `TakeOver` and `Reconcile` are done (5), the setup host runs
  `Create { forge, org, repository, files }` (skein's `forge/setup/forgejo.md`,
  1.9) by the admin's token: the org, private and owned by the person;
  the repository, private, with Forgejo's README on `main`; the bot a
  collaborator with `write`, and its hook on the webhooks' listener;
  then the starter workflow, below. Then the forge connects, as before
  (5). Each step finds what is already there and leaves it, so a set
  up asked again finishes what the last one left. The engine's limits
  for it are 2 files of at most 4 KiB each, the starter workflow and,
  for the playground's copy, its `src/main.rs` (skein's
  `forge/setup/forgejo.md`, 1.1).
- **The starter workflow,** `.forgejo/workflows/ci.yml`, its commit
  the bot's, run on a push to `main` and on each pull, on the owned
  runner's label, `temper-host`. At a managed connect it uses `first.label`
  instead: 1–64 ASCII letters, digits, `.`, `_` or `-`, checked at the wire
  and again before the YAML is made. The workflow is otherwise the same:

  ```
  on:
    push:
      branches: [main]
    pull_request:
  jobs:
    check:
      runs-on: temper-host
      steps:
        - run: echo "temper's starter check passed; replace this step with your own checks"
  ```

  Its push is the repository's first CI run, on the owned runner: the
  owned forge enables Actions (2), and Forgejo gives a repository
  made through its API the Actions unit (skein's `forge/setup/forgejo.md`,
  section 2). The step needs no network and no action from elsewhere.
- **The playground's copy,** asked with `SetUp`'s `playground`
  (`../../client/ctl/areas.md`, `forge setup --playground`): the first
  repository is `playground/widget`, its organization and name fixed,
  made with the playground's `src/main.rs` (`playground.md`, `temper-fake`'s
  constant, one source for both) beside the starter workflow, in one
  Create, so that the playground's three suggested requests (`playground.md`) are
  asked of a real forge and a real model. Forgejo's README stays, since
  Create keeps a file already there (skein's `forge/setup/forgejo.md`, 1.9);
  the fake's `check.yml` is not copied, since the box's runner runs the
  starter. ctl's call adopts it, as below, into a new project named
  `widget` unless the call names one. A set up asked again takes its own request's fields, with
  the flag or without: a first repository already made stays, and the
  new one is made beside it.
- **The row says it,** the intent, `starts_with`, and the forge's first
  repository beside its connection (`../../client/forge/deployment.md`;
  `../../client/forge/records.md`): with `Copies` or `Nothing` the first
  repository is none; else none; making `ORG/NAME` from the `SetUp`'s admission; made
  `ORG/NAME`, unadopted, once `Create` is done; or failed
  `ORG/NAME`, at the step that stopped (the admin, the org, the
  repository with its access, or the file). A `Create` that stops
  blocks the connect as any step does (5): the forge is set up
  failed, its `why` naming the step, and "Set up again" asks with the
  same fields. A restart reads the row's first repository from the
  record, making `ORG/NAME`, until `Create` answers again.
- **Adopted by a client's act, never by the set up.** The welcome's
  Ready, its last act, adopts it into the project that act makes
  (`../../client/pages/README.md`), as ctl's call does, below, and the
  project opens without waiting for it; `ConnectForge` adopts nothing:
  there the made repository waits, unadopted, for the person to adopt
  it into a project of theirs from the adoption form
  (`../../client/forge/family.md`, 4), one choice away. A repository the
  person's Ready left refused or unanswered waits the same way, Home
  offering it again.
  `temper ctl forge setup` and `forge connect … --first` hold their
  `--project` in their own run (`../../client/ctl/areas.md`): once the
  row says made, the call sends, under the same party, jig's
  `MakeProject` for a new project, its name and policy 1, the default
  document's (`init.md`, 3), when `--project` names no project, then `Adopt`:
  the project, the forge, `ORG`, `NAME`, landing on `main`, owned, the
  project's home, with CI. Nothing in the engine reads a project for
  it.
- **A restart before the repository is made** keeps the intent, since
  the record holds it: the start's sequence, or the box's `started`,
  makes the repository at the next connect, after reconcile (5).
- **A set up writes as it goes:** each act and plan keeps what it makes
  when it makes it, the forge's record and its intent at `SetUp`'s
  admission, each credential record as its plan writes it (`reconcile.md`, 1), and the
  box's group file and record as the group starts (`box.md`, 1). Nothing is
  staged in memory to be written at the end, and no write spans two
  stores.
- **Found later, it starts over.** A set up cut short, by a restart or a
  failure, leaves its forge kept but not connected, or its first
  repository not made: the welcome's Forge step shows it unfinished,
  "FORGE · not finished", with Start over (`../../client/pages/README.md`),
  which removes the forge (`../../domain/forge/adoption.md`, 1) and opens the
  step's choices again. An owned Forgejo then sets up afresh, as a new
  forge numbered next: its binaries from the cache (6), which its
  removal keeps, and its directory prepared anew (3), the old one
  removed with its forge (`box.md`, 1).
- **GitHub goes on by default:** its App's records are kept as they come
  (`github.md`, 2), so the step offers Go on, to the App's installation, before
  Start over, which removes the forge and names the App on GitHub as the
  person's to delete (`../../domain/forge/adoption.md`, 1).
- **An account added before** stays: it is the person's, not the set
  up's, and the Model step shows it ready (`../../client/pages/README.md`).
- **Its code:** the intent, in the forge's record, in
  `temper-engine-domain-forge`; `Create`'s start and its end in
  `temper-service`'s setup; the starter workflow's bytes and the plan's
  limits in `temper-forge-setup`.

## 8. Signing in through temper

**A box's owned Forgejo needs none of this:** it makes its key at its first run
(`init.md`, 2) and adds its own source (`box.md`, 1), with no secret and no command for the person
(`../openid-provider.md`, 9). What follows is for every other arrangement.

`temper-box forge sign-in [DIRECTORIES] FORGE`, for a Forgejo of
`application.sign_in.forges` (`../openid-provider.md`), managed or
owned, on temper's host, before `temper-box serve`:

1. **The provider's key,** when its file, `engine.clients.provider.key`,
   is missing: `openssl genpkey -algorithm RSA -pkeyopt
   rsa_keygen_bits:3072`, run with umask 077 into a file beside the
   key's and renamed over it, so the key is PKCS#8 PEM, mode 0600 and
   the service's own user's. It is never printed, copied or logged; a
   key there is left as it is. No `openssl` on the `PATH` refuses the
   command, naming it.
2. **The forge's client secret,** when its file, the forge's client's
   `secret`, is missing: 32 bytes of the system's randomness in
   base64url, written whole and private (`README.md`, section 3). A secret there is
   left as it is.
3. **It prints,** never running them: the `add-oauth` command
   (`../openid-provider.md`, 5) with the client's id, the secret file's absolute
   path in `--secret "$(cat PATH)"`, never the secret's bytes, and the
   discovery document's URL at temper's public origin, or its listener's
   with none. The source's callback is the forge's browser origin's
   `/user/oauth2/temper/callback`: its owned setup's `public` when present,
   else its record's `web` when it has one, else its `git` URL
   (`../openid-provider.md`, 1 and 7), matching Forgejo's
   `ROOT_URL` (2). For an owned Forgejo, the command has the binary and
   its `--work-path` and `--config`; and Forgejo's settings of
   `../openid-provider.md`, 4, which an owned Forgejo's `app.ini` already holds
   (2), and a managed one's owner
   writes.

Then `temper-box serve`, and the printed command, once, on Forgejo's host.

- **Printed for `temper-box serve`'s owned Forgejo too, not run** (with o/sign-in,
  2026-10-23), for two reasons: `add-oauth` fetches temper's discovery
  document at once and adds no source when temper does not answer, and
  the preparation runs before `temper-box serve` (`owned.md`, 3, `box.md`, 2); and `add-oauth`
  takes its client secret only as an argument, where the preparation
  puts no secret (3). A source added by Forgejo's command line while
  Forgejo runs works at once, so the order costs a person nothing.
- **Its code:** the command, and the key's and the secret's making, in
  `temper-engine-shell`'s `forge_sign_in.rs`, dispatched from its
  `forge.rs`; the printed bytes in `temper-forge-setup`'s `sign_in.rs`.

## 9. A repository made or copied

- **The service runs what the connector holds**
  (`../../domain/forge/adoption.md`, 5), in the setup's one pull
  (`temper-service`'s `lifecycle::advance`): on a forge connected with no
  writing run live, the connector's oldest `Waiting` row starts, the
  connector told so before the run's first event. A forge whose setup is
  excluded, ended or not connected keeps its rows waiting.
- **`Empty` runs `Create`** and **`Copy` runs `Copy`** (skein's
  `forge/setup/forgejo.md`, 1.9 and 1.10), each with the starter workflow alone
  in `files`, on the forge's label: the record's `first.label` when it
  has one, or else `temper-host` on an owned forge and `docker` on a
  managed one. The plan's limits are 7's; `copy_deadline` is 30
  minutes, a constant of `small`'s.
- **`Copying`** is no credential record: the host's `Read` of it is
  answered from the row, and its `Write` becomes the connector's event,
  answered `Written` once that decision is kept.
- **Its end** goes to the connector: `Done` as made, with the id and the
  branch; `Stop` as failed, with its step, its why and what
  `Event::Said` said. An event the root is too busy to take is held and
  given again, since a row, unlike a view, is kept.
- **The other writing runs wait for it:** the service starts no
  reconcile, adoption's access or `Create` beside a run of a row, and
  one that finds it `Busy` waits as a held `Install` does. A copy may
  last 30 minutes, so a forge's restart and its reconcile wait as long.
- **Adopting** (`../../domain/forge/adoption.md`, 5) is asked by the same pull: a
  row `Made` or `Adopting` with no act in flight, on a connected forge,
  is given to the connector to act.

## 10. Open questions

- **An owned forge made managed** (`README.md`, section 2): an act that makes an
  owned forge's record managed, after which temper stops running
  Forgejo, its directory's `forgejo/` is a stock work path a person
  runs, and its `AdminToken` record, taken over (`owned.md`, 5), stays valid.

- **A lock and an exec in skein's shell,** onto which `owned.md`, 4's module and
  Claude Code's lock and launcher (`owned.md`, 6) move once skein's shell offers an
  exec and a lock held across it, which skein's `Lock` is not (skein's
  `runtime/io/files.md`). A second service that execs is what would ask
  skein for them.

- **A copy of a bottle of temper's own** (`owned.md`, 6), only if a pinned bottle
  leaves `ghcr.io` while a released temper still pins it: until then
  the refusal, `mirror` and a binary given by path answer it, and a copy
  temper hosts is GPL code it distributes, its source to keep beside it
  (`../../draft/macos.md`, section 7).

- **The person's `claude` installed after init** (`owned.md`, 6, `init.md`, 2): taken in
  place of the fetched build, or not; for now init's choice holds until
  the box is made again (`../../draft/claude-provider.md`, 6).

- **Reading a runner's row** (`owned.md`, 3): a preparation goes by
  `runner/config.yaml`, since Forgejo's command line cannot read the
  row. So a row a person removed stays gone until that file is removed
  before a start, which no help page says until one covers removing a
  runner, and `register`, where a preparation still runs it, leaves a
  runner still running elsewhere on a raw row until it starts again.
  Noticing either needs a reader with Forgejo running, at
  `GET /api/v1/admin/actions/runners` as `testing.md`, 3's facts read it, which
  the preparation is not.

- **Overrides of `app.ini`.** Default: none; the settings are temper's.

- **Backups beyond an upgrade's.** Default: the person's, of the forge's
  directory and the store, with the box stopped.

- **A handover made by a Forgejo's own restart** (`owned.md`, 5, `box.md`, 1): a Forgejo
  step that exits under a running serve runs its preparation again,
  which makes a new handover, and no `started` reaches serve for it, so
  the file stays until serve's next start. It needs the box to tell
  serve of that restart, and serve to take a handover on a forge it
  holds connected. `owned.md`, section 5's second read handles serve's own start
  alone.
