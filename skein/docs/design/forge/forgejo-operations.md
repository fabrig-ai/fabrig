# Forgejo, as the kit uses it: operations

## 1. Each operation

What each call sends, in order; R is `/api/v1/repos/{owner}/{name}`, from
the repository's path, and `<size>` the page size of 5.1. Calls of more
than one request make them in turn on one connection, and end on the
first failure, but where a row says otherwise.

| Call | Requests | Answer from |
|---|---|---|
| `Read::Items { since, page, kind }` | `GET R/issues?state=all&type=issues\|pulls&since=<since>&sort=leastupdate&page=<page>&limit=<size>`, `type` only with a kind | each row a `Summary` (`forgejo.md`, 1.3): `number`; a `pull_request` object makes it a pull; `state`; `user.id`; `labels[].name`, for display; `updated_at`; `title`; `body` and its key; `more` when the page came full; `now` from `Date` |
| `Read::Item { number, after, since }` | `GET R/issues/<number>`; `GET R/issues/<number>/comments?since=<since>` (unpaged), `since` only when not 0 | the issue as a `Summary`; its comments with ids above `after`, oldest first, as many as the answer holds (`forgejo.md`, 1.3): `id`, `user.id`, `created_at`, `body` and its key, its provenance and revision; `more` when one is left |
| `Read::Pull { number, ci }` | `GET R/pulls/<number>`; then, only when `ci`, `Read::Checks`' status pages for `head.sha`, from page 1, until one comes short, with none of its job reads | `number`; `state`; `head.ref`, `base.ref`; `head.sha` as `commit`; `base.sha` as `base_commit`, none when empty; `merge_commit_sha` when `merged`; `mergeable`; `requested_reviewers[].id`; `size` from `changed_files` as `files`, `additions` and `deletions`, at `head.sha`, none when one is absent, null or no whole number, as Forgejo leaves them out where it could not count the change (vocabulary.md, 1.6); CI, when `ci`, the first page's combined `state` when that page came short and its `state` is not `warning` (`forgejo.md`, 1.1), else folded from every context's latest status over every page, as `Checks` folds them, no row kept; `Ci::None` without `ci` |
| `Read::PullFor { head, base }` | `GET R/pulls/<base>/<head>`; when it names a pull request that is not open, `GET R/pulls?state=open&head=<head>&base=<base>&sort=oldest&page=<n>&limit=<size>` from page 1, until a row has the head and base or a page comes short; no status read | the open pull request for the head and base, else the one the lookup named; as `Pull` without `ci`; a lookup answered `404` names none, `Missing`, and keeps the repository row (`forgejo.md`, 1.4) |
| `Read::Reviews { number, page, skip }` | `GET R/pulls/<number>/reviews?page=<page>&limit=<size>` | submitted reviews, oldest first, past the page's first `skip` rows, each kept whole while it fits (`forgejo.md`, 1.3): `id`, `user.id`, `state` as the verdict, `commit_id`, `body` and its key, `submitted_at` as `at`, `official`, provenance and revision; `more` when the page came full or was cut; `cut` when it was |
| `Read::Statuses { commit, page }` | `GET R/commits/<commit>/status?page=<page>&limit=<size>`; for each status whose link names a run of this repository's Actions, the job resolved as `Checks` resolves it | each context's latest status on the page: `updated_at`, `context`, `status`, `description`, `target_url` as its link (links are absolute, below), its `job` as for `Checks`; `more` when the page came full, never by `total_count` (`forgejo.md`, 1.1); `ci` the page's combined `state` and `total_count`, the commit's verdict only on a first page that came short (`forgejo.md`, 1.1) |
| `Read::Remarks { number, review, page, skip }` | `GET R/pulls/<number>/reviews/<review>/comments` (unpaged) | the `page`th slice of `<size>`, past its first `skip` comments, each kept whole while it fits (`forgejo.md`, 1.3): `id`, `user.id`, `path`, `position` as `line`, `body`; `more` when a comment follows what was kept; `cut` when the answer's bytes cut the slice |
| `Read::Branch { branch }` | `GET R/branches/<branch>` | `commit.id` |
| `Read::Branches { prefix }` | `GET R/git/refs/heads/<prefix>` | branch names from matching `refs/heads/` refs, as many as `answer_bytes` holds by the measure (`vocabulary.md`, 1.7), with no row bound; no matches (`404`) gives an empty list and keeps the repository row; matching names past `answer_bytes`, `TooLarge` |
| `Read::PullFiles { number, head, page, skip, sides }` | `GET R/pulls/<number>`; `GET R/pulls/<number>/files?page=<page>&limit=<size>`; when `sides`, for each file past the page's first `skip`, while the files kept fit (`forgejo.md`, 1.3), `GET R/raw/<path>?ref=<merge_base>` (but for `added`; at `previous_filename` for `renamed`) and `GET R/raw/<filename>?ref=<head>` (but for `deleted`) | `head`; each file's `filename`, with, when `sides`, its contents at the merge base and at the head, a side absent when the file's `status` says so or its read answers `404`, and without, its name alone, both sides `None`; `more` from `X-HasMore`, or when cut; `cut` when a file's sides would pass what is left of the answer, that file and those after it left to the next read; a `head.sha` other than `head` fails the call `Stale` before any file is read |
| `Read::Compare { before, after }` | `GET R/compare/<before>...<after>?verification=false`; `GET R/compare/<after>...<before>?files=false&verification=false` | `files[].filename`, in Forgejo's order, kept while they fit `rows` and what is left of the answer's bound, and each counted in `changed`; `ahead`, the first comparison's `total_commits`, and `behind`, the second's; no commit's id and no file's side is kept or read |
| `Read::Range { before, after }` | `GET R/compare/<before>...<after>?files=false&verification=false` | `total_commits` as `commits`; the `commits` list skipped as it streams, none of it kept |
| `Read::Checks { commit }` | `GET R/commits/<commit>/status?page=<n>&limit=<size>`, every page; for each run the statuses' links name, pending, passed or failed, once, in the order its first status comes: `GET R/actions/runs?head_sha=<commit>&run_number=<n>&page=1&limit=1`, then `GET R/actions/runs/<id>/jobs`; every status linking to that run takes its job from that one list | each context's latest status as for `Statuses`, its `job` the commit, the run's `id`, the job's `id` and its `attempt` when found, none otherwise: the link's index of the job in its run is never taken for the job's `id` |
| `Read::Job { attempt, max_bytes, part }` | `GET R/actions/runs/<run>`; `GET R/actions/jobs/<job>`; `GET R/actions/jobs/<job>/logs?attempt=<attempt>&step=<number>` with `Range: bytes=-<max_bytes>`, `step` the failed step's, absent when no step failed; for `Start`, `GET R/actions/jobs/<job>/logs?attempt=<attempt>` with `Range: bytes=0-<max_bytes − 1>` and no `step` | the failed step, its `number` and `name` (`forgejo.md`, 1.3), none when no step failed; for `End`, the last `max_bytes` bytes of that step's log, or of the job's whole log when no step failed, `truncated` when `Content-Range` names bytes before them; for `Start`, the job's whole log's first `max_bytes` bytes, `truncated` when `Content-Range` names bytes after them; `MissingJob` when a read answers `404`, the run's `commit_sha` is not the attempt's head, or the job's `run_id` or `attempt` is not the attempt's |
| `Read::File { head, path, max_bytes }` | `GET R/raw/<path>?ref=<head>` with `Range: bytes=0-<max_bytes>` | the first `max_bytes` bytes; `truncated` when more came |
| `Read::Protection { branch }` | `GET R/branches/<branch>`; then `GET R/branch_protections/<effective_branch_protection_name>` when it names a rule, or `GET R/branch_protections` when it names none | `rule_name`, `status_check_contexts` when `enable_status_check`, `required_approvals`, `dismiss_stale_approvals`, `require_signed_commits`; none when no rule applies; a `403` (a writer who is not an admin) `Forbidden` |
| `Read::Settings` | none when the repository's row holds its settings (connection-calls.md, 1.2); else `GET /api/v1/repositories/<id>` | `default_branch`; `allow_merge_commits`, `allow_squash_merge`, `allow_rebase`; the repository's row kept afresh |
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
| `Write::Close { number }` | `PATCH R/issues/<number>` `{state: closed}`; where `forgejo.md`, 1.4 makes its answer `Refused`, `GET R/issues/<number>` once | `Done`; `Done` too where that read finds the item closed, a merged pull request among them, else `Refused` (`forgejo.md`, 1.4, *A close of a closed item*) |
| `Write::Reopen { number }` | `PATCH R/issues/<number>` `{state: open}` | `Done` |
| `Write::Merge { number, head, how }` | `POST R/pulls/<number>/merge` `{Do, head_commit_id: <head>}`, `Do` the repository's style or `fast-forward-only`; for `Recorded { commit }`, `{Do: manually-merged, MergeCommitID: <commit>}`; `GET R/pulls/<number>` | `Merged` from `merge_commit_sha` (the merge answers with no body); a fast-forward-only merge whose head no longer contains the base's tip answers `500` ("DivergingFastForwardOnly"), the base unchanged, which the dialect keeps as `Timeout`, so the client settles it by its find (client-effects.md, 1.5); a record whose commit the base lacks, or in a repository whose `allow_manual_merge` is off, is refused, `Refused` |
| `Write::Update { number, style }` | `POST R/pulls/<number>/update?style=merge\|rebase` | `Done` |
| `Write::Status { commit, context, check, description, link }` | `POST R/statuses/<commit>` `{state: pending\|success\|failure, context, description, target_url}`, the last two only when not empty | `Done` |
| `Read::Tree { object, page }` | `GET R/git/trees/<object>?page=<page>&per_page=<size>` | each entry's `path` as its name, `mode`, `type` (`blob`, `tree`, `commit`) and `sha`, in git's order; `more` when the page came full (Forgejo's `truncated` stays true on a short last page, so it is not read); a `tree` of `null` past the last page is an empty page |
| `Read::Runs { workflow, commit, page }` | `GET R/actions/runs?workflow_id=<workflow>&head_sha=<commit>&page=<page>&limit=<size>` (the dialect still keeps only the rows whose workflow file is the call's, so a forge that ignores the filter answers the same) | each run as a `Run` without its jobs, newest first, its event from `trigger_event`, its user from `trigger_user.id`, made at `created`; `more` when the provider's page came full, counted on its rows before the dialect's filter |
| `Read::Run { run }` | `GET R/actions/runs/<run>`; `GET R/actions/runs/<run>/jobs`; for each failed job, `GET R/actions/jobs/<job>` for its steps | the `Run` with its jobs and each failed job's failed step (1's notes, "A failed job's log") |
| `Read::Artifact { run, name, max_bytes }` | `GET R/actions/runs/<run>/artifacts?name=<name>`; `GET R/actions/artifacts/<id>/zip` | the archive's one entry, inflated where deflated, its first `max_bytes` bytes; `truncated` when more came; `Missing` when no artifact has the name; `Refused` for an archive of several entries or another compression |
| `Read::Commits { before, after, page }` | `GET R/compare/<before>...<after>?files=false&verification=false`; for a page past the first, that request twice: the first for its `total_commits` alone, its `commits` skipped as they stream, and, unless the page starts past the range's end, the second for the page's commits | each commit's `sha`, `parents[].sha` and `commit.message`, oldest first; `more` when commits are left past the page. Forgejo lists them newest first, so a page's commits are those at its places counted from the oldest: the first page the oldest `<size>` of them, kept as they stream, and a later page those its offset and `<size>` place by the first request's count. Every page of a range is answered, however long the range, holding only its own commits; a page starting past the range's end is empty, with no `more`; a page whose commits the answer's rows and bytes don't hold whole is `TooLarge`, and a second request whose `total_commits` differs from the first's is `InvalidAnswer` |
| `Write::Dispatch { workflow, branch, inputs }` | `POST R/actions/workflows/<workflow>/dispatches` `{ref: <branch>, inputs}` | `Done`; `Missing` for a `404` (no such workflow); `Refused` for a `400` (no such branch: "could not expand reference") |
| `Write::Rerun { attempt }` | `POST R/actions/jobs/<attempt's job>/rerun` | `Done`; `Refused` for a `400` (the run has not completed) |
| `Write::CreateBranch { branch, commit }` | `POST R/branches` `{new_branch_name: <branch>, old_ref_name: <commit>}` | `Branch(Created)`; a `409` (the branch exists) `Branch(Exists)` |
| `Write::DeleteBranch { branch }` | `DELETE R/branches/<branch>` | `Done` |

`Branches` requires a nonempty prefix bounded like a branch name. An empty
prefix ends with `Error::Empty` before repository resolution or any GET.
Only matching names count against the answer-byte bound, and no row
bound applies to them: a name is small, so a prefix's names are as many
as `answer_bytes` holds (`vocabulary.md`, 1.7).

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
  forge answers the same way (../testing/fakes/forgejo.md, section 1), and `forgejo.md`, section 2
  names the real one's facts.
- **An item's comments are not paged by Forgejo.** The call reads the
  thread Forgejo sends, keeps the comments with ids above `after` as the
  answer holds them, and skips the rest as they stream, bodies unread;
  the next call names the last comment kept as its `after`. Forgejo
  filters the thread by `since`, sending only comments updated at or
  after it, so a thread read again with the owner's `since` (`vocabulary.md`, 1.2) sends
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
  writes it (`forgejo.md`, section 2); the same link with the whole `html_url` before
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
  `body_bytes_in` like any body (connection-machines.md, section 1); a
  range over an empty log is an empty log (`forgejo.md`, section 2). The log's search
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
  its path (connection-calls.md, section 1): the merge style is the
  repository's own. `head_commit_id` makes the merge conditional on the
  head the call names, which Forgejo refuses when it moved (`Stale`).
  The merge answers with no body; the read after it gives the merge's
  commit. If that read fails, the call answers `Timeout`: the merge's
  outcome is unknown, and the client finds it by reading the pull
  request (client-effects.md, section 1).
- **Updates** merge the base into the branch (`style=merge`), or rebase
  its commits onto the base (`style=rebase`), as the call names (`vocabulary.md`, 1.2).
  Both are idempotent transitions; a late rebase may drop a later push
  (client-effects.md, 1.3).
- **A pull request for a head and base.** Forgejo keeps at most one pull
  request open for a head and a base. Its lookup, `pulls/{base}/{head}`,
  names one of several, the oldest (`forgejo.md`, section 2). So `PullFor` answers the
  open one: the lookup's, or, when the lookup names one closed or
  merged, the row of the open pull requests' listing whose `head.ref`,
  `head.repo_id` and `base.ref` are the call's head, its repository and
  its base, every other row skipped as it streams. The listing is asked
  with the `head` and `base` filters, the call's branches, so Forgejo
  sends the rows of those branches alone; the row check stays, so a
  forge that ignores a filter answers the same, at a scan's cost
  (`forgejo.md`, section 2). With none open, it answers the one the lookup
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
  `files=false` leaves the comparison's own `files` empty too (`forgejo.md`, section 2
  ), so `Compare`'s first comparison, which keeps them, never sends it.
  The answer is decoded in the same way: what the call does not keep
  is skipped as it streams.
- **Opening compares first.** Forgejo opens a pull request whose head
  adds nothing to its base (`forgejo.md`, section 2), which the vocabulary refuses as
  `NothingToMerge`. So `OpenPull` reads both branches and compares their
  commits before it writes: a head with no commit its base lacks
  (`total_commits` 0, a head equal to its base included) fails the call
  `NothingToMerge`, and nothing is written. The three reads count in the
  call's cost, and fail as `forgejo.md`, 1.4 says for them. A base that moves to
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
  is a malformed answer (`forgejo.md`, 1.3). A read's is `200`, and a raw file's or a
  log's `200` or `206`. A write's is `201`, as Forgejo answers its
  creations and its edits of an issue (`Edit`, `Close`, `Reopen`), but
  `200` for `Review`, `Merge` and `Update`, and `204` for
  `DeleteBranch`; `SetReviewers` adds with `201` and removes with
  `204`. The reads a write makes succeed as reads do.
