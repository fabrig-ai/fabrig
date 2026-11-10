# Setting up GitHub

## 1. GitHub's plans

One App per deployment, owned by the connecting account, public, made by
its manifest (1.1, 1.2) or by the person (1.8). Its requests and their
documents are ../github.md's (`../github.md`, section 11); the plans say which, in
what order, and what each answer decides. Every App's request
(`/app...`) carries the App's JWT (1.3); the conversion and the bot's
lookup carry none.

### 1.1 The manifest

`Manifest { forge, target }` makes the form that sends the person's
browser to GitHub to make the App. `target` is `Personal` or `Org(login)`.

- **A pure function** of the given values: the word and a random part,
  three bytes drawn, as six lowercase hex digits, the name being
  `<word>-<random>`; GitHub's web origin, `<web>`, which is
  `https://github.com` or the fake's (../github.md, section 11); the
  deployment's URL; the redirect and setup addresses on the owner's
  clients' listener, whole URLs; and the state, which the host draws
  (1.7).
- **The manifest:** `name`; `url`, the deployment's; `public` true;
  `redirect_url`, the redirect address; `setup_url`, the setup address,
  with `setup_on_update` true; `hook_attributes`, the URL
  `https://<name>.invalid/` and active (1.5); and `default_permissions`
  and `default_events`, the wanted ones (1.6).
- **The form:** its target, `<web>/settings/apps/new?state=<state>`,
  or for an org `<web>/organizations/<login>/settings/apps/new?state=<state>`,
  and the manifest's JSON, which the form posts as its `manifest` field,
  each at most `page_bytes`. `Done(Manifest { state, target, manifest })`
  carries them and the state they were made for. The page is the
  owner's: it renders a page whose form posts the manifest to the
  target (its words, its script and its policy are the owner's, never
  the kit's), serves it at its own address for that state, and keeps it
  no longer than the state lives (1.7).

The person may rename the App on GitHub's page: the slug the conversion
answers is the App's, whatever the manifest said.

### 1.2 The conversion

`Convert { forge, code, state }` runs when GitHub redirects the
person's browser back with the code and the state. The host checks the
state first (1.7). Then:

1. **The conversion,** `POST /app-manifests/{code}/conversions`, no
   credential, within the code's hour. It answers the App's id, slug,
   owner, private key as PEM and webhook secret, and an OAuth client's
   id and secret. The plan keeps none of the OAuth pair, nor the webhook
   secret (1.5).
2. **The key, turned into DER:** the PEM's `RSA PRIVATE KEY` block, its
   base64 decoded, at most `key_bytes`. Any other block, a key past
   `key_bytes`, or base64 that does not decode stops it `KeyForm`.
3. **The App's read,** `GET /app` on the JWT, proving the key signs:
   the id answered must be the conversion's (`OtherApp` otherwise), and
   an `Unsigned` stops it `KeyForm`. It also reads the App's account
   (../github.md, section 11).
4. **The bot's id** (1.3).
5. **The writes:** `AppMade` first (`README.md`, 4.4), then the App's id, slug, key
   (DER) and bot's id. `Done(Made { owner })`, the account step 3 read,
   comes after the last is `Written`. `AppMade` goes first, as `Copying`
   does (`forgejo.md`, 1.10), so that an App's id a conversion kept always has its mark.

The owner answers the browser only after `Done`, so a person never sees
an App made whose key the service lost, and then connects the forge
(../connection.md, section 3.5) and offers the install link (1.4). A
code used twice, or past its hour, is refused by GitHub: `Refused(404)`
or the status it answers.

### 1.3 The bot's id, and the JWT

- **The bot** is the App's user, `<slug>[bot]`: `GET
  /users/<slug>%5Bbot%5D`, with no credential, answers its id. A `404`
  stops the plan `Missing`. The id is the access's user, which the
  component checks at each start on an installation's token
  (../connection.md, section 3.4).
- **The JWT** is the dialect's `github::jwt` (../github.md, section
  8.1), signed by the host with `skein_tls::sign_rs256` (../../net/tls.md, section
  4) on the wall's time, issued a minute in the past and valid ten
  minutes. The host signs one for each request; the plan never sees one.

### 1.4 Installing, and accepting

- **`Install { forge }`** answers the link that installs the App, with
  a fresh state: `<web>/apps/<slug>/installations/new?state=<state>`.
  It reads the App's slug and sends nothing. Installing on another
  account is the same plan again. Which repositories the installation
  takes is the person's choice on GitHub's page; ones granted later
  arrive as its repositories' change (../connection.md, section 4.4).
- **`Accept { forge, installation, state? }`** runs when GitHub's setup
  address brings the person back with the installation's id and the
  state, or, with no state, when the owner accepts an installation that
  exists already (made on GitHub before the service knew it) on its
  person's own act, taken under that person's authority (temper's
  tools, under the person's sign-in). A state binds a browser's return
  to the install its person started; an act of the person's needs none.
  The host checks a state as one of `Install`'s (1.7), and none
  without one. The plan
  reads the App's id and key and the accepted set, reads the
  installation (`GET /app/installations/{id}`; a `404` stops it
  `Missing`), adds its id and writes the set. The owner then hands the
  set down (../connection.md, section 4.4).
- **Only installations accepted this way are the service's.** Another
  account may install a public App: such an installation is never used
  until it is accepted. Reconcile keeps it unless the owner permits
  uninstalling strays (1.6, 1.10); an installation made before the
  service knew the App can be accepted with no state as above.

### 1.5 The hook, at no listener

The App's hook is at `https://<name>.invalid/` at its making, and at
`https://<slug>.invalid/` once reconcile or a paste sets it, active,
with no secret the service keeps. GitHub cannot deliver there, and keeps
each delivery in the App's log, which the component reads on the App's
JWT (../connection.md, section 6.5). So a deployment needs no address
GitHub can reach, and no webhooks' secret is kept for GitHub: an access
that carries one is refused (../connection.md, section 3.5).

### 1.6 The App's checks: GitHub's reconcile

`Reconcile { forge }` runs at each start, before the owner connects the
forge, and after a connect. It reads the App's id and key and the
accepted set; then:

1. **The App,** `GET /app`: a `401` is the key no longer taken, and
   stops it `Refused(401)`, which the owner shows with the offer to
   replace the key (1.8); another id stops it `OtherApp`. It also reads
   the App's account (../github.md, section 11). A read that fails
   stops the run as any does, so it answers no account, and what the
   owner holds from an earlier read stands.
2. **Its permissions and events** against the wanted: contents, pull
   requests and issues, and merge queues, to write; checks, statuses,
   actions, administration and metadata to read; never workflows, never
   administration to write. The events: push, pull request and its
   reviews and review comments, issue comments, issues, check runs and
   suites, statuses, workflow jobs and runs, branch creation and
   deletion, and merge groups. Each lacking one is named in the outcome,
   `Lacking`, at most the wanted's count, never refused: GitHub asks the
   person to approve what a newer service asks more, on the App's page.
3. **Its hook,** `GET /app/hook/config`; one whose URL is not
   `https://<slug>.invalid/` is set to it (`PATCH /app/hook/config`),
   when `repair_hook` is true (1.10); otherwise it is read and left.
4. **Its installations,** `GET /app/installations`, paged: an accepted
   id gone is dropped from the set, which is written; one not accepted,
   made more than `state_life` ago, is uninstalled (`DELETE
   /app/installations/{id}`; a `404` is as good) only when
   `uninstall_strays` is true (1.10). Otherwise it is kept for the owner
   to show and accept, regardless of age. A younger one may be waiting
   for its person's `Accept`.

The run ends `Done(Lacking { lacking, owner })`: what step 2 names, and
the account step 1 read.

The setup host's rows-only replies make an empty page the installation
listing's terminal marker; even a short nonempty page is followed.
`pages` counts that terminal response. The accepted set is pruned only
after a complete listing; permitted stray uninstalls happen even at the
page bound. A listing past `pages` ends `Crowded` at `Installation`,
naming `installations`, after any permitted uninstalls, leaving the
accepted set unpruned.

A second run with nothing changed writes nothing.

### 1.7 States, one-time

- **The host draws each state:** 16 bytes, as 32 lowercase hex digits,
  for `Manifest` and `Install`, bound to the forge and to the plan that
  will take it, `Convert` or `Accept`.
- **It lives `state_life`,** at most an hour, the conversion code's
  life, in the host's memory alone: at most `states`, the oldest going
  first when full. A restart forgets them; the person starts again.
  The owner keeps a manifest's page (1.1) only while its state lives,
  so at most `states` pages of `page_bytes` each.
- **Taken once:** `Convert` or `Accept` with a state that is unknown,
  used, expired, of another forge or of the other plan stops at once,
  `State`, `Unknown`, and sends nothing. The state is used when its plan
  starts, whatever its end.

### 1.8 An App a person made, pasted

`Paste { forge, app }` connects an App the person made, with its id
and private key pasted. The owner serves the pasted key as `Pasted`
(`README.md`, 4.4), read once and never written.

1. **The key:** the pasted PEM turned into DER, as 1.2's step 2.
2. **The App's read,** `GET /app`: its id must be `app`, and the id
   kept, when the forge has one, must be too (`OtherApp`); its slug is
   read.
3. **The bot's id** (1.3); a forge that has a user kept must find the
   same (`OtherUser`).
4. **Its hook** set, and **its permissions and events** checked, as
   1.6's steps 2 and 3; with `repair_hook` false it is read and left (1.10).
5. **The writes:** the App's id, slug, key (DER) and bot's id, then
   `Done(Lacking { lacking, owner })`, as 1.6's. It writes no `AppMade`
   and clears none: an App pasted over another has an id the mark doesn't
   equal, and a key pasted over the App made keeps its mark (`README.md`, 4.4).

The person installs it through `Install` (1.4). Replacing the key is the
same plan, with the id kept.

### 1.9 The repositories an installation reaches

`Reachable { forge, installation: Some(id), page }` (`forgejo.md`, 1.8) reads one page
of the repositories an installation of the App reaches, for the owner's
picker.

1. **The App:** its id and key, as 1.3, and `Accepted`. An id not in
   `Accepted` stops the run `Installation`, `Missing`: a picker shows
   only an installation the service accepted (1.4).
2. **The installation's token,** minted on the JWT, full
   (../github.md, 8.2). A `404` is `Missing`; another status is
   `Refused(status)`.
3. **One page:**
   `GET /installation/repositories?per_page=<n>&page=<page>`, on that
   token as `Bearer(secret)` (`forgejo.md`, 1.2), `n` being the least of `reachable`
   and 100: each row's `id`, `owner.id`, `owner.login`, `name` and
   `private` (`github::Reachable`, ../github.md, 8.3).
4. **The outcome,** `Done(Reachable { rows, more })`, as `forgejo.md`, 1.8's; `more`
   when the page holds `n` rows.

- **The token is dropped** when the page answers: never an item, never
  in an outcome, a failure or the trace, never kept past the run nor
  handed out. The next page mints again: one mint per page, accepted,
  since a picker is rare.
- **Paged and bounded** as `forgejo.md`, 1.8 says, from page 1, by `reachable`.
- **Live,** no plan of this section runs (`README.md`, section 5): the dialect's
  conformance reads an installation's repositories on the probe App
  (`github.md`, section 2), and revokes each token it mints.

### 1.10 The owner's permissions to change the App

GitHub's given values hold two independent permissions, `repair_hook`
and `uninstall_strays`. The owner chooses both for each forge.

- **`repair_hook` false:** 1.6's step 3 and 1.8's step 4 read
  `GET /app/hook/config` and complete whatever its URL, sending no
  `PATCH`. True permits the hook repair those steps describe.
- **`uninstall_strays` false:** 1.6's step 4 lists the installations
  and keeps the accepted set as it says, pruning an accepted id gone,
  and sends no `DELETE`; a stray stays the owner's to show and accept,
  however long before the connect it was made. True permits uninstalling
  a stray past `state_life`, as for strangers' installations of a public App.
- **A shared App:** the owner gives both as false when another person
  or tool uses it, as a probe App shared with a conformance suite is.
  temper always gives `uninstall_strays` false: a person's App's
  installations are theirs; temper uninstalls nothing it did not make.
- **The rest is unchanged:** `Manifest` and `Convert`, `Install`'s
  link, `Accept`'s read of an installation, `Reachable`'s mint and page,
  and the checks of the App's id, key, permissions and events, which
  still name what it lacks.

## 2. What GitHub's plans rely on of GitHub

Each fact is recorded by the conformance against github.com, on the
probe App and repositories alone, at the API's pin (../github.md,
section 12), and named in its section 13. No suite makes these calls. A
live run reads the App and changes nothing of it (`README.md`, section 5), so a fact
that needs the App changed is shown on the fake and stays `unchecked` on
github.com.

- **The manifest flow,** `unchecked` live: a manifest posted with a
  state comes back with a code and the same state; the conversion
  answers the id, slug, owner, PKCS#1 PEM key and webhook secret once,
  and refuses the code again; a hook at an `.invalid` address is taken,
  active.
- **The App's routes on its JWT:** `GET /app` answers its id, slug,
  account, permissions and events; `GET /app/hook/config` reads the hook's URL;
  installations list and read by id. Left `unchecked` live: a JWT of a
  revoked key is a `401`, `PATCH /app/hook/config` sets the URL, and an
  installation is deleted.
- **The bot:** `GET /users/<slug>%5Bbot%5D` answers its id with no
  credential; GraphQL's `viewer` on an installation's token answers the
  same.
- **An installation's repositories** (1.9): a full token minted on the
  JWT lists them by `per_page` and `page` at
  `GET /installation/repositories`, the private ones too, each with its
  id, its owner's id and login, its name and `private`; a mint for an
  id the App lacks is a `404`; a revoked token is refused.
- **Installing,** `unchecked` live, since it takes a person's browser:
  the install link with a state brings the person back to the setup
  address with the installation's id and the state.
- **To find:** whether an `installation_target` row reaches the log
  unsubscribed; whether an inactive hook shows in the App's read stays
  `unchecked` live (`README.md`, section 6).
