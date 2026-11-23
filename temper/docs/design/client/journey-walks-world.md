# A journey's walk in a world

The world that executes a journey's walk is split from [journeys walked in a world](journey-walks.md).

## 1. A walk in a world

### 1.1 The world and its facts

- **One process and nothing else:** no child, socket, clock,
  environment or network; the walk reads only its chain's files.
- **What runs** is the playground's composition, as story A starts it
  (`../playground.md`, 3), at `facts::start`, with the chain's joined
  scripts where any journey of it holds a `scripts.json`, under the
  harness `Clients`, with the box's peer at the service's box pipe
  (1.4).
- **The forge at `FORGE`,** `127.0.0.1:10002`, the address a person's
  pages drew from a box (`../draft/archive/journey-walks.md`, 4.1): the
  composition started at a forge address of its caller's
  (`../playground.md`, 4), which every part that names the forge takes
  from that one address: the fake forge's listener, the forges' runtime
  destination, the place chooser's port for every setup file and the
  handover `story::prepare` lays, the hosted git's base, and the forge's
  authority pattern. So a page that draws the forge draws
  `127.0.0.1:10002`, as `project-landing`'s does. Story A's own start,
  the page's and its stories', keeps `documents::FORGE`.
- **The model's plan as a box's fake backends give it**
  (`../protocol/setup/box-backends.md`, 1): the deployment's providers made by
  `temper_fake::backends::providers`, provider 1's sign-in registered
  against the fake issuer at `ISSUER` and its callback at
  `127.0.0.1:1455`, `auth/callback`, every model endpoint at the world's
  fake model; the fake issuer, `temper_fake::oauth`'s document at
  `ISSUER` with that callback, at `SEED`, as a box's `fake oauth --seed
  7`, a host of the world's; and the service given its accounts'
  runtime. So the bearer the issuer grants is one the world's model
  takes. Story A's own start holds no issuer and its own providers.
- **The forge's sign-in through temper, as a box's:** the engine's
  OpenID provider as `temper-box init --box` writes it
  (`../protocol/setup/place.md`, 6, *The box's provider*): its signing
  key, `provider.key` in the data directory, on the world's disk,
  private: the 3072-bit RSA key `SEED` drew, a fixture, so no walk draws
  one at its start; no client; `addresses` `[1]`. The owned forge's
  `SetUp` adds the client `forgejo` and the setup file's `sign_in` names
  it, as on a box, and the peer says the group's `source` step (1.4). So
  the engine serves the forge's `temper-forge-sign-in` fact, and a page
  that draws the forge's repository draws its link, `work/hello ↗` to
  `http://127.0.0.1:10002/user/oauth2/temper?redirect_to=%2Fwork%2Fhello`,
  as `project-landing`'s does. The fact reaches the page as a browser's
  does, in the page the clients' listener serves: the walk's start
  adopts a browser bundle of one HTML page and no other file
  (`web/serving.md`, 1), in memory, so the engine serves that page
  with its forges' metas, and `web::Page`, at its start and at each
  reload, fetches the page at the path it starts at and gives the host
  each `temper-forge-sign-in` meta's content, read by
  `temper_client_view::forge_sign_in::Forges::read` as the web shell's
  start reads them. The native host draws the page; the bundle's page
  is read only for its metas. Story A's own start holds no provider and
  no bundle.
- **A box's bounds:** where story A's configuration
  (`temper_playground::documents::configuration::story_a`) holds a bound
  tighter than a box's, and a step a box takes is refused by it, the
  walk's start takes the deployment's default for that bound, as a
  box's deployment does: `tasks.spec_bytes`, from 187 to 131072 bytes,
  for the goal's specification from a draft; and
  `people.amendment_bytes`, from 1218 to 135168 bytes, for that
  specification with its authority in the person's create request.
  Story A's own start keeps its bounds.
- **The browser starts** on the page the printed link opens, signed
  in, at `/welcome`. Nothing else is implied: the walk's first line
  acts on that page.
- **The TUI starts** at the walk's first `tui` line, not signed in, at
  `journey-walks.md`, 3.3's start: 80 columns by 24 rows, its build `0.0.0
  (000000000000)`, the service's origin, a terminal that answers no
  scheme. A walk that needs it signed in says `tui approve`.
- **The same bytes every time:** the simulator owns the schedule. The
  browser's first start takes `SEED`; each reload takes the next seed
  from a `skein_lib::Rng` started at `SEED`, as a browser load supplies
  fresh entropy, so its request keys name new acts, not an earlier
  load's. Each TUI takes `facts::tui_seed`, and both clients the world's
  clocks, so a walk takes the same turns on any host, on any day.

### 1.2 Each line

A browser line's bound is the drive session's figure (`drive.md`, 3.2),
60 s, and a TUI line's its own (`drive.md`, 3.3), 30 s, in the world's
time.

- **`web press`, `submit`, `toggle`:** `wait` for the control, by its
  role, its whole name and its order from 0 (`Find::exactly_named`,
  then `nth`; `Find::text_box` for `textbox`), with the line's act;
  then `act` on it.
- **`web input`:** the same, its act `Act::Input` with VALUE.
- **`web see TEXT`:** `wait` for `Find::text(TEXT)`, no act.
- **`web reload PATH`:** the page started again at PATH: the world, its
  cookies and the page's small store kept; a new host and its page,
  with the sign-in ways the start had and the store's records, no
  secret, and the served page's forge metas and a fresh seed from
  1.1's sequence, as a browser's reload builds them
  (`temper-client-shell-web`'s start); then `wait` for the main region.
- **`web mark`:** in the walked journey, a mark: the page's HTML, its
  heading (`print::heading`), and the tree kept for the link (`journey-walks.md`, 2.4); in
  an ancestor, nothing.
- **`tui key KEY`:** `tui_key`, the key given as a terminal gives it;
  no bound.
- **`tui see TEXT`:** `tui_wait` for TEXT, 30 s.
- **`tui stop`:** `tui_stop`. The next `tui` line starts a TUI again
  from what the last one held, its records, its credential and its
  note, as a person's next `temper` starts from its files.
- **`tui approve`:** `tui_approve`, within 30 s for the whole line:
  wait until the TUI has asked to browse an address on the service's
  origin, `http://SERVICE`, which may hold already; the page reloaded
  at that address's path and query, as `web reload` does; button
  "Approve" found and pressed, as `web press` does; then the page
  reloaded at the address it showed when the line began
  (`Page::address`). The approval reaches the TUI by the TUI's own
  asks, which its next `tui see` waits for.
- **`tui mark`:** in the walked journey, a mark: the frame as drawn, and
  its key (`journey-walks.md`, 2.4); in an ancestor, nothing.

### 1.3 A stop

A line that does not hold stops the walk: `Stopped`, naming the chain's
journey, the line's number and what it sought, as `button "Try the
playground" not found within 60 s`, `"then press Approve." not seen in
the TUI within 30 s`, `the TUI asked to browse no address on the
service within 30 s`, `the TUI closed`, `the TUI asked to lend its
terminal, which a world does not play` or `the world failed: WORDS`.
Its marks print nothing. A chain that does not read stops its walk as
well, with `Unreadable`'s journey, line and why.

### 1.4 The box's peer

The world's `temper box`, as far as the service meets it: the box's
pipe (`../protocol/setup/box.md`, 1), played by `box_peer::BoxPeer`, a
host of the world's, on the world's time. Nothing else of the box runs:
no group's steps, no listener of its own, no `box.log`, no record under
`box/`.

- **The pipe:** the service adopts the box's two pipes at its start,
  `temper_service::Service::adopt_box`, within the 256 bytes a line the
  composition gives the box's lines, their other ends the peer's, as
  `temper box` gives serve its pipes at each spawn. The one-process
  world's box, `temper/tests/one-process/src/ready_copies.rs`, is the
  service's parent and answers a start alone; the peer starts from it.
- **Each line read whole** to its line feed, one ask at a time, and
  answered as the box answers it (`../protocol/setup/box.md`, 1, the
  table and the paragraph after it):
  - **`start N` for an owned group,** its setup file of the owned kind
    on the world's disk (`temper_forge_setup::setup_file::path`): the
    script below, from the turn the line is read;
  - **`start N` for the playground's group,** of the fake kind: no
    answer. The composition models that group standing from its start,
    its handover laid before the service starts
    (`temper_playground::story::prepare`), and its forge connects by the
    engine's own read of it, as story A's does with no pipe. A `started`
    would start the engine's handover again, or stop it as busy, so the
    peer leaves the ask open, as the composition leaves it today;
  - **`start N` for a group whose script has ended:** `started N`; while
    a script runs, for another forge, `failed N busy`;
  - **`stop N`** and **`remove N`:** `stopped N` and `removed N`, at once;
  - **any other line,** or one past 256 bytes: ignored, as the box
    ignores one.
- **An owned group's script,** `OWNED`: each line at its time after the
  `start` was read, in the world's time. Its figures are Forgejo's 100 MB
  and its runner's 20 MB, round sizes that draw a percentage; two
  seconds between lines, so that the page draws each line's state
  before the next arrives. What the client draws of each is
  `forge/deployment.md`'s, *Setting up, in the row*, and its table's
  *setting up* row:

  | After | Line |
  |---|---|
  | 0 s | `step N fetch` |
  | 2 s | `fetched N forgejo 25000000 100000000` |
  | 4 s | `fetched N forgejo 100000000 100000000` |
  | 6 s | `fetched N runner 10000000 20000000` |
  | 8 s | `fetched N runner 20000000 20000000` |
  | 10 s | `step N prepare` |
  | 12 s | `step N run` |
  | 14 s | the handover laid, then `started N` |

  A group whose setup file names a `sign_in` (1.1) takes the box's
  `source` step after `run`: `step N source` at 14 s, `source N added`
  at 16 s, then the handover and `started N` at 18 s
  (`../protocol/setup/box.md`, 1). A later `start N` for that group says
  `source N added` again before its `started N`.

- **The handover:** at `started N`'s time, before the line, the peer lays
  what the box's preparation leaves the engine
  (`../protocol/setup/owned.md`, 5): `admin_token` in the group's
  directory, as its setup file's entry names it
  (`../protocol/setup/place.md`, 3), on the world's disk, private, holding
  the admin token the world's fake forge takes, as `story::prepare` lays
  the playground's. The forge is the composition's, at its own address,
  where the composition's place chooser puts every setup file's port
  (`temper_playground::documents::FORGE`). The engine then takes the
  handover, reconciles, connects as temper and makes the first
  repository, as with a box's Forgejo.
- **No step through ctl:** the world holds no ctl. An owned set-up goes
  on by the walk's acts, the peer's lines and the engine's own steps
  alone, and the client draws each as it draws a box's.
- **The same bytes every time:** the script's times are the world's, so
  a walk meets each line at the same turn on any host.

### 1.5 A sign-in's tab

The tab a model's sign-in opens (`../draft/sign-in-new-tab.md`, 2),
carried as a browser carries it, so that an account is connected through
the client (`../draft/archive/journey-walks.md`, 4.1).

- **The page's tab duties,** `temper-client-shell-web`'s `Duties::tab`,
  are carried in the turn that gives them, before its transport, as the
  shipped browser carries them: `TabAct::Open` opens the world's tab,
  which a world never refuses, so the host is given no
  `Report::TabRefused`; `TabAct::Point { to }` sends it to `to`;
  `TabAct::Close` closes it, any visit under way abandoned. A tab opened
  again replaces the last.
- **The world's tab,** `skein_playground::tab::Tab`, skein's (skein's
  `playground.md`, 4), a host of the world's: a `GET` of the address on
  a connection of its own, its `Host` the address's authority, with no
  cookie and no `Origin`, as a new tab whose opener was cut sends it; a
  redirect, 301, 302, 303, 307 or 308, followed to its `Location`, at
  most 8; the last answer kept as `tab::Shown`, its address, status and
  body up to the tab's bound. An answer refused, reset or past its
  bound, or a ninth redirect, is kept as the last. The tab draws
  nothing a mark prints.
- **The way there and back:** the fake issuer grants at once
  (`../protocol/setup/box-backends.md`, 1) and redirects to the engine's
  callback, `http://127.0.0.1:1455/auth/callback?code=…&state=…`; the
  engine's listener answers "Sign-in received. You can close this
  tab." and exchanges the code with the issuer; the page, still on its
  address, moves on by its `accounts` watch, to Ready on the welcome
  (`../draft/sign-in-new-tab.md`, 3). That is the page the sign-in
  returns to, its account connected through the client.
- **A tab that does not get there** leaves the page waiting, as a
  person's would: the next line that needs the account stops the walk
  by its bound (1.3).
- **The same bytes every time:** the sign-in's state, verifier and code
  come from the world's seeds, and the tab moves on the world's time.

## 2. Open questions

- **A forge's departures** carried by the world's tab as a model's
  sign-in is, once a journey walks GitHub's App or install
  (`../draft/sign-in-new-tab.md`, 6).
- **The playground's group answered** as the box answers it, once the
  composition no longer lays that group's handover before its start
  (`journey-walks-world.md`, 1.4).
- **The box's `mark` compared with the world's page,** by their
  accessible text, as a check that a world draws what a box does.
