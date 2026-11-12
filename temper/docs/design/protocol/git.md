# The checkout's io: git

Provisional, 2026-10-14, revised 2026-10-20. How the worker's protocol
layer runs the checkout's operations (`../domain/worker/checkouts.md`):
the git component, `temper-worker-protocol-git`. For each operation of
`temper-worker-domain-checkout`'s `git::Op`, the fixed sequence of git
invocations it runs, their arguments and environment, how its terminal is
read from what git says, and how its failures are classified; then the
processes, deadlines and cancel, the limits, and how it is tested. The
mechanics are skein's `programming-model.md` (sections 4 and 9) and io's
processes (skein's `runtime/io/processes.md`). The credentials it hands git are
credentials.md's (`credentials.md`, section 4). What is open is listed in `git.md`, section 10.

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
  (channel.md, section 4), or else `workers[].git` (configuration/README.md,
  section 3); with neither, the invocation fails, naming the forge's host.
  It runs in an environment given whole, with settings on the command line
  that turn off what could run another program or outlive the invocation:
  hooks, credential helpers, the file system monitor, background
  maintenance, every transport but HTTPS and HTTP.
- **Credentials in the environment only.** An operation that reaches the
  forge gets its git identity's token as an `http.extraHeader` set
  through `GIT_CONFIG_*`, and a commit its author through `GIT_AUTHOR_*`
  and `GIT_COMMITTER_*`; never in an argument, a URL, a repository's
  configuration or a file (credentials.md, section 4).
- **The remote is a URL on the command line.** The engine's routing
  makes each repository's remote from its forge's `git` base, its owner
  and its name (channel.md, section 4); every invocation that reaches the
  forge names that URL itself, so a repository's own configuration never
  says where git connects, but for the lazy fetch of a `CheckOut` or a
  `Merge`, which git runs from `origin`, the URL `Clone` wrote from the
  remote (4.2).
- **Read by exit status first,** then by git's machine output: porcelain
  lines, NUL-separated paths and object ids. Standard error only
  classifies a failure, read in the C locale, and its tail is kept as the
  run's diagnostic (`PushDiagnostic`).
- **Never forced,** but for one rewrite. A push moves a branch only as a
  fast-forward; an expected head adds a compare-and-swap; a branch is
  created only where none exists. The one exception is `Rewrite`
  (4.10), a prepare job's candidate onto its branch, under a lease.
- **Commits keep the checkout's ancestry at delivery,** using HEAD and a
  merge's second parent; an unchanged local commit is kept unless `always` requires a fresh one. Preparation
  and saved-work commits use the parents the domain names. Git's plumbing (`write-tree`,
  `commit-tree`) records both parents, and delivery advances HEAD (4.7).
- **Make is file operations:** the old workspace renamed aside and a new
  one made; a sweep empties what was set aside, one file operation at a
  time.
- **One store per repository.** A bare repository on the worker's disk,
  outside every workspace, holds a repository's objects for every checkout
  of it, which borrows them through its alternates file: the store is made
  once, a bare clone of the forge's default branch, which takes the
  remote's object format; a checkout's clone asks the forge for nothing,
  and takes the store's format; and a fetch goes into the store and then
  points the checkout's reference (4.2, 4.3, 4.11;
  `../domain/worker/checkouts.md`).
- **Stores are blob-less.** A store's clone and every fetch into it carry
  `--filter=blob:none`: the commits and trees whole, no file contents. A
  `CheckOut` or `Merge` names its repository's identity and runs its
  checkout or merge with `origin` as a promisor remote, so git fetches the
  contents it lacks into the checkout, within the operation and under its
  remote deadline (4.5, 4.6).
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
│   └── <repository>/           a repository's working tree and its .git, side by side with the others
├── stores/<token>/             a repository's store, bare, named by the store's token in decimal (4.11), kept across starts
├── making/<m>/                 a store being made, moved into `stores` once whole (4.11)
├── trash/<n>/                  what Make set aside, emptied by the sweep
└── home/                       git's HOME: empty, private
```

- **The root's requests reach it through the routing,** which keeps no
  state. For an operation that names an identity, the routing reads the
  identity's newest value from the worker's identities' table and hands
  over one token, its login and the author with the operation
  (credentials.md, section 4), with the program its repository's item
  named, if one did (section 3): Forgejo's one token, or on GitHub the
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
  component renames `workspaces`, `making` and `home` into `trash` if they
  exist, and makes them afresh, private (0o700), with `trash` and `stores`
  if they are absent. `stores` stays: a cache whose loss costs only clones
  made again, so a restart finds each remote's store made
  (`../domain/worker/checkouts.md`), while what must survive is never on
  the worker (`../domain/worker/README.md`, section 1). A store half made
  when the worker stopped is in `making`, and goes with it; git's `HOME`
  must be empty.
- **What it depends on:** `skein-lib`, `skein-io` and the checkout's
  vocabulary. It never decodes a channel record or reads configuration.

## 3. Invocations

Every invocation has this shape:

```
program    the repository's item's git, else workers[].git: an absolute path (configuration/README.md, section 3)
argv       git -c <setting> … <command> <options> [--] <operands>
directory  beneath the worker's root: the repository's directory, the workspace's for Clone's init, the store's for Clone's format and Fetch's fetch, or `making` for Store's clone
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
| `merge.conflictStyle=merge` | conflicts carry the three markers section 4.6 reads |
| `submodule.recurse=false` | no submodule is touched |

`CheckOut`'s `git checkout` and `Merge`'s `git merge` add one more, after
these: `-c remote.origin.promisor=true`, which makes `origin` a promisor
remote for that invocation alone, so git fetches from it the file contents
the blob-less store lacks (4.5). No other invocation names it: one that
finds an object missing fails, and a push's thin pack skips a delta base it
lacks rather than fetching it.

- **Command-line settings win** over the repository's own configuration
  for the keys they name. The rest is the repository's own, which `Clone`
  wrote; standalone trusts it until contained trees keep agents out of
  the git directory (section 10).
- **The working directory is the spawn's,** resolved by io beneath the
  worker's root, never `-C`: a repository's directory is
  `workspaces/<token>/<repository>`, with the workspace's token in
  decimal and the repository's name, one path component
  (`../domain/worker/README.md`, 4.1), and a store's is `stores/<token>`.
- **Operands.** Object ids are hex the component writes; branches appear
  only inside refspecs the component composes (`refs/heads/<branch>`);
  the remote URL and Clone's and Store's directories come after `--`. So no operand can
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

| Operation | Invocations | Ends in |
|---|---|---|
| `Make` | none: two file operations, and one for each store it names | `Succeeded` |
| `Clone` | 3, and a file's write | `Succeeded` |
| `Fetch` | 2: one in the store, one in the repository | `Fetched { commit }` |
| `Create` | 1 | `Succeeded`, `Exists` |
| `CheckOut` | 2 | `Succeeded` |
| `Merge` | 1 or 2 | `Merged`, `Conflicted { files }` |
| `Commit` | 1 to 4; a delivery adds 2 to 5, and a scan of files for a merge | `Committed { commit }`, `Unchanged`, `Conflicted { files }` |
| `Push` | 1, or 2 with an expected head | `Succeeded`, `Rejected` |
| `IsAncestor` | 1 | `IsAncestor { contains }` |
| `Store` | 1, a clone, and two or three file operations | `Succeeded` |
| `Point` | 1 | `Succeeded` |

Each may also end failed, with the fault of section 6. Below, `<url>` is
the operation's `remote`, `<dir>` its repository's name, and ids are as
section 3 writes them. The settings of section 3 precede each command.

### 4.1 Make

`Make { workspace, stores }`: the workspace's directory, empty, and its
references in each store it names gone.

1. `Rename` `workspaces/<token>` to `trash/<n>`, `<n>` the component's
   next serial number in decimal, counted from a number the component
   draws from its seed at each start, so a name an earlier start left in
   `trash` is not taken again. A workspace that does not exist is
   nothing to move.
2. `MakeDirectory` `workspaces/<token>`, mode 0o700.
3. For each store, in order, `Rename` `stores/<store>/refs/temper/<token>`
   to `trash/<n>`, the workspace's references there (4.3): a git killed
   while it fetched for this workspace may have left a lock among them,
   which would refuse the next fetch. One that does not exist is nothing
   to move.

`Succeeded` once all are done; a failed file operation is `Broken`. So
what the cache evicted, or a worker left behind before a restart, never
fails a prepare (`../domain/worker/README.md`, section 8), and Make takes two
file operations however large the old tree was.

**The sweep** empties `trash`, which io cannot do in one operation (io
never removes a tree: skein's `runtime/io/files.md`). While `trash` is not
empty and the worker is not shutting down, the component runs one file
operation at a time:

- it lists a page of `trash` (`Scan`, within `sweep_entries` and
  `sweep_bytes`) and takes the first entry;
- it tries to remove the entry as a file; one that is a directory it
  lists a page of, and each child it removes as a file, or, if the child
  is a directory, renames to `trash/<n>`;
- it removes a directory once its listing is empty.

Every directory it lists is at most two levels below the worker's root,
so the sweep holds one page, whatever the depth of the tree.
Nothing is followed through a link. What is left at shutdown is swept
after the next start. A sweep's failure is counted and the entry is left
for the next pass, which starts at the next `Make` or the next start.

### 4.2 Clone

`Clone { at, remote, store }`: the repository, empty, borrowing the
store's objects. It names no identity: nothing reaches the forge.

1. In the store's directory, `stores/<store>`:
   ```
   git rev-parse --show-object-format
   ```
   whose stdout is the store's format, `sha1` or `sha256`, and a newline;
   anything else is `Broken`. A repository borrows only from a store of its
   own format.
2. In the workspace's directory, which `Make` left without it:
   ```
   git init --quiet --object-format=<format> -- <dir>
   ```
3. The file `<dir>/.git/objects/info/alternates` written whole, holding
   `../../../../../stores/<store>/objects` and a newline: git reads a
   relative alternate from the repository's object directory, so the file
   names no path of the machine, and a run's own git reads the store too.
4. In the repository's directory:
   ```
   git remote add origin -- <url>
   ```
   so `remote.origin.url` is the remote, as `git clone` wrote it, and no
   reference is fetched.

- **Exit 0 from all three, and the file written:** `Succeeded`. Any other
  exit, or a failed file operation, is `Broken`. A repository the forge
  lacks is the store's `Missing { Repository }` (4.11), or, once its store
  is made, the first fetch's (4.3).
- **Commits and trees, no checkout.** No `--depth`: the store holds every
  commit and tree of what was fetched, and no file contents (4.11), which
  `CheckOut` and `Merge` fetch into the repository as they need them, from
  `origin`, as the remote `remote add` named (4.5, 4.6). The working tree
  is made by `CheckOut`.
- **Borrowed, never shared.** The repository's configuration, references,
  index and own objects are its own; only the store's objects are read
  through it. The store is written by the worker's git alone.

### 4.3 Fetch

`Fetch { at, store, remote, want, into, identity }`: two invocations.

1. In the store's directory, `stores/<store>`:
   ```
   git fetch --porcelain --verbose --filter=blob:none --no-tags --no-write-fetch-head --no-recurse-submodules -- <url> +<source>:refs/temper/<token>/<name>
   ```
2. In the repository's directory:
   ```
   git update-ref refs/temper/<name> <new-id>
   ```

where `<source>` is `refs/heads/<branch>` for `Want::Branch`, the commit's
full id for `Want::Commit`, and `HEAD`, the forge's default branch, for
`Want::Default`; `<token>` is the repository's workspace's token in
decimal; and `<name>` is `fetched` for `Into::Fetched` and `landing` for
`Into::Landing`. So `<into>` below is `refs/temper/fetched` or
`refs/temper/landing` in the repository, and the store keeps each
workspace's last fetches under its own token.

- **Exit 0 and one porcelain line** `<flag> <old-id> <new-id> <store-ref>`,
  its flag one of space, `+`, `*` or `=`: the second invocation, and then,
  at its exit 0, `Fetched { commit: <new-id> }`. `--verbose` makes git
  print the line when the reference was already there. A line naming
  another reference is `Broken`, and so is any other exit of the second.
- **Only what the store lacks.** git offers the forge every reference the
  store holds, each workspace's last fetches, so the forge sends only the
  objects none of them reaches, and of those the commits and trees alone:
  `--filter=blob:none` keeps the store blob-less (4.11) whether or not it
  was made so. The second invocation transfers nothing. The repository
  reads them through its alternates file (4.2).
- **`refs/temper/fetched`** keeps what was fetched reachable, in the store
  under the workspace's token and in the repository, and is moved freely by
  the next fetch.
- **`refs/temper/landing`** is the landing pin: the landing branch's tip as
  prepare fetched it, or, for a writable item that names no landing branch,
  the head its push branch had (or its start's commit, where that branch
  does not exist yet), which the project's checks take their base from
  (`tools/pre-pr/design.md`, section 2). Only prepare's one pin writes it,
  a fetch into `Into::Landing` or a `Point` (4.12) at a commit the prepare
  fetched (`../domain/worker/checkouts.md`), forced, since a cached
  checkout's pin from an earlier hold may be anywhere; no other operation
  names it, so it stays where prepare put it for the hold.
- **These two are the references the component writes** in a repository,
  and their twins under the workspace's token in a store.
- **Missing:** `Repository` for a repository the forge does not have,
  `Branch` for a branch, or a default branch, it does not have; `Commit`
  for a commit it does not have (section 6). A full id is required: git
  looks an abbreviation up as a reference name.
- **A push that ended ambiguously** is verified by the domain with a
  fetch of its branch (`../domain/worker/checkouts.md`): no other
  invocation is needed for it.

### 4.4 Create

`Create { at, remote, branch, commit, identity }`: the branch, only if
the forge has none.

```
git push --porcelain --force-with-lease=refs/heads/<branch>: -- <url> <commit>:refs/heads/<branch>
```

The lease with an empty expected value requires that the branch not
exist (git-push(1)); the forge's receive applies it as a compare-and-swap
on the old value. The status line of `refs/heads/<branch>` decides:

| Flag and summary | Ends in |
|---|---|
| `*` `[new branch]` | `Succeeded` |
| `=` `[up to date]`: it exists, at `commit` | `Exists` |
| `!` `[rejected] (stale info)`: it exists elsewhere | `Exists` |
| `!` `[remote rejected] (…)`, its reason ending in `declined` | the fault `Refused`, with its diagnostic |
| any other, or no status line | the fault of section 6 |

### 4.5 CheckOut

`CheckOut { at, commit, clean, identity, account }`: the working tree
exactly `commit`'s tree.

```
git -c remote.origin.promisor=true checkout --force --detach <commit>     with the identity's header (section 5)
git clean -ffdxq                                                          only with clean
```

- **The contents it lacks.** The store is blob-less (4.11), so the
  checkout finds the commit's trees there and not every file's contents:
  git fetches those it lacks from `origin` in one batch before it writes
  the tree, into the repository's own objects, where the next checkout of
  the repository finds them. The fetch is git's own (`git fetch origin
  --filter=blob:none --stdin`, its object ids on stdin), a child in the
  invocation's process group, run with the invocation's settings and
  environment, so the identity's header reaches it and nothing else does.
  It runs under the operation's one deadline, the remote one (section 7),
  and a failure of it fails the invocation, classified by section 6's
  table.
- **The first** detaches HEAD at the commit, sets the index to its tree,
  so that nothing stays staged and a file staged new goes with it,
  overwrites what is in the way, tracked or not, and clears a merge's
  state (`MERGE_HEAD` and the rest). **The second** removes every untracked and ignored file and
  directory, nested repositories included.
- **Exit 0 from both:** `Succeeded`; any other exit of the checkout is
  classified by section 6's table, and of `git clean`, `Broken`.

### 4.6 Merge

`Merge { at, theirs, identity, account }`: merge fetched `theirs` into the
checked-out commit, without committing, leaving conflicts and the merge's
state on disk.

```
git -c remote.origin.promisor=true merge --no-commit --no-ff -- <theirs>     with the identity's header (section 5)
git diff-files --name-only -z --diff-filter=U          only after exit 1
```

- **The contents it lacks:** those of the files the merge reads, theirs,
  the base's and, for renames, the candidates', fetched as `CheckOut`'s are
  (4.5), in batches, under the operation's remote deadline. A failure of
  that fetch ends the merge with an exit other than 0 and 1, classified by
  section 6's table.
- **Exit 0:** `Merged`, a commit already contained in HEAD ("Already up
  to date") included: the next commit still records both parents
  (section 4.7). A producer's start asks first, and runs no merge of a
  base its branch contains (section 4.9).
- **Exit 1, and a list that is not empty:** `Conflicted { files }`, the
  unmerged paths, NUL-separated, relative to the repository's top, in
  git's order. One longer than its `path_bytes` among those kept is
  `Broken`.
- **More than the checkout's `conflicts`** is still `Conflicted`, with
  the first `conflicts` paths. A listing past `stdout_bytes` is read as
  far as it was kept, and a last path that was cut is dropped; section
  8's rule for `stdout_bytes` keeps room for `conflicts` whole paths. The
  merge's state stays on disk whole, every file marked in the tree and
  unmerged in the index, whatever their count (2026-11-09). The merge command's stdout is human progress, never parsed: it is
  drained and bounded, and truncating it cannot change exit 0 or exit 1.
  The later NUL listing alone reads a kept prefix; every other parsed
  invocation's stdout past its bound is `Broken`, as section 6 says.
- **Exit 1 with an empty list,** as for a commit git cannot merge, or any
  other exit: `Broken`.
- **`diff-files`,** the plumbing command, lists the index's unmerged
  entries whatever the repository's diff settings say.
- **The committer** git requires even with `--no-commit` is the
  identity's author (section 5); Merge makes no commit.
- **Who commits it** is the domain's (`../domain/worker/checkouts.md`): a
  producer's clean merge and each head of a base with heads merged in at
  once, any other merge at its run's first push or save.

### 4.7 Commit

`Commit { at, parent, merging, always, title, body, identity, snapshot }`: the working
tree exactly as it is, on the parents the domain names, or those the checkout
holds when `snapshot` names the push remote and its account. `always` makes
the commit even when the tree is the parent's: the next commit of a hold
after an unchecked save (`../domain/worker/checkouts.md`). The message,
given on stdin, is `title`, then, when `body` is not empty, a blank line and
`body`, then a newline. `.gitignore` is honoured: an ignored file is not
part of the tree. **A hidden log is no part of a change:** a file named
`.*.log`, at any depth, is what a tool left in the workspace. It is never
staged, for a save or a delivery alike, whatever the repository ignores, so
it reaches no branch and never lands. One the parent already tracks keeps the
parent's contents. Every other file of the tree is taken, a new one included,
since a run's new work is as untracked as a stray file: only this name and the
repository's own ignore rules tell them apart.

**A checked delivery or unchecked save** (`snapshot` is present) first reads the checkout, with no other
writer, before the staging and commit sequence below:

```
git log -1 --format='%H %P' HEAD --                    HEAD and its committed parents
git rev-parse --verify --quiet MERGE_HEAD             exit 1: no merge in progress
```

HEAD replaces `parent`. A different HEAD is already a replacement for an
unchecked save, so it clears `always`; a matching local commit is kept.
MERGE_HEAD, when present, replaces `merging`.
Without MERGE_HEAD, a changed HEAD means a producer's committed work and
clears the remembered `merging`; an unchanged HEAD retains an explicit
second parent the domain named, even if git wrote no MERGE_HEAD. More than
two committed parents, or more than one MERGE_HEAD, refuses delivery.
Before staging, a second parent in progress, or HEAD's committed second
parent when no merge is in progress, is checked against the remote:

```
git fetch --quiet --prune --no-tags --no-write-fetch-head --no-recurse-submodules -- <remote> '+refs/heads/*:refs/temper/delivery/heads/*' '+refs/tags/*:refs/temper/delivery/tags/*'
git for-each-ref --count=1 --format='%(objectname)' --contains <second> refs/temper/delivery/
```

The fetch prunes obsolete references, so a cached remote tip cannot vouch
for a commit the remote no longer reaches. An empty listing refuses with
`delivery second parent is not reachable from the branch's remote`.
The fetch alone carries that account's credential, under the operation's
remote deadline. Local metadata and listings obey the usual stdout bounds;
a malformed id is `Broken`. A MERGE_HEAD listing with more than one id
refuses, naming that delivery supports at most two merge parents.

Without an in-progress merge and with `always` false, `diff-index` exit 0 returns
`Committed { commit: HEAD }`, preserving a local commit, including its
parents, message and identity. An edited tree commits on HEAD. With a merge
in progress, the sequence below always records both parents. After a new
commit, delivery runs `git reset --mixed -q <commit>`: HEAD moves, the index
is that commit's tree, working files stay, and the merge state clears.
Its failure fails the operation; no push follows. A later delivery therefore
reads the preceding delivery as HEAD. An unchanged existing commit needs
no reset.

**Without a second parent** (`merging` is `None`):

```
git add --all -- . ':(exclude,glob)**/.*.log'
git diff-index --cached --quiet <parent> --            exit 0: Unchanged, or Committed { HEAD } for a snapshot; skipped with `always`
git write-tree
git commit-tree <tree> -p <parent> -F -
```

**With a second parent** (`merging` is `Some(m)`):

```
git diff-files --name-only -z --diff-filter=U          the originally conflicted paths still unmerged in the index
                                                       then the scan of those files, below
git add --all -- . ':(exclude,glob)**/.*.log'
git write-tree
git commit-tree <tree> -p <parent> -p <m> -F -
```

- **`Committed { commit }`:** `commit-tree`'s stdout, one id.
- **`diff-index` exit 1** means the tree changed; any exit but 0 and 1 is
  `Broken`.
- **The scan.** Before anything is staged, each listed path is read
  beneath the repository's directory, without following a link, in
  chunks of `read_bytes`. A path holds a marker when one of its lines,
  split at newlines with one trailing carriage return dropped, begins
  with seven `<` and a space, is exactly seven `=`, or begins with seven
  `>` and a space. Paths with markers end the operation `Conflicted {
  files }`, in the list's order, and nothing is staged, so the index
  still knows them at the next attempt. A path that is gone, a directory
  or a link holds no marker: deleting a conflicted file resolves it, as
  `git add --all` then records.
- **Why plumbing.** The domain always records both parents of a merge,
  also when git found `theirs` already merged and wrote no `MERGE_HEAD`,
  and git refuses to write that pseudo-reference through `update-ref`.
  `commit-tree` records exactly the parents asked, signs nothing and runs
  no hook.
- **Without a delivery snapshot, HEAD is not moved,** nor the merge's
  state cleared, by a commit: HEAD and the merge's state are `CheckOut`'s.
  These commits compare and record against the parents the domain names.
  Delivery reads and advances HEAD as above.
- **The author and committer** are the identity's author (section 5); the
  date is the clock's, in UTC.

### 4.8 Push

`Push { at, remote, commit, branch, expected, identity }`: `commit` to
the branch as a fast-forward, never forced.

Without an expected head:

```
git push --porcelain -- <url> <commit>:refs/heads/<branch>
```

With `expected: Some(e)`:

```
git merge-base --is-ancestor <e> <commit>              exit 1: Rejected, and nothing is pushed
git push --porcelain --force-with-lease=refs/heads/<branch>:<e> -- <url> <commit>:refs/heads/<branch>
```

- **Why both.** A lease alone overrides the fast-forward rule: with the
  remote at `e`, git would replace `e` with a commit that does not
  descend from it. The ancestry check makes the update a fast-forward
  first; the lease then makes the forge apply it only if the branch is
  still at `e`. `--force` and a `+` refspec are never used.
- **`merge-base` exits** 0 (go on), 1 (`Rejected`), anything else
  `Broken`.

The push's status line decides:

| Flag and summary | Ends in |
|---|---|
| space (a fast-forward), `*` `[new branch]`, `=` `[up to date]` | `Succeeded`: the branch is at `commit` |
| `!` `[rejected] (non-fast-forward)`, `(fetch first)`, `(stale info)` | `Rejected` |
| `!` `[remote rejected] (…)`, its reason ending in `declined` | the fault `Refused`, with its diagnostic: a protected branch, a hook |
| any other, `+` included, or no status line | the fault of section 6 |

Forgejo refuses a push to a protected branch with that `[remote
rejected]` line, its reason ending in `declined`, as
`temper-git-transcripts` records (fact
`protected-push-is-remote-rejected-declined`).

A push that fails in a way that may have landed (`Unreachable`, `Broken`,
`TimedOut`, `Cancelled`) is verified by the domain (section 4.3).

### 4.9 IsAncestor

`IsAncestor { at, ancestor, commit }` asks whether fetched `ancestor` is
contained in fetched `commit`, including equality. It is local, changes no
file or ref, and needs no forge credential:

```
git merge-base --is-ancestor <ancestor> <commit>
```

Exit 0 ends `IsAncestor { contains: true }`, exit 1 ends it with false,
and other exits fail as `Broken`. The same command inside Push retains
section 4.8's separate contract: exit 1 rejects that push. The checkout
uses this operation to compare an existing producer head with the landing
tip frozen by its prepare, and to ask whether a start's checkout already
contains a head or a producer's base it would merge
(`../domain/worker/checkouts.md`); the comparison alone is never a push
receipt.


`DiffPaths { at, before, after }` admits a design delivery's changed paths:

```
git diff-tree --no-commit-id --name-only --no-renames -r -z <before> <after> --
```

The complete stdout stream is read as NUL-terminated paths, including
additions and deletions. Each name stays within `path_bytes`; no count of
changed files is imposed by the merge-conflict limit. The component keeps
only the current path and the first path not ending in `.md`, and returns
`Paths` with that first non-Markdown name or no names. An incomplete,
empty or oversized path fails as `Broken`. A later code path cannot hide
behind a long list of Markdown files or the stdout retention cap.

### 4.10 Rewrite

A prepare job's (`../domain/worker/jobs.md`). `Rewrite { at,
remote, commit, branch, expected, identity }`: the branch moved to
`commit` only if it is still at `expected`, whether or not `commit`
descends from it:

```
git push --porcelain --force-with-lease=refs/heads/<branch>:<e> -- <url> <commit>:refs/heads/<branch>
```

- **Only for a prepare job's candidate,** onto the change's own branch
  or a landing's: the checkout refuses it for any other item.
- **The lease is the guard:** a branch not at `e` is `Rejected`, and
  nothing moves. `--force` and a `+` refspec are still never used.

### 4.11 Store

`Store { store, aside, remote, identity, account, git }`, `git` the
program (section 3): `stores/<store>` a bare clone of the remote's default
branch, unless one is there, made whole before it is moved into place, so
that a store there is always whole, a fetch that follows finds it so, and
two `Store`s of one store at once, which the cache never starts
(`../domain/worker/checkouts.md`), would still leave one.

1. Where `aside` names a store (one no workspace borrows, whose slot the
   cache gives another remote), `Rename` `stores/<aside>` to `trash/<n>`;
   one that does not exist is nothing to move.
2. Where `stores/<store>` is there, a directory, `Succeeded`, with no clone
   and no invocation: a store kept from before the worker's start, or made
   since, is whole and its remote's, since its token is its remote's
   (`../domain/worker/checkouts.md`), and the fetches that follow bring it
   up to date. Something there that is not a directory is `Broken`.
3. `MakeDirectory` `making/<m>`, mode 0o700, `<m>` the component's next
   serial number, the one `trash` names take, and in `making`:
   ```
   git clone --bare --filter=blob:none --single-branch --no-tags --quiet -- <url> <m>
   ```
   which git accepts into the empty directory.
4. `Rename` `making/<m>` to `stores/<store>`, never replacing: where
   `stores/<store>` is there already, `making/<m>` is renamed to
   `trash/<m>` instead.

- **Exit 0 and the file operations done:** `Succeeded`, whichever rename
  took, or at step 2 with nothing made. A failed clone is classified by
  section 6's table, as a fetch is (a repository the forge lacks is
  `Missing { Repository }`, a token it refused `Refused`); any other
  failed file operation is `Broken`; and what step 3 made is renamed to
  `trash` if it can be. A worker stopped mid-clone leaves it in `making`,
  which its next start sets aside (section 2).
- **A clone, for the object format.** `git init` makes a SHA-1 repository,
  and git 2.47's fetch from a SHA-256 remote into it fails (`fatal:
  mismatched algorithms: client sha1; server sha256`); a clone takes the
  remote's format, an empty remote's too, which `Clone` then reads (4.2).
  `--single-branch` brings the forge's default branch alone, the history a
  first prepare most likely fetches next, so that fetch brings nothing
  more; nothing else is fetched but what an assignment names (4.3).
- **Blob-less.** `--filter=blob:none` brings every commit and tree of the
  default branch and no file contents: on one deployment's forge, 2.3 s
  and 50 MB where the whole clone took 17.4 s and 198 MB (2026-11-12). git
  records the store as a partial clone of `origin`; nothing in the store
  reads a file's contents, and a checkout fetches those it needs (4.5).
  A store made whole before stores were blob-less stays so, and serves as
  well.
- **Its configuration is the clone's:** `remote.origin` names the remote
  and its default branch, and no fetch reads it: every fetch names its URL
  on the command line (section 3). Its `refs/heads/<default>` stays where
  the clone put it. Nothing of a workspace's reaches it but objects the
  forge sent and references under the workspace's token.
- **Never maintained:** `gc.auto=0` and `maintenance.auto=false` hold for
  it as for every repository, so nothing prunes an object a checkout may
  borrow (`../domain/worker/checkouts.md`, section 1).

### 4.12 Point

`Point { at, into, commit }`: the repository's `into` reference at
`commit`, which the repository already reaches through its store, with
no fetch. Local; it names no identity. In the repository's directory:

```
git update-ref refs/temper/<name> <commit>
```

`<name>` as in 4.3. Exit 0: `Succeeded`; any other is `Broken`.

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
  Contained trees close it (section 10).

## 6. Output and failures

**What is kept** of each invocation:

- **stdout,** up to `stdout_bytes`, which is enough for every output
  section 4 reads. Past it, the bytes are read and dropped, counted, so
  git is never blocked on its pipe, and an operation whose output passed
  the cap ends `Broken`, but for a merge's progress, which is not parsed,
  and its listing of unmerged paths, read as far as it was kept (4.6).
  A design's changed paths stream through their own admission (4.9).
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
3. **The exit status and stdout give the outcome** of section 4.
4. **Otherwise, stderr's tail classifies it.** The component reads the
   tail's lines that begin `fatal: ` or `error: `, from the last to the
   first; the first that matches a row below decides, and none matching
   is `Broken`.

| A line of stderr (C locale) | Fault |
|---|---|
| `fatal: repository '…' not found` | `Missing { Repository }` |
| `fatal: couldn't find remote ref …` (Fetch) | `Missing { Branch }` |
| `fatal: remote error: upload-pack: not our ref …` (Fetch) | `Missing { Commit }` |
| `fatal: could not read Username for …`, `fatal: Authentication failed for …` | `Refused`: the forge did not take the token |
| `fatal: unable to access '…': The requested URL returned error: 401` or `403` | `Refused` |
| `fatal: unable to access '…': The requested URL returned error: 404` | `Missing { Repository }` |
| `fatal: unable to access '…': ` and anything else: another status, a refused connection, a name not resolved, a TLS failure, a timeout | `Unreachable` |
| `fatal: the remote end hung up unexpectedly`, `fatal: early EOF`, `error: RPC failed…` | `Unreachable` |

- **Only invocations that reach the forge** (`Store`'s clone, `Fetch`'s
  fetch, `Create`, `Push`, and `CheckOut`'s checkout and `Merge`'s merge,
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
- **The messages are git 2.47's** in the C locale, as the transcripts of
  section 9 record them; a version that words them otherwise classifies
  them `Broken`, which the domain retries as transient and the process
  tier shows.

## 7. Processes, deadlines and cancel

- **One invocation at a time per operation,** each spawned in a process
  group of its own (skein's `runtime/io/processes.md`); contained process trees
  are for hosts apart (jig's `protocol/README.md`, section 10; `git.md`, section 10
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
  the current invocation's group and closes it; no further invocation
  starts; the operation ends `TimedOut` once the child has closed.
- **Cancel** does the same and ends `Cancelled`. An operation whose last
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
  the engine has drained and no agent process is left (README.md,
  section 6; jig's `protocol/service.md`, section 10). Operations in
  flight end as io closes them.

## 8. Entrance limits and kernel submissions

The git component's `Limits`, set by temper's profile `small` and
overridable under `git_component.` (configuration/limits.md):

| Limit | What it bounds | Rule |
|---|---|---|
| `operations` | operations in flight at once | at least the checkout's `workspaces`, since each hold runs one at a time |
| `stdout_bytes` | stdout kept per invocation | at least the largest output section 4 reads under the checkout's limits: the unmerged list (`conflicts` paths of `path_bytes`, each with its NUL), a push's status lines |
| `stderr_bytes` | the stderr tail kept per invocation | at least `PushDiagnostic::CAPACITY` |
| `read_bytes` | one read of Commit's scan | above 0 |
| `sweep_entries`, `sweep_bytes` | one page of the sweep's listing: entries, and bytes of names | above 0; `sweep_bytes` at least 255, the longest name io lists (skein's `runtime/io/files.md`) |

The start is refused, naming the limit, when a rule does not hold. The
component has no deadline of its own: an operation's is the domain's,
and io's close timeout bounds the end of a killed invocation.

- **Memory:** operation state and parsed paths grow as they fill; stdout,
  stderr, reads and sweep pages are bounded at their entrances. No heap
  sum is a startup condition: `check` refuses a limit set only for the
  operations, stdout, stderr, read and sweep rules above.
- **Processes:** at most `operations` git process groups at once.
- **Time:** an operation ends within its deadline and io's close timeout.
- **Kernel submissions:** `KERNEL_SUBMISSIONS = 1`, one file operation
  at the queue edge toward skein-io. Internal hand-offs reserve no room.
- **Disk:** the workspaces are the cache's (`../domain/worker/checkouts.md`
  ); `trash` holds no more than the workspaces it took, until the sweep
  has emptied it.

## 9. Testing

The tiers, their budgets and the fakes are `../testing/README.md`'s. What
checks this document's contract:

- **Step tests** in `temper-worker-protocol-git`:
  - **the builders:** for each operation and case (each `Want`, a push
    with and without an expected head, a commit with and without a second
    parent), the exact argument vector, directory and environment of
    sections 3 to 5, a fetch's store and repository and a store's `<m>` in
    `making` among them, the store's clone and fetch blob-less, and a
    checkout's and a merge's promisor setting and header; that no value of
    an identity appears in an argument; the
    message's bytes; the alternates file's bytes;
  - **the parsers:** fetch's porcelain line, push's status lines, NUL
    lists, ids of 40 and 64 digits and malformed ones, the conversions
    both ways, and the classifier of section 6, each against transcripts
    and against their mutations;
  - **the scan's rule** on lines cut across reads, carriage returns, and
    markers of six and eight characters; **the sweep's step** over a
    listing;
  - **the program:** an item's spawned in place of `workers[].git`, its
    `git version` at first use, again for a different one and never for
    the same, a version below 2.41 failing the operation unrun; and no
    program, each invocation failing 128, naming the host, with nothing
    spawned.
- **Transcripts** of real git 2.47, captured once, outside the suites, by
  `temper-git-transcripts`: every invocation of section 4, and every
  failure of section 6 (a missing repository, branch and commit, a wrong
  token, no permission, a protected branch, a refused connection),
  against skein's throwaway Forgejo (`skein/docs/design/testing/fakes/forge.md`, section 6;
  `../testing/process-tier.md`). Each keeps the argument vector,
  exit status, stdout and stderr, masked: object ids, ports, paths and
  times. No credential is ever written. The facts this document relies
  on, named in `git.md`, sections 3 and 4.8, are in `crates/temper-git-transcripts-tool/facts.txt`
  with git's and Forgejo's versions. They are captured again when an
  invocation changes, or to move to another git.
- **The git component's machine world,** `tests/worker/protocol-git`:
  the component between scripted operations above and scripted io below:
  a spawn refused; a process's exit before and after its pipes end;
  output cut at random, past each cap, and empty; an exit by a signal;
  the deadline at every invocation of every sequence; a cancel at every
  invocation and racing an exit; a missing identity's value; file
  operations failing; a `Store` whose store another made first; the sweep
  over deep and wide trees in pages of two. Its referee: each operation ends exactly once, with the outcome
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
- **The process tier** runs the machine's git, 2.41 or later, against
  its owned Forgejo: clone, fetch, create, check out, commit, push, in
  its story (`../testing/process-tier.md`).

## 10. Open questions

- **Containment.** In standalone, git and agents run as one user in
  process groups: an agent could read a running git's environment, or
  write a repository's configuration (a URL rewrite, a proxy) before the
  next invocation, or a store's, which sits beside the workspaces. A fetch
  runs in the store, whose configuration only `Store`'s clone wrote, so a
  checkout's own configuration reaches the forge only through a push, the
  delivery's second-parent fetch (4.7), and a check out's or a merge's
  lazy fetch, which takes `origin`'s URL from it (4.5). Default: accepted on one person's machine
  (credentials.md, section 8); the command-line settings of section 3
  and the URL on the command line cover what they name. Contained trees
  (skein's `runtime/io/processes.md`, 3), with the git directory and the
  stores unwritable by an agent's tree, close both. An agent's program surviving a forced stop
  also needs containment on one person's machine; git's configured
  helpers stay in its group (`git.md`, section 7; setup/README.md, section 9).
- **A stray file of another name** (4.7): `git add` still takes it, a new
  file being indistinguishable from a run's new work, unless the repository
  ignores it. Default: the repository's `.gitignore` is where such a name goes.
  A hidden log in conflict in a merge stays unmerged, so `write-tree` ends
  `Broken`; no story makes one.
- **A merge's commit past the checkout's `conflicts`** (4.7): its scan
  must read every unmerged path, so a listing past `conflicts` or
  `stdout_bytes` ends it `Broken`, which the domain takes as transient. A
  merge job's command stages each file it settles, so its commit lists
  only what it left. A resolution's agent stages nothing, so one that
  settles more files than `conflicts` cannot deliver. Default: as it
  stands; scanning the listing in pages, staging each page found clean
  before listing the next, would close it.
- **HEAD after saving unfinished work.** Default: a commit without a delivery snapshot
  does not move HEAD (section 4.7). Preparation follows each clean producer
  merge or merged head's commit with `CheckOut`, moving HEAD and clearing
  the merge's state before the run (`../domain/worker/checkouts.md`).
  A saved-work commit retains its domain-named parents; an unchecked save
  and checked delivery read and advance HEAD.
- **A forge whose certificate needs its own trust for git.** Default: the
  machine's roots, for git as for the forge's API, whose record names
  its TLS name alone (forge/engine.md, 1). Between hosts, the trust would
  come with the workspace's items.
- **Git LFS.** Default: not supported; no filter is configured, so a
  repository's large files stay pointer files in its working tree.
- **Another git's messages.** Default: classified against 2.47's, from
  2.41 on; a version that words a failure otherwise reads `Broken` until
  its transcripts are captured.
- **A dirty checkout's transcripts.** 4.5's clearing of staged files and a
  merge's state rests on one run by hand, and no transcript records it, since
  capturing again takes a pinned Forgejo and rewrites every case. Default: so,
  until an invocation changes or git moves (section 9).
- **A clone that outlives its workstream.** Section 3 turns off automatic
  garbage collection and maintenance, which costs little while a new
  workstream makes its workspace again. A clone kept past one
  (`../domain/worker/checkouts.status.md`) gathers old objects until a make, and a
  store gathers every fetch's for the worker's life (`../domain/worker/checkouts.md`, 1). Default: nothing collects them.
