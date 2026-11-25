# The forge's part of the deployment

The forge's part of the deployment's connectors (jig's
`client/domain/pages.md`, section 7), on two pages of temper's
(`../pages/README.md`; with the user, 2026-10-23): `Forges`, which manages
the forges the person has, and `ConnectForge`, a flow that connects one
more. Each is read for a person whose role on project 0 is `Owner`, in
standalone the deployment's owner; any other's read is refused, in
jig's words (`../engine.md`). Their mockups are `forges.html`
and `connect-forge.html`.

## 1. The Forges page
- **The Forges page** lists the person's forges: each forge the store
  keeps (`../../protocol/forge/engine.md`, 1), fakes aside, in number order,
  whatever its connection; a fresh deployment lists none until one is
  set up or connected. It is Settings' `Forges` tab (`../pages/README.md`,
  *The `Forges` tab*), whose lede says "Each forge temper is connected to,
  or setting up. Add one to bring its repositories to temper.": the
  level-2 heading "Your forges · N", with "+ Connect a forge", a link to
  `ConnectForge`, in its head; with none, a quiet band: the tag "No forge
  yet", "No forge of yours yet." and "Connect a forge", the same link. With
  no project its level-1 heading is "Forges", and the lede follows it. While the playground runs, neither link is
  drawn: there is nothing to add (`family.md`, 7). Each row is a disclosure, in the rows' look
  (`../pages/README.md`, *The rows*), folded to its summary: its name, "Forge N · KIND · WEB" (its number, its kind and
  its `web` address), its connection's words (below) and its line:
  connected, its repositories, "OWNER/NAME · project PROJECT" each;
  setting up, its set up's first repository, "ORG/NAME";
  set up failed or stopped, why. Opened, it holds the rest of the row's
  parts below and its actions. Every row starts folded.
- **"Open Forgejo ↗"** on a connected Forgejo that signs its people in
  through temper (`../../protocol/openid-provider.md`): a link to the
  forge's `ROOT/user/oauth2/temper`, `ROOT` its browser origin as
  `../../protocol/openid-provider.md`, 1 defines it, in a new tab,
  and under the actions the line "Open Forgejo signs you in as LOGIN,
  through temper: Forgejo’s sign-in is temper’s.", LOGIN the member's
  login. Forgejo starts its sign-in with the temper source, temper
  answers at once, and Forgejo opens signed in.
- **A box's Forgejo that can't sign in through temper:** on a row whose page facts
  say so (`../../protocol/openid-provider.md`, 9), the line "This Forgejo asks for its
  password: temper couldn't connect its sign-in. box.log says why." under the
  row's actions. Nothing else changes, and the set-up is done.
- **A repository's link:** in the browser, wherever a project's repository
  is named with its forge known (the switcher's row, `../view-navigation.md`, 1; the
  body's "Repositories" row below; the Forge line and Settings' Repositories,
  `../pages/README.md`), OWNER/NAME is the link "OWNER/NAME ↗", in a new tab,
  underlined on hover, in the look of the words it replaces. Its address:
  - a Forgejo whose page facts offer sign-in (`../../protocol/openid-provider.md`, 3):
    `ROOT/user/oauth2/temper?redirect_to=%2FOWNER%2FNAME`, ROOT as for "Open
    Forgejo ↗", so that one click signs the member in and lands on the
    repository's page;
  - any other forge, GitHub among them, with a `web` address: `WEB/OWNER/NAME`;
  - otherwise none, and the words stay plain, as the playground's do. An empty
    OWNER or NAME has no address either, on any forge.

  OWNER and NAME are each percent-encoded as a path's segment, all but ASCII
  letters, digits, `-`, `.`, `_` and `~`, and in `redirect_to` that path is
  encoded again as a query's value, so a name can't end the path, the query or
  the address. A whole segment `.` or `..` has its dots encoded twice
  (`%252E`), since browsers normalize a once-encoded dot segment too.
  An address that isn't a safe `http` or `https` URL is none.
  The terminal draws no link, and its words are unchanged. A folded row's
  summary stays words: a disclosure's description is text in its `<summary>`,
  and a link there is the trap a link in a button is; its link is in the
  row's body, under "Repositories".
- **A forge's name:** "Forgejo on this machine" for an owned Forgejo,
  "GitHub" for GitHub, "Forgejo at HOST" for another Forgejo, HOST its
  `web` address's host, and "Playground forge" for the playground's
  (`../pages/welcome.md`, 2). PROJECT is a project as the switcher names it
  (`../view.md`, 1).
- **What the row carries** (`records.md`): its number, kind,
  mode, connection, offer, its first repository, its `web` address, the
  repositories adopted on it, each with its project, on GitHub its
  installations, and its last connect's end. Opened, a row holds its
  Account line, connected ("LOGIN · user ID"), and its actions: "Set up
  again" on set up failed, "Open its set-up ›",
  a link to `ConnectForge` at it, while it sets up or connects, an
  owned forge's restart words when stopped, and, connected, "Adopt a
  repository", a link to `Adopt { project }` for the project the person
  is in. On GitHub it holds, connected, the App's line (below, "On
  GitHub, temper's App"), each installation by its account's login, or
  "installation ID" while the forge has not heard its account, with
  "suspended" and a stray's words (below) where they hold,
  never "all repositories" or "selected repositories" nor a link to
  the installation's page, which wait on the forge hearing an
  installation's selection (skein's installation events carry none);
  and "Install on another account", which sends `Install`, its way in a
  new tab (*GitHub's way, in a new tab*, below), once an installation is
  accepted, or "Install on GitHub" before (the actions' table, below). Every row
  holds its last connect's line where it has one (the row's table,
  below), and, last, "Remove" (Removing a forge, below).
- **Removing a forge** (`../../domain/forge/adoption-connection.md`, 1): "Remove", on every
  row of the person's, asks first, in the row: "Remove this forge?",
  then what goes and what stays on the forge, by its kind and mode: an
  owned Forgejo, "Its Forgejo, its directory and its repositories' work
  here go."; a managed or connected Forgejo, "Its bot and hooks stay on
  your Forgejo."; GitHub, "The GitHub App stays yours: delete it in its
  settings.", with "the App's settings", an external link to
  `WEB/settings/apps`, WEB the forge's `web` address, github.com's
  without one; then "Remove", which sends `RemoveForge { forge }`
  (`records.md` and `../engine.md`), disabled while it is out.
  Answered removed, the forge leaves the rows; refused, the refusal
  shows under it in `ForgeRefusal`'s words (Details, below). The
  playground's forge has no Remove: its reset removes it
  (`family.md`, 7).
- **The playground's forge,** the playground's fake, follows under the
  level-2 heading "The playground", folded, while the playground is
  read (`family.md`, 7): "Playground forge", its summary OWNER/NAME,
  the playground's repository; opened, its Account line. Once the
  playground is reset, it shows nowhere.
- **Owned setup versions** come from the forge row's `owned` pins.
  Details names the Forgejo version in what temper box downloads;
  without `owned`, it keeps "Forgejo and its runner". Each step names
  no version, even with `owned.runner`: "Fetching Forgejo" until
  fetched, then "Fetched Forgejo". These are shared by Connect a forge
  and the welcome. Preparing names no future login.
## 2. Connecting a forge

Moved to [`deployment-connect.md`](deployment-connect.md), section 1.

## 3. A forge’s row
A row for each forge, by its number, its kind and its `web` address,
says:

- **Its connection,** one of:

  | State | The row says |
  |---|---|
  | not connected | "Not connected", with Connect; where its records say why, "Not connected: its records are unreadable" or "Not connected: its records were made for KIND at ADDRESS", and connecting again overwrites them (`../../protocol/forge/README.md`, at start) |
  | not set up | an owned forge not prepared yet (`../../protocol/setup/owned.md`, 5): under the box, "Not set up", with "Set up a Forgejo here"; without it, "Not set up: start temper with temper box", with no action, since its group is the box's (`../../protocol/setup/box.md`, 4) |
  | setting up | the box runs its steps, asked: "Setting up: fetching", "Setting up: preparing" or "Setting up: starting", by the box's last word of the step it runs, else "Setting up" (`../../protocol/setup/box.md`, 1); while it fetches, how far the row says, "Setting up: fetching Forgejo · 42%", then "Setting up: fetching its runner · 80%", the percentage `done × 100 / total` rounded down, or with no size its bytes, "Setting up: fetching Forgejo · 41 MB", `done / 1000000` rounded down, each binary's own (`../../protocol/setup/owned.md`, 6); then "Connecting", as its handover is taken and reconciled (`../../protocol/setup/reconcile.md`, 1) |
  | set up failed | "Set up failed: STEP", naming the box's step, or, after the box's `started`, the engine's step that stopped (taking the handover, reconciling, removing the handover file; `../../protocol/setup/owned.md`, 5), with `temper box`'s output for the rest until its `failed` line carries a reason; with "Set up again"; on a managed Forgejo, the plan's step and why its start stopped at (`../../protocol/setup/reconcile.md`, 1), with Connect in its place; the engine and every other forge run on |
  | connecting | "Connecting": a run holds the forge, and what it waits for: a managed or owned Forgejo's reconcile, the forge's answers to a check, or GitHub's conversion of its App; on GitHub, also connected with no accepted installation, "Connecting: installed nowhere yet" with none, or "Connecting: no installation accepted yet" with strays alone, with "Install on GitHub". The person's browser on GitHub is no run, and shows nowhere |
  | connected | "Connected as LOGIN", the user the records hold: the bot's on Forgejo, the App's on GitHub; its id on an Account line |
  | stopped | "Stopped: WHY", the component's reason in words (its key or token refused, a user other than its own, an access it finds invalid, a refusal that asking again would not mend), with Replace where offered; an owned forge says "Restart temper box: its preparation mends it"; the engine and every other forge run on |

  A connect that ends otherwise is said on the row until the next one,
  its last connect's line, "The last connect stopped: WHY": a run of a
  `Connect` or an `Install` that stopped, or, on GitHub, the way in its
  new tab ending without an installation accepted (`served.md`, 1,
  *Each way's end reaches the forge's row*: its link expired, an
  organization member's request, a run of the routes' that stopped,
  temper busy), WHY the setup host's reason or the routing's
  in words, at most 256 bytes, kept by the forge domain in memory per
  forge until that forge's next connect or install starts or it
  connects, and dropped by a restart. A refusal before any run (the
  person's role, the forge's kind or mode, "busy": the forge's one
  writing run is taken, or `setup.runs` runs are running) is its
  request's outcome, shown by the page or the tool that sent it, and
  never on the row. The reasons a run stops with include
  "the pasted value changed", "the pasted values are not a Forgejo's
  tokens", a connected Forgejo's `Check`'s ("the git token answers as
  another user than the API token", "the tokens answer as another user
  than LOGIN, this forge's: connect that user as a new forge", LOGIN
  the record's, "the API token is a site admin's: give the tokens of a
  user that is not one", "Forgejo refused the API token" or "Forgejo
  refused the git token"; `../../protocol/setup/README.md`, section 4), "LOGIN is
  not temper's bot: add `temper:bot` to its full name on Forgejo"
  (`repositories.md`, 4), the forge unreachable, a `Manifest` or an `Install` that
  stopped, its failure said here and never in its answer
  (`../engine.md`), or "interrupted; paste again" after a
  restart, once the connect's own record is kept; until then a restart
  drops a running connect unsaid, its carried value erased at start
  (`../../protocol/credentials.md`, 2.2), and the forge reads as its
  records say. A GitHub App lacking what temper needs connects all the same
  (the setup's host's `Lacking` is done): the row says it beside
  "Connected as LOGIN", each permission and event named (`repositories.md`, 4), until the
  forge's next check of the App ends: a paste's (a connect or a Replace)
  or the reconcile after each connect, the one at each start among them
  (`../../protocol/setup/github.md`, 5), which clears the line or names what
  the App still lacks. The line is kept in memory alone: a restart drops
  it, and the start's reconcile names it again while the App still lacks
  it. A forge other than a fake, connected while a worker's `git` is
  null (`../../protocol/configuration/worker.md`, section 1), says beside
  "Connected as LOGIN": "No git on this machine: install git 2.41 or
  later and set workers[].git, then restart"; its repositories' commands
  fail on that worker, naming the forge's host, until it has one
  (`../../protocol/git.md`, section 3).
- **Its first repository,** when its set up asked for one
  (`../../protocol/setup/owned-repositories.md`, 1): "Making ORG/NAME" from the ask until
  it is made; once made and connected, the body's "Repositories" row
  says "ORG/NAME · project PROJECT", resolving PROJECT to its name,
  as the summary does, ORG/NAME the repository's link (`deployment.md`, *A repository's
  link*). There is no separate Made paragraph. Or "Making
  ORG/NAME failed: STEP", the step one of "checking the admin's
  token", "making the organization", "making the repository" (its
  bot's access with it) or "adding the starter workflow", beside "Set
  up failed". Made and failed are the engine's memory alone; the
  intent is the forge's record's, so after a restart the line says
  "Making ORG/NAME" until the repository is made again
  (`../../protocol/setup/owned-repositories.md`, 1). The welcome’s Ready step and
  `temper ctl forge setup` adopt the made repository into its project
  (`../ctl/areas.md`).
- **Its mode** (`../../protocol/setup/README.md`, the modes): owned, where the
  forge's setup entry, kept on its record, owns it
  (`../../protocol/forge/engine.md`, 1); managed, where temper keeps a
  site admin's token for it; connected otherwise.
- **Its Access line,** on Forgejo: "Site admin, with temper's own token"
  for owned; "Site admin, with a token you gave" for managed; "Its bot's
  tokens, which you gave" for connected. The setup host alone reads an
  admin's token, never the forge component.
- **Owned's Runs line:** Forgejo and its runner on this machine, their
  versions, and CI jobs on this machine as the person. Before set up,
  the card says CI jobs run here unisolated, Forgejo stops with `temper
  box`, and it is kept in temper's data directory. Once set up
  (connecting, connected or stopped), its First password line names the
  `person_password` file's path, never the password: "Your first
  password is in PATH. Change it on Forgejo when you like." It stays, since temper keeps no record of the first sign-in
  that uses it (`../../protocol/setup/owned.md`, 5).
- **On GitHub, temper's App:** a line labelled "temper’s App", "SLUG ·
  INSTALLATIONS", INSTALLATIONS each installation's words joined by
  ", ", or "installed nowhere yet" with none: its slug, and its installations, each
  with its account's login, "all repositories" or "selected
  repositories" (linked to the installation's page, `repositories.md`, 4), "suspended"
  while GitHub suspends it, and a stray, an installation never
  accepted, "not accepted · kept" (`../../protocol/setup/github.md`, 4). The tools accept a stray (`../ctl/areas.md`,
  `forge accept`); the page offers no act for it yet. When
  connected, it shows the setup URL the person must set on GitHub:
  `http://<listener>/forges/<n>/github/setup`, with redirect on update,
  no callback URL and no user's authorization at install
  (`../../protocol/setup/github.md`, 3).
- **A job's log on an origin temper does not read:** "Job-log origin
  refused: ORIGIN", ORIGIN its scheme, host and port, in the last
  connect's line's place while no connect's stop holds it: the forge's
  latest `Fact::Origin` (`../../protocol/forge/README.md`, 2.6), kept per forge
  in memory, at most `forge_component.log_origins` of them, and
  forgotten when the forge next connects. The line names no fix, since
  no setting adds an origin: a job log is read only from the origins
  its forge's kind names, GitHub's results hosts, none for a Forgejo or
  a GitHub the machine names, and a check's failure then shows without
  its log. The same origin is in the engine's log, once a start.
- **Catching up,** after a restart, while the row's `catching_up` is
  there (`records.md`): a line under its connection's,
  "Catching up: 3 fresh reads, 2 uncertain writes", its words
  `../../domain/forge/catching-up.md`, 5's, whatever its connection
  says; gone once the forge has caught up.
- **Its load:** its request budget and what uses it (fresh reads,
  writes, keeping up, the slow pass; `../../domain/forge/keeping-up.md`, 4); a
  pause for its rate, and until when; its outbox, the effects waiting
  and those uncertain, being looked for; and when each repository's
  webhooks were last heard.
- **A full table,** the forge connector's and so every forge's, comes
  first, above the forges' rows: while the forges' document carries one
  full past its report bound (`../../protocol/service-process.md`, 1.3), a line
  for it, "The forge's outbox is full for AGE: WORDS" or "The forge's
  read admission is full for AGE: WORDS", WORDS the document's, the
  service's line's after `AGE: `, and AGE from its `since` by the
  client's clock in 6.3's words, kept current as the page is, with no
  act. Gone with its table's free place.

Its actions, each a request of the forge's family (`../engine.md`
), offered by kind, mode, setup and state. The engine domain decides
every offer; the client reads it (`../../domain/forge/adoption-connection.md`, 1):

| Action | Offered | What it does |
|---|---|---|
| Connect | GitHub, to make, or kept, not connected or stopped | "Connect GitHub", with no `back`: the answer is a link to the manifest's page (`served.md`, 1), the whole address on the clients' listener, `http://LISTENER/forges/N/github/manifest?state=S`, the terminal's as the browser's. The terminal prints it; the browser goes to its path and query on the page's own origin in a new tab (*GitHub's way, in a new tab*, above); the page posts temper's manifest to GitHub, the person creates the App under their own account and installs it on an account GitHub offers, and the setup route ends the tab on its own page, while temper's page waits and moves on by itself. Or "Connect an App of your own": its id, and its private key pasted as GitHub gives it; no webhook secret is asked, since temper points the App's hook at `https://SLUG.invalid/` and hears changes from its delivery log |
| Connect | Forgejo of the connected mode, kept, not connected or stopped, or one to make by its addresses | "Connect with its tokens": its bot's API token, its git token and, where the person made hooks to temper's listener, their secret, pasted together; `Check` finds both answer as one user, not a site admin, and on a kept forge as its kept user; only then are they kept, its login with the record, and the forge connected; tokens of another user stop it, the forge as it was (`../../protocol/setup/README.md`, section 4) |
| Connect | Forgejo with `admin` true, not connected or connecting, or one to make by its addresses | "Connect as its admin": a site admin's token pasted, with the first repository, project and runner label (default `docker`); Reconcile makes or finds the marked bot, then Create makes the repository; only after Done is the admin token kept and the forge connected; never `TakeOver` (`../../protocol/setup/reconcile.md`, 5) |
| Replace | GitHub, connected or stopped; a Forgejo of the connected mode, connected or stopped | "Replace", which opens its fields in the row: on GitHub "Private key", the App's new key as GitHub gives it, restored as Connect's is, and "Replace the key"; on a Forgejo "API token", "Git token" and "Webhook secret", and "Replace the tokens"; then `Replace { forge }`, the key or the tokens and the secret given carried as one bundle; they must answer as the forge's user (on GitHub, the same App), and each is kept over the old; answered connecting, the row says Connecting, and a stop leaves it as it was with its last connect's line; refused, the refusal shows under the button in `ForgeRefusal`'s words (`not connected`, "Connect it first."); the terminal shell, with no secret field of several lines, says in the key's field's place "Paste the key on this page in a browser, or run temper ctl forge replace N FILE, FILE the key's file, readable by you alone." (`../ctl/areas.md`) |
| Install the App; Install on another account | GitHub, connected: "Install on GitHub" before any accepted installation (Connecting), "Install on another account" after one, and the row’s "Choose its repositories on GitHub" | a fresh link to temper's App's install page, asked with no `back`, where the browser goes in a new tab (*GitHub's way, in a new tab*, above); the person picks the account and its repositories on GitHub, keeping GitHub's default, all of them, or choosing some, and the setup route ends the tab on its own page, while temper's page waits and moves on by itself |
| New repository | Forgejo owned or managed, connected | the set up's same fields: organization (default `work`), repository (default `hello`) and a project, one of the person's or a new one; the general form of the first repository (`../../protocol/setup/README.md`, section 9) |
| Forget the admin's token | managed | the admin's token erased; the forge is connected, its bot and tokens kept, and adopting a repository grants nothing more: the person makes the bot a collaborator |
| Set up a Forgejo here | an owned Forgejo to make, under the box, or the row's offer offered, not set up (`records.md`) | its organization (default `work`), its repository (default `hello`) and its project, a new one named as the repository (`../../draft/archive/first-run.md`, 2.4), on `ConnectForge` and the welcome alike (`../pages/README.md`); then the box asked to run the forge's steps (`../../protocol/setup/box.md`, 1): its binaries fetched, its preparation, Forgejo and its runner started, then its handover taken and reconciled, the repository made with the starter workflow (`../../protocol/setup/owned-repositories.md`, 1), and, once the row says it made, the client's `MakeProject` and `Adopt` into it; the card names the path of `person_password`, never its password (`../../protocol/setup/owned.md`, 5) |
| Set up again | the row's offer offered, set up failed | the same ask again: what the last one left is kept, a fetched binary hashed and not fetched again |
| Remove | every forge of the person's, the playground's aside | "Remove", confirmed in its row, its words naming what goes and what stays on the forge (Removing a forge, above); then `RemoveForge`, which ends the forge at once and removes it whole (`../../domain/forge/adoption-connection.md`, 1) |

GitHub's Connect, but on a stopped forge or one whose records say why
it is not connected, "Connect with its tokens", "Connect as its admin"
and "Set up a Forgejo here" are the flow's, at its Details, which a
row offering one links to, "Connect ›", `ConnectForge` at its forge;
every other action is a row's, in its opened part.

A pasted value is a field of its own (`../root.md`, 1.6 and `../engine.md`),
emptied as the request leaves and shown nowhere, then or after; the
page shows that it was given, never what. Two refusals are the page's
own, before anything is sent, the fields kept for the person to fix: a
field the action needs left empty is marked invalid, "Required." (a
Replace of a Forgejo's tokens with both empty: "Paste a new API token,
git token or both."); and the bundle of the action's values past 4 KiB,
jig's bound on a carried value (`../ctl/bounds.md`, section 1), shows "The
pasted values are past 4 KiB, the most a request carries." under the
action's button. The link a connect or an install answers is where
the browser's tab goes: the root asks its shell to leave
for it once, from the answer, never from its state, and the page shows it as an external
link meanwhile. The terminal prints it and opens nothing (`../line.md`,
5).

## 4. Open questions

- **A repository link in a folded forge row** (`deployment.md`): the summary stays
  words and the link lives in the body's Repositories. Offering it while
  folded needs a row whose navigation and disclosure are separate controls.
  Default: the body's link, as the `ui-repo-link` cut decided.

- **Mending a stopped managed or owned forge.** Default: Replace is
  offered for what a person pasted (a GitHub App's key, a connected
  Forgejo's tokens); a managed forge is mended by connecting as its
  admin again, reconcile making the bot's tokens anew for the same
  user; an owned one by its preparation and a restart.
