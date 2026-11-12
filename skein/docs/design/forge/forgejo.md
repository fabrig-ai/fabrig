# Forgejo, as the kit uses it

## 1. Forgejo's API

How the Forgejo dialect uses the API, at its pin. The owner's values
are the word, the user agent, the credential and the limits (`README.md`, section 2.2
). The operations are the calls of `README.md`, 3.2 and the start's reads (`README.md`, 4.1);
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
  the id (connection.md, section 4). A call whose repository has
  no path kept first resolves it with `GET /api/v1/repositories/{id}`,
  and that request counts in its cost. A locate names an owner and a
  name itself, and is addressed with the forge's number and repository 0
  (`README.md`, 3.7).
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
  `README.md`, 4.1), and the number of that listing's largest rows the client's
  `answer_bytes` holds (1.3), each row by the measure of `README.md`, 3.7, so the
  size is the same on every target. `Items`, `Reviews` and `PullFiles`,
  and a `Remarks` slice, are sized for real rows: the smallest of `rows`
  and `max_response_items`, the answer keeping each row whole while it
  fits `answer_bytes` and cut before the first that doesn't (1.3), so a
  page of a repository's issues, of a pull request's reviews or files,
  or of a review's remarks holds as many as its bytes allow, not one
  where one row could fill it. An `Items` page is of one where a cut
  within rows of one time asks it (the client's `forge/client.md`, 4);
  a cut page of the others is read on at the same page, past the rows
  its `skip` names (`README.md`, 3.2). A listing has
  more after it when it came full, or when Forgejo says so (`X-HasMore`
  on a pull request's files). A combined status's `state` and
  `total_count` are the page's, not the commit's (Forgejo 17 computes
  them over the page asked for), so a status listing has more after it
  when its page came full, never by its `total_count`. A first page
  that comes short holds every status of the commit, so its `state`,
  each context's latest folded by Forgejo, is the commit's verdict, read
  as 1.2's notes on CI map it: a read that needs the verdict alone
  (`Pull`'s) takes it from that page, and reads and folds every page
  only when the first came full. A `warning` state is the one exception:
  Forgejo may rank it above a pending context, so a short page whose
  `state` is `warning` has its own rows folded, which the read keeps
  folding as they stream, keeping none.
  Listings Forgejo does not page are read whole and cut as they stream.
- **Users by id.** The vocabulary names people by their forge user id;
  the API names them by login in paths and bodies. The component keeps
  a table of the two (connection.md, section 4).

### 1.2 Each operation

What each call sends, in order; R is `/api/v1/repos/{owner}/{name}`, from
the repository's path, and `<size>` the page size of 5.1. Calls of more
than one request make them in turn on one connection, and end on the
first failure, but where a row says otherwise.

| Call | Requests | Answer from |
|---|---|---|
| `Read::Items { since, page, kind }` | `GET R/issues?state=all&type=issues\|pulls&since=<since>&sort=leastupdate&page=<page>&limit=<size>`, `type` only with a kind | each row a `Summary` (1.3): `number`; a `pull_request` object makes it a pull; `state`; `user.id`; `labels[].name`, for display; `updated_at`; `title`; `body` and its key; `more` when the page came full; `now` from `Date` |
| `Read::Item { number, after, since }` | `GET R/issues/<number>`; `GET R/issues/<number>/comments?since=<since>` (unpaged), `since` only when not 0 | the issue as a `Summary`; its comments with ids above `after`, oldest first, as many as the answer holds (1.3): `id`, `user.id`, `created_at`, `body` and its key, its provenance and revision; `more` when one is left |
| `Read::Pull { number, ci }` | `GET R/pulls/<number>`; then, only when `ci`, `Read::Checks`' status pages for `head.sha`, from page 1, until one comes short, with none of its job reads | `number`; `state`; `head.ref`, `base.ref`; `head.sha` as `commit`; `base.sha` as `base_commit`, none when empty; `merge_commit_sha` when `merged`; `mergeable`; `requested_reviewers[].id`; `size` from `changed_files` as `files`, `additions` and `deletions`, at `head.sha`, none when one is absent, null or no whole number, as Forgejo leaves them out where it could not count the change (README.md, 3.6); CI, when `ci`, the first page's combined `state` when that page came short and its `state` is not `warning` (1.1), else folded from every context's latest status over every page, as `Checks` folds them, no row kept; `Ci::None` without `ci` |
| `Read::PullFor { head, base }` | `GET R/pulls/<base>/<head>`; when it names a pull request that is not open, `GET R/pulls?state=open&head=<head>&base=<base>&sort=oldest&page=<n>&limit=<size>` from page 1, until a row has the head and base or a page comes short; no status read | the open pull request for the head and base, else the one the lookup named; as `Pull` without `ci`; a lookup answered `404` names none, `Missing`, and keeps the repository row (1.4) |
| `Read::Reviews { number, page, skip }` | `GET R/pulls/<number>/reviews?page=<page>&limit=<size>` | submitted reviews, oldest first, past the page's first `skip` rows, each kept whole while it fits (1.3): `id`, `user.id`, `state` as the verdict, `commit_id`, `body` and its key, `submitted_at` as `at`, `official`, provenance and revision; `more` when the page came full or was cut; `cut` when it was |
| `Read::Statuses { commit, page }` | `GET R/commits/<commit>/status?page=<page>&limit=<size>`; for each status whose link names a run of this repository's Actions, the job resolved as `Checks` resolves it | each context's latest status on the page: `updated_at`, `context`, `status`, `description`, `target_url` as its link (links are absolute, below), its `job` as for `Checks`; `more` when the page came full, never by `total_count` (1.1); `ci` the page's combined `state` and `total_count`, the commit's verdict only on a first page that came short (1.1) |
| `Read::Remarks { number, review, page, skip }` | `GET R/pulls/<number>/reviews/<review>/comments` (unpaged) | the `page`th slice of `<size>`, past its first `skip` comments, each kept whole while it fits (1.3): `id`, `user.id`, `path`, `position` as `line`, `body`; `more` when a comment follows what was kept; `cut` when the answer's bytes cut the slice |
| `Read::Branch { branch }` | `GET R/branches/<branch>` | `commit.id` |
| `Read::Branches { prefix }` | `GET R/git/refs/heads/<prefix>` | branch names from matching `refs/heads/` refs, as many as `answer_bytes` holds by the measure (`README.md`, 3.7), with no row bound; no matches (`404`) gives an empty list and keeps the repository row; matching names past `answer_bytes`, `TooLarge` |
| `Read::PullFiles { number, head, page, skip, sides }` | `GET R/pulls/<number>`; `GET R/pulls/<number>/files?page=<page>&limit=<size>`; when `sides`, for each file past the page's first `skip`, while the files kept fit (1.3), `GET R/raw/<path>?ref=<merge_base>` (but for `added`; at `previous_filename` for `renamed`) and `GET R/raw/<filename>?ref=<head>` (but for `deleted`) | `head`; each file's `filename`, with, when `sides`, its contents at the merge base and at the head, a side absent when the file's `status` says so or its read answers `404`, and without, its name alone, both sides `None`; `more` from `X-HasMore`, or when cut; `cut` when a file's sides would pass what is left of the answer, that file and those after it left to the next read; a `head.sha` other than `head` fails the call `Stale` before any file is read |
| `Read::Compare { before, after }` | `GET R/compare/<before>...<after>?verification=false`; `GET R/compare/<after>...<before>?files=false&verification=false` | `files[].filename`, in Forgejo's order, kept while they fit `rows` and what is left of the answer's bound, and each counted in `changed`; `ahead`, the first comparison's `total_commits`, and `behind`, the second's; no commit's id and no file's side is kept or read |
| `Read::Range { before, after }` | `GET R/compare/<before>...<after>?files=false&verification=false` | `total_commits` as `commits`; the `commits` list skipped as it streams, none of it kept |
| `Read::Checks { commit }` | `GET R/commits/<commit>/status?page=<n>&limit=<size>`, every page; for each run the statuses' links name, pending, passed or failed, once, in the order its first status comes: `GET R/actions/runs?head_sha=<commit>&run_number=<n>&page=1&limit=1`, then `GET R/actions/runs/<id>/jobs`; every status linking to that run takes its job from that one list | each context's latest status as for `Statuses`, its `job` the commit, the run's `id`, the job's `id` and its `attempt` when found, none otherwise: the link's index of the job in its run is never taken for the job's `id` |
| `Read::Job { attempt, max_bytes, part }` | `GET R/actions/runs/<run>`; `GET R/actions/jobs/<job>`; `GET R/actions/jobs/<job>/logs?attempt=<attempt>&step=<number>` with `Range: bytes=-<max_bytes>`, `step` the failed step's, absent when no step failed; for `Start`, `GET R/actions/jobs/<job>/logs?attempt=<attempt>` with `Range: bytes=0-<max_bytes − 1>` and no `step` | the failed step, its `number` and `name` (1.3), none when no step failed; for `End`, the last `max_bytes` bytes of that step's log, or of the job's whole log when no step failed, `truncated` when `Content-Range` names bytes before them; for `Start`, the job's whole log's first `max_bytes` bytes, `truncated` when `Content-Range` names bytes after them; `MissingJob` when a read answers `404`, the run's `commit_sha` is not the attempt's head, or the job's `run_id` or `attempt` is not the attempt's |
| `Read::File { head, path, max_bytes }` | `GET R/raw/<path>?ref=<head>` with `Range: bytes=0-<max_bytes>` | the first `max_bytes` bytes; `truncated` when more came |
| `Read::Protection { branch }` | `GET R/branches/<branch>`; then `GET R/branch_protections/<effective_branch_protection_name>` when it names a rule, or `GET R/branch_protections` when it names none | `rule_name`, `status_check_contexts` when `enable_status_check`, `required_approvals`, `dismiss_stale_approvals`, `require_signed_commits`; none when no rule applies; a `403` (a writer who is not an admin) `Forbidden` |
| `Read::Settings` | none when the repository's row holds its settings (connection.md, 4.2); else `GET /api/v1/repositories/<id>` | `default_branch`; `allow_merge_commits`, `allow_squash_merge`, `allow_rebase`; the repository's row kept afresh |
| `Read::Collaborators { page }` | `GET R/collaborators?page=<page>&limit=<size>`; for each user, `GET R/collaborators/<login>/permission` | each user's `id` and permission; `more` when the page came full |
| `Read::Permission { user }` | `GET R/collaborators/<login>/permission` | `permission`: `none`, `read`, `write`, `admin`, and `owner` as `Admin`; the `id` and `login` of the answer's `user` |
| `Read::Locate { owner, name }` | `GET /api/v1/repos/<owner>/<name>` | `Located { repository }` from `id`; the repository's row kept |
| `Write::CreateIssue { key, title, body, labels }` | `GET R/labels` for the labels' ids, when it names any; `POST R/issues` `{title, body, labels}`, the body behind the key's marker | `Created` from `number`; `Missing` for a label the repository lacks, before any write (Forgejo drops an unknown id silently) |
| `Write::Labels { number, labels }` | `GET R/labels`; `PUT R/issues/<number>/labels` `{labels: [ids]}` | `Done`; `Missing` for a label the repository lacks, before any write |
| `Write::CreateLabel { name, color }` | `GET R/labels`, every page; `POST R/labels` `{name, color}` only when no label has the name (Forgejo makes a second label of a name, never answering `409`); an uncertain answer settled by finding the name again | `Done`; `Exists` when a label has the name |
| `Write::OpenPull { title, body, head, base }` | `GET R/branches/<head>`; `GET R/branches/<base>`; `GET R/compare/<base's commit.id>...<head's commit.id>?files=false&verification=false`, of which only `total_commits` is kept; when it is 0, nothing more; else `POST R/pulls` `{head, base, title, body}` | `Created` from `number`; `NothingToMerge` when `total_commits` is 0, the pull request not written |
| `Write::Post { number, key, body }` | `POST R/issues/<number>/comments` `{body}`, behind the key's marker | `Commented` from `id` |
| `Write::Review { number, key, verdict, body }` | `POST R/pulls/<number>/reviews` `{event, body}`, `event` `APPROVED`, `REQUEST_CHANGES` or `COMMENT`, the body behind the key's marker | `Reviewed` from `id` |
| `Write::Edit { number, title, body }` | `PATCH R/issues/<number>` `{title, body}`, each only when given | `Done` |
| `Write::SetReviewers { number, reviewers }` | `GET R/pulls/<number>`; `POST R/pulls/<number>/requested_reviewers` `{reviewers: [logins]}` for those missing; `DELETE R/pulls/<number>/requested_reviewers` with the same body for those extra | `Done` |
| `Write::Close { number }` | `PATCH R/issues/<number>` `{state: closed}` | `Done` |
| `Write::Reopen { number }` | `PATCH R/issues/<number>` `{state: open}` | `Done` |
| `Write::Merge { number, head, how }` | `POST R/pulls/<number>/merge` `{Do, head_commit_id: <head>}`, `Do` the repository's style or `fast-forward-only`; `GET R/pulls/<number>` | `Merged` from `merge_commit_sha` (the merge answers with no body); a fast-forward-only merge whose head no longer contains the base's tip answers `500` ("DivergingFastForwardOnly"), the base unchanged, which the dialect keeps as `Timeout`, so the client settles it by its find (client.md, 7.5) |
| `Write::Update { number, style }` | `POST R/pulls/<number>/update?style=merge\|rebase` | `Done` |
| `Write::Status { commit, context, check, description, link }` | `POST R/statuses/<commit>` `{state: pending\|success\|failure, context, description, target_url}`, the last two only when not empty | `Done` |
| `Read::Tree { object, page }` | `GET R/git/trees/<object>?page=<page>&per_page=<size>` | each entry's `path` as its name, `mode`, `type` (`blob`, `tree`, `commit`) and `sha`, in git's order; `more` when the page came full (Forgejo's `truncated` stays true on a short last page, so it is not read); a `tree` of `null` past the last page is an empty page |
| `Read::Runs { workflow, commit, page }` | `GET R/actions/runs?workflow_id=<workflow>&head_sha=<commit>&page=<page>&limit=<size>` (the dialect still keeps only the rows whose workflow file is the call's, so a forge that ignores the filter answers the same) | each run as a `Run` without its jobs, newest first, its event from `trigger_event`, its user from `trigger_user.id`, made at `created`; `more` when the provider's page came full, counted on its rows before the dialect's filter |
| `Read::Run { run }` | `GET R/actions/runs/<run>`; `GET R/actions/runs/<run>/jobs`; for each failed job, `GET R/actions/jobs/<job>` for its steps | the `Run` with its jobs and each failed job's failed step (1.2's notes, "A failed job's log") |
| `Read::Artifact { run, name, max_bytes }` | `GET R/actions/runs/<run>/artifacts?name=<name>`; `GET R/actions/artifacts/<id>/zip` | the archive's one entry, inflated where deflated, its first `max_bytes` bytes; `truncated` when more came; `Missing` when no artifact has the name; `Refused` for an archive of several entries or another compression |
| `Read::Commits { before, after, page }` | `GET R/compare/<before>...<after>?files=false&verification=false`, its `commits` from the page's offset | each commit's `sha`, `parents[].sha` and `commit.message`, oldest first; `more` when a page's worth is left. Forgejo lists them newest first, so a range longer than the answer's rows and bytes hold keeps its oldest commits, as many as they hold, giving up the newest as older ones come; a page those hold whole is answered, with `more`, and a page they don't, or a range whose oldest commit alone passes the answer, is `TooLarge` |
| `Write::Dispatch { workflow, branch, inputs }` | `POST R/actions/workflows/<workflow>/dispatches` `{ref: <branch>, inputs}` | `Done`; `Missing` for a `404` (no such workflow); `Refused` for a `400` (no such branch: "could not expand reference") |
| `Write::Rerun { attempt }` | `POST R/actions/jobs/<attempt's job>/rerun` | `Done`; `Refused` for a `400` (the run has not completed) |
| `Write::CreateBranch { branch, commit }` | `POST R/branches` `{new_branch_name: <branch>, old_ref_name: <commit>}` | `Branch(Created)`; a `409` (the branch exists) `Branch(Exists)` |
| `Write::DeleteBranch { branch }` | `DELETE R/branches/<branch>` | `Done` |

`Branches` requires a nonempty prefix bounded like a branch name. An empty
prefix ends with `Error::Empty` before repository resolution or any GET.
Only matching names count against the answer-byte bound, and no row
bound applies to them: a name is small, so a prefix's names are as many
as `answer_bytes` holds (`README.md`, 3.7).

Notes on the table:

- **The keep-up listing** is the only listing of items: always
  `state=all`, always `since`, so its rows are what changed since the
  last pass and its cost follows the rate of change, not the number of
  pull requests held (client.md, section 4). A find for an issue
  lists from the entry's start, `type=issues`. Nothing is listed by
  label.
- **`sort=leastupdate`** is Forgejo's least-recently-updated order. Its
  default is `latest`, newest *created* first, and a GitHub-style
  `sort=updated&direction=asc` is not in its vocabulary: a client that
  sends the wrong words gets the wrong order with no error. The fake
  forge answers the same way (../testing/fakes/forge.md, section 4), and section 2
  names the real one's facts.
- **An item's comments are not paged by Forgejo.** The call reads the
  thread Forgejo sends, keeps the comments with ids above `after` as the
  answer holds them, and skips the rest as they stream, bodies unread;
  the next call names the last comment kept as its `after`. Forgejo
  filters the thread by `since`, sending only comments updated at or
  after it, so a thread read again with the owner's `since` (`README.md`, 3.2) sends
  what changed since, not the whole thread; a comment at or below
  `after` that it still sends, one edited since, is skipped as before.
- **Issues and pull requests share one numbering,** so an `Item` read of
  a pull request is its conversation, and `Edit`, `Close` and `Reopen`
  are the issue routes for both.
- **Reviews** are mapped by their state: `APPROVED` to `Approve`,
  `REQUEST_CHANGES` to `RequestChanges`, `COMMENT` to `Comment`. Pending
  reviews and review requests (`PENDING`, `REQUEST_REVIEW`) are not
  reviews, and are skipped, though they count towards a page coming
  full. A dismissed review is carried as `Comment`: its words and its
  key stay, its verdict does not (`README.md`, section 10).
- **CI.** A combined state of `failure` or `error` is `Failed`, `pending`
  is `Pending`, `success`, `warning` and `skipped` are `Passed`, and a
  `total_count` of 0 is `None`, whatever the state says. A status's own
  state maps the same way to a `Check`. Forgejo's Actions report each
  job as a status on the commit it ran on, so CI is read from statuses
  alone, never from a history of runs. `Checks` and each requested
  `Statuses` page resolve every status whose link names a run of the
  repository's Actions, pending, passed or failed, to the run's and the
  job's API ids and the job's attempt, two requests per run the call's
  statuses link to, made once: the statuses of one run, one per job,
  take their jobs from the one list read for it, and a run not found is
  not asked again in the call. An unresolved link names no job, and a
  link's index of the job in its run is never taken for its id. `Pull`'s
  CI resolves none.
- **Job attempts.** An Actions status links to its job's page,
  `<the path of the repository's html_url>/actions/runs/<run
  number>/jobs/<job index>`, relative to the forge's origin, as Forgejo
  writes it (section 2); the same link with the whole `html_url` before
  it is read the same way. The run's number is its `index_in_repo`,
  which the runs' listing filters by (`run_number`); the job is the one
  at that index of the run's jobs; its attempt is the job's `attempt`. A
  link of another shape, one that starts `//` or names another scheme or
  host, or a run or a job not found, leaves the status's `job` none, and
  the call answers all the same.
- **A failed job's log is its failed step's end.** Forgejo answers a job
  (`actions/jobs/<job>`) with its steps, those of its latest attempt:
  number 0 is the runner's set-up (`Set up job`), the last its
  completion (`Complete job`), and the workflow's own steps are between.
  The failed step is the first whose `status` is `failure`: the
  completion step reports `failure` whenever the job fails, so it is the
  failed step only when none before it failed. The steps are read as
  they stream, and only the failed one is kept, the others skipped as
  they pass: a job of any number of steps is read, its steps costing
  time, not memory. Its log alone is read (`step`), and of it the end,
  by a suffix range, since a step tells its failure last. A job with no
  step failed (one cancelled, or one whose runner was lost) is read
  whole, its end the same way; one never given to a runner has no log
  (`404`, `MissingJob`). The job's `attempt` is its latest: one other
  than the call's (the job ran again since) is `MissingJob`, as a log
  Forgejo no longer keeps is (`404`). A `200` is read through, its last
  `max_bytes` bytes kept as it streams, within the component's
  `body_bytes_in` like any body (connection.md, section 7); a
  range over an empty log is an empty log (section 2). The log's search
  (`q`) and its NDJSON form are not read: neither answers a range.
- **Links are absolute.** A status's link is carried to the owner
  whole: one relative to the forge's origin (a `target_url` that starts
  with one `/`) gets the origin (scheme, host and port) of the
  repository's `html_url` before it, so a person or an agent reading it
  has a link that opens; any other is carried as Forgejo wrote it.
- **Who posted a status is not carried.** A status written by Actions
  has no `creator`, and nothing in the vocabulary reads a status's
  poster: CI is read by context and state. `Status` has no author.
- **Contents of a pull request's files** are read file by file, two
  reads each, since the vocabulary carries each changed file whole on
  both sides (`File`); the reads count in the call's cost. A side that
  does not fit what is left of the answer's bound fails the call as
  `TooLarge`. A comparison reads no file's contents: its files are
  Forgejo's names, from the two commits' merge base, and a reader asks
  for a file's contents by path (`File`).
- **Merges.** `Do` is the repository's `default_merge_style`, kept with
  its path (connection.md, section 4): the merge style is the
  repository's own. `head_commit_id` makes the merge conditional on the
  head the call names, which Forgejo refuses when it moved (`Stale`).
  The merge answers with no body; the read after it gives the merge's
  commit. If that read fails, the call answers `Timeout`: the merge's
  outcome is unknown, and the client finds it by reading the pull
  request (client.md, section 7).
- **Updates** merge the base into the branch (`style=merge`), or rebase
  its commits onto the base (`style=rebase`), as the call names (`README.md`, 3.2).
  Both are idempotent transitions; a late rebase may drop a later push
  (client.md, 7.3).
- **A pull request for a head and base.** Forgejo keeps at most one pull
  request open for a head and a base. Its lookup, `pulls/{base}/{head}`,
  names one of several, the oldest (section 2). So `PullFor` answers the
  open one: the lookup's, or, when the lookup names one closed or
  merged, the row of the open pull requests' listing whose `head.ref`,
  `head.repo_id` and `base.ref` are the call's head, its repository and
  its base, every other row skipped as it streams. The listing is asked
  with the `head` and `base` filters, the call's branches, so Forgejo
  sends the rows of those branches alone; the row check stays, so a
  forge that ignores a filter answers the same, at a scan's cost
  (section 2). With none open, it answers the one the lookup
  named. An owner that opens a pull request only when none exists for its
  branches, and reopens it when it was closed, follows the oldest, and
  the listing is read only when that one is closed or merged, or another
  hand opened one for its branches. Each page counts in the call's cost;
  the call's deadline bounds them.
- **Comparisons ask for what they keep.** Every comparison sends
  `verification=false`, since no commit's signature is read; one read
  only for its `total_commits` (`OpenPull`'s, `Range`'s and `Compare`'s
  second) also sends `files=false`, so Forgejo lists no commit's files.
  Forgejo still lists the range's commits, which such a read skips as
  they stream: `Range` keeps nothing but the count, so containment and
  emptiness cost one request and no file side, however long the range.
  `files=false` leaves the comparison's own `files` empty too (section
  2), so `Compare`'s first comparison, which keeps them, never sends it.
  The answer is decoded in the same way: what the call does not keep
  is skipped as it streams.
- **Opening compares first.** Forgejo opens a pull request whose head
  adds nothing to its base (section 2), which the vocabulary refuses as
  `NothingToMerge`. So `OpenPull` reads both branches and compares their
  commits before it writes: a head with no commit its base lacks
  (`total_commits` 0, a head equal to its base included) fails the call
  `NothingToMerge`, and nothing is written. The three reads count in the
  call's cost, and fail as 1.4 says for them. A base that moves to
  contain the head between the comparison and the `POST` still gets a
  pull request with nothing to merge; that window is the only way a
  pull request with nothing to merge is opened.
- **Keys.** `CreateIssue`, `Post` and `Review` write the key's marker,
  then the text (`README.md`, section 5). A pull request's key is its branches.
  `Edit` writes the title and body it is given.
- **Several requests, one call.** `Item`, `Pull`, `PullFor`,
  `PullFiles`, `Compare`, `Checks`, `Job`, `Protection`,
  `Collaborators`, `OpenPull`, `SetReviewers` and `Merge` make more than
  one; a path to resolve or a login to look up adds one. A `404` on a
  file's side, or on a failing status's run or job, ends nothing: it is
  an answer, and the repository's path stays. The client asks for what
  it needs, and the answer's cost tells its budget what that took
  (client.md, section 6).
- **Success.** Each request succeeds with one status; any other `2xx`
  is a malformed answer (1.3). A read's is `200`, and a raw file's or a
  log's `200` or `206`. A write's is `201`, as Forgejo answers its
  creations and its edits of an issue (`Edit`, `Close`, `Reopen`), but
  `200` for `Review`, `Merge` and `Update`, and `204` for
  `DeleteBranch`; `SetReviewers` adds with `201` and removes with
  `204`. The reads a write makes succeed as reads do.

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
  say so (`truncated`); a side of `PullFiles` is never cut (1.2), and a comparison's names stop at the answer's bound, counted on in `changed`.
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
  `modified` or `changed`, `renamed` or `copied`; `removed` is what a real
  Forgejo's comparison says of a file gone, and reads as `deleted` does. Any
  other word is a malformed answer.
- **What may follow** is told by the page coming full, `X-HasMore`, or
  `total_count`. For a call made of several requests, `more` is its
  listing's.
- **Absent headers.** An `Items` answer without `Date` is malformed,
  since its `now` becomes the next listing's `since`. A `429` without
  `Retry-After` is `RateLimited { after: 0 }`, and a listing without
  `X-HasMore` has more only when it came full.
- **A malformed answer** (a success status other than its request's,
  1.2; a document that does not decode, a field missing, a number out
  of range, a commit id of the wrong length, a string past its cap, a
  body past the component's `body_bytes_in`, a log read through
  included; a repository read by its id answering another) fails a
  write as `Timeout`: the request reached the forge, so what it asked
  may have been done. It fails a read as `InvalidAnswer`. Either way the
  connection is closed, since what followed on it can no longer be
  trusted. A repository id past the vocabulary's 32 bits fails the call
  as `TooLarge` (`Forge`, `limit` 4,294,967,295, `size` the id).

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
| every other `404` that ends the call (one 1.2 says ends nothing is an answer, and the path stays), including repository resolution, its own GET and listings | `Missing`, the repository's path dropped |
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

- **The plan gives the `Error`,** from what it hears (`README.md`, 4.2). The
  component tells it how an exchange failed, `Unseen` or `Seen`, from
  skein-http's errors and its own deadlines (connection.md,
  section 8).
- **Unavailable is provable only before the first byte.** skein-http's
  client tells a request the server never saw (`Closed`) from one it
  may have read (`Truncated`, `Stream`; http.md, section 3.4). A
  connection kept alive that the forge closed just as a request went out
  may have been read: it is a `Timeout`. To make that rare, an idle
  connection is closed before the forge's keep-alive timeout would
  close it (`idle`, connection.md, section 7).

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
- **Not errors:** a `409` on `CreateBranch` is `Branch(Exists)`, which
  the client follows with a read of the branch; a `404` on
  `DeleteBranch` is `Missing`, which the client counts as the branch
  gone (client.md, section 7). Forgejo answers the deletion of a
  branch that is not there `500` with an empty body (section 2): a
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
release (`README.md`, 4.3). Each is named by its fact in the conformance's record
(../testing/fakes/forge.md, section 7), where it `holds`; a fact marked *fails* is
phrased the other way there, and its failing is what the dialect relies
on. What no run records yet has its own bullet, near the end.

- **Every call of 1.2, made by the component.** Each read and write
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
    come whole however many there are (1.2, `Item` and `Remarks`);
  - that a job no runner took has no steps, and that its log, and a log
    Forgejo no longer keeps, answer `404` (1.2, "A failed job's log");
  - that the open pull requests' listing honours `head` and `base`, an
    issue's comments `since`, and the Actions runs' listing
    `workflow_id` (1.2). Each narrows what Forgejo sends, and the
    dialect's own row checks stay, so where one is ignored the answer is
    the same and only its cost grows.
- **Actions, trees and the turn, recorded** (Forgejo 17.0.1 and runner
  13.2.1, the conformance's `landings` facts: 24 hold, 7 recorded as
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
  `tree` `null` past it.
- **Still to record before a plan relies on them:** a dispatch's inputs
  reaching the run; a job's rerun answering `204`, and `400` while its
  run has not completed; a run's artifact listed by name and downloaded
  as a zip; a comparison's commits oldest first with their messages; an
  update with `style=merge` of a head already holding its base (the fake
  answers `Done`, unchanged, as `style=rebase` is recorded to); what a
  rebase's push does to a push made while it works (client.md, 7.3
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
