# The forge kit

Provisional, 2026-10-20. What skein gives a service that works on a code
forge, on Forgejo and on GitHub (github.md; section 8). The kit
is:

- a neutral vocabulary of forge calls, rendered by a dialect;
- a component that runs those calls and receives the forge's webhooks;
- a client domain that keeps a bounded view of a forge and writes to it
  at most once;
- a setup machine that makes a forge ready for a service;
- the fake forge and the conformance tooling that a service's worlds and
  tests need.

This document is the kit as a whole and its first crate, `skein-forge`:
the vocabulary, the dialects, keys' markers, deliveries and credentials.
The others are connection.md (the component), client.md (the
client), setup/README.md (the setup) and ../testing/fakes/forge.md (the fakes, the
throwaway Forgejo and the conformance).

## 1. In one page

- **What a connector stands on, not the connector.** The kit is the
  layers a forge connector is built on: the documents, the call plans,
  the component, the client's keep and write lanes, the setup's machines
  and the fakes. It holds nothing a connector decides: what to watch,
  which effects exist and what each is for, when to write, and what
  people see (section 2.3). The kit names no service.
- **Shaped like skein's LLM crates.** `skein-forge` is to forges what
  `skein-llm` is to providers: a neutral vocabulary rendered by
  dialects. Each call is answered once, and nothing is retried.
  `skein-forge-connection` is to it what `skein-llm-connection` is to
  `skein-llm`: a component that an owner's protocol layer owns, and that
  connects, runs calls and keeps connections (../llm/README.md;
  ../llm/connection.md).
- **One call, one answer, and its cost.** Each `Read` or `Write` is
  answered by exactly one `Answer` or `Error`, made of one or a few HTTP
  exchanges. The answer carries the count of requests the call cost.
  Budgeting the forge's rate is the client's job (client.md,
  section 6).
- **Repositories by the forge's id.** A repository is named by the
  forge's number and the forge's own id, which survives a rename or a
  transfer. A repository is first found by its owner and name, with a
  locate.
- **The owner names its marks.** A creation's key travels in a marker
  whose word the owner gives (`<!-- <word>:key … -->`; temper's word is
  `temper`). The user agent is the owner's, and setup marks its bot with
  the same word.
- **Dialects hold the documents and the plans.** Forgejo's dialect holds
  both sides of each document, its deliveries, the setup's documents and
  its pin. Each call is run by a call plan of the dialect: a small pure
  machine that does no io (section 4).
- **Credentials have a kind, set by the dialect.** A token lasts until
  revoked (Forgejo's, or a GitHub fine-grained token). A GitHub App
  installation's token is minted from the App's key and lasts an hour.
  The component holds the current credential, renews it when the dialect
  says, and gives it to no one (section 7).
- **Webhooks for latency, polling for liveness.** Deliveries are
  verified as they stream and become hints. The client's keep polls
  whether or not they arrive.
- **The client keeps and writes, and decides nothing.** Its working set,
  request budget and write lanes are bounded. Each write carries a key
  that its owner made, so a write whose answer was lost is found after a
  restart, not made twice. What an effect is for is opaque bytes of the
  owner's.
- **Setup brings a forge to one kind of result: an access.** On any
  forge that result is the same:
  - the user it acts as;
  - its API and git credentials;
  - the repositories it may write;
  - a webhook secret and its hooks.

  Each forge has its own plan to reach it. Forgejo's is reconcile, with
  a site admin's token. GitHub's is the App manifest flow
  (setup/README.md). The kit supplies only the machines; the host is the service's.
- **The fake forge is shared, with a layer per dialect.** Its domain is
  one; Forgejo's protocol layer and GitHub's follow their dialects, and
  skein-fake-peers hosts them. Forgejo's conformance runs against a
  throwaway Forgejo and records the facts that section 9 lists;
  GitHub's runs against the fake, and opt-in against github.com
  (../testing/fakes/github.md, sections 1 and 2).
- **GitHub is a second dialect,** `skein_forge::github`, beside
  `forgejo` (github.md). It adds to the vocabulary validators, the
  rate's window, `Queued`, the protection's queue and bypass,
  `NotConnected`, `NotInstalled` and the App's credential (section 8).
  Forgejo's dialect ignores each addition or answers its neutral value.

## 2. The kit's crates

### 2.1 The crates

```
skein-forge                     the vocabulary (api, locate); what the dialects share (4.4); the
│                                 dialects: Forgejo's documents, both
│                                 sides, its call plans, its deliveries, the setup's documents, the
│                                 pin; GitHub's likewise (forge-github.md); keys' markers;
│                                 credentials' kinds
├── skein-lib, skein-json
skein-forge-client              the keep (working set), the request budget, the write lanes and their
│                                 recovery, keys
├── skein-lib, skein-forge (its vocabulary only)
skein-forge-connection          the component: calls, connections, each forge's start, repositories'
│                                 paths, users, credentials and their renewal; the webhooks' listener
├── skein-lib, skein-io, skein-http, skein-tls, skein-json, skein-forge
skein-forge-setup               the access and each forge's plan: Forgejo's reconcile machine;
│                                 GitHub's manifest flow
├── skein-lib, skein-io, skein-http, skein-json, skein-forge
crates/skein-fake-forge-domain     the fake forge's domain
crates/skein-fake-forge-http       what its two layers share: a connection's core and the git route
crates/skein-fake-forge-forgejo   Forgejo's API over it on skein's HTTP server; the setup's routes;
                                     signed deliveries; the tally of requests served
crates/skein-fake-forge-github    GitHub's API over it, at its pin (../testing/fakes/github.md, section 1)
crates/skein-fake-peers            hosts the forge peer, beside the LLM and OAuth peers
testing/skein-throwaway-forgejo    a real Forgejo and its runner, at the pin, for the conformance
tests/forge/machine, protocol,     the kit's worlds, one crate each (section 11)
  client, setup
tests/forge/conformance            a tool outside the suites: records the dialect's facts
tests/forge/github                 GitHub's conformance: on the fake, and opt-in live (../testing/fakes/github.md,
                                     section 2)
```

- **`skein-forge` is `no_std` (with `alloc`) and lib-only,** as the
  client is, whose vocabulary it carries. It has no io and no clock.
- **The client and the component do not depend on each other.** Both
  stand on the vocabulary. The owner carries calls to the component, and
  answers and hints back to the client (client.md, section 2;
  connection.md, section 2).
- **Each crate has its document:** connection.md for the
  component, client.md for the client, setup/README.md for the
  setup, and ../testing/fakes/forge.md for the fakes, the forge peer, the throwaway
  and the conformance.

### 2.2 What the owner gives

The owner is the service code that owns a part: its protocol layer owns
the component, its forge connector (a domain) owns the client, and its
setup shell hosts reconcile. The kit reads no file and no configuration
(connection.md, section 3). The owner hands it values:

- **The word.** The marker's and the bot's (section 5).
- **The user agent,** on every request (`forgejo.md`, section 1.1).
- **The namespace and the identity.** The namespace is the owner's keys'
  namespace. The identity is the owner's bytes for an effect, opaque to
  the kit (client.md, section 7).
- **For each forge:** its number, its dialect, its API's address and
  transport, the value of its credential, the forge user that credential
  must stand for, and its webhook secret when it has one.
- **The listener's address,** when webhooks are used.
- **The limits** of each crate it owns.

### 2.3 What stays with the service

- **The connector's top.** It is what the client's owner builds on the
  client:
  - what it watches;
  - which effects exist, and what each is for;
  - policy: gates, merges and landings;
  - adoption's checks;
  - what people see.

  It is not in skein: it is the service's.
- **The protocol layer's translation,** between its own boundary and the
  kit's calls, answers and hints; and the store's records of the
  client's rows (the `Stored` rows that the client hands its owner to
  keep).
- **The configuration and its files.** The kit reads none.
- **The setup's host:** its command, its files, the forge's lock, its
  exit status. Owned Forgejo's preparation (`app.ini`, migrations, the
  runner's registration, upgrades' dumps) is the service's.
- **The service's git component,** and that component's fakes.
- **The service's worlds' stories,** and its cost ceilings. They run on
  the kit's fakes.

## 3. The vocabulary

### 3.1 What it is

`skein-forge`'s `api` and `locate` modules are the forge's neutral
vocabulary. It describes bounded requests and answers between the
client and a dialect, and retains no state. It names no task, no grant
and no policy. Every type is a plain value.

- **Calls:** `Op`, which is a `Read` or a `Write`.
- **Their result:** `Answer`, `Error`, and the enums they use (`Kind`,
  `State`, `Ci`, `Check`, `Verdict`, `Permission`, `BranchCreation`,
  `Provenance`, `Bypass`).
- **What an answer says of the forge** (3.3): `Rate`, the rate's window,
  and `Validator`, a read's validator.
- **A hint:** `Hint`, with its `Change`.
- **The items** that answers carry (3.6).
- **Addressing:** `Repository { forge, repository }`, `forge` being the
  owner's number for a forge (16 bits) and `repository` the forge's own
  id (32 bits). A commit is 32 bytes (a SHA-1 id zero-padded). Times
  are `skein_lib::Time` and `Duration`.

### 3.2 The calls

25 reads and 18 writes, in families. Each call has one answer.

| Family | Calls | Answer |
|---|---|---|
| Find a repository | `Locate` | `Located` |
| The change feed | `Items` | `Items` |
| One item | `Item`, `Pull`, `PullFor` | `Item`; `Pull`; `Pull` |
| Review | `Reviews`, `Remarks` | `Reviews`; `Remarks` |
| CI | `Statuses`, `Checks`, `Job` | `Statuses`; `Checks`; `Job` |
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
  items (`forgejo.md`, 1.2).
- **A keyed write** takes the key the owner made. The dialect writes it
  into the marker (section 5). A pull request's key is its branches, so
  `OpenPull` takes none.
- **`PullFor { head, base }`** asks for the open pull request of a head
  and a base, else the oldest. It reads no CI.
- **`Pull { number, ci }`** asks for a pull request and, when `ci`, its
  head's CI, folded as `Checks` folds its rows. Without `ci` the forge is
  asked for no status, and the answer's `ci` is `Ci::None`, as `PullFor`'s
  is: a reader that uses CI asks `ci`, or reads `Checks`.
- **`Reviews`, `Remarks` and `PullFiles` take `skip`,** the rows of their
  page already read: a page the answer's bytes cut (3.3) is read on at
  the same page past them, so a cut loses no row and repeats none.
- **`PullFiles { number, head, page, skip, sides }`** reads a page of a
  pull request's changed files at `head`. With `sides`, each `File`
  carries its contents at the merge base and at the head, `None` for a
  side the file lacks; without, its path alone, both `None`, and the
  dialect reads neither a merge base nor a side: the files' listing is
  the read. A reader that needs paths alone, as goal overlap does
  (temper's `domain/forge/README.md`, 5.3), asks without `sides`.
- **`Item { number, after, since }`** asks for an item and its comments
  with ids above `after`. `since` is a time at or before the update of
  every comment above `after`, which the dialect sends as its forge's own
  filter so that the forge sends fewer; 0 sends none. `after` stays the
  answer's check (3.7): a comment at or below it that the forge still
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
  a goal's report quotes (temper's `domain/forge/branches.md`, 2). `File {
  head, path, max_bytes }` asks for the start of one file at a commit.
  Both are cut to `max_bytes`, or to what the answer has left (`forgejo.md`, 1.3), and
  say when they cut: a `Start` that cut kept the log's first bytes and
  dropped the rest.
- **`Merge { number, head }`** is conditional on the head. A merge's
  base is the client's precondition, checked by reading the pull
  request afresh (client.md, section 7). Where a merge queue
  guards the base, it answers `Queued`: the pull request was put in the
  queue at that head, and the landing comes later, as a hint
  (github.md, section 5.1).
- **`Protection`** needs an admin to read an absent rule. A writer who
  is not an admin gets `Forbidden` (`forgejo.md`, 1.2).
- **`Merge { number, head, how }`** merges in the repository's own style
  (`Style`), or `FastForwardOnly`: the base moved to exactly the head,
  refused (`Conflict`) when the head does not contain the base's tip.
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

### 3.3 The answers

30 answers: `Located`, `Items`, `Item`, `Pull`, `Reviews`, `Statuses`,
`Remarks`, `Commit`, `Branches`, `Commits`, `PullFiles`, `Compare`, `Range`,
`Checks`, `Job`, `File`, `Tree`, `Runs`, `Run`, `Protection`, `Settings`,
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
- **`Compare`** carries `ahead`, the commits `after` has that `before`
  lacks; `behind`, those `before` has that `after` lacks, 0 when `after`
  contains `before`; `files`, the changed paths in the forge's order, as
  many as the answer holds, at most `rows` within `answer_bytes`; and
  `changed`, how many files changed in all, so that `changed` past
  `files`' length says how many it left out. A comparison is refused for
  neither the paths it leaves out nor its commits, so two branches
  however far apart compare, unless the forge cuts its own listing
  (github.md, 4.1).
- **`Queued` is final for its call.** It answers `Merge` alone, and
  says the merge is the queue's now: neither a merge nor a refusal. The
  client keeps the write open until a read finds the pull request merged
  or out of the queue (client.md, section 7). Forgejo's dialect
  never answers it.
- **What an answer says of the forge,** beside its cost, travels with
  it in the component's envelope (connection.md, section 4), and
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
    (connection.md, 4.6). Both dialects send them on every read;
    Forgejo sends an `ETag` on a file's raw read and none on a branch's
    JSON, and a read with none kept is sent as before.

### 3.4 The errors

20 errors. A call's `Error` is the first failure of its requests, and
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
| `Conflict` | the change conflicts with its base |
| `Protected` | a protection rule refused the write |
| `Refused` | the repository's settings, or the forge's policy, refuse the write |
| `InvalidAnswer` | a decoded answer does not match its call |
| `Busy` | the client's own admission refused the call; no API call was made |
| `NotConnected` | the forge is not connected, or stopped; no API call was made |
| `NotInstalled` | no installation of the App reaches the repository's owner; no API call was made |

- **`Busy` is never a dialect's.** It is the client's own refusal
  (client.md, section 2.3). The component never produces it
  (connection.md, section 7).
- **`NotConnected` and `NotInstalled` are the component's,** never a
  dialect's, and cost nothing (connection.md, section 7).
  `NotInstalled` is GitHub's alone: the component chooses an
  installation by the repository's owner (section 7).
- **A pause is `RateLimited { after }`:** `Retry-After` when the forge
  gives it; else, when the window has none left, its reset less the
  forge's clock; else, for GitHub's secondary limits, 60 seconds
  (github.md, section 7).
- **`Timeout` is the error that leaves a write's outcome unknown.** The
  client finds the outcome by the write's key, or by reading
  (client.md, section 7).
- **`TooLarge` names its bound and its size,** as `Excess { bound,
  limit, size, exact }`, a plain value:
  - `bound`, a `Bound`: `OpBytes`, `AnswerBytes` or `Rows`, the client's
    own configured bounds (client.md, 2.2 and 2.3), passed by a
    call's bytes, an answer's bytes by the measure (3.7), or an answer's
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

### 3.5 The hint

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
  (section 6).

### 3.6 The items

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
  files' listings are capped (`github.md`, 4.1), so a reader that keeps
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
  credential's bypass of the rules that do (github.md, 4.5). Forgejo's dialect reads signatures
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

### 3.7 The rules on them

- **One answer per call,** and its cost: the number of requests the call
  wrote, the fills of the component's tables included, and 0 when it
  wrote none. The component reports the cost with the answer
  (connection.md, section 4), and the budget charges it
  (client.md, section 6).
- **Repositories by the forge's id.** A call names `Repository`; the
  dialect addresses the forge by the repository's path, which the
  component keeps for the id (connection.md, section 4).
- **Locate by owner and name.** `Locate { owner, name }` names a
  repository's account and name, and is addressed with the forge's number
  and repository 0. It answers `Located` with the forge's id, which is
  never 0. The call's bytes are the lengths of its `owner` and `name`.
- **Every dialect answers every call.** A call that a dialect cannot
  answer as Forgejo does is named when that dialect is designed (section
  8). GitHub's one is `Status`, which answers `Forbidden` where the
  App lacks `statuses` write (github.md, section 4.2). The
  Actions calls, `Tree`, `Artifact`, `Commits`, a fast-forward-only
  merge and an update by rebase take GitHub's routes of
  github.md, 4.5.
- **An answer fits its call.** The client checks each answer against
  its call before it takes it (client.md, section 2.3):

  | Call | An answer fits when |
  |---|---|
  | any | it has the shape in 3.2's table; a read's answer past `answer_bytes`, or with a list past `rows` but `Branches`' names, is `TooLarge`, naming the bound with the answer's exact measure (3.4), a write's `InvalidAnswer`, a lost answer (client.md, 7.4); any other mismatch is `InvalidAnswer` |
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
  bytes alone, each its width and its length (`forgejo.md`, 1.2). `Range` holds no
  list, and measures 0.
  The page size (`forgejo.md`, 1.1), the client's checks (client.md, section 2.3)
  and the component's limits (connection.md, section 7) all use
  it.
- **A call is admitted within bounds.** The client admits an op only
  when its bytes are at most `op_bytes`, a `Job` or `File`'s `max_bytes`
  is between 1 and `answer_bytes`, a page is at least 1 and a file's
  path is not empty. The component and the plans never see one that is
  not (client.md, section 2.3).
- **The vocabulary retains no state.** A row is a value, and the owner
  stores it as it likes.

## 4. Dialects and their call plans

### 4.1 What a dialect holds

A dialect renders the vocabulary on one forge's API. Forgejo's, at its
pin (4.3), holds:

- **Each request,** as a sized encoder: its bytes are counted before any
  is written, so a request's head and body are within the bounds the
  component gives.
- **Each answer,** as a decoder: a state machine over skein-json's
  tokens with a bounded stack, which keeps the few fields its answer
  needs and skips the rest as they pass (`forgejo.md`, 1.3).
- **The server's side of each document,** for the fake forge: the
  decoder of each request, and the encoder of each answer
  (../testing/fakes/forge.md, section 4).
- **The failures:** error bodies, the refusals whose message tells two
  failures of one status apart, and the map from a status to an `Error`
  (`forgejo.md`, 1.4).
- **The deliveries:** the headers, the events subscribed, each payload's
  decoder and encoder, the table to hints and the listener's path for a
  forge (section 6).
- **The setup's documents:** the admin's reads, the bot, its tokens,
  orgs, a repository's read by id, collaborator and hooks, and the
  repositories the admin can see (setup/forgejo.md, section 1).
- **Names, times and ids:** percent-encoding of a path's segments, RFC
  3339 times in both directions, and object ids in 40 or 64 hexadecimal
  digits.
- **How a credential renders on a request** (section 7).
- **The start's reads,** two call plans that the component runs when a
  forge starts and decides from, counted in no call's cost
  (connection.md, section 3):
  - **the identity read,** which says who the credential stands for:
    Forgejo's is `GET /api/v1/user`, for the user's `id`;
  - **the page cap read,** which gives the forge's page cap: Forgejo's
    is `GET /api/v1/settings/api`, for `max_response_items`, which must
    be above 0 (`forgejo.md`, 1.1).
- **The renewal,** for a renewed kind of credential: the plan that
  renews it, and how long before expiry it is due (section 7).
- **The pin** (4.3).
- **The call plans** (4.2).
- **Its limits,** `Limits`, which the component's owner gives:
  - `json` and `writer`: skein-json's tokenizer and sized writer;
  - `string_bytes`: the largest name, title, context, description, link
    or path kept;
  - `body_bytes`: the largest body kept whole;
  - `inner`: the most elements kept of a list inside one document;
  - `target_bytes`: the longest request target, its path and query.

  `check` refuses a zero, a `json.string` below `string_bytes` or
  `body_bytes`, and a depth below 8 (Forgejo's documents need about 8).
  The component's own limits (the connections, the bytes of a head or of
  a body read, the deadline) are connection.md's, section 7.

A dialect holds no io, no clock and no connection.

GitHub's dialect, `skein_forge::github`, holds the same at its own pin,
with the App's documents (the JSON Web Token's bytes, the mint and the
installations' reads) where Forgejo's has the setup's admin reads
(github.md, section 2).

What holds for both dialects alike sits at the crate's root (4.4).

### 4.2 Call plans

A call plan is a dialect's small pure machine for one call. The
component runs any dialect's plans on its connections, and the only
dialect knowledge it keeps is in the plans it is handed.

- **Its steps.** `begin` takes the call and returns a step. `heard`
  takes what the plan heard and returns the next. A step is one of:
  - `Send`: an exchange, to be made on the call's connection. It holds
    the request and a range: the head or the tail bytes of a raw answer
    to keep (a job's log is kept from its end, a file from its start);
  - `Done`: the call's `Answer` or `Error`.
- **What a plan hears is the dialect's type.** Forgejo's plans hear,
  for an exchange:
  - `Head`, with the status and the few header fields they read
    (`Date`, `Retry-After`, `X-HasMore`, `Content-Range` and
    `Connection: close`);
  - `Row`, for each element of a listing that the exchange's decoder
    keeps;
  - `Done`, with the body: a document the decoder read, raw bytes (and
    whether more follow), an empty body, or a malformed one;
  - or `Failed`: the exchange failed with the server never having seen
    the request (`Unseen`), or possibly having seen it (`Seen`).
- **It is handed its bounds as plain values,** not the client's
  `Limits`: the page size (`forgejo.md`, 1.1), `rows`, `answer_bytes` and `body_bytes`.
  The component takes them from the client's limits and its own, and
  checks them at start (connection.md, section 7).
- **It does no io and keeps no connection.** The component makes each
  exchange, and tells the plan what it heard.
- **It resolves a repository's path and a user's login** in two tables
  that the component keeps and hands it: it reads a row, and fills one
  from the answers it hears (connection.md, section 4). A row of
  a repository holds its `owner` and `name` (its path), the object
  format, the merge style and the `html_url`. The tables are lib's `Lru`
  (../runtime/lib.md, 5): bounded, and they evict the row used longest ago.
- **It ends with the call's result,** its `Done`, and counts nothing:
  the call's cost is the component's, one for each request whose first
  byte it wrote (connection.md, section 4.1). A plan ends on the
  first failure, except where `forgejo.md`, 1.2 says otherwise.
- **It maps a failure to an `Error`.** `classify` maps a status, a
  refusal's message and `Retry-After` to an `Error`, and says whether
  the path or the row is dropped (`forgejo.md`, 1.4).
- **It knows its dialect and its call,** and nothing of the budget or of
  the other calls in flight.

### 4.3 The pin

The dialect is pinned to one release of the forge, the one whose
behaviour the conformance checked.

- **Constants:** `VERSION`, Forgejo's release (17.0.1), and
  `RUNNER_VERSION`, forgejo-runner's (13.2.1).
- **Checks:** `names_forgejo` is true for the first line of
  `forgejo --version` when it names `VERSION`, ended by nothing, a `+`
  or a space; `names_runner` is true for the first line of
  `forgejo-runner --version` when it is `forgejo-runner version v`
  and `RUNNER_VERSION`, and nothing more.
- **Readers:** the throwaway Forgejo, the conformance, and a service's
  owned Forgejo check their binaries against the pin (../testing/fakes/forge.md,
  section 6).
- **Digests on Linux:** the SHA-256 of each release's Linux binary, per
  architecture, as lowercase hex:

  | Constant | Of | Bytes | SHA-256 |
  |---|---|---|---|
  | `FORGEJO_SHA256_LINUX_AMD64` | `forgejo-17.0.1-linux-amd64` | 122,057,320 | `51a8cdf4a1252e79f4b3855d7a7891af6c6635105420768c460e2f0690385d6f` |
  | `FORGEJO_SHA256_LINUX_ARM64` | `forgejo-17.0.1-linux-arm64` | 113,764,224 | `1e99403746406e8199efc903ddd1940d5728a60e50241f7aa6c9f2b907bd8ec9` |
  | `RUNNER_SHA256_LINUX_AMD64` | `forgejo-runner-13.2.1-linux-amd64` | 22,306,978 | `bb369fe1ecb50fdc983c0b1d993f19b0920df6e5864511892b3cdbf16d858b34` |
  | `RUNNER_SHA256_LINUX_ARM64` | `forgejo-runner-13.2.1-linux-arm64` | 20,840,610 | `eeddb7902f79abe488972f4189fbe3ea02762ee827d1b406b469420bf0d62dd9` |

  The sizes are not constants: they are what the digests were taken
  over, and what a fetch's bound must admit.
- **Digests on macOS:** neither project publishes a darwin build, so
  the pins are Homebrew's bottles for macOS 15 (`arm64_sequoia`), on
  arm64 alone, two digests for each binary: the archive's, which is its
  address, and the binary's inside it, which a file already in place is
  held to. A bottle is the same version as Linux's release and another
  build:

  | Constant | Of | Bytes | SHA-256 |
  |---|---|---|---|
  | `FORGEJO_BOTTLE_SHA256_MACOS_ARM64` | the bottle of `forgejo` 17.0.1 | 43,841,143 | `19db7c548064dfd4097264a7b4c4cf3b228bfd6190bc33a3c6e8a9b26c3d0e06` |
  | `FORGEJO_SHA256_MACOS_ARM64` | its `forgejo/17.0.1/bin/forgejo` | 111,660,034 | `4b6ed0e7e6ef034d298872125f69332e2aad49367435f66c58ee5eb2c5680cf6` |
  | `RUNNER_BOTTLE_SHA256_MACOS_ARM64` | the bottle of `forgejo-runner` 13.2.1 | 7,960,512 | `0a8cb32f024121e50ebca1840ed2f5650d65d3eb3e183679c0f8f0bc14d6fb48` |
  | `RUNNER_SHA256_MACOS_ARM64` | its `forgejo-runner/13.2.1/bin/forgejo-runner` | 21,661,778 | `2829f0125068f0ad92f908cb08c4d3c2f458cc22eec15fd675c332823cbece4b` |

  Each binary's load commands name macOS's own libraries alone
  (`libSystem`, `libresolv`, `CoreFoundation`, `Security`).
- **Sources:** each binary's release file, named by the constants and
  the architecture:
  - Forgejo's on codeberg.org,
    `https://codeberg.org/forgejo/forgejo/releases/download/v17.0.1/forgejo-17.0.1-linux-{amd64,arm64}`;
  - forgejo-runner's on code.forgejo.org,
    `https://code.forgejo.org/forgejo/runner/releases/download/v13.2.1/forgejo-runner-13.2.1-linux-{amd64,arm64}`.
    Codeberg serves no runner release (404).

  Each answers a single `200` with the file, and no redirect. A Linux
  fetcher follows none, and takes any other status as the source
  failing.
- **The bottles' sources,** fixed by the archive's digest, never looked
  up: `https://ghcr.io/v2/homebrew/core/forgejo/blobs/sha256:DIGEST` and
  `https://ghcr.io/v2/homebrew/core/forgejo-runner/blobs/sha256:DIGEST`.
  Asked with `Authorization: Bearer QQ==`, the anonymous value Homebrew
  sends, each answers a `307` to its storage's `https` address, which
  answers `200` with the archive and is sent no token.
- **TLS:** with the machine's roots. The digest, not the certificate, is
  what makes the file the pinned one.
- **An architecture without a digest** is refused: a fetcher names the
  architecture and fetches nothing. The pins are Linux on amd64 and
  arm64, and macOS on arm64.
- **Readers of the digests:** a service that fetches the pinned binaries
  (temper's `forge fetch`) checks each file it streams, and each file it
  finds already in place, against the digest of its own platform and
  architecture. skein-forge does no io for this: it holds the constants
  and a `digest(binary, platform, arch, form)` that answers the pinned
  digest, or none where there is no such pin: `Platform` is `Linux` or
  `Macos`, `Form` is `Binary` or `Archive`, and Linux has no `Archive`.
- **Moving the pin** moves the digests with it: a new `VERSION` or
  `RUNNER_VERSION` comes with the new files' digests, all of them, taken
  as above, in the same change, and only to versions with bottles.

### 4.4 What the dialects share

What does not depend on a forge's documents sits at `skein_forge`'s
root, and both dialects stand on it:

- `time`: RFC 3339 and the HTTP date, on lib's `Civil` (../runtime/lib.md, 8);
- `path`: percent-encoding and a target's reading;
- `text`: `Body`, a text kept whole or only counted;
- `decode`: `Malformed`;
- `limits`: `Limits`, `Problem` and `check`, which takes the depth a
  dialect needs (8 for Forgejo, 10 for GitHub);
- `plan`: an exchange's `Failure` and the `Error` each reads as (`forgejo.md`, 1.4);
- `users`: the logins' table (4.2).

A dialect keeps its public name for each by re-export, and names no item
of the other dialect. Its documents, ids, plans, `Bounds` and repository
rows stay its own. The dialects stay modules of one crate (section 10).

## 5. Keys' markers

What the kit writes inside what it creates: keys' markers, and nothing
else. No record, outcome or page of the owner's is written into the
forge. Labels are written only by the calls that name them
(`CreateIssue`'s, `Labels`, `CreateLabel`); the
kit decides nothing by one, and whether its owner reads them back is
the owner's.

- **A creation's key,** at the head of an issue's body, a comment or a
  review: `<!-- `, the word, `:key `, the key in lowercase hexadecimal,
  then ` -->` and a line feed, then the text. It is an HTML comment,
  which Forgejo's rendering hides. A pull request has no marker: its key
  is its branches.
- **The word is the owner's.** It is 1 to 16 bytes of lowercase ASCII
  letters, digits and `-`, checked at start. The same word marks the
  setup's bot, `<word>:bot` (setup/forgejo.md, section 1).
- **Services that share a forge** each have their own word, their own
  bot and their own keys' namespace. A body whose marker carries
  another word has no key.
- **The key is the owner's bytes.** The client builds it with
  `effect_key(namespace, identity, max)` (client.md, section 7):
  - version 1, one byte;
  - the namespace's length, as two bytes big-endian;
  - the namespace, which is the owner's;
  - the identity, which is the owner's opaque bytes for the effect.

  The key's first byte is its version; the marker has none of its own.
  Owners that share a repository tell their creations apart by
  namespace.
- **Decoded** from a body whose first bytes are `<!-- `, the word and
  `:key `, then an even count of lowercase hexadecimal digits, then
  ` -->` and a line feed; any other body has no key.
- **The marker code only encodes and decodes,** and never reads the key.
  Whether to believe a key is the client's call: its version, its
  namespace, and whether its author is a configured writer
  (client.md, section 7). A person who copies a marker into their
  own comment gives it no provenance.
- **Bounded.** The key is within the client's `op_bytes`. The marker is
  twice the key's length plus the word's length plus 15 bytes: with a
  6-byte word, twice the key plus 21. The body sent is the marker and
  the text, which `body_bytes` holds (`forgejo.md`, 1.3).
- **One code serves every use:** the writes' encoders, and the decoders
  of items, comments, reviews and deliveries.

## 6. Deliveries

Forgejo's deliveries, made hints: they cut polling's latency, and
decide nothing. Polling stands with no webhook (client.md,
section 4), and finds whatever a delivery lost, refused or never sent
told of. The listener, its connections and its caps are
connection.md, section 6. This section is what the dialect holds.

- **The listener's path.** A forge's deliveries arrive at
  `/hooks/<number>`, the forge's number in decimal with no sign and no
  leading zero, within 16 bits. A path of another shape names no forge.
- **Signed, and verified as it streams.** Forgejo signs each delivery's
  body with its hook's secret: HMAC-SHA256, in lowercase hexadecimal, in
  `X-Forgejo-Signature`. The component computes it as the body arrives,
  with skein lib's `sha256::Hmac` keyed by the forge's secret, and
  decodes the body at the same time into the few fields its hint reads;
  nothing of it is believed until the whole body is read and the
  signature holds, compared with `sha256::equal`, in constant time
  (../runtime/lib.md, section 6.2).
- **Its verdicts:**

  | A delivery | Answer | Hint |
  |---|---|---|
  | with no signature | `403` from its head, its body unread | none |
  | with a wrong signature | `403` once its body is read | none |
  | whose body length passes its cap | `413` | none |
  | verified, and its body malformed | `400` | none |
  | verified, and an event of no row below (a tag's included) | `204` | none |
  | verified, and mapped | `204`, queued first | one |

  An unknown path, a forge with no secret and a method other than `POST`
  are the listener's (connection.md, section 6).
- **Answered first.** Once the body is read and verified, the response
  (`204`) is queued, and only then does the hint go up. The component
  never makes Forgejo wait on its owner's work. skein's HTTP server lets
  the component answer when it chooses (http.md, section 5.4).
- **The events a hook subscribes to** are every event whose delivery the
  table maps to a hint, and no other: 22 in all. The component reads
  each delivery by `X-Forgejo-Event`, the header it arrives under, which
  several events share; each hint's repository is the forge's number and
  the payload's `repository.id` (`run.repository.id` for a job's), so no
  path is needed (`forgejo.md`, 1.1). A repository id past 32 bits makes no hint.

  | Events subscribed | `X-Forgejo-Event` | `Hint` |
  |---|---|---|
  | `issues`, `issue_assign`, `issue_label`, `issue_milestone` | `issues` | `Change::Item(number)`, with the key of the issue's body |
  | `pull_request`, `pull_request_assign`, `pull_request_label`, `pull_request_milestone`, `pull_request_sync`, `pull_request_review_request` | `pull_request` | `Change::Item(number)`, with no key: a pull request's key is its branches (section 5) |
  | `issue_comment`, `pull_request_comment` | `issue_comment` | `Change::Item(number)`, with the key of the comment's body |
  | `pull_request_review_approved`, `pull_request_review_rejected`, `pull_request_review_comment` | `pull_request_approved`, `pull_request_rejected`, `pull_request_comment` | `Change::Item(number)`, with the key of the review's body |
  | `push` | `push` | of a branch, `Change::BranchMoved { branch, head, actor }`, the commit it moved to and the pusher's id; of a branch deleted (its `after` all zeros), `Change::Branch(branch)` |
  | `create`, `delete` | `create`, `delete` | of a branch, `Change::Branch(branch)` |
  | a job's ends: `workflow_job_success`, `workflow_job_failure`, `workflow_job_cancelled`, `workflow_job_skipped` | the same | `Change::Commit(commit)`, the run's `commit_sha` |
  | none | anything else; a tag's `push`, `create` or `delete` | nothing |

  A body not kept whole (`forgejo.md`, 1.3) gives no key. The decoder yields a
  `Delivery`: `Issue`, `Pull`, `Comment`, `Review`, `Push`, `Ref` or
  `Job`, a job's end being `Success`, `Failure`, `Cancelled` or
  `Skipped`. Each row's events, its header and the fields its hint reads
  are recorded on the pinned release (`forgejo.md`, section 2). A row that went wrong
  would cost a poll's wait, never a decision: a hint only brings a read
  sooner (client.md, section 4).

  **Who caused it** is what lets the owner drop its own writes' echoes,
  by key (a creation's, for an item) and by pusher (for a push); the
  client reads on every hint it holds a resource for, and uses the
  pusher for drift alone (client.md, section 4). A job's delivery
  has no top-level repository or sender: its `run.repository.id` and
  `run.commit_sha` stand for them, and its hint names the commit and no
  actor. A hint for a repository the owner has not adopted goes up all
  the same; the owner drops it.
- **CI is hinted as each job ends.** A job's delivery is sent for each
  of its attempts, so a job run again is hinted again when it ends. The
  run's deliveries (`workflow_run_*`), sent once its last job ends, and
  the deprecated `action_run_*` are not subscribed.
- **What has no delivery** is found by polling alone (client.md,
  section 4): a commit status, which Forgejo never delivers, so CI that
  a system other than Actions reports and the verdicts the owner posts
  as statuses.
- **What Forgejo settles after it delivers** (a pull request's
  mergeability, a job's commit status: `forgejo.md`, section 2) is read again by the
  owner's fresh reads, within its `settle` of the move; the client runs
  no window for it (client.md, sections 4 and 5). The component
  hints at once.
- **Bounded.** Push payloads list commits, which the decoder skips. The
  listener's connections, head, body and deadline are capped
  (connection.md, sections 6 and 7).
- **Not sent again.** Forgejo does not resend a failed delivery: one
  lost while the owner is down, refused for its size or its signature,
  or never sent is found by polling. A hint that arrives before the
  owner's domain is ready is dropped, and the keep's first listing finds
  what it told of.
- **Hooks are set up by reconcile, or by the person,** at the listener's
  URL for the forge, with the forge's secret, the events above, JSON and
  active (setup/forgejo.md, section 1.5). A repository with no hook polls.

## 7. Credentials

- **A credential has a kind, and each dialect names the kinds it
  takes:**
  - **a token,** which lasts until revoked: Forgejo's, or GitHub's
    fine-grained token;
  - **an App** (GitHub's: `Credential::App { id, key }`): the App's id,
    and its private key, an RSA key of 2048 bits as PKCS#1 DER. The
    component signs a JSON Web Token with it (RS256, through `ring` as
    `skein_tls::sign_rs256` has it), mints from that each installation's
    token, which expires within the hour, and chooses the installation
    by a repository's owner (connection.md, section 5). The
    dialect gives the token's bytes and the mint (github.md,
    section 8); GitHub's tokens render as `Authorization: Bearer <t>`.
- **The owner gives the value** at start, for each forge (section 2.2).
  The component reads no file.
- **The dialect says how a credential renders on a request.** Forgejo's
  is `Authorization: token <t>` (`forgejo.md`, 1.1). The request's head, that field
  among them, fits skein-http's `request` limit: the component bounds
  the credential's size at start (connection.md, section 7).
- **For a renewed kind,** the dialect also gives the plan that renews
  the credential, and how long before expiry it is due. The component
  runs it (connection.md, section 5).
- **A credential's value reaches no record, request line, fact or
  hint.** It is not logged, granted or stored.
- **A credential that is expired or revoked** reads as `Forbidden` on
  every call (section 10). An App's key that GitHub refuses (a `401` on
  the JSON Web Token) is the same.

## 8. GitHub

GitHub's dialect is `skein_forge::github`, a module of `skein-forge`
beside `forgejo`, at the pin `X-GitHub-Api-Version: 2026-03-10`.
github.md holds its documents, plans, failures, deliveries, the
App's documents and the facts it relies on; ../testing/fakes/github.md, sections 1
and 2, its fake layer and its conformance. Here is what it adds to the
kit.

- **A provider is documents plus call plans.** Nothing above the
  Forgejo dialect and its plans assumes Forgejo. GitHub's calls are REST
  at the pin and three GraphQL documents (the viewer, the queue's
  enqueue, a pull's merge commit), each call's cost counted by the
  component as Forgejo's is (4.2).
- **The vocabulary gains:**
  - validators on reads and their not-modified answer, free of the rate
    (3.3);
  - the rate's window, `Rate`, which the budget takes in place of its
    configured one (3.3);
  - pauses, as `RateLimited { after }` (3.4);
  - `Queued`, a merge put in a merge queue (3.2, 3.3), and `Pull`'s
    `queued`;
  - the protection's queue, signatures, linear history, code owners and
    `Bypass` (3.6);
  - `NotConnected` and `NotInstalled`, the component's (3.4);
  - the App's credential, `Credential::App` (section 7).

  Each addition is optional for a dialect: Forgejo's carries no rate and
  no validator, answers no queue and `Bypass::Never`, and never
  `Queued`. Reads batched into one query are not added.
- **Credentials:** the App (section 7); git over HTTPS with an
  installation's token narrowed to contents.
- **Deliveries:** `X-Hub-Signature-256`, `X-GitHub-Event` and
  `X-GitHub-Delivery`, one URL for the App, each naming its
  installation. A pushed delivery and one read from the App's delivery
  log go through one decoder (github.md, section 9).
- **CI:** check runs and Actions' jobs, whose logs come as a redirect to
  a download host the component reaches (github.md, section 10).
- **Setup:** the App manifest flow (setup/README.md). There is no owned
  mode.
- **The fake:** a GitHub protocol layer over the same domain
  (../testing/fakes/github.md, section 1).

## 9. Verification

The dialect's step tests, in `skein-forge`, with no io:

- **Documents.** Each decoder and encoder is tested:
  - against the conformance's transcripts;
  - against answers generated from the seed;
  - against mutations of both.
- **Encoders.** Golden bytes for each encoder; bounds at the limit and
  one past.
- **Decoders.** A fuzz target per decoder; the bounds at the limit and
  one past (`string_bytes`, `body_bytes`, `inner`, the stack's depth); a
  string cut at every byte; the server's encoders met by the client's
  decoders, and the client's encoders by the server's decoders.
- **Plans.** Each plan against canned answers, for every call of `forgejo.md`, 1.2:
  - the page size, the smallest of three (`forgejo.md`, 1.1);
  - names percent-encoded, a slash in a branch's name `%2F`;
  - times read and written, 2554 included;
  - each failure of `forgejo.md`, 1.4, in the status and the message that tell it;
  - the first failure ending a call;
  - a request that was never seen and one that may have been, for a
    write and for `OpenPull`'s reads;
  - the tables filled from the answers heard, and a row dropped.
- **The marker.** Encoded and decoded with several words, a word of 1
  and of 16 bytes, a body whose marker carries another word, an odd
  count of digits, an upper-case digit, a missing line feed, and a key
  at the limit of `op_bytes`.
- **Deliveries.** Each event of section 6 decoded to its hint; a
  signature right, wrong and missing; a branch's deletion; a tag; a job
  that ends; a repository id past 32 bits.
- **Transcripts** are format-faithful and content-irrelevant: volatile
  fields (times, ids, hashes) are masked, and no credential or secret
  reaches a file (../testing/fakes/forge.md, section 7).

The component's machine world, the protocol world, the client's world,
the setup's world and the conformance are in connection.md,
client.md, setup/README.md and ../testing/fakes/forge.md. The worlds are under
`tests/forge/`, one crate each (`machine`, `protocol`, `client`,
`setup`); the conformance, `tests/forge/conformance`, is a tool outside
the suites (../testing/README.md, section 6).

## 10. Open questions

- **A real Forgejo's refused reads, unexplained** (`README.md`, 3.4, `forgejo.md`, 1.3): on a deployment,
  an agent's `PullFor` was refused `InvalidAnswer`. The answers captured there
  show a comparison's `removed` (`forgejo.md`, 1.3), and no cause for that: none is a by-branch
  answer, and none was read with CI red. What would settle it is that answer,
  with its request path and the head and base asked, and a section's reads at a
  red head. A comparison whose `commits` come shorter than its `total_commits`
  is refused the same way (`skein-forge/src/forgejo/plans/commits.rs`); whether
  Forgejo answers one on a long range is unknown. Default: as they stand, until
  an answer shows a fault.
- **Dismissed reviews:** whether one counts as no verdict, or as the
  comment it was. Default: the comment it was.
- **Contents on demand.** A pull request's files with `sides` cost two
  reads per file, since `File` carries contents. Default: a reader that
  needs paths alone, as an overlap, asks without `sides` (3.2); one that
  needs contents asks them for the whole page, or reads one path by
  `File` at each commit; a comparison reads names alone (3.2).
- **A file too large for the answer** fails its call as `TooLarge`.
  Default: so; a side cut with a mark would need `File` to carry one.
- **Bodies past `body_bytes`** are carried empty. Default: so, the
  component counting them; a skein-json read that keeps a string's first
  bytes and digests the rest would let it keep their heads and keys.
- **An edit within a creation's second** reads as `Original`, so a
  person who edits the owner's keyed comment within that second is not
  heard. Default: `Original` when the times are equal.
- **The merge style:** the repository's `default_merge_style`. A style
  an owner names per repository would need `Write::Merge` to carry it.
  Default: the repository's.
- **An update's real time** (`forgejo.md`, 1.4): aborting its call does not stop the
  forge's git work. How often that work outlasts the client's `lifetime`,
  and what another copy costs, await measurement (client.md, 11).
  Default: the lifetime stays, and recovery uses the idempotent class.
- **An expired or revoked credential** reads as `Forbidden` on every
  call until the next start, which it stops (connection.md,
  sections 3 and 5). Default: so; the owner could tell it apart, so a
  person is told once.
- **What the dialects still write apart.** Their field walkers, over
  different masks and `Want`s: 4.4 takes what depends on no forge's
  documents, and joining the walkers is a redesign. And
  `skein-forge-setup`'s `host/limits.rs` builds the one `Limits` in two
  functions, `documents` and `github`. Default: the walkers stay apart;
  the setup's two functions can become one, which nothing asks for yet.
- **The conformance on macOS.** A bottle is the pin's version and another
  build (4.3), so what the conformance checks is Linux's binaries, and a
  bottle answers only to temper's one story
  (`temper/docs/design/testing/README.md`, 2.8). Default: so, until the
  conformance joins the macOS workflow with skein's suites
  (`temper/docs/design/draft/macos.md`, section 6).
- **GitHub's further gaps.** Settled: its dialect is a module of
  `skein-forge`, and section 8 lists what it adds. Open:
  github.md, section 15.
- **What the call plans' states leave** (4.2): a plan's cells are arms of one
  `match`, where ../../foundation/programming-model.md, section 5.4 has one handler each.
  Default: as they stand, until a debt item names them. The `if let`s
  `forgejo/client` and `forgejo/server` keep on their own enums go with `D015`
  of `skein/docs/development/debt/`.
- **A document kind no plan names** (4.2): the exchange's decoder picks the kind,
  so a plan's match over a decoded document ends in `_` under a named `#[expect]`,
  where ../../foundation/programming-model.md, section 10.3 has no `_` on an own enum, and a new
  `d::Document` kind goes silent there. Decided under the user's standing word,
  for the user's later review (`README.status.md`). Default: so.
