# The fake forge's Forgejo layer

Provisional, 2026-11-21. The fake forge's Forgejo layer over the domain of
`forge.md`, section 2.

## 1. The protocol layer

It depends on skein-lib, skein-http, skein-json, `skein-forge`, the domain
and what the two layers share (`github.md`, section 3), and holds no clock of its own.

- **A forge it is handed.** It serves a fake forge its owner hands it and
  builds none (`forge.md`, section 5). The forge peer hosts it.
- **Types.** `Server` is one HTTP connection (`MAX_UP` = 3, `MAX_DOWN` =
  18). `Service::new(config, &limits, seed)` owns routing, the tally and the
  setup store; `Service::target` hands a domain reply to its connection, or
  returns it whole when that connection has gone. Its `Limits` sit beside
  the domain's, and `new` checks `valid` before constructing the service.
- **The scenario's tables** (`Config`): the hooks' `Endpoint { forge, host,
  secret }`, if any, `host` being the loopback `ip:port` the forge peer
  connects to and sends as `Host` (`forge.md`, section 5), and with none the setup's
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
  Contents reads and combined status reads also accept a named branch:
  the layer reads that branch with the caller's permission, then reads its
  commit, answering `404` when the branch is absent, including before a
  repository's first commit. A contents batch (`POST
  /repos/OWNER/NAME/contents`) validates every create or update, including
  each update's blob SHA, before storing one commit. In a repository with
  no branch it creates the first branch and a commit without a parent;
  otherwise it commits on the named branch. With `new_branch`, `branch`
  names an existing base: the batch makes the new branch at that base's
  head with the commit, whose parent and unchanged files come from the
  base. An existing new branch is refused (`422`), a missing base is
  `404`; neither changes a branch or stores a commit. Every validation
  and room check precedes storing the commit and making the branch.
  Moving that head starts the repository's ordinary CI (`forge.md`, 2.3). A present
  file's contents read answers
  JSON with its blob SHA and base64 content, so the next batch can update
  it.
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
  a branch's deletion answer `204` with no body; a close or reopen of a
  merged pull request answers `412`, `cannot change state of this pull
  request, it was already merged`, before the domain is called, and changes
  nothing (the GitHub layer still closes it, `200`); deleting a branch that is
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
  filter saved (`../../forge/forgejo.md`, 1.2; `forge.md`, section 7 lists them as not yet
  recorded). A reviewer's request answers with no body, where Forgejo
  sends the pull's reviews, which no client reads.
- **Actions' states, in Forgejo's words** (`../../forge/job-state.md`,
  3): a run, and each job of it, reads `waiting` while its `wait` is
  `Runner`, `blocked` while it is `Needs`, `running` while it runs
  with none, then its conclusion (`forge.md`, 2.3). A job's read lists the steps it
  has: none while it waits, `Set up job` and the step it is at,
  `running`, while it runs, and each step once it completes.
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
    organization's hook takes none (`forge.md`, section 7).
  - A configured endpoint wins: with one, `deliveries` yields the
    endpoint's alone, and `Service::delivery` stays, the endpoint's
    rendering, for the worlds that pin an address.
- **The git route** is private, for a fake of a service's git that runs
  in another process (`forge.md`, section 3); Forgejo has none like it. A request is
  `POST /.fake/git/<call>` with a JSON body, and its answer is JSON, one
  call each. A first segment that begins with a dot names no Forgejo
  account, so the route shadows none of Forgejo's. A commit is its count,
  a JSON number; a branch or repository name is a string; paths, contents
  and messages are base64, so bytes cross whole. The tally counts it
  route by route, and the byte faults reach it as they reach every route.
- **The repository's calls** are `clone`, `fetch`, `create`, `push` and `push-new`,
  `forge.md`, section 3's `Op::Git` one for one, and `branch`, the domain's read of a
  branch's head, which a fake asks to word its output as git does. Each
  names a repository by its full name and is one domain call, with
  `forge.md`, section 2.3's latency, rate and faults. Its credential is
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
  store them as `forge.md`, section 3's scenario functions do, and `ancestor` asks
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
  (`forge.md`, 2.6) alike, as a Forgejo whose runner takes every repository runs a
  new one's checks; `made_checks` with no contexts, its default, makes
  them report none, as before. A made repository keeps the checks it was
  made with across an image (`forge.md`, 2.7). `Config.made_workflows` gives every
  repository it makes its dispatch-only workflows, none by default;
  `Config.made_manual_merges` gives its manual-merge settings, both off by
  default. A made repository and a copy's destination alike take these,
  and keep them across an image.
  Collaborator lists contain explicit grants only: site administration
  and organization ownership grant access without adding membership.
  Pagination counts the listed collaborators after excluding implicit access.
- **What the setup world turns and reads** (../../forge/setup/README.md, 5).
  `Service::refuse` answers one decoded route `403`, and
  `Server::take_request` gives the decoded requests in the order they
  came, refused ones among them. The setup store's `seed_repository_hook`
  and `delete_repository_hook` put a repository's hook in place, or take
  it away, with no exchange.
- **Its state, kept** (`forge.md`, 2.7): `Service::image(&self) -> Box<[u8]>` and
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
- **A copy is the domain's too** (`forge.md`, 2.6). `Config.sources` is the
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
    domain's repository made as a copy's is at its arrival (`forge.md`, 2.6), private,
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
  account's user number (`forge.md`, section 5) the same permission on the
  repository, as Forgejo's collaborator has it. So the bot that adoption
  grants reads and writes through the domain's routes, as an
  application's demo needs (temper's `protocol/setup/playground.md`). Only the
  grant crosses: the store's other rows stay its own.

The git decoder's `decoder_bound` bounds retained input scratch;
`tree_bound` gives the largest admitted tree's entries and bytes.

## 2. Open questions

The fake's open questions are `forge.md`'s, section 7.
