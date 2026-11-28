# GitHub's App

This holds GitHub's App integration, split from the [forge connection draft](archive/github.md).

## 1. temper as a GitHub App

- **An App, one per deployment, made by the manifest flow** (section 2),
  or by the person (2.5), owned by the account that connects it. A
  shared public "temper" App cannot be: its private key would have to
  live with every deployment.
- **Never the person.** GitHub refuses an author's approval of their own
  pull request, and temper tells its writes and their echoes from
  others' by its user's id (`../domain/forge/keeping-up.md`, 2): acting as the
  person would turn the person's own comments into echoes. The person's
  token is never asked for.
- **Public, installed only through temper's link.** A private App can
  be installed only on the account that owns it; to work in an org the
  person does not own the App under, or in their own account and an
  org, the App must be public. Each installation is accepted only when
  it comes back through temper's link with the `state` temper gave it
  (2.3); any other is uninstalled (`DELETE /app/installations/{id}`).
- **One forge, many accounts.** The App has an installation per
  account, user or org, each with its own token. The forge is
  github.com; installations are its runtime state, found and followed
  through the delivery log (section 3), never configured. skein's
  component holds one credential per forge today (skein's
  `forge/connection.md`, section 5): for GitHub that credential is the
  App, and the component mints, renews and chooses an installation's
  token by the repository a call names. A forge per account would have
  broken forks across accounts, and made adding an account a
  configuration change.
- **Its identity** is the App's bot user, `<slug>[bot]`, with an id of
  its own: the forge writer (`Writer { forge, author }`) and the user
  echoes are told by. At connect, before any installation, its id is
  read with no credential, `GET /users/<slug>%5Bbot%5D`; at each start,
  GraphQL's `viewer { login databaseId }` on an installation's token
  checks that the credential answers as it, since REST's `GET /user`
  refuses an installation's token (section 7, P2).
- **Its permissions,** asked in the manifest and nothing more: contents,
  pull requests and issues to write; checks, statuses, actions,
  administration and metadata to read (administration reads the rules
  adoption shows); merge queues to write. Never workflows (`archive/github.md`, section 18),
  never administration to write. The engine holds the App's key, so the
  App's permissions are temper's least power on GitHub.
- **Its events:** push, pull request and its reviews and review
  comments, issue comments, issues, check runs and suites, statuses,
  workflow jobs and runs, branch creation and deletion, merge groups;
  installations and their repositories come to every App.

## 2. Connecting GitHub: the manifest flow

### 2.1 The two clicks

1. **Connect.** The person chooses "Connect GitHub" in the client; the
   engine answers an address on the clients' listener, with a one-time
   `state` tied to that request and the page's address to come back
   to, and the browser's tab goes there, as for a sign-in (the terminal
   prints it and opens no browser; `../client/line.md`, 5).
2. **The manifest.** That address serves a page whose form posts the
   manifest to `https://github.com/settings/apps/new?state=<state>` as
   it loads: the App is always the person's own, and public, so it
   installs on any account they may install it on (with the user,
   2026-10-23). The person clicks "Create GitHub App" on GitHub.
3. **The exchange.** GitHub sends the browser to the manifest's
   `redirect_url` on the clients' listener, with a `code` and the
   `state`. The engine checks the state, exchanges the code with `POST
   /app-manifests/{code}/conversions` within the hour GitHub allows, and
   keeps the App's id, slug, key and webhook secret in the store before
   it answers the browser. The client and secret of the App's OAuth are
   not kept (`archive/github.md`, section 18).
4. **The install.** The engine sends the browser to
   `https://github.com/apps/<slug>/installations/new?state=<state>`.
   The person picks an account and its repositories, all or selected,
   and clicks Install. GitHub sends the browser to the manifest's
   `setup_url`, on the clients' listener, with the installation's id and
   the state; temper accepts the installation and sends the browser
   back to the page that started the connect, which lists its
   repositories to start in. A member of an organization who isn't its
   owner gets a request, not an installation: GitHub comes back with
   `setup_action=request` and none, and temper's page says why it
   can't go on, and to install it on their own account or an
   organization they own.

### 2.2 The manifest

- `name` `temper-<random>`, under GitHub's bound on names; `url` the
  deployment's (or temper's project page); `public` true (section 1).
- `redirect_url` and `setup_url` on the clients' listener, with
  `setup_on_update` true, so a person changing the repositories on
  GitHub comes back to temper. GitHub takes `http://127.0.0.1:<port>`
  for both (section 7).
- `hook_attributes`: `url` `https://<slug>.invalid/` (RFC 2606's
  `.invalid` never resolves), `active` true, so deliveries are recorded
  (section 3); or the deployment's public address, when it has one.
- `default_permissions` and `default_events`, section 1.
- **Built by skein's setup plan** for GitHub (skein's `forge/setup/github.md`,
  section 1), a pure function of the given values, and served by
  temper's server on the clients' listener: the page, the redirect and
  the setup route are temper's, its owner's.

### 2.3 Another account later

- **"Install on another account"** in the client gives a fresh state
  and the install link, so the person adds their org, or another, at
  any time; the same `setup_url` brings the tab back.
- **Repositories granted later** on GitHub's installation page reach
  temper as an `installation_repositories` delivery (section 3), and
  appear in the picker.

### 2.4 Reconcile, on GitHub

At each start and after a connect, temper checks what it can, and
repairs nothing on GitHub but its own hook's address:

- the App's permissions and events against what this temper needs; a
  newer temper asking more names the setting, and GitHub asks each
  installation's owner to accept it;
- the hook's address against the deployment's (`PATCH /app/hook/config`,
  with the App's JWT);
- the installations against those accepted (section 1);
- the App's key still taken: a key revoked on GitHub stops the forge,
  and the client offers to replace it.

### 2.5 Managed and connected, on GitHub

skein's modes (skein's `forge/setup/README.md`, section 3) on GitHub, where no
token makes an App:

- **Managed is the manifest flow:** temper drafts its App, and the
  person only clicks (2.1). It is the way the client offers first.
- **Connected is an App the person made** on GitHub's settings, by hand
  or by their own tooling: its id and private key pasted in the client,
  a request carrying values (`archive/github.md`, section 3). Reconcile checks it as 2.4
  does, and names each permission or event missing; it sets the hook's
  address itself, the one repair the App's key allows. The person
  installs it through temper's link (2.3), as for managed.
- **A fine-grained token** as connected is later (`archive/github.md`, section 18): the
  person's token would make temper the person (`github-app.md`, section 1), so it would
  have to be a second account's, a machine user.
- **Owned:** none; github.com is GitHub's.

## 3. Hearing changes without a public address

- **GitHub pushes, never streams.** Its API has no long poll, stream or
  subscription. The Events API is polled, with a latency GitHub
  documents as 30 seconds to 6 hours. `gh webhook forward` streams, but
  for testing alone ("not supported for use in production"), one
  forwarder per repository or org, and not for an App's hook. A webhook
  needs an address GitHub can reach, with TLS.
- **The delivery log is the hint feed.** The App's hook points nowhere
  (2.2), so every delivery fails, and GitHub records each one: `GET
  /app/hook/deliveries`, with the App's JWT, lists them newest first,
  each with its `guid`, `event`, `action`, `installation_id`,
  `repository_id`, `status` and `delivered_at`; `GET
  /app/hook/deliveries/{id}` gives one whole, with its payload. One
  conditional read every `log_poll` covers every repository of every
  installation, and answers `304` when nothing was delivered. A change
  is in the log about two seconds after it is made (section 7, P7).
- **A hint from each delivery,** as the listener makes one (skein's
  `forge/README.md`, section 6): the event, the installation and the repository
  from the list; where the hint names more (a branch, a pull request, a
  commit), the delivery's payload, read whole within its bound and
  decoded by the same decoders as a pushed delivery's. Events whose
  hint the list alone gives, or which no wait needs, are not read.
- **An installation's events are always read whole:** `installation`
  and `installation_repositories` come with no installation or
  repository in the list (section 7, P7), and say which account was
  installed, and which repositories were granted or withdrawn. They
  change what the component may reach, and what the picker lists.
- **A delivery's id** is a 19-digit number, decoded exactly as a `u64`
  (a JSON decoder going through a float would lose it).
- **The component's puller** sits beside its listener (skein's
  `forge/connection-webhooks.md`, section 1): it keeps the newest `guid` seen,
  reads back to it at each poll (paging by cursor when more arrived),
  and at start reads back within `log_catch_up`, so deliveries made
  while temper was down still hint. Its reads run on the App's JWT,
  apart from the installations' rates.
- **What temper waits on, read conditionally.** A fresh read with the
  validator of the last answer is answered `304` when nothing changed,
  which GitHub does not count against the rate (section 7, P6), so the
  client's keep can read a waited-on pull request's CI every few
  seconds at no cost. This is skein's validators addition (skein's
  `forge/README.md`, section 8).
- **Polling is unchanged:** the keep's paces and slow pass are the
  backstop, as on Forgejo (skein's `forge/client.md`, section 4).
- **A public address,** later and optional: the person's proxy, or
  skein's TLS server side, named in configuration; reconcile sets the
  hook to it (2.4); pushed deliveries, checked by `X-Hub-Signature-256`,
  and the log's are told apart by `guid`, and each hints once.
- **QA does not need it.** QA's boxes run an owned Forgejo, whose
  webhooks are on loopback (`../protocol/setup/webhooks.md`, section 1).

## 4. Credentials that expire

- **The App's key** signs a JWT (RS256, through `ring`, as skein-tls has
  it), valid for minutes, for the App's own routes: its installations,
  its hook's configuration and delivery log, and minting tokens.
- **An installation's token** is minted from it (`POST
  /app/installations/{id}/access_tokens`) and expires within the hour
  (section 7, P1). skein's renewal (skein's `forge/connection.md`, 5.1)
  is built for it: the dialect's plan mints the next token `renew_before`
  its expiry; a call needing it meanwhile waits, within its deadline. A
  token may be minted narrower than the App, in permissions and
  repositories, and a write beyond it is refused (section 7, P1): the
  component mints git's with contents alone. Neither names repositories:
  a token minted for the whole installation reaches a repository granted
  after it at once, where one minted for named repositories does not
  (P12).
- **Each installation's token has its own rate,** 5,000 requests an hour
  on REST's core (P1); the App's JWT calls carry no rate headers, and
  are few (section 3).
- **Git's identity on GitHub** is one identity for the forge, whose value
  holds each accepted installation's current git token, `x-access-token`
  as the login, and the bot's author; the git component takes the token
  of the repository's account. Each renewal, and each installation
  accepted or removed, is a new generation (`archive/github.md`, section 4). The author is
  the bot's noreply address, `<id>+<slug>[bot]@users.noreply.github.com`,
  so GitHub shows its commits as the App's.
- **The open question** on short-lived git credentials
  (`../protocol/credentials-git-identities.md`, section 1) is answered by this.

## 5. The dialect

- **In skein,** GitHub's documents and call plans beside Forgejo's
  (`skein/docs/design/forge/dialects.md`, section 1, and `skein/docs/design/forge/README.md`, section 8): REST, pinned to API version
  `2026-03-10` (the `X-GitHub-Api-Version` header; requests without it
  get 2022-11-28, and a version stays supported 24 months after the next
  one), with GraphQL for what REST lacks: entering the merge queue
  (6.2), the commit a queue landed, and the bot's identity (`github-app.md`, section 1).
  temper does not use auto-merge: it decides each landing itself.
- **The pin** is the API version and the conformance's date. GitHub is a
  service, not a release a deployment runs: the conformance runs on a
  schedule, and a fact that changes is a finding (`archive/github.md`, section 17).
- **The vocabulary's additions** (skein's `forge/README.md`, section 8):
  validators, whose `304` the budget counts at no cost; the rate's
  window from `x-ratelimit-*` on every answer, which the client's budget
  takes in place of its configured one; the secondary limits' refusals,
  a `403` or `429` with `Retry-After`, as a pause. Batched GraphQL reads
  are later.
- **Each call's GitHub shape,** the facts in section 7 and the old
  crate's quirks (`~/src/rust/old/temper/crates/temper-forge-github`,
  a catalogue only, never code to take): pull requests listed among
  issues and told apart by `pull_request`; `merged` absent from listings
  (`merged_at`); a fork's head repository null; reviews `APPROVED`,
  `CHANGES_REQUESTED`, `COMMENTED`, `DISMISSED`, `PENDING`; `mergeable`
  computed after a read, null meanwhile; pages by `Link`; pull requests
  and issues numbered together, and a new issue missing from a listing
  for a moment (section 7, P13).
- **Refusals the dialect tells apart,** by status and GitHub's message,
  since several share a status: a stale head (`409`), not mergeable yet
  or a rule unmet (`405`, P5, P8, P9), an expected head that moved
  (`422` on update, `UNPROCESSABLE` in GraphQL), beyond the token's
  reach (`403` "Resource not accessible by integration"), a push refused
  by a rule (`GH013`) or for touching workflows (P3, P10).
- **CI** is check runs and commit statuses, read apart: the combined
  status stays pending with no status at all while Actions' check runs
  end, and other Apps' suites (a Codecov, a Travis) may sit queued
  forever (section 7, P4). So CI on a head is the checks the adoption
  names: the branch's required status checks when its rules have them,
  and otherwise the repository's Actions workflows' check runs. Actions'
  jobs, with their steps, are read through the Actions API.
- **A job's log** is a redirect to Azure's blob storage, signed, needing
  no credential, valid about ten minutes, and blind to `Range`: the
  component follows it to that host alone, on TLS, sending nothing of
  the forge's, and keeps the log's tail as it streams, within the call's
  bound (section 7, P4).
- **Credentials' kinds:** the App (key, id), whose renewed kind is the
  installation's token (8). A fine-grained token is a static kind,
  later (`archive/github.md`, section 18).
- **Deliveries:** `X-GitHub-Event`, `X-GitHub-Delivery`,
  `X-Hub-Signature-256`; payloads decoded keeping the hint's fields; the
  delivery log's two documents.
- **The setup's documents:** the manifest, its conversion, the
  installations and their repositories, the hook's configuration.

## 6. Adoption and landing on GitHub

### 6.1 Merging

- **At the head temper saw:** `PUT /repos/{o}/{r}/pulls/{n}/merge` with
  `sha`; another head is refused, `409`, and the change goes on as on
  Forgejo. Just after a head moves, GitHub may answer `405` "Pull
  Request is not mergeable" in its place (section 7, P5): within
  `settle` of a move, a `405` is not known yet, and read again, as
  Forgejo's not-mergeable-while-checking is (`../domain/forge/keeping-up.md`, 2).
- **CI does not hold a merge** where no rule asks it to (P5). temper's
  landing rules are its own enforcement (`../domain/forge/adoption.md`
  ), so nothing changes.
- **The merge method** is the first the repository allows of merge,
  squash and rebase, unless the adoption names one (`archive/github.md`, section 19).
- **Updating from the base:** `PUT .../pulls/{n}/update-branch` with
  `expected_head_sha`; a head that moved is refused, `422`; the right
  one is accepted, `202`, and done later: the head moves once GitHub has
  merged, and temper hears it as any move (P5).
- **Mergeability settles after a read:** `mergeable` null is read again
  within `settle` (P4).

### 6.2 The merge queue

- **Where a branch requires GitHub's merge queue,** a direct merge is
  refused, `405` "Changes must be made through the merge queue" (section
  7, P9). temper enters the queue with GraphQL's `enqueuePullRequest`,
  giving `expectedHeadOid`, the head its decision saw: a head that moved
  is refused, and the change goes on as for a stale head. Its decision
  holds at the queue's door.
- **GitHub's queue is then that branch's landing queue**
  (`../domain/forge/queues.md`): temper's own queue for the branch
  decides only what it enters and when. GitHub makes a merge group, a
  commit of its own on the base, runs CI on it (`merge_group`), and
  lands it.
- **The landed commit is the group's,** not the head temper saw, and
  REST reads the pull request merged with no `merge_commit_sha` (P9):
  temper reads it from GraphQL's `mergeCommit`, or from the base's tip
  once the pull request reads closed and merged.
- **What temper hears,** through the delivery log: the pull request
  `enqueued`, then `dequeued` with its reason (`MERGE`, `CI_FAILURE`,
  and GitHub's others); `merge_group` `checks_requested` and `destroyed`
  (`merged`, `dequeued`). The change's panel says "in GitHub's merge
  queue" and its position, read from the entry
  (`../client/forge/README.md`, 2.1).
- **Ejected,** a pull request dequeued for any reason but `MERGE` is a
  repair, as a failed CI is (`../domain/forge/change/repairs.md`). GitHub ejected
  a failed group about half a minute after its CI failed (P9).
- **Merge queues exist** on org-owned public repositories, and private
  ones on GitHub Enterprise Cloud; elsewhere the branch merges directly.

### 6.3 What adoption reads, and says

- **Reach and permission:** the installation's, and the repository's
  role in it.
- **The rules on the landing branch,** as GitHub applies them: `GET
  /repos/{o}/{r}/rules/branches/{branch}`, each rule with the ruleset it
  comes from, and each ruleset's `current_user_can_bypass`, the App's
  own; never the bypass list, which the App cannot read (section 7,
  P11). A branch under rulesets reads "Branch not protected" at
  `/branches/{branch}/protection` (`404`, P11), so adoption reads both,
  rulesets and classic protection, and takes neither's absence for the
  other's. Each rule is said in words with what it means for landing
  (`archive/github.md`, 2.3):
  - a merge queue: temper enters it (6.2);
  - required approvals: a person approves, or the App bypasses;
  - required status checks: CI as the landing rule's;
  - code owners: a person reviews;
  - signed commits: temper's git pushes are refused (P10), so its
    landings wait (`archive/github.md`, section 18);
  - linear history: a merge method that keeps it;
  - temper's prefix reserved to the App: welcome.
- **Workflows:** the App never changes `.github/workflows/`, and GitHub
  refuses a push that does (P3). Adoption says so where the repository
  has workflows; a change whose push touches them is refused, and its
  panel says why (`archive/github.md`, section 18).
- **Archived** repositories adopt as context only. **Actions off:** the
  adoption names the checks that stand for CI.
- **Actions on, no workflow:** when adoption finds no workflow and no
  required checks, it says "No CI found: Actions is on, but this repository
  has no workflow and no required checks." It asks: **Land without CI**,
  or **Name the checks that stand for CI**. "Land without CI" adopts with
  CI off; naming checks makes those checks the gates in place of CI. This
  choice does not waive required checks or any other landing rule.
- **Outside contributions** stay participating objects
  (`../domain/forge/README.md`, section 7). A first-time contributor's CI
  waits for a maintainer's approval on GitHub, which temper shows as a
  wait on a person. CI on GitHub's runners is GitHub's isolation:
  ../protocol/setup/README.md's concern for CI's isolation is a self-hosted runner's.
- **Context** reads any public repository on github.com, with no
  installation.

## 7. What the probes found

Run on 2026-10-20 with a throwaway public App, `temper-probe-d0fc06`,
installed on the probe account (repository `temper-probe`) and the org
`fabrig-ai`, then named
`fabrig-dev` (repository `fabrig-ai/temper-probe`; the API answers the
old names with a 301), with API version 2026-03-10; each repository
had one Actions workflow, a job `ci` that sleeps 45 seconds and fails
when the head's message says `FAIL`. Rulesets, pull requests, issues,
branches and tokens made by the probes were removed after. The scripts
and their evidence are not kept; the live conformance records the
facts for good (`archive/github.md`, section 17). The App and the repositories are kept as
its fixtures, with a spare repository on the probe account,
`temper-probe-2`, not granted to the App. The App's key, its hook secret, the person's
tokens (`archive/github.md`, section 17) and a README are in the person's private
fixtures directory, never in a repository. Each fact
is cited by its probe's number across this document.

- **Setup.** The manifest flow takes `http://127.0.0.1:<port>` as its
  `redirect_url` and `setup_url`; the conversion answers `id`, `slug`,
  `pem`, `webhook_secret`, `client_id`, `client_secret` and the App's
  `permissions` and `events`; every permission and event of section 1
  was taken, `merge_queues` and `merge_group` among them. A hook at
  `https://hooks.temper-probe.invalid/github` is taken.
- **P1, tokens.** An installation's token expires an hour after it is
  minted. One minted narrower, in permissions or repositories, is
  refused beyond them: `403` "Resource not accessible by integration".
  REST on an installation's token answers `x-ratelimit-limit` 5000 on
  the `core` resource; the App's JWT calls answer no `x-ratelimit-*`.
- **P2, identity.** `GET /user` on an installation's token: `403`.
  GraphQL's `viewer { login databaseId }` on it names the bot, its id
  the bot user's (GraphQL calls it a `User`, REST a `Bot`); `GET
  /users/<slug>%5Bbot%5D` gives the same id. `/app` and its routes need
  the JWT: `401` on an installation's token.
- **P3, git.** Clone and push over HTTPS with `http.extraHeader` basic
  authentication, `x-access-token` and the token, work. The App's git
  commits are unsigned (`verified` false, `unsigned`); its commits made
  through the contents API are GitHub's signed ones (`verified` true). A
  push changing `.github/workflows/ci.yml` is refused: "refusing to
  allow a GitHub App to create or update workflow
  `.github/workflows/ci.yml` without `workflows` permission".
- **P4, pull requests and CI.** A new pull request reads `mergeable`
  null, `mergeable_state` `unknown`, then settles within two seconds.
  Actions make a check run per event (push and pull request, each `ci`),
  with jobs and steps; suites of other Apps installed on the account (a
  Codecov, a Travis) sit `queued` with no run. The combined commit
  status stays `pending` with no status while Actions' runs end. A job's
  log answers `302` to `productionresultssa1.blob.core.windows.net`,
  fetched there with no credential, `Range` ignored (`200`, the whole
  log), its signed address valid 600 seconds (GitHub's documents say a
  minute).
- **P5, merging and updating.** A merge with a stale `sha`: `409` "Head
  branch was modified. Review and try the merge again."; made at once
  after the head moved: `405` "Pull Request is not mergeable"; with the
  head: `200`. With no rule, a merge succeeds while the head's CI runs.
  `update-branch` with a stale `expected_head_sha`: `422` "expected head
  sha didn’t match current head ref."; with the head: `202`, done
  asynchronously.
- **P6, conditional reads.** `If-None-Match` with the last `ETag` is
  answered `304` on a pull request, a commit's check runs and the
  delivery log, and a `304` leaves the rate's counters as they were. A
  pull request's `ETag` also changes when its repositories'
  `updated_at` does.
- **P7, the delivery log.** Each failed delivery is listed, newest first,
  with `status` "failed to connect to host", `status_code` 502, and its
  `guid`, `event`, `action`, `installation_id`, `repository_id`,
  `delivered_at`, `redelivery` and `duration`. A push was listed about
  1.5 seconds after it, a pull request opened about 2.2 seconds after.
  Pages go by `per_page` and an opaque `cursor`, with `Link` `next`. One
  delivery read whole holds the request's headers and payload (a push's
  about 8.6 KB) and the response's. `ping`, `installation` and
  `installation_repositories` are listed with no installation and no
  repository; their payloads name them. No delivery was retried. Ids
  have 19 digits.
- **P8, reviews.** The App approving its own pull request: `422`
  "Review Can not approve your own pull request". Its approval of a
  person's pull request counts toward a ruleset's required approval.
  Under a ruleset asking one approval, the App's merge of its own pull
  request: `405` "Repository rule violations found ... At least 1
  approving review is required by reviewers with write access."; with
  the App on the ruleset's bypass list (`Integration`, its id,
  `always`): `200`.
- **P9, the merge queue.** Under a ruleset requiring the queue, a REST
  merge: `405` "Repository rule violations found ... Changes must be
  made through the merge queue". `enqueuePullRequest` with a stale
  `expectedHeadOid`: `UNPROCESSABLE` "Failed to add PR #4: expected head
  oid does not match the current head oid"; with the head: `QUEUED`,
  then `AWAITING_CHECKS` on a group commit, which is the commit that
  lands, read by GraphQL's `mergeCommit` and on the base; REST reads
  `merged` true and `merge_commit_sha` null. Deliveries: `merge_group`
  `checks_requested` and `destroyed` (`merged` or `dequeued`), the pull
  request `enqueued`, `closed` and `dequeued` (`MERGE`, `CI_FAILURE`). A
  failed group's pull request was dequeued 28 seconds after its CI
  failed, unmerged. `enablePullRequestAutoMerge` on a repository not
  allowing it: "Auto merge is not allowed for this repository".
- **P10, signed commits.** Under a ruleset requiring them, the App's git
  push: refused, `GH013` "Commits must have verified signatures."; its
  commit through the contents API: taken.
- **P11, rules.** With administration to read, the App reads a branch's
  rules (`/rules/branches/{branch}`) with their rulesets, and each
  ruleset, with no bypass list but `current_user_can_bypass`, which
  turned from `never` to `always` once the App was on it. A branch under
  rulesets alone reads `404` "Branch not protected" at
  `/branches/{branch}/protection`.
- **P12, an installation's repositories.** A repository added to or
  removed from a selected installation by its owner is reached, or not,
  at once by a token minted for the whole installation before the
  change; `installation_repositories` `added` and `removed` are
  delivered. A fine-grained token with "GitHub App installation
  repository access" adds and removes one too (checked after the
  probes), but is refused the installation's list of repositories,
  which the App's token reads.
- **P13, issues.** The App opens issues and comments; issues and pull
  requests share numbers; `/issues` lists pull requests, each with a
  `pull_request` member; a new issue was missing from the first listing
  after it.
