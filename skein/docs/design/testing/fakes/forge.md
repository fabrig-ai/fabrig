# The fake forge

Provisional, 2026-10-20. The kit's fakes of a forge: a domain that plays a
forge's state and git, a protocol layer that serves Forgejo's API over it,
GitHub's layer that serves github.com's (`github.md`, section 1),
the forge peer that hosts it on loopback io, the throwaway Forgejo that runs
the real one, and the conformance that holds the dialect to what it does.
The dialect is ../../forge/README.md's; the parts under test are the kit's others.

## 1. In one page

- **The domain** (`crates/skein-fake-forge-domain`, on skein-lib alone) is
  a step crate with a vocabulary of its own. It is Forgejo-shaped, and no
  better (section 2).
- **Its git** lives in the same store, so a branch pushed is the head the API
  reads. It is reached through a typed transport: a shared handle within a
  process, and a private git route from another one.
  Git's own protocol is never served (section 3).
- **The protocol layer** (`crates/skein-fake-forge-forgejo`) serves
  Forgejo's API over the domain on skein's HTTP server, with its quirks, the
  setup's routes, signed deliveries and byte faults (`forgejo.md`, section 1).
- **The forge peer** (in `crates/skein-fake-peers`, beside the LLM and OAuth
  peers) hosts the protocol layer on loopback, in plaintext, over a forge its
  owner hands it and may share with another fake in the process (section 5).
- **The throwaway Forgejo** (`testing/skein-throwaway-forgejo`) runs the
  pinned binary on loopback, on SQLite in a scratch directory (`throwaway.md`, section 1).
- **The conformance** (`tests/forge/conformance`, outside the suites) drives
  the throwaway and records each fact the dialect relies on as `holds`,
  `fails` or `unchecked`, with masked transcripts (`throwaway.md`, section 2).
- **GitHub's layer** (`crates/skein-fake-forge-github`) serves github.com's
  API at the dialect's pin over the same domain: the App, its installations
  and tokens, the browser's pages as plain redirects, the merge queue,
  rulesets, the delivery log and the job log's blob host. Where it differs
  from github.com is written down as contracts (`github.md`, section 1).
- **What the two layers share** (`crates/skein-fake-forge-http`): one
  connection's core and the private git route, on neutral types, so
  neither layer stands on the other (`github.md`, section 3).
- **GitHub's conformance** (`tests/forge/github`) runs the same cases on
  that layer, in the focused suite, and, opt-in and from the owner alone,
  on github.com with the probe App's fixtures (`github.md`, section 2).
- **No serde.** The throwaway and the conformance use skein's HTTP client,
  driven by `skein-shell`'s `drive` as a one-shot host, and skein-json.
- **The service's own:** its fake of its git component, its worlds, stories,
  cost ceilings and git transcripts, and an owned Forgejo's preparation
  (../../forge/setup/README.md, section 6).

## 2. The domain

It knows no HTTP, no JSON and no dialect. Its `Read`, `Write` and `Git` are
its own types, close to ../../forge/README.md, section 3; the layers above translate.

### 2.1 The boundary

- **In:** `Event::Call { reply_to, user, repository, op }`: a user number,
  never a credential, and an `Op` of `Read`, `Write` or `Git`. **Out:**
  `Request::Reply`, exactly one per call; `Request::Hook`, best effort, with
  no terminal event.
- **Entry points:** `step` takes an event, `fire` the clock, the only way
  time moves. Each emits at most one request; `next_deadline` says
  when to fire.
- **A call is decided as it arrives:** refused for its rate, failed before
  it is made, made and then timed out, or made and answered after a drawn
  latency. One beyond `calls` is answered at once as unavailable; a change
  beyond the `hooks` in flight sends no hook.

### 2.2 What it holds and refuses

- **It holds** repositories with their users' permissions, labels, issues
  and pull requests on one numbering, comments, reviews, CI statuses,
  Actions' runs and jobs, merges, a wiki and webhooks. Twenty-one limits
  bound it: counts of each kind, bytes of a name, title, body and file, the
  page size, and the calls, hooks and observations held. Over one, a call is
  refused as too large (bytes) or full (counts).
- **It refuses what Forgejo refuses:** too little permission, an undefined
  label, an empty title, a stale head on merge, a merge its base's
  protection does not allow. It keeps what Forgejo keeps and no more.
- **Listings are Forgejo's:** items updated at or after a time, at the
  forge's resolution, least recently updated first, paged by number. A page
  is read from the order as it stands when its call arrives, so an item
  that changes between two pages can shift another onto a page already read.
- **An item's updated time** moves for a comment, a review, a label, a
  title or body, a push to its head, closing, reopening and merging, not for
  a status on its head or a comment edited or deleted (`status_updates` and
  `edit_updates` make either move it). A pending review is hidden until
  submitted. Stale approvals are dismissed only where the protection says so
  (`dismiss_stale`). Anyone with write permission may edit or delete
  another's comment.
- **Merges** are squashes at an exact head: one commit on the base's tip,
  three-way from the newest commit head and base share, conflicting where
  both changed a path differently. A repository has one protected branch,
  `Protection { branch, contexts, approvals, dismiss_stale }`: a merge needs
  the statuses and approvals asked; pushes, creation and deletion are
  refused. Deleting a branch closes the pull requests it headed or based.
- **A pull request's size** is the fake's own count, as a forge's is
  (`../../forge/README.md`, 3.6): at its head, `files` the paths whose
  contents differ from the merge base's, those its `PullFiles` lists;
  for each, its head side's lines that its base side's don't match,
  counted as additions, and its base side's that its head side's don't
  match, as deletions, lines matched as a multiset, a missing side
  having none. Both protocol layers write them on a pull request's
  document as `changed_files`, `additions` and `deletions`, as the
  forges do.
- **A close refused by policy** (temper's `draft/failure-worlds.md`,
  4.1): `scenario::refuse_close(domain, repository, number, refused)`
  makes every `Write::Close` of that pull request answer `Protected`,
  closing nothing, until a world sets it back. Deleting its branch
  still closes it. An image keeps none.

### 2.3 Timing, faults and CI

Chances are per mille, in `Config`, and drawn from the domain's seed.

- **Latency:** `latency_min..=latency_max`, or with the `late` chance
  `late_min..=late_max`. `unavailable` fails a call having done nothing;
  `timeouts` makes it, then fails it; `landing` fails a write or a git call
  as timed out and makes it `land_min..=land_max` after the answer.
- **Rate:** a user may make `rate_limit` calls in a window of `rate_window`,
  which starts with their first call after the last one ended; zero is no
  limit. A call beyond is `limited`; one that finds every window kept
  running is `crowded`.
- **Clock:** the forge keeps its own, at `resolution` seconds, skewed from
  the world's by `Skew::{None, Ahead, Behind}`; every time it shows is on it.
  Hooks come after `hook_min..=hook_max`, late with the `hooks_late` chance
  (`late_min..=late_max`), never with `hooks_lost`.
- **A commit that becomes a head** gets a pending status per context of its
  repository (`Checks { contexts, latency_min, latency_max, silent, passes,
  reruns, cue }`), then a verdict after a drawn latency: passed with the
  `passes` chance, failed otherwise, never reported with the `silent`
  chance. CI runs once per commit and repository. `Cue::File { path, green }`
  replaces the chance: a commit whose file at `path` holds `green` passes,
  any other fails. `Cue::Message { fail }` instead fails a commit whose
  message contains `fail`, and passes any other commit. With no cue the
  configured chance decides; no cue changes latency, silence or reruns.
- **A context's own cue:** `Checks.cues`, at most one `ContextCue {
  context, cue }` per configured context, each naming one, overrides the
  repository's `cue` for that context; a context without one inherits
  `cue`, then the chance. So a world fails one check of a head and
  passes the others, as a person's gate beside a starter check does.
- **`Cue::PullFile { path, no_pull, step, log }`** judges a head by its
  pull request: it passes when the head's tree holds `path` with `{pull}`
  replaced by the pull request's number, by presence, never content, and
  fails otherwise; a head that is no open pull request's reports
  `no_pull`, a settled verdict (`Skipped`, typically), never pending.
  A commit's pull request is the lowest-numbered open one whose head it
  is. A failure names `step` as its job's failed step and ends the job's
  log with `log`, `{pull}` replaced too, read through the ordinary jobs,
  steps and logs (`forgejo.md`, section 1); every other context, and a `File` or `Message`
  cue, keeps the generic step and text. Templates are bounded by their
  expansion at a full 64-bit number (20 digits per `{pull}`).
- **`Cue::Declared { path }`** judges a head by what its own tree
  declares at `path`, as a repository's workflow file says what its
  checks do, so that whoever commits decides a head's verdict by what it
  holds (temper's `draft/landed-qa.md`, 6): with no file at `path`, the
  context passes; else the file's first line, its trailing newline
  aside, is the rule: `pass` passes; `fail` fails; `pull-file TEMPLATE`
  judges as `Cue::PullFile { path: TEMPLATE, no_pull: Skipped, step:
  CONTEXT, log: Missing TEMPLATE }`, CONTEXT the context's name; any
  other line, or a file past the content limit, fails, the job's log
  ending `unreadable PATH`. The declaration is read from the commit
  judged, at its verdict.
- **The pull binding** is the one exception to CI once per commit: a
  commit judged by a pull-file cue, or by a declared one, runs again
  when it becomes a pull request's head after it was pushed, binding
  that number, as a forge's `pull_request` event runs on a commit its
  `push` already did. The commit keeps its latest run alone.
- **`Skipped`,** a check's settled state beside passed and failed, counts
  as a pass in a combined status and as neither pending nor failed in a
  head's verdict; the protocol layer serves it as `skipped` and reads a
  forge's `skipped` as `Skipped`, never as passed.
- **A rerun** happens once per context, with the `reruns` chance: pending
  again, then a new verdict, so a head's CI can move after it settled.
  People with write permission report statuses of their own.
- **Actions' identities** follow the statuses kept: one `Run` per commit and
  repository, numbered from 1, and one `Job` per context, at attempt 1 or,
  after a rerun, 2. A commit whose statuses are forgotten loses its run.
- **Dispatch-only workflows:** a repository may declare workflows
  (`Workflow { file, jobs, latency_min, latency_max, silent, passes,
  cue }`) that run only when dispatched (`Write::Dispatch`), never on a
  push. Each dispatch makes a run of its own at the branch's head as the
  dispatch finds it, numbered after the repository's other runs, its
  inputs kept, and a job per declared job, writing no commit status, as
  Forgejo's dispatch-only jobs write none (`../../forge/forgejo.md`,
  2); then each job's verdict
  after a drawn latency, by the chance or the cue as for `Checks`, and
  the run completes once its last job does. `Runs` lists them, `Run`
  reads one with its jobs, and `Rerun` runs a completed run's failed job
  again, at its next attempt. A fault draws a job failed in its set-up
  step (`setup_faults`), or a run lost with no step failed (`lost`), so a
  world reaches the runner's faults; `Cue::Message` fails a job's
  workflow step, as a test would. A run in flight cancelled by a user
  ends each job not completed cancelled, with no step failed, and
  completes `Cancelled`, naming who cancelled it, so a world reaches a
  run cancelled by someone other than its dispatcher; a completed run
  is not cancelled.
- **A job's state** (`../../forge/job-state.md`, section 5): a job names
  the runner labels it asks for and the jobs it needs, and the world the
  labels its runners serve, all by default. A job asking for a label no
  runner serves waits, with no steps, its status pending; one whose
  needs have not ended is blocked; one running is at a step, so a
  `JobState` read names each.
- **A declared wait for a runner** (temper's `draft/ci-runner-wait.md`,
  1 and 2): a declaration's first line (`Cue::Declared`, above) may
  open with phases before its rule, one space apart: `wait SPAN`, then
  `run SPAN`, each optional, in that order, as in `wait 40m run 5m
  pass` or `run 90s fail`. A SPAN is one to five decimal digits and a
  unit, `s`, `m` or `h`, at most 24 hours; phases that do not read so
  make the line any other line, failing `unreadable PATH`. A job such
  a line judges, from its run's start or its rerun's:
  - **waits for a runner** for the `wait` span: no steps, its status
    pending, its `wait` `Runner`, as a job asking for a label no runner
    serves;
  - **then runs** for the `run` span, or for the drawn latency where
    the line names none: at its workflow step, `Set up job` passed and
    the workflow step running;
  - **then reports** the rule's verdict, judged as the rule alone is.

  A line with no phase is judged as before, its draws unchanged. The
  phases count on the forge's clock from the run's start, so a new run
  (a pull binding's included) or a rerun's attempt waits afresh. A
  dispatched workflow's job judged by a declaration (`Workflow.cue`)
  waits its span from its dispatch, beside its labels and needs, then
  runs its span in place of the drawn latency. A phase's end is a
  timer, kept by an image as all timed work is (2.7). So whoever
  commits a head decides how long its run waits for a runner and runs,
  as a shared runner's queue makes it on Forgejo.
- **A run's wait,** `Run.wait`, an `Option<JobWait>` as a job's `wait`
  is, follows its jobs as Forgejo words a run: none once a job runs,
  and once the run completes; else `Runner` while a job waits for a
  runner; else `Needs` while a job is blocked. It is derived from the
  jobs whenever they change, an image's decoding included, and never
  kept.
- **One runner for every job:** `scenario::runners(domain, Some(n))`,
  none by default, lets at most `n` dispatched jobs of the whole forge
  run at once, on runners numbered from 1. A job ready to run while
  every runner is taken waits, its `wait` `Runner`, in the order it
  became ready, and runs its span from taking one; it holds its runner
  until it completes, is lost or is cancelled. Commit statuses' CI is
  outside it. An image keeps none: a restored waiting job is runnable.
- **Lagging views** (temper's `draft/failure-worlds.md`, 4.1):
  `scenario::lag(domain, Lag { chance, min, max })`, `chance` per
  mille, none by default. When a write moves a branch, each open pull
  request it heads or bases shows, with the chance, its former head or
  base, and its former mergeability, to every read and listing for a
  span drawn from `min..=max`, while the branch reads its new tip. A
  merge (`Write::Merge`) is, with the chance, recorded that long after
  it is made: its caller is answered, its base moves and its hook
  goes, but reads show the pull request open as it was until then, so
  a later write shows first. A view ends at its time, with no timer;
  an image keeps none.
- **Merges and updates by style:** a fast-forward-only merge moves the
  base to the head and refuses one that does not contain the base's tip
  (`Conflict`); an update by rebase replays the branch's commits onto the
  base's tip and refuses a conflict, as a merge update does.
- **A merge recorded at a commit** (`Merge` with `how` `Recorded {
  commit }`) moves nothing: an open pull request whose base contains
  `commit` reads merged at it, its head and branch as they were; one
  whose base lacks it, or a repository whose settings allow no manual
  merge, is refused (`Refused`). A record repeated at the same commit on
  a pull request it merged answers `Merged` again.
- **Manual merges,** two settings of a repository, each off unless a
  world sets it, as Forgejo's: `allow_manual_merge` admits a record;
  `autodetect_manual_merge` marks an open pull request merged once its
  base branch, moved by any write, contains its head, merged at the
  first merge commit on the ancestry path from its head to the base's
  tip, or at its head where there is none (a fast-forward), as Forgejo's
  detection reads it. Nothing else of the pull request moves.
  Both settings survive an image. Images append a tagged settings table
  when either is enabled; an image without the table keeps both off.
- **Trees:** every commit's tree is read through `Read::Tree`, its
  entries in name order, paged.
- **A run's report:** a workflow may name failing tests by id (`fails:
  [id]`, or `Cue::Tests` reading them from a file at the commit); each
  completed run uploads an artifact of one file, JUnit or one id per
  line as the workflow declares, naming the ids it failed; a run given
  the workflow's selection input (`select`, its name as the workflow
  declares it) runs only the ids it lists. A workflow may also name flaky ids
  (`flaky: [id]`, bounded as `fails`, empty by default): each fails in
  a run not given the selection input and passes in a run whose
  selection lists it, stateless, so that a world shows a flake's rerun
  passing. It may name ids that fail once (`once: [id]`, bounded as
  `fails`, empty by default): each fails at the first attempt of the
  workflow's first run over a commit's tree, and passes in a job's rerun
  and in every later run over that tree, read from the runs it keeps, so
  that a world shows a gate run once more at the same content passing.
  `Read::Artifact` reads it, and
  `Read::Commits` a range's commits with their messages.

### 2.4 Seen from outside

- **Every change is observed,** content and all, and so is every write the
  forge refused, in a bounded queue a referee drains (`pop_observation`). A
  full queue drops and counts (`observations_lost`).
- **Written out, each is a JSON line,** for a referee in another process,
  such as temper's fake forge's `--observations` file (temper's
  `protocol/setup/playground.md`). The protocol layer's `observations` holds the
  encoding, since the domain has no JSON: `encode` writes a `Recorded`, an
  observation or a checkpoint, as one object and a newline, and
  `decode` reads one back exactly. For example:

      {"observation":"moved","repository":"temper/widget","branch":"landing","from":3,"to":4,"by":2,"forward":true}
      {"observations_lost":0}

  - **Lines come in drain order,** the order `pop_observation` gave them,
    which is the order the forge made them.
  - **The object's first member** is `observation`, the variant's name in
    lower case, as `moved` or `wiki`, and the rest are its fields by their
    names, in their order.
  - **A moved line also says `forward`,** which the writer computes as it
    drains, from `Domain::is_ancestor`: whether `to` descends from `from`,
    or, for a branch made again (`from` null), from the `at` of that
    branch's last deleted line; `null` if there is neither. The writer
    keeps each deleted branch's `at` until the branch is made again, as
    many as the domain's `Limits::branches` per repository, forgetting the
    oldest past it (a branch made again whose deletion it forgot has
    `null`). It is the only member not in the observation itself, since a
    reader in another process has no domain to ask.
  - **A checkpoint,** `{"observations_lost":N}`, is the count lost so
    far, 0 included: cumulative, one at each flush. A writer ends each
    flush with one, and flushes in each pass that observed or lost
    anything and once at its stop: a file is whole up to its last
    checkpoint, and what follows it is a flush still being written. A
    stopped writer's last line is a checkpoint, so a stream that does not
    end with one is truncated, and a judge reading it rules Undecided.
    temper's writer sends nothing of a pass before its flush is written
    (temper's `protocol/setup/playground.md`), so a reader that has seen a
    reply, or what a reply caused, finds what it observed.
  - **A number,** a commit, a user, an item, an id, a revision or a
    position, is a JSON integer, exact to `u64`. A flag, a group's
    `ended` or `landed`, is `true` or `false`. An absent value is
    `null`, a list is an array, and an opened pull request's `branches`
    an object of its fields.
  - **A name,** a repository's, a branch's, a label's, a context's or a
    page's, is a string. Content, a title, a body or a page's content, is
    base64, so its bytes cross whole, as on the git route (`forgejo.md`, section 1). A
    name that is not UTF-8 has no line: `encode` answers none, and the
    writer counts it with the lost.
  - **An enum** is its variant's name in snake case: a `kind`, a
    `verdict`, a `state`, and a refusal's `what`, its operation
    (`create_issue` to `create`). A refusal's `error` is an object: its
    `name`, the error's in snake case, then its fields: `reset`, in
    nanoseconds as the domain's `Time` holds it, for `rate_limited`; `what` for
    `missing`; and `number`, `head` and `position` for `already_queued`.
  - **A reader** takes the members in any order and refuses an unknown,
    a repeated or a missing one, naming it. So a reader older than its
    writer fails by name rather than misreading.
- **A world acts from outside the API** with free functions (grants,
  commits, branch moves, CI outcomes, reachability) and, at settle, reads the
  store (`inspect`), the `tally` of what the forge did and refused, and the
  `room` left, so a forge that filled up is told from one that works. Needing
  room, it forgets the statuses of the oldest commit no head shows, never a
  commit.
- **What fell** (`../sweeps.md`, 3): each fault the domain injects is
  kept as it falls, a `Fell { at, until, fault, repository, user, call,
  reply_to, pull, commit, run, runner }`, a field `None` where the
  fault strikes none, in a queue a world drains (`pop_fell`), bounded
  as observations are (`limits.observations`), a full one counting
  what it drops (`fell_lost`). `until` is when a fault that lasts
  stops: a late answer's or a landing's moment, a view's end, a wait's,
  a wait being kept once it ends; `call` is `Called::{Read, Write, Merge, Git}`; `reply_to` is the
  call's `Token`, never its `ReplyTo`, so a record copies and the right
  to answer stays the domain's alone; `run` is a run's id.
  An image keeps none. `Fault::family` and `Fault::name` are the words
  a ledger keeps:

  | `Fault` | family | name | falls when |
  |---|---|---|---|
  | `Busy` | `busy` | `no_room` | a call finds no room (`limits.calls`) |
  | `Unavailable`, `TimedOut` | `busy` | `unavailable`, `timed_out` | those chances |
  | `Landing`, `Late` | `busy` | `landing`, `late` | made, or answered, at `until` |
  | `Limited`, `Crowded` | `busy` | `limited`, `crowded` | the rate |
  | `HookLate`, `HookLost`, `HookDropped` | `hooks` | `late`, `lost`, `dropped` | a hook |
  | `Silent`, `Rerun` | `ci` | `silent`, `rerun` | a context or job never reports; `reruns` |
  | `SetUp`, `Lost` | `ci` | `set_up`, `lost` | `setup_faults`, `lost` |
  | `RunnerWait` | `ci` | `runner_wait` | a job waits for a runner (2.3) |
  | `HeadBehind`, `BaseBehind`, `MergeLate` | `lagging` | `head_behind`, `base_behind`, `merge_late` | a lagging view (2.3) |
  | `CloseRefused` | `refusing` | `close_refused` | a refused close (2.2) |

### 2.5 The promises' judge

The domain's `promises` module judges a client's writes over the decoded
stream of section 2.4. Its `Profile` supplies the client's user id, key
markers, owned branch prefixes, allowances on foreign items and table
bounds. `Recorded` supplies observations with the encoder's `forward`
answer and cumulative loss checkpoints. `Decision` supplies the exact
repository, pull number and head the client considered; `decided()` says
all decisions have arrived, even when there are none.

- **Once:** two client creations with the same marked key, per repository
  and kind (issue, pull or comment). Only the first opening marker and its
  next closing marker count.
- **Forced:** a client move that does not descend, including a remake
  after deletion, or a rejected push. An existing branch moved without a
  `forward` answer leaves the judgement undecided.
- **Foreign:** writes to branches, items or comments the client did not
  make in the stream, except owned branch prefixes and the profile's
  explicit item allowances. A body edit over another user's last edit
  breaches this promise even on an owned object. Label allowances apply
  to added and removed labels against the last set seen; an unseen
  item's first `Labelled` is a baseline and judges nothing. The forge's
  merge base moves and queue group branch creations and deletions are
  recognized by their observation order and commit, and exempted.
- **Order:** a client merge or enqueue must have a decision at that exact
  head. A queue merge at the enqueued head shares its obligation.
  Decisions can arrive before or after the observation.

`Defined`, `Reported`, `Wiki` and `Refused` are judged by no promise.
Groups identify the queue's writes; dequeues introduce no obligation.

The judge allocates repository slots and each repository's item, comment,
branch, key, label and evidence tables at `bounds.objects`, and the
shared decision table at `bounds.decisions`. Last editors live with
items and comments; labels, including removed labels, are remembered per
item. Any table overflow is sticky and leaves the judgement undecided.

`verdict()` yields `Pass`, `Fail` with breaches, or `Undecided` with its
reason and the breaches still visible. Reasons take precedence in this
order: the latest lost count when nonzero, no final checkpoint, a full table,
a merge or enqueue before `decided()`, and a move lacking `forward`.
Breaches are deduplicated by promise and subject, in first-seen order.
Their `line()` is `promise/<promise>/<subject>`: names and keys are
percent-encoded byte by byte, numbers decimal. The judge does no parsing,
printing or network work.

### 2.6 A copy, and deleting

What Forgejo's migration fetches from a public host, the fake finds in
a catalogue its protocol layer holds (`forgejo.md`, section 1): the domain copies one
of its repositories into another, and reaches no network.

- **`Write::Copy { owner, name, fill }`,** called on the source: the
  source is a repository of the domain's, seeded as any, and the copy's
  owner and name go with the call, so that the call has a repository
  and takes the domain's faults as any write does. Its `fill` is
  `Whole`, `Slow(duration)` or `Cut`.
- **From its arrival the copy is there and empty,** private, owned by
  `owner`, no branch, as a Forgejo migration under way reads. A name
  taken is `Exists`, and nothing is made.
- **At the call's end it is filled:** every branch of the source and
  its default branch, its commits the source's own, sharing the store
  (section 3), after the call's drawn latency, and `duration` more for
  `Slow`. So a copy is the one write that is not done when it arrives
  (2.1), a call state beside a late landing's.
- **`Cut`,** or a fault of the call's (2.3): the call fails as timed out
  and the copy stays there and empty.
- **Not a mirror:** nothing moves between a source and its copy after
  the call, and the source hears nothing of it: no head moves, no hook
  and no observation names it.
- **`Write::DeleteRepository`** frees the repository's name. Its slot
  stays, marked gone, so that what holds its id reads nothing; nothing
  walks a gone repository. A copy's call still running on it ends as
  the conformance records of Forgejo (`throwaway.md`, section 2).

### 2.7 Its state, kept

An owner that runs the fake as a product across restarts keeps the
forge's state in a file of its own (temper's box, whose copies hold it:
temper's `draft/landed-qa.md`, 6, and `protocol/setup/box.md`, 5). The
domain gives an image of its state and is built again from one.

- **`Domain::image(&self) -> Result<Box<[u8]>, image::Refusal>`,**
  refused with `Refusal::Busy` while a call awaits its reply, so that an
  owner takes it once its peer has shut down (section 5). Its bytes are
  the domain's own format, length-prefixed binary written and read by
  the domain alone, with no JSON, since the crate stands on skein-lib
  alone. Before v0.1.0 it changes in place, as every stored format
  does: an image a newer build cannot read is refused by name.
- **`Domain::from_image(config: &Config, bytes: &[u8]) -> Result<Domain,
  image::Refusal>`:** the domain the image held, under `config`'s
  limits and chances. An image that does not decode, that refers to
  what it does not hold, or that holds past `config`'s limits, is
  refused, naming what (`Refusal::Malformed`, `Refusal::Limits`), and
  nothing is built.
- **What it keeps:** everything that lasts: every repository, gone
  slots among them, with its permissions, labels, issues and pull
  requests on their numbering, comments, reviews, statuses, Actions'
  workflows, runs and jobs, merges, protection, queue, wiki and hooks'
  settings; the store's commits and the counters that number commits,
  comments, reviews, runs and jobs, so that a restored domain's next
  commit id is the one it would have made (section 3); the clock's last
  reading; and its `Rng`'s state, so that its draws go on as they
  would have.
- **Its timed work** (an undecided CI run, a rerun, a dispatched
  workflow, a queue's timer, a late landing, an unfinished copy) is
  kept with the time it had left, and armed again on the restored
  clock. **What is in flight is dropped:** a hook not yet released, as
  a delivery that cannot be written is (section 5); the rate windows,
  which start again empty; the observations not yet drained.
- **Its clock goes on:** the restored domain's clock reads at least the
  image's last reading at its first pass, so that no item's time goes
  back across a restart.
- **Restored, it answers as it would have:** every read after
  `from_image` answers as the same read before `image` did, and a write
  then is decided as it would have been, its faults drawn from the same
  stream.

## 3. Its git, and the remote boundary

What a fake of a service's git component may rely on. That fake is the
service's own (temper's is the first); the domain is the remote it acts
against.

- **One store.** Commits are named by a count the store keeps, from 1, so a
  seed replays the names. A commit is `Object { parent, merge_parent, tree,
  message }`: a first parent, an optional second, a tree of paths to
  contents, and a message of opaque bytes, at most `title_bytes + body_bytes
  + 2`. Squash merges and `advance` carry an empty message.
- **The calls** are `Op::Git`. `Clone` (every branch, and the default) and
  `Fetch { Branch | Commit | Default }` (the commit and those before it, by
  both parents) need read permission. `Push { branch, commit, expected }`
  `Create { branch, commit }` and `PushNew { branch, commit }` need write;
  a protected branch takes none of them.
- **A push** moves a branch by a fast-forward, or creates it; any other move
  is `Rejected`. A given `expected` must be the branch's head, or the push
  is rejected. `Create` requires the repository already to hold the commit
  and answers `Created` or `Exists`. `PushNew` uploads the store commit and
  its ancestry, as `Push` does, only under an empty branch lease: it creates
  an absent branch, or answers the existing head without moving it or
  transferring objects. The check and creation are one domain call.
- **Every move of a branch** is observed, starts CI on the new head, and
  moves the head of each open pull request that follows it. `advance` is
  the world's other party moving a branch under a run.
- **The scenario's commits:** `commit` returns `None` for an unchanged tree;
  `merge_commit` and `commit_tree` store one anyway. A parent's id is less
  than its child's, as `git::Remote` asks (checkout.md, section 4).
- **Faults, time and identity.** A git call meets section 2.3's latency, rate
  and faults; a fake that needs an instant answer sets the latencies to
  zero. The caller is a user number, mapped from its credential by the
  protocol layer's tables (`forgejo.md`, section 1), so a token for the API and one for git
  stand for the same user.
- **Within a process,** a working tree reaches the store through
  `git::Remote`, answered on the shared forge (section 5) without the
  network.
- **From another process,** the same calls travel the private git route
  (`forgejo.md`, section 1) on the forge peer's listener, with the
  store's reads and commits beside them. So a fake of the service's git
  can run as a program of its own, as temper's `temper fake git` does
  beside a demo's forge. The domain stays the only store: such a program
  holds nothing the store has not answered, and keeps no object past its
  command.
- **No git over HTTP.** Git's own protocol, its smart HTTP and its packs,
  is never served; commands, arguments and their outputs are the
  service's. A bridge for real git is designed separately, and lands
  here.

## 4. The protocol layer

Moved to [`forgejo.md`](forgejo.md), section 1.

## 5. The forge peer

`skein_fake_peers::forge::Peer` hosts the protocol layer as the LLM and OAuth
peers host theirs. The real loop hosts the same one.

- **Its shape is the other peers'.** `new(address, limits, ..)` listens on a
  loopback address, port 0 for the kernel's, and refuses a non-loopback
  address or unusable limits (`Error::Limits`) before any effect.
  `address()` is the bound address, known after io's listening event.
  `observations()` holds frozen records under the world's caps. `shutdown()`
  closes admission, then each connection once its client has settled. It
  implements `Host` (../../runtime/shell.md, section 11). Edge limits bound the
  connections, decoded inputs and observations. By default it records connections, request heads, calls
  with their terminals, and deliveries with their outcomes: two records
  for each connection (its opening and its close), two for each call (its
  head and its terminal) and one for each delivery. Nothing drains them, and none
  is dropped: a record past the world's caps fails the peer's assertion,
  "frozen record count fits" or "frozen payload admitted before copying",
  and with it the story. So a world's caps are a stated room for a whole
  story, each named for what it holds, with its reason, and a story that
  outgrows it fails by name rather than losing its history.
- **A peer that serves a person keeps no records.** An application that
  runs the fake as a product, such as a demo's forge (temper's
  `protocol/setup/playground.md`), lives longer than any story's caps.
  `without_records()`, called on a fresh peer before its first pass,
  makes it keep none: `observations()` stays empty and the assertion
  never runs. A cap of zero
  still means a story that records nothing, and fails at its first
  record. The LLM and OAuth peers have the same mode (llm.md,
  section 3).
- **A story keeps what it reads.** `keeping(kept)`, on a fresh peer before
  its first pass, names the kinds it keeps (connections, heads, calls,
  deliveries); the others are never copied. The caps still bound what is
  kept and fail by name past it. A world whose referee reads only
  deliveries keeps those and the connections, so its records grow with
  the story's events, not with its clients' polls over wall time.
- **A Forgejo peer may serve TLS.** `tls()`, called on a fresh peer before
  its first pass, serves the fixed test chain (`ROOT`, for `skein.test` and
  `127.0.0.1`) with `https://` origins. Hook deliveries stay plaintext to loopback.
- **The owner may seed the setup's accounts.** `setup_mut()` lends the
  protocol layer's setup store (`forgejo.md`, section 1), as `Service::setup_mut`
  does, so that an owner makes, before the peer serves, the site
  administrator and its token that an application's setup takes over as
  an owned forge's (temper's `protocol/setup/owned.md`, 5). A setup
  account's number is the user number its tokens act as in the domain,
  so an owner that seeds both keeps the scenario's `users` apart from
  them.
- **It may keep its forge across a restart** (2.7). `Forge::image(&self)
  -> Result<Box<[u8]>, Refusal>`, taken once the peer has shut down: the
  domain's image (2.7) and its protocol layer's (`forgejo.md`, section 1; `github.md`,
  1.7), one byte string, each part length-prefixed, refused while the
  domain is busy. `Forge::from_image(config, service, bytes)` builds the
  forge from one in place of seeding it: the owner gives the
  configuration and the layer's service as for a new forge, and the
  image's state replaces what the service would seed. A refusal of
  either part names it, and nothing is built.
- **Plaintext by default.** Its `base` is `http://`; its face is the shared one
  with `Transport::Plaintext` and a `queue` of at least 64. The peer supplies
  its bound origin to either dialect's service before it accepts requests.
- **It hosts the layer over a forge it is handed.** The owner builds the
  forge once: its domain, its configuration, the time of the last pass and
  the replies not yet routed. It hands the peer a single-threaded shared
  handle, and may hand the same forge to any fake in its process that acts
  on it, such as a fake of a service's git component (section 3). A fake
  in another process reaches it on the peer's listener by the git route
  (`forgejo.md`, section 1), each request borrowing the forge in the peer's pass, by
  rule 1.
- **Rules of sharing:** (1) each entry point borrows the forge for one pass,
  so borrows never overlap; (2) the peer writes the world's time into the
  forge at the start of each pass, and another fake reads it; (3) whoever
  steps the domain leaves a reply or hook it did not ask for on the forge's
  pending list, and the peer routes the list with `Service::target` at the
  end of its pass; (4) each pass the peer fires the domain once, then
  reclaims the domain and the service; `next_deadline` is the earlier of
  io's and the domain's.
- **It writes the deliveries.** A hook the domain releases is rendered
  (`forgejo.md`, section 1) and delivered over loopback, once for each address its
  rendering names: the endpoint's `host`, or each setup hook's URL's
  authority, the address it connects to and the request's `Host` alike,
  as a Forgejo posts to its hook's URL. Each is a delivery of its own,
  with its own outcome and record. An owner whose listener binds port 0
  sets the endpoint in the forge it shares, or makes its hook through the
  setup routes, as an application in another process does, once that
  listener listens, its address known after io's listening event as the
  peer's own is, not at its bind, before a hook it means to have written
  is due. Until then, and for a `host` that is no loopback `ip:port`,
  there is no address; a setup hook that takes none (`forgejo.md`, section 1) is no
  delivery. The deliveries in flight are bounded by the connections, and
  each delivery is sized by the longest it can be: the
  endpoint's, or a setup hook's whose URL and secret are the longest the
  setup routes decode. Delivery is best effort:
  one that cannot be written (no address, no room, the connection refused
  or reset) is dropped and observed, never resent. A hook due before the
  owner's listener listens is such a drop, and one due once the owner has
  begun to stop may be; neither the peer nor the owner holds a hook back
  for the listener. A world asserts a delivery written only for a hook due inside
  its listener's life: a story set up before any host runs loses the
  setup's hooks, and a story that asserts the hook its own end causes
  ends only once the peer holds a delivery's outcome for each hook the
  story's own changes cause, a count the story states. A hook is scheduled
  at its change and released later, so no count of the hooks the forge
  has released so far (its domain's tally) says what a story is still
  owed: the story, which made the changes, does. Its domain's tally is
  checked against that count at the end.

## 6. Coverage and budgets

- **Step tests** sit in each crate. The domain's cover each call's decision,
  listings, merges, protection, CI, a declared wait's phases and a run's
  wait, and hooks; each fault of 2.4's table falling, once, with its
  moment and identities, and a full queue's count; a lagging view's
  reads and end, and a merge recorded late; a refused close; and one
  runner's queue, freed by a lost job. The protocol layer's cover Actions' states,
  routes, errors, faults, the tally, the setup routes and deliveries, each
  decoded by `skein-forge`, with a signature that holds for its body only;
  deliveries that follow the setup's hooks: one for each hook that takes
  it, in id order, each with its own URL's target and `Host` and its own
  secret's signature, the events' filter by row, the hooks that take none
  (inactive, an empty secret, an `https` URL, a name, no port, an address
  off loopback, an organization's), and an endpoint that wins; and the git
  route: each call's statuses, a credential's login, the
  store's calls beside the domain's faults, and `objects` at its bound.
  The observations' lines: every variant, error and checkpoint
  round-trips through `encode` and `decode`, a name that is not UTF-8
  has none, and a line with a member unknown, repeated or missing is
  refused by its name; `forward` for a fast-forward, a move that is not
  one, a branch made again after its deletion, one made for the first
  time, and one whose deletion the writer forgot.
- **Memory:** a forge filled to its limits, and a long merge history walked
  through both parents, stay under named scenario ceilings with headroom
  under the counting allocator (../README.md, section 6).
- **Replay and the peer:** a seed replays the names, replies, observations
  and deliveries. The peer's tests run a client over loopback against the
  protocol layer, with deliveries written, late, lost and refused, a
  repository's two setup hooks each written to its own listener, and a
  forge shared with a second fake.
- **The throwaway and the conformance:** tests of masking, facts, `app.ini`
  and refused versions and scratches need no binary and run in the focused
  suite; those that need one are ignored unless the caller names it. The
  tool itself runs outside both suites.
- **Budgets:** the focused and fuzzy suites' budgets for their layers
  (../README.md, section 7) bind these tests.

## 7. Open questions

- **A bridge for real git** needs a protocol it speaks, over a listener of
  the peer's. It is designed separately and lands in section 3. Default:
  none; a fake of a service's git reaches the store through
  `git::Remote`, in its process or by the git route (`forgejo.md`, section 1).
- **The GitHub layer's speculative merges.** A queue's group is built
  from the entry ahead, one at a time (`github.md`, 1.2); github.com batches groups.
  Default: one at a time, until a story needs batches.
- **An organization's hooks** are stored and route nothing: reconcile
  makes a repository's (../../forge/setup/forgejo.md, section 1.5), which deliveries
  follow (`forgejo.md`, section 1). Default: they stay so until a story needs an
  organization's.
- **The conformance against the fake.** Forgejo's conformance does not run
  against its protocol layer; that layer's step tests hold its quirks.
  GitHub's runs against its layer (`github.md`, section 2). Default: Forgejo's stays
  as it is.
- **Recorded live transcripts.** A live run's masked transcripts would serve
  the dialect's decoder tests. Default: the owner reviews them for leaks
  before committing them; the fake's own transcripts serve until then.
- **A commit's spelling on Forgejo's layer.** `routes::object` and
  `routes::commit` stand on the domain's `commit_bytes` and `commit_count`
  (`github.md`, section 3), and stay on that layer for their readers across skein's
  worlds, GitHub's among them, and temper. Default: they stay.
