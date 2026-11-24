# The checkout's git operations

This document holds the checkout's git operations, split from [`git.md`](git.md).

## 1. Each operation

| Operation | Invocations | Ends in |
|---|---|---|
| `Make` | none: four file operations, and one for each store it names | `Succeeded` |
| `Clone` | 4, a file's write and the view | `Succeeded` |
| `Fetch` | 2: one in the store, one in the worker's directory | `Fetched { commit }` |
| `Counts` | 2: one in the store, one in the worker's directory (1.3) | `Fetched { commit }`, `Succeeded` |
| `Create` | 1 | `Succeeded`, `Exists` |
| `CheckOut` | 2 | `Succeeded` |
| `Merge` | 1 or 2 | `Merged`, `Conflicted { files }` |
| `Commit` | 1 to 4; a delivery adds 2 to 5, and a scan of files for a merge | `Committed { commit }`, `Unchanged`, `Conflicted { files }` |
| `Push` | 1, or 2 with an expected head | `Succeeded`, `Rejected` |
| `IsAncestor` | 1 | `IsAncestor { contains }` |
| `Store` | 1, a clone, and two or three file operations | `Succeeded` |
| `Point` | 1 | `Succeeded` |
| `Pack` | 1 to 4 | `Succeeded` |

Each may also end failed, with the fault of `git.md`, section 6. Below, `<url>` is
the operation's `remote`, `<dir>` its repository's name, and ids are as
`git.md`, section 3 writes them. The settings of `git.md`, section 3 precede each command,
and then the directories it names. Each operation that writes the
worker's directory then writes the run's view (1.13).

### 1.1 Make

`Make { workspace, stores }`: the workspace's directory and its git
directories' (`gitdirs/<token>`), empty, and its references in each store
it names gone.

1. `Rename` `workspaces/<token>`, then `gitdirs/<token>`, to `trash/<n>`, `<n>` the component's
   next serial number in decimal, counted from a number the component
   draws from its seed at each start. Every rename into `trash` refuses
   replacement: `Exists` takes the next serial number and tries the same
   source again, one submission at a time, within the operation's deadline
   where it has one. Thus a name an earlier start left in `trash` is not
   taken again, even when the seed repeats after a power loss. A directory
   that does not exist is nothing to move.
2. `MakeDirectory` `workspaces/<token>`, then `gitdirs/<token>`, mode 0o700.
3. For each store, in order, `Rename` `stores/<store>/refs/temper/<token>`
   to `trash/<n>`, the workspace's references there (1.3): a git killed
   while it fetched for this workspace may have left a lock among them,
   which would refuse the next fetch. One that does not exist is nothing
   to move.

`Succeeded` once all are done; a failed file operation is `Broken`. So
what the cache evicted, or a worker left behind before a restart, never
fails a prepare (`../domain/worker/README.md`, section 8), and Make takes four
file operations however large the old tree was.

**The sweep** empties `trash`: `git-sweep.md`, sections 2 to 4.

### 1.2 Clone

`Clone { at, remote, store }`: the repository's two git directories,
empty, the worker's borrowing the store's objects. It names no identity:
nothing reaches the forge.

1. In the store, `--git-dir=stores/<store>`:
   ```
   git rev-parse --show-object-format
   ```
   whose stdout is the store's format, `sha1` or `sha256`, and a newline;
   anything else is `Broken`. A repository borrows only from a store of its
   own format.
2. The worker's directory, in `gitdirs/<token>`, which `Make` left without it:
   ```
   git init --quiet --bare --object-format=<format> -- gitdirs/<token>/<dir>
   ```
3. The file `gitdirs/<token>/<dir>/objects/info/alternates` written whole,
   holding `../../../../stores/<store>/objects` and a newline: git reads a
   relative alternate from the object directory, so the file names no path
   of the machine.
4. In the worker's directory:
   ```
   git remote add origin -- <url>
   ```
   so `remote.origin.url` is the remote, as `git clone` wrote it, and no
   reference is fetched.
5. The run's directory, made as 1.13 makes it, with `<format>`: the work
   tree's directory and its `.git`, given the view.

- **Exit 0 from all four, and the files written:** `Succeeded`. Any other
  exit, or a failed file operation, is `Broken`. A repository the forge
  lacks is the store's `Missing { Repository }` (1.11), or, once its store
  is made, the first fetch's (1.3).
- **Commits and trees, no checkout.** No `--depth`: the store holds every
  commit and tree of what was fetched, and no file contents (1.11), which
  `CheckOut` and `Merge` fetch into the worker's directory as they need them, from
  `origin`, as the remote `remote add` named (1.5, 1.6). The working tree
  is made by `CheckOut`.
- **Bare, its work tree named.** The worker's directory holds no work
  tree of its own: git takes it from `--work-tree` alone (`git.md`, section 3).
- **Borrowed, never shared.** The worker's directory's configuration,
  references, index and own objects are its own; only the store's objects
  are read through it. The store is written by the worker's git alone.

### 1.3 Fetch

`Fetch { at, store, remote, want, into, identity }`: two invocations.

1. In the store, `--git-dir=stores/<store>`:
   ```
   git fetch --porcelain --verbose --filter=blob:none --no-tags --no-write-fetch-head --no-recurse-submodules -- <url> +<source>:refs/temper/<token>/<name>
   ```
2. In the worker's directory:
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
  `--filter=blob:none` keeps the store blob-less (1.11) whether or not it
  was made so. The second invocation transfers nothing. The repository
  reads them through its alternates file (1.2).
- **`refs/temper/fetched`** keeps what was fetched reachable, in the store
  under the workspace's token and in the repository, and is moved freely by
  the next fetch.
- **`refs/temper/landing`** is the landing pin: the landing branch's tip as
  prepare fetched it; or, for a writable item that names a pin, that commit
  (`../domain/worker/checkouts.md`, *A named pin*), fetched by its full id
  where the prepare has not; or, for a writable item that names neither,
  the head its push branch had (or its start's commit, where that branch
  does not exist yet), which the project's checks take their base from
  (`tools/pre-pr/design.md`, section 2). Only prepare's one pin writes it,
  a fetch into `Into::Landing` or a `Point` (1.12) at a commit the prepare
  fetched (`../domain/worker/checkouts.md`), forced, since a cached
  checkout's pin from an earlier hold may be anywhere; no other operation
  names it, so it stays where prepare put it for the hold.
- **These two, and main's counts (below), are the references the
  component writes** in the worker's directory, and their twins under the
  workspace's token in a store; the view copies them into the run's (1.13).
- **Missing:** `Repository` for a repository the forge does not have,
  `Branch` for a branch, or a default branch, it does not have; `Commit`
  for a commit it does not have (`git.md`, section 6). A full id is required: git
  looks an abbreviation up as a reference name.
- **A push that ended ambiguously** is verified by the domain with a
  fetch of its branch (`../domain/worker/checkouts.md`): no other
  invocation is needed for it.

**Main's counts.** `Counts { at, store, remote, identity }`: the forge's
`refs/temper/counts`, the one commit holding `counts.json` that the gate's
runner publishes after each landing (`../draft/test-budgets.md`, 3), in
the repository as `refs/temper/counts`, with that file's contents; or,
where the forge has none, no such reference in the repository. A prepare
asks it beside the pin (`../domain/worker/checkouts.md`, *Main's counts*).
Two invocations:

1. In the store, `--git-dir=stores/<store>`:
   ```
   git fetch --porcelain --verbose --no-filter --no-tags --no-write-fetch-head --no-recurse-submodules -- <url> +refs/temper/counts:refs/temper/<token>/counts
   ```
2. In the worker's directory, where the forge has it:
   ```
   git update-ref refs/temper/counts <new-id>
   ```
   and where it has none:
   ```
   git update-ref -d refs/temper/counts
   ```

`<token>` is as for Fetch above.

- **Present:** exit 0 and one porcelain line, read as Fetch's is, naming
  `refs/temper/<token>/counts`: the update, and then, at its exit 0,
  `Fetched { commit: <new-id> }`. A line naming another reference is
  `Broken`, and so is any other exit of the update.
- **Absent:** a fetch that fails with the stderr line `fatal: couldn't
  find remote ref refs/temper/counts`, exactly, is no fault (`git.md`,
  section 6): the deletion, and then, at its exit 0, whether or not the
  reference was there, `Succeeded`. So a repository that had none is as it
  was, and one a cached checkout kept from an earlier hold never outlives
  a forge that has none. Any other exit of the deletion is `Broken`.
- **Any other failure** of the fetch is classified as Fetch's is
  (`git.md`, section 6), and the repository's reference is left as it is.
- **The blob comes with it.** This is the one fetch that names
  `--no-filter` in place of `--filter=blob:none`, so the commit, its tree
  and `counts.json`'s blob come whole into the blob-less store (1.11),
  and a run reads the file offline with lazy fetches off, as the
  project's checks run (`GIT_NO_LAZY_FETCH`, `tools/pre-pr/pre_pr.py`),
  through the alternates its view names (1.13). `--no-filter`, named: a
  fetch that names no filter in a partial clone may take one git keeps for
  the remote it fetches from (`remote.<name>.partialclonefilter`), and
  this one must take none. The forge still sends only what the store
  lacks: one commit, one tree and one blob for each publish the store has
  not seen, and no other branch's file contents. Chosen over a filtered
  fetch followed by a fetch of the blob by its id, which takes a third
  invocation, a read of the tree to find the id, and the forge's leave to
  send an object by id.
- **Frozen for the hold:** only a prepare's `Counts` writes the
  reference, forced, since a cached checkout's from an earlier hold may be
  anywhere; no other operation names it, so it stays where prepare put it.

### 1.4 Create

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
| any other, or no status line | the fault of `git.md`, section 6 |

### 1.5 CheckOut

`CheckOut { at, commit, clean, identity, account }`: the working tree
exactly `commit`'s tree.

```
git -c remote.origin.promisor=true checkout --force --detach <commit>     with the identity's header (section 5)
git clean -ffdxq                                                          only with clean
```

- **The contents it lacks.** The store is blob-less (1.11), so the
  checkout finds the commit's trees there and not every file's contents:
  git fetches those it lacks from `origin` in one batch before it writes
  the tree, into the worker's directory's objects, where the next checkout of
  the repository and the run's git find them. The fetch is git's own (`git fetch origin
  --filter=blob:none --stdin`, its object ids on stdin), a child in the
  invocation's process group, run with the invocation's settings and
  environment, so the identity's header reaches it and nothing else does.
  It runs under the operation's one deadline, the remote one (`git.md`, section 7),
  and a failure of it fails the invocation, classified by `git.md`, section 6's
  table.
- **The first** detaches HEAD at the commit, sets the index to its tree,
  so that nothing stays staged and a file staged new goes with it,
  overwrites what is in the way, tracked or not, and clears a merge's
  state (`MERGE_HEAD` and the rest). **The second** removes every untracked and ignored file and
  directory, nested repositories included.
- **Exit 0 from both:** `Succeeded`; any other exit of the checkout is
  classified by `git.md`, section 6's table, and of `git clean`, `Broken`.
- **Told apart.** Both run under the operation's one remote deadline, and a
  failure names the invocation it ended in (`git.md`, section 6): `checkout`, its
  lazy fetch's time within it, or `clean`. So a deadline that passes
  during the checkout and one that passes during the clean read apart in
  the prepare's detail (`../domain/worker/checkouts.md`, 1.1).

### 1.6 Merge

`Merge { at, theirs, identity, account }`: merge fetched `theirs` into the
checked-out commit, without committing, leaving conflicts and the merge's
state on disk.

```
git -c remote.origin.promisor=true merge --no-commit --no-ff -- <theirs>     with the identity's header (section 5)
git -c remote.origin.promisor=true log --no-merges -M --diff-filter=R --format=%H HEAD...<theirs> --     only after exit 1, with the identity's header
git diff-files --name-only -z --diff-filter=U          only after exit 1
```

- **The contents it lacks:** those of the files the merge reads, theirs,
  the base's and, for renames, the candidates', fetched as `CheckOut`'s are
  (1.5), in batches, under the operation's remote deadline. A failure of
  that fetch ends the merge with an exit other than 0 and 1, classified by
  `git.md`, section 6's table.
- **The contents its readers compare.** A conflict is left for a reader:
  the merge job's command, which reads each side's history with rename
  detection to say what each side did to a file it leaves
  (`tooling.md`, 4.1), and a resolution's agent, told to read the same
  history (its `read`). Rename detection reads the contents of the files a
  commit removed and added, which the merge never read where a later
  commit changed them again: an older copy of a file the other side still
  holds. The run's git can fetch nothing: its directory names no promisor
  remote and its environment holds no credential (1.13,
  `../domain/worker/jobs.md`). So after exit 1 the merge's second
  invocation makes that comparison itself: each commit of either side's
  history (`HEAD...<theirs>`, the merge base's ancestors left out), but a
  merge, against its parent, with rename detection, as `git diff-tree
  -M <commit>^1 <commit>` makes it, git fetching what it lacks as 1.5's
  fetch does, one batch for each commit whose detection reads contents,
  into the worker's directory's objects, where the run's git finds them
  through the view's alternates; none enters the store. It runs under the
  operation's remote deadline, with the merge's settings and environment.
  Its stdout, the commits that renamed a file, is drained and never
  parsed, as the merge's is; the filter only keeps it short, since git
  detects renames in every commit before it filters. Exit 0 goes on to
  `diff-files`. Any other exit ends the merge, classified by `git.md`,
  section 6's table, as the merge's own fetch is, and `Broken` where no
  row names it: an object the forge can't give ends so, git's last lines
  naming the object and the invocation named `log` (`git.md`, section 6).
  A merge that ends `Merged` leaves no conflict, and runs none. (Real git
  2.47.3, 2026-11-24: in a blob-less store's checkout, a run's `diff-tree
  -M` of the base's commit that deleted a file another commit had changed
  ended `fatal: unable to read <object>`, exit 128, before this
  invocation, and listed the deletion after it.)
- **Exit 0:** `Merged`, a commit already contained in HEAD ("Already up
  to date") included: the next commit still records both parents
  (section 1.7). A producer's start asks first, and runs no merge of a
  base its branch contains (section 1.9).
- **Exit 1, and a list that is not empty:** `Conflicted { files }`, the
  unmerged paths, NUL-separated, relative to the repository's top, in
  git's order. One longer than its `path_bytes` among those kept is
  `Broken`.
- **More than the checkout's `conflicts`** is still `Conflicted`, with
  the first `conflicts` paths. A listing past `stdout_bytes` is read as
  far as it was kept, and a last path that was cut is dropped; `git.md`, section 8
  's rule for `stdout_bytes` keeps room for `conflicts` whole paths. The
  merge's state stays on disk whole, every file marked in the tree and
  unmerged in the index, whatever their count (2026-11-09). The merge command's stdout is human progress, never parsed: it is
  drained and bounded, and truncating it cannot change exit 0 or exit 1.
  The later NUL listing alone reads a kept prefix; every other parsed
  invocation's stdout past its bound is `Broken`, as `git.md`, section 6 says.
- **Exit 1 with an empty list,** as for a commit git cannot merge, or any
  other exit: `Broken`.
- **`diff-files`,** the plumbing command, lists the index's unmerged
  entries whatever the repository's diff settings say.
- **The committer** git requires even with `--no-commit` is the
  identity's author (`git.md`, section 5); Merge makes no commit.
- **Who commits it** is the domain's (`../domain/worker/checkouts.md`): a
  producer's clean merge and each head of a base with heads merged in at
  once, any other merge at its run's first push or save.

### 1.7 Commit

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

**A checked delivery or unchecked save** (`snapshot` is present) first reads the worker's directory, with no other
writer, before the staging and commit sequence below. Its HEAD and
MERGE_HEAD are the worker's, which only the component's operations move,
so a commit or a merge a run made in its own directory reaches a delivery
as its files alone (1.13):

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
for a commit the remote no longer reaches. An empty listing ends the
operation `ParentUnreachable { head, merging, parent }` before anything is
staged: HEAD and MERGE_HEAD as read, and the second parent no remote
reference reaches. The domain refuses it with `delivery second parent is
not reachable from the branch's remote`, or catches up with a landed
dependency (`../domain/worker/checkouts.md`, *A landed dependency's catch-up*).
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
- **Past the checkout's `conflicts`,** no count refuses a commit. A
  listing cut by `conflicts` or `stdout_bytes`, its first `conflicts`
  paths kept as 1.6 keeps a merge's, is a page, and the rest come in pages
  through an index of the component's own: it copies the index whole to
  `gitdirs/<token>/<dir>/temper-scan`, and while a listing was cut, scans
  its page, stages it in the copy (`git update-index --add --remove -z
  --stdin`, the page's paths on stdin) and lists again, these two with
  `GIT_INDEX_FILE` naming the copy by its absolute path. A listing that
  ends whole is the last page, and the sequence goes on in the index
  itself. A page with markers ends `Conflicted { files }`, its marked
  paths, nothing staged in the index; every way out removes the copy. So
  a merge job's or a resolution's commit scans every file its merge left,
  whatever the run staged in its own index.
- **Why plumbing.** The domain always records both parents of a merge,
  also when git found `theirs` already merged and wrote no `MERGE_HEAD`,
  and git refuses to write that pseudo-reference through `update-ref`.
  `commit-tree` records exactly the parents asked, signs nothing and runs
  no hook.
- **Without a delivery snapshot, HEAD is not moved,** nor the merge's
  state cleared, by a commit: HEAD and the merge's state are `CheckOut`'s.
  These commits compare and record against the parents the domain names.
  Delivery reads and advances HEAD as above.
- **The author and committer** are the identity's author (`git.md`, section 5); the
  date is the clock's, in UTC.

### 1.8 Push

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
| any other, `+` included, or no status line | the fault of `git.md`, section 6 |

Forgejo refuses a push to a protected branch with that `[remote
rejected]` line, its reason ending in `declined`, as
`temper-git-transcripts` records (fact
`protected-push-is-remote-rejected-declined`).

A push that fails in a way that may have landed (`Unreachable`, `Broken`,
`TimedOut`, `Cancelled`) is verified by the domain (section 1.3).

### 1.9 IsAncestor

`IsAncestor { at, ancestor, commit }` asks whether fetched `ancestor` is
contained in fetched `commit`, including equality. It is local, changes no
file or ref, and needs no forge credential:

```
git merge-base --is-ancestor <ancestor> <commit>
```

Exit 0 ends `IsAncestor { contains: true }`, exit 1 ends it with false,
and other exits fail as `Broken`. The same command inside Push retains
section 1.8's separate contract: exit 1 rejects that push. The checkout
uses this operation to compare an existing producer head with the landing
tip frozen by its prepare, and to ask whether a start's checkout already
contains a head or a producer's base it would merge
(`../domain/worker/checkouts.md`); the comparison alone is never a push
receipt.

`Brings { at, head, base, upstream }` asks whether fetched `head` holds a
commit that fetched `base` lacks and fetched `upstream` holds: a prepare
job's question of a member under `squash`, which brings its upstream's
commits only by a merge (`../domain/worker/jobs.md`; `../domain/landing-prepare.md`,
1.2). It is local, changes no file or ref, and needs no forge credential:

```
git merge-base --all <head> <upstream>                 the merge bases, one id a line
git rev-list --max-count=1 <m>... ^<base> --           each merge base m a positive
```

A commit both hold and the base lacks lies below a merge base of the two
that the base lacks, so the listing names one exactly when there is
such a commit: `Brings { brings: true }`. An empty listing, or
`merge-base` ending 1 with nothing printed (no commit in common), ends
it with false. Any other exit, a malformed id, or output past
`stdout_bytes` is `Broken`.

`DiffPaths { at, before, after, markdown }` streams changed paths. With
`markdown` true it retains the first differing path, including Markdown,
for a prepare's whole-tree comparison (`../domain/landing-prepare.md`, 1.2).
With `markdown` false it admits a design delivery's changed paths:

```
git diff-tree --no-commit-id --name-only --no-renames -r -z <before> <after> --
```

The complete stdout stream is read as NUL-terminated paths, including
additions and deletions. Each name stays within `path_bytes`; no count of
changed files is imposed by the merge-conflict limit. The component keeps
only the current path and the first admitted path, and returns `Paths`
with that name or no names. A design comparison excludes names ending
in `.md`; a prepare comparison includes them. An incomplete,
empty or oversized path fails as `Broken`. A later code path cannot hide
behind a long list of Markdown files or the stdout retention cap.

### 1.10 Rewrite

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

### 1.11 Store

`Store { store, aside, remote, identity, account, git }`, `git` the
program (`git.md`, section 3): `stores/<store>` a bare clone of the remote's default
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
   serial number, the one `trash` names take, and:
   ```
   git clone --bare --filter=blob:none --single-branch --no-tags --quiet -- <url> making/<m>
   ```
   which git accepts into the empty directory.
4. `Rename` `making/<m>` to `stores/<store>`, never replacing: where
   `stores/<store>` is there already, `making/<m>` is renamed to
   `trash/<n>` instead, taking the next unused serial name (1.1).

- **Exit 0 and the file operations done:** `Succeeded`, whichever rename
  took, or at step 2 with nothing made. A failed clone is classified by
  `git.md`, section 6's table, as a fetch is (a repository the forge lacks is
  `Missing { Repository }`, a token it refused `Refused`); any other
  failed file operation is `Broken`; and what step 3 made is renamed to
  `trash` if it can be. A worker stopped mid-clone leaves it in `making`,
  which its next start sets aside (`git.md`, section 2).
- **A clone, for the object format.** `git init` makes a SHA-1 repository,
  and git 2.47's fetch from a SHA-256 remote into it fails (`fatal:
  mismatched algorithms: client sha1; server sha256`); a clone takes the
  remote's format, an empty remote's too, which `Clone` then reads (1.2).
  `--single-branch` brings the forge's default branch alone, the history a
  first prepare most likely fetches next, so that fetch brings nothing
  more; nothing else is fetched but what an assignment names and main's
  counts (1.3).
- **Blob-less.** `--filter=blob:none` brings every commit and tree of the
  default branch and no file contents: on one deployment's forge, 2.3 s
  and 50 MB where the whole clone took 17.4 s and 198 MB (2026-11-12). git
  records the store as a partial clone of `origin`; nothing in the store
  reads a file's contents, and a checkout fetches those it needs (1.5).
  The one file a store holds whole is main's counts' `counts.json`, which
  its fetch brings unfiltered (1.3).
  A store made whole before stores were blob-less stays so, and serves as
  well.
- **Its configuration is the clone's:** `remote.origin` names the remote
  and its default branch, and no fetch reads it: every fetch names its URL
  on the command line (`git.md`, section 3). Its `refs/heads/<default>` stays where
  the clone put it. Nothing of a workspace's reaches it but objects the
  forge sent and references under the workspace's token.
- **Repacked, never pruned:** `gc.auto=0` and `maintenance.auto=false`
  hold for it as for every repository, so nothing prunes an object a
  checkout may borrow; `Pack` repacks it, keeping every object (1.14).

### 1.12 Point

`Point { at, into, commit }`: the repository's `into` reference at
`commit`, which the repository already reaches through its store, with
no fetch. Local; it names no identity. In the worker's directory:

```
git update-ref refs/temper/<name> <commit>
```

`<name>` as in 4.3. Exit 0: `Succeeded`; any other is `Broken`.

### 1.13 The run's git directory

The `.git` in the work tree is the run's (`../domain/worker/checkouts.md`,
*Two git directories*): its index, references and configuration are the
run's to write, and no invocation reads it (`git.md`, section 3). The component
writes into it, never reading what is there, and makes it:

- **The view,** after each operation that writes the worker's directory
  (`Clone`, `Fetch`, `Counts`, `Point`, `CheckOut`, `Merge`, `Commit`, `Reset`),
  before it ends, unless it ends failed: the
  worker's `HEAD`, `index`, `MERGE_HEAD`, `MERGE_MSG`, `MERGE_MODE`,
  `refs/temper/fetched`, `refs/temper/landing` and `refs/temper/counts`,
  each copied whole where
  the worker's directory has it and removed where it has none; and
  `objects/info/alternates`, holding
  `../../../../../gitdirs/<token>/<repository>/objects` and a newline, so
  the run's git reads the worker's objects and, through their alternates,
  the store's. So a run's checkout is clean at its start, shows a merge
  the worker left in progress, and has the pin its checks take their base
  from (`tools/pre-pr/design.md`, section 2) and main's counts where the
  forge has them (1.3).
- **Each file whole or not at all:** written to `temper-view.tmp` in its
  directory, then renamed over its name, so a link or a file the run left
  there is replaced, never followed; a directory missing on the way is
  made, 0o700. The worker's files are read without following a link, in
  chunks of `read_bytes`.
- **Made again where it fails.** Where a step in the run's directory
  fails (it is a file naming another directory, a link, or holds a file
  where a directory goes), or there is none yet (1.2), the component
  renames `.git` to `trash/<n>` (none is nothing to move), runs `git init
  --quiet --object-format=<format> -- workspaces/<token>/<repository>`,
  the format the worker's directory's (1.2's first command, on it), and
  writes the view into what it made. What the run kept there, its own
  branches and its checks' caches, goes with it. A failure then, or one
  reading the worker's directory, is `Broken`.
- **The rest it touches there:** the goal's reds files, beneath
  `workspaces/<token>/<repository>/.git/temper`, a path the component
  writes and never asks of git (`../domain/forge/reds.md`, 5.2); and the
  check's report, `temper/check-results` beside them, the one file of the
  run's it reads, as bytes, without following a link, never through git
  (`../domain/forge/reds.md`, 5.5).

### 1.14 Pack

`Pack { at, store }`: the store `stores/<store>`, where `store` names one,
then the worker's git directory of the repository (`git.md`, section 3), each
repacked once it holds more than 50 packs, the bound git's own
`gc.autoPackLimit` keeps. Local; it names no identity.
git keeps each fetch from a promisor remote as a pack of its own, a store's
fetch (1.3) and a check out's or merge's fetch of contents (1.5, 1.6)
alike, and `git.md`, section 3's settings leave no git to repack them, so a prepare
asks a `Pack` for a repository once enough of those have gone into its
store or into it (`../domain/worker/checkouts.md`). For each place, with
its git directory named as in `git.md`, section 3:

1. The count:
   ```
   git count-objects -v
   ```
   whose stdout's one `packs: <n>` line, in decimal, counts its packs; a
   stdout without exactly one is `Broken`.
2. Only where `<n>` is above 50:
   ```
   git -c remote.origin.promisor=true repack -A -d -l -q --no-write-bitmap-index
   ```

- **Exit 0 from each it ran:** `Succeeded`; any other exit is `Broken`.
  git puts the new pack in place before it deletes the old ones, so a
  repack killed or failed at any point leaves every object where some pack
  holds it.
- **Every object kept.** With the promisor setting, git repacks the packs
  fetched from the forge apart, into one pack that keeps every object they
  held, reachable or not, so whatever a checkout borrows stays; `-A` turns
  any other unreachable object loose rather than dropping it, and nothing
  prunes. `-l` keeps a repository's pack to its own objects, not its
  store's. On git 2.47.3 (2026-11-19), a store's 9 packs became 1 and a
  checkout's 11 lazily fetched packs 2, with no object lost and nothing
  fetched.
- **Not geometric.** git 2.47's `repack --geometric` refuses a repository
  with a promisor remote, which every blob-less store and checkout is
  (`fatal: cannot use internal rev list with --stdin-packs`).
- **A store's lock,** the component's, so that a repack never runs beside
  another repack or a fetch in the same store (the user, 2026-11-19).
  Fetch's and Counts' store fetches (1.3) share it with other fetches; a `Pack`'s count
  and repack in the store hold it alone. An operation that cannot take it
  waits, with no invocation, behind those that came before it, under its
  own deadline (`git.md`, section 7), and takes it once those ahead of it have it or
  are gone. A repository needs none: it runs one operation at a time.
  Other readers of the store, a check out's and a merge's, take no lock.
- **Under the local deadline** (`git.md`, section 7): a repack it kills leaves its
  temporary files in `objects/pack` (`git-later.md`, section 1).

## 2. Open questions

None.
