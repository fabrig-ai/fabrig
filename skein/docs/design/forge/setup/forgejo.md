# Setting up Forgejo

## 1. Forgejo's plan: reconcile

Reconcile is the pure machine for a managed or an owned Forgejo. It
decides from what it reads, and keeps no record of its own: the forge is
its record. The plans beside it (1.5 to 1.8) share its events, needs and
failures.

### 1.1 What it is given

- **Given values, not configuration,** when the owner makes it:
  - the forge's number;
  - the bot's login and email;
  - the tokens' names, one for the API and one for git;
  - the orgs to make and their owner, one optional value: when the forge
    is owned, the person's orgs, at most the limit's, each made when
    absent and owned by the person's login; none when managed;
  - the person's login, one optional value: the account whose id the run
    answers (1.3, step 7); with none, that step reads nothing;
  - the repositories adopted, by id, at most `repositories` (1.5);
  - the name a failure gives the admin;
  - the listener's address, if any. The hook's URL is derived from it:
    `http://<address>/hooks/<number>`, the number being the forge's
    (1.5);
  - the word, which marks the bot;
  - its limits (1.4).

  The first owner's are temper's: the word its own name, the tokens
  `<word>-api` and `<word>-git`, the admin's name `<word>-admin`, and 8
  orgs, 120 s, 100 ms and 5 s; for `Create` (1.9) and `Copy` (1.10), 2
  files of at most 4 KiB each; for `Copy`, 30 minutes.
- **Asked of its host, as needs,** when a step reaches them: the admin's
  token, the API and git tokens, the bot's id as last kept, and the
  webhooks' secret. These are the items (`README.md`, 4.4). Each is named by what it
  is, never by where it lives. The host gives it, or says it has none;
  an empty value is none, for every item. The admin's token is asked for
  like the rest and held to the run's end.
- **The API's address and transport are its host's:** reconcile says a
  request, never an address. The host speaks plaintext to loopback, or
  TLS (`README.md`, section 6).

### 1.2 Events in, needs out

`step(machine, event, out)` takes one event and pushes at most one need.
The host collects needs in an uncapped queue, reserving no room before
a step (../../../foundation/programming-model.md, section 5.2). Every plan, Forgejo's and
GitHub's, has this shape.

| Need | The host's event |
|---|---|
| `Send { request, auth }` | `Reply`: a document, a page of rows, nothing, a status, unsigned, or down |
| `Read(item)` | `Read`: the value kept, or none |
| `Write { item, bytes }` | `Written`, once the value is kept whole |
| `Draw(n)` | `Drawn`: `n` random bytes |
| `Done(outcome)`, `Stop(failure)` | none: the plan has ended |

- **The first event is `Begin`.** An event a step did not ask for ends
  the plan with `Unexpected`; a machine that has ended ignores events.
- **A plan reads its own dialect's reply.** The words every plan shares
  (`Need`, `Event`, `Reply`, `Failure`, `Stage`, `Why`) belong to no dialect.
  The host hands a plan one `Reply`, and a gate of the plan's dialect turns
  it into that dialect's before the plan looks: Forgejo's has its `Document`
  and `Rows`, `Empty`, `Status` and `Down`; GitHub's has its own `Document`
  and `Rows`, the same three, and `Unsigned`. The gate ends the plan on what
  the dialect never receives: `Unsigned` to a Forgejo plan as `KeyForm`,
  another dialect's body as `Unexpected`.
- **A request carries no credential.** Its `auth` travels beside it, and
  the host puts it in the header: `Token(secret)`, a Forgejo token sent
  as `token <value>`; `App { id, key }`, an App's JWT, which the host
  signs afresh for each request (`github.md`, 1.3) and sends as `Bearer <jwt>`;
  `Bearer(secret)`, an installation's token sent as `Bearer <value>`
  (`github.md`, 1.9); or `None`. A key the host cannot sign with answers `Unsigned`,
  and the request is never sent.
- **The host decodes a reply** with `skein-forge`'s decoders (../forgejo.md,
  section 1; ../github.md, sections 8 and 11): 2xx with a document,
  `Document`; with a listing, `Rows`, one page; 2xx with no body,
  `Empty`; any other status, `Status(code)`; no answer or a body that
  does not decode, `Down`.
- **The outcome** is `Done` for most plans, and carries what the owner
  serves or shows: the manifest's state and page (`github.md`, 1.1), the App made and
  who owns it (`github.md`, 1.2), the install link (`github.md`, 1.4), what an App lacks and who
  owns it (`github.md`, 1.6, 1.8), the person's id (1.3, step 7), or a page of the
  repositories a picker shows (1.8).

### 1.3 The steps

Seven steps, in this order. Each reads first and writes only where the
forge differs.

1. **The admin.** It asks for the admin's token: absent or empty stops
   the run with `Missing`. `GET /api/v1/user` with it must answer a
   site admin, or the run stops with `NotAdmin`. Either failure names
   the admin by the given name. Every later request carries the token,
   but step 3's check of a kept token.
   - Its scopes, four: `write:admin`, for making users and orgs and
     managing users' tokens (steps 2, 3 and 5, and 1.7); `read:user`,
     for `GET /api/v1/user` and the bot's and the person's user reads;
     `write:organization`,
     for reading orgs (step 5) and making their repositories
     (`CreateRepository` below); and `write:repository`, for a repository's
     read, collaborator and hooks (1.5), and the admin's search (1.8).
     Reconcile alone needs only `read:organization`; setup's repository
     creation needs the write scope, which includes that read. The owner
     mints the token with all four, or tells the person who mints it to.
     A site admin still needs each route's token scope: a missing one
     answers `403`, and the run stops `Refused(403)` at the step that sent
     it. Section 2 records the least scope for each route.

2. **The bot.** It reads the user of the bot's login.
   - Absent (404): it draws 32 bytes and makes the user (`POST
     /api/v1/admin/users`) with them as 64 lowercase hex digits for a
     password nobody keeps, and `must_change_password` false, since
     nobody signs in to change it. The user is never a site admin, and
     its full name carries the mark, `<word>:bot`, set by the request
     that makes it, so no run cut short leaves the bot unmarked.
   - Present: a full name without the mark stops the run with
     `Unmarked`, so a login written wrong never takes over a person's
     account; a marked user that is a site admin stops it with
     `BotIsAdmin`. The mark is found whole: at the name's start or after
     a byte outside `[a-z0-9-]`, and at its end or before such a byte,
     so the word `b` is not found in `ab:bot`.
   - A bot that passes is never edited: its email is set at its making.
3. **Its two tokens,** the API's, then git's. It asks for the one kept,
   which stays when `GET /api/v1/user` with it answers as the bot.
   - Otherwise (none kept, a 401 or a 403, or another user's answer) it
     is made again by the admin's token alone: the bot's tokens are listed
     (`GET /api/v1/admin/users/{login}/tokens`, paged) for the one of
     the token's name, which is deleted (`DELETE .../tokens/{id}`; a 404
     is as good); then made again (`POST` to the list's path) and the
     value in the answer is written to the host. A 403 is a token
     without `read:user`, as one made for git alone is. Any other
     status stops the run (`Refused`), and so does a `Down` (1.4).
   - The scopes: `read:user`, `write:issue` and `write:repository` for
     the API token; `write:repository` and `read:user` for git's, so
     that this step can ask whose it is.
   - It never uses the bot's password and never signs in as the bot.
4. **The bot's id.** It asks for the id kept, and writes it (`README.md`, 4.4) when
   it differs.
5. **Each org to make,** in the order given, when the forge is owned:
   an absent org (404) is made (`POST
   /api/v1/admin/users/{owner}/orgs`), private and owned by the orgs'
   owner. A present one is left as it is. No team is made: the bot's
   reach is each adopted repository's (step 6). With no orgs given, the
   step reads nothing.
6. **Each adopted repository's access,** in the order given: the bot a
   collaborator, and its hook when the listener is on loopback, checked
   and repaired as 1.5 says. With none adopted, the step reads nothing.
7. **The person,** when a login is given (1.1): the user of that login,
   by the admin's token, the bot's user read's route, and the run ends
   `Done(Person { id })`, that user's id.
   - Absent (404): the account is gone, which is no failure, and the run
     ends `Done(Person { id: none })`.
   - An id of 0 stops the run `Unexpected`; any other status, or a `Down`,
     stops it as any step's does (1.4), at `Person`, naming the login.
   - With no login given the step reads nothing, and the run ends
     `Done(Nothing)`.
   - It writes nothing and keeps nothing: what the id means is the owner's.

The Forgejo dialect also provides `CreateRepository { org, name, private }`
for the owner to make a person's first repository: `POST
/api/v1/orgs/{org}/repos`, the org encoded as one path segment. Its JSON
body fixes `auto_init` to `true` and `default_branch` to `main`; Forgejo
writes the README. The answer is `Document::CreatedRepository`, with
`id`, `full_name`, `default_branch`, `clone_url` and `ssh_url`, sufficient
to adopt and clone the repository. The existing repository-read document
carries settings rather than clone URLs, so creation has its own document.
`Create` (1.9) sends it, grants the bot and puts the owner's files on
`main`, and the owner adopts the repository it names.

`CreateFile { owner, name, path, branch, content, message, author }`
puts one file on a branch through Forgejo's contents API: `POST
/api/v1/repos/{owner}/{name}/contents/{path}`, the path's segments each
encoded, with `content` in base64, the branch, the message, and the
author as both author and committer (a name and an email). It answers
`201` with `Document::CreatedFile`, the new commit's id. A file already
there is `422`, a missing repository or branch `404`.

As with `CreateOrg`, refusal statuses are preserved for the caller:
`409` names an existing repository, `422` an invalid name, `404` a missing
org, and `401` or `403` a caller who cannot create it. The shared request
transport retains its encoding bounds, malformed-answer and connection
failures. The creation codec adds no retry or transport policy.

`Migrate { at, from }` copies a public repository into `at` by
Forgejo's own migration: `POST /api/v1/repos/migrate`, with `from` as
`clone_addr`, `at`'s owner and name as `repo_owner` and `repo_name`.
Its JSON body fixes `service` to `git` and `private` to `true`, and
`mirror`, `issues`, `labels`, `milestones`, `pull_requests`,
`releases`, `wiki` and `lfs` to `false`, so nothing but git comes. It
answers once the copy is whole, however long that takes, `201` with
`Document::CreatedRepository`, as `CreateRepository` does, its
`default_branch` the source's. A refusal keeps its status, and its JSON
`message` goes up as words (`README.md`, 4.3); section 2 names each.

`DeleteRepository { at }` deletes one: `DELETE
/api/v1/repos/{owner}/{name}`, each encoded as one path segment. It
answers `204` with no body, `404` when the repository is not there.

### 1.4 Converging, bounds and failures

- **From Forgejo, every time.** A second run with nothing changed
  writes nothing to the host and only the hooks' patches to the forge.
  A bot deleted, a token revoked, a hook deleted or a collaborator
  removed is made again at the next run.
- **Bounded:** the orgs at most `orgs`, the repositories at most
  `repositories`; every listing paged by the forge's page cap, read once
  from `GET /api/v1/settings/api` (`max_response_items`; zero is
  `Unexpected`) and lowered to the host's `inner` when larger, a page
  shorter than that cap being the last; the run
  within `deadline`, and a `Copy` within `copy_deadline` (1.10). The
  limits are `orgs`, `repositories`, `reachable`, `files`, `file_bytes`,
  `deadline`, `copy_deadline`, `backoff` and `backoff_max`, each at
  least 1, with
  `backoff <= backoff_max < deadline <= copy_deadline`; the crate names
  the pair that breaks it.
- **It waits for the forge.** The host makes the first request again on
  a `Down` or a 5xx, after `backoff`, doubling, never past `backoff_max`
  nor the time `deadline` leaves (`README.md`, 4.2, 4.3), so a host may start
  reconcile beside Forgejo with no readiness probe. After the first
  answer a `Down` stops the machine.
- **A failure** is `{forge, step, subject, why}`, and shows no secret
  (../../runtime/lib.md, section 6). The step is `Admin`, `Bot` (its id included),
  `Token`, `Org`, `Repository` (its collaborator and hook), `Person`
  (1.3, step 7), `File` (1.9), and for
  GitHub's plans `App`, `Installation` (a picker's too, 1.8) and `State`
  (`github.md`, section 1). The subject is the admin's given name, the bot's login,
  the token's name (`api` or `git` when connected, 1.6), the org's name,
  the person's login,
  the repository's path, a file's path, or its id in decimal before its path is read,
  the App's slug or id, an installation's id, or the plan whose state it
  is. The why is `Missing`, `Refused(status)`, `NotAdmin`, `BotIsAdmin`,
  `Unmarked`, `OtherUser`, `OtherApp`, `KeyForm`, `Unknown`, `Busy`,
  `Taken`, `Empty`, `TimedOut`, `Down` or `Unexpected` (`Taken`, `Empty`
  and `TimedOut` are `Copy`'s, 1.10). It stops the run and leaves each
  value written whole and valid.
- **Several services on one forge** each have their own word and bot,
  so their own collaborators and hooks; another's bot is `Unmarked`.
- **What it cannot set:** Forgejo's settings its API does not change,
  the webhooks' `ALLOWED_HOST_LIST` and whether Actions are enabled. A
  forge whose host list refuses the listener delivers no hook, and
  polling keeps up.

### 1.5 Access to a repository, at adoption and at each start

`Adopt { forge, repository }` gives the bot access to one repository
when the owner adopts it, by the admin's token, before the owner's
first call to it; reconcile's step 6 runs the same for each adopted
repository at each start, so access a person removed is repaired.
`repository` is its id, or its path, for an owner who holds only the
path a person gave, of a repository the bot may not see yet. Reconcile's
are ids.

1. **The admin's token,** as step 1 (1.3). Reconcile's run has it
   already. Steps 2 to 4 need its `write:repository`: a `403` on any
   of them stops the plan `Refused(403)` at `Repository`, naming the
   repository's id before step 2 has read its path, and its path
   after; one given by its path is named by that path throughout.
2. **The repository,** `GET /api/v1/repositories/{id}`, or `GET
   /api/v1/repos/{owner}/{repo}` when given its path, each encoded as
   one path segment: its path, as the forge spells it. A
   `404` stops `Adopt` with `Missing`; reconcile passes over a
   repository the forge no longer has, which the owner's client finds
   gone.
3. **The bot a collaborator:** `GET
   /api/v1/repos/{owner}/{repo}/collaborators/{bot}/permission`; below
   `write`, `PUT .../collaborators/{bot}` with `{"permission":"write"}`.
   A repository the bot owns, or one whose org gives the bot more, is
   left as it is.
4. **Its hook,** when the listener is on loopback; with none, or one
   elsewhere, the plan ends `Done` after step 3. A loopback listener is
   in reach of a forge on its own host alone, so an owner gives none to
   a forge elsewhere, whose hook would post, signed, to that forge's own
   loopback (`temper/docs/design/protocol/setup/reconcile.md`, 5).
   - The URL is `http://<address>/hooks/<number>`, the address being
     `127.x.y.z:port` or `[::1]:port`.
   - The secret is asked for once. When the host has none, the plan
     draws 32 bytes, renders 64 lowercase hex digits and writes them.
   - The repository's hooks are listed (`GET .../hooks`, paged). Of
     those with that URL the lowest id is kept and each other is deleted
     (a 404 is as good), since each would deliver every change again. A
     hook of another URL is left alone.
   - The kept hook is patched at every run, or made when none is kept:
     the URL, the secret, the subscribed events (../README.md, section 6),
     JSON and active. Forgejo never shows a hook's secret.

A collaborator or a hook on a listing's second page is found there,
neither repeated nor passed over.

### 1.6 Connected's two tokens, checked

`Check { forge }` runs when a person connects a Forgejo, and when they
replace its tokens. It asks for the API token and git's, as the person
gave them (the owner serves them from what the request carried), and
the user kept, if any.

- `GET /api/v1/user` with the API token answers the identity: its id
  and login, which the owner shows ("connected as temper, id 7"). One
  that is a site admin is refused `BotIsAdmin`: the service acts as no
  admin.
- `GET /api/v1/user` with git's must answer the same user. Either token
  refused (a 401, or git's 403 for a missing `read:user`) stops the plan
  `Refused(status)`; two users stop it `OtherUser`, each naming which
  token, `api` or `git`.
- A user kept that differs stops it `OtherUser`: a replacement answers as
  the user the forge already has. Otherwise the plan writes the user's id
  and ends `Done`; the owner keeps the tokens and the secret it was given.

### 1.7 An owned forge's admin's token, taken over

`TakeOver { forge, admin_token }` takes the token the owned forge's
preparation left in its handover file, at the service's start. The
owner reads the file and hands its value; the plan:

1. checks it as step 1 (1.3), naming the admin;
2. writes it as the admin's token;
3. lists the admin's own tokens (`GET
   /api/v1/admin/users/{admin}/tokens`, paged, the login from step 1),
   and deletes each but the one whose last eight characters are the
   taken token's, by step 3's routes (1.3), so no older admin's token
   stays valid.

The owner removes the file after `Done`, never before, and then runs
reconcile as for a managed forge. Each preparation makes a new token;
a start with no file reads the token kept.

### 1.8 The repositories a picker shows

`Reachable { forge, installation, page }` reads one page of the
repositories a forge reaches, for the owner's picker: the person picks
among them the ones to adopt (1.5). It writes nothing, to the forge or
to its host, so it runs beside the forge's one writing plan (`README.md`, 4.3).
Every picker's read is this plan's; the forge client has none.

- **`installation`** is an installation's id on GitHub (`github.md`, 1.9), and none
  on Forgejo. One given where the forge's kind takes none, or none where
  it takes one, stops the run at once, `Installation`, `Unexpected`,
  and sends nothing; the subject is the id given, or `none`.
- **On Forgejo, the token kept decides the listing:** the admin's
  token, when one is kept (managed or owned); else the API token that
  1.6 checked (connected). Forgetting the admin's token makes the forge
  connected (`README.md`, section 3), and its picker with it.

1. **The token.** The admin's token, read first, checked as step 1
   (1.3): `NotAdmin`. When none is kept, the API token; none of either
   is `Missing` (`Token`, `api`). The admin's search needs its
   `write:repository`: a `403` there stops the run `Refused(403)`,
   `Token`, naming the admin.
2. **The page cap,** read from `GET /api/v1/settings/api` as 1.4 says.
3. **One page,** `n` being the least of `reachable` and the cap:
   - with the admin's token,
     `GET /api/v1/repos/search?sort=id&order=asc&page=<page>&limit=<n>`.
     The answer is `{"ok": true, "data": [...]}`, the rows in `data`;
     one whose `ok` is not true is `Unexpected`. An admin's search finds
     every owner's repositories, the private ones too;
   - with the API token, `GET /api/v1/user/repos?page=<page>&limit=<n>`:
     the repositories its user owns or reaches as a collaborator or
     through an org's team, the private ones too, as section 2 records,
     each with its `permissions`. A row is kept only when
     `permissions.push` is true: the service works only where it can
     write.
4. **The outcome,** `Done(Reachable { rows, more })`: each row the
   repository's id, its owner's id and login, its name, and whether it
   is private; `more` when the page held `n` rows before any was
   dropped, so a connected forge's page may hold fewer and have more.

- **Paged by number,** from 1, since neither listing has a keyset: the
  owner asks for `page + 1` while `more`. In a stable order a repository
  made while the person pages lands on the last page; one deleted moves
  the later rows back by one, so the next page may pass one over, and
  the next listing finds it.
- **Bounded** by `reachable`, at least 1, the rows of one page, held in
  memory with the outcome.

### 1.9 A first repository, made

`Create { forge, org, repository, files }` makes a repository for the
person on a managed or owned Forgejo, ready for the service: its org
when missing, the repository with a README, the bot's access, and the
files the owner gives, on `main`. The owner runs it by the admin's
token, after reconcile (1.3). Each step reads first and writes only
what is missing, so a run cut short, run again whole, finishes what the
last one left, and a second run writes nothing.

1. **The admin's token,** as step 1 (1.3).
2. **The org,** `GET /api/v1/orgs/{org}`: absent (404), it is made as
   step 5 makes one (1.3), private and owned by the orgs' owner (1.1).
   With no owner given, as managed has none, it is made owned by the admin
   the run has just verified (step 1), the token's own user.
   A present org is left as it is, whoever owns it.
3. **The repository,** `GET /api/v1/repos/{org}/{name}`: absent, it is
   made, `CreateRepository { org, name, private: true }` (1.3), which
   writes a README on `main`. A `409` there, the repository made
   between the read and the write, reads it again. A present one is
   left as it is. Either way the plan holds its id and its default
   branch. A failure stops at `Repository`, the subject `org/name`.
4. **The bot's access,** as 1.5's steps 3 and 4: the bot a collaborator
   with `write`, and its hook when the listener is on loopback.
5. **Each file,** in the order given, on the default branch: its head
   read once (`Branch`), then the file at that head (`Raw`); absent
   (404), it is put there, `CreateFile` (1.3),
   its message `Add <path>`, its author the bot by its login and email
   (1.1), so the history names the service that wrote it. A file
   present is left as it is, whatever it holds: after its first commit,
   it is the person's. A failure stops at `File`, the subject the
   path.
6. **The outcome,** `Done(Created { id })`: the repository's id, which
   the owner adopts (1.5).

- **The files** are the owner's: a path and its bytes each, at most
  `files` of them, each path a relative one of at most 255 bytes whose
  segments are neither empty, `.` nor `..`, each content at most
  `file_bytes`. The org follows `given::login`'s grammar; the repository name is
  1 to 100 bytes of `[A-Za-z0-9._-]`, neither `.` nor `..`. Both must
  fit the host's `string_bytes`. `given::check_create` checks a plan against them
  and the owner checks first; a plan that fails it is the owner's bug,
  which the host asserts, as for `Reconcile`'s `adopted` (`README.md`, 4.3).
- **Actions** are the forge's, not the plan's: a repository made on a
  Forgejo whose Actions are enabled has them, and runs a workflow
  among its files from that file's push (section 2). The plan sends no
  setting.
- **A writing run,** one per forge at once (`README.md`, 4.3).

### 1.10 A public repository, copied

`Copy { forge, org, repository, from, files }` copies a public
repository into a managed or owned Forgejo, ready for the service. The
owner runs it by the admin's token, after reconcile (1.3), as it does
`Create`. Forgejo fetches the source itself; no byte of it passes
through the host, and no git credential is made for it. Each step reads
first and writes only what is missing, so a run cut short, run again
whole, finishes what the last one left.

1. **The admin's token,** as 1.9's step 1.
2. **The org,** as 1.9's step 2. A name that is a user's on the forge
   reads 404 as an org, and `CreateOrg` refuses it: the run stops at
   `Org`, `Refused(status)`, the subject the name (section 2).
3. **The repository,** `GET /api/v1/repos/{org}/{name}`:
   - absent (404), the plan writes `Copying` (`README.md`, 4.4), then sends `Migrate
     { at, from }`. Its `201` gives the id and the default branch. Its
     `409`, the name made between the read and the write, stops at
     `Taken`.
   - present, the plan reads `Copying`. None: the repository is not
     this copy's, the run stops at `Taken`, the subject `org/name`, and
     has written nothing but the org.
   - present and `Copying` kept: the plan reads its default branch
     (`Branch`). There, it is this copy, finished. A 404, it is what a
     copy cut short or refused left, or a copy Forgejo is still making
     (section 2): the plan sends `DeleteRepository { at }`, takes a 404
     as good, writes `Copying` again and sends `Migrate`.

   After a `Migrate`'s `201`, the plan reads the default branch too. A
   404 there is a source with no branch: the run stops at `Empty`, and
   the repository stays for the owner's next run to delete.
   Either way the plan holds the repository's id and its default
   branch, the source's. A failure stops at `Repository`, the subject
   `org/name`.
4. **The bot's access,** as 1.5's steps 3 and 4.
5. **Each file,** as 1.9's step 5, on the default branch. A file the
   source already has is left as it is.
6. **The outcome,** `Done(Copied { id, branch })`: the repository's id
   and its default branch, which the owner adopts (1.5).

- **`from`** is an address the owner has checked: `https://`, a host,
  then an owner and a name, as one string of at most `string_bytes`.
  `given::check_copy` holds a plan to the org's, the name's and the
  files' rules of 1.9, and `from` to: starts `https://`, then at least
  one byte, and no byte outside visible ASCII. The rest of an address's
  grammar is its owner's, and whether its host answers is Forgejo's.
- **`Copying`** is the owner's record that this copy's `Migrate` has
  been sent, kept with its own record of the copy. It is what tells a
  repository this copy left from one somebody else has, so that the
  plan never deletes or adopts a repository it did not start. The
  owner clears it when a run stops at `Taken`.
- **Its deadline is its own,** `copy_deadline`, where every other plan
  has `deadline`: a copy's time is the source's size over the network's
  speed, and its `Migrate` is one exchange unanswered for all of it. No
  time but the run's bounds that exchange, and it is never sent again.
  A run past it stops at `Repository`, `TimedOut`, its connection aborted
  (`README.md`, 4.5); every other plan's stops `Down`, as does a `Copy` a `Remove`,
  an `Abort` or a `Close` ends. The copy Forgejo may still be making is
  found at the next run (step 3).
- **Forgejo's refusals** of `Migrate`, each a stop at `Repository`,
  `Refused(status)`, its message going up first (`README.md`, 4.3): a source
  missing or private, a host refused by Forgejo's migration settings, a
  source that is not a git repository. A missing or private source leaves
  an empty repository at the name (section 2), which `Copying` names as
  this copy's, so the same copy, run again, deletes it (step 3).
- **Not for a connected Forgejo or GitHub,** where the owner holds no
  admin's token; `supports` refuses it, as it does `Create`.
- **A writing run,** one per forge at once (`README.md`, 4.3): a second writing run
  beside a `Copy` stops `Busy` for as long as the copy lasts, so an
  owner that copies holds its other writing runs until the copy ends,
  and copies on one forge run one after another.

## 2. What reconcile relies on of Forgejo

Each fact is recorded by the conformance on the throwaway Forgejo at the
pin (../README.md, section 4.3), and named by the fact's name.

- **Users and tokens, by the admin's token** (steps 1 to 3, and 1.7):
  - the admin's token reads as a site admin; another's is refused:
    `setup-the-admin-token-reads-user-as-a-site-admin`,
    `setup-a-non-admin-token-is-refused-the-admin-routes`;
  - a made user is no site admin and keeps its full name; a missing
    login is a 404:
    `setup-create-user-answers-201-with-login-email-and-full-name`,
    `setup-created-user-is-no-site-admin-and-keeps-its-full-name`,
    `setup-user-of-a-login-reads-admin-flag-and-full-name`,
    `setup-user-of-a-missing-login-is-404`;
  - tokens list by name, scopes and last eight characters, never value,
    and page; are made with their value, refused under a used name, and
    deleted:
    `setup-token-list-shows-names-and-scopes-without-values`,
    `setup-token-list-shows-the-last-eight`,
    `setup-token-list-pages-by-page-and-limit`,
    `setup-admin-route-makes-a-token-answering-201-with-name-scopes-and-value`,
    `setup-token-of-an-existing-name-is-refused`,
    `setup-token-delete-answers-204-and-ends-the-token`;
  - the scopes: listing tokens needs `read:admin`, making and deleting
    them `write:admin`; `GET /user` and `GET /users/{login}` need
    `read:user`; the repository routes, 1.5's and the search, the repository
    scope (1.3, step 1); a git token `read:user`; an API token
    `write:issue`:
    `setup-admin-token-without-write-admin-is-refused-the-token-routes`,
    `setup-admin-token-without-a-user-scope-is-refused-user`,
    `setup-admin-token-without-write-repository-is-refused-the-repository-routes`,
    `setup-a-git-token-of-write-repository-alone-is-refused-user`,
    `setup-a-git-token-of-write-repository-and-read-user-answers-user-as-the-bot`,
    `setup-api-token-without-write-issue-is-refused-an-issue-write`.
- **Least token scopes, measured per route.** A site-admin token with
  only the listed scope succeeds; one carrying only unrelated `read:issue`
  gets `403` naming the required scope. Forgejo refuses to mint a token
  with an empty scope list. Write includes read within its own category,
  and `all` includes every category. The twenty-seven `setup-scope-*`
  facts record these grants and refusals; settings is
  `setup-scope-settings-needs-no-scope`. A `DeleteRepository` in an org
  takes `write:organization`
  (`setup-scope-delete-repository-is-write-organization`), so
  `setup-scope-delete-repository-is-write-repository` and
  `setup-scope-delete-repository-without-write-repository-is-refused`
  are recorded `fails`.

  | Request | Least scope |
  |---|---|
  | `User`, `UserOf` | `read:user` |
  | `CreateUser` | `write:admin` |
  | `Tokens` | `read:admin` |
  | `CreateToken`, `DeleteToken` | `write:admin` |
  | `Org` | `read:organization` |
  | `CreateOrg` | `write:admin` |
  | `CreateRepository`, `DeleteRepository` | `write:organization` |
  | `RepositoryById`, `Repository`, `Branch`, `Raw`, `Permission`, `RepoHooks`, `RepoSearch` | `read:repository` |
  | `AddCollaborator`, `CreateRepoHook`, `EditRepoHook`, `DeleteRepoHook`, `CreateFile`, `Migrate` | `write:repository` |
  | `ApiSettings` | none; the route also answers anonymously |

  The conformance runner's `--admin-scopes` mode records each grant and
  refusal on a fresh loopback Forgejo, and `--migrate` those of `Migrate`
  and `DeleteRepository`. The four-scope token succeeds on
  every route; removing each category is refused at a request that needs
  it (`setup-admin-scopes-need-each-of-the-four-categories`). The fake enforces these scopes
  through its issued-token model; its configured tokens have unrestricted
  scope.
- **A first repository** (1.9): an admin's `CreateRepository` in a
  person's private org answers 201 with a README commit on `main`; a
  second answers 409; a repository made so on a Forgejo whose Actions
  are enabled has them, with no setting sent; `CreateFile` answers 201
  with its commit on the branch, whose head's raw read then finds it,
  and 422 for a file already there; the raw read of a missing file is
  404:
  `setup-repository-made-in-an-org-is-201-with-a-readme-on-main`,
  `setup-repository-made-again-is-409`,
  `setup-repository-made-has-actions-when-the-forge-enables-them`,
  `setup-file-put-is-201-and-read-back-on-its-branch`,
  `setup-file-put-again-is-422`,
  `setup-file-missing-reads-raw-404`.
- **A copy** (1.10), from a source served by a second throwaway on
  loopback, with `ALLOW_LOCALNETWORKS` on in the first throwaway alone
  (../../testing/fakes/forge.md, section 7): `Migrate` refuses a plain `http` source
  until `ALLOW_UNENCRYPTED` is on; it answers 201 once whole, private,
  with the source's default branch and every branch; it brings no
  issues; a name already there answers 409; a missing source, and a
  private one asked with no credentials, are each refused with a message
  and leave an empty repository at the name, which reads 200, empty, with
  no branch; a source with no commit answers 201 with no branch; an address on a local
  network is refused by Forgejo's default settings; `OFFLINE_MODE`
  leaves migrations on; a migration cut short by stopping Forgejo leaves
  a repository that reads with no default branch, and it deletes and
  copies again; a migration whose client went away goes on, and a
  `DeleteRepository` while it runs answers 204, the repository stays
  gone once the migration ends, and the name copies again;
  `DeleteRepository` answers 204 and a second 404; a name that is a
  user's reads 404 as an org and `CreateOrg` refuses it; with
  `.forgejo/workflows` and `.github/workflows` both there, Actions reads
  the first alone:
  `setup-migrate-loopback-http-needs-allow-unencrypted`,
  `setup-migrate-answers-201-whole-and-private`,
  `setup-migrate-keeps-the-sources-default-branch`,
  `setup-migrate-brings-nothing-but-git`,
  `setup-migrate-of-a-name-taken-is-409`,
  `setup-migrate-of-a-missing-source-says-why-and-leaves-an-empty-repository`,
  `setup-migrate-of-a-private-source-says-why-and-leaves-an-empty-repository`,
  `setup-migrate-of-a-source-with-no-commit-is-201-with-no-branch`,
  `setup-migrate-of-a-local-address-is-refused-by-default`,
  `setup-migrate-runs-with-offline-mode`,
  `setup-migrate-cut-short-leaves-a-repository-with-no-branch`,
  `setup-migrate-cut-short-deletes-and-copies-again`,
  `setup-migrate-goes-on-when-its-client-goes`,
  `setup-migrate-deleted-while-it-runs`,
  `setup-repository-delete-is-204-then-404`,
  `setup-org-of-a-users-name-reads-404-and-making-it-is-refused`,
  `setup-actions-reads-forgejo-workflows-alone-when-both-are-there`.
- **Orgs** (step 5): an org an admin makes for a user is private and
  takes `write:admin` alone; a missing org is a 404:
  `setup-org-made-for-a-user-by-an-admin-is-201-and-private`,
  `setup-org-made-for-a-user-takes-write-admin-alone`,
  `setup-org-missing-is-404`.
- **Repositories' access** (step 6, 1.5): a repository reads by id for
  the admin, a private one too; an admin's `PUT` of a collaborator
  answers 204 and grants `write`, which its permission reads back; a
  repository's loopback hook is made, JSON and active, lists its events
  and never shows its secret; two may share a URL; a patch or a deletion
  leaves the repository's others; a listing pages; `ALLOWED_HOST_LIST`
  refuses loopback unless it names the listener:
  `setup-repository-by-id-reads-its-path-for-an-admin`,
  `setup-collaborator-put-by-an-admin-answers-204-and-grants-write`,
  `setup-collaborator-permission-reads-back-write`,
  `setup-repository-hook-for-a-loopback-address-is-made`,
  `setup-repository-hook-is-json-and-active`,
  `setup-repository-hook-lists-the-events-it-was-made-with`,
  `setup-repository-hook-never-shows-its-secret`,
  `setup-repository-hook-of-one-url-can-be-made-twice`,
  `setup-repository-hook-patch-and-delete-leave-the-others`,
  `setup-repository-hook-listing-pages-by-limit`,
  `delivery-loopback-is-refused-by-default`,
  `delivery-loopback-is-allowed-when-named`.
- **The admin's search** (1.8): it lists to a site admin every owner's
  repositories, the private ones too; answers `ok` and `data`; and pages
  by `page` and `limit` in id order:
  `setup-search-by-an-admin-lists-every-owners-private-repositories`,
  `setup-search-answers-ok-and-data`,
  `setup-search-sorted-by-id-pages-by-page-and-limit`.
- **A user's repositories** (1.8, connected): `GET /user/repos` on a
  user's token lists the repositories it owns, and those it reaches as a
  collaborator or through an org's team, private ones too, each with
  `permissions.push`; and pages by `page` and `limit` in a stable order:
  `setup-user-repos-lists-owned-collaborating-and-team-repositories`,
  `setup-user-repos-rows-carry-permissions-push`,
  `setup-user-repos-pages-by-page-and-limit`.
- **Paging:** the page cap is 50 by default
  (`api-default-page-cap-50-and-30`).
- **Order:** reconcile relies on none after a delivery.
