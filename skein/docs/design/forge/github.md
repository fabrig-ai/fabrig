# GitHub's dialect

Provisional, 2026-10-20. `skein_forge::github`: GitHub's documents and
call plans, beside Forgejo's (`skein_forge::forgejo`), holding what
dialects.md 1.1 says a dialect holds. It renders README.md's vocabulary on
github.com's REST API at one API version, with three GraphQL documents
for what REST lacks, and adds to the vocabulary what GitHub needs
(vocabulary.md, section 1, and README.md, section 8). The component that runs it is
connection.md's; the setup plans that run its setup documents are
setup/README.md's; the fake's GitHub layer and GitHub's conformance are
../testing/fakes/github.md's, sections 1 and 2.

## 1. In one page

- **A module of `skein-forge`,** `skein_forge::github`, with no io, no
  clock and no connection: encoders, decoders, call plans, the
  failures' map, deliveries' decoders, and the App's documents (the
  JSON Web Token's bytes, the mint, installations, the delivery log).
- **Pinned to an API version,** `X-GitHub-Api-Version: 2026-03-10`, on
  every request (section 3). GitHub is a service, not a release: the
  pin is the version and the date the conformance last held, and a fact
  that changes is a finding (section 13).
- **REST, and three GraphQL documents:** the bot's identity (`viewer`),
  entering the merge queue (`enqueuePullRequest`) and a pull request's
  landed commit and queue entry (`mergeCommit`, `mergeQueueEntry`).
- **Every call of the vocabulary is answered** (`github-operations.md`, section 1), with
  GitHub's shapes told in each decoder: pull requests among issues,
  `merged` absent from listings, `mergeable` null until computed, pages
  by `Link`, CI from check runs and statuses read apart.
- **A merge tries REST first** (`github-operations.md`, section 2): `PUT .../merge` with `sha`,
  and on GitHub's merge-queue refusal it enters the queue with
  `expectedHeadOid` and answers `Queued`. A moved head is `Stale` on
  either route. The plan keeps no state, so it holds when a branch's
  rules change after adoption.
- **Protection is read from rulesets and classic protection,** both,
  and neither's absence is taken for the other's; it says whether the
  branch queues, wants signatures, a linear history or code owners, and
  whether the App may bypass.
- **Refusals are told apart by status and GitHub's whole message,**
  since several share a status (section 7); secondary limits are
  `RateLimited`, and every answer's `x-ratelimit-*` is the rate's
  window (`api::Rate`).
- **The App's documents:** the JSON Web Token's signing input and its
  assembly (the signature is the component's, `skein_tls::sign_rs256`),
  the mint of an installation's token, the installations and their
  repositories, and the start's identity read (`github-app.md`, section 1).
- **The delivery log is the hint feed:** its list page, with a cursor
  and a validator, and one delivery whole; a pushed delivery and a
  logged one decode to the same `github::Delivery` (section 9).
- **A job's log is a redirect,** parsed into `Redirect { origin, target
  }`; the component follows it to an origin it knows (section 10).
- **Setup's documents:** the manifest, the form, the install page, the
  conversion, the App's own reads and its hook (`github-app.md`, section 2). The plans
  that run them are setup/README.md's.
- **Ids are exact,** `u64` in decimal; a delivery's id has 19 digits.

## 2. What the module holds

By dialects.md 1.1's list, what holds for both dialects alike sitting at
the crate's root (dialects.md, 1.4):

- **Each request,** as a sized encoder, and **each answer,** as a
  decoder over skein-json's tokens with a bounded stack, keeping only
  its fields (`github.md`, section 3, and `github-operations.md`, section 1, and `github.md`, section 6).
- **The server's side of each document,** for the fake's GitHub layer:
  the decoder of each request and the encoder of each answer
  (../testing/fakes/github.md, section 1).
- **The failures** and the map from a status and a message to an
  `Error` (section 7).
- **The deliveries:** the headers, the events, each payload's decoder
  and encoder, the table to hints, and the log's documents (section 9).
- **The setup's documents** (`github-app.md`, section 2).
- **Names, times and ids** (section 3).
- **How a credential renders:** `Authorization: Bearer <t>`, for a
  token, an installation's token and the App's JSON Web Token alike.
- **The start's reads:** the identity read (`github-app.md`, section 1.4); the page cap
  is GitHub's constant 100 (`PAGE_CAP`), so no read is made for it.
- **The renewal:** the mint's request and decoder, and the JSON Web
  Token's bytes (`github-app.md`, section 1). When it is due, and which installation, is
  the component's (connection.md, section 5).
- **The pin** (section 12), **the call plans** (`github-operations.md`, sections 1 and 2) and
  **its limits,** `github::Limits`, with dialects.md 1.1's fields; `check`
  refuses a zero, a `json.string` below `string_bytes` or `body_bytes`,
  and a depth below 10 (a logged delivery wraps its payload in two more
  levels than a pushed one).

The plans hear `github::Head`: the status and the header fields `Date`,
`Retry-After`, `Link` (whether it has `rel="next"`, and that link's
`cursor`), `ETag`, `Location`, `Content-Type` (JSON or not),
`x-ratelimit-limit`, `x-ratelimit-remaining`, `x-ratelimit-reset`,
`x-ratelimit-resource` and `Connection: close`. Each else as dialects.md
1.2 says: `Row`, `Done` with the body, or `Failed` (`Unseen`, `Seen`).

The plans' tables are the module's own except for the root's logins'
table (dialects.md, 1.4); the component keeps and hands them as it does
Forgejo's (dialects.md, 1.2):

- **A repository's row:** its `owner` and `name` (its path), its
  owner's account id (by which the component chooses the installation,
  connection.md, section 5), whether that owner is an
  organization, the merge method (`github-operations.md`, 2.1) and its `html_url`. Object ids
  are SHA-1's, 40 digits: GitHub has no other format.
- **A user's row:** an id and its login.

## 3. Conventions

- **Base and transport.** Every request goes to the forge's `api`
  address, by default `api.github.com:443` over TLS, at the paths below
  with no prefix; GraphQL is `POST /graphql`. The fake's GitHub layer
  answers the same paths in plaintext on loopback (../testing/fakes/github.md,
  section 1). `Host` is made by `skein-forge`'s `host` (forgejo.md, 1.1).
- **Every request's head** carries `Accept: application/vnd.github+json`
  (a raw file's `application/vnd.github.raw+json`),
  `X-GitHub-Api-Version: 2026-03-10`, the owner's `User-Agent`
  (GitHub refuses a request with none), and `Authorization: Bearer <t>`
  but where a row says no credential. `skein-forge`'s owner gives a
  `User-Agent` that is not empty, checked at start.
- **JSON in and out,** as forgejo.md 1.1. A GraphQL request is
  `{"query": <document>, "variables": {...}}`, the document a constant
  of the module (`github-operations.md`, section 1.3).
- **Repositories by id.** R is `/repos/{owner}/{name}`, from the
  repository's row. A call whose repository has no row resolves it
  first with `GET /repositories/{id}`, which counts in its cost.
- **Names in paths** are percent-encoded per segment. A branch keeps its
  slashes as the path's: GitHub takes the rest of the path as the
  branch (`branches/{branch}`, `git/refs/heads/{branch}`), recorded as
  `branch-with-slash-reads` (section 13). A query's values are
  percent-encoded whole (`head=<owner>:<branch>`). A bot's login in a
  path is `<slug>%5Bbot%5D`.
- **Times** are RFC 3339 in UTC (`2026-10-20T08:00:00Z`), read to the
  second, as the forge's clock. `since` is written so, up to 2554 as
  forgejo.md 1.1 says. An answer's time is its `Date`.
- **Commits** are 40 lowercase hexadecimal digits, carried as README.md's
  32 bytes, zero-padded.
- **Ids** (users, repositories, comments, reviews, check runs, jobs,
  runs, installations, deliveries, the App) are decimal `u64`s, read
  exactly; a number past `u64` is a malformed answer. A repository id
  past the vocabulary's 32 bits fails the call `TooLarge`, as Forgejo's
  (section 15).
- **Paging is explicit:** `per_page=<size>&page=<n>`, the size the
  smallest of the client's `rows`, `PAGE_CAP` (100), and the number of
  the listing's largest rows `answer_bytes` holds (forgejo.md, 1.1), but
  for pages sized for real rows: `Items` (one row when `single`), an
  item's comments, `Reviews`, `Remarks` and `PullFiles` ask the smallest
  of `rows` and `PAGE_CAP`, and keep each row whole while it fits
  `answer_bytes`, cut before the first that doesn't, as forgejo.md 1.3
  says (a file with the sides asked; comments end with `more`). A
  listing has more when its `Link` has `rel="next"`, or when cut; a page
  that comes full without one has none. `Reviews`, `Remarks` and `PullFiles` pass
  their page's first `skip` rows as they stream (`vocabulary.md`, 1.2). The
  delivery log pages by cursor (9.3).
- **Users by id,** with the login from the users' table, filled by `GET
  /user/{id}` (`user-by-id-reads`), one request in the cost.

## 4. Each operation

Moved to [`github-operations.md`](github-operations.md), section 1.

## 5. Merging, updating, and the rules

Moved to [`github-operations.md`](github-operations.md), section 2.

## 6. Decoding answers

As forgejo.md 1.3: streams, strings kept whole or not, the answer's
bound, revisions, markers and malformed answers; and GitHub's:

- **Provenance** from `created_at` and `updated_at` (a review's
  `submitted_at` and none: a review is `Original` unless its body was
  not kept), at a second's resolution (`comment-updated-at-on-edit`).
- **Raw bodies cut as they stream.** A file's first `max_bytes` bytes
  and a log's last are kept within what is left of `answer_bytes`, and
  the rest read through within `body_bytes_in`
  (connection-machines.md, section 1); `truncated` says so.
- **Absent headers.** An `Items` answer without `Date` is malformed. A
  listing without `Link` has no next. An answer without
  `x-ratelimit-*` carries no `Rate`, as the JSON Web Token's never do.
- **Nulls** read as absent: a body, a fork's head repository, a
  `mergeable`, an `action`, a `line`.
- **GraphQL's ids** (`databaseId`, `position`) are JSON numbers, and its
  node ids opaque strings within `string_bytes`, sent back as they came.

## 7. Failures

| What happened | `Error` |
|---|---|
| nothing written, as forgejo.md 1.4 | `Unavailable` |
| a `301` or `307` on a repository's route (renamed or moved): the row dropped | `Unavailable` |
| a `403` or `429` with `x-ratelimit-remaining` 0, before any write succeeded | `RateLimited { after }`: `Retry-After` when given, else `x-ratelimit-reset` less `Date` |
| a `403` or `429` "You have exceeded a secondary rate limit", or with `Retry-After`, before any write succeeded | `RateLimited { after }`: `Retry-After`, else 60 seconds |
| on `SetReviewers`, once its addition succeeded: nothing heard, a `5xx`, a malformed answer or a rate limit on the removal | `Timeout`, no rollback; a typed refusal stays final |
| after a byte was written, as forgejo.md 1.4, a `5xx` included | `Timeout` |
| a malformed answer (section 6) | `Timeout` on a write; `InvalidAnswer` on a read |
| `401`, on a token or the JSON Web Token | `Forbidden` |
| `403` "Resource not accessible by integration", or any other `403` | `Forbidden` |
| a `404` or `410` on a branch leaf (`Read::Branch`, `Stage::Read`) or contents leaf (`Read::File`, `Stage::Raw`), classified as `Asked::Leaf` | `Missing`, `Drop::Nothing`: the repository row stays |
| every other `404` or `410` that ends the call, including repository resolution by id, its own GET and listings (`Asked::Read`) | `Missing`, the row dropped (`Drop::Row`); `MissingJob` on `Job`'s |
| `409` on `Merge`, "Head branch was modified. Review and try the merge again." | `Stale` |
| `405` on `Merge`, "Pull Request is not mergeable" | `Conflict` |
| `405` on `Merge`, "Repository rule violations found" and the merge queue's line | the queue (`github-operations.md`, 2.1) |
| `405` on `Merge`, "Repository rule violations found" and any other line | `Protected` |
| `405` on `Merge`, "... merges are not allowed on this repository." | `Refused`, the row dropped |
| GraphQL `UNPROCESSABLE`, "... expected head oid does not match the current head oid" | `Stale` |
| GraphQL `UNPROCESSABLE`, "Pull request Required status check ..." on `enqueue` | `Protected` |
| any other GraphQL error on `enqueue` | `Refused` |
| `422` on `Update`, "expected head sha didn’t match current head ref." | one pull read: moved head `Stale`, unchanged open head `Conflict`, closed `Closed` (`github-operations.md`, 2.2) |
| `422` on `Update`, "merge conflict between base and head" | `Conflict` |
| `422` on `OpenPull`, "No commits between ..." | `NothingToMerge` |
| `422` on `OpenPull`, "A pull request already exists for ..." | `Exists` |
| `422` on `Review`, "Can not approve your own pull request" | `Refused` |
| `422` on a write whose title or body is empty | `Empty` |
| `422` on `CreateBranch`, "Reference already exists" | `Branch(Exists)`, an answer |
| `422` on `DeleteBranch`, "Reference does not exist" | `Missing` |
| a pull request's head other than `PullFiles` names | `Stale` |
| an answer past the component's bounds; a write's text not UTF-8 | as forgejo.md 1.4 |
| any other `4xx` | `Refused` on a write; `InvalidAnswer` on a read |

- **The message is matched whole,** by line: GitHub's `message`, and a
  `422`'s string entries in `errors` or objects' `errors[].message`,
  against the texts the conformance records
  (section 13). Enqueue's required-check refusal matches the prefix
  "Pull request Required status check " only for `UNPROCESSABLE`.
  A rule violation's message is a first line,
  "Repository rule violations found", then one line per rule. An
  unknown message is `Refused` on a write, `InvalidAnswer` on a read.
- **A merge just after a move** may answer `405` "Pull Request is not
  mergeable" while GitHub computes, or `409` "Head branch was modified.
  Review and try the merge again." (`merge-after-move-not-merged`). The
  dialect says `Conflict` for the former and `Stale` for the latter; the
  owner reads the pull again. The client reads again within `settle`
  before taking the former for a conflict (client.md, section 5).
- **A `401` on the JSON Web Token** is the App's key refused (deleted,
  or a clock more than a minute off): `Forbidden`, which the component
  takes to stop the forge (connection.md, section 5).
- **Repeating a successful merge with the same sha** answers 200 with the
  same merge sha and merged true. A different sha on that merged pull is
  inferred to be 409; the fake keeps the stale-head check before mergeability.
- **A `304`** is not a failure: it answers a conditional read (`github-app.md`, 1.5).

A repository gone or outside the installation loses its row at its next
repository-level call. An absent leaf alone does not invalidate the
repository that answered it.

## 8. The App's documents

Moved to [`github-app.md`](github-app.md), section 1.

## 9. Deliveries

### 9.1 Pushed

As README.md section 6, in GitHub's words:

- **At `/hooks/<number>`,** signed `X-Hub-Signature-256: sha256=<hex>`,
  HMAC-SHA256 of the body with the App's webhook secret, verified as it
  streams; the event in `X-GitHub-Event`, the delivery's guid in
  `X-GitHub-Delivery`. The verdicts are README.md 6's.
- **Not yet reached:** GitHub delivers from the internet, and temper's
  App points its hook at an address that does not resolve (temper's
  `draft/github-app.md`, 1). Every delivery fails and is listed in the log, which the
  component reads (9.3). A public address is later (temper's `draft/archive/github.md`, 18);
  the fake's layer pushes to a listener when a test asks (../testing/fakes/github.md,
  section 1).

### 9.2 One decoder, two ways in

`github::delivery::decode(event, body)` reads one payload into a
`github::Delivery`; a pushed body and a logged delivery's
`request.payload` give the same. The guid is a UUID's text, read into
`[u8; 16]`; another text is malformed. The App receives `installation`,
`installation_repositories` and `installation_target` without
subscribing; the others are `EVENTS`' (`github-app.md`, section 2), and no more.

| `X-GitHub-Event` | `github::Delivery` | `Hint` |
|---|---|---|
| `issues` | `Issue` | `Item(issue.number)`, the key of the issue's body |
| `issue_comment` | `Comment` | `Item(issue.number)`, the key of the comment's body |
| `pull_request` (each action, `enqueued` and `dequeued` with its `reason` among them) | `Pull` | `Item(number)`, no key |
| `pull_request_review` | `Review` | `Item(pull_request.number)`, the key of the review's body |
| `pull_request_review_comment` | `Remark` | `Item(pull_request.number)`, no key |
| `push`, of a branch (`refs/heads/`) | `Push` | `BranchMoved { branch, head: after, actor: sender.id }`; `deleted` true, `Branch(branch)` |
| `create`, `delete`, `ref_type` `branch` | `Ref` | `Branch(ref)` |
| `check_run` and `check_suite` `completed`, `status`, `workflow_job` `completed`, `workflow_run` `completed` | `Ci` | `Commit(head_sha)`; a status's `sha` |
| `merge_group` (`checks_requested`, `destroyed`) | `Ci` | `Commit(merge_group.head_sha)` |
| `installation` (`created`, `deleted`, `suspend`, `unsuspend`) | `Installation` | none: the component's (below) |
| `installation_repositories` (`added`, `removed`) | `Installation`, `Repositories` | none |
| `installation_target` `renamed` | `Installation`, `Renamed { login }` | none |
| `ping`; a tag's `push`, `create` or `delete`; any other | `Other` | none |

- **The repository** is the payload's `repository.id`, past 32 bits no
  hint; the forge, the component's number for it.
- **`Installation { installation, account, change }`**: the
  installation's id, its account's login and id, and the change:
  `Created`, `Deleted`, `Suspended`, `Unsuspended`, `Repositories`
  (with no names: the owner reads them again, `github-app.md`, 1.3) or `Renamed {
  login }`. The component makes them `Event::Installation`
  (connection.md, section 5).
- **Bounded.** A push's `commits`, a pull's diff fields and every
  repository field but its id, owner's login and name are skipped as
  they pass. The login is at most 39 bytes and the name at most 100;
  either past its bound is malformed. The decoder can hand out their
  path when both are present; id-only bodies remain valid.

### 9.3 The delivery log

- **A list page:** `GET /app/hook/deliveries?per_page=<n>`, with
  `&cursor=<c>` past the first, on the JSON Web Token, newest first.
  Each row, `github::log::Row`: `id` (19 digits), `guid`, `event`,
  `action`, `installation_id` and `repository_id` (none for `ping`,
  `installation` and `installation_repositories`), `status_code`,
  `delivered_at` and `redelivery`. The next page's cursor is the
  `cursor` of `Link`'s `rel="next"`, a `github::log::Cursor` of at most
  128 printable bytes; none when there is no next.
- **The first page is conditional:** sent with the last page's
  `Validator` as `If-None-Match`, a `304` says nothing new, at no cost.
- **One delivery whole:** `GET /app/hook/deliveries/<id>`, on the JSON
  Web Token. The plan carries the listed row's bounded event word and
  starts the payload decoder from it, whatever the JSON member order.
  Its `event` and `request.headers.X-GitHub-Event` must both be present
  and equal the row's event, or the read is malformed. The decoder reads
  `request.headers` (`X-GitHub-Event`, `X-GitHub-Delivery`) and
  `request.payload` as a pushed body is (9.2); its `response` is skipped.
  The logged delivery carries the payload's repository path when both
  its owner's login and name are present, and none for a kind with no
  repository.
- **Trusted by its channel,** not its signature: the payload comes back
  re-encoded, so no HMAC is checked; the read is the App's own, over
  TLS, on its token.
- **The puller** (which guid is newest, `log_poll`, `log_catch_up`) is
  the component's (connection-webhooks.md, section 1).

## 10. A job's log

- **`GET R/actions/jobs/<job>/logs`** answers `302` with `Location`, a
  signed address on another host, valid about ten minutes, which wants
  no credential (`job-log-302-blob`). It honours `Range: bytes=0-0`
  with `206` and `Content-Range` (`job-log-range-honoured`). Skein sends
  no Range, and rejects an unsolicited 206 before the log body decoder.
- **`github::Redirect { origin: { tls, host, port }, target }`,** parsed
  from `Location`: `https` over TLS, `http` in plaintext (the fake's,
  ../testing/fakes/github.md, 1.1), with its host and port (443 or 80 when none),
  the target its path and query. Another scheme, or none, is
  `InvalidAnswer`. A plaintext origin is never among temper's defaults.
- **The component follows it** to an origin among its `log_origins`,
  sending nothing of the forge's, and an origin it does not know is
  `Refused` and a diagnostic (connection-calls.md, section 1).
- **The log is read whole** as it streams, within the component's input
  bound. GitHub serves the job's log. With a failed step in the validated
  metadata, `Retention::FailedStep` keeps at most `max_bytes` ending at
  the first runner `##[error]Process completed with exit code ` line's
  newline. Later bytes, including post-job checkout cleanup, are drained
  without replacing that suffix. Recognition crosses stream chunks and
  uses constant state beside the bounded suffix. Without a failed step,
  or without that runner marker, the job's last `max_bytes` are kept.
  A `Start` read keeps the log's first `max_bytes` with
  `Retention::Head`, whatever its failed step, and drains the rest; it
  sends no Range, as an `End` read does not.
  The step name shares the answer's bound with the log. `truncated` says
  that any prefix or cleanup bytes were omitted.
- **Later:** identifying failed action boundaries without a runner process
  marker. Those logs use the bounded job-tail fallback; no read window or
  stored request shape is added here.

## 11. The setup's documents

Moved to [`github-app.md`](github-app.md), section 2.

## 12. The pin

- **`API_VERSION`,** `2026-03-10`, sent on every request. A request
  without it gets 2022-11-28, and GitHub supports a version for 24
  months after its next.
- **`CHECKED`,** the date the live conformance last held on github.com
  (2026-10-20, until its first run).
- **Readers:** the conformance (../testing/fakes/github.md, section 2), which
  records each fact of `github.md`, section 13 and finds a change.

## 13. What it relies on of GitHub

Each fact by its conformance name, checked on the fake and, opt-in, on
github.com (../testing/fakes/github.md, section 2).

- **Tokens and identity:** `token-expires-in-an-hour`,
  `token-narrowed-creates-not-edits`; `rate-core-5000`,
  `jwt-calls-no-rate`; `viewer-is-bot`, `user-on-installation-403`,
  `app-routes-401-on-installation-token`; `jwt-iss-string`;
  `app-owner-id-login-type` (`GET /app`'s `owner` has `id`, `login` and
  `type`, as GitHub's documentation lists them);
  `bot-permission-is-not-a-role` (the App's bot's collaborator
  permission is `none` with an empty `role_name`, and the full mint's
  `permissions` name the installation's levels; observed 2026-10-23).
  Contents narrowing returns mint permissions contents: write,
  metadata: read. Issue creation is 201 with the accepted permissions
  header; a title PATCH of that issue with the same token is 403,
  "Resource not accessible by integration".
- **Pull requests and CI:** `mergeable-null-then-settles`,
  `combined-pending-with-no-status`, `job-log-302-blob`,
  `job-log-range-honoured`; `merge-stale-409`,
  `merge-after-move-not-merged`, `merge-unheld-by-ci`,
  `update-stale-422`, `update-202-later`; `etag-304-free`.
  Mergeability is null in the create response or a read soon after a head
  move, then boolean within the wait; the first read may already be boolean.
  The move fact requires either the recorded 405 or 409 and a pull read
  that is not merged. The Range fact expects 206 and Content-Range.
  The ETag fact reads an unchanging repository with an installation token, and expects 304 with
  the same remaining rate. `merge-repeat-idempotent` expects 200, merged
  true and the same merge sha for a repeat with the same sha, then compares
  both PUTs with the merged commit of the dialect's pull answer. At the pin,
  REST omits merge_commit_sha, so this answer reads mergeCommit.oid through GQ.
  `merge-repeat-any-sha-200` expects 200 and merged true for a repeat with a
  different sha on a merged pull; an open pull's stale sha remains 409.
- **The log:** `log-lists-failed-deliveries`, `log-cursor-in-link`,
  `log-ids-19-digits`, `log-delivery-whole`,
  `log-installation-events-unscoped`;
  `installation-target-in-log` (an App not subscribed to it). The fake checks
  an account rename; live records it unchecked because the run renames no
  account, so the event remains unobserved live.
- **Rules and the queue:** `approve-own-422`, `ruleset-approval-405`,
  `bypass-always-merges`; `queue-direct-merge-405`,
  `enqueue-stale-unprocessable`, `enqueue-pending-check-protected`,
  `queue-merged-no-merge-commit-sha`,
  `queue-dequeued-delivered`; `signatures-push-gh013`;
  `rules-branches-current-user-can-bypass`,
  `classic-protection-404-under-rulesets`. Own approval is 422 with
  message Unprocessable Entity and a string error, Review Can not approve
  your own pull request. Pull request rules carry all five parameters:
  the approval count and the four booleans false, with an empty ref exclusion.
  Required status checks include strict_required_status_checks_policy: false
  and context ci. Merge queues carry all seven parameters: timeout 60
  minutes, grouping ALLGREEN, maximum build and merge entries 5 each,
  method MERGE, minimum entries 1, and minimum-entry wait 0 minutes.
  One entry and no batching wait keep single-pull cases immediate; the
  check timeout is a ceiling, not a mandatory delay. Queue cases and their
  rulesets use an organization's repository and its matching tokens.
  The pending-check fact approves the pull while its head workflow is still
  in progress, tests Merge as Protected, and tests raw enqueue as UNPROCESSABLE
  with the required-check prefix. No case posts commit statuses. The merge
  case waits up to 180 seconds (90 seconds before the user's doubling of 10-22, provisional) for the head's ci to pass, admits through Merge
  as Queued, and polls the pull up to 300 seconds (150 seconds before the user's doubling of 10-22, provisional) while the group's own ci
  runs. REST omits merge_commit_sha and the dialect reads the merged commit
  through GQ. The dequeue case sends a SQUASH ruleset and creates two commits
  with passing messages, putting FAIL only in the pull title. It records the
  repository's squash title setting and the group commit's message. The group
  branch appears asynchronously after enqueue; the case polls its branch
  listing at most twenty-five times, five seconds apart (120 seconds; sixty seconds before the user's doubling of 10-22, provisional), and
  records the number of reads. A listing with 100 or more branches fails the
  fact as queue branches listing full; an exhausted poll fails it as queue
  group branch absent. A setting other than COMMIT_OR_PR_TITLE, or a passing
  group check, makes that fact
  unchecked: the title-driven group failure remains unconfirmed live. A failed
  group check must produce a dequeued delivery. Head and group checks and
  merge reads are polled every 15 seconds. Each wait expiry names the head ci,
  group ci, or merge wait in a failed fact. Approval, admission, branch and
  check read refusals fail their fact; a raw enqueue refusal includes its GQ
  message. Later cases still run.
- **Repositories and issues:** `installation-repositories-at-once`;
  `issues-list-pulls`, `new-issue-may-miss-listing`.
  The pull listing gets three tries, two seconds apart, without since.
  The new-issue fact records immediate visibility in an unfiltered listing;
  either visibility is valid. `issues-since-epoch-empty` first waits for its
  made issue in the unfiltered listing, with at most three reads two seconds
  apart, and records the read count. Exhaustion fails the fact as made issue
  absent from listing and sends no later reads. Once listed, raw epoch since
  must answer 200 with an empty array, then the dialect's zero-time Items
  read must contain the issue.
  `installation-repository-removal-refusals` expects 422, "Cannot remove the
  last repository from this installation.", from each fixture installation.
- **Additional protocol facts:**
  `branch-with-slash-reads`, `issues-since-inclusive`,
  `user-by-id-reads`, `repository-allows-shown`,
  `pulls-422-no-commits`, `pulls-422-exists`, `refs-422-exists`,
  `refs-delete-missing-422`, `update-conflict-422`,
  `compare-files-capped-300`,
  `comment-updated-at-on-edit`. The inclusive-since case takes a settled
  row's updated_at from an unfiltered listing and finds that row when since
  equals it. Both pull-creation 422 cases read errors[].message beneath
  "Validation Failed", in a PullRequest resource object with code custom:
  "No commits between main and <head>" or "A pull request already exists for
  <owner>:<head>." The comment edit uses the App's installation token and
  requires updated_at to advance. For setup and the component
  (setup/README.md and connection.md cite them):
  `inactive-hook-visible`, and `manifest-hook-invalid-host-accepted`
  (an App hook at `https://<name>.invalid/`, such as
  `https://hooks.example-app.invalid/github`, is taken), both
  `unchecked` on github.com and held on the fake alone, since a live
  run changes nothing of the App (../testing/fakes/github.md, section 2); and
  `job-log-host-per-owner` (whether the log's redirect host differs
  across the two owners' repositories).

A fact's texts (each refusal's message) are the conformance's
transcripts, and the decoders' tests take them (section 14). A response
framing, length or parse error preserves the masked request and received
response head and fails that fact; later cases run, while guard, budget,
journal and cleanup errors stop the run. A missing input from an earlier
response fails only its own fact; later cases run.
The runner re-signs the App JWT at half its 600-second life before an App
request. Close-delimited responses are read to stream end within the 4 MiB
bound on a connection used for one request, and each records its case and
status in provenance.

## 14. Verification

The module's step tests, in `skein-forge`, with no io, by README.md
section 9, and GitHub's:

- **Recorded answers:** each decoder against the conformance's
  transcripts (the fake's, and github.com's once a live run has held),
  answers generated from a seed, and mutations of both.
- **Each refusal of section 7** told apart from its neighbours of one
  status, by its message, and an unknown message on a read and a write.
- **The merge plan's routes:** merged; refused for the queue and
  queued; stale at the merge and at the queue; each other `405`.
- **Protection:** rulesets alone, classic alone, both, neither, a
  ruleset's bypass of each value, more rulesets than `inner`.
- **The log:** a page with a next and without, a cursor at 128 bytes
  and one past, a `304`, 19-digit ids and `u64::MAX`, a row with no
  installation, a whole delivery decoded as its pushed body is.
- **The App:** the JSON Web Token's bytes, golden, and a span of 600
  seconds and 601; the mint's two bodies; a guid as a UUID, upper-case,
  and not one.
- **The redirect:** a `Location` with a port, without, `http`, `ftp`,
  none.
- **Fuzzing:** one target per decoder, as Forgejo's.

The fake's GitHub layer is tested by the conformance against it, and the
component's worlds run it (../testing/fakes/github.md, sections 1 and 2).

## 15. Open questions

- **Stricter App account decoding** (`github-app.md`, section 2). Default: a missing or
  incomplete account leaves the answer usable without one. Live evidence
  for `app-owner-id-login-type` (`github.md`, section 13) precedes any stricter rule;
  the dialect assigns no authority from the account it answers.
- **Conditional reads beyond the log.** GitHub answers `304`, free, on a
  stable repository (`etag-304-free`); broader use would keep validators
  for other resources too. Default: the log alone.
- **The update's head.** `Update { number }` carries none, so the plan
  reads it first; `Update { number, head }` would make it conditional on
  the head the client saw, as `Merge` is. Default: read first.
- **The queue's read on every pull.** An open pull request of an
  organization's repository costs a GraphQL read for its position.
  Default: so; a repository known to have no queue could skip it.
- **Long threads.** An item's comments are read from page 1 each time.
  Default: so; a `since` the client named would skip pages.
- **Official reviews** by `author_association`. Default: owner, member
  and collaborator; the rule's required reviewers would be exact, at a
  cost.
- **Repository ids past 32 bits** are `TooLarge`, as Forgejo's.
  github.com's are about a third of the way there. Default: so, until
  the vocabulary widens `Repository`.
- **The blob host's address:** connection.md, section 11's runtime-DNS question.
- **A classic rule's merge queue** is not read by `Protection`: the
  merge's refusal finds it (`github-operations.md`, 2.1). Default: so.

## 16. The live conformance's organization selection

The live fixture format is described in ../testing/fakes/github.md, section 2, and
`skein/tests/forge/github/README.md`. At startup, before minting tokens or making
repository changes, the runner reads each installation's probe repository
with that owner's person token. The first probe whose `owner.type` is
`Organization` selects the installation for every merge-queue case,
starting with `queue-direct-merge-405`, and their rulesets. Those cases
use that installation's token and its owner's person token where a person
acts. The fixture document needs no account-type field. If none qualifies,
startup stops with "no organization installation for the merge-queue cases"
before any write. The fake's `acme` is a user and `globex` an organization.

The plan's marker word is the tool's own `b"conformance"` constant, checked
with `marker::valid_word` in `Runner::start` before any request. It is
independent of the App's slug, which names the bot and appears in provenance.

Each live fixture repository must carry `.github/workflows/ci.yml`, triggered
by `push`, `pull_request`, and `merge_group`, with job and check run named
`ci`. It runs for about 45 seconds. It reads the message at
`pull_request.head.sha`, else `merge_group.head_sha`, else the pushed sha,
and fails when that message contains `FAIL`; otherwise it succeeds. The
queue ruleset requires this check. The conformance fake uses the shared
domain's CI, runs, jobs and message cue with 45 fake seconds of latency.

The repository's `squash_merge_commit_title` must be `COMMIT_OR_PR_TITLE`
for the title-driven dequeue probe: one commit uses that commit's message,
more than one uses the pull's title. The case reads and records the setting;
it changes no repository setting. Its two head commits pass, while its
`FAIL` pull title is intended to fail the squash group's workflow.
