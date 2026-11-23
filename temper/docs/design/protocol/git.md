# The checkout's io: git

Provisional, 2026-10-14, revised 2026-11-19. How the worker's protocol
layer runs the checkout's operations (`../domain/worker/checkouts.md`):
the git component, `temper-worker-protocol-git`. For each operation of
`temper-worker-domain-checkout`'s `git::Op`, the fixed sequence of git
invocations it runs, their arguments and environment, how its terminal is
read from what git says, and how its failures are classified; then the
processes, deadlines and cancel, the limits, and how it is tested. The
mechanics are skein's `programming-model.md` (sections 4 and 9) and io's
processes (skein's `runtime/io/processes.md`). The credentials it hands git are
credentials.md's (`credentials-git-identities.md`, section 1). What is open is listed in `git-later.md`, section 1.

## 1. In one page

- **One component, every operation.** The worker's root asks for a
  `git::Op` with a deadline (`Request::Io`) and may cancel it
  (`Request::CancelIo`); the git component runs it and answers with
  exactly one `Event::Done`. It is step code, a child of the worker's
  protocol layer, with its own entrance limits (components.md
  ).
- **A fixed sequence per operation:** at most four git invocations, each
  its own process in a process group of its own, all under the
  operation's one deadline. Which invocation comes next depends only on
  what the last one said, never on a retry: the component never retries
  (components.md).
- **git as it ships, told exactly what to do.** git is run by an absolute
  path: the one a repository's item names, a fake forge's program
  (channel.md, section 4), or else `workers[].git` (configuration/worker.md,
  section 1); with neither, the invocation fails, naming the forge's host.
  It runs in an environment given whole, with settings on the command line
  that turn off what could run another program or outlive the invocation:
  hooks, credential helpers, the file system monitor, background
  maintenance, every transport but HTTPS and HTTP.
- **Credentials in the environment only.** An operation that reaches the
  forge gets its git identity's token as an `http.extraHeader` set
  through `GIT_CONFIG_*`, and a commit its author through `GIT_AUTHOR_*`
  and `GIT_COMMITTER_*`; never in an argument, a URL, a repository's
  configuration or a file (credentials-git-identities.md, section 1).
- **The remote is a URL on the command line.** The engine's routing
  makes each repository's remote from its forge's `git` base, its owner
  and its name (channel.md, section 4); every invocation that reaches the
  forge names that URL itself, so a repository's own configuration never
  says where git connects, but for the lazy fetch of a `CheckOut` or a
  `Merge`, which git runs from `origin`, the URL `Clone` wrote from the
  remote (`git-operations.md`, 1.2).
- **Read by exit status first,** then by git's machine output: porcelain
  lines, NUL-separated paths and object ids. Standard error only
  classifies a failure, read in the C locale, and its tail is kept as the
  run's diagnostic (`PushDiagnostic`).
- **Never forced,** but for one rewrite. A push moves a branch only as a
  fast-forward; an expected head adds a compare-and-swap; a branch is
  created only where none exists. The one exception is `Rewrite`
  (`git-operations.md`, 1.10), a prepare job's candidate onto its branch, under a lease.
- **Commits keep the checkout's ancestry at delivery,** using the worker's HEAD and a
  merge's second parent; an unchanged local commit is kept unless `always` requires a fresh one. Preparation
  and saved-work commits use the parents the domain names. Git's plumbing (`write-tree`,
  `commit-tree`) records both parents, and delivery advances HEAD (`git-operations.md`, 1.7).
- **Make is file operations:** the old workspace renamed aside and a new
  one made; a sweep empties what was set aside, one file operation at a
  time, passing over and reporting what it can't remove (`git-sweep.md`).
- **One store per repository.** A bare repository on the worker's disk,
  outside every workspace, holds a repository's objects for every checkout
  of it, which borrows them through its alternates file: the store is made
  once, a bare clone of the forge's default branch, which takes the
  remote's object format; a checkout's clone asks the forge for nothing,
  and takes the store's format; and a fetch goes into the store and then
  points the checkout's reference (`git-operations.md`, 1.2, 1.3, 1.11;
  `../domain/worker/checkouts.md`).
- **Two git directories per checkout:** the worker's, named with the
  work tree on every invocation, its configuration, hooks and references
  the worker's alone; and the run's, the work tree's `.git`, never read,
  given a view of the worker's (`git.md`, section 3, `git-operations.md`, section 1.13).
- **Stores are blob-less.** A store's clone and every fetch into it carry
  `--filter=blob:none`: the commits and trees whole, no file contents. A
  `CheckOut` or `Merge` names its repository's identity and runs its
  checkout or merge with `origin` as a promisor remote, so git fetches the
  contents it lacks into the checkout, within the operation and under its
  remote deadline (`git-operations.md`, 1.5, 1.6).
- **Kept packed.** Each such fetch leaves a pack, and git repacks nothing
  itself here (section 3), so a prepare's `Pack` repacks a store or a
  repository past 50 packs, keeping every object (`git-operations.md`, 1.14).
- **A deadline or a cancel kills the invocation's process group,** and
  the operation ends only once its process has closed, so nothing of it
  touches the workspace afterwards.
- **Tested** by step tests against transcripts of real git, the git
  component's machine world, the fake git program checked against the
  same transcripts, and real git in the process tier (section 9).

## 2. Where it sits

```
temper-worker-domain            the worker's root: the checkout's Request::Io, Request::CancelIo
  ▲ git::Op and its deadline; Event::Done
the routing                     hands each operation over, with its identity's value from the identities' table
  ▲
temper-worker-protocol-git      the operations in flight: their invocations, pipes, deadlines; the sweep
  ▲ spawn, pipe reads and writes, signal, close; file operations
io                              processes (skein's `runtime/io/processes.md`) and files (skein's `runtime/io/files.md`)
  ▲
the worker's root directory     workers[].root
├── workspaces/<token>/         a workspace, named by the checkout's token in decimal
│   └── <repository>/           a repository's work tree, side by side with the others; its .git the run's (4.13)
├── gitdirs/<token>/<repository>/  the worker's git directory of that work tree, bare (4.2)
├── stores/<token>/             a repository's store, bare, named by the store's token in decimal (4.11), kept across starts
├── making/<m>/                 a store being made, moved into `stores` once whole (4.11)
├── trash/<n>/                  what Make set aside, emptied by the sweep
├── home/                       git's HOME: empty, private
└── agent/                      the agent's program and configuration, pinned at start (../domain/worker/agent-processes.md, section 1)
```

- **The root's requests reach it through the routing,** which keeps no
  state. For an operation that names an identity, the routing reads the
  identity's newest value from the worker's identities' table and hands
  over one token, its login and the author with the operation
  (credentials-git-identities.md, section 1), with the program its repository's item
  named, if one did (`git.md`, section 3): Forgejo's one token, or on GitHub the
  entry whose account id is the repository's owner's, which its
  workspace item carries (channel.md, sections 4 and 6). So the
  component holds a value only while that operation is in flight. An
  operation whose identity has no value in the table fails `Broken`
  without running git: grants arrive with the assignment, so this is a
  fault of the worker's side. One whose GitHub value has no entry for
  the owner's account fails `Refused` without running git, named
  "temper's App is not installed on the account owning <name>"
  (channel.md, section 6).
- **Each operation ends exactly once:** `Done` with its own terminal, or
  `Failed`/`FailedWithOutput` with a fault, and `Cancelled` only if the
  cancel won the race (section 7). Its `owner` token is echoed.
- **The worker's root directory** is opened once at start by the
  worker's service and is the root every spawn and file operation
  resolves beneath (skein's `runtime/io/files.md`). At every start the
  component renames `workspaces`, `gitdirs`, `making` and `home` into `trash` if they
  exist, and makes them afresh, private (0o700), with `trash` and `stores`
  if they are absent. `stores` stays: a cache whose loss costs only clones
  made again, so a restart finds each remote's store made
  (`../domain/worker/checkouts.md`), while what must survive is never on
  the worker (`../domain/worker/README.md`, section 1). A store half made
  when the worker stopped is in `making`, and goes with it; git's `HOME`
  must be empty. It never touches `agent/`, which the
  worker's start writes before its loop and no operation of the
  component's names.
- **What it depends on:** `skein-lib`, `skein-io` and the checkout's
  vocabulary. It never decodes a channel record or reads configuration.

## 3. Invocations

Every invocation has this shape:

```
program    the repository's item's git, else workers[].git: an absolute path (configuration/worker.md, section 1)
argv       git -c <setting> … [--git-dir=<dir> [--work-tree=<tree>]] <command> <options> [--] <operands>
directory  the worker's root, always
environment  given whole (section 5)
pipes      stdin (written, then closed), stdout, stderr
```

**The settings,** on every invocation, in this order, each with `-c`:

| Setting | Why |
|---|---|
| `core.hooksPath=/dev/null` | no hook runs, whatever the repository holds |
| `core.fsmonitor=false` | no file system monitor program or daemon |
| `protocol.version=2` | fetch by any object id (gitprotocol-v2: wants are not limited to advertised refs) |
| `protocol.allow=never` | no transport by default: not `file`, `ssh` or `ext` |
| `protocol.https.allow=always` | the forge's transport |
| `protocol.http.allow=always` | the forge's transport where its record's remotes' base is plaintext, on loopback only (forge/engine.md, 1; configuration/engine.md) |
| `credential.helper=` | the empty value clears the list: no credential helper runs |
| `http.sslVerify=true` | the forge's certificate is always checked |
| `gc.auto=0` | no automatic garbage collection, which detaches and outlives the invocation |
| `maintenance.auto=false` | no automatic maintenance, likewise |
| `rerere.enabled=false` | no recorded resolution changes a merge |
| `merge.conflictStyle=merge` | conflicts carry the three markers `git-operations.md`, section 1.6 reads |
| `submodule.recurse=false` | no submodule is touched |

`CheckOut`'s `git checkout` and `Merge`'s `git merge` add one more, after
these: `-c remote.origin.promisor=true`, which makes `origin` a promisor
remote for that invocation alone, so git fetches from it the file contents
the blob-less store lacks (`git-operations.md`, 1.5). `Pack`'s `git repack` names it too, so
that git repacks the forge's packs apart and keeps their objects (`git-operations.md`, 1.14). No
other invocation names it: one that
finds an object missing fails, and a push's thin pack skips a delta base it
lacks rather than fetching it.

- **Command-line settings win** over the worker's directory's own
  configuration for the keys they name. The rest is that directory's own,
  which `Clone` wrote and no run's git writes (`git-operations.md`, 1.13); a run's writing it
  directly is confinement's to close (`git-later.md`, section 1).
- **Named, never found.** Every invocation runs at the worker's root,
  never `-C`, and names its repository after the settings, by paths
  beneath the root: in a checkout `--git-dir=gitdirs/<token>/<repository>`
  and `--work-tree=workspaces/<token>/<repository>`, the workspace's token
  in decimal and the repository's name one path component
  (`../domain/worker/hosted-runs.md`, 1.1); in a store `--git-dir=stores/<token>`
  alone. `git version` names neither, and `init` and `clone` name the
  directory they make as their operand. So git never looks for a
  repository from where it runs, never reads the `.git` a work tree holds,
  and reads the work tree as files: its `.gitignore` and `.gitattributes`
  are content, and no filter, merge or diff driver runs, since only the
  worker's configuration could name one.
- **Operands.** Object ids are hex the component writes; branches appear
  only inside refspecs the component composes (`refs/heads/<branch>`);
  the remote URL and the directories `init` and `clone` make come after `--`. So no operand can
  be read as an option.
- **A renamed repository.** The remote is made from the owner and name
  the engine's root holds (channel.md, section 4). git follows Forgejo's
  redirect from a renamed repository's old path for its first request
  (`http.followRedirects`, whose default is `initial`), as
  `temper-git-transcripts` records (fact
  `git-follows-a-renamed-repository-redirect`); a remote that changes
  misses the checkout's cache and the worker's store, which clone and fetch
  afresh.
- **Object ids** cross the boundary as the checkout's 32-byte `Commit`.
  From git: exactly 40 or 64 lowercase hex digits, anything else is
  `Broken`; a SHA-1 id becomes its 20 bytes followed by twelve zero
  bytes, a SHA-256 id its 32 bytes. To git: the first 20 bytes as 40
  digits when the last twelve are zero, else all 32 as 64 digits. (A
  SHA-256 id whose last twelve bytes are zero, one in 2^96, is named by
  its first 40 digits, which git reads locally as an unambiguous
  abbreviation.)
- **The program,** for each operation, is the one its repository's
  item names (channel.md, section 4), the fake forge's, which the
  domain carries from the item to each `git::Op` on the repository and
  never reads; or, where the item names none, `workers[].git`. Every
  invocation of an operation runs the one program.
- **No program.** With `workers[].git` `null` and an item naming none,
  nothing is spawned: each invocation fails as git's fatal exit would,
  status 128, its stderr `fatal: no git on this machine for HOST`, HOST
  the remote's host with its port, and an operation without a remote
  (`CheckOut`, `Merge`, `Commit`) names no host, `fatal: no git on this
  machine`; so the operation ends `FailedWithOutput { Broken, diagnostic }`
  (section 6), and the forge's row warns (`../client/forge/deployment.md`).
- **The version.** At the worker's start the component runs `git
  version` once with `workers[].git`, unless `null`, in the environment
  of section 5, and refuses the start, naming the program, when it fails
  or names a version below 2.41, the first with `fetch --porcelain`. A
  program an item names is checked so at its first use, before the
  operation's first invocation: one that fails it fails the operation
  `Broken`, naming the program, without running it. The component keeps
  the last such program it checked and its answer, one at a time, and
  checks a different one afresh. The component is tested against git
  2.47 (section 9).

## 4. Each operation

Moved to [`git-operations.md`](git-operations.md), section 1.

## 5. Credentials and environment

The environment is given whole: what is below, and nothing else. No
variable of the worker's own process reaches git.

| Variable | Value | For |
|---|---|---|
| `HOME` | the absolute path of `home` under the worker's root, an empty private directory | every invocation: no `.gitconfig`, `.netrc` or other file of a user's reaches git |
| `LC_ALL` | `C` | every invocation: the messages section 6 reads |
| `TZ` | `UTC` | every invocation: a commit's dates carry no host's zone |
| `GIT_CONFIG_NOSYSTEM` | `1` | every invocation: no system configuration |
| `GIT_CONFIG_GLOBAL` | `/dev/null` | every invocation: no user configuration |
| `GIT_TERMINAL_PROMPT` | `0` | every invocation: git never asks |
| `GIT_ASKPASS` | empty | every invocation: no askpass program, whatever a repository's `core.askPass` says |
| `GIT_COMMITTER_NAME`, `GIT_COMMITTER_EMAIL` | the identity's author; or `temper` and `temper@temper.invalid` for an operation that names no identity | every invocation: the committer of a commit, or of a reference's log entry |
| `GIT_AUTHOR_NAME`, `GIT_AUTHOR_EMAIL` | the identity's author | `Commit` |
| `GIT_CONFIG_COUNT` | `1` | `Store`'s clone, `Fetch`'s fetch, `Create`, `Push`, `CheckOut`'s checkout, `Merge`'s merge |
| `GIT_CONFIG_KEY_0` | `http.extraHeader` | likewise |
| `GIT_CONFIG_VALUE_0` | `Authorization: Basic ` and the base64 of `<login>:<token>` | likewise |

- **No `PATH`.** git is run by its absolute path and starts its own
  helpers (`git-remote-https`, `index-pack`) from its build's exec path;
  nothing in these invocations runs a program by name, so none can be
  found in a working tree.
- **The identity's value** is the one channel.md, section 6 encodes, as
  the routing hands it over (section 2): one token, Forgejo's or the
  GitHub entry it picked, with the login and the author. The header's
  value is base64 and a fixed prefix, so no byte of the token can break
  it.
- **Never elsewhere:** no credential in an argument, a URL, a
  repository's configuration or a file; no `GIT_TRACE*` or
  `GIT_CURL_VERBOSE`, so git never prints the header, and the diagnostic
  tail a run sees (section 6) holds none.
- **What standalone accepts:** an agent run as the same user could read a
  git process's environment while it runs (credentials.md, section 8).
  Contained trees close it (`git-later.md`, section 1).

## 6. Output and failures

**What is kept** of each invocation:

- **stdout,** up to `stdout_bytes`, which is enough for every output
  `git-operations.md`, section 1 reads. Past it, the bytes are read and dropped, counted, so
  git is never blocked on its pipe, and an operation whose output passed
  the cap ends `Broken`, but for a merge's progress, which is not parsed,
  and its listing of unmerged paths, read as far as it was kept (`git-operations.md`, 1.6).
  A design's changed paths stream through their own admission (`git-operations.md`, 1.9).
- **stderr,** its last `stderr_bytes`, with the count of bytes dropped
  before them. It is read only to classify a failure, and its last
  `PushDiagnostic::CAPACITY` bytes, with that count, are the diagnostic
  (`PushDiagnostic::new`). Every fault, including `Broken` when no
  classification matches, keeps those bytes through push verification
  and into the delivery refusal's `tail`, with its `cut` count.

**How an operation ends,** in this order:

1. **The deadline passed or the cancel won** (section 7): `TimedOut` or
   `Cancelled`.
2. **The spawn failed,** or git ended by a signal: `Broken`.
3. **The exit status and stdout give the outcome** of `git-operations.md`, section 1.
4. **Otherwise, stderr's tail classifies it.** The component reads the
   tail's lines that begin `fatal: ` or `error: `, from the last to the
   first; the first that matches a row below decides, and none matching
   is `Broken`.

| A line of stderr (C locale) | Fault |
|---|---|
| `fatal: repository '…' not found` | `Missing { Repository }` |
| `fatal: couldn't find remote ref refs/temper/counts`, exactly (Counts) | none: the forge has no counts, and `Counts` goes on to its deletion (`git-operations.md`, 1.3) |
| `fatal: couldn't find remote ref …` (Fetch) | `Missing { Branch }` |
| `fatal: remote error: upload-pack: not our ref …` (Fetch) | `Missing { Commit }` |
| `fatal: could not read Username for …`, `fatal: Authentication failed for …` | `Refused`: the forge did not take the token |
| `fatal: unable to access '…': The requested URL returned error: 401` or `403` | `Refused` |
| `fatal: unable to access '…': The requested URL returned error: 404` | `Missing { Repository }` |
| `fatal: unable to access '…': ` and anything else: another status, a refused connection, a name not resolved, a TLS failure, a timeout | `Unreachable` |
| `fatal: the remote end hung up unexpectedly`, `fatal: early EOF`, `error: RPC failed…` | `Unreachable` |

- **Only invocations that reach the forge** (`Store`'s clone, `Fetch`'s
  and `Counts`' fetches, `Create`, `Push`, and `CheckOut`'s checkout and `Merge`'s merge,
  whose lazy fetch may) are classified by the table. A local one that
  fails is `Broken`, as is a checkout's or a merge's failure no row names.
- **The faults** are the checkout's (`git::Fault`): `Missing` where the
  forge lacks what was asked, `Refused` where it refused the identity or
  the update, `Unreachable` where it could not be reached or failed,
  `Broken` for the worker's side and anything git said that the table
  does not name, `TimedOut` and `Cancelled` as section 7 says.
- **With output or without.** A failure after git ran ends
  `FailedWithOutput { fault, diagnostic }`, the diagnostic from the
  invocation that failed; one before any invocation ran (a missing
  identity's value, a failed spawn of the first, a named program that
  fails its version, Make's file operations) ends `Failed { fault }`,
  but for no program, whose diagnostic names the host (section 3).
- **The command it ended in.** The component names, on every
  `FailedWithOutput` it ends with, the invocation the operation was in
  when it failed (`PushDiagnostic::command`): git's subcommand as its argv
  names it (`git-operations.md`, section 1), `checkout`, `clean`, `fetch`, `merge-base` and the
  rest, or, for one of the sequence's file operations, its word: `store`,
  `install`, `alternates`, `reds`, `results`, `scan` or `view`. It is the
  invocation running when the deadline passed, the cancel won or git
  failed, or, once the next was asked of io, that one. A `Failed { fault }`
  names none, nor does a diagnostic made outside the component (no
  identity, an App not installed: `git.md`, section 2); a diagnostic's other
  readers (a delivery's refusal, push verification) take its output and
  cut alone, so no wire changes.
- **The messages are git 2.47's** in the C locale, as the transcripts of
  section 9 record them; a version that words them otherwise classifies
  them `Broken`, which the domain retries as transient and the process
  tier shows.

## 7. Processes, deadlines and cancel

- **One invocation at a time per operation,** each spawned in a process
  group of its own (skein's `runtime/io/processes.md`); contained process trees
  are for hosts apart (jig's `protocol/needs.md`, section 1; `git-later.md`, section 1
  ). git's own children (its remote helper, `index-pack`,
  `pack-objects`) stay in that group. The settings of `git.md`, section 3 leave
  git nothing that would leave the group: no detached maintenance, no
  credential daemon, no file system monitor.
- **An invocation ends** once git has exited, stdout and stderr have
  reached their ends, and io has closed the child, which kills whatever
  is left of its group (skein's `runtime/io/processes.md`). The next invocation
  starts only then.
- **One deadline per operation:** the domain's, from the checkout's
  `remote_timeout` or `local_timeout`. It covers the whole sequence,
  Commit's scan included. When it passes, the component signals `Kill` to
  the current invocation's group, its reason `Deadline` (skein's
  `runtime/io/processes.md`, section 5), and closes it; no further invocation
  starts; the operation ends `TimedOut` once the child has closed.
- **Cancel** does the same, its kill's reason `Cancelled`, and ends
  `Cancelled`. A kill for broken io names `Closed`. An operation whose last
  invocation had already exited, its output read, when the cancel came
  ends with its own terminal: the outcome that won the race (the
  checkout's boundary, `Request::Cancel`). A cancel between two
  invocations starts no further one.
- **Kill at once.** A killed git may leave a lock file or half an index;
  the checkout does not trust a workspace whose operation timed out, was
  cancelled or broke, and makes it again from an empty directory
  (`../domain/worker/checkouts.md`).
- **stdin** is written whole (the message, for `commit-tree`; nothing
  otherwise) and closed; stdout and stderr are read as they come, both at
  once, so git never waits on a full pipe.
- **At the worker's end** the service closes every child, killing their
  groups: in temper's one process, the service closes the worker once
  the engine has drained and no agent process is left (service-process.md,
  section 1; jig's `protocol/service.md`, section 10). Operations in
  flight end as io closes them.

## 8. Entrance limits and kernel submissions

The git component's `Limits`, set by temper's profile `small` and
overridable under `git_component.` (configuration/limits.md):

| Limit | What it bounds | Rule |
|---|---|---|
| `operations` | operations in flight at once | at least the checkout's `workspaces`, since each hold runs one at a time |
| `stdout_bytes` | stdout kept per invocation | at least the largest output `git-operations.md`, section 1 reads under the checkout's limits: the unmerged list (`conflicts` paths of `path_bytes`, each with its NUL), a push's status lines |
| `stderr_bytes` | the stderr tail kept per invocation | at least `PushDiagnostic::CAPACITY` |
| `read_bytes` | one read of Commit's scan | above 0 |
| `sweep_entries`, `sweep_bytes` | one page of the sweep's listing (`git-sweep.md`, 2): entries, and bytes of names | above 0; `sweep_bytes` at least 255, the longest name io lists (skein's `runtime/io/files.md`) |

The start is refused, naming the limit, when a rule does not hold. The
component's one deadline of its own is the sweep's next pass
(`git-sweep.md`, 4): an operation's is the domain's, and io's close
timeout bounds the end of a killed invocation.

- **Memory:** operation state and parsed paths grow as they fill; stdout,
  stderr, reads and sweep pages are bounded at their entrances, and the
  sweep's reported entries at 64 (`git-sweep.md`, 3). No heap
  sum is a startup condition: `check` refuses a limit set only for the
  operations, stdout, stderr, read and sweep rules above.
- **Processes:** at most `operations` git process groups at once.
- **Time:** an operation ends within its deadline and io's close timeout.
- **Kernel submissions:** `KERNEL_SUBMISSIONS = 1`, one file operation
  at the queue edge toward skein-io. Internal hand-offs reserve no room.
- **Disk:** the workspaces are the cache's (`../domain/worker/checkouts.md`
  ); `trash` holds no more than the workspaces it took, until the sweep
  has emptied it of all but what it reports it can't remove
  (`git-sweep.md`, 3).

## 9. Testing

The tiers, their budgets and the fakes are `../testing/README.md`'s. What
checks this document's contract:

- **Step tests** in `temper-worker-protocol-git`:
  - **the builders:** for each operation and case (each `Want`, a push
    with and without an expected head, a commit with and without a second
    parent), the exact argument vector, directory and environment of
    `git.md`, section 3, and `git-operations.md`, section 1, and `git.md`, section 5, a fetch's store and repository and a store's `<m>` in
    `making` among them, the store's clone and fetch blob-less, and a
    checkout's and a merge's promisor setting and header; that no value of
    an identity appears in an argument; the
    message's bytes; both alternates files' bytes; that every invocation
    runs at the root and names its directories (`git.md`, section 3); the view's
    files, its removals and its making again (`git-operations.md`, 1.13); a `Pack`'s count and
    repack in the store and then the repository, each repack only past 50
    packs, and the store's lock: a fetch and a `Pack`'s store part never
    run together in one store, nor two `Pack`s, and waiters take it in
    order;
  - **the parsers:** count-objects' `packs:` line, fetch's porcelain line, push's status lines, NUL
    lists, ids of 40 and 64 digits and malformed ones, the conversions
    both ways, and the classifier of section 6, each against transcripts
    and against their mutations;
  - **the scan's rule** on lines cut across reads, carriage returns, and
    markers of six and eight characters; **the sweep's steps**
    (`git-sweep.md`, 5);
  - **the program:** an item's spawned in place of `workers[].git`, its
    `git version` at first use, again for a different one and never for
    the same, a version below 2.41 failing the operation unrun; and no
    program, each invocation failing 128, naming the host, with nothing
    spawned.
- **Transcripts** of real git 2.47, captured once, outside the suites, by
  `temper-git-transcripts`: every invocation of `git-operations.md`, section 1, and every
  failure of `git.md`, section 6 (a missing repository, branch and commit, a wrong
  token, no permission, a protected branch, a refused connection),
  against skein's throwaway Forgejo (`skein/docs/design/testing/fakes/throwaway.md`, section 1;
  `../testing/process-tier.md`). Each keeps the argument vector,
  exit status, stdout and stderr, masked: object ids, ports, paths and
  times. No credential is ever written. The facts this document relies
  on, named in `git.md`, section 3, and `git-operations.md`, section 1.8, are in `crates/temper-git-transcripts-tool/facts.txt`
  with git's and Forgejo's versions. They are captured again when an
  invocation changes, or to move to another git.
- **The git component's machine world,** `tests/worker/protocol-git`:
  the component between scripted operations above and scripted io below:
  a spawn refused; a process's exit before and after its pipes end;
  output cut at random, past each cap, and empty; an exit by a signal;
  the deadline at every invocation of every sequence; a cancel at every
  invocation and racing an exit; a missing identity's value; file
  operations failing; a `Store` whose store another made first; the
  sweep's stories (`git-sweep.md`, 5); a view whose every step in the
  run's directory fails once, made again, and fails twice. Its referee: each operation ends exactly once, with the outcome
  its scripted outputs imply; no invocation starts after a deadline or a
  cancel; every child is closed before its operation's terminal; no
  credential in an argument; scenario memory peaks under their ceilings.
  Its scripted outputs are the transcripts.
- **The fake git program,** its core `crates/temper-fake-git` and its
  hosted half `crates/temper-fake-git-hosted` (`../testing/fakes.md`,
  3), answers this document's invocations as real
  git does, and is checked against the same transcripts: each
  transcript's arguments and environment, run on the fake over the
  equivalent fixture, give the same exit status, the same stdout once
  masked, and stderr lines that classify alike.
- **Real git, focused:** the checkout world's `suite`, on a throwaway
  Forgejo: a run's `.git` rewritten as a file naming another directory, a
  filter driver, a hook path and an alias in its configuration, its
  references moved, and the worker's next operations running none of it
  and delivering the work tree's files; a run's own git at its start and
  after a delivery; a checkout made with one directory taken on; a merge's
  commit past `conflicts`.
- **The process tier** runs the machine's git, 2.41 or later, against
  its owned Forgejo: clone, fetch, create, check out, commit, push, in
  its story (`../testing/process-tier.md`).

## 10. Open questions

See [The checkout’s git: open questions](git-later.md).
