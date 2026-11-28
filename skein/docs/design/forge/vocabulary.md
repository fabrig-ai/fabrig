# The forge kit: the vocabulary

## 1. The vocabulary

### 1.1 What it is

`skein-forge`'s `api` and `locate` modules are the forge's neutral
vocabulary. It describes bounded requests and answers between the
client and a dialect, and retains no state. It names no task, no grant
and no policy. Every type is a plain value.

- **Calls:** `Op`, which is a `Read` or a `Write`.
- **Their result:** `Answer`, `Error`, and the enums they use (`Kind`,
  `State`, `Ci`, `Check`, `Verdict`, `Permission`, `BranchCreation`,
  `Provenance`, `Bypass`).
- **What an answer says of the forge** (1.3): `Rate`, the rate's window,
  and `Validator`, a read's validator.
- **A hint:** `Hint`, with its `Change`.
- **The items** that answers carry (1.6).
- **Addressing:** `Repository { forge, repository }`, `forge` being the
  owner's number for a forge (16 bits) and `repository` the forge's own
  id (32 bits). A commit is 32 bytes (a SHA-1 id zero-padded). Times
  are `skein_lib::Time` and `Duration`.

### 1.2 The calls

26 reads and 18 writes, in families. Each call has one answer.

| Family | Calls | Answer |
|---|---|---|
| Find a repository | `Locate` | `Located` |
| The change feed | `Items` | `Items` |
| One item | `Item`, `Pull`, `PullFor` | `Item`; `Pull`; `Pull` |
| Review | `Reviews`, `Remarks` | `Reviews`; `Remarks` |
| CI | `Statuses`, `Checks`, `Job`, `JobState` (job-state.md) | `Statuses`; `Checks`; `Job`; `JobState` |
| Actions | `Runs`, `Run`, `Artifact`; `Dispatch`, `Rerun` | `Runs`; `Run`; `File`; `Done`; `Done` |
| Branches | `Branch`, `Branches`, `Commits` | `Commit`; `Branches`; `Commits` |
| Contents | `PullFiles`, `Compare`, `Range`, `File`, `Tree` | `PullFiles`; `Compare`; `Range`; `File`; `Tree` |
| The repository | `Protection`, `Settings`, `Collaborators`, `Permission` | the same names |
| Creations | `CreateIssue` (keyed), `OpenPull` | `Created` |
| Labels | `Labels`, `CreateLabel` | `Done`; `Done` or `Exists` |
| Comments, keyed | `Post`, `Review` | `Commented`; `Reviewed` |
| Edits | `Edit`, `SetReviewers`, `Close`, `Reopen` | `Done` |
| Merging | `Merge`, `Update`, `UpdateRef` | `Merged` or `Queued`; `Done`; `Done` |
| Marks and branches | `Status`, `CreateBranch`, `DeleteBranch` | `Done`; `Branch`; `Done` |

- **The change feed** is one listing of items since a time, least
  recently updated first, in pages from 1. It is the only listing of
  items (`forgejo-operations.md`, 1).
- **A keyed write** takes the key the owner made. The dialect writes it
  into the marker (`README.md`, section 5). A pull request's key is its branches, so
  `OpenPull` takes none.
- **`PullFor { head, base }`** asks for the open pull request of a head
  and a base, else the oldest. It reads no CI.
- **`Pull { number, ci }`** asks for a pull request and, when `ci`, its
  head's CI, folded as `Checks` folds its rows. Without `ci` the forge is
  asked for no status, and the answer's `ci` is `Ci::None`, as `PullFor`'s
  is: a reader that uses CI asks `ci`, or reads `Checks`.
- **`Reviews`, `Remarks` and `PullFiles` take `skip`,** the rows of their
  page already read: a page the answer's bytes cut (1.3) is read on at
  the same page past them, so a cut loses no row and repeats none.
- **`PullFiles { number, head, page, skip, sides }`** reads a page of a
  pull request's changed files at `head`. With `sides`, each `File`
  carries its contents at the merge base and at the head, `None` for a
  side the file lacks; without, its path alone, both `None`, and the
  dialect reads neither a merge base nor a side: the files' listing is
  the read. A reader that needs paths alone, as goal overlap does
  (temper's `domain/forge/topics.md`, 1.3), asks without `sides`.
- **`Item { number, after, since }`** asks for an item and its comments
  with ids above `after`. `since` is a time at or before the update of
  every comment above `after`, which the dialect sends as its forge's own
  filter so that the forge sends fewer; 0 sends none. `after` stays the
  answer's check (1.7): a comment at or below it that the forge still
  sends is skipped. An owner that knows no such time sends 0.
- **`Range { before, after }`** counts the commits after `before` up to
  `after`, those `after` holds and `before` lacks, and lists none: no
  commit, no file, no side. It is the read of containment and of
  emptiness: `before` contains `after` exactly when it counts 0, so an
  owner asking whether a head holds a base's tip asks
  `Range { before: head, after: tip }`, and one asking whether a range
  is empty asks `Range` over it. Its answer holds no list, so no range is
  too long for it.
- **`Job { attempt, max_bytes, part }`** asks for one part of one job
  attempt's log: `End`, the end of its failed step's log, or of its
  whole log when no step failed; or `Start`, the first bytes of its
  whole log, where a CI's opening lines are, such as the selection line
  a goal's report quotes (temper's `domain/forge/branches-closer.md`, 1). `File {
  head, path, max_bytes }` asks for the start of one file at a commit.
  Both are cut to `max_bytes`, or to what the answer has left (`forgejo.md`, 1.3), and
  say when they cut: a `Start` that cut kept the log's first bytes and
  dropped the rest.
- **`Merge { number, head }`** is conditional on the head. A merge's
  base is the client's precondition, checked by reading the pull
  request afresh (client-effects.md, section 1). Where a merge queue
  guards the base, it answers `Queued`: the pull request was put in the
  queue at that head, and the landing comes later, as a hint
  (github-operations.md, section 2.1).
- **`Protection`** needs an admin to read an absent rule. A writer who
  is not an admin gets `Forbidden` (`forgejo-operations.md`, 1).
- **`Merge { number, head, how }`** merges in the repository's own style
  (`Style`), or `FastForwardOnly`: the base moved to exactly the head,
  refused (`Conflict`) when the head does not contain the base's tip;
  or `Recorded { commit }`, last: nothing moves, and the pull request is
  recorded merged at `commit`, which its base must already contain,
  for an owner that landed the pull request's change by a commit of its
  own (temper's squash, temper's `domain/landing-turn.md`, section 1). It
  answers `Merged` at `commit`; a commit the base does not contain, or
  a repository that allows no such record, is `Refused`, and a dialect
  whose forge records none answers `Refused` too.
  **`Update { number, style }`** brings a pull request's branch up to
  its base by a merge of the base into it (`Merge`), or by a rebase of
  its commits onto the base (`Rebase`), which rewrites the branch.
- **`UpdateRef { branch, to }`** moves a branch to a commit as a
  fast-forward only, refused (`Stale`) when the commit does not
  descend from the branch's tip, for an owner that
  fast-forwards a branch where the forge merges no pull request so. A
  dialect whose forge has no such route answers `Refused`: Forgejo's
  owner fast-forwards by a merge instead.
- **`Status { commit, context, check, description, link }`** writes a
  context's state with its words, at most `DESCRIPTION_BYTES` (140), and
  its link, an absolute URL, either empty for none.
- **Actions** (connection.md has no part in them beyond any
  call's): `Dispatch { workflow, branch, inputs }` runs a workflow,
  named by its file, at a branch's head, with at most `INPUTS` (10)
  inputs, each a name and a value; the forge keeps no key for it, and
  answers no run, so its owner finds the run with `Runs { workflow,
  commit, page }`, the runs of one workflow at one commit, newest
  first. `Run { run }` reads one run: its number, workflow, commit,
  status, conclusion and jobs, each job with its id, name, attempt,
  status, conclusion and failed step. `Rerun { attempt }` runs one job
  again, named by its `JobAttempt`, with the jobs that need it, once its
  run has completed; its find reads the run for the job's next
  attempt.
- **`Artifact { run, name, max_bytes }`** reads one file of a run's
  artifact by the artifact's name: the archive's single file, stored or
  deflated, cut to `max_bytes`, answered as a `File` at the run's
  commit with the file's path. An archive of several files, or one
  compressed otherwise, is `Refused`.
- **`Commits { before, after, page }`** lists the commits after
  `before` up to `after`, oldest first, each its id, its parents and its
  message, in pages from 1.
- **`Compare { before, after }`** compares two commits by names and
  counts alone: the paths changed from their merge base to `after`, and
  the commits each has that the other lacks, so whether `after` contains
  `before`. It carries no file's contents and lists no commit. A reader
  that asks only whether one commit contains another reads the ancestry
  range instead, `Commits { before: container, after: contained, page: 1
  }`, complete and empty; one that needs a file's contents reads it by
  path at each commit (`File`).
- **`Tree { object, page }`** reads one tree, not recursively: a
  commit's root tree, or a tree by its id, as its parent's entry names
  it. Each entry is a name, a mode, a kind and an object id, in name
  order, in pages from 1.

### 1.3 The answers

31 answers: `Located`, `Items`, `Item`, `Pull`, `Reviews`, `Statuses`,
`Remarks`, `Commit`, `Branches`, `Commits`, `PullFiles`, `Compare`, `Range`,
`Checks`, `Job`, `JobState`, `File`, `Tree`, `Runs`, `Run`, `Protection`, `Settings`,
`Collaborators`, `Permission`, `Created`, `Commented`, `Reviewed`,
`Merged`, `Queued`, `Branch`, `Done`.

- **`Range { before, after, commits }`** carries its call's `before` and
  `after` and the count, `commits`, a `u64`.

- **A paged answer says whether more follows** (`more`): `Items`,
  `Item` (its comments), `Reviews`, `Statuses`, `Remarks`, `PullFiles`
  and `Collaborators`. `Items`, `Reviews`, `Remarks` and `PullFiles` say
  too whether the answer's bytes cut their page (`cut`), with `more`
  set: a cut page is never the last. `Branches` and `Checks` carry none. The answer to
  `Items` also carries `now`, the forge's clock when it answered.
- **`Created`, `Commented` and `Reviewed`** carry the new object's
  number or id. `Merged` carries the merge's commit. `Branch` carries
  `BranchCreation`: `Created`, or `Exists`.
- **`Done`** answers `Update`, `Edit`, `SetReviewers`, `Close`,
  `Reopen`, `Status`, `DeleteBranch`, `Dispatch` and `Rerun`.
- **`Runs`, `Tree` and `Commits` are paged,** and say whether more
  follows.
- **`Pull` answers both `Pull` and `PullFor`;** the `Commit` answer is a
  branch's tip.
- **`File` at a directory answers `Directory`:** its call's head and
  path, its immediate entries in the forge's order, each a name and an
  `EntryKind`, and `truncated` where the answer's bytes left some out.
  The kinds are `File`, a regular file; `Link`, a symbolic link, whose
  target the listing doesn't give; `Directory`; and `Submodule`. Both
  dialects' listings name a link apart (`symlink`), so neither reads it
  as a file, and a reader that wants a link's bytes or a file's asks for
  them by kind. An entry of any other kind is left out.
- **`Compare`** carries `ahead`, the commits `after` has that `before`
  lacks; `behind`, those `before` has that `after` lacks, 0 when `after`
  contains `before`; `files`, the changed paths in the forge's order, as
  many as the answer holds, at most `rows` within `answer_bytes`; and
  `changed`, how many files changed in all, so that `changed` past
  `files`' length says how many it left out. A comparison is refused for
  neither the paths it leaves out nor its commits, so two branches
  however far apart compare, unless the forge cuts its own listing
  (github-operations.md, 1.1).
- **`Queued` is final for its call.** It answers `Merge` alone, and
  says the merge is the queue's now: neither a merge nor a refusal. The
  client keeps the write open until a read finds the pull request merged
  or out of the queue (client-effects.md, section 1). Forgejo's dialect
  never answers it.
- **What an answer says of the forge,** beside its cost, travels with
  it in the component's envelope (connection-calls.md, section 1), and
  each dialect fills what its forge sends:
  - **`Rate { limit, remaining, reset_at }`,** the rate's window: the
    requests the window allows, those left, and when it resets, in
    seconds since the epoch. The client's budget takes it in place of
    its configured window (client.md, section 6). Forgejo's
    answers carry none.
  - **`Validator`,** a read's validator (an `ETag`, or a
    `Last-Modified`), at most 128 printable bytes (`VALIDATOR_BYTES`).
    A read sent with one that the forge answers `304` is answered from
    the answer the component kept, counted in `free`
    (connection-calls.md, 1.6). Both dialects send them on every read;
    Forgejo sends an `ETag` on a file's raw read and none on a branch's
    JSON, and a read with none kept is sent as before.

### 1.4 The errors

21 errors. A call's `Error` is the first failure of its requests, and
`forgejo.md`, 1.4 says which failure gives which.

| Error | Means |
|---|---|
| `Unavailable` | the forge could not be reached, or the request provably was not read |
| `Timeout` | the request may have reached the forge: a write's outcome is unknown |
| `RateLimited { after }` | the forge refuses calls for `after` |
| `Forbidden` | the writer lacks the permission (a bad or lacking credential included) |
| `Missing` | the object is absent |
| `MissingJob` | the job attempt is absent, or is no longer the attempt asked for |
| `TooLarge(Excess)` | a request or an answer passes a bound, which it names with the size met |
| `Empty` | the write has no content |
| `Full` | the forge has no room for the write (a quota) |
| `Exists` | the object already exists |
| `NothingToMerge` | the pull request would add nothing to its base |
| `Closed` | the target is closed or merged |
| `Stale` | the head the call required has moved |
| `BaseMoved { base }` | an update's pull request is at the head it required, on another base commit than the one it was decided on: `base` the one the forge reported, none where it reported none |
| `Conflict` | the change conflicts with its base |
| `Protected` | a protection rule refused the write |
| `Refused` | the repository's settings, or the forge's policy, refuse the write |
| `InvalidAnswer` | a decoded answer does not match its call |
| `Busy` | the client's own admission refused the call; no API call was made |
| `NotConnected` | the forge is not connected, or stopped; no API call was made |
| `NotInstalled` | no installation of the App reaches the repository's owner; no API call was made |

- **`Busy` is never a dialect's.** It is the client's own refusal
  (client.md, section 2.3). The component never produces it
  (connection-machines.md, section 1).
- **Nor is `BaseMoved`:** the client's check of an update's condition
  gives it (client-effects.md, 1.4), so its owner can tell a base gone on from a
  head moved.
- **`NotConnected` and `NotInstalled` are the component's,** never a
  dialect's, and cost nothing (connection-machines.md, section 1).
  `NotInstalled` is GitHub's alone: the component chooses an
  installation by the repository's owner (`README.md`, section 7).
- **A pause is `RateLimited { after }`:** `Retry-After` when the forge
  gives it; else, when the window has none left, its reset less the
  forge's clock; else, for GitHub's secondary limits, 60 seconds
  (github.md, section 7).
- **`Timeout` is the error that leaves a write's outcome unknown.** The
  client finds the outcome by the write's key, or by reading
  (client-effects.md, section 1).
- **`TooLarge` names its bound and its size,** as `Excess { bound,
  limit, size, exact }`, a plain value:
  - `bound`, a `Bound`: `OpBytes`, `AnswerBytes` or `Rows`, the client's
    own configured bounds (client.md, 2.2 and 2.3), passed by a
    call's bytes, an answer's bytes by the measure (1.7), or an answer's
    longest list; `Forge`, a bound of the dialect's, the component's or
    the forge's own: a body past `body_bytes_in`, a string past
    `string_bytes`, a listing the forge cuts, a count past the
    vocabulary's width; `Unnamed`, where the check kept no figure.
  - `limit`, the bound's figure, and `size`, what was met, both in the
    bound's unit, bytes or rows. Both are 0 for `Unnamed`, and for a
    `Forge` bound whose figure the check does not hold.
  - `exact`: `size` is the whole's, measured; otherwise it is a lower
    bound, what had been read when the bound stopped the read.

  So `size` less `limit` says by how much, at least. A call's
  `max_bytes` past `answer_bytes` is `AnswerBytes`, its `size` the
  `max_bytes` asked. A dialect names the client's bound where its own
  check is the client's figure (the answer's bytes left for a file's
  side, a page's rows), and `Forge` or `Unnamed` otherwise.

### 1.5 The hint

`Hint { repository, change, key }` says that something may have changed.
It is not a fact.

- **`Change`:**
  - `Item(number)`: an issue or a pull request;
  - `Branch(branch)`: a branch was made or deleted;
  - `BranchMoved { branch, head, actor }`: a branch moved to a commit,
    pushed by the user `actor`;
  - `Commit(commit)`: CI moved at a commit.
- **`key`** is the key of the body that caused the hint, when the
  delivery carried one. The owner drops its own echoes by `key` and by
  `actor` (the pusher); the client reads on every hint it holds a
  resource for, and uses `actor` for drift alone (client.md,
  section 4).
- **A hint decides nothing.** It brings a read sooner, never a decision
  (client.md, section 4).
- **No request causes a hint.** It comes from a verified delivery
  (`README.md`, section 6).

### 1.6 The items

By name; the code holds the fields.

- **`Summary`:** one row of the change feed: number, kind, state,
  author, labels (for display), key, title, body and updated time.
- **`Comment`:** a comment, with its id, author, created time, its key,
  its `Provenance`, its `revision` and its body.
- **`Pull`:** a pull request's head and base branches and commits, its
  merge commit, whether it is mergeable, its reviewers, its `Ci`, and
  `queued`, its position in a merge queue when it is in one, and
  `dequeued`, the forge's words for why a queue last ejected it, at most
  `DEQUEUED_BYTES` (256) bytes, and `size`, its `Size` at the head the
  same answer gives, none where the forge's answer gives none.
- **`Size`:** a pull request's size as the forge counts it at its head:
  `files`, the files it changes, and `additions` and `deletions`, the
  lines it adds and removes, each a `u64`. They are the forge's own
  counts, read from its answer about the pull request, never counted
  from the files or a listing. An answer that lacks one of the three,
  or gives one that is no whole number within 64 bits, gives no `Size`,
  and the pull request reads as it does without one: a size never fails
  a read. They are kept as given: no fact on record caps them, as the
  files' listings are capped (`github-operations.md`, 1.1), so a reader that keeps
  them narrower says where it cut one.
- **`Review`:** a review, with its `Verdict`, its commit, its key, its
  `Provenance`, its `revision`, its time and whether it is official.
- **`Remark`:** a comment attached to a review at a path and a line.
- **`Status`:** a CI context: its time, context, `Check`, description,
  link and, for an Actions job, its `JobAttempt`.
- **`JobAttempt`:** a job's head, run, job and attempt.
- **`Run`:** an Actions run: its id, number, workflow file, commit,
  its event (`dispatch` or another), the user who triggered it, by the
  forge's id, `RunState` (`Waiting`, `Running`, `Completed`), `Conclusion`
  (`Success`, `Failure`, `Cancelled`, `Skipped`, none while it runs),
  its link, when it was made, and its jobs, each a `RunJob`: id, name,
  attempt, state, conclusion and its failed `Step`, none when no step
  failed.
- **`Entry`:** one entry of a tree: its name, its mode, its `EntryKind`
  (`Blob`, `Tree`, `Commit` for a submodule) and its object id.
- **`Step`:** a job's failed step, by number and name.
- **`File`:** a changed file: its path and its bytes before and after,
  either absent when the file was added or deleted.
- **`Protection`:** a rule's branch, required contexts, required
  approvals and whether it dismisses stale approvals; whether a merge
  queue guards it, and whether it requires signatures, linear history
  or code owners' reviews; and `Bypass`, whether the credential may pass
  it: `Never`, `PullsOnly` or `Always`; `pulls_required`,
  whether a rule requires a pull request, and `pulls_bypass`, the
  credential's bypass of the rules that do (github-operations.md, 1.5). Forgejo's dialect reads signatures
  from `require_signed_commits`, and answers no queue, no linear history,
  no code owners and `Never`.
- **`Settings`:** a repository's default branch and the merge styles it
  allows.
- **`Collaborator`:** a user's id and `Permission`.
- **`UserPermission`,** `Permission`'s answer: the user's id, the login
  the forge names them by and their `Permission`. The login is empty
  where the permission is read without one, as a GitHub bot's from its
  installation.

`Provenance` is `Original`, `Revised` or `Unknown`. A dialect that
cannot tell whether a historical creation was edited answers `Unknown`,
never `Original` from a missing field. A `revision` changes whenever the
body the forge keeps changes (`forgejo.md`, 1.3).

### 1.7 The rules on them

- **One answer per call,** and its cost: the number of requests the call
  wrote, the fills of the component's tables included, and 0 when it
  wrote none. The component reports the cost with the answer
  (connection-calls.md, section 1), and the budget charges it
  (client.md, section 6).
- **Repositories by the forge's id.** A call names `Repository`; the
  dialect addresses the forge by the repository's path, which the
  component keeps for the id (connection-calls.md, section 1).
- **Locate by owner and name.** `Locate { owner, name }` names a
  repository's account and name, and is addressed with the forge's number
  and repository 0. It answers `Located` with the forge's id, which is
  never 0. The call's bytes are the lengths of its `owner` and `name`.
- **Every dialect answers every call.** A call that a dialect cannot
  answer as Forgejo does is named when that dialect is designed (`README.md`, section 8
  ). GitHub's one is `Status`, which answers `Forbidden` where the
  App lacks `statuses` write (github-operations.md, section 1.2). The
  Actions calls, `Tree`, `Artifact`, `Commits`, a fast-forward-only
  merge and an update by rebase take GitHub's routes of
  github-operations.md, 1.5.
- **An answer fits its call.** The client checks each answer against
  its call before it takes it (client.md, section 2.3):

  | Call | An answer fits when |
  |---|---|
  | any | it has the shape in 1.2's table; a read's answer past `answer_bytes`, or with a list past `rows` but `Branches`' names, is `TooLarge`, naming the bound with the answer's exact measure (1.4), a write's `InvalidAnswer`, a lost answer (client-effects.md, 1.4); any other mismatch is `InvalidAnswer` |
  | `Items` | `updated` never decreases; rows of one time rise strictly by number; numbers are nonzero and unique; each row's kind is the call's, when it names one |
  | `Item` | the number is the call's; comment ids rise strictly and are above `after` |
  | `Reviews` | ids rise strictly |
  | `Pull`, `PullFor` | the number is the call's; for `PullFor`, the head and base are the call's |
  | `PullFiles` | the head is the call's |
  | `Compare` | `before` and `after` are the call's, and `files` holds at most `changed` paths |
  | `Range` | `before` and `after` are the call's |
  | `Checks` | each job's head is the call's commit |
  | `Job` | the attempt is the call's, and the log is within `max_bytes` |
  | `Runs` | each run's workflow and commit are the call's; ids are unique |
  | `Run` | the id is the call's |
  | `Tree` | names rise strictly |
  | `Commits` | the ids are unique |
  | `Artifact` | the bytes are within `max_bytes` |
  | `File` | the head and path are the call's, and the bytes are within `max_bytes` |
  | `Locate` | the id is nonzero |

- **The measure.** A call's or an answer's bytes are the lengths of the
  byte strings it holds, plus a fixed width for each element of each
  list in it; a value outside a list adds no width. `skein-forge`'s
  `measure::op(&Op)` and `measure::answer(&Answer)` give them, none when
  the sum passes a `u64`. The widths:

  | Element | Width |
  |---|---|
  | a `Summary`, a `Review` | 128 |
  | a `Status` | 160 |
  | a `Comment` | 96 |
  | a `Remark`, a `File` | 64 |
  | a `Commit` | 32 |
  | a `Run` | 128 |
  | a `RunJob`, an `Entry` | 64 |
  | an input | 32 |
  | a `Collaborator`; a string in a list (a label, a branch, a context, a compared path) | 16 |
  | an id in a list (a reviewer) | 8 |

  Each width is at least its type's size on the target, asserted when
  the crate is built, so the measure bounds memory and is one on every
  target. A row count is no part of it: `rows` is checked beside it, on
  every list but `Branches`' names, which are small and bounded by their
  bytes alone, each its width and its length (`forgejo-operations.md`, 1). `Range` holds no
  list, and measures 0.
  The page size (`forgejo.md`, 1.1), the client's checks (client.md, section 2.3)
  and the component's limits (connection-machines.md, section 1) all use
  it.
- **A call is admitted within bounds.** The client admits an op only
  when its bytes are at most `op_bytes`, a `Job` or `File`'s `max_bytes`
  is between 1 and `answer_bytes`, a page is at least 1 and a file's
  path is not empty. The component and the plans never see one that is
  not (client.md, section 2.3).
- **The vocabulary retains no state.** A row is a value, and the owner
  stores it as it likes.
