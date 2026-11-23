# GitHub's dialect: operations, merges and protection

## 1. Each operation

### 1.1 Reads

R as in `github.md`, section 3; `<size>` the page size; `GQ` a GraphQL request
(1.3). Several requests run in turn on one connection, as forgejo-operations.md 1.
"Every page" in Pull's and Checks' rows stops at 50 pages a listing:
the call fails `TooLarge` when a 50th page still has a next.

| Call | Requests | Answer from |
|---|---|---|
| `Read::Items { since, page, kind }` | `GET R/issues?state=all[&since=<since>]&sort=updated&direction=asc&per_page=<size>&page=<page>` | omit `since` at zero: an epoch `since` lists nothing; otherwise send the UTC bound, inclusive. Each row a `Summary`: `number`; a `pull_request` member makes it a pull, and its `merged_at` makes one closed merged; `state`; `user.id`; `labels[].name`; `updated_at`; `title`; `body` (null as empty) and its key. A row of the other kind than the call's is skipped, and counts toward the page. Rows of one second are put in order of number before the page answers. `more` from `Link`; `now` from `Date` |
| `Read::Item { number, after, since }` | `GET R/issues/<number>`; `GET R/issues/<number>/comments?[since=<since>&]per_page=<size>&page=<n>` from page 1, `since` only when not 0, until the answer is full or a page has no next | the issue as a `Summary`; its comments with ids above `after`, oldest first, as forgejo-operations.md 1's: `id`, `user.id`, `created_at`, `body` and its key, provenance, revision; `more` when one is left |
| `Read::Pull { number, ci }` | `GET R/pulls/<number>`; when `ci`, `GET R/commits/<head.sha>/status?per_page=1` and `GET R/commits/<head.sha>/check-runs?filter=latest&per_page=<size>&page=<n>`, every page; `GQ pull` when the pull is open and the repository's owner an organization, or merged (`merge_commit_sha` is absent at the pin) | `number`; `state`, `merged` making it merged; `head.ref`, `base.ref`, `head.sha`, `base.sha`; `merged` from `merge_commit_sha`, else `mergeCommit.oid`; `mergeable`, null as false; `requested_reviewers[].id`; `queued` from `mergeQueueEntry.position`; `dequeued` from the last `RemovedFromMergeQueueEvent`'s `reason`; `size` from `changed_files` as `files`, `additions` and `deletions`, at `head.sha`, none when one is absent, null or no whole number (vocabulary.md, 1.6); `Ci` from both (1.4), `None` without `ci` |
| `Read::PullFor { head, base }` | `GET R/pulls?state=open&head=<owner>:<head>&base=<base>&per_page=1`; when none, `GET R/pulls?state=all&head=<owner>:<head>&base=<base>&sort=created&direction=asc&per_page=1`; then as `Pull` without `ci` for the number found | the open pull request for the head and base, else the oldest; `Missing` when there is none. `<owner>` is the repository's own: a fork's head is not followed (temper's `draft/archive/github.md`, 18) |
| `Read::Reviews { number, page, skip }` | `GET R/pulls/<number>/reviews?per_page=<size>&page=<page>` | as forgejo-operations.md 1; `state` mapped `APPROVED` to `Approve`, `CHANGES_REQUESTED` to `RequestChanges`, `COMMENTED` and `DISMISSED` to `Comment`; `PENDING` and a null `submitted_at` skipped, counting toward the page; `commit_id`; `official` when `author_association` is `OWNER`, `MEMBER` or `COLLABORATOR` (`github.md`, section 15) |
| `Read::Statuses { commit, page }` | `GET R/commits/<commit>/status?per_page=<size>&page=<page>` | `Ci` from `state` and `total_count`, 0 being `None`; each `statuses[]` row: `updated_at`, `context`, `state`, `description`, `target_url`; no job; `more` from `Link` |
| `Read::Remarks { number, review, page, skip }` | `GET R/pulls/<number>/reviews/<review>/comments?per_page=<size>&page=<page>` | `id`, `user.id`, `path`, `line`, else `original_line`, else 0, `body`; `more` from `Link` |
| `Read::Branch { branch }` | `GET R/branches/<branch>` | `commit.sha` |
| `Read::Branches { prefix }` | `GET R/branches?per_page=<size>&page=<n>`, from 1 until no next | only names starting with the nonempty prefix, within `string_bytes`; a longer one skipped; only matches count against the answer bound, with no row bound (vocabulary.md, 1.7), past it `TooLarge`; an empty prefix is `Empty` before any GET |
| `Read::PullFiles { number, head, page, skip, sides }` | `GET R/pulls/<number>`; when `sides`, `GET R/compare/<base.sha>...<head.sha>?per_page=1`, for `merge_base_commit.sha`; `GET R/pulls/<number>/files?per_page=<size>&page=<page>`; when `sides`, for each file kept, the raw read (below) at the merge base, at `previous_filename` for `renamed`, but for `added`, and at the head but for `removed` | as forgejo-operations.md 1; a `head.sha` other than `head` is `Stale` before any file is read; `more` from `Link` |
| `Read::Compare { before, after }` | `GET R/compare/<before>...<after>?per_page=1` | `files[].filename`, kept while they fit `rows` and the answer's bound, and each counted in `changed`; `ahead` from `ahead_by`, `behind` from `behind_by`; no raw read; `TooLarge` when `files` holds 300, GitHub's cap (`compare-files-capped-300`) |
| `Read::Range { before, after }` | `GET R/compare/<before>...<after>?per_page=1` | `total_commits` as `commits`; the one commit and the files it lists skipped as they stream |
| `Read::Checks { commit }` | `GET R/commits/<commit>/status?per_page=<size>&page=<n>`, every page; `GET R/commits/<commit>/check-runs?filter=latest&per_page=<size>&page=<n>`, every page; for each failed Actions check with a resolvable link, `GET R/actions/jobs/<job>` | each status as for `Statuses`; each check run as a `Status` (1.4), its `job` the commit, the run from `run_id`, the job its `id` and the attempt its `run_attempt`, after validating the linked run and job and the requested head (1.4), none otherwise |
| `Read::Job { attempt, max_bytes, part }` | `GET R/actions/jobs/<job>`; `GET R/actions/jobs/<job>/logs`, answered `302`; the redirect followed (`github.md`, section 10) | the failed step: the first of `steps[]` whose `conclusion` is `failure`, its `number` and `name`; for `End`, the log's last `max_bytes` bytes (10), for `Start`, its first (10); `MissingJob` on a `404` or `410`, or when `run_id`, `run_attempt` or `head_sha` is not the attempt's |
| `Read::File { head, path, max_bytes }` | the raw read of `path` at `head` | the first `max_bytes` bytes, `truncated` when more came; `Missing` on a `404`, and on a JSON answer (a directory's listing) |
| `Read::Protection { branch }` | `GET R/rules/branches/<branch>?per_page=100&page=<n>`, every page; for each ruleset named, `GET R/rulesets/<ruleset_id>`; `GET R/branches/<branch>/protection` | section 2.3 |
| `Read::Settings` | `GET /repositories/<id>` | `default_branch`; `allow_merge_commit`, `allow_squash_merge`, `allow_rebase_merge`; the row kept afresh |
| `Read::Collaborators { page }` | `GET R/collaborators?affiliation=all&per_page=<size>&page=<page>` | each `id` and `role_name` as below; `more` from `Link` |
| `Read::Permission { user }` | for the bot (the identity read's `databaseId`, `github-app.md`, 1.4), none; otherwise `GET R/collaborators/<login>/permission` | the bot's from its installation (`github-app.md`, 1.2): `Write` when the full mint's `permissions` hold `contents: write` and `pull_requests: write`, `Read` when they hold `contents` at `read` or above, `None` otherwise; another user's from `role_name`: `admin` and `maintain` `Admin`, `write` `Write`, `triage` and `read` `Read`, `none` and the empty string `None` |
| `Read::Locate { owner, name }` | `GET /repos/<owner>/<name>` | `id`; the row kept |

The dropped rows are names the client can't hold, such as queue branches
past the bound; the client makes no name past `string_bytes`, so it never
looks for a branch it dropped (`Read::Branches { prefix }`, 2.3's row).

- **Compare reads one page.** Its first page carries the files for the
  whole comparison, capped at 300, and its counts; later pages carry
  commits alone, which Compare keeps none of, so it asks for one commit a
  page and reads no later page. Containment is `behind_by == 0`, for a
  diverged comparison too; a nonzero `ahead_by` alone does not show it.
  PullFiles with `sides` asks for only the merge base and keeps `per_page=1`.
- **The raw read** is `GET R/contents/<path>?ref=<commit>` with `Accept:
  application/vnd.github.raw+json`, the path's slashes kept: a `200`
  that is not JSON is the file's bytes, a `404` an absent side.
- **No `Range` header.** skein sends none: a file and a log are read through,
  and cut as they stream (`github.md`, section 6). The signed job-log blob honours
  Range; skein requests the whole body.
- **The keep-up listing** is as Forgejo's (forgejo-operations.md, 1), in GitHub's
  words: `sort=updated&direction=asc`. A new issue can be missing from
  the first listing after it (`new-issue-may-miss-listing`): the next
  pass, from the same `since`, finds it, since `since` is inclusive
  (`issues-since-inclusive`).
- **An item's comments are paged by GitHub,** oldest first: the call
  reads pages from the first and keeps those above `after`; each page
  counts in its cost (`github.md`, section 15).

### 1.2 Writes

| Call | Requests | Answer from |
|---|---|---|
| `Write::CreateIssue { key, title, body }` | `POST R/issues` `{title, body}`, the body behind the key's marker | `Created` from `number` (`201`) |
| `Write::OpenPull { title, body, head, base }` | `POST R/pulls` `{title, body, head, base}` | `Created` from `number` (`201`); a `422` "No commits between ..." `NothingToMerge`, and "A pull request already exists for ..." `Exists` (`github.md`, section 7) |
| `Write::Post { number, key, body }` | `POST R/issues/<number>/comments` `{body}`, behind the marker | `Commented` from `id` (`201`) |
| `Write::Review { number, key, verdict, body }` | `POST R/pulls/<number>/reviews` `{event, body}`, `event` `APPROVE`, `REQUEST_CHANGES` or `COMMENT`, behind the marker | `Reviewed` from `id` (`200`) |
| `Write::Edit { number, title, body }` | `PATCH R/issues/<number>` `{title, body}`, each only when given | `Done` (`200`) |
| `Write::SetReviewers { number, reviewers }` | `GET R/pulls/<number>`; `POST R/pulls/<number>/requested_reviewers` `{reviewers: [logins]}` for those missing (`201`); `DELETE` the same with those extra (`200`) | `Done` |
| `Write::Close { number }`, `Write::Reopen { number }` | `PATCH R/issues/<number>` `{state: closed\|open}` | `Done` (`200`) |
| `Write::Merge { number, head }` | section 2.1 | `Merged`, or `Queued` |
| `Write::Update { number }` | `GET R/pulls/<number>`; `PUT R/pulls/<number>/update-branch` `{expected_head_sha: <head.sha>}` | `Done` (`202`); section 2.2 |
| `Write::Status { commit, context, check }` | `POST R/statuses/<commit>` `{state: pending\|success\|failure, context}` | `Done` (`201`); `Forbidden` where the App has no `statuses` write (below) |
| `Write::CreateBranch { branch, commit }` | `POST R/git/refs` `{ref: refs/heads/<branch>, sha}` | `Branch(Created)` (`201`); a `422` "Reference already exists" `Branch(Exists)` |
| `Write::DeleteBranch { branch }` | `DELETE R/git/refs/heads/<branch>` | `Done` (`204`); a `422` "Reference does not exist" `Missing` |

- **Opening asks GitHub,** which refuses a head with nothing to merge
  and a second open pull request itself, each with its own message: one
  request, where Forgejo's compares first (forgejo-operations.md, 1).
- **A status needs `statuses` write.** The fixture App's installation token
  receives `Forbidden`, GitHub's `403` "Resource not accessible by integration";
  the fixture person tokens receive `403` "Resource not accessible by personal
  access token". The queue cases use the fixture workflow's check run instead
  (`github.md`, section 16). This is the call GitHub answers otherwise than Forgejo
  (vocabulary.md, 1.7).
- **Keys and markers** are as Forgejo's (README.md, section 5).

### 1.3 The GraphQL documents

Constants of the module, each sent as `POST /graphql` with its
variables, on the installation's token. A GraphQL answer is `200` with
`data`, or with `errors[]`, each with a `type` and a `message`; an
answer with `errors` is a refusal (`github.md`, section 7).

- **`viewer`:** `query { viewer { login databaseId } }`, the identity
  (`github-app.md`, 1.4).
- **`pull`:** `query($owner: String!, $name: String!, $number: Int!) {
  repository(owner: $owner, name: $name) { pullRequest(number: $number)
  { id mergeCommit { oid } mergeQueueEntry { position } timelineItems(last:
  1, itemTypes: [REMOVED_FROM_MERGE_QUEUE_EVENT]) { nodes { ... on
  RemovedFromMergeQueueEvent { reason } } } } } }`, a pull request's node
  id, its landed commit, its queue position and why the queue last
  ejected it.
- **`enqueue`:** `mutation($id: ID!, $head: GitObjectID!) {
  enqueuePullRequest(input: {pullRequestId: $id, expectedHeadOid:
  $head}) { mergeQueueEntry { position } } }`.

Batched reads are later (temper's `draft/archive/github.md`, 18).

### 1.4 CI

- **A check run as a `Status`:** `completed_at`, else `started_at`, as
  its time; `name` as its context; `output.title` as its description;
  `html_url` as its link. Its `Check`: a `conclusion` of `success`,
  `neutral` or `skipped` is passed; `failure`, `timed_out`,
  `cancelled`, `action_required`, `startup_failure` or `stale` failed;
  a run not `completed` (`queued`, `in_progress`, `waiting`,
  `requested`, `pending`) pending.
- **A status's state:** `success` passed, `failure` and `error` failed,
  `pending` pending.
- **`Ci` on a pull request** is both read apart: failed when any status
  or check run failed, else pending when any is pending, else passed
  when any is there, else `None`. The combined status's `pending` with
  `total_count` 0 is no status (`combined-pending-with-no-status`), and
  another App's suite with no run lists no check run.
- **Which checks are CI** is the adoption's, the client's: the branch's
  required checks when its rules name them, else the Actions workflows'
  (temper's `draft/github-app.md`, 5). The dialect reads them all.
- **An Actions job** has its own id, separate from the check-run id.
  For a failed check, `html_url` with the repository's owner and name and
  `/actions/runs/<run>/job/<job>` supplies candidate run and job ids;
  an optional query is ignored. `app.slug` may be `github-actions` or
  absent; an explicit other App and classic statuses have no job pin.
  The link is only a candidate: skein requests the job through the
  configured API, never the link's origin, and pins it only when `id`,
  `run_id` and `head_sha` match that candidate and the requested head,
  and `run_attempt` is positive. An unresolvable link leaves no pin;
  conflicting job metadata fails the call. `Read::Job` revalidates the
  full head/run/job/attempt before reading the log.

### 1.5 Actions, trees, refs and labels

The GitHub routes of vocabulary.md 1.2's Actions, tree, ref and label calls.

| Call | Requests | Answer from |
|---|---|---|
| `Write::Dispatch { workflow, branch, inputs }` | `POST R/actions/workflows/<workflow>/dispatches` `{ref, inputs}` | `Done`; the owner finds the run by `Runs`, as on Forgejo (GitHub's `return_run_details` is not asked: one way to find a run on both forges) |
| `Read::Runs { workflow, commit, page }` | `GET R/actions/workflows/<workflow>/runs?head_sha=<commit>&page=<page>&per_page=<size>` | each run as a `Run`, its event from `event`, its user from `triggering_actor.id` |
| `Read::Run { run }` | `GET R/actions/runs/<run>`; `GET R/actions/runs/<run>/jobs` | as README.md's row, the failed step from each job's `steps` |
| `Write::Rerun { attempt }` | `POST R/actions/jobs/<job>/rerun` | `Done`; `Refused` for a run not completed (`403` naming it) |
| `Read::Artifact { run, name, max_bytes }` | `GET R/actions/runs/<run>/artifacts?name=<name>`; its `archive_download_url` | as README.md's row |
| `Read::Tree { object, page }` | `GET R/git/trees/<object>` | entries from `tree`; GitHub answers a tree whole up to its own cut, `truncated` past it, which is `TooLarge` |
| `Read::Commits { before, after, page }` | `GET R/compare/<before>...<after>?page=<page>&per_page=<size>` | its `commits`, oldest first |
| `Write::UpdateRef { branch, to }` | `PATCH R/git/refs/heads/<branch>` `{sha: <to>, force: false}` | `Done`; `Stale` for a `422` "Update is not a fast forward"; `Protected` for a rule the credential may not bypass |
| `Write::Labels`, `Write::CreateLabel`, `CreateIssue`'s labels | `PUT R/issues/<number>/labels` `{labels}`; `POST R/labels`; `POST R/issues` `{labels}` | as README.md's rows |

- **A commit range keeps commits alone.** Its decoder requires
  `total_commits` and `commits`, including each sha, parent sha and
  commit message, and skips changed files and their contents. The page
  preserves GitHub's oldest-first order. `more` follows the next link;
  a terminal page short of the remaining total fails `InvalidAnswer`,
  so a missing commit list cannot prove an empty range.
- **A ref update is a push** to GitHub's rules: it passes a rule that
  requires a pull request only for a credential on that rule's
  ruleset's bypass list with `Always`. `Protection` carries both apart
  (2.3): `pulls_required`, whether any rule on the branch requires a
  pull request, and `pulls_bypass`, the App's bypass of the rulesets
  that do (`Never`, `PullsOnly` or `Always`, the least of them), beside
  the aggregate `bypass`.
- **Stories against the fake GitHub** (../testing/fakes/github.md, sections 1 and
  2): a workflow dispatched and its run found; a
  status posted on a commit; a ref update fast-forwarded under `Always`;
  one refused as not a fast-forward; one refused under a rule without
  `Always`; a lost ref update's answer found made by its branch.
- **To check before relying on them** (`github.md`, section 13): a dispatched run
  found by `Runs` at its commit; a rerun's refusal; a ref update's refusal when not a
  fast-forward and under a rule; whether a ref update bringing a change
  under `.github/workflows/` is refused without `workflows`; the
  per-repository "Auto-close issues with merged linked pull requests"
  read by `Read::Settings`.

## 2. Merging, updating, and the rules

### 2.1 The merge

1. `PUT R/pulls/<number>/merge` `{sha: <head>, merge_method: <the
   row's method>}`. A `200` answers `Merged` from `sha`.
2. A `405` whose message names the merge queue ("Changes must be made
   through the merge queue", `github.md`, section 7): `GQ pull`, for the node id;
   then `GQ enqueue` with `expectedHeadOid` the call's `head`. Its entry
   answers `Queued`.
3. Every other refusal is `github.md`, section 7's.

- **A row with no merge method** makes `Merge` answer `Refused` before
  anything is sent, and so does a `Merge` with `how` `Recorded`: GitHub
  records no pull request merged by hand (`vocabulary.md`, 1.2).
- **A lost enqueue is never resent.** When nothing is heard from `GQ
  enqueue` the call is a `Timeout`, and the client reads the pull: an
  observed `queued` entry is adopted as the call's `Queued`.

- **REST first, always.** The plan keeps nothing between calls, so it
  learns of a queue by GitHub's refusal, which costs one request on a
  queued branch and holds when the rules change after adoption: a
  branch that came to require the queue is entered, and one that
  stopped is merged. The latest `Protection` saying `queue` is the
  client's to show; the plan never takes it for GitHub's answer.
- **A repeat merge on a merged pull** answers `200`, `merged: true`, with
  the same merge sha, even when the supplied head sha differs. A REST pull
  read omits `merge_commit_sha` at the pinned 2026-03-10 version, for open
  and merged pulls alike; 2022-11-28 sends it. Every merged `Read::Pull`
  therefore costs the additional `GQ pull` read for `mergeCommit.oid`.
  The conformance compares that plan answer with both PUTs' sha.
- **An open pull's moved head is `Stale`** on both routes: `409` "Head branch was
  modified. Review and try the merge again." at the merge, and
  `UNPROCESSABLE` "... expected head oid does not match the current head
  oid" at the queue.
- **`Queued` is final for the call.** The landing, or the ejection,
  comes later: the log's `pull_request` `dequeued`, with its reason, and
  `merge_group` `destroyed` (`github.md`, section 9) hint the item; the client reads
  the pull, whose `merged` is the group's commit, from `mergeCommit`
  (1.1). An ejected pull is open with no `queued`, and its `dequeued`
  holds the queue's reason, as GitHub words it: a hint carries no
  payload, so the reason is read with the pull.
- **The method** is the row's: the first of merge, squash and rebase
  that the repository allows (`allow_merge_commit`,
  `allow_squash_merge`, `allow_rebase_merge`). A queue merges by its own
  method.
- **The merge's answer carries its commit** (`sha`): no read follows,
  as Forgejo's must (forgejo-operations.md, 1).

### 2.2 The update

`Update` reads the pull's head, then sends it as `expected_head_sha`, so
a push between the two is refused (`422` "expected head sha didn’t
match current head ref.", the apostrophe U+2019 as GitHub writes it).
That refusal is ambiguous: the plan reads the pull once more, without
repeating the update. A head different from the pinned head is `Stale`;
an open pull still at that head is `Conflict`. A closed pull is `Closed`.
The refusal and the unchanged head take precedence over the read's
mergeability hint, including `mergeable: true`. A failed verification
read keeps its read failure; it cannot establish a conflict. GitHub merges
the base into the head later (`202`), and the
move is heard as any push. A head the client saw would need `Update` to
carry it (`github.md`, section 15).

### 2.3 The protection

`Protection` reads both sources GitHub applies, and takes neither's
absence for the other's (`classic-protection-404-under-rulesets`):

- **Rulesets:** `rules/branches/<branch>` lists the rules that apply,
  each with its `ruleset_id`; each ruleset is read once for its
  `current_user_can_bypass` (`always`, `pull_requests_only`, `never`),
  the App's own, since the bypass list is not readable
  (`rules-branches-current-user-can-bypass`). The rules kept:
  `pull_request` (`required_approving_review_count`,
  `dismiss_stale_reviews_on_push`, `require_code_owner_review`),
  `required_status_checks` (each `context`), `merge_queue`,
  `required_signatures`, `required_linear_history`. Others are skipped.
- **Classic protection:** `branches/<branch>/protection`, a `404`
  "Branch not protected" being none: `required_status_checks.contexts`,
  `required_pull_request_reviews` (`required_approving_review_count`,
  `dismiss_stale_reviews`, `require_code_owner_reviews`),
  `required_signatures.enabled`, `required_linear_history.enabled`.
- **One `Protection`,** none when neither source has a rule kept:
  `branch` the call's; `contexts` the union, in order of first seen;
  `approvals` the largest; `dismiss_stale`, `code_owners`, `signatures`,
  `linear` and `queue` each true when any source says so; `bypass` the
  least of every ruleset with a rule kept, classic protection counting
  as `Never`.
- **Its cost:** two requests and one per ruleset named, bounded by
  `inner`; a branch under more rulesets than `inner` is `TooLarge`.
