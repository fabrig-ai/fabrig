# What the forge holds, and keeping up

The working set, the keep, fresh reads and the budget are the client's
(skein's `forge/client.md`, sections 3 to 6). This section keeps what
temper puts in them and the members that set them: the client's limits
are `forge.client.<limit>`, with durations in `_ms`
(`../../protocol/configuration/limits.md`).

## 1. The working set

Held by skein's `forge/client.md`, section 3. temper's part:

- **The live resources, and nothing else** (jig's
  `domain/connectors.md`, 3.4). A resource enters when a live task names
  it, and leaves when none does. The top names them to the client. A
  task's names leave the working set at its release: what the
  connector keeps of them for a kept task watches nothing (`adoption.md`, 1).
- **What temper watches:**
  - per repository: the landing branches' tips, and the CI on each tip;
  - per pull request temper owns that a live task names: its head and
    base, its state (open, closed, merged), its mergeability, CI on its
    head (each context's latest status, and the combined state), and,
    when a decision wants them, its files at its head;
  - per branch a live task holds: its tip as temper last pushed or
    updated it, to tell drift from temper's own moves;
  - per participating object: its comments and reviews not yet given to
    its task (`README.md`, section 7);
  - per issue of a live goal: what temper last wrote into it, so an
    unchanged projection is not written again.

  A change watches its base alone until its branch has a head (its producer's result, a handed branch, or a fresh read that finds one: `change/README.md`, 2, step 1), then its branch and its base until its pull is known, then its branch and its pull: a branch nothing has pushed is never polled, and the change's own fresh reads of it wait on what can push it (2, a branch nothing has pushed). A goal's run branch (`branches.md`) is held and never watched, since nothing pushes it. A change of the pull it owns wakes the change,
  as a hint does. A pull observation that changes the gathered facts also
  invalidates any in-flight gather for that change. Before deciding, the
  gather restarts its fresh reads; a wake coalesced while the procedure is
  already due cannot leave it waiting on an older CI verdict.
- **Its capacity:** `forge.client.resources` live resources and
  `forge.client.repositories` repositories. A batch naming resources past
  either bound is refused with the configuration member, its configured
  value and required usage. Task names, their count and their bytes are
  checked against `forge.tasks`, `forge.resources_per_task`,
  `forge.name_bytes` and the client's `forge.client.op_bytes` as well.
  A refusal carries the failed operation, its resource and its typed cause
  to the root. The task document carries the complete retained hold account,
  up to 512 bytes, independently of its general text preview bound. Both ctl
  output forms show that account. A persistent hold explains the act that resolves it:
  reduce the named work, or change the named configuration member and
  restart, then release; an access or backend failure names its own cause.
  A released change whose initial names were refused asks for them again;
  while the bound still refuses them it holds again.
- **The stock resource capacity is 16:** three chats naming one repository
  each and three changes naming two resources each use at most nine distinct
  watches. Two followed tier branches and up to four departing reads
  (`forge.client.pending`) keep the overlap within fifteen slots; shared
  resources count once. Six participating tasks and their six branch holds
  fit small's `forge.tasks` and `forge.holds`, both eight; each change's two
  names fit `forge.resources_per_task`, which is three. Landed changes keep their watches until
  they clear (`../after-landing.md`, section 6). temper's `standard`, which
  `temper-box init`'s box runs, sizes them for a project of fabrig's scale
  (`../../protocol/configuration/stock.md`): `forge.tasks` and
  `forge.holds` 128, and the resource capacity 128, at least the holds,
  each a held branch's watch.
- **Temporary admission pressure waits:** when recovery is incomplete,
  or a departing resource's read still occupies capacity, the desired
  names are saved and the complete watch set is retried after one second.
  Removing a task removes its desired names; retry never revives it.
  A restart rebuilds the watch set from those durable names. No person
  releases this wait. A fresh change read refused as busy keeps its timed
  procedure wait; other branch, base and pull read failures retain their
  operation, resource and backend cause in the hold.
- **Landing subscriptions own watches too:** every live subscription
  contributes its repository and branch, independently of task names and
  followed tiers. The complete union counts each resource once. Accepting,
  replacing or removing a landing subscription rebuilds that union; a shared
  watch remains until its final owner leaves. Admission checks the branch
  and the union's resource and repository bounds before retaining the
  subscription. Restart reconstruction, task-name changes and forge removal
  use the same union, excluding ended forges.
- **Follows own watches too:** each follow contributes its source
  repository and branch (`follows.md`, section 3) to the same union, for
  as long as its setting lives; accepting or removing a follow rebuilds
  it, and admission checks the union's bounds before the follow is kept.

## 2. Keeping up

Held by skein's `forge/client.md`, section 4: the listing per
repository since the forge's time, a pull request's backoff, the hints,
the landing branches' tips, the slow pass and the forge's clock. temper's
part:

- **Its values:** `forge.client.poll_ms`, `poll_max_ms`, `hinted_ms`,
  `slow_ms`, `settle_ms` and `fallback_ms`. The paces are what they are
  with no webhook, so a delivery lost, refused or never sent costs
  latency, never liveness (skein's `forge/client.md`, section 4).
- **Listings only for items.** A repository is listed only while one of
  its issue or pull watches lives (skein's `forge/client.md`, section 4):
  one temper names only for branches, as a goal's, a landing's and a
  tier's are, or watches as itself, lists nothing, and a hint there starts
  no listing. Its branches keep their own paces.
- **Events first, polls as a fallback.** While a repository's
  deliveries arrive, its timed reads, and temper's own polls of it,
  fall back to `fallback`, 5 minutes as shipped (skein's
  `forge/client.md`, 4.1). temper knows deliveries arrive by their
  arrival: each hint the client takes for a repository, temper's own
  echoes among them, keeps it hearing for `fallback`; no heartbeat or
  test delivery is sent. It is safe because nothing waits on a pace: a
  hint still reads at `hinted`, the read due at the last hint plus
  `fallback` reads all a lost delivery could have announced, and a
  repository with no delivery for `fallback` polls at the paces above
  until its next one. 5 minutes bounds that latency for a lost
  delivery while a busy repository, hearing every few minutes, makes
  each timed read about once per `fallback` in place of once per
  `poll`. `fallback_ms` 0 keeps today's paces. A Forgejo hook that
  stops, or a GitHub App's log not read, stops a repository's
  deliveries alike, and it stops hearing within `fallback`.
- **A branch nothing has pushed waits on what can push it.** A change
  has no head until its producer's result, a handed branch or a fresh
  read gives it one (`change/README.md`, 2, step 1). Until then only its
  own producer, whose result wakes it, or another hand, whose push the
  forge delivers, can make its branch, so it reads its branch afresh
  only on one of these: a hint naming the branch, or a push naming
  none; its producer's result, or a lost writer's reread (`branches.md`,
  1); its first step since the connector started; or `fallback` since
  it last read the branch. A step woken otherwise takes the branch as
  the last read found it, absent, and reads nothing the change does not
  decide on while it has no head: no `PullFor`, since no pull request
  has a branch that never existed, and no CI. Where its procedure names
  no deadline, its wait is `fallback`, not `queue_window`. A branch the
  keep watches before its first push, as a landing subscription may
  name one, rests at `fallback` after `Missing`, whether or not its
  repository hears, and a hint naming it or the owner's `Pushed` reads
  it at `hinted` (skein's `forge/client.md`, 4, a branch never there).
  **`fallback` is the unpushed branch's fallback,** 5 minutes as
  shipped, with no member of its own. It is safe because a change's
  branch is made by its own producer, whose result reaches the change
  at once, or by another hand, whose push's delivery reaches it at
  `hinted`: only a lost delivery of another's push waits, at most
  `fallback`, the latency a hearing repository already takes for each
  of its timed reads. It is long enough because a change waits on its
  dependencies or its producer for many minutes, through which `slow`'s
  40 seconds read an absent branch, and through it the repository's
  row (skein's `forge/connection.md`, 4.2), about seven times in five
  minutes for nothing. `fallback_ms` 0 keeps `slow` and `queue_window`.
- **Hints** come from the forge's webhook deliveries, or a GitHub App's
  delivery log (`../../protocol/forge/README.md`, section 4 and 4.1). Every hint
  goes to the client, temper's own echoes among them, which keep their
  repository hearing. The client reads on every hint it holds a
  resource for, and drops the news temper's own writes cause by its
  echo ledger and keys (skein's `forge/client.md`, 4 and 7.6); a hint's
  pusher is used for drift alone.
- **Which change a hint wakes:** within its repository, a named branch
  wakes changes with that head branch or landing base; a commit is
  news of its CI for the changes using it as their last head or
  observed base tip, including facts already gathered by an unfinished
  fresh-read step, and wakes them as a commit's news (below) has it. If the branch
  and pull reads saw different base tips, both remain dependencies until
  the next step reads them again. An item wakes its pull's change (including reviews on that pull), or a change naming
  that issue or pull. Named resources and subscriptions also retain their
  dependencies. Before a change knows its pull or commit dependencies,
  it cannot exclude those hints. A change with no head (above) has no
  pull request and no CI of its own: a branch concerns it only when it
  is its own or a push names none, and a commit or an item only through
  its named resources and subscriptions. A branch matching no change,
  including an unnamed push or a name no longer recognised after a
  rename, wakes every change in the repository that has a head. A commit
  or an item matching no change wakes none: each change whose
  dependencies are not yet known matches it already, so no other can
  be its news. The client still receives every hint:
  filtering procedure wakes never filters its applicable keep reads.
  A relevant wake during a fresh-read step still restarts that step before
  deciding; an irrelevant hint adds no restart. Gate results, task
  dependencies and queue changes keep their existing wake routes. Each
  decision still uses the rules held then (`change/gates.md`, 2, and `change/README.md`, 5, and `change/gates.md`, 3); hints do not replace
  polling, settling or stall deadlines (`change/gates.md`, section 1 and `change/repairs.md` and `queues.md`).
- **temper's own polls fall back too.** A landing's or a tier's flight
  polling its runs in flight (`../landing.md`, 7.1; `../after-landing.md`,
  section 3), a landing waiting on a fact it lacks (`../landing.md`, 5.2)
  and a queued merge's look (skein's `forge/client.md`, 7.7)
  are due at `Domain::poll_at` for their repository with `poll`, so
  while it hears they come at the later of `poll` and its last hint
  plus `fallback`. A job's delivery still wakes its flight at once.
  Settling's rereads (below), fresh reads and stall deadlines are not
  stretched.
- **Failures stop or back off.** A keeping read refused for its inputs
  (`TooLarge`: a branch's comparison, a comment or review scan past its cap,
  an answer past the client's bounds) is reported once and not read again
  until a hint or the listing names its resource, or its branch reads another
  tip (skein's `forge/client.md`, 4.2). Any other failure is read again after
  the client's `backoff`, 1 second as shipped, doubling after each failure in
  a row to `slow_ms`, and an answer sets it back; a failing listing waits
  alike. These waits are not stretched while a repository hears, and a
  restart forgets a refusal and tries once more.
- **What Forgejo settles after it delivers** is read again sooner, by
  the change procedure, which reads its pull request afresh (3), not
  by the keep. Forgejo checks a pull request's mergeability after a push
  moves its head or base, and answers it not mergeable while it checks;
  and temper relies on no order between a job's end being delivered and
  its commit status being written, only on both settling within `settle`
  (skein's `forge/forgejo.md`, section 2). So, within `settle` of a move of
  its head or base (known by a hint, a read or temper's own push), a
  pull request that reads not mergeable is read again at `hinted`,
  doubling up to `poll` apart, until it reads mergeable or `settle`
  has passed; and a commit a job's hint named is read again the same
  way while its CI does not yet show the ends the hints told of. Each
  job's hint is one job's end: a read of the commit's checks showing as
  many more rows concluded, since the last read before those hints, as
  hints named the commit settles them, and its pending is then believed.
  A later hint for the commit counts one more end and reads it once at
  `hinted`, without starting the doubling again, so a CI of many jobs
  is read about once a job, not every few seconds through its run. These rereads are temper's fresh reads,
  of the commit's checks alone (a commit's news, below). `settle` is a limit
  of the forge client's, which checks only that it is above 0 and runs
  no window (skein's `forge/client.md`, sections 4 and 5).
- **The window is the change procedure's own wait.** Until `settle` has
  passed since the move, not mergeable is not known: the change waits
  until the window's end at most, never stalls on it, and is never taken
  as conflicted on it; once the window has passed, not mergeable is a
  conflict (`change/README.md`, 2). A move long before, such as a head pushed before a
  long CI, leaves no window open. The window's start is the later of the
  change's own last push, which it keeps, and the last move a hint or a
  read told of, which it does not: after a restart, only its own push
  opens a window.
- **A commit's news.** A hint naming a commit, a job's on Forgejo, a
  status's or a check run's on GitHub, is news of its CI: it reads that
  commit's checks for each change using it as its head or its base's
  tip, once, and wakes a change only when they fold to a verdict other
  than the one the change last read there (3), or when `settle` passes
  with the ends its hints told of unshown. It is the one read a commit's
  hint makes of that CI: the client reads no pull request on it, and a
  pull watch's later read whose only news is a CI verdict the change
  has already read wakes it not again. The change's own fresh read
  of its other facts, its pull, branches, ancestry and reviews, runs
  when it decides, not on every hint. A hint naming a branch or an item
  wakes as above.
- **A checking change's pace.** While a change waits on CI, its pull and
  its head's CI are read on the forge's news where it arrives, and
  otherwise by the keep's fallback poll, its pull resting `poll`,
  doubling after each quiet read to `poll_max`, and stretched to
  `fallback` while its repository hears (above; skein's
  `forge/client.md`, 4 and 4.1): never on a timer of a few seconds. Pinned
  for one checking change on Forgejo whose Actions CI stays pending for
  ten minutes, in every minute past its first two
  (`../../protocol/forge/README.md`, section 9): hearing, a job's delivery
  every 20 seconds, at most 40 requests a minute in all, its head's
  statuses read at most 9 times and its pull at most 4; not hearing, its
  head's statuses and its pull each read at most 3 times a minute, and
  at most 30 requests a minute in all, its repository's listing among
  them. A read is a request to that route, a page counted as one. The
  head's 9 while hearing bound three a minute for each of the causes
  that read it on a job's hint: the commit's news read, and settling's
  reread of a status written after its delivery. The keep reads no pull
  request on a commit's hint (skein's `forge/client.md`, 4), and a
  change's gather reads its head's CI once (3).
- **GitHub settles too.** It answers mergeability unknown until it has
  checked, and may refuse a merge as not mergeable just after a head
  moves (skein's `forge/github.md`): within the window both are not
  known, and read again, as Forgejo's not mergeable is. An update GitHub
  accepts it makes after it answers: the update is done once the head,
  read afresh, contains the base's tip it asked for (`README.md`, section 6).

## 3. Fresh reads

Held by skein's `forge/client.md`, section 5: a fresh read goes first
in the request budget (4). temper's part is what it asks for. Before a
decision depends on a fact, the change child domain asks for it afresh:
the branch's head, the base's tip, the pull request, CI on the exact
head and on the base's tip, the ancestry range from the head to the base's
tip, the pull request's reviews, and the files of the pull requests an
overlap compares. These are also the facts the forge's requirements are
judged on (`change/gates.md`, 2).

A change that has landed or ended gathers no brief, and no comparison of its saved work or its branch against its head is read for it, however its watched resources move; a read already in flight is answered and its answer dropped. Its verification's reads of its landing (`../after-landing.md`, section 6) are its only reads. Its panel keeps its last facts (`change/README.md`, 5).

An agent's fresh read refused `Busy` by the client's local admission
returns the host's predecision Busy reply, never a rendered forge-read
error. The root releases that attempt's read correlation and rendering
flight. The host retries the same logical call after its backoff, within
the caller's deadline; cancellation ends that wait. Each retry goes
through the same authority and client admission checks, preserving the
call slab limit and background reserve (skein's `forge/client.md`, 5–6).
An admitted answer or any other terminal forge error follows the normal
read-answer path once.

**Each answer reaches the read that asked.** The client answers a read
under the owner it was asked with, and two reads under one owner cannot
be told apart. The root's fresh reads take their owners from a range
that is the root's alone (`temper_engine_domain_forge::root_read`): bit
61 set, bits 59, 60, 62 and 63 clear, and the root's serial below
2^59. The connector names none of its own reads there: its change
steps, landings and every other read it makes take owners outside it,
from its own sequence. It routes a client answer to its own pending
read first and passes an answer in the root's range to the root, so a
refused `Busy`, an answer or a failure reaches the one read that asked
for it, never a read of another kind or another party's.

- **Containment needs only ancestry.** The landing decision reads
  `Read::Commits { before: head, after: base_tip, page: 1 }`, tied to
  the head and base tip this step observed. A complete empty reverse
  range proves the head contains that base tip. Any commit the range
  lists disproves it, whether or not more follow, and so does a range
  the forge refuses as too large to list (`TooLarge`), which only a
  range with commits in it can be: the head lacks the base's tip
  however far behind it is, and the change takes its base (`change/README.md`, 2). A
  failed read otherwise, or an empty page that says more follow, leaves
  containment unknown and never permits a merge. A catch-up's range,
  from its landing branch's tip to the commit it takes, is read by the
  same rule (`change/README.md`, 2, step 1). This read collects no changed
  files or file contents, so a goal's diff and forward history do not
  bound its landing. After an update the proof is made again at the
  new head (`README.md`, section 6).
- **Every containment reader reads ancestry, never a comparison.** A
  reader that needs only whether one commit contains another reads the
  ancestry range by the rule above, never `Read::Compare`: `Read::Commits
  { before: container, after: contained, page: 1 }`, the commits the
  contained one has that the container lacks. On Forgejo that is one
  comparison with `files=false` (skein's `forge/forgejo.md`, 1.2), so a branch
  however far ahead of and behind the other answers in a page, and no
  file's name or side is read. Complete and empty, it is contained; a
  range that lists a commit, whether or not more follow, or one the forge
  refuses as too large to list, is not; any other failure, or an empty
  page that says more follow, leaves it unknown. Its readers:
  - **a landing's candidate,** whether its head contains main's tip
    (`../landing.md`, 5.2): `before` the head, `after` main's tip;
    contained, the head is the candidate as it is; not contained, it is
    prepared; unknown, the landing waits as it does for any fact it lacks;
  - **a goal's closer,** whether its base holds the goal branch's tip
    (`branches.md`, 2): `before` the base's tip, `after` the goal's tip;
  - **a lost writer's reread,** whether the branch moved forward from the
    head it last knew (`branches.md`, 1): `before` the observed head, `after` the
    known one; contained, the move is forward; not contained, it is
    drift; unknown, the reread fails as any failed reread does.
- **CI on a commit** is read as its checks, skein's `Read::Checks`: the
  commit statuses and, on GitHub, the check runs, each a row with its
  context and verdict (skein's `forge/forgejo.md`, 1.2; `skein/docs/design/forge/github.md`, 4.4),
  never the combined status alone, which on GitHub stays pending with no
  status at all while Actions' check runs end. Which rows are CI is the
  adoption's: where the landing branch's rules name required checks
  (`adoption.md`, 4, `Protection`'s contexts), those contexts, each of which must
  have reported; otherwise every row. They fold to one verdict: failed
  when any of them failed; else pending when any is pending or a
  required context has no row; else passed when any is there; else
  unknown, which the change waits on as pending (`change/repairs.md`, stalls). The
  head's, the base tip's and the CI a topic publishes (`README.md`, 5.1) are read
  and folded alike, on every forge: on Forgejo the checks are
  its statuses, each context's latest, so its CI reads as before.
- **Each commit's CI is read once in a gather.** A pull request's read
  asks for its CI only where its reader uses it, and `PullFor` reads
  none (skein's `forge/README.md`, 3.2). In a change's gather, where no
  required checks are named and its pull request is known by number,
  the pull request is read with its CI, which folds every row of its
  head as `Checks` does (skein's `forge/github.md`, 4.4), and that is the
  head's CI when the pull request is at the head: no `Read::Checks` of
  the head is made. Otherwise, required checks named, the pull request
  found by `PullFor`, or the pull request at another commit, the pull
  request is read without CI and one `Read::Checks` of the head is its
  CI. The base tip's CI is read once, and not again when the base tip
  is the head. A gathered pull request read without CI differs from a
  watched one (1) by none of its facts but CI. The keep's pull watch
  reads its CI, and so does the agents' `read_pull`, whose answer shows
  it; a landing's members, a brief, an overlap's bracketing reads, a
  merge's verification and every other reader that uses no CI read
  none.
- **One read shared.** Fresh reads of one resource asked together are
  made once (skein's `forge/client.md`, 5, shared): the goal branch,
  which each of its changes reads as its base and its landing as a
  member's head, is read once for the changes one hint wakes together,
  not once each. A landing's gather reads each branch and each pull
  request once, however many of its members name it.
- **The repository's own facts are read once** and kept by the forge
  component (skein's `forge/connection.md`, 4.2): its settings answer
  `Settings` with no request, the agents' `read_settings` among them,
  until a redirect, a `404` that is no answer, a merge refused its style
  or a write refused `Forbidden` drops them, each the forge's word that
  they may have changed. A refusal's refresh of what temper may do
  (jig's `domain/connectors.md`, 12) so reads them afresh. A pull request found by `PullFor`
  as none, a missing branch, file, run, job or log keeps them.
- **Every page folded, never a page's combined state.** A forge may
  compute a combined state and a count over the page it was asked for
  (Forgejo 17 does: a read of one status reads success while a job is
  still pending), so CI on a commit, a pull request's included, is
  every row of every page, folded as above, and a combined state read
  from a page that came full is never a verdict. A first page that came
  short holds every row, so its combined state is their fold, which
  skein's Forgejo dialect takes for a pull request's CI but where it
  reads `warning` (skein's `forge/forgejo.md`, 1.1). The fake forge computes a
  page's combined state as Forgejo does, so that the worlds meet it.

## 4. The request budget

Held by skein's `forge/client.md`, section 6: the four queues in their
order, the charge of each call and the wait after a refusal for the
rate. The order is the one jig's `domain/connectors.md`, section 13
gives: fresh reads first, a write's own checks and finds among them;
then writes; then keeping up; then the slow pass. temper's values:
`forge.client.rate` requests per `forge.client.window_ms`, with
`forge.client.reserve` of each window kept for keeping up and the slow
pass.

## 5. A forge away

A forge that stops answering for a while (connections refused, a 502 or
a 503, timeouts), because it restarts, fails or its host is down, and
then answers again, is ridden: nothing temper holds fails or is held for
an outage that ends, and what changed meanwhile is read once it is
back. The forge stays connected throughout: connected is its access
(`adoption.md`, 1), not whether it answers.

- **Keeping reads** wait and read again as skein's `forge/client.md`,
  section 4 has them.
- **Fresh reads are asked again.** A fresh read that ends `Unavailable`
  or `Timeout` once the client has spent its attempts (skein's
  `forge/client.md`, section 5) is asked again by the step that asked
  it, after a wait that starts at `forge.client.backoff_ms`, doubles
  after each such ending in a row to at most `forge.client.slow_ms`, and
  is set back by an answer. Meanwhile the change, the landing, the
  goal's branch or the lost writer whose read it is stays where it was:
  no `ResourceUnavailable` or `EffectFailed` hold is made for it, and
  the wait counts toward no stall (`change/repairs.md`). A refusal for its inputs
  (`TooLarge`, `Forbidden`, `Refused`) and `Missing` keep their own
  rules.
- **Writes are never sent blind.** One the client holds uncertain is
  found as its class says (`README.md`, section 4). One the client fails
  `Unavailable`, never sent, once it has spent its attempts, is
  committed again by the connector, as one failed `NotConnected` is
  (`adoption.md`, 1), after a wait as a fresh read's: its purpose gives it the same
  key, so nothing is made twice.
- **CI is read again.** A checks read (3) that ends `Unavailable` or
  `Timeout` is asked again as any fresh read is, never dropped. And once
  a forge answers again after such an ending, each CI subscription on
  its repositories reads its head's checks afresh, so a status posted
  while it was away is read whether a delivery comes or not.
- **A run followed counts no lifetime unseen.** A gate's or a tier's run
  (`../landing.md`, `../after-landing.md`) is lost for not being found only
  once a read has answered without it past its lifetime: reads that end
  `Unavailable` or `Timeout` are waited out, and neither a runner fault
  nor a retry is spent for them.
