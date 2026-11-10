# Connecting GitHub

GitHub is managed or connected, never owned. Its managed mode means an App
made through the manifest, not Forgejo's string-managed admin connect (`reconcile.md`, 5).
Both GitHub modes end in the same
records (`README.md`, section 3), and the engine runs both alike after; they differ
in who made the App. The plans are skein's (`forge/setup/github.md`, section
1), run by the setup host (`reconcile.md`, 1); the browser's routes are the clients'
listener's (`../../client/forge/README.md`, the browser's steps).

## 1. GitHub's App and its permissions

- **One App per deployment,** public, so that it installs on any
  account (skein's `forge/setup/github.md`, section 1). Its permissions:
  contents, pull requests, issues and merge queues, write; checks,
  statuses, actions, administration and metadata, read; never
  workflows, never administration's write. Its events are skein's
  (`reconcile.md`, 6). Beside them (`../../domain/landing.md`, 8): statuses and actions,
  write,
  so that a project landing by turn has its gates dispatched and its
  receipts and verdicts posted; installations made before keep reading
  only until their owner accepts the new permissions, and adoption says
  so. The `Always` bypass the turn needs is a repository's ruleset's,
  granted by its admin, never asked by the App. `workflows` stays
  unasked: a change under `.github/workflows/` (a gate's or a tier's
  definition, or a member rebased over one) is pushed by a person, and
  whether GitHub refuses a ref update or a merge that brings one
  without it is to check.
- **Its hook** points at `https://<slug>.invalid/`, an address that
  never resolves, so GitHub delivers nowhere and holds no secret of
  temper's; the App's changes come from its delivery log (6).
- **Its records:** `AppId`, `AppSlug`, `AppKey` (DER; a PEM is never
  kept), `User` (the App's bot user), `Accepted` and `Origin`
  (../credentials.md, the records).
- **Its addresses,** its record's place, are the machine's GitHub
  place's, github.com's when the machine's `github` is null (`init.md`);
  the machine names a place only to point at a fake (`box.md`, 5).
- **The clients' listener at a fixed address,** a loopback port the
  person's browser reaches, since GitHub's redirect and setup URLs name
  it; a GitHub `Connect` on its port 0 is refused, naming
  `application.clients.listener` (../forge/engine.md, 1; ../configuration/engine.md
  ).
- **A repository's own rules** are read and said at adoption, and never
  changed (`../../domain/forge/README.md`, what adoption says of the branch's
  rules): a merge queue is taken as it is; required approvals need the
  App on the rule's bypass list or a person's approval; required
  signatures, linear history and code owners are each named with the
  one setting that lets temper land.

## 2. Managed: the manifest flow

1. **Connect,** "Connect GitHub" on the deployment's page or its tool
   of `temper ctl` (`Connect` with `GitHubManifest { back? }`). The
   setup host runs `Manifest { forge, target }` (skein's
   `forge/setup/github.md`, 1.1), `target` always `Personal`, for a fresh
   state, ending `Done(Manifest { state, target, manifest })`; temper
   renders the page from them (../../client/forge/served.md, 1) and holds it and
   `back` with the state. With `back`, the link is a path on the
   page's own origin, `/forges/<n>/github/manifest?state=<s>`: the
   browser goes to it in the page's own tab,
   and comes back to `back` at the end. Without `back`, the link is
   the whole address on the clients' listener,
   `http://<listener>/forges/<n>/github/manifest?state=<s>`; the
   terminal prints it and never opens it.
2. **The manifest's page** posts the manifest to
   `<web>/settings/apps/new?state=<s>` as it loads, by one inline
   script its policy allows by hash, and shows its button where the
   script doesn't run or the page was reached through the browser's
   history; its headers allow a form to `<web>` alone, keep no copy
   and send no referrer.
3. **GitHub's form:** the person creates the App, under their own
   account. Nothing asks for an organization: the App is public, so
   GitHub's install page offers every account the person may install
   it on (with the user, 2026-10-23; an App an organization owns is
   for later, `../../draft/github.md`, section 18).
4. **The redirect back,** `http://<listener>/forges/<n>/github/redirect`:
   `Convert { forge, code, state }` (`reconcile.md`, 2) exchanges the code for the
   App's id, slug and key, finds its bot user, and writes the records,
   `AppMade` first, whose bytes are the App's id (../credentials.md, the
   items); the conversion's PEM, webhook secret and OAuth pair are never
   kept.
   Once they are kept, `Install` (`reconcile.md`, 4) answers the App's install link,
   the browser goes on to it, and the forge connects (../forge/README.md, the
   setup's host).
5. **Installing:** the person picks an account, and all repositories,
   GitHub's default, or those they select. GitHub sends the browser to
   `http://<listener>/forges/<n>/github/setup`, where `Accept` (`reconcile.md`, 4)
   checks the installation and adds it to `Accepted`, and the route
   sends the browser back to `back`, or, with none, says "temper has
   the installation; you may close this tab." A member of an
   organization who isn't its owner can only request the App there:
   GitHub comes back with `setup_action=request` and no installation,
   and the route says why temper can't go on, and to install it on
   their own account or an organization they own (../../client/forge/served.md,
   1).

- **The App's account sets the forge's link to its person,** on a box,
  from each `GET /app` a plan answers (`../../draft/forge-identity.md`,
  4.2): when `AppMade` equals `AppId` and the account's `type` is `User`,
  the link is its id, and anything else ends it. A read that fails changes
  nothing, and that's logged.
- **States** are one use, drawn by the setup host, at most
  `setup.states`, each living `setup.state_life_ms`, at most the
  conversion code's hour; a used, unknown or expired one is refused,
  "This link has ended. Start again from temper.", and a restart
  forgets them all.
- **The redirect and setup URLs** temper gives the plans are whole,
  built from the clients' listener's address (../forge/README.md, the setup's
  host). The manifest's `url`, the App's homepage, is
  `http://<listener>/`.

## 3. Connected: an App pasted

- **The person makes an App** on GitHub with 1's permissions and
  events, and pastes its id and its private key as GitHub gives it,
  PEM: "Connect an App of your own" (`Connect` with `GitHubApp { id, read_only?, key
  }`). No webhook secret is asked: the paste check sets the App's hook
  unless `--read-only` leaves it as read (7).
- **Its setup URL is the person's to give** on GitHub's form, since no
  plan can set it: `http://<listener>/forges/<n>/github/setup`, with
  redirect on update, no callback URL and no user's authorization at
  install (the walk `github`). An installation of an App without
  it never comes back through the setup route, and is a stray (4).
- **The paste check,** `Paste { forge, app }` (skein's
  `forge/setup/github.md`, 1.8): the key, kept by jig until its run ends,
  decoded to DER; the App read with it, its id the one pasted, else
  `OtherApp`; its bot user found; its hook set to
  `https://<slug>.invalid/` unless read-only (7); its permissions and events checked as its
  reconcile checks them. It writes `AppId`, `AppSlug`, `AppKey` and
  `User`, the records the manifest flow writes, and ends done, or
  `Lacking` with each permission or event named, never refused for it.
  A forge whose `app_read_only` is true has its hook read and left
  (7).
- **A restart during the paste check** drops its run: the carried key
  is erased at start, and the forge reads as its records say
  (`../credentials.md`, 2.2; a stored connect, which would say
  "interrupted; paste again", is an open question there).
- **Installing** is 4's.
- **Replace,** a new key: the paste check with the kept id, so a key of
  another App stops it `OtherApp` and the forge stays as it was.

## 4. Another account, and repositories later

- **"Install on another account"** (`Install { forge, back? }`,
  skein's `forge/setup/github.md`, 1.4): a fresh state and GitHub's install
  link (`Link`), where the browser goes as at 2; the person installs
  on another account or org, and the setup route accepts it and sends
  the browser back.
- **Repositories added later** to an installation of selected
  repositories are the person's, on the installation's page on GitHub;
  GitHub's visit to the setup route with no state refreshes an accepted
  installation's repositories and sends the browser to Forges, and is
  refused otherwise. The forge
  component reads an installation's repositories again on its events.
- **An installation temper did not accept,** a stray made with no
  state of temper's, is shown as "not accepted · kept", regardless of
  age. A person's App's installations are theirs: temper uninstalls
  nothing it did not make. One deleted on GitHub leaves `Accepted`;
  one suspended is shown.
- **Accepting a stray,** `Accept { forge, installation }` (the tools'
  `forge accept`, `../../client/ctl/areas.md`): an installation that
  exists already, made on GitHub before temper knew it, is taken by the
  deployment's owner's own act, with no browser. The setup host runs
  `Accept` with no state (skein's `forge/setup/github.md`, 1.4): the person's
  sign-in stands for the state, which binds only a browser's return to
  the install its person started. The installation is read on GitHub
  as the App's, a `404` refusing it, and added to `Accepted` as the
  setup route's accept adds it. Its refusals, before any run, are
  `Install`'s; the setup route's own accept still requires a state.

## 5. Its reconcile

- **`Reconcile { forge }`** (skein's `forge/setup/github.md`, 1.6) runs after
  each `Connected`: the App's key, permissions, events and installations
  checked. It repairs only its hook's address, and with `app_read_only`
  (7) leaves the hook as read. A stray is always kept, never uninstalled.
- **A key GitHub refuses** stops the forge: the forge component, which
  signs with the same key, stops it (../forge/engine.md, 3), named, with
  Replace (3); the reconcile's own `Stop` is named to the root and
  changes nothing.
- **A permission or event lacking** is named, `Lacking`, never refused:
  the person grants it in the App's settings and approves it on each
  installation, as GitHub asks.

## 6. Job logs, and the delivery log

- **A job's log** lies behind a redirect to a storage host, which the
  forge component follows only to an origin its forge's log origins
  list (../forge/README.md, GitHub's API as temper uses it): on GitHub one origin
  by default, its kind's. An origin not listed is named on the
  deployment's page.
- **The delivery log** is read by the forge component every
  `log_poll`, and its deliveries become hints as Forgejo's webhooks do
  (../forge/README.md, a GitHub App's delivery log).

## 7. A pasted App another owns: read-only

- **`forge connect [FORGE] github-app APP FILE --read-only`** sets the
  kept forge's record's `read_only` (../forge/engine.md, 1), called
  `app_read_only` below. The wire's `GitHubApp.read_only` is optional, absent meaning
  false; it travels through the engine's `How::GitHubApp` to that record.
  It says the
  pasted App (3), its hook and its installations are another's to
  change: a probe App shared with skein's conformance, whose hook stays
  where its owner put it, as the live tier's (`../../testing/live-tier.md`
  ). The setup host gives skein `repair_hook: !read_only` and
  `uninstall_strays: false` (skein's `forge/setup/github.md`, 1.10). Without the
  flag, ctl says in one line that temper points the App's webhook at
  `https://SLUG.invalid/` and to connect with `--read-only` to leave it.
- **What changes:** the paste check (3) and reconcile (5) read
  the App's hook and leave it; reconcile lists the installations and
  keeps `Accepted` as it says, and a stray (4) is shown as kept,
  never uninstalled, until the person accepts it.
- **What doesn't:** the App's key, id, permissions and events are read
  and checked, a lacking one named; `Install`'s link, the setup route's
  `Accept` and the accept of a stray (4), which read; and the forge
  component's connection, its mints and its delivery log (6).
