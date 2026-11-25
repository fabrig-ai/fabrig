# Forgejo, as the kit uses it

## 1. Forgejo's API

How the Forgejo dialect uses the API, at its pin. The owner's values
are the word, the user agent, the credential and the limits (`README.md`, section 2.2
). The operations are the calls of `vocabulary.md`, 1.2 and the start's reads (`dialects.md`, 1.1);
the setup's requests are the dialect's too, and setup/forgejo.md,
sections 1.3 and 1.5, list them.

### 1.1 Conventions

- **Base, transport and authentication.** Every request goes to its
  forge's `api` address, under `/api/v1`, over HTTP/1.1: over TLS with
  the configured server name and trust, or in plaintext to a loopback
  address. Each carries `Authorization: token <t>`, the forge's API
  credential (`README.md`, section 7), which the component holds from start and never
  logs. `Host` is the transport's name, the TLS server name or the name
  a plaintext forge was configured by, else the address's IP, an IPv6
  one in brackets; either with `:port` when the port is not the scheme's
  default (443, 80). A TLS server name that is an IP address is written
  as the address's IP is, with the address's port, whichever IP the
  address holds: IPv4 in dotted decimal, IPv6 in brackets as its eight
  groups in lowercase hexadecimal without leading zeros, never
  compressed (`[2001:db8:0:0:0:0:0:7]`); a DNS name is written as it
  was given. `skein-forge`'s `host` makes it from the address and the
  name, for the component and a setup's host alike; no owner writes
  it. `User-Agent` is the owner's, given at start.
- **JSON in, JSON out.** Bodies are written with skein-json's sized
  writer (`Content-Type: application/json`), and answers decoded from
  its tokens. Raw files and job logs are read as bytes. Strings are
  UTF-8; text a call carries is bytes, and a write whose text is not
  UTF-8 is refused before it is sent (1.4).
- **Repositories by id.** A call names `Repository { forge, repository }`,
  `repository` being Forgejo's repository id. Forgejo's routes name a
  repository by path, so each request is made at
  `/api/v1/repos/{owner}/{name}`, from the path the component keeps for
  the id (connection-calls.md, section 1). A call whose repository has
  no path kept first resolves it with `GET /api/v1/repositories/{id}`,
  and that request counts in its cost. A locate names an owner and a
  name itself, and is addressed with the forge's number and repository 0
  (`vocabulary.md`, 1.7).
- **Names in paths** (a repository's owner and name, a branch, a login,
  a file's path) are percent-encoded, each as one path segment; a slash
  inside a branch's name is `%2F`. A file's path keeps its slashes
  between segments.
- **Times.** Forgejo writes RFC 3339 times with an offset; they are read
  as the forge's clock, in nanoseconds since the epoch, at its
  resolution of a second. `since` is written in RFC 3339, in UTC, for
  any time the vocabulary names, up to the largest (`u64::MAX`
  nanoseconds, in 2554), with which the client reads the forge's clock
  alone: the answer is an empty page with its `Date`. The time an answer
  was made is its `Date` header, which a listing's answer carries as
  `now`.
- **Commits.** A commit is named by 32 bytes (a SHA-1 id zero-padded). A
  repository's object format (`object_format_name`, kept with its path)
  says whether its ids are 40 or 64 hexadecimal digits; anything else is
  a malformed answer. Ids are written in lowercase hexadecimal.
- **Paging is always explicit.** Every listing Forgejo pages is sent
  with `page` and `limit`; a listing without both is never sent:
  Forgejo answers with everything where it ignores the size. The size
  is fixed per forge and per listing: the smallest of the client's
  `rows`, the forge's `max_response_items` (read at the forge's start,
  `dialects.md`, 1.1), and the number of that listing's largest rows the client's
  `answer_bytes` holds (1.3), each row by the measure of `vocabulary.md`, 1.7, so the
  size is the same on every target. `Items`, `Reviews` and `PullFiles`,
  and a `Remarks` slice, are sized for real rows: the smallest of `rows`
  and `max_response_items`, the answer keeping each row whole while it
  fits `answer_bytes` and cut before the first that doesn't (1.3), so a
  page of a repository's issues, of a pull request's reviews or files,
  or of a review's remarks holds as many as its bytes allow, not one
  where one row could fill it. An `Items` page is of one where a cut
  within rows of one time asks it (the client's `forge/client.md`, 4);
  a cut page of the others is read on at the same page, past the rows
  its `skip` names (`vocabulary.md`, 1.2). A listing has
  more after it when it came full, or when Forgejo says so (`X-HasMore`
  on a pull request's files). A combined status's `state` and
  `total_count` are the page's, not the commit's (Forgejo 17 computes
  them over the page asked for), so a status listing has more after it
  when its page came full, never by its `total_count`. A first page
  that comes short holds every status of the commit, so its `state`,
  each context's latest folded by Forgejo, is the commit's verdict, read
  as `forgejo-operations.md`, 1's notes on CI map it: a read that needs the verdict alone
  (`Pull`'s) takes it from that page, and reads and folds every page
  only when the first came full. A `warning` state is the one exception:
  Forgejo may rank it above a pending context, so a short page whose
  `state` is `warning` has its own rows folded, which the read keeps
  folding as they stream, keeping none.
  Listings Forgejo does not page are read whole and cut as they stream.
- **Users by id.** The vocabulary names people by their forge user id;
  the API names them by login in paths and bodies. The component keeps
  a table of the two (connection-calls.md, section 1).

### 1.2 Each operation

Moved to [`forgejo-operations.md`](forgejo-operations.md), section 1.

### 1.3 Decoding answers

- **Streams, not documents.** Each answer's decoder is a state machine
  over skein-json's tokens with a bounded stack, pulled as the body
  arrives (../data/json.md, section 3.1). It keeps only the fields its answer
  needs, as plain values; every other field, nested object and array is
  skipped as it passes.
- **A string is kept whole, or not at all.** skein-json keeps a string
  only when it fits the cap its reader names, and otherwise reads it
  through and says its length (`Text` and `Long`, ../data/json.md, section 3.1).
  So:
  - a name, a title, a context, a description, a link or a path is kept
    within `string_bytes`; a longer one makes the answer malformed;
  - but for a job step's name, which Forgejo makes of the step's whole
    script when the workflow names none: a longer one is carried empty,
    and the step is named by its number alone, so that a long script
    never makes a job unreadable;
  - a body (an issue's, a comment's, a review's, a remark's) is kept
    whole within `body_bytes`. A longer one is carried empty, its key
    none, its provenance `Unknown`, and its revision made from its
    length (below); the component counts it among its diagnostics.
    `body_bytes` is at least the largest body the owner writes, its
    marker included (the client's `op_bytes`), checked at start, so the
    owner's own creations are always read whole and their keys found;
  - every string is read as UTF-8 by skein-json; a body that is not
    valid UTF-8 is a malformed answer.
- **The answer's bound.** The page size (1.1) holds a full page of a
  listing's largest rows within the client's `answer_bytes`, so a paged
  answer never passes it. A page sized for real rows (`Items`,
  `Reviews`, `PullFiles`, a `Remarks` slice; 1.1) keeps its rows whole
  while they fit `answer_bytes` and is cut before the first that would
  pass it, a pull request's file being its name and both its sides: the rows kept, in order, `more` set, and the
  answer saying it was cut, so a cut is never read as the forge's last
  page. A first row that alone passes it is `TooLarge`, as before. An item's comments are kept until the next
  would pass `answer_bytes` or `rows`, and `more` is then set. Raw
  contents are read within what is left of `answer_bytes`: a file's
  after its path, a job's log after its step's name, so the answer never
  passes it. `File` and `Job` are cut there as at their `max_bytes`, and
  say so (`truncated`); a side of `PullFiles` is never cut (`forgejo-operations.md`, 1), and a comparison's names stop at the answer's bound, counted on in `changed`.
- **Revisions.** A comment's or a review's revision is the first eight
  bytes, big-endian, of the SHA-256 of its whole body as the forge
  returns it, computed as the string streams past; for a body not kept,
  of its length and its `updated_at`. So it changes whenever the body
  does, at any resolution of the forge's clock, and names what the forge
  stored, not what was sent.
- **Provenance.** `Original` when `updated_at` equals `created_at` (a
  review's `submitted_at`), `Revised` when it is later, `Unknown` when
  either is missing or the body was not kept. Forgejo's clock has a
  second's resolution: an edit within the second of the creation reads
  as `Original` (`README.md`, section 10).
- **Markers** at the head of a body, a comment or a review are decoded
  into its `key` (`README.md`, section 5).
- **A changed file's `status`** is `added`, `deleted` or `removed`,
  `modified`, `changed` or `unchanged`, `renamed` or `copied`; `removed` is what a real
  Forgejo's comparison says of a file gone, and reads as `deleted` does.
  `unchanged` is what Forgejo's pull files listing says of a file
  modified with no line added or deleted (a binary file, as a wire
  golden is, or a change of mode alone), and reads as `modified` does:
  with `sides`, both its sides are read. Any other word is a malformed
  answer.
- **What may follow** is told by the page coming full, `X-HasMore`, or
  `total_count`. For a call made of several requests, `more` is its
  listing's.
- **Absent headers.** An `Items` answer without `Date` is malformed,
  since its `now` becomes the next listing's `since`. A `429` without
  `Retry-After` is `RateLimited { after: 0 }`, and a listing without
  `X-HasMore` has more only when it came full.
- **A malformed answer** (a success status other than its request's,
  `forgejo-operations.md`, 1; a document that does not decode, a field missing, a number out
  of range, a commit id of the wrong length; a log read through past the
  component's `body_bytes_in`; a repository read by its id answering
  another) fails a write as `Timeout`: the request reached the forge, so
  what it asked may have been done. It fails a read as `InvalidAnswer`.
  An answer past a bound (a string past its cap; a JSON document past
  the component's `body_bytes_in`, as an unpaged comparison far behind
  its base can be) fails a write as `Timeout` too, and a read as
  `TooLarge` (1.4): a document past `body_bytes_in` names `Forge`, its
  `limit` that figure and its `size` what had been delivered when the
  bound stopped it, not exact; a string past its cap names `Unnamed`.
  Either way the connection is closed, since what followed on it can no
  longer be trusted. A repository id past the vocabulary's 32 bits fails
  the call as `TooLarge` (`Forge`, `limit` 4,294,967,295, `size` the id).

### 1.4 Failures

A call's `Error` is the first failure of its requests. "Nothing written"
means the call failed before any byte of its first request was written;
such a request costs nothing.

| What happened | `Error` |
|---|---|
| nothing written: no connection, a refused connect, a TLS handshake that failed; or the server never saw the request (skein-http's `Closed`); or the call's deadline while it waited | `Unavailable` |
| a `3xx`, as Forgejo answers a moved repository's old path before it acts: the path dropped | `Unavailable` |
| `429`, with `Retry-After` (seconds or a date), before any of the call's writes succeeded | `RateLimited { after }`: the wait, a date taken from the answer's `Date` |
| anything after a byte was written: a reset, the end of the stream, the call's deadline, `5xx`; a `429` after one of the call's writes succeeded (`Merge`'s read after its merge) | `Timeout` |
| a malformed answer (1.3) | `Timeout` on a write; `InvalidAnswer` on a read |
| `401`, `403` | `Forbidden`; on a write, the repository's row dropped, so that what reads after a refusal reads its settings afresh |
| a `404` on a leaf, classified as `Asked::Leaf`: a branch's (`Read::Branch`'s own read, not its resolution), contents' (`Read::File`'s own read), `Read::PullFor`'s lookup of a head and base that have no pull request, and an Actions run's, job's or log's own read | `Missing`, `MissingJob` on `Job`'s reads, `Drop::Nothing`: the repository row stays |
| every other `404` that ends the call (one `forgejo-operations.md`, 1 says ends nothing is an answer, and the path stays), including repository resolution, its own GET and listings | `Missing`, the repository's path dropped |
| `413` (Forgejo's quota) | `Full` |
| `423` (the repository is archived) | `Forbidden` |
| `409` on `OpenPull` (a pull request for the head and base is open) | `Exists` |
| `OpenPull`'s comparison counting no commit (`total_commits` 0): the pull request not written | `NothingToMerge` |
| on `OpenPull`'s reads, before its `POST`: what this table makes a `Timeout` or an `InvalidAnswer` (the pull request was not written) | `Unavailable` |
| `422` on a write whose title or body is empty | `Empty` |
| `405` on `Merge` for a pull request merged or closed | `Closed` |
| `405` on `Merge` for a conflict | `Conflict` |
| `405` on `Merge`, `403` on `DeleteBranch`, for a branch's protection | `Protected` |
| `405` on `Merge` for a style the repository does not allow | `Refused`, the repository's row dropped |
| `409` on `Merge`, `head out of date` | `Stale` |
| `409` on `Update`, `merge failed because of conflict` | `Conflict` |
| a pull request's head other than `PullFiles` names | `Stale` |
| an answer past the component's bounds | `TooLarge` on a read, `Forge` with the bound and what was read where the component gives them, else `Unnamed`; `Timeout` on a write, which may have been done |
| a write whose text is not UTF-8, which JSON cannot carry; nothing written | `Refused` |
| any other `4xx` | `Refused` on a write; `InvalidAnswer` on a read |

A repository gone or inaccessible loses its row at its next repository-level
call. An absent leaf alone keeps the row that answered.

- **The plan gives the `Error`,** from what it hears (`dialects.md`, 1.2). The
  component tells it how an exchange failed, `Unseen` or `Seen`, from
  skein-http's errors and its own deadlines (connection-machines.md,
  section 2).
- **Unavailable is provable only before the first byte.** skein-http's
  client tells a request the server never saw (`Closed`) from one it
  may have read (`Truncated`, `Stream`; http.md, section 3.4). A
  connection kept alive that the forge closed just as a request went out
  may have been read: it is a `Timeout`. To make that rare, an idle
  connection is closed before the forge's keep-alive timeout would
  close it (`idle`, connection-machines.md, section 1).

- **One status, several meanings.** Where Forgejo answers two failures
  with one status (`405` on a merge), the decoder reads its `message`
  into the failure. The dialect names a refusal only when both the
  status and the whole message match: a merge on a closed pull request,
  a conflict, a protection, a style, or a head out of date; an update
  with a conflict; an empty title or body. The messages are those the
  pinned release answers, in the conformance's transcripts (section 2).
  A `4xx` means Forgejo did not act, so a message the decoder does not
  know is `Refused` on a write, never a guess at which failure it was;
  on a read, `InvalidAnswer`.
- **A close of a closed item.** Forgejo refuses a state edit of a
  merged pull request, `412`, `cannot change state of this pull
  request, it was already merged`, and changes nothing; it answers
  `412` too where an open issue's dependencies forbid its close. A
  `Close` answers `Done` (`vocabulary.md`, 1.3), and the fake's domain
  closes a closed item so; a client closing a pull request merged by
  another hand would otherwise be refused for good. So a `Close` whose
  `PATCH` this table makes `Refused`, whatever its status and message,
  reads its item once, `GET R/issues/<number>` (the request `Item`
  starts with, its comments unread), and the call answers by what
  the item's `state` says: `closed`, merged or not, is `Done`, the
  state the write names holding (client-effects.md, 1.3); `open` keeps
  `Refused`. That read's own failure keeps `Refused` too: the close
  was not made, and the read changes nothing. The call costs both
  requests. `Reopen` gains no read: a merged pull request cannot be
  reopened, and its refusal stands.
- **Not errors:** a `409` on `CreateBranch` is `Branch(Exists)`, which
  the client follows with a read of the branch; a `404` on
  `DeleteBranch` is `Missing`, which the client counts as the branch
  gone (client-effects.md, section 1). Forgejo answers the deletion of a
  branch that is not there `500` with an empty body (`forgejo.md`, section 2): a
  `Timeout`, after which the client's find reads the branch, and that
  read's `404` is `Missing`, the branch gone.
- **Reads before a write.** `OpenPull`'s reads come before its `POST`,
  so a failure among them leaves the pull request provably not written:
  what would be a `Timeout` or an `InvalidAnswer` is `Unavailable`, and
  the client may send the call again. Their other failures are as the
  table says (a `404`, a branch not there, `Missing`; a `429`
  `RateLimited`; a `401` or a `403` `Forbidden`).
- **The deadline.** Each call has the component's `deadline`, which the
  component checks at start is at most the client's `lifetime`, so no
  byte of a write is sent past the time the client counts on. A call
  still waiting at its deadline is `Unavailable`; one in flight is a
  `Timeout`, and its connection is aborted.
- **Never `Busy`.** The component holds as many calls as the client may
  have in flight (its `calls`), so it never refuses one: `Busy` is the
  client's own admission.

## 2. What skein-forge relies on of Forgejo

What the component and the client rely on of Forgejo, on the pinned
release (`dialects.md`, 1.3). Each is named by its fact in the conformance's record
(../testing/fakes/throwaway.md, section 2), where it `holds`; a fact marked *fails* is
phrased the other way there, and its failing is what the dialect relies
on. What no run records yet has its own bullet, near the end.

- **Every call of `forgejo-operations.md`, 1, made by the component.** Each read and write
  answers as its row says, its cost the requests it sent, and the keys
  the owner writes are found on reading back; a forge's start is ready,
  and another user or a wrong token stops it (every `component-` fact:
  one per call, from `component-read-items` to
  `component-write-delete-branch`, with
  `component-read-pull-for-open-after-closed` and
  `component-write-open-pull-no-changes`; one per start's outcome, from
  `component-start-ready`; `component-costs-equal-requests-sent`;
  `component-keys-found-on-read-back`).
- **Paging.** With Forgejo's default settings, `max_response_items` is
  50 and `default_paging_num` 30 (`api-default-page-cap-50-and-30`). A
  pull request's files are paged, with `X-HasMore`
  (`pull-files-page-with-x-hasmore`); a comparison ignores `page` and
  `limit` and answers everything (`compare-ignores-page-and-limit`).
- **The keep-up listing.** `sort=leastupdate` lists the least recently
  updated first, ties by number
  (`leastupdate-orders-least-recently-updated-first`,
  `leastupdate-ties-ordered-by-number`); `since` is inclusive at the
  second (`since-is-inclusive-at-the-second`), and in 2554 answers an
  empty page with its `Date`
  (`since-in-2554-answers-an-empty-page-with-date`). An item's updated
  time moves on a comment, a review, a push to its head, closing and
  reopening (`updated-moves-on-comment`, `updated-moves-on-review`,
  `updated-moves-on-head-push`, `updated-moves-on-close`,
  `updated-moves-on-reopen`), and stays on a comment's edit or
  deletion, a status and a push to its base
  (`updated-stays-on-comment-edit`, `updated-stays-on-comment-delete`,
  `updated-stays-on-head-status`, `updated-stays-on-base-push`): so a
  pull request's CI and base are read on a backoff (client.md,
  section 4).
- **Bodies and their versions.** A marker's line feed is kept
  (`marker-line-feed-kept`); bodies of 64 KiB and 1 MiB are kept whole
  (`body-of-65536-bytes-kept-whole`, `body-of-1048576-bytes-kept-whole`).
  An unedited comment's or review's updated time is its creation's
  (`unedited-comment-updated-equals-created`,
  `unedited-review-updated-equals-submitted`), and an edit moves it
  (`edit-moves-comment-updated`). An issue's comments and a pull
  request's reviews come oldest first (`comments-come-oldest-first`,
  `reviews-come-oldest-first`); a dismissed review keeps its body
  (`dismissed-review-keeps-its-body`); a remark's `position` is its line
  (`remark-position-is-its-line`).
- **Pull requests.** `PATCH issues/{index}` edits a pull request's title
  and body (`patch-issue-edits-a-pull-request`); its `base.sha` is its
  base's tip (`base-sha-is-the-base-tip`). The lookup
  `pulls/{base}/{head}` takes a percent-encoded slash
  (`pull-for-accepts-encoded-slash`) and names the oldest of several
  pull requests, a closed one before a newer open one
  (`pull-for-names-the-newest-of-several`, *fails*). Opening one whose
  head adds nothing to its base answers `201` and opens it
  (`open-pull-no-changes-is-422`, *fails*); opening one for a head and
  base that have one open answers `409` (`open-pull-exists-is-409`).
- **Merges and updates.** A merge answers `200` with no body
  (`merge-answers-200-with-no-body`); one naming an old head `409`,
  `head out of date` (`merge-old-head-is-409-head-out-of-date`); one
  refused for a closed pull request, a conflict, a protection or a style
  `405` (`merge-closed-is-405`, `merge-conflict-is-405`,
  `merge-protected-is-405`, `merge-style-is-405`). An update refused for
  a conflict answers `409`, `merge failed because of conflict`
  (`update-conflict-is-409-merge-failed`).
- **Branches.** `branches/{branch}` takes a percent-encoded slash
  (`branch-path-takes-encoded-slash`). A branch is created at a full
  commit id (`create-branch-at-full-commit-id`), and creating one that
  exists answers `409` (`create-branch-exists-is-409`). Deleting a
  protected branch answers `403` (`delete-protected-branch-is-403`), and
  deleting one that is not there `500`, with an empty body
  (`delete-missing-branch-is-404`, *fails*).
- **Protection.** A write collaborator who is not an admin is refused
  protection reads, `403` (`writer-protection-reads-are-403`), and reads
  a branch's `effective_branch_protection_name` empty
  (`effective-protection-name-for-a-writer`, *fails*), which names the
  rule for an admin (`effective-protection-name-for-an-admin`): so an
  owner that is not an admin records protection as unknown, never
  absent.
- **CI.** A combined status with no status counts 0
  (`combined-status-without-status-counts-0`), and `skipped` with
  `success` combines as passed (`skipped-and-success-combine-as-passed`).
  Actions report each job as a status on its head
  (`actions-report-each-job-as-a-status-on-its-head`), linking
  `<…>/actions/runs/<run number>/jobs/<job index>` and naming no attempt
  (`actions-status-link-is-runs-n-jobs-i`,
  `actions-status-link-names-no-attempt`); a run's number is its
  `index_in_repo` (`run-number-is-index-in-repo`), and its jobs come in
  index order (`run-jobs-come-in-index-order`).
- **Jobs and their logs.** A job's steps are numbered from 0, `Set up
  job` first and `Complete job` last
  (`job-steps-number-from-0-set-up-first-complete-last`); a failed
  workflow step is `failure`, and the completion step with it
  (`failed-workflow-step-and-completion-step-are-failure`). A step's log
  holds that step's lines alone (`step-log-holds-that-steps-lines-alone`),
  and a step past the last answers `404` (`step-past-the-last-is-404`).
  A range answers `206`, over a job's log and over a raw file
  (`job-log-range-answers-206`, `raw-range-answers-206`); a suffix range
  over a step's log, its last bytes, with a `Content-Range` naming where
  they start (`step-log-suffix-range-answers-206-with-its-last-bytes`,
  `step-log-suffix-content-range-names-its-first-byte`); over an empty
  log, nothing (`empty-log-suffix-range-answers-206-empty`).
- **Comparisons.** `files=false` lists no commit's files and leaves the
  comparison's own `files` empty, and keeps `total_commits` and the
  commits' ids (`compare-files-false-lists-no-commits-files`,
  `compare-files-false-keeps-its-own-files-empty`,
  `compare-files-false-keeps-total-commits-and-shas`);
  `verification=false` changes nothing the calls keep
  (`compare-verification-false-changes-nothing-kept`).
- **Access and errors.** A repository the token cannot read answers
  `404`, by path and by id (`unreadable-repository-by-path-is-404`,
  `unreadable-repository-by-id-is-404`); a moved repository's old path
  answers a redirect (`moved-repository-is-a-redirect`); a write to an
  archived repository `423` (`archived-write-is-423`); an empty title
  `422` (`empty-title-is-422`). No rate-limit headers are sent
  (`no-rate-limit-headers`).
- **Deliveries.** Each row of `README.md`, section 6's table arrives under its
  `X-Forgejo-Event` with the fields its hint reads
  (`delivery-issues-row`, `delivery-pull-requests-row`,
  `delivery-comments-row`, `delivery-reviews-row`, `delivery-push-row`,
  `delivery-create-delete-row`, `delivery-job-ends-row`,
  `delivery-cancelled-job-end-is-delivered`), with its repository's id
  (`delivery-carries-the-repository-id`); a hook's events read back as
  made (`delivery-subscribed-events-read-back`); the signature is
  HMAC-SHA256 in lowercase hexadecimal
  (`delivery-signature-is-hmac-sha256-hex`);
  refs are full, and a tag's told from a branch's
  (`delivery-refs-are-full-and-tags-are-told-apart`); a branch's
  deletion is delivered as a push too
  (`delivery-branch-deletion-sends-no-push`, *fails*). Forgejo delivers
  the actor's own writes (`delivery-echoes-the-actors-own-writes`), so
  the owner drops their echoes, by key and by pusher (`README.md`, section 6); it
  refuses to deliver to loopback by default, and delivers there when its
  settings name the address
  (`delivery-loopback-is-refused-by-default`,
  `delivery-loopback-is-allowed-when-named`); a failed delivery is not
  sent again (`delivery-failed-delivery-is-not-sent-again`).
- **What it settles after it delivers.** A pull request reads
  `mergeable` false while Forgejo checks it
  (`delivery-pull-reads-not-mergeable-while-checked`). A pull request is
  still being checked when the push that moved its head or base is
  delivered (`delivery-pull-is-checking-when-its-push-is-delivered`;
  `delivery-pull-is-settled-when-its-push-is-delivered`, *fails*), and a
  job's commit status is written before its end is delivered
  (`delivery-job-end-follows-its-commit-status`;
  `delivery-job-end-precedes-its-commit-status`, *fails*). The client
  relies on neither order and runs no window for them: it checks only
  that `settle` is above 0 (client.md, section 4), and the rereads
  that wait on a settling are the owner's fresh reads (client.md,
  section 5).
- **What no run records yet,** read from Forgejo's API description and
  sources alone, so no fact names it:
  - that an issue's comments and a review's comments take no page and
    come whole however many there are (`forgejo-operations.md`, 1, `Item` and `Remarks`);
  - that a state edit of a merged pull request (`PATCH issues/{index}`
    with a `state`) answers `412`, `cannot change state of this pull
    request, it was already merged`, and changes nothing (1.4, *A close
    of a closed item*);
  - that a job no runner took has no steps, and that its log, and a log
    Forgejo no longer keeps, answer `404` (`forgejo-operations.md`, 1, "A failed job's log");
  - that the open pull requests' listing honours `head` and `base`, an
    issue's comments `since`, and the Actions runs' listing
    `workflow_id` (`forgejo-operations.md`, 1). Each narrows what Forgejo sends, and the
    dialect's own row checks stay, so where one is ignored the answer is
    the same and only its cost grows.
- **Actions, trees and the turn, recorded** (Forgejo 17.0.1 and runner
  13.2.1, the conformance's turn facts: 33 hold, 7 recorded as
  failing and followed by the dialect and the fake): a dispatch answers
  `204` with no run, `404` for an absent workflow and `400` for an
  absent branch, and runs at the branch's head as it is read; a
  dispatched run's `event` is empty and its `trigger_event` is
  `workflow_dispatch`, its user `trigger_user`; the runs' listing
  honours `head_sha`, newest first, and ignores `workflow`; a
  dispatch-only job publishes no commit status on its head, so a run is
  followed by reading it, never by statuses; a fast-forward-only merge
  with `head_commit_id` moves the base to the head and marks the pull
  request merged at it, and a diverged one is `500` with the base
  unchanged; an update with `style=rebase` is refused `409` on a
  conflict, and an up-to-date one answers `200` unchanged; a status's
  `description` and `target_url` read back as written; a tree comes in
  git's order, its `truncated` true on its short last page and its
  `tree` `null` past it. A completed job's rerun answers `204`
  (`actions-rerun-completed-is-204`), and one whose run has not completed
  answers `400` (`actions-rerun-running-is-400`).
- **Dispatch inputs and artifacts, recorded** (Forgejo 17.0.1 and runner
  13.2.1): a declared input of several lines, each ended by LF, reaches
  the job whole beside its other declared inputs
  (`actions-dispatch-multiline-input-reaches-job-log`); an undeclared
  input is ignored without refusing the dispatch or changing its
  declared inputs (`actions-dispatch-undeclared-input-is-ignored`). An
  inline host-shell job uploads artifacts with nothing downloaded
  (`actions-artifact-upload-completes`); its run keeps the dispatch head
  (`actions-artifact-run-read-keeps-dispatch-head`); listing by name
  selects only that artifact
  (`actions-artifacts-name-selects-only-that-artifact`), an absent name
  gives an empty array (`actions-artifacts-absent-name-is-empty`), and an
  unknown run gives `404` (`actions-artifacts-unknown-run-is-404`). A
  download is a direct `200` ZIP (`actions-artifact-zip-is-direct`),
  containing the uploaded file whole
  (`actions-artifact-zip-keeps-one-failed-id-file`).
- **Still to record before a plan relies on them:** a comparison's
  commits oldest first with their messages; an
  update with `style=merge` of a head already holding its base (the fake
  answers `Done`, unchanged, as `style=rebase` is recorded to); what a
  rebase's push does to a push made while it works (client-effects.md, 1.3
  takes it as forced).
- **Labels, recorded:** Forgejo 17 makes a second label of a name it
  already has (no `409`), and drops an unknown label id from an issue's
  create or set without a word: so a label is made only after a read
  finds no label of its name, and a name resolves to its id by that
  read, a missing one refused before any write. A duplicate comes only
  from another writer racing the read.
- **The setup's, and the owner's git.** What reconcile relies on
  (Forgejo's command line, its admin routes, orgs, repositories'
  collaborators and hooks, the runner) is setup/forgejo.md, section 2's.
  What an owner's git relies on of Forgejo (a renamed repository's
  redirect, a protected branch's refusal of a push) is the owner's own,
  recorded in its own transcripts.
