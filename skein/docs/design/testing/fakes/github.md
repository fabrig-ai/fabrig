# The fake GitHub

## 1. GitHub's layer

`crates/skein-fake-forge-github` serves github.com's API, at the
dialect's pin (../../forge/github.md), over the same domain, beside the Forgejo
layer and in its shape (`forge.md`, section 4): a `Server` per connection, a
`Service` that owns routing, the tally and the layer's own stores, a
`Config` of the scenario's tables, and `Limits` checked by `valid`.
Each route is decoded by the server side of `skein_forge::github` and
answered as GitHub answers. GitHub's quirks live here. The domain gains
only what GitHub has and Forgejo lacks (1.2).

The layer serves the checks and job reads, signed job-log redirects, the
peer's blob listener and its redirect scenarios, drawn mergeability settle
deadlines, person credentials, installation scopes and revocation, and byte
faults; the service and the hosted peer each have an allocator binary.

### 1.1 What it serves

- **REST at the pin.** Every route of ../../forge/github.md, sections 4, 8, 9
  and 11, at github.com's paths, with no prefix. A request without
  `X-GitHub-Api-Version: 2026-03-10`, or without `User-Agent`, is
  refused as GitHub refuses it (`400`, `403`). Pages are GitHub's: up to
  `per_page` rows, at most 100, with `Link` next while more remain.
  Issues and pull requests share one numbering, and the issues' listing
  holds pull requests, each with its `pull_request` member. An absent `since`
  lists all rows; otherwise the bound is inclusive (`updated_at >= since`).
  A `since` before `1971-01-01T00:00:00Z` answers `200 []`. The named
  `ISSUES_SINCE_MIN` bound in the GitHub reads layer is the earliest observed
  nonempty bound; the intervening dates were not probed further. An
  issue's comments take `since` the same way, inclusive on each comment's
  `updated_at`, before they page.
- **GraphQL at `POST /graphql`.** The layer answers the dialect's three
  documents, matched byte for byte against its constants (`viewer`,
  `pull`, `enqueue`), each with its variables. Any other document is
  answered `200` with an `errors` entry. This is a difference from
  github.com, which answers any query.
- **Credentials.** `Authorization: Bearer <t>` names one of three kinds:
  - a person's token (the scenario's `tokens`, as in `forge.md`, section 4);
  - an installation's token, which the layer mints;
  - the App's JSON Web Token, whose signature the layer checks with
    `skein_tls::verify_rs256`, by the public half of the App's key
    (`rs256_public_pkcs1`, ../../net/tls.md, section 4); a key with no public half
    verifies nothing. The layer
    keeps the last token it verified, its exact bytes and its claims, so
    the same token again under the same key skips that check; a new key
    or a revoked one forgets it, and the claims are checked on every
    request. A token past its `exp`, or more than 600 seconds after its `iat`, or not
    yet valid by more than 60 seconds on the forge's clock, is `401`.

  The App's routes on another kind are `401`. `GET /user` on an
  installation's token is `403`. Tokens stay in the layer, as in
  `forge.md`, section 4. Each person secret names a configured account's domain user;
  repository permissions and write authorship follow that user. An optional
  expiry on each configured person token emits
  `github-authentication-token-expiration` on that token's answers, in
  `YYYY-MM-DD HH:MM:SS UTC` form. No expiry emits no header.
- **The App** is held in the layer's store: id, slug, owner, key,
  webhook secret, permissions, events and the hook's URL. Its bot is a
  domain user, `<slug>[bot]`, who acts on the domain. `GET /app`, `GET`
  and `PATCH /app/hook/config`, and `GET /users/<slug>%5Bbot%5D` read and
  change the store. The bot read needs no credential and answers its
  `login`, `id` and `type: "Bot"`; another slug is `404`. The scenario
  owns the domain bot's stable id, distinct from the App's id; conversion
  skips it when allocating an App id. `GET /app` names the App's account
  from the scenario's accounts, `id`, `login` and a `type` from the
  account's kind; an owner no account holds answers none.
- **Installations and their tokens.** An installation is an account and
  either all its repositories or a list of them, and may be suspended.
  A repository's `owner.type` is its account's kind, so one an
  organization owns reads `Organization`, and its pulls take the queue
  (../../forge/github.md, 4.1 and 5.1).
  The mint answers a token that expires an hour later on the forge's
  clock, full, or contents-only when the body asks. The domain sees the
  bot's user number, and the layer refuses beforehand what the token
  does not reach:
  - a repository outside the installation is `404`;
  - issue creation succeeds even for a contents-only token, with `201`
    and `x-accepted-github-permissions: issues=write`; editing its title
    is refused with `403` "Resource not accessible by integration";
    the mint's narrowed permissions are `contents: write`, `metadata: read`;
    the remaining contents-only route refusals are inferred `403` responses;
  - an expired token is `401`.

  `GET /app/installations` lists them and `GET /app/installations/<id>`
  reads one, each row with its account's type and its creation time on
  the forge's clock. `DELETE /installation/token` revokes a token. The
  App's `DELETE /app/installations/<id>` removes an installation.
  Every routed REST request and
  GraphQL document is checked against full, contents-only, expired, revoked,
  person and JWT credentials. Contents-only GraphQL refusals use `errors`.
  The conformance case records the mint response permissions object and
  the statuses of issue creation and its title PATCH with the same token.
  An installation's all-repositories selection stays within its account.
  A person's `PUT` or `DELETE /user/installations/<id>/repositories/<repo>`
  checks that user's domain admin permission and answers `204` when it changes
  the selection. The repository must belong to the installation's account.
  Removing from an all-repositories installation is `422` "Validation Failed",
  a docs-derived text unchecked live. Removing the last listed repository is
  `422` "Cannot remove the last repository from this installation.", checked
  live on both fixture installations. Neither changes reach. `PUT` on
  an all-repositories installation is `204` without changing reach.
- **Git, on the private route.** The layer serves `forge.md`, section 3's private
  git route at the same path as the Forgejo face, over the same domain
  and its store, so the fake git program reaches a GitHub forge as it
  reaches a Forgejo one. A git call is authenticated by HTTP basic
  authentication with the login `x-access-token` and an installation's
  token that the layer minted, full or contents-only: the token
  unexpired on the forge's clock and not revoked, its installation
  present and not suspended, and the repository its account's and
  within its selection. Clone, fetch and a branch's read need the token
  alone; a push and a creation need contents write, which both kinds
  have, and the domain's protection still applies. No credential is
  `401`; a token that fails one of these, another login, or a person's
  token is `403`, and a repository outside the installation `404`, as
  its REST routes answer. The domain sees the bot's user number. The
  mint's life is the scenario's (`Config`'s token life), an hour unless
  it says otherwise, so that a story on real time sees a renewal. The
  layer keeps a bounded record of its git calls, each with its
  installation and the mint number of its token, which a scenario
  reads to tell which token a push carried.
- **Ruleset mutations.** A person's `POST /repos/<o>/<n>/rulesets`
  replaces the domain's repository protection, requiring admin permission;
  `DELETE /repos/<o>/<n>/rulesets/<id>` removes it. The subset is strict:
  `name`, branch `target`, active `enforcement`, a single
  `conditions.ref_name.include` with empty `exclude`, `bypass_actors`,
  and approvals, required status checks and merge queue rules. Unknown and duplicate fields are
  `422` "Validation Failed". The pull request rule requires the approval
  count and all four booleans (false in this subset). Required status checks
  require both the contexts array and `strict_required_status_checks_policy`
  (false in this subset). Merge queues require all seven documented
  parameters: `check_response_timeout_minutes`, `grouping_strategy`,
  `max_entries_to_build`, `max_entries_to_merge`, `merge_method`,
  `min_entries_to_merge`, and `min_entries_to_merge_wait_minutes`.
  A missing required parameter answers the observed 422 Invalid request
  message naming `/rules/N` for that rule's index and data matching no
  possible input. The accepted shapes follow the documented schema.
  A merge-queue ruleset on a user-owned repository is an inferred 422
  "Validation Failed"; live's refusal form is unknown.
  Creation answers `201` with the configured ruleset id and the existing
  server encoder's bypass field; only the id is recorded, other fields are
  inferred. Deletion's empty `204` is recorded. Installation tokens receive
  an inferred `403` "Resource not accessible by integration".
- **Comment edits.** `PATCH /repos/<o>/<n>/issues/comments/<id>` takes
  exactly `{"body": "..."}` and calls the domain's `EditComment` write.
  A full installation token may edit as the domain's authorship and write
  permission allow; installation reach still applies. Fixture person tokens
  receive `403` "Resource not accessible by personal access token".
  It answers `200` with one comment from the server's comment encoder,
  including its updated timestamp. Unknown or duplicate fields are `422`.
  The person-token refusal is checked live. The installation-token edit and
  its advancing updated timestamp remain unconfirmed live.
- **The browser's pages, as plain HTTP** (seams S4), at the `web`
  routes, which the same listener serves:
  - `GET /favicon.ico` is a browser resource, with no API version or
    credential. The fake supplies no icon and answers a bodyless `204`,
    keeping the connection available for the App's next page. Other API
    routes still require their version header. This keeps the browser's
    automatic icon lookup from reporting an API-version error as trouble.
  - `POST /settings/apps/new?state=<s>`, and
    `/organizations/<org>/settings/apps/new?state=<s>`, take the form's
    `manifest` field. The answer is `200` with a small page holding one
    link, `Create GitHub App` (id `create`), to the manifest's
    `redirect_url?code=<c>&state=<s>`; a test follows it as a person
    clicks GitHub's own form, and no form's `form-action` governs it.
    The code is single-use and expires an hour later. The layer keeps
    each state's posted manifest, which a test reads through the
    layer's handle by state (its `redirect_url`, `setup_url`,
    `hook_attributes` and the rest), since the page's post is not
    otherwise in sight.
  - `POST /app-manifests/<code>/conversions` makes the App and answers
    as ../../forge/github.md, section 11, says. The converted App retains every
    posted `default_permissions` and `default_events` entry. The scenario's
    `Config::app_facts` is `Complete`, `Omitted` (GET /app omits both fields
    while retaining the grants), or `Narrow` (conversion retains only the
    first permission at read access and the first event). The posted
    manifest remains readable in each case. Conversion replaces the fake's
    one App, clearing the previous App's installations, tokens and pending
    installation events while keeping ids and random streams advancing.
  - `GET /apps/<slug>/installations/new?state=<s>` takes `account=<login>`
    (by default the App's owner), and `repositories=all` (the default)
    or `repository_ids=<id>,...`. It installs and answers `302` to
    `setup_url?installation_id=<i>&setup_action=install&state=<s>`; for
    an account already installed, `setup_action=update`.
  - An existing seeded App can register its callback with
    `Service::set_setup_url`; it takes an absolute HTTP(S) URL within the
    configured string bound, without a query or fragment, or `None`.
    Manifest conversion replaces that callback with the new App's.
  - A malformed manifest, or an unknown account, is `422` with a
    message.

  The keys an App can get are the layer's fixed test keys (`KEYS`,
  2048-bit, made for the fake and public). The conversion hands them out
  in turn.
- **Status posts.** `POST /repos/<o>/<n>/statuses/<sha>` refuses installation
  tokens with `403` "Resource not accessible by integration", including an
  App advertising statuses write. Fixture person tokens receive `403`
  "Resource not accessible by personal access token". Both are checked live.
- **Pull creation refusals.** No commits and an existing pull answer `422`
  with top-level message "Validation Failed" and one errors object with
  resource "PullRequest", code "custom", and message "No commits between
  <base> and <head>" or "A pull request already exists for <owner>:<head>.".
  These forms are checked live; the dialect reads errors[].message.
- **Checks.** The domain's CI statuses for a commit are rendered as the
  `github-actions` App's check runs, one per context. The run is the
  domain's `Run`, the job its `Job`, and the attempt the job's. Statuses
  that people report stay statuses. The combined status is `pending`
  with `total_count` 0 when there are none. The conformance fixture configures
  `ci.rs`, `actions.rs`, and `Cue::Message` to stand for its repositories'
  `.github/workflows/ci.yml`: push, pull opening/head movement, and merge
  groups run `ci` on their head commit. It is in_progress for 45 fake seconds,
  then completed with failure when that commit's message contains FAIL,
  success otherwise. The run and job use the same ids and logs as other
  Actions cases; the GitHub layer renders them in `reads/checks.rs`. CI runs
  once per commit and repository, coalescing overlapping fixture triggers.
  A job's metadata renders the domain's first failed step by number and
  name, or an empty steps list when none failed.
- **A job's log** is a `302` to the blob origin: a second listener of
  the peer, the origin `http://<blob>`. The target is a path signed for
  600 seconds on the forge's clock. The blob listener serves the job's
  log and wants no credential. The HTTP listener honours the one-byte
  `Range: bytes=0-0` probe with `206` and `Content-Range`; other ranges
  are ignored. The in-process blob call records header presence only.
  The scenario may name a third origin, one the component does not know, for the diagnostic's
  stories. The HMAC-SHA-256 key is drawn from the seed; the signed input
  names the repository, job and attempt and includes the expiry. At the
  expiry instant the target is refused. A tampered or expired target is
  `403` with an empty body. That shape is inferred: github.com's blob
  store is not modelled. Other paths are `404`.

  `Config::third_origin` replaces the API redirect's blob origin;
  `Config::blob_redirect` makes the blob listener answer a `302` to that
  origin with the same signed target. Both default to `None` in fixtures.
  Each origin is admitted only when its origin plus the largest signed
  target fits `Limits::github.target_bytes` and `Limits::response`.
  The listener records the path and whether `Authorization` and `Range`
  were present, never their values. `Limits::blob_records` bounds this
  FIFO, with the oldest request evicted when full; each path is bounded
  by `Limits::http.head`.
- **Own approval.** A review of one's own pull answers 422 with message
  `Unprocessable Entity` and string errors containing
  `Review Can not approve your own pull request`.
- **Mergeability, merges and the queue.** `mergeable` is null for a
  drawn `settle_min..=settle_max` after a push to the head or the base.
  The domain draws from its seed, carries `checking` separately from
  `mergeable`, and defaults both bounds to two seconds in fixtures.
  Forgejo continues to render false while checking.
  A merge then decides as the domain does:
  - a merged pull answers any repeat merge `200`, `merged: true`, with the
    same merge sha, including a different supplied sha;
  - an open pull's stale head is `409`;
  - a merge refused for its checks or its approvals is `405` with
    `Repository rule violations found\n\n`, then the unmet approval
    sentence, then the unmet context lines, each followed by `\n\n`;
  - the approval sentence is `At least 1 approving review is required
    by reviewers with write access.` for 1, and `At least 2 approving
    reviews are required by reviewers with write access.` for 2.
    The singular and the plural for 2 are recorded live; counts of 3
    and up use the plural by inference;
  - a branch that queues refuses a direct merge with `405`, the same
    header, then `Changes must be made through the merge queue\n\n`,
    then the unmet approval and context lines in the same order.
    A live record shows the queue line followed by the approval
    line. Context lines after the queue line are inferred: neither
    shape was recorded with checks, and the record lists every
    violated rule. With no unmet rules, only the queue line follows
    the header;
  - an enqueue refused for approvals answers HTTP `200`, with
    `data.enqueuePullRequest` null and one `UNPROCESSABLE` error.
    Its message is `Pull request ` followed by the approval sentence,
    without trailing blank lines, and nothing is queued. Live session
    1917 records this for 1; other counts use the shared sentence;
  - after approvals, enqueue requires every queue ruleset check to have
    passed on the pull's head. An unmet check answers HTTP `200`, null
    `data.enqueuePullRequest` and one `UNPROCESSABLE` error with message
    `Pull request Required status check "<name>" is in progress.`; no
    entry is admitted. Missing, pending and failed head checks use this
    refusal. Group CI is separate: after admission, a status posted on
    the `gh-readonly-queue/` commit is accepted, and ci success lets the
    queue merge. The GitHub layer checks admission before the shared
    domain's enqueue; the domain's own queue admission remains its
    approval check.

  The layer's `pull` document answers `mergeQueueEntry` and
  `mergeCommit.oid` from the domain for queued, direct and repeated merges.
  Every REST pull object omits `merge_commit_sha`, as at the pinned
  2026-03-10 version, whether open or merged. Every merged pull read
  therefore costs the additional GQ read.
- **Rulesets and classic protection.** The scenario says how the
  domain's protection shows (`Shown::{Ruleset, Classic, Both}`):
  - as a ruleset, with its id and the App's `current_user_can_bypass`;
  - as classic protection, otherwise `404` "Branch not protected";
  - or both.
- **Rates.** Every answer on a token carries `x-ratelimit-*` from the
  domain's window: `rate_limit`, what remains of it, and its end. The
  App JWT calls carry none. A call refused for the rate (`limited`)
  is `403` with `x-ratelimit-remaining: 0`. A crowded one is `403` "You
  have exceeded a secondary rate limit" with `Retry-After`. A request
  with an `If-None-Match` that matches the answer's `ETag` is `304`, and
  not counted. A repository ETag hashes the rendered metadata and settings.
  Conditional repository reads check the domain's current settings before
  reusing the identity's rate snapshot for a 304; a changed response goes
  through normal admission. The snapshot map is bounded by `github.inner`.
  GitHub's `Config` sets the quota and window. Admission
  charges the domain once per authenticated request, including local table
  reads; continuations keep the snapshot and skip further rate charges.
  Installation and domain-user windows have distinct keys in the same bounded
  table. Person tokens charge their user's window; installation tokens charge
  their installation's. Admission resolves a token once to its bounded store
  slot or person-table index and carries no credential bytes. Revocation removes
  the admitted slot only after a successful charge; rate refusal leaves it in
  place. Tokens on App routes are charged before their credential-class refusal.
  The primary message "API rate limit exceeded" is inferred and
  unrecorded; `texts.rs` marks it. Both rate refusals supply `Retry-After`.
- **Faults** are `forge.md`, section 4's, at the bytes, with GitHub's statuses. Built:
  `Config::faults` uses the Forgejo face's `Faults` type. Reset closes before
  reading the next request, including on reuse. Drop and server-error draws
  follow the final domain continuation, so an applied write survives the
  lost answer or `500`. A seed replays the draws.

### 1.2 What the domain gains

These are small:

- **Protection** gains `queue`, `linear`, `code_owners` and a list of
  bypassing users. A bypassing user's merge skips the checks and the
  approvals. The domain does not sign, so it keeps no signatures rule:
  the layer shows `required_signatures` from its `Config` and refuses a
  push as `GH013` when the scenario asks for it. Its text is the one
  `signatures-push-gh013` records (../../forge/github.md, section 13).
- **The merge queue.** `Write::Enqueue { number, head }` refuses a moved
  head, as a merge does. An entry's group commit is the squash of its
  head onto the base's tip, or onto the entry ahead of it. That commit
  becomes the head of `gh-readonly-queue/<base>/pr-<n>-<head>`, with `<head>`
  in `forge.md`, section 4's SHA-1 spelling, so the domain's CI runs on it as on any head.
  Every queue group is modelled as a squash, regardless of merge_method.
  Its message follows
  COMMIT_OR_PR_TITLE: one commit relative to the pull's base uses that
  commit's message, more than one uses the pull's title. The GitHub
  repository response advertises squash_merge_commit_title as
  COMMIT_OR_PR_TITLE, and branch reads expose the stored commit message:
  - when it passes, the base moves to it and the pull request is merged;
  - when it fails, the entry leaves with the reason `CI_FAILURE`;
  - entries behind a failed one are built again.

  Each leaving is a hook (`dequeued`, and the group's `destroyed`).
- **Installations' events.** Hooks for an installation made, removed,
  suspended, given repositories or renamed, released by the layer's
  store.

### 1.3 Deliveries and their log

Built in the GitHub layer: installation lifecycle and rename events, domain
queue action snapshots, the loopback HTTP sender that retains the recipient
status, the bounded signed-delivery store, seeded UUIDs
and increasing 19-digit ids, visibility delay, retention and capacity
removal, cursor pages honoring `per_page`, whole-delivery reads, redelivery,
first-page ETag/304, and key revocation. The server-side route decoder and
log encoders live in `skein_forge::github::server`. The scenario's
`Config::deliveries` and `Limits::deliveries` admit these bounds before
allocation. `payload_size` enlarges text-bearing issue, comment and review
payloads to exercise consumers' oversize refusal.


- **Every delivery is logged.** A domain hook is rendered as GitHub's
  payload, with `X-GitHub-Event`, `X-GitHub-Delivery` (a UUID drawn from
  the seed) and `X-Hub-Signature-256`, and is entered in the layer's log.
  Each entry has an id of 19 digits, starting at 10^18 plus a seeded
  offset, and increasing.
- **Pushing is the scenario's choice.** By default the hook's URL does
  not resolve, as temper's App's does not: no delivery is written, and
  each is logged as failed, `status_code` 502. Pushing writes it over
  loopback to the configured endpoint, as `forge.md`, section 5 says, and logs the
  answer's status.
- **The log** is `GET /app/hook/deliveries`, newest first, paged by an
  opaque `cursor` in `Link`. The first page's `ETag` changes when an
  entry is added, so a first page asked with it unchanged is `304`.
  `GET /app/hook/deliveries/<id>` answers the delivery whole: the
  request's headers and the payload, re-encoded, and the response. An
  entry is listed `log_min..=log_max` after its change (default `1.5` to
  `2.2` seconds, as on github.com), and kept for `log_keep` (default three
  days) or until `log_entries` is full, oldest first out.
- **No installation, no repository** in the list for `ping`,
  `installation` and `installation_repositories`, as on github.com.

### 1.4 Where it differs from github.com

These are contracts: a story that leans past one of them is wrong, not
the fake.

- **Only the dialect's routes and documents** are served. Others are
  `404`, and other GraphQL documents are `errors`, including bodyless
  `HEAD` refusals and `OPTIONS`; representative unknown routes and documents
  have a table test.
- **Plaintext, on loopback,** at three origins: `api`, which also
  serves `web`'s routes, `blob`, and an optional third one.
- **Messages** follow the conformance's recorded texts for the refusals of
  ../../forge/github.md, section 7, and the live records of 1.1, which marks
  the inferred approval counts and queue context lines.
- **Merges** follow the domain: squashes, whatever the method asked,
  and a queue's group is built from the entry ahead, not batched.
  While a head or base move is still checking mergeability, the GitHub
  route answers `405` "Pull Request is not mergeable"
  (`merge-after-move-not-merged`, ../../forge/github.md, section 7); GitHub
  may also answer the recorded 409 just after a move, with no merge.
  An open pull's stale supplied SHA keeps its `409` precedence. A merged
  pull's repeat merge answers `200` for any supplied SHA.
- **Ids** are the domain's numbers, small and sequential. Deliveries'
  ids are 19 digits, so `u64` is held exactly.
- **What is not modelled:** GitHub Apps other than the one App and
  `github-actions`; forks; organizations' members and teams; signed
  commits; workflows; any secondary limit beyond `crowded`; a review's
  inline comments, which the domain does not store: a pull's review
  comments read as an empty list.

### 1.5 The peer and its tests

- **The forge peer hosts it** (`forge.md`, section 5) with
  `Peer::github(address, blob, limits, ..)`. It works as the Forgejo
  face does, over a forge it is handed, with the same rules of sharing.
- **Step tests** cover each route, each refusal and its message, the
  mint and the expiry, the JWT's window, the browser's routes, the git
  route's acceptance and each of its refusals, the queue
  through to landing and to ejection, the log's paging and its `304`,
  and a delivery decoded by `skein_forge::github` alike pushed and
  logged.
- **Memory** is checked as `forge.md`, section 8 says, with the stores full. Built in
  `suite::github_memory` tests under `tests/fake-forge` and
  `tests/fake-peers`: installation rows and listed reach, minted and person
  tokens, App grants and events, manifest states and blob records are full,
  with a queued pull in every domain repository. The hosted test fills both
  connection slots and checks the owner's measured peak at each iteration
  against the ceiling its scenario names, with headroom (../../runtime/shell.md, 11);
  the simulator and client are metered outside the owner's calls.
  The API/browser and blob listeners share the peer's connection limit
  and io buffers; admission reserves two listener sockets. Each accepted
  connection uses the same bounded HTTP envelope. The scenario's ceiling
  also covers the blob record slots and paths, seed key, configured origins
  and signed-target scratch.
- **The budget** is the focused suite's, as `forge.md`, section 8 says. The conformance
  against the fake (`github.md`, section 2) runs there as one test, within 6 seconds (3
  seconds before the user's doubling of 10-22, provisional). The fake's
  clock carries the CI's and the queue's waits, so nothing sleeps.

### 1.6 Without a peer

Each of the forge component's machine, joint and protocol worlds plays its
own wire, so no peer stands between them and the layer. They meet the `Server` the peer hosts all the same, one fake at every
tier (`skein/docs/foundation/testing-strategy.md`, 4), through
`skein_fake_forge_github::inline`, which hosts it in the caller's turn, over
a domain the caller lends, with no io.

- **A domain lent for a pass.** `Lent` holds the domain, the clock it runs
  on and whether due hooks are released ahead of each call. A call settles
  at zero latency: `call` steps it as admitted and fires the domain until
  its reply. `release` fires what is due, hands each hook to `Service::hook`
  and polls the service. A world calls both directly for a fixture's or a
  person's call.
- **A connection driven.** `pass` drives one `Server` the world has started
  and fed: each `Event::Rate` through `domain::rate` and `server::admitted`,
  each `Event::Domain` through `call` and `server::down`, and the server
  resumes, until it waits on bytes or holds a head. So a request's admission
  and its continuations finish inside one pass, as `Service::request`
  requires. `Close` and `Closed` ask nothing of a pass: the stream is the
  world's, and the world ends the connection. It moves `Up`s and `Down`s
  over that stream, and takes each head between passes.
  `take_head` blanks the credential as it does for the peer, and
  `Server::credential` lends its value until the next pass, for a tally
  that counts a request under the identity it presented.
- **A request in hand.** `Inline` holds a service, a domain and their
  bounds, for a world with a whole request that wants a whole response.
  `serve` and `blob` each run one request through a fresh `Server`, of the
  API's listener and of the blob listener, and return the response's bytes.
  A cut, a pause or a lost reply is the world's, on those bytes. A request
  is what the HTTP server admits: an HTTP/1.1 request without `Host` is
  refused `400`, as a real server refuses it.
- **Media types.** `Server` names the media types github.com does, with
  `; charset=utf-8`, on the peer's connections and in `Inline` alike. Bytes
  of a blob are `application/octet-stream`, as they stand.
- **What a scenario turns.** `Service::ambiguous_updates` answers an
  update's conflict with the stale head's refusal, the one spelling
  ../../forge/github.md, 5.2 reads as ambiguous. `Installations::minted_token`
  lends a live token by its mint number, so a world that checks for leaks
  holds every token the layer minted, a reply lost on the wire included.
- **The fixture.** `skein_fake_forge_github::fixture` is the scenario a
  world starts from, built once: the App (`APP`, `test-app`, the first
  public key, no grants) and its bot (`BOT`), the accounts `acme` and
  `alice`, `acme`'s repositories (`repositories`, or `widgets` alone), a
  `Config` and `Limits` over them, their installations (`install_one`,
  `install_three`), the domain's zero-latency `domain_config` and a
  repository's `setup`. A world names what it varies by struct update, and
  no more. A scenario's own accounts and repositories, such as the
  conformance's, stay its own tables inside a fixture `Config`. The domain's
  branches, pulls and commits are the world's: they are what its stories
  read.

### 1.7 Its state, kept

Beside the domain's image (`forge.md`, 2.7), for an owner that keeps the
fake across restarts (`forge.md`, 5): `Service::image(&self) -> Box<[u8]>`
and `Service::restore(&mut self, bytes: &[u8]) -> Result<(),
image::Refusal>`, in the layer's own format, on a service built from the
same configuration.

- **What it keeps:** the App as it stands, its key's revocation and its
  permissions, events and webhook secret; the accounts, the signed-in
  person and the person's tokens; the repositories' metadata and
  protection's display facts; the installations, their reach,
  suspension and times, each minted token with its grant, its expiry on
  the wall clock and its mint number, and the counters; the browser
  pages' store, its manifests, codes with their expiry and use, the
  setup URL and the key's rotation; the delivery log, its entries,
  signatures, visibility times and ids; the job logs' signing key and
  origins; and each of the layer's `Rng` streams' state, so that ids
  and tokens minted after a restart are those it would have minted.
- **What it drops:** what lasts a connection or a launch: posts queued
  and not yet delivered, which are lost as a delivery that cannot be
  written is (`forge.md`, 5); the git-call records; the rate headers'
  snapshots and the JWT verification cache, which fill again; the job
  logs' request records.
- **Times on the wall clock** stay as they were: a token or a code
  whose expiry passed while the fake was stopped is expired after it,
  as github.com's would be.
- **A refusal,** an image that does not decode or holds past the
  layer's limits, names what, and the service is left as it was built.

## 2. GitHub's conformance

`tests/forge/github` is a tool beside `forge.md`, section 7's. It runs the same
cases against two targets and records each fact of ../../forge/github.md,
section 13, as `holds`, `fails` or `unchecked`. Its transcripts, facts,
masking and refusal of leaked secrets are `forge.md`, section 7's.

- **The fake** (`--fake`, the default) is the GitHub layer in-process,
  hosted by the forge peer. It also runs as one test in the focused
  suite, which refuses failed facts and unexpected unchecked facts.
- **github.com** (`--live <directory>`) is opt-in. It runs from the
  user's session alone, and only on the user's word. It is never
  run by a suite or a gate, and never from CI.
- **A fact that holds on github.com and fails on the fake** is a finding
  for the fake. One that fails on github.com is a finding for the
  dialect, and its pin moves only after it is understood (../../forge/github.md,
  section 12).
- **The fixtures' directory** is the person's
  (a private directory outside any repository, by its README), named by
  `--live`. The tool reads no variable, and refuses a directory that is
  not `0700` or a file that is not `0600`. It holds:
  - `app.json`: the probe App's id, slug, permissions, events and
    installations;
  - `app.pem`: the App's key;
  - `webhook_secret`: checks logged deliveries' signatures;
  - `person-<owner>.token`: one fine-grained token per owner, for any
    number of owners, found by name. Each is limited to that owner's
    probe repositories.
- **The fixtures:** an App installed on a user-owned and an organization-owned
  probe repository, plus an initially ungranted spare. A run adds the spare
  to the installation and removes it again. Each probe repository carries
  the ci workflow specified in ../../forge/github.md, section 16: push, pull_request
  and merge_group triggers, job ci, 45 seconds, and a FAIL head-message cue.
  The dequeue case requires COMMIT_OR_PR_TITLE and two passing head commits
  with FAIL only in the title; it reads and records the repository setting
  and changes no repository setting.
- **Queue installation discovery.** Before minting or repository changes,
  the startup repository reads select the first installation whose probe
  has `owner.type: Organization`. Every queue case uses its probe repository,
  installation token and matching person token. A run with no such account
  stops before any write (../../forge/github.md, section 16).
- **Who does what:**
  - The App's key and minted tokens drive the dialect's cases.
  - The person's tokens drive the person's part: rulesets, approvals,
    and an installation's repositories added or removed. They cannot
    list an installation's repositories, so the App's token reads them.
- **Expiry.** Each run reads each person's token's
  `github-authentication-token-expiration` and names, in `facts.txt`'s
  provenance, any token that expires within 30 days.
- **Secrets** never leave the directory. The tool registers the key,
  every token, every JWT, every minted token and the webhook secret
  before the first request. It copies none, and it masks
  `Authorization`, signed blob addresses, guids, node ids and ETags as
  well as `forge.md`, section 7's. Numeric ids are masked by first appearance, since
  github.com's do not come in a fixed order. Request-path numbers use the
  same map as body numbers. Every response keeps its status and headers;
  transcript records are separated by CRLF so later status lines start a line.
- **The App is read only, live.** The fixture installations are
  accepted by no setup state, so setup's reconcile would take them for
  strays (../../forge/setup/github.md, section 1). A live run therefore never sends
  `DELETE /app/installations/<id>` or `PATCH /app/hook/config`, makes
  or converts no manifest, and changes nothing of the App: it reads
  the App's documents alone. The tool refuses those routes on the live
  target before any byte is written. Strays, the uninstall and the
  hook's address being set are checked on the fake alone, and their
  facts are `unchecked` live (../../forge/github.md, section 13).
- **Access is put back.** A repository the run adds to an installation
  or removes from it, with the person's token's `204`, is restored at
  the run's end. Each such change is written to `made.txt` before its
  request, so `--clean` restores it too.
- **A run cleans up what it made:** rulesets, pull requests (closed),
  issues (closed), branches, the spare's grant, and minted tokens
  (revoked). Each thing made is first written to `made.txt` in `--out`,
  and `--clean <out>` finishes a run that stopped partway. Each successful
  issue or pull creation also records its returned number durably; cleanup
  addresses that number. A refused creation cancels its intention without
  marking an object done. When a killed run retained only an intention,
  cleanup follows every issues-list page, and reads pull details when
  matching head and base. An unfound object remains pending. An item is
  marked done only after a read shows it closed. Cleanup mints a journaled
  installation token for the repository's lookups, close, and read-back;
  branch deletion uses the person's token. The cleanup tokens are revoked
  after restoration, including when restoration fails. Minted tokens
  stay in memory and are revoked on every controlled exit; after an
  externally killed process, cleanup reports each unacknowledged mint's
  installation and one-hour expiry in `facts.txt`, without recovering
  or writing its value.
- **Calls are few.** A full live run stays under 400 requests and about
  ten minutes, most of it waiting for the probe workflow's 45 seconds.

## 3. What the two layers share

Forgejo's layer (`forge.md`, section 4) and GitHub's (`github.md`, section 1) serve different
APIs from one domain, and speak HTTP the same way. What they have in
common sits in `skein-fake-forge-http`, which depends on skein-lib,
skein-http, skein-json, `skein-forge`'s neutral root (../../forge/README.md, 4.4) and
the domain. It names no item of either dialect, and neither layer names
the other's crate: a change to one fake reaches the other only through
this one. Forgejo's layer is the crate `skein-fake-forge-forgejo`, which
says what it is.

- **One connection's core.** `connection::Connection` holds what a
  connection is whatever it serves: its state, skein's HTTP server, the
  queues between them, the captured head and the reply in hand. The core
  admits an event by the state, captures a head with its `Authorization`
  blanked, streams a reply within the send limit, and closes. It decides
  nothing about a request.
- **A server keeps its pass.** Each layer's `Server` holds a `Connection`
  and runs its own bounded pass: it asks the core for a `Turn`, one
  event at a time, and answers it by its own routing. So a layer keeps
  its events, its body's reading, its headers, its fault draws and its
  order of work, and the core draws no chance and calls no one.
- **The git route** (`forge.md`, section 4) is typed here: `git::Limits` holds the
  HTTP server's limits and the documents' (`skein_forge::limits::Limits`),
  all the route reads. `git::status` answers a domain error's status
  on the route. Each layer serves the route through its own entrance,
  with its own credential rule (`forge.md`, section 4, and `github.md`, section 1.1).
- **`Hosted` and `Faults`,** the forge lent for one pass and the byte
  faults' chances, are the layers' alike, so they sit here and each
  layer's `Config` names `Faults`.
- **What stays apart.** Each layer's own: its `Service`, its routes, its
  answers, the spelling of a reply's head, and whether a status carries a
  body. A commit's 32 bytes are the domain's (`commit_bytes`,
  `commit_count`, beside `object_bytes`), which both layers read.
