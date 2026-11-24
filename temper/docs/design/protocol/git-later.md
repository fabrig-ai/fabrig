# The checkout’s git: open questions

## 1. Open questions

- **Containment.** No git metadata a run writes in its `.git` reaches an
  invocation (`git.md`, section 3, `git-operations.md`, section 1.13), but git and agents still run as one user:
  an agent's command could read a running git's environment, or write the
  worker's directory or a store directly, beside its workspace. Default:
  accepted until confinement's fence keeps a run's commands out of both
  (`../draft/confinement.md`, 5.2 and 6.3); the command-line settings of
  `git.md`, section 3 and the URL on the command line cover what they name. An
  agent's program surviving a forced stop also needs containment on one
  person's machine; git's configured helpers stay in its group (`git.md`,
  section 7; setup/README.md, section 9).
- **A run's git and the contents no step fetched.** A run's git borrows
  the worker's objects and the store's, and fetches nothing: its
  directory names no promisor remote and no credential reaches it
  (`git-operations.md`, 1.13; `git.md`, section 5). So it reads the files
  its checkout checked out and merged, and after a conflict those its
  sides' histories' comparison reads (`git-operations.md`, 1.6); a read of
  another commit's file no step fetched, such as an agent's `git show` of
  an old revision, fails `fatal: unable to read <object>`. Default: so,
  since a fetch of its own would put the identity's token in the run's
  reach; a worker's operation that brings named contents, if runs show the
  need.
- **A nested repository** in a work tree, a directory holding its own
  `.git`, is staged by `add --all` as git stages one, by the commit its
  HEAD names, which git reads without running anything it configures; a
  `clean` removes it (`git-operations.md`, 1.5). Default: so, the run's delivery being its own.
- **A stray file of another name** (`git-operations.md`, 1.7): `git add` still takes it, a new
  file being indistinguishable from a run's new work, unless the repository
  ignores it. Default: the repository's `.gitignore` is where such a name goes.
  A hidden log in conflict in a merge stays unmerged, so `write-tree` ends
  `Broken`; no story makes one.
- **A repack cut short** (`git-operations.md`, 1.14): one killed at its deadline leaves git's
  temporary files in `objects/pack`, which `count-objects` counts as
  garbage and no pack lookup reads; and a store whose repack outgrows the
  local deadline is cut short each time a prepare asks it, every 25
  fetches into it; and a fetch waiting on the store's lock behind a
  repack spends its own deadline there. Default: as it stands, since a
  blob-less store holds no file contents (50 MB on one deployment's forge,
  `git-operations.md`, 1.11); a repack's own deadline, or a store's upkeep left off once it
  timed out, if a deployment shows one.
- **HEAD after saving unfinished work.** Default: a commit without a delivery snapshot
  does not move HEAD (`git-operations.md`, section 1.7). Preparation follows each clean producer
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
- **A dirty checkout's transcripts.** `git-operations.md`, 1.5's clearing of staged files and a
  merge's state rests on one run by hand, and no transcript records it, since
  capturing again takes a pinned Forgejo and rewrites every case. Default: so,
  until an invocation changes or git moves (`git.md`, section 9).
- **A clone that outlives its workstream.** `git.md`, section 3 turns off automatic
  garbage collection and maintenance, which costs little while a new
  workstream makes its workspace again. A clone kept past one
  (`../domain/worker/checkouts.status.md`) gathers old objects until a make, and a
  store gathers every fetch's for the worker's life (`../domain/worker/checkouts.md`, 2). Default: nothing collects them.
