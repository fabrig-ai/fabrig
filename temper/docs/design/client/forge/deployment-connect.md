# Connecting a forge

This document holds connecting a forge, split from [`deployment.md`](deployment.md).

## 1. Connecting a forge
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
  - *Forgejo on this machine:* headed “Local”, “A forge of your own:
    temper starts Forgejo on this machine and manages it. You can import
    existing repositories into it, from GitHub or elsewhere.” (the look
    request `local-forge`). “Run on this machine” opens the welcome’s
    sheet, “Local forge”: the sample by default, with no fields, or
    copies or nothing (`../pages/welcome.md`, 1, *Forge*).
  - *GitHub:* “Your repositories on GitHub. temper makes its own App
    there; you choose the account and the repositories it may use.”
    “In a new tab, on GitHub · about two minutes” (the look request
    `github-new-tab`). “Connect GitHub” sends `Connect` of a new GitHub
    forge, its way in a new tab (*GitHub's way, in a new tab*, below). Nothing
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
    (`../pages/README.md`): the sample, copies or nothing, then “Run on this
    machine”. It says that CI jobs run on this machine as the person,
    without isolation, and that Forgejo stops with temper box while its
    repositories stay in temper’s data directory. Run sends `SetUp` of a
    new or kept owned Forgejo (`records.md`). The sample is `work/hello`,
    its names not shown and never written (`../../protocol/setup/owned-repositories.md`,
    1); refusals stay under the act, with the choice and the copies'
    addresses kept for repair. Accepted, Settings opens Forges with its row expanded;
    setup adopts nothing. The made repository waits for adoption into a
    project; the welcome’s Ready handles its own adoption. “Set up again”
    sends the previous fields, the sample again where the row says it was
    one. Outside temper box no owned setup is
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
  "Fetching Forgejo" ("Fetched Forgejo"
  once done), and while the row says how far, "· 42%" or "· 41 MB"
  after it, as the row's words do (the table below), then
  "Preparing Forgejo",
  "Starting Forgejo", "Connecting as temper", and
  "Making ORG/NAME". Each names Forgejo alone, the list's verbs saying
  what happens, so that each stays on one line in the welcome's pane
  (`../pages/welcome.md`, 1; the look request `forge-steps`, the user's
  word of 2026-11-15). The box's step
  names the one under way (fetch, prepare or run); none named yet, no
  individual step is marked. Connecting is the fourth, making the
  fifth, done once the first repository is made, unadopted. Before the
  target is known, the fifth reads "Making the first repository". A set up
  asked with copies or nothing has four steps, the row's `starts_with`
  saying so (`../../protocol/setup/owned-repositories.md`, 1).
  GitHub rows never list Forgejo's steps. No step names a version: the
  Details alone name one, Forgejo's as the row's `owned` pins it, "temper
  box downloads Forgejo 17.0.1 and its runner, ..."; the person's
  login stays off until the row carries it. Then "You may
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
  GitHub", which sends `Install`, its way in a new tab as Connect
  GitHub's is (below), ending on Forges with the expanded row showing
  the accepted installation. The person meets it only once a way
  ended before installing, or on a page opened afresh: GitHub's redirect
  takes the new tab from the App's form straight to the install page,
  while temper's page waits. Under its words, the row's last connect's
  line, where it has one (the row's table, below), says why the last
  way ended. An App of the person's
  own meets it always, since its connect has no redirect. The row's
  `app` says which (`records.md`): `Manifest`, the words above;
  otherwise, an App pasted or one connected before a restart, since
  the two end in the same records (`../../protocol/credentials.md`, 2.1),
  "SLUG is connected to temper, and installed nowhere yet. Install it,
  and choose the repositories temper may use.". SLUG is the
  connected login without its `[bot]` (the App's bot user's login is
  its slug and `[bot]`). The link `Install` answers is where the new
  tab goes (below); refused, its refusal shows under
  the button in `ForgeRefusal`'s words (Details, above, and `not
  connected`, "Connect the App first."). A connect whose App is
  not made yet shows nowhere but in the waiting note of the page that
  sent it: no run holds the forge, and Connect GitHub again, or "Go
  on", draws a fresh state, the first ending unused and unsaid
  (`served.md`, 1).
- **GitHub's way, in a new tab** (the user, 2026-11-19, the look request
  `github-new-tab`, as a sign-in's: `../pages/accounts.md`, 1,
  `ConnectModel`, *The sign-in's tab*). Every departure to GitHub goes
  so: "Connect GitHub" on the welcome's Forge step and on Settings'
  Connect a forge, "Go on" on an unfinished GitHub forge, Connecting's
  "Install on GitHub", and the Forges row's "Install on another
  account" and "Choose its repositories on GitHub".
  - *The press* opens a tab in its own turn, before anything is sent
    (`Request::Tab`, `Tab::Open`), holding "Opening GitHub…" until the
    link comes: a browser blocks as a pop-up a tab opened once an answer
    has come back. It sends `Connect` or `Install` with no `back`. The
    answer's link, the manifest's page on the clients' listener or
    GitHub's install page, is pointed in that tab (`Tab::Point`), its
    opener cut, once, from that answer; the App, its install, GitHub's
    redirect and the setup route run there (`served.md`, 1). The
    manifest's page, answered as a whole address on the clients'
    listener, the browser points at by its path and query on the page's
    own origin, `/forges/N/github/manifest?state=S`, the address less its
    scheme and host, as the page went with `back`: a browser's platform
    points a tab only at a path on the origin or an `https` address, its
    plaintext loopback a test page's alone
    (`skein/docs/design/ui/web/README.md`, 8 and 14), and a page reached
    through another address than the listener's names its own origin.
    The terminal prints the whole address. An answer
    with no link (connecting, refused, unknown or lost) closes the tab
    (`Tab::Close`), and the page says why, as it does today. The page's
    own address stays; nothing leaves.
  - *While it waits,* the press's place (the choices on the welcome's
    Forge step and on Connect a forge, Connecting's act, the row's acts
    on Forges) holds the note a sign-in draws, by the same function
    (`../pages/welcome.md`, 1, its Model step, *The sign-in under way*),
    in its two forms: the welcome's, with its spinner, on the welcome's
    Forge step; the connection's, as a sign-in's on Settings' Connect a
    model, on Connect a forge and on Forges. GitHub's words: "Waiting
    for GitHub, in the new tab"; for a connect "Make temper’s App there,
    then choose the account and the repositories it may use. This page
    moves on by itself.", for an install "Choose the account and the
    repositories temper’s App may use there. This page moves on by
    itself."; then "Open it again ›", drawn as the sign-in's external
    link. In the browser it is the client's link (`Role::Link`), its
    `href` the manifest's path and query, its press opening a tab and
    pointing it there as the first press did (`Tab::Open`, then the
    point on the page's origin), sending nothing: skein's external link
    takes only a whole `http` or `https` address
    (`skein-ui`'s `Builder::external`, `markdown::safe_url`), and a
    forge's view knows no page origin to make one. The terminal's note
    and an install's link keep the whole address. A path the client
    can't point closes the tab and draws the refused note. A
    browser that refuses the tab is told to the root
    (`Event::TabRefused`): nothing is pointed, the title reads "Waiting
    for GitHub", and the link is the primary "Open GitHub ›". The
    terminal draws the same note with "Waiting for GitHub" and "Open
    GitHub ›", and prints the address, as it did (`../line.md`, 5). No
    paste follows it: GitHub's way needs none.
  - *"Open it again ›" follows the way* (the look request: "the
    manifest page until the App exists, then the install's link"). A
    connect's note holds the manifest's page's link until a `forges`
    document since the press shows the forge's row with the App it
    lacked at the press (its `app`, `records.md`). From then, "Open it
    again ›", or "Open GitHub ›" where the tab was refused, is the
    forge's Install, a press as every departure is, drawn with the same
    look as the link it replaces: it opens a tab of its own, sends
    `Install` with no `back`, points the tab at the install's link, and
    starts the install's way in this one's place, its note in the
    install's words and its end counted from that press. Nothing is
    asked for unpressed: an `Install` sent as the App appears would
    meet the redirect's own runs holding the forge and be answered
    connecting. An install's way, whose App was there at its press,
    keeps its link throughout. The terminal's "Open GitHub ›" is the
    same Install, whose answer it prints.
  - *Its end:* the page moves on by itself, on the `forges` watch: the
    way has ended once the forge's row shows an accepted installation
    it did not have at the press, or a last connect's line where the
    press's admission cleared it (`served.md`, 1, *Each way's end
    reaches the forge's row*), the client taking a line only once a
    document since the press has shown the row without one. The note
    goes, and the page draws the forge as the watch says, as the page
    the tab came back to drew it: on the welcome's Forge step, *The
    Forge step with a forge*, Connecting or the cut-short card
    (`../pages/welcome.md`, 1); on Forges, the row; a page still at
    Connect a forge goes to Forges, its address replacing its own, with
    the row open, as Completion does. A hidden tab lets its stream go
    after a minute (skein's `ui/web/README.md`, section 8), so a slow way
    moves on when the person returns. The new tab ends on the setup
    route's "temper has the installation; you may close this tab.", or
    an ending's "You can close this tab." (`served.md`, 1); temper
    closes no tab it has pointed (`../pages/accounts.md`, 1,
    `ConnectModel`, *Its end*; its closing itself is open there, 3).
  - *A reload while it waits* draws the page from the watch alone, with
    no note: the way is the client's memory, never the small store's.
    The forge reads as its row says, Connecting or cut short, whose "Go
    on" or "Install on GitHub" starts a fresh way.
- **Completion** lands on Forges with a confirmation line and the newly
  connected row open, adopting nothing (`../../protocol/setup/owned-repositories.md`, 1).
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

## 2. Open questions

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

- **A connect's end told:** the forges' page reads how a connect ended
  on the deployment's part. Default: that read, watched while the page
  is open; no notice in the inbox.

- **The third step's words while sign-in is connected** (`deployment.md`,
  *Setting up, in the row*; the look request `forge-steps`, which leaves
  them): while the box's step is the sign-in's source, the third step
  reads "Connecting sign-in through temper" in place of "Starting
  Forgejo", longer than the steps that wrapped in the welcome's pane, so
  it likely wraps there too; no request has drawn it. A look question for
  the user. Default: those words.
