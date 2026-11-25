# The forge's deployment records

This document holds the forge's deployment records, split from [`records.md`](records.md).

## 1. Deployment reads
- **The playground's read, `playground`:** of the deployment's scope,
  with no fields, and no subject: what it answers changes with the
  `Playground` row, made by a start and erased by the reset (`family.md`, 7;
  `../../protocol/setup/playground.md`). Its answer is none, with no row, or
  `PlaygroundAnswer`: its forge's number (u16); its project (u32),
  once its start has made it, else none; its model's LLM provider
  (u32); its repository's owner and name (byte strings, bounded as
  `Adopt`'s); its state, `PlaygroundState`, starting, running or
  resetting; and its cues, at most 8, each 1 to 256 bytes of text, in
  the configuration's order (`../engine.md`). On the wire it is
  `Reading`'s `playground`, `Reading` naming which of the family's
  reads a read is; none is the read's empty answer.
- **The deployment's read, `deployment`:** of the deployment's scope,
  with no fields and no subject: what it answers is the document's, so
  it changes only with a start. Its answer is always a record,
  `DeploymentAnswer`: `welcome` (bool), whether the document shows the
  welcome (`../../protocol/configuration/engine.md`); and `owned`
  (bool), whether Forgejo on this machine can be set up: the document
  has `application.machine.owned` and the engine runs under the box
  (`../../protocol/setup/box.md`, 4). A first run's store keeps no forge, so
  no row of the `forges` watch can say so (`deployment.md`), and a wait
  for a host reads it as the fact of a box (`../pages/accounts-words.md`, 1); and
  `program_login` (optional bytes, at most 8,192), *the printed command*
  below; and `push` (optional bytes, 65), the public half of the
  deployment's VAPID key, which a browser subscribes with, none while
  push is off (`../../domain/forge/attention.md`, 7.3), the record's last
  field. On the wire it is `Reading`'s `deployment`, beside `playground`.
- **The printed command:** what the pages print works when pasted. Where
  the deployment's one worker is on the engine's machine
  (`../../protocol/configuration/README.md`, section 1, the standalone shape),
  `program_login` is that machine's whole command but NAME, the running
  binary's word and the three directories in effect, each as a word
  (*The words of a command for this machine*): `~/temper-box program-login
  --config '…' --data '…' --cache '…'`. The pages print it with NAME after "On this machine,
  run:". It is a path of a machine that the party signed in to it
  may read: a read carries a sign-in (jig's `protocol/client/README.md`), and no
  answer goes before one. A path that isn't UTF-8, a line past its
  bound, and a deployment whose workers are elsewhere send none: the pages
  then print the placeholders the tools do, `temper-box program-login --config
  CONFIG --data DATA --cache CACHE NAME`, "On each host, with its
  directories:". Once the owner's `program_environment` answer comes, it
  replaces this command: an empty answer prints `temper-box program-login NAME`,
  and any other prints its words followed by `program-login NAME`.
- **The program's environment, `program_environment`:** of the deployment's
  scope, with no fields and no subject: what `temper-box program-login` needs
  around it to act on this deployment, from this machine's shell. It is the
  party's read, and only the deployment's owner has it, at any time, since the
  welcome's Model step shows them the command during set up and Connect a model
  after it; anyone else's is refused, and their page prints the command without
  its words. Its answer is
  the record `ProgramEnvironmentAnswer`: `environment` (bytes), the words the
  command starts with, which a page draws before `program-login NAME`. They are
  none when `temper-box` runs this binary and each of the three directories
  (`../../protocol/configuration/directories.md`) is the one the command finds by itself
  in the environment the engine started in. Otherwise they are the word that
  runs the binary, after `TEMPER_CONFIG=…`, `TEMPER_DATA=…` and `TEMPER_CACHE=…`,
  in that order, each only where its own directory isn't that one: a box with
  its data alone moved says `TEMPER_DATA=~/x temper-box`. Where that environment
  gives the command no directories by itself, no `HOME` or XDG variable placing
  them, all three are named. It is one string, composed where the
  binary and the directories are known, so that the client holds no path as a
  fact, quotes nothing and decides no default; it is a read of its own,
  separate from `DeploymentAnswer`. A page asks when its address names a
  program's way, the only place that draws the command with these words;
  Accounts' has none of them (`../README.md`, section 5). On the wire it is `Reading`'s
  `program_environment`.
- **The words of a command for this machine:** how `program_environment`'s are
  made.
  - *The word that runs the binary.* The running process's executable, less the
    ` (deleted)` Linux adds to one rebuilt while it runs. The word is `temper-box`
    when the first executable file named `temper-box` on the process's `PATH` is
    that file, by canonical paths: only the first counts, as that is what a
    shell runs, and an entry of `PATH` that isn't absolute counts for nothing,
    since the person's shell stands elsewhere. Otherwise it is the executable's
    path, as a word (below). An executable that can't be read gives `temper-box`.
  - *A path as a word,* for the executable and the three `TEMPER_` values alike:
    repeated slashes collapsed, then a leading `$HOME`, whole components of it,
    as `~/`, and the path alone as `~`. It stays unquoted where it can, so that
    the shell expands `~/`, which it does right after `=` in an assignment
    prefix too. A path needing quotes quotes only what follows `~/`
    (`~/'my dir/temper-box'`), and whole where no `$HOME` leads it (`'/my dir/temper-box'`).
    `$HOME` unset, relative or `/` leads nothing.
  - *The bound.* The words fit the answer's 49,216 bytes: a word for the binary
    that would pass it gives way to `temper-box`.
## 2. The deployment’s forges
- **The deployment's forges, the subject `forges`** (`deployment.md` and `served.md`
  ): of the deployment's scope, with no fields, its window
  unused; on the wire `Watching`'s `forges`. Its document, `Forges`, a
  `ForgeRow` for each forge the store keeps and has not ended, at most
  `FORGES` (16), in number order, the playground's fake among them while
  it runs (`../../protocol/forge/engine.md`, 1): its number (u16); its canonical resource `host` (bytes, at most
  2,048, from the git authority including its explicit port, the same
  identity adoption uses); its kind (Forgejo or GitHub); its mode
  (connected, managed, owned or fake); and its connection: not
  connected; not set up; setting up, with its step when the box has
  named one (fetch, prepare or run), and while it fetches and the box
  has said how far, `fetched`: the binary (Forgejo or its runner), the
  bytes written (u64) and the size (u64), if known
  (`../../protocol/setup/owned.md`, 6), which a client reads as none where the
  size is zero or the bytes pass it, and which changes in place, no
  version kept for an older side; set up failed, with its optional step (fetch, prepare, run or connect)
  and why (text of
  at most 256 bytes, the box's line bound); connecting; connected, with
  the user's id (u64) and login (a byte string bounded as `Adopt`'s);
  or stopped, with the component's reason;
  `admin`, whether the forge is managed, its connect taking a site
  admin's token; `read_only`, its App's read-only choice
  (`../../protocol/setup/github.md`, 7), false on Forgejo; `ended_stop`, none
  or `AdminStop`: not admin, refused with status (u16) and `SetupStage`
  (admin, bot, token, org, repository, file, app, installation or state),
  unmarked, or other. The diagnostic `ended` stays separate; the client
  selects its stop's words from this typed fact, never its text;
  its first repository, `first`: none, or the set up's org, name,
  whether it is the playground's copy (`playground`, appended) or the
  sample (`sample`, appended after it), runner
  label (at most 64 bytes) and its state, `FirstState`, with no project:
  making; made, with the repository's id (u64);
  or failed, with its step, `CreateStep`: admin, org, repository (its
  bot's access with it) or file (`../../protocol/setup/owned-repositories.md`, 1); its
  intent, `starts_with`, `StartsWith`: `repository`, that `first` or none
  known yet; `copies`, `CopyAddresses`, the set up's, kept after they are asked;
  or `nothing`; its
  installations, `installations`, on GitHub, at most
  `INSTALLATIONS` (64, the configuration's `forges.installations` at
  most it) in id order, empty on Forgejo, each
  `Installation`: its id (u64), its account, none until the forge has
  heard it or its id (u64) and login (a byte string bounded as
  `Adopt`'s), whether it is suspended and whether it is a stray, never
  accepted through temper's link (booleans); its last connect's end,
  `ended`: none, or `ConnectStopped`, why the run of a `Connect` or an
  `Install` stopped, or, on GitHub, how the way in its new tab ended
  without an installation accepted (`served.md`, 1, *Each way's end
  reaches the forge's row*), text of at most 256 bytes, in the setup
  host's or the routing's words (`deployment.md`), its shape unchanged;
  on GitHub, its App's way, `app`: none,
  `Manifest` or `Pasted`, how the App its records hold was connected
  since the engine started, the forge domain's memory alone, so none
  after a restart and while no App is connected (`deployment.md`,
  Connecting); its `web` address, none on a forge with none; the
  repositories adopted on it, `repositories`, at most 16, each
  `Adoption`, its owner, name and project (u32); on an owned Forgejo,
  `owned`, `OwnedForge`, the Forgejo and runner versions the box pins
  (at most 64 bytes each, the runner's optional) and the path of the
  person's first password's file, never the password (at most 512
  bytes; `deployment.md`); `makings`, the repositories being made or
  failed on it, at most `MAKINGS` (64) in key order, each
  `RepositoryMaking`: its org and name, its `from` and `into`, and its
  state, `MakingState`: waiting; making; made, with the id (u64) and the
  branch (bounded as `Adopt`'s landing branch); adopting, with both; or
  failed, `MakingFailed`, with its step (`MakeStep`: admin, org,
  repository, file or adoption), its reason (`MakeFailure`: taken, empty,
  timed out, refused with status (u16), not allowed or other) and the
  forge's words (at most 256 bytes); its
  offer, the forge domain's (`../../domain/forge/adoption-connection.md`, 1): offered, not
  under the box, or not offered, which is what "Set up a Forgejo here", "Set up again"
  and the words without the box follow (`deployment.md`); and `catching_up`,
  none, or while the forge catches up after a restart its `CatchingUp`
  (`../../domain/forge/catching-up.md`, 5): `fresh`, `gathers`, `ci`,
  `goal_files`, `lost_writers`, `goal_branches` and `uncertain`, each a
  u32, delivered again as they change. Beside the rows,
  `linked`, whether the printed link has signed a person in since the
  engine started (jig's `protocol/client/sign-in.md`, `LinkSignedIn`):
  the clients' component's memory alone, as `app` is the forge domain's, so
  false after a restart; the engine delivers the document again when it
  turns true. After `linked`, `full`, the forge connector's tables full
  past their report bound (`../../protocol/service-process.md`, 1.3), at most 2,
  each `FullTable`: its `table`, `FullKind` (outbox or read admission),
  `since`, the wall time it filled in nanoseconds (u64), and `words`, the
  holders' words after the engine's line's `AGE: ` (at most 1,024
  bytes); empty while none is, the forge domain's memory alone, so empty
  after a restart until a table fills again; the engine delivers the
  document again as one is added, its words change or it goes. A field
  added to a versioned record changes the client wire's layout, its
  version frozen (`../../protocol/wires.md`, 9). A TUI in front of a box reads it for its welcome's "Signed in
  as the owner" (`../tui/box.md`). The read of the forges, the
  tools' `forge list` (`../ctl/areas.md`, area 6), is the watch's first
  document. Text prints `host: HOST`, and JSON carries `host` unchanged
  for a charter document's `forge` (`../../protocol/configuration/settings.md`, 6).
  *The client's forge side watches it once,* for the
  session, from when `me` is first delivered while the link is live,
  for every person, so that the engine judges the role: a watch
  refused or ended is kept as such, with no rows, and not opened again
  until the next sign-in. Its rows feed the Forges page,
  `ConnectForge`, the welcome and its store's state (`../pages/README.md`), and the
  Forges entry's dot.

## 3. Open questions

None.
