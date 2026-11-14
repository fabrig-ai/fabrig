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
  setup's routes, signed deliveries and byte faults (section 4).
- **The forge peer** (in `crates/skein-fake-peers`, beside the LLM and OAuth
  peers) hosts the protocol layer on loopback, in plaintext, over a forge its
  owner hands it and may share with another fake in the process (section 5).
- **The throwaway Forgejo** (`testing/skein-throwaway-forgejo`) runs the
  pinned binary on loopback, on SQLite in a scratch directory (section 6).
- **The conformance** (`tests/forge/conformance`, outside the suites) drives
  the throwaway and records each fact the dialect relies on as `holds`,
  `fails` or `unchecked`, with masked transcripts (section 7).
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
  steps and logs (4); every other context, and a `File` or `Message`
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
  inputs kept, and a pending status per job on that commit, its context
  `<file> / <job>`, its link the job's page; then each job's verdict
  after a drawn latency, by the chance or the cue as for `Checks`, and
  the run completes once its last job does. `Runs` lists them, `Run`
  reads one with its jobs, and `Rerun` runs a completed run's failed job
  again, at its next attempt. A fault draws a job failed in its set-up
  step (`setup_faults`), or a run lost with no step failed (`lost`), so a
  world reaches the runner's faults; `Cue::Message` fails a job's
  workflow step, as a test would.
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
  passing. `Read::Artifact` reads it, and
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
    base64, so its bytes cross whole, as on the git route (section 4). A
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
a catalogue its protocol layer holds (section 4): the domain copies one
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
  the conformance records of Forgejo (section 7).

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
  protocol layer's tables (section 4), so a token for the API and one for git
  stand for the same user.
- **Within a process,** a working tree reaches the store through
  `git::Remote`, answered on the shared forge (section 5) without the
  network.
- **From another process,** the same calls travel the private git route
  (section 4) on the forge peer's listener, with the
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

It depends on skein-lib, skein-http, skein-json, `skein-forge`, the domain
and what the two layers share (`github.md`, section 3), and holds no clock of its own.

- **A forge it is handed.** It serves a fake forge its owner hands it and
  builds none (section 5). The forge peer hosts it.
- **Types.** `Server` is one HTTP connection (`MAX_UP` = 3, `MAX_DOWN` =
  18). `Service::new(config, &limits, seed)` owns routing, the tally and the
  setup store; `Service::target` hands a domain reply to its connection, or
  returns it whole when that connection has gone. Its `Limits` sit beside
  the domain's, and `new` checks `valid` before constructing the service.
- **The scenario's tables** (`Config`): the hooks' `Endpoint { forge, host,
  secret }`, if any, `host` being the loopback `ip:port` the forge peer
  connects to and sends as `Host` (section 5), and with none the setup's
  hooks route them (below); `base`, an `http://` origin; `tokens`,
  secrets to user numbers; `users`, numbers to logins; each repository's
  `id`, full name and protected branch; `max_response_items`,
  `default_paging_num` and `faults`.
- **Routes.** Outside the setup's, each route is one domain call: decoded by
  the server side of `skein-forge` (../../forge/forgejo.md, section 1.2), translated to a
  `Read` or `Write`, and its answer encoded as Forgejo encodes it. A write
  Forgejo answers with what it made or changed (an issue, a pull request,
  a comment, a review, an edited issue, a status, a branch) makes one call
  more, the domain's read of that, on the same request. A commit is a
  40-digit SHA-1 id: SplitMix64's finalizer of the domain's count, then the
  count, each eight bytes big-endian, then four zero bytes; any other id
  names none.
- **Credentials.** The header is `Authorization: token <value>`; an unknown
  token is `401`. Tokens stay in this layer: no domain call, observation or
  tally holds one. The tally counts requests at the entrance: total, by route
  (up to 74), by user, unrouted and unauthorized.
- **Forgejo's quirks are kept,** so a client that leans on Forgejo being
  better than it is fails here first: the default order is newest created
  first, and `sort=leastupdate` gives least recently updated first; a page is
  capped at `max_response_items`; a size without a page is ignored; an
  issue's comments, a review's comments and a comparison come unpaged;
  collaborators come without their permissions; a merge answers with no body;
  `pulls/{base}/{head}` names the oldest of several; a pull request whose
  head adds nothing is opened (`201`); an issue's edit, close or reopen
  answers `201`, as a reviewer's request does, and a reviewer's removal and
  a branch's deletion answer `204` with no body; deleting a branch that is
  not there answers `500`; a failed job's completion step reports
  `failure`; a push to a pull request's head or base starts a mergeability
  check. `../../forge/forgejo.md`, section 2 names the fact that records each, or lists
  those not yet.
- **Conditional reads, as Forgejo 17.0.1 answers them** (`../../forge/connection.md`,
  4.6): a raw file's read (`GET R/raw/<path>?ref=<ref>`) answers an `ETag`
  derived from the bytes it serves, the same for the same bytes, and a
  read of it whose `If-None-Match` names that `ETag` answers `304` with
  no body, after the read's own authorization and resource checks, a
  `Range` read included. Every other route sends no validator and
  ignores `If-None-Match` and `If-Modified-Since`, as Forgejo's branch
  JSON does. The tally counts a `304` as the request it is.
- **Forgejo's narrowing filters are honoured:** the open pull requests'
  listing keeps only rows whose head and base branches are its `head` and
  `base`, when given; an issue's comments keep only those updated at or
  after `since`, when given, still unpaged; the Actions runs' listing
  keeps only runs of the workflow file `workflow_id` names, when given,
  before it pages, and still ignores `workflow`. Each is a filter of the
  domain's read, as `head_sha` is, so the tally shows what a client's
  filter saved (`../../forge/forgejo.md`, 1.2; `forge.md`, section 9 lists them as not yet
  recorded). A reviewer's request answers with no body, where Forgejo
  sends the pull's reviews, which no client reads.
- **Faults move down to bytes.** A call refused for its rate is `429` with
  `Retry-After` in whole seconds, rounded up; one failed unavailable is `503`;
  one made and then timed out is `500`. The layer adds per mille byte
  faults: `reset` closes the connection before the next request is read;
  `drop` closes it, and `server_error` answers `500`, after the domain
  answered, so the call is made and its answer lost.
- **Deliveries.** `Service::delivery(&hook, &limits)` renders a domain hook as
  a `Post { head, body }`: a `POST` to `/hooks/<number>` with the event header
  and an HMAC-SHA256 signature of the body in lowercase hexadecimal, by the
  endpoint's secret (../../forge/README.md, section 6). A status the fake's CI made is
  delivered as its job's end; a status anyone else reports sends nothing, as
  on Forgejo. Of ../../forge/README.md's table (`../../forge/README.md`, section 6) it renders `issues`,
  `pull_request`, `issue_comment`, `pull_request_approved` for every review
  whatever its verdict (the hint is the same), `push` of a branch, `delete` of
  a branch, and `workflow_job_success` and `workflow_job_failure`; never
  `pull_request_rejected`, `pull_request_comment`, `create`, a tag's events,
  or a job cancelled or skipped.
- **Deliveries follow the setup's hooks** when the scenario names no
  endpoint, as Forgejo's follow the hooks its API made, such as the one
  an application's reconcile makes on each repository it adopts
  (../../forge/setup/forgejo.md, section 1.5). `Service::deliveries(&hook, &limits)`
  renders a domain hook once for each hook the setup store keeps on its
  repository that takes it, in id order, each with the loopback address
  to post it to: its target is the hook's URL's path, its `Host` the
  URL's authority, its signature by the hook's own secret, and its
  address that authority's.
  - A hook takes a delivery when its events include any event subscribed
    of the delivery's row in ../../forge/README.md's table (`../../forge/README.md`, section 6). The fake does
    not tell an issue's assignment from its edit, so a hook subscribed to
    `issue_assign` alone takes every `issues` delivery: a stated
    over-approximation, which a client that subscribes as ../../forge/README.md does
    never meets.
  - A hook takes none that is inactive, whose secret is empty, or whose
    URL is not `http://` with a loopback IP literal, a port and a path.
    So a fake never posts beyond its machine, nor unsigned. An
    organization's hook takes none (section 9).
  - A configured endpoint wins: with one, `deliveries` yields the
    endpoint's alone, and `Service::delivery` stays, the endpoint's
    rendering, for the worlds that pin an address.
- **The git route** is private, for a fake of a service's git that runs
  in another process (section 3); Forgejo has none like it. A request is
  `POST /.fake/git/<call>` with a JSON body, and its answer is JSON, one
  call each. A first segment that begins with a dot names no Forgejo
  account, so the route shadows none of Forgejo's. A commit is its count,
  a JSON number; a branch or repository name is a string; paths, contents
  and messages are base64, so bytes cross whole. The tally counts it
  route by route, and the byte faults reach it as they reach every route.
- **The repository's calls** are `clone`, `fetch`, `create`, `push` and `push-new`,
  section 3's `Op::Git` one for one, and `branch`, the domain's read of a
  branch's head, which a fake asks to word its output as git does. Each
  names a repository by its full name and is one domain call, with
  section 2.3's latency, rate and faults. Its credential is
  `Authorization: Basic` of a login and a token, as git sends what its
  remote URL or `http.extraHeader` holds: `tokens` maps the token to a
  user number, and `users` must give that number the login. A missing or
  unknown credential is `401`. A domain error is a status and a JSON body
  naming it: `403` forbidden, protected or refused, so the caller knows
  it was known; `404` missing, naming the repository, branch or commit;
  `502` unreachable; and `429`, `503` and `500` as above. A push answers
  pushed or rejected, a creation created or exists; `push-new` answers
  `existing: null` on creation or the unchanged head, all `200`.
- **The store's calls** reach git's object database, which is the
  domain's: `objects` reads commits, `commit`, `merge` and `commit-tree`
  store them as section 3's scenario functions do, and `ancestor` asks
  `is_ancestor`. They are no domain call, so they meet no latency, rate
  or domain fault, and they take no credential, as a local git command
  has none. That is the stated limit of a fake: anyone on loopback can
  read every object by its count and store new ones, though never move a
  branch. A store with no room is `507`, a tree past the domain's limits
  `413`. `new` refuses layer limits whose request or answer cannot hold
  the largest tree the domain allows.
- **`objects`** answers each commit it is asked whole (parents, tree and
  message), and the graph alone (the parents) of the commits before them
  by both parents, newest first, bounded by `max_response_items` and the
  answer's room. An object never changes, so a caller keeps what it has
  been given for the rest of its command.
- **The wire** uses standard, padded base64 for paths, contents and
  messages. Repository names are full names; commits are JSON integers.
  Members may arrive in any order.
  - `clone`: `{"repository"}` answers `{"default", "branches":
    [{"branch", "commit"}]}`.
  - `fetch`: `{"repository"}` with at most one of `"branch"` or `"commit"`
    answers `{"commit"}`; neither asks for the default branch.
  - `create`: `{"repository", "branch", "commit"}` answers
    `{"result":"created"|"exists"}`.
  - `push`: `{"repository", "branch", "commit"}` with optional `"expected"`
    answers `{"result":"pushed"|"rejected"}`.
  - `branch`: `{"repository", "branch"}` answers `{"commit"}`.
  - `objects`: `{"commits":[..]}` answers `{"objects":[{"commit", "parent",
    "merge_parent", "tree":[{"path", "content"}], "message"}],
    "graph":[{"commit", "parent", "merge_parent"}]}`. An absent parent is
    `null`.
  - `commit`: `{"parent", "tree", "message"}`; `merge`: `{"parent",
    "merge_parent", "tree", "message"}`; `commit-tree`: `{"parent", "tree",
    "message"}` with optional `"merging"`. Each answers `{"commit":N|null}`;
    trees carry `[{"path", "content"}]` as above.
  - `ancestor`: `{"ancestor", "commit"}` answers `{"ancestor":true|false}`.
  - `push-new`: `{"repository", "branch", "commit"}` answers
    `{"existing":N|null}`, the head when the branch exists, or `null` when
    this call created it by uploading the commit.
  - An error answers `{"error": NAME}`, NAME the domain error in snake
    case (`missing`, `rate_limited`, `already_queued`, ...), with the
    members that error carries: `"what"` for `missing` (`"repository"`,
    `"branch"`, `"commit"`, ...), `"reset"` in nanoseconds for
    `rate_limited`, and `"number"`, `"head"` and `"position"` for
    `already_queued`, so every domain error crosses whole. A refusal the
    layer makes before the call is decoded (an unknown call `404`, a wrong
    method `405`, a missing or unknown credential `401`, one that may not
    write `403`) has no body, and the client's decoder says whether an
    error was named, so a caller tells those, and a host with no route,
    from a domain error.
  - An `objects` call asking for more commits than `max_response_items`
    is `422`, before the store is read.
  - A body other than one object, an unknown, repeated or missing member,
    a wrong type, invalid base64, or both `"branch"` and `"commit"` on a
    fetch is malformed, `422`.
- **The setup's routes** (accounts, tokens, organizations, a repository's
  read by id, collaborators and hooks, and the admin's repository search,
  in id order and paged by number as Forgejo pages it) are served from this
  layer's own bounded store, not the domain's (../../forge/setup/forgejo.md, section 1).
  A repository read by id answers the domain's settings when the host
  lends the domain. A site admin's read of a configured repository by its
  path is answered the same way, with no domain call, as Forgejo shows a
  site admin every repository. A hook's patch sets its URL, events and
  active and keeps the secret the hook was made with, ignoring the one it
  sends, as Forgejo does (../../forge/setup/forgejo.md, section 2), so its
  deliveries are still signed with that secret.
- **A first repository is made in the domain** (../../forge/setup/forgejo.md, 1.9).
  The setup's `CreateRepository` in an org the setup store keeps makes
  a domain repository owned by that org, private, whose `main` is one
  commit holding `../../README.md`, `# NAME`; a name the domain has already is
  `409`. `CreateFile` on a domain repository commits the file on its
  branch, its author and committer the request's, and answers its
  commit; a path already there is `422`. Its git serves both commits,
  and its branch, raw and repository reads find them, as they find a
  seeded repository's. **Its CI** is `Config.made_checks`, the forge's
  CI for every repository it makes, this one and a copy's destination
  (2.6) alike, as a Forgejo whose runner takes every repository runs a
  new one's checks; `made_checks` with no contexts, its default, makes
  them report none, as before. A made repository keeps the checks it was
  made with across an image (2.7).
  Collaborator lists contain explicit grants only: site administration
  and organization ownership grant access without adding membership.
  Pagination counts the listed collaborators after excluding implicit access.
- **What the setup world turns and reads** (../../forge/setup/README.md, 5).
  `Service::refuse` answers one decoded route `403`, and
  `Server::take_request` gives the decoded requests in the order they
  came, refused ones among them. The setup store's `seed_repository_hook`
  and `delete_repository_hook` put a repository's hook in place, or take
  it away, with no exchange.
- **Its state, kept** (2.7): `Service::image(&self) -> Box<[u8]>` and
  `Service::restore(&mut self, bytes: &[u8]) -> Result<(), image::Refusal>`,
  in this layer's own format, beside the domain's. The image holds the
  setup store whole: its accounts, issued tokens, organizations, teams,
  subscriptions, collaborators, repository hooks and privacy, and the
  counters that number them, so that a token issued before a restart
  still acts as its account after it. It also keeps the runtime repository
  table, including repositories created or copied after startup, and the
  hooks' endpoint the owner set. A restore replaces those on a service built from the same
  configuration, and refuses, naming what, an image that does not
  decode or holds past the store's bounds. What lasts one connection
  (its decoded requests, refusals the setup world turned) is not kept.
- **A copy is the domain's too** (2.6). `Config.sources` is the
  catalogue, each `Source { address, repository, private, fill }`: an
  address, the domain's repository it stands for, whether it is private,
  and a `Fill`. Its repository has no row among `repositories`, so no
  route reads it, its git included. An address's host must end
  `.invalid` (RFC 2606): `valid` refuses any other, so no scenario can
  name a host that exists.
  - `Migrate` in an org the setup store keeps finds `from` in the
    catalogue, reserves the copy's row at the entrance, as a late
    write's must be read, and makes the domain's `Write::Copy` on the
    source: answered `CreatedRepository`, with the source's default
    branch and a new id, `409` for a name taken. A source the catalogue
    has not, or a private one, is refused with Forgejo's status and
    message, and leaves the empty repository Forgejo leaves
    (../../forge/setup/forgejo.md, section 2): its row reserved and the
    domain's repository made as a copy's is at its arrival (2.6), private,
    owned by the org, with no branch and `main` as its default branch, as
    the conformance's `setup-migrate-missing-source-read` reads. Nothing
    fills it; a `DeleteRepository` frees its name. The checks before the
    source's (an admin, the org, the name, a name taken, the store's
    room) come first, so a refusal of theirs leaves nothing.
  - `DeleteRepository` drops the repository's row, its collaborators
    and hooks and is the domain's `Write::DeleteRepository`, `204`, or
    `404` for a name it has not. An id is never given twice.
  - `Migrate` takes `write:repository` and `DeleteRepository`
    `write:organization`, as the admin-scope table has it (../../forge/setup/forgejo.md,
    section 2).
- **A setup grant reaches the domain.** A collaborator grant made through
  the setup routes is kept in the setup store, and the domain grants the
  account's user number (section 5) the same permission on the
  repository, as Forgejo's collaborator has it. So the bot that adoption
  grants reads and writes through the domain's routes, as an
  application's demo needs (temper's `protocol/setup/playground.md`). Only the
  grant crosses: the store's other rows stay its own.

The git decoder's `decoder_bound` bounds retained input scratch;
`tree_bound` gives the largest admitted tree's entries and bytes.

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
  protocol layer's setup store (section 4), as `Service::setup_mut`
  does, so that an owner makes, before the peer serves, the site
  administrator and its token that an application's setup takes over as
  an owned forge's (temper's `protocol/setup/owned.md`, 5). A setup
  account's number is the user number its tokens act as in the domain,
  so an owner that seeds both keeps the scenario's `users` apart from
  them.
- **It may keep its forge across a restart** (2.7). `Forge::image(&self)
  -> Result<Box<[u8]>, Refusal>`, taken once the peer has shut down: the
  domain's image (2.7) and its protocol layer's (section 4; `github.md`,
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
  (section 4), each request borrowing the forge in the peer's pass, by
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
  (section 4) and delivered over loopback, once for each address its
  rendering names: the endpoint's `host`, or each setup hook's URL's
  authority, the address it connects to and the request's `Host` alike,
  as a Forgejo posts to its hook's URL. Each is a delivery of its own,
  with its own outcome and record. An owner whose listener binds port 0
  sets the endpoint in the forge it shares, or makes its hook through the
  setup routes, as an application in another process does, once that
  listener listens, its address known after io's listening event as the
  peer's own is, not at its bind, before a hook it means to have written
  is due. Until then, and for a `host` that is no loopback `ip:port`,
  there is no address; a setup hook that takes none (section 4) is no
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

## 6. The throwaway Forgejo

The throwaway runs the pinned Forgejo for the tools and process tiers that
need a real one. It never meets an existing Forgejo.

- **The binary.** `Forgejo::from_path` takes the path from its caller and
  reads no environment. It runs `--version` and refuses, naming it
  (`Refusal`), a binary that is unreadable or whose version line does not
  name the pin's `VERSION` (../../forge/README.md, section 4.3). It computes the SHA-256.
  The pin's `RUNNER_VERSION` and `names_runner` are re-exported. The kit's
  tests that need the binaries are ignored by default; their test files
  read the paths from `SKEIN_FORGEJO` and `SKEIN_FORGEJO_RUNNER`, and fail
  naming the variable when it is unset. No library code reads a variable,
  and a service's tests read their own.
- **The instance.** `start(scratch, &Settings)` writes an `app.ini` for
  loopback, SQLite and the page cap, makes the admin user and its token, and
  waits for the API. The scratch must be empty or absent and not under
  `/tmp`. The environment is cleared, with a home in the scratch, a fixed
  `PATH` and `TZ`, and no system or user git configuration.
  `Settings::small()` pages at 2, enables Actions and bounds an answer at
  16 MiB. `Settings::local_migrations`, off in `small()`, writes
  `[migrations] ALLOW_LOCALNETWORKS = true`, for the conformance's copies
  alone (section 7).
- **Attaching.** `Instance::attach(child, port, scratch, admin_token,
  &Settings)` takes a Forgejo its caller started and keeps, such as a
  service's owned forge. It waits for the API and reads the admin's login
  and id with `GET /api/v1/user`, naming no login of its own. Its `stop`
  ends the child and leaves the scratch, which stays its caller's, and
  `cli` refuses, having no work path.
- **Provisioning:** users, tokens, repositories (`Init::{Readme, Empty,
  Sha256}`), collaborators and `fresh_secret`. `Forgejo::work_path` and
  `Instance::cli` run Forgejo's command line without and with a web
  process, keeping the exit code and both streams.
- **Exchanges.** `exchange` (a token, or none) and `exchange_basic` send one
  request and return what was sent and received, through skein's HTTP client
  driven by `drive` as a one-shot host (../../runtime/shell.md, section 11). A recording
  holds no credential: `token MASKED` or `Basic MASKED`. `secrets()` lists
  every secret the instance made, for a leak scan.
- **`stop`** removes the instance and its scratch; a start that fails cleans
  up. `kill` ends the process at once and keeps the scratch, and
  `restart` starts it again on that scratch and port. `exchange_patient`
  is `exchange` with the caller's own time for the answer, for a
  migration's one long request. The conformance starts the runner
  (section 7), not the throwaway.
- **Its server dies with its test,** however the test ends: `web` starts
  dying with the thread that starts it (`spawn_dying_with_parent`,
  ../../runtime/shell.md, 3, with Kill), so an abort or a runner's kill
  leaves no Forgejo on the scratch's socket and database. Its temporary
  files are the scratch's `tmp` (`TMPDIR`), so nothing it writes lies
  outside the scratch. The commands it waits on, `migrate`, the admin's
  creation and `cli`, stay plain spawns. A service that starts its own
  Forgejo and attaches it starts it dying with its test the same way.

## 7. The conformance

`tests/forge/conformance` is a tool outside the suites. It drives a throwaway
Forgejo with a page cap of 2, so paging shows, and records what the dialect
relies on (../../forge/forgejo.md, section 2). It makes its own admin user and tokens.

- **Inputs:** `--forgejo <path>`, `--scratch` (empty or absent, not under
  `/tmp`), `--out`, and optionally `--runner <path>`, checked with the pin's
  `names_runner`. It reads no variable.
- **Transcripts:** `transcripts/<case>/{request,response}` are raw HTTP/1.1
  bytes. A response is unframed, with chunking removed and `Content-Length`
  recomputed; JSON key order, whitespace and types stay as received. A case
  replaces only its own directory. Keys, nesting, types and status lines
  must match; ids, times, counts and text may vary. `transcripts/binary.txt`
  holds the binary's version line and SHA-256.
- **Deliveries:** `deliveries/<event>/{request,response}`, one per event of
  ../../forge/README.md, section 6, with a mask map of their own.
- **Facts:** `facts.txt` has provenance lines, then a line per fact (../../forge/forgejo.md,
  section 2): name, `holds`, `fails` or `unchecked`, reason.
- **Masking:** one map masks object ids and RFC 3339 times by first
  appearance, keeping width and offset; ports, `Date` and `Last-Modified`
  take fixed values; numeric resource ids stay (provisioning has a fixed
  order).
- **Secrets never reach a file.** Before masking, the tool registers every
  token's value and its eight-digit tail, and every password. It scans each
  response body before masking and every output before writing. A leak stops
  the run naming the case and the kind of secret, never its value.
- **A second instance,** for the `setup-migrate-` facts
  (../../forge/setup/forgejo.md, section 2), run by the focused mode `--migrate`,
  which writes its own transcripts and facts alone: the source, a
  throwaway of its own on loopback, started beside the first, whose
  `[migrations] ALLOW_LOCALNETWORKS` is on in the first alone
  (`Settings::local_migrations`, `forge.md`, section 6, which no other caller sets).
  The default's refusal is recorded on the second, which keeps
  Forgejo's default. A loopback source speaks plain `http`, so the mode
  records Forgejo's refusal of it, then writes `[migrations]
  ALLOW_UNENCRYPTED = true` into these two instances' `app.ini` alone. The
  source holds a private repository beside its public ones, asked with
  no credentials for a private source's refusal. The
  fact of which workflow directory Actions reads takes `--runner`, and is
  `unchecked` without it. A loopback proxy of the tool's own stands between
  them and stalls a source's bytes on demand, so a migration is cut
  short at a known point: by killing the first, or by the client going
  away.
- **The groups:** the dialect's cases; the setup (accounts, tokens,
  organizations, repositories' collaborators and hooks, the admin's
  repository search, under the least admin scopes, a copy's migration); `cli-` facts
  (Forgejo's command line, outcomes only); `runner-` facts
  (registration, the runner's connection, a workflow that clones a
  private repository with the job's token); and the Actions facts, which
  run two inline host-shell jobs, one succeeding and one failing, with
  nothing downloaded. The runner groups are `unchecked` without the
  runner.
- **The runner's daemon dies with the tool,** however the tool ends:
  it starts dying with its parent (../../runtime/shell.md, 3, with
  Terminate, so it ends its host-shell jobs within its one-second
  shutdown), its home, configuration, jobs and log in the scratch's
  `runner`, and its `TMPDIR` there too.
- **`--component`** runs the kit's component (../../forge/connection.md) through
  real io sockets on the shell's ring against a fresh fixture, under the
  scopes `read:user`, `write:issue` and `write:repository`. It replaces only
  the `component-` facts and records no transcripts. Each call's cost is
  compared with the request lines counted from socket sends.

## 8. Coverage and budgets

- **Step tests** sit in each crate. The domain's cover each call's decision,
  listings, merges, protection, CI and hooks. The protocol layer's cover
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
- **Budgets:** the focused suite's 40 CPU seconds and the fuzzy suite's
  120 (../README.md, section 7) bind these tests.

## 9. Open questions

- **A bridge for real git** needs a protocol it speaks, over a listener of
  the peer's. It is designed separately and lands in section 3. Default:
  none; a fake of a service's git reaches the store through
  `git::Remote`, in its process or by the git route (section 4).
- **The GitHub layer's speculative merges.** A queue's group is built
  from the entry ahead, one at a time (`github.md`, 1.2); github.com batches groups.
  Default: one at a time, until a story needs batches.
- **An organization's hooks** are stored and route nothing: reconcile
  makes a repository's (../../forge/setup/forgejo.md, section 1.5), which deliveries
  follow (`forge.md`, section 4). Default: they stay so until a story needs an
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
- **The runner's start.** The conformance starts the runner (section 7).
  Whether a second user moves that start into the throwaway is open.
  Default: it stays in the conformance until a second user needs it.
