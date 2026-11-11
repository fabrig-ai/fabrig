# The forge's part of the deployment

The forge's part of the deployment's connectors (jig's
`client/domain/pages.md`, section 7), on two pages of temper's
(`../pages/README.md`; with the user, 2026-10-23): `Forges`, which manages
the forges the person has, and `ConnectForge`, a flow that connects one
more. Each is read for a person whose role on project 0 is `Owner`, in
standalone the deployment's owner; any other's read is refused, in
jig's words (`../engine.md`). Their mockups are `forges.html`
and `connect-forge.html`.

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
  is named with its forge known (the switcher's row, `../view.md`, 3; the
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
  and "Install on another account", which sends `Install` with `back`,
  the Forges page's address, once an installation is accepted, or
  "Install on GitHub" before (the actions' table, below). Every row
  holds its last connect's line where it has one (the row's table,
  below), and, last, "Remove" (Removing a forge, below).
- **Removing a forge** (`../../domain/forge/adoption.md`, 1): "Remove", on every
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
  the first of Each step's five lines names both Forgejo and runner
  versions, "Fetching" until fetched, then "Fetched". Without `owned`,
  Details keeps "Forgejo and its runner"; without `owned.runner`,
  that first step keeps its unversioned wording. These are shared by
  Connect a forge and the welcome. Preparing names no future login.
- **Connecting a forge,** `ConnectForge { forge }` (the request
  `connect-flows`): the welcome's cards, panels and words, under Settings'
  path and in its column, with no list of steps; one or two pages, then the
  forge's row on `Forges`. Choose has the heading "Connect a forge", the
  lede "Where your code lives: you tell temper what you want, agents push
  and land code.", the cards for Forgejo on this machine and GitHub, the
  first way not connected the default (a connected one says "✓ Connected ·
  ORG/NAME" and "Open Forgejo ↗"), and under them "Or a server of your own",
  the row "Your Forgejo" with "Connect your Forgejo". A forge's Details is
  one panel, with a quiet "Other forges", to Choose, beside its main button.
  "Your Forgejo" shows one way at a time, "As its admin" or "With its bot’s
  tokens", the rare fields folded ("Its git or web address, if they differ",
  "Its runner label: docker", "A webhook secret, if you made hooks to
  temper’s listener"); before its restart, it is the panel "A Forgejo
  elsewhere", numbered steps each with its Copy. Forgejo on this machine's
  way is the welcome's sheet. **Setting up lives in the row:** from a forge's
  connect on, for a managed Forgejo and for GitHub's return, the address is
  `Forges`, and the forge's row is open with its live steps (below), with
  `Forges`' line saying what was added once it is connected, as `Accounts'`
  does. `forge` is none at Choose,
  or the choice: a kind to make, GitHub, Forgejo on this machine or
  Your Forgejo, or a kept forge's number (`../web/README.md`, 2.3). The step
  shown is the choice's: none, Choose; a kind to make, or a forge whose
  row offers its connect or set up, Details (on GitHub, an App of the
  person's own); a forge
  setting up, set up failed, connecting or connected, the row (above),
  but a GitHub forge with no accepted installation, Connecting (below). A number that names no row, or
  the playground's forge, is Choose; at Choose, a forge of the
  person's cut short is shown in the choices' place, "FORGE · not
  finished", with "Start over" and, on GitHub, "Go on", as the
  welcome's Forge step shows it (`../pages/README.md`, The welcome); once
  removed, "Start over" returns to `Forges`. The page's level-1 heading is
  “Connect a forge”, under its path (`../pages/README.md`, *The flows' paths*).
  The choices sit directly below its lede; each way is one panel under
  its level-2 heading. There is no Steps list or outer Choose card.
  Leaving the flow leaves what runs running; the Forges entry's dot
  then says it (Elsewhere, below).
- **Choose** uses the welcome’s two cards, Forgejo on this machine
  then GitHub, with “Or a server of your own” and Your Forgejo below.
  Each card is a region named by its level-2 heading, with its words,
  note and act (`../pages/README.md`, The welcome). The first unconnected
  way is primary. Connected cards say “✓ Connected” and offer the
  forge’s external link. The choice makes the forge’s record at admission,
  never before (`../../protocol/forge/engine.md`, 2). While the playground is read,
  Settings offers nothing to add (`family.md`, 7).
  - *Forgejo on this machine:* “A forge of your own: temper starts it on
    this machine and manages it. You can import existing repositories
    into it, from GitHub or elsewhere.” “Run on this machine” opens the
    welcome’s sheet: an empty repository by default, `work` and `hello`
    until written, or copies or nothing (`../pages/README.md`).
  - *GitHub:* “Your repositories on GitHub. temper makes its own App
    there; you choose the account and the repositories it may use.”
    “In this tab, on GitHub · about two minutes”. “Connect GitHub” sends
    `Connect` of a new GitHub forge and takes the tab to GitHub. Nothing
    asks for an organization: the App is the person’s, and installation
    offers every account they may install it on. An App of their own
    keeps its Details at a kept GitHub forge’s address (`../web/README.md`, 2.3),
    with no card linking there.
  - *Your Forgejo:* “Your own Forgejo server. temper connects as its admin,
    or with its bot’s tokens.” “Connect your Forgejo” opens its panel,
    or the numbered configuration instructions before a restart.
- **Details** are the choice's, under its level-2 heading, with “Other
  forges” beside the main act, a quiet link back to the choices; the address of a kind Choose does not
  offer, Forgejo on this machine with no `owned`, is Choose:
  - *GitHub,* an App of the person's own, at the GitHub forge's
    address: "An App of your own", "Its id, and its private key as GitHub
    gives it.", "On GitHub, set the App’s setup URL to temper’s address
    followed by /forges/N/github/setup, and give it the permissions and
    events temper’s help lists." (the client knows no address of its
    own to print), its fields "App id" and "Private key" and its
    button, "Connect the App"; then "A pasted value is sent once and
    shown nowhere, then or after." "App id" is a line of digits, the
    App's id as GitHub shows it; "Private key" is a secret text field,
    one line, where the person pastes the PEM file's text as GitHub
    gives it. A browser draws it as a password input, masked and never
    spellchecked, and strips its line feeds; skein's builder marks only
    a text field secret, and a text area would show the key and offer
    it to the browser's spellcheck. So the client restores the armour
    alone: the text trimmed of spaces, tabs, CRs and LFs at both ends,
    one LF put after `-----BEGIN RSA PRIVATE KEY-----` and one before
    `-----END RSA PRIVATE KEY-----` where none is, nothing else changed
    (the body's line breaks matter to no one: skein's parse ignores
    them); a text that is not that armour goes as it is, for `Paste` to
    refuse. "Connect the App" sends `Connect { forge, how: GitHubApp {
    id } }`, the key its carried value, a bundle of one `Pasted` item
    of that text (`../../protocol/credentials.md`, 2.2), and empties the
    key's field as it leaves (below, a pasted value). The bundle holds
    4 KiB; the setup host takes an RS256 key of at most 1196 DER bytes
    (skein-tls's `RS256_KEY_BYTES`: 2048 bits, the size GitHub gives),
    about 1.7 KB of PEM, and refuses a larger one at `Paste`, in words
    that never repeat the key. A secret text area, its line feeds kept,
    is skein's to offer later (10). The
    page's own refusals, before anything is sent, the fields kept: a
    field empty, "Required."; an id that is not a number from 1 to
    2^64 − 1, "An App's id is a number, as GitHub shows it on the App's
    page."; the bundle past 4 KiB, as below. Answered connecting, the
    flow opens Forges with the row expanded: its live progress while the
    run holds the forge, then its installation act (below); a run that stopped
    leaves the row not connected with its last connect's line (the
    row's table, below) shown under the button, the key asked for
    again. Refused, the refusal shows under the button, the id kept:
    `role`, "Only the deployment's owner connects a forge."; `no such
    forge`, "No forge N is configured."; `not taken`, "Forge N does not
    take an App."; `connected`, "GitHub is connected already."; `busy`,
    "temper is busy with forge N: try again when it ends."; `no key`,
    "The key did not reach temper: paste it again."; unknown past the
    retention, "It is not known whether the App was connected: see
    Forges.". Its install is Connecting's, as for temper's App.
    The terminal shell has no secret field of several lines: its
    Details for an App of your own say, in place of the fields and the
    button, "Paste the key on this page in a browser, or run temper
    ctl forge connect N github-app APP FILE, FILE the key's file,
    readable by you alone." (`../ctl/areas.md`);
  - *Forgejo on this machine,* under the box, is the welcome’s sheet
    (`../pages/README.md`): empty, copies or nothing, then “Run on this
    machine”. It says that CI jobs run on this machine as the person,
    without isolation, and that Forgejo stops with temper box while its
    repositories stay in temper’s data directory. Run sends `SetUp` of a
    new or kept owned Forgejo (`records.md`). Its names default to `work` and `hello`. Empty names
    say “Required.”; other refusals stay under the act, with fields
    kept for repair. Accepted, Settings opens Forges with its row expanded;
    setup adopts nothing. The made repository waits for adoption into a
    project; the welcome’s Ready handles its own adoption. “Set up again”
    sends the previous fields. Outside temper box no owned setup is
    offered (`../../protocol/setup/box.md`, 4);
  - *Your Forgejo,* a forge to make: its addresses, "Its address", its
    API's host, "Its git base", the base of its remotes, and "Its web
    address”, its pages, optional. The git and web overrides are folded.
    “How temper connects” chooses one form at a time: “As its admin”,
    a managed Forgejo’s, or “With its bot’s tokens”, a connected Forgejo’s
    (both below). "Connect as its admin" sends `Connect` of a new
    managed forge, `managed` naming those addresses, and "Connect with
    its tokens" of a new connected one, `connected` naming them
    (`records.md`), which the request's decision makes as its record
    (`../../protocol/forge/engine.md`, 2); with no address, nothing is sent.
  - *A connected Forgejo,* kept or to make: "Connect with its bot’s
    tokens", then "Make a Forgejo user for temper, not a site admin, with
    write access to each repository temper will work on, and paste two
    of its tokens: one with the scopes read:user, write:issue and
    write:repository, and one for git with write:repository and
    read:user. temper acts as that user." Its fields, "API token" and
    "Git token", and "Webhook secret", with the hint "Only if you made
    hooks to temper’s listener: their secret.", each a password field
    passed straight to a `Secret`, outside text actions, as the admin's
    token is. The webhook secret is folded under “A webhook secret, if
    you made hooks to temper’s listener”. The button "Connect with its tokens" sends `Connect {
    forge, how: ForgejoTokens }`, `forge` the kept forge's number or the
    new one's addresses, the tokens and the secret carried as one bundle
    of `ApiToken`, `GitToken` and `WebhookSecret` when given
    (`../../protocol/credentials.md`, 2.2), and empties the three fields as
    it leaves; a stop empties them again. Before sending: an empty
    token, "Required."; the bundle past its bound, "The pasted values
    are past 4 KiB, the most a request carries." Refused, the refusal
    shows under the button in `ForgeRefusal`'s words (Details, above;
    `connected`, "Forge N is connected already: replace its tokens on
    Forges."). A kept connected Forgejo, not connected or stopped, shows
    this form alone, under its name.
  - *A managed Forgejo,* kept or to make: "Connect as its admin", then "Paste a
    site admin’s token with the scopes write:admin, read:user,
    write:organization and write:repository. temper keeps it, and uses it
    only to make its bot and your first repository, and to give the bot
    access to it. To stop temper using it, delete the token on Forgejo."
    "The site admin’s token" is a password field, passed straight to a
    `Secret`, outside text actions. "Your first repository" has
    "Organization" (`work`), "Repository" (`hello`) and "Runner label"
    (`docker`), and no project's field. Its hint is "Its CI runs on your forge’s runners with this
    label. Forgejo’s runner registers as docker unless told otherwise."
    The button "Connect as its admin" sends `Connect { forge, how:
    ForgejoAdmin { first: Some(FirstAsk) } }`, `forge` the kept forge's
    number or the new one's addresses, with the three fields and
    the token carried as a bundle of one `AdminToken` item, at most 4
    KiB in all. No RSA armour restoration applies. Sending
    empties the token field; a stop empties it again and keeps the other
    fields. Before sending: an empty token or name, "Required."; a bundle
    past its bound, "The pasted values are past 4 KiB, the most a request
    carries."; a label outside `[A-Za-z0-9._-]{1,64}`, "Letters, digits,
    dots, dashes and underscores, at most 64." Sign-out erases the draft.
- **Connecting, as a Forgejo's admin,** says "Connecting forge F as its
  admin." and "Making its bot, then ORG/NAME." The welcome keeps Forge
  highlighted. A stop returns to Details with its token empty and the
  other fields kept. `ended_stop`, the row's optional `AdminStop`, selects
  the words; a refused stop carries its status and `SetupStage`. `ended` is a
  diagnostic, never parsed to decide the stop:
  - `NotAdmin`: "The token you gave is not a site admin’s. Paste one a
    site admin made.";
  - refused 401: "Forgejo refused the token. Check that it was copied
    whole, and that it hasn’t been deleted.";
  - refused 403: "The token lacks a scope: STAGE was refused. Make one
    with write:admin, read:user, write:organization and write:repository."
    STAGE names the actual operation, for example "making the bot";
  - `Unmarked`: "A user temper didn’t make holds the bot’s login, without
    temper’s mark: add temper:bot to its full name on Forgejo, or change
    the bot’s login in the configuration.";
  - another stop: the `ended` line.
  A failed `first` names ORG/NAME and its failed step (admin, organization,
  repository or starter workflow), with the diagnostic and "Paste the
  token again to finish it." Every stop says "temper kept nothing of the
  token." No "Set up again" action is offered.
- **Connecting, with its bot's tokens,** says "Connecting forge F with
  its bot’s tokens." and "Checking that both answer as one user, not a
  site admin." A stop returns to Details with its fields empty and the
  row's last connect's line under the button (the row's table, below).
  Connected, Forges holds the confirmation and its expanded row, with no
  first repository: “Connected as LOGIN.”, LOGIN the user its tokens answer
  as. There is no Ready page.
- **Refused at admission,** the refusal shows above the fields, the token
  asked for again. `first is bot`: "That organization has the name of
  temper's bot on this forge, and on Forgejo users and organizations
  share names: name another organization.", and the organization's field
  is marked invalid.
- **Connected, on that Forgejo,** the open row says "Connected as LOGIN (id ID), managed."
  and "Made ORG/NAME. Its CI runs on your forge’s
  runners labelled LABEL; with none, a change’s checks wait." LABEL is
  `first.label`, including after reload. Nothing is adopted: “Adopt a
  repository”, opening the sheet, and, with `row.web`, “Open ORG/NAME on Forgejo ↗”
  links to the repository.
- **Setting up, in the row,** shows the set up in Forges’ expanded
  disclosure, its first repository’s line, “ORG/NAME”, then its compact
  live list, each line with
  its state in words, "done", "under way" or none for one to come:
  "Fetching Forgejo and its runner" ("Fetched Forgejo and its runner"
  once done), and while the row says how far, "· 42%" or "· 41 MB"
  after it, as the row's words do (the table below), then
  "Preparing: its database, temper’s admin, your account",
  "Starting Forgejo and its runner", "Connecting as temper", and
  "Making ORG/NAME". The box's step
  names the one under way (fetch, prepare or run); none named yet, no
  individual step is marked. Connecting is the fourth, making the
  fifth, done once the first repository is made, unadopted. Before the
  target is known, the fifth reads "Making the first repository". A set up
  asked with copies or nothing has four steps, the row's `starts_with`
  saying so (`../../protocol/setup/owned.md`, 7).
  GitHub rows never list Forgejo's steps. The first and the Details
  name the versions the row's `owned` pins, "Fetching Forgejo 17.0.1
  and its runner 13.2.1"; the person's login stays off until the row
  carries it. Then "You may
  leave this page: the steps go on." A failed setup keeps all five steps: those before the failed
  step are done, that step is marked failed (✕), and those after it
  are to come. After the list, "STEP failed: WHY", STEP one of
  "Fetching", "Preparing", "Starting", "Connecting" or "Making
  ORG/NAME". The first four read the row's `at`; the fifth reads
  `first.state`'s failed creation step. With no step named, the list
  has nothing marked and the line is "Set up failed: WHY". The welcome
  keeps the same list and failure line. Then "Set up again", which
  sends `SetUp` with the same fields. While a `SetUp` is pending its button is disabled;
  refused, its refusal shows under it in the set up's words
  (`records.md`).
- **Connecting, on GitHub,** is the App made and installed nowhere:
  a GitHub forge connected with no accepted installation, which the
  row also says as connecting (below), since it reaches no repository
  yet. The level-2 heading "Install temper’s App", then "SLUG is made
  on GitHub, under your account, and installed nowhere yet. Install
  it, and choose the repositories temper may use." and "Install on
  GitHub", which sends `Install` and takes the tab to GitHub's install
  page, as Connect GitHub does; with `back`, Forges’ address,
  so that the setup route sends the tab back to Forges, where the
  expanded row shows the accepted installation. The person meets it only by coming
  back to temper before installing: GitHub's redirect takes the tab
  from the App's form straight to that page. An App of the person's
  own meets it always, since its connect has no redirect. The row's
  `app` says which (`records.md`): `Manifest`, the words above;
  otherwise, an App pasted or one connected before a restart, since
  the two end in the same records (`../../protocol/credentials.md`, 2.1),
  "SLUG is connected to temper, and installed nowhere yet. Install it,
  and choose the repositories temper may use.". SLUG is the
  connected login without its `[bot]` (the App's bot user's login is
  its slug and `[bot]`). The link `Install` answers is where the tab
  goes, as `deployment.md`'s last paragraph says; refused, its refusal shows under
  the button in `ForgeRefusal`'s words (Details, above, and `not
  connected`, "Connect the App first."). A connect whose App is
  not made yet shows nowhere: its tab is GitHub's, no run holds the
  forge, and Connect GitHub again draws a fresh state, the first
  ending unused (`served.md`, 1).
- **Completion** lands on Forges with a confirmation line and the newly
  connected row open, adopting nothing (`../../protocol/setup/owned.md`, 7).
  The row holds “Connected as LOGIN.”, the made first repository where
  supplied, its external link and “Adopt a repository”, opening the sheet
  for the current project. No project means no adoption link. The made
  repository waits unadopted; `temper ctl forge setup` adopts it in its
  own run (`../ctl/areas.md`). GitHub’s row holds its App and accepted
  installations; it makes no first repository or project. Its reachable
  repositories are the sheet’s picker (`family.md`, 4).
- **Elsewhere:** the banner's navigation's "Forges", Deployment's tab
  "Forges" and the rail's "Deployment"
  carry a dot, amber while a forge sets up or connects, red
  while one needs the person: set up failed, stopped, or not connected
  with a reason (`../view.md`, 3). The dot's words, the entry's state
  in words: "a forge is setting up" or "a forge needs you". The
  playground's forge is none of them.

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
  `Connect` or an `Install` that stopped, WHY the setup host's reason
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
  null (`../../protocol/configuration/README.md`, section 3), says beside
  "Connected as LOGIN": "No git on this machine: install git 2.41 or
  later and set workers[].git, then restart"; its repositories' commands
  fail on that worker, naming the forge's host, until it has one
  (`../../protocol/git.md`, section 3).
- **Its first repository,** when its set up asked for one
  (`../../protocol/setup/owned.md`, 7): "Making ORG/NAME" from the ask until
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
  (`../../protocol/setup/owned.md`, 7). The welcome’s Ready step and
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
- **Its load:** its request budget and what uses it (fresh reads,
  writes, keeping up, the slow pass; `../../domain/forge/keeping-up.md`, 4); a
  pause for its rate, and until when; its outbox, the effects waiting
  and those uncertain, being looked for; and when each repository's
  webhooks were last heard.

Its actions, each a request of the forge's family (`../engine.md`
), offered by kind, mode, setup and state. The engine domain decides
every offer; the client reads it (`../../domain/forge/adoption.md`, 1):

| Action | Offered | What it does |
|---|---|---|
| Connect | GitHub, to make, or kept, not connected or stopped | "Connect GitHub", with `back`, the page's address to come back to: the answer is a link to the manifest's page (`served.md`, 1), a path on the page's own origin, `/forges/N/github/manifest?state=S`; without `back`, the terminal's link is the whole address on the clients' listener. The browser goes to it in the page's own tab; the page posts temper's manifest to GitHub, the person creates the App under their own account and installs it on an account GitHub offers, and the setup route sends the tab back to `back`. Or "Connect an App of your own": its id, and its private key pasted as GitHub gives it; no webhook secret is asked, since temper points the App's hook at `https://SLUG.invalid/` and hears changes from its delivery log |
| Connect | Forgejo of the connected mode, kept, not connected or stopped, or one to make by its addresses | "Connect with its tokens": its bot's API token, its git token and, where the person made hooks to temper's listener, their secret, pasted together; `Check` finds both answer as one user, not a site admin, and on a kept forge as its kept user; only then are they kept, its login with the record, and the forge connected; tokens of another user stop it, the forge as it was (`../../protocol/setup/README.md`, section 4) |
| Connect | Forgejo with `admin` true, not connected or connecting, or one to make by its addresses | "Connect as its admin": a site admin's token pasted, with the first repository, project and runner label (default `docker`); Reconcile makes or finds the marked bot, then Create makes the repository; only after Done is the admin token kept and the forge connected; never `TakeOver` (`../../protocol/setup/reconcile.md`, 5) |
| Replace | GitHub, connected or stopped; a Forgejo of the connected mode, connected or stopped | "Replace", which opens its fields in the row: on GitHub "Private key", the App's new key as GitHub gives it, restored as Connect's is, and "Replace the key"; on a Forgejo "API token", "Git token" and "Webhook secret", and "Replace the tokens"; then `Replace { forge }`, the key or the tokens and the secret given carried as one bundle; they must answer as the forge's user (on GitHub, the same App), and each is kept over the old; answered connecting, the row says Connecting, and a stop leaves it as it was with its last connect's line; refused, the refusal shows under the button in `ForgeRefusal`'s words (`not connected`, "Connect it first."); the terminal shell, with no secret field of several lines, says in the key's field's place "Paste the key on this page in a browser, or run temper ctl forge replace N FILE, FILE the key's file, readable by you alone." (`../ctl/areas.md`) |
| Install the App; Install on another account | GitHub, connected: "Install on GitHub" before any accepted installation (Connecting), "Install on another account" after one, and the row’s "Choose its repositories on GitHub" | a fresh link to temper's App's install page, with `back`, where the browser goes in the page's own tab; the person picks the account and its repositories on GitHub, keeping GitHub's default, all of them, or choosing some, and the setup route sends the tab back to `back` |
| New repository | Forgejo owned or managed, connected | the set up's same fields: organization (default `work`), repository (default `hello`) and a project, one of the person's or a new one; the general form of the first repository (`../../protocol/setup/README.md`, section 9) |
| Forget the admin's token | managed | the admin's token erased; the forge is connected, its bot and tokens kept, and adopting a repository grants nothing more: the person makes the bot a collaborator |
| Set up a Forgejo here | an owned Forgejo to make, under the box, or the row's offer offered, not set up (`records.md`) | its organization (default `work`), its repository (default `hello`) and its project, a new one named as the repository (`../../draft/archive/first-run.md`, 2.4), on `ConnectForge` and the welcome alike (`../pages/README.md`); then the box asked to run the forge's steps (`../../protocol/setup/box.md`, 1): its binaries fetched, its preparation, Forgejo and its runner started, then its handover taken and reconciled, the repository made with the starter workflow (`../../protocol/setup/owned.md`, 7), and, once the row says it made, the client's `MakeProject` and `Adopt` into it; the card names the path of `person_password`, never its password (`../../protocol/setup/owned.md`, 5) |
| Set up again | the row's offer offered, set up failed | the same ask again: what the last one left is kept, a fetched binary hashed and not fetched again |
| Remove | every forge of the person's, the playground's aside | "Remove", confirmed in its row, its words naming what goes and what stays on the forge (Removing a forge, above); then `RemoveForge`, which ends the forge at once and removes it whole (`../../domain/forge/adoption.md`, 1) |

GitHub's Connect, but on a stopped forge or one whose records say why
it is not connected, "Connect with its tokens", "Connect as its admin"
and "Set up a Forgejo here" are the flow's, at its Details, which a
row offering one links to, "Connect ›", `ConnectForge` at its forge;
every other action is a row's, in its opened part.

A pasted value is a field of its own (`../README.md`, 3.6 and `../engine.md`),
emptied as the request leaves and shown nowhere, then or after; the
page shows that it was given, never what. Two refusals are the page's
own, before anything is sent, the fields kept for the person to fix: a
field the action needs left empty is marked invalid, "Required." (a
Replace of a Forgejo's tokens with both empty: "Paste a new API token,
git token or both."); and the bundle of the action's values past 4 KiB,
jig's bound on a carried value (`../ctl/README.md`, section 9), shows "The
pasted values are past 4 KiB, the most a request carries." under the
action's button. The link a connect or an install answers is where
the browser's tab goes: the root asks its shell to leave
for it once, from the answer, never from its state, and the page shows it as an external
link meanwhile. The terminal prints it and opens nothing (`../line.md`,
5).

## 1. Open questions

- **A repository link in a folded forge row** (`deployment.md`): the summary stays
  words and the link lives in the body's Repositories. Offering it while
  folded needs a row whose navigation and disclosure are separate controls.
  Default: the body's link, as the `ui-repo-link` cut decided.

- **A secret text area for the pasted key** (`deployment.md`). Default: the key is
  a masked one-line field, its armour's line feeds restored by the
  client; a text area skein can mark secret (spellcheck and
  autocomplete off, its text masked) would keep the PEM's lines as
  GitHub gives them and retire the restore. A change to skein's
  builder and page, not temper's.

- **An App of one's own, offered again** (`deployment.md`). Default: GitHub's card
  offers Connect GitHub alone, and the own App's Details are reached by
  their address; a link returns to the card once v0.1.0's successor
  supports the own App officially.

- **A connect's stop told from the one before it** (`deployment.md`). Default: the
  stop that empties a connected Forgejo's tokens again is their sent
  connect's own, its forge's row stopped where the connect last saw it
  not stopped, or stopped with other `ended` words (`records.md`);
  that stop, the row connected or an answer that starts no connect
  ends what the connect watches, and another forge's row empties
  nothing. A second connect whose stop has the first's connection and
  words, with no row delivered between, is not told from it and
  empties nothing more, its fields emptied at its send and its answer;
  a request's id on the row would tell them apart, a change to the
  forges records.

- **A browser on another machine** for GitHub's connect. Default: the
  link names the clients' listener's fixed address, and the person
  opens it where that address reaches (in standalone, the engine's
  machine) or forwards the port themselves; temper makes no tunnel.

- **Mending a stopped managed or owned forge.** Default: Replace is
  offered for what a person pasted (a GitHub App's key, a connected
  Forgejo's tokens); a managed forge is mended by connecting as its
  admin again, reconcile making the bot's tokens anew for the same
  user; an owned one by its preparation and a restart.

- **A connect's end told:** the forges' page reads how a connect ended
  on the deployment's part. Default: that read, watched while the page
  is open; no notice in the inbox.
