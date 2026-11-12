# The view

## 1. Its shape

`temper-client-view` builds temper's pages as skein-ui's tree (skein's
`ui/README.md`), one owner, temper's:

- **Its state:** skein-ui's view (the last tree, a spare, node ids never
  reused); temper's phrasebook, English, with jig's words for jig's
  enums and a decision's facts (`jig-client`'s `words`) under it,
  overridden phrase by phrase where temper words them otherwise
  ("Funds 40 from your pool · 120 left, 80 after"), and
  temper's own phrase for what `jig-client`'s `words` does not say (an
  executor, an outcome); temper's bindings for the last tree; its
  limits; the instant its words next change (6).
- **`build`:** at the shell's time, when the domain's shown count moved
  or that time has reached the instant its words next change (6),
  it builds the page in one outline (skein's `ui/README.md`, section 12), in
  the mockups' order:
  - *the rail (3):* on a project page, every page but `Welcome`
    while a project is open and the link is neither signed out nor out
    of date, in place of the top bar below. In the browser the tab bar
    (3) follows the main;
  - *the top bar,* on any other page, the frame wherever the rail
    isn't (the look request `outside-pages`; `web/look.md`,
    *Outside a project*): temper's mark, "temper", a link to `/`; then,
    signed in, at its right: "Offline" while the link is offline and
    "Behind" while a page's watch reopens, as the rail's foot shows them,
    the person's name, "Sign out did not reach the engine." while that
    holds (`line.md`, 6), and "Sign out" once `me` is read or while that alert
    shows.
    Nothing else: no menu, no switcher, no navigation, no "Set up
    temper". Its link region holds the link's state's word in every
    state, as the rail's foot's does, so that the terminal's first line
    prints it (`line.md`, 4); the browser shows the region only while it says
    "Offline", so no "Signed out", "Connecting" or "Live" shows there.
    Until `me` is read, the top bar shows its mark alone, so that no
    other frame shows before the first page. On `Welcome`, temper box's
    line and the welcome's steps follow the mark once the deployment's
    read answers; its right is as on every other page, before that read
    too (`pages/welcome.md`, 1);
  - *while `me` is read,* the top bar alone, at every address but the
    sign-in page, the `Authorize` and approval places and the pages not
    about the work (below), which draw without `me`: no banner, an empty
    main marked busy and no tab bar, and the browser's terminal look waits
    too, so that no page's body shows before `me` says which project is open
    (the look request `outside-pages`);
  - *the accounts' banner,* while no account the person's work may use
    is usable (the `accounts` watch; jig's
    `client/domain/contract/README.md`, 5.1), but in the playground's project
    while the playground runs or is leaving, whose work its scripted
    model runs (`forge/family.md`, 7): a region named "LLM account", its words and three buttons
    (`pages/accounts.md`, 2), on every page but the accounts page, `ConnectModel` and
    `Welcome`;
  - *the playground's banner:* a region named "Playground", one line,
    its words and its link "set up temper" (`pages/welcome.md`, 2); on `Chats` and
    `NewChat`, while the playground runs or is leaving
    (`forge/family.md`, 7); on every other page, `Welcome`, the accounts and the forges
    pages and a chat's page among them, while it is leaving alone;
    above the main, after the accounts' banner where both show,
    the rail's "Set up temper" being the way out while
    it runs (the user, 2026-11-03); while the link is signed out or out
    of date, none;
  - *the main:* the page at the address (`pages/README.md`), its title its level-1
    heading, but `Home`'s, whose headline changes with what it says:
    "Home". Under the top bar the main is *the column* (`web/look.md`
    , *Outside a project*): its level-1 heading, its words, then its
    acts, the first filled, the others outlined, a quiet link after
    them; a field and its act share a form. The sign-in page, the
    `Authorize` place, the approval page, the pages not about the work
    (below) and no project yet (`pages/welcome.md`, 1, *With no project yet*) are drawn
    in it; `Welcome`'s main is its own (`pages/welcome.md`, 1);
  - *at the approval place* (`pages/README.md`, 2, *The approval page*), in the browser
    alone: the top bar, as at the `Authorize` place, then the page's
    column as the main, whatever the link's state; no rail, tab bar,
    accounts' or playground's banner, and no paragraph of the pages
    not about the work (below), whose words the column's own states
    take.

  Then skein-ui diffs it and keeps the tree.
- **The pages not about the work** (jig's `client/domain/pages.md`,
  section 8). While the link is signed out or out of date, `me` has
  ended (jig's `client/domain/client/watches.md`, 2). The top bar is then
  temper's mark alone (the terminal's link region with it, `line.md`, 4), no
  navigation is offered, and the main, in place of the page at the
  address, is the column: the link's state its level-1 heading,
  "Signed out" or "Out of date", and temper's words under it, one
  paragraph:
  - *signed out:* in the shell's words, which the view takes at
    construction: the browser's "To sign in, open a sign-in link temper
    printed." (`web/README.md`, 2.4); the terminal's heading alone, which
    says it (`line.md`, 6);
  - *signed out, with a way of signing in:* in the browser alone, in
    place of the column above, the sign-in page's column: the heading
    "Sign in", and a link for each way the view was given at
    construction, "Continue with Google", filled, to
    `/sign-in/google?then=PATH`, `PATH` the page's printed address
    (`../protocol/sign-in.md`, section 7), then "Read public
    projects", quiet;
  - *signed out by Sign out, with a forge that signs in through
    temper:* the sign-in page, under its heading, for each such forge,
    "You signed out. FORGE keeps its own sign-in: sign out there too.",
    FORGE the forge's name (`forge/deployment.md`), read from the page's facts
    (`../protocol/openid-provider.md`, 3 and 6);
  - *signed out at the authorization:* the `Authorize` place's sign-in
    page, "to continue to FORGE" under its heading, its link's `then`
    the authorization's address (`../protocol/openid-provider.md`, 3);
  - *signed out at the approval:* the approval page's own column, "Sign
    in first" (`pages/README.md`, 2, *The approval page*), in place of the column above
    and of the sign-in page: with Google's way given, its "Continue with
    Google" returns to the approval with its code; without it, the
    printed link is the browser's way in;
  - *signed out, the link's sign-in not taken:* while the link's view
    says why its last sign-in installed nothing and the link stays
    usable (jig's `client/domain/client/README.md`, section 9,
    `not_signed_in`), the paragraph says so in place of the shell's
    words: full, "All N sign-ins are in use; the earliest ends MOMENT
    UTC. This link stays usable until LINK UTC; after that, opening it
    has `temper-box serve` print a fresh one." (N the deployment's room,
    MOMENT the earliest saved expiry and LINK the link's own, each with
    its date beside its clock, "Oct 24, 09:30", as the accounts page says
    its moments, `pages/accounts.md`, 2: a sign-in's life is a day or more, so a bare clock
    would name tomorrow's as today's; the minute is the expiry's own, so
    the link serves a little past what it says); busy, "temper is busy. This
    link stays usable until it expires: open it again in a moment."; not
    ready, "temper is starting or stopping. This link stays usable until
    it expires: open it again in a moment.";
  - *signed out, the link spent:* while the link's view says why it can't
    sign in, the paragraph says so in place of the shell's words and of the
    sign-in page, in either shell, saying only what whoever holds the link already knows: used,
    "This link was already used. A fresh one is in your terminal."; past
    its expiry, "This link has expired. A fresh one is in your terminal.";
    any other, a real link of an older box or one that never was, "This
    link isn't from this temper. Use the one its terminal printed." The
    reason is the engine's: it remembers only its last 8 used or expired
    links in memory, so a forgotten link, after a restart included, reads
    as unknown (jig's `protocol/client/sign-in.md`). A page reached with
    no link keeps the shell's words;
  - *out of date,* the engine serving no version of jig's records that
    this client speaks: "Run the temper the engine runs." under "Out of
    date". What is pending stays in the small store (jig's
    `client/domain/client/watches.md`, 2). In the browser, a page whose
    link turns out of date, or whose forge's records are not served
    (`forge/family.md`, 4), first reloads itself, once, to load the
    engine's bundle (`../protocol/wires.md`, section 8); the column
    shows when the page that reloaded for it is out of date again, and
    it reloads no more.

  Both are phrases of temper's phrasebook: `jig-client`'s `words` has
  none for the link's states, and temper arranges none of jig's
  pages (`README.md`, section 1). The page at the address is shown again once the link is
  starting, live or offline. When jig's records are served and the
  forge's are not, the link stays live, and the form says its own out
  of date (`forge/family.md`, 4).
- **`act`:** a shell's act on a node is resolved by skein-ui in the
  last tree to its binding, and decoded by temper's bindings into the
  domain's action. A node gone, or a token of an older tree, decodes to
  nothing (skein's `ui/README.md`, section 6).
- **Every node a person acts on has a role and a name,** since skein's
  fake person finds nodes only by them; nothing a person must know is
  said by style alone (skein's `ui/README.md`, section 8).
- **The look:** the mockups' (`web/mockups/style.css`),
  as style tokens each shell interprets or ignores; the terminal prints
  none, and the browser's stylesheet draws them (`web/look.md`).
  The composers are text areas that submit on Enter (skein-ui's
  `SubmitsOnEnter`), for the browser, and whose words may hold line
  breaks, counted within their bounds (4; `web/look.md`).

## 2. The forge's view

- **Words:** the forge's phrases in temper's phrasebook: the connector,
  resources, patterns and kinds (`forge/family.md`, 1); "Shown on the forge." (`forge/family.md`, 2);
  a change's panel's: its tag, its pull request's line, its steps,
  its step's lines, its holds, its gates' rows, its bounds and its
  panel's slot words (`pages/task.md`, 3); the form's title, field names, roles,
  rules and button (`forge/family.md`, 4), its title by the project's number included;
  the picker's nodes, "Not listed" and its refusals (`forge/family.md`, 4); the outcome
  (`forge/family.md`, 6); the form's out-of-date sentence (`forge/family.md`, 4); the forges' page's
  (`forge/deployment.md` and `forge/repositories.md`, 4); the navigation entries. Names (owners,
  repositories, branches, logins) are copied in, never translated.
- **The parts:** a connector, resource, pattern or kind is one text
  node of its words; a part the forge's side does not read is the
  paragraph of `forge/family.md`, 2; a part it reads is shown as `forge/README.md` says.
- **The form's page:** a level-1 heading (the title), then the form's
  nodes in the order of `forge/family.md`, 4: the forge's field, the picker's nodes,
  text fields with their values and write counts, the role as a choice
  group of three choices, home and ci as checks, the button, and after
  them a status for progress and an outcome, or an alert for a refusal.
- **The forges' page:** a level-1 heading, "Forges", then "Your
  forges" with their count and "+ Connect a forge", then a disclosure per
  listed forge, folded, named by the forge's name, its summary the
  forge's number, kind and `web` address, its connection and its line,
  holding its rows and its actions, "Remove" last, its confirmation in
  its place (`forge/deployment.md`); the playground's forge in a section of
  its own; a pasted value a secret field (`README.md`, 3.6), a
  link an external link (`line.md`, 5), and a connect's or an action's status
  under its button. While the `forges` watch is refused, the page says
  so in jig's words; until it has delivered, its headings alone.
- **`ConnectForge`:** its Settings path and level-1 heading, “Connect a
  forge”; the shared choice cards or the selected way’s panel, with its
  fields and acts. No Steps list or outer Choose card. A running or
  connected forge opens its row on Forges, where progress and installation
  acts live (`forge/deployment.md`).
- **The accounts page:** its level-1 heading, “Accounts”, its lede and
  account groups, each row a folded disclosure. Its summary names the
  account, holder, deciding source’s tag and line (`pages/accounts.md`, 2). Opening adds
  source order, source acts, rule, use facts and account acts, without
  repeating the summary’s name, models, Ready or Pays for. A secret field
  is the row’s (`README.md`, 3.6), and an act’s status stays under its row. Refused
  reads say why; undelivered reads draw only the headings.
- **`ConnectModel`:** its Settings path and level-1 heading, “Connect a
  model”; shared plan cards, then API key rows, or the selected way’s
  panel and acts. A sign-in’s address is an external link (`line.md`, 5). Success
  lands on Accounts with its confirmation and Start a chat, without a
  Ready page. The holder switch is one quiet link at the foot.
- **A flow's parts,** shared by `ConnectForge`, `ConnectModel` and the
  welcome, are the view's `flow` module. Choice cards, panels and controls
  use the shared connection tokens (`web/look.md`); progress is a list
  whose lines retain their state in words. Forges rows use the compact
  live list. The welcome alone keeps its frame’s Steps links (`pages/README.md`).
  Settings flows have no steps bar, outer Choose card or Ready card.
- **Bindings:** writing a field, choosing a role, toggling home or CI,
  choosing the picker’s forge or installation, checking repository rows,
  searching and nearing the list’s end, sending its plural adoption,
  making or copying, each forge action and navigation entry.
  Typed input is made here: the forge number's digits become a number,
  the checks' words a list, or nothing when they are not one, which the
  form marks invalid.
- **No drawings.**

## 3. Navigation

**The rail** (`../draft/surfaces.md`, section 2; the whole story's mockups,
`web/whole-story/`, each page's `nav.rail`), on a project page (1): a
navigation named by the open project, in this order:

- temper's name, a link to where "Home" goes, named "temper
  home";
- the switcher (*Projects in the rail,* below);
- "Home", to the open project's `Home`, and to `AllHome` on a page across
  projects; "Needs you", to *Needs you* filtered by the open project, and
  on a page across projects to *Needs you* across them, `Inbox` with no
  project, with the count of what needs the person there
  (`pages/needs-you.md`, 1, *What the client keeps*), omitted when zero
  (*All projects,* below); "Work", to
  `Work { project, filter: All }` (`pages/work.md`, 1, *Work*); "Team", while the rail's
  parts show it (*The rail's parts,* below), to Team's page
  (`../draft/surfaces.md`, section 6); "Ask temper", to the person's newest
  Ask temper chat open in the project or else `Ask { project }` (`pages/ask-temper.md`, 1, *The
  first chat*), only while `me` lists template 5 among the project's
  templates (`../draft/ask-temper.md`, section 9; the look request
  `ask-temper-place`); a hairline, `div.temper-rail-sep`, named nothing;
  and "How work runs", to `Settings { project }` (`pages/settings.md`, 1).
  Kind 7's switches hide no entry of the rail: Goals' and
  Changes' hide Work's filters (`pages/work.md`, 1, *Work*). "Work" is current on `Work`, `Goals`, `Changes`, `Chats`,
  `NewChat`, a `Chat` but an Ask temper chat's, and a `Task`; "Ask temper"
  on `Ask` and an Ask temper chat; "How work runs" on `Settings`,
  `Repositories` and `Adopt`;
- the new box, `div.temper-rail-new`, one dashed box. While the project
  offers drafts (`drafts_offered`, `pages/chat/document.md`, 1), it is "+ New draft or change", the
  button of skein-ui's menu (`Role::Menu`; skein's `ui/web/elements.md`),
  named "New draft or change", opening a panel under the box 288 pixels
  wide, past the rail's edge and over the page. Its items, a name and a line
  each: "Draft", "A design session that ends in a goal, best had in the
  terminal.", to `NewChat { draft: true }`; "Change", "Ask for one change.",
  to `NewChat { draft: false }`; and, while the rail shows "Ask temper",
  "Ask temper", "Ask about this project, or change how its work runs.",
  where the rail's "Ask temper" goes. With no drafts offered, as at Pair
  (`../draft/surfaces.md`, section 12), the box is "+ New change", a link
  named "New change", to `NewChat { draft: false }`, with no menu. The box
  takes the hover's grey; open, the button keeps it. Skein's keys, arrows
  and Escape; a press on an item or outside closes it; a focused item
  takes the hover's grey. A phone doesn't show the box. Which templates it
  offers is fixed until `../draft/template-places.md` is taken up;
- at its foot: the Away switch, while `pages/README.md`, 2's *Away* shows it; while the
  playground runs and is not leaving (`forge/family.md`, 7),
  "Set up temper", the playground's way out (`pages/welcome.md`, 2); the link's state, as
  1 has it, the browser hiding it while live; "Behind" while a
  page's watch reopens, beside the link's region in the browser, so
  that it shows while the link is live, and in the region in the
  terminal, whose first line reads it (`line.md`, 4); "Deployment", to `Accounts`,
  Deployment's first tab (`pages/settings.md`, 1), current on
  `Accounts`, `Forges`, `Models`, `Builders`, `Members`, `Limits`,
  `ConnectForge` and `ConnectModel`, with the dots "Forges" and "Accounts"
  carry below, each its state in words; the person's name, a line in the
  terminal and the person's menu in a browser; and "Sign out did not reach
  the engine." while `me` or the page cannot say otherwise (`line.md`, 6), the
  terminal's "Sign out" being the `Accounts` tab's (`pages/README.md`);
- the person's menu, a browser's: the person's name is the button of
  skein-ui's menu (`Role::Menu`; skein's `ui/web/README.md`, 5), in a
  `div.temper-rail-person`, drawn as the foot's other rows. It opens a panel
  above the row, as wide as it, in the look of the switcher's menu. Its items:
  "Accounts", to `Accounts`; "What temper remembers about you", to
  `Remembered`, Ask temper's memory of the person (`pages/remembered.md`); "Telling you", to
  `Told`, where pushes go (`pages/told.md`); and "Sign out", the
  `Accounts` tab's act. A press
  on an item, Escape and a press outside close it, the focus back on the row
  unless that press took it elsewhere. Open, the row keeps the hover's grey. A
  phone has no foot, and signs out on the `Accounts` tab (`pages/README.md`);
- the night switch, a browser's, the rail's last child, after its foot and drawn
  at the foot's right: a button named "Night mode", pressed while the night look
  is on, a moon by day and a sun by night, drawn by style; on a phone, at the top
  bar's right. It shows on every page that draws the rail, at every width,
  wherever the rail's other parts go: in the rail folded beside a chat's Split,
  at the folded rail's foot; on a phone's chat layout (`pages/chat/README.md`, 1, *One layout for
  every chat*), where the rail and its top bar go, at the right of the chat's
  own bar, which keeps room for it (the look request `night-mode`, which
  places it at the rail's foot or a phone's top bar). Pressing it turns the look, on every page of that browser, and
  keeps it (`pages/chat/terminal.md`, 1, *Its look, kept per browser*; `web/look.md`, *Night*). The
  terminal has none.

**While To begin shows** (`pages/home.md`, 1, *To begin*), on `Home` and on `Ask { project }`
alone: the rail's navigation holds "Home", "Ask temper", while `me` offers
template 5, the hairline and "How work runs", and no other entry
(`../draft/surfaces.md`, section 2; `web/whole-story/pair/first-run.html`);
`div.temper-rail-new` is not drawn; the foot is as above, "Deployment" among
it, so that today's Settings is "How work runs" and "Deployment" there too.
The tab bar holds "Home", "Ask" and "Deployment", three equal tabs, How work
runs reached by the switcher's menu as on any phone page (*On a phone,*
below). Both shells draw the
rail so, the terminal reading its entries as the browser does. The view asks
`temper_client_domain::pages::home::beginning(root, project, wall)`, true on
those two addresses while `facts(..).to_begin()` holds; every other page draws
every row (the look request `to-begin-two-steps`, the user's word of
2026-11-07).

**The rail's parts** (`../draft/surfaces.md`, section 12): what the rail and
the tab bar show is one pure function of the domain's,
`temper_client_domain::pages::rail::parts(&Facts) -> Parts`, which both shells
draw from and neither decides again. `rail::facts(root, project, wall)`
gathers its facts, plain values, once a build; no fact is a page's address,
and what is current is the view's. Its rules:

| Part | Shows when | Its fact |
|---|---|---|
| "All projects", the switcher's first row | the person is in two projects or more | `me`'s projects |
| "Needs you"'s count | it is more than zero | `pages().needs().count(..)`, as above; on a page across projects, the count across them (`pages/needs-you.md`, 1, *What the client keeps*) |
| "Team" | the project has a second member or a standing role | the project's people, from its document, and its live standing roles, from the roles' watch (`pages/team.md`, 1, *Its reads*) |
| "Ask temper", in the rail, the tab bar and the new box's menu | `me` lists template 5 for the project | `me`'s templates |
| the Away switch and chip | `pages/README.md`, 2's *Away* shows it; on All projects, it shows for any of the person's projects (`pages/README.md`, 2, *Away*, *On All projects*) | `Domain::away_shown`; on All projects, `pages::away_all::across` |
| the new box's menu, "+ New draft or change" | the project offers drafts | `drafts_offered`; else "+ New change" alone |
| To begin's cut, above | `home::beginning` holds | `home::beginning(root, project, wall)` |
| "Home", the hairline, "How work runs", "Deployment" | always | |
| "Needs you", "Work" | always, To begin's cut aside | |

"Forges", "Accounts" and "Adopt a repository" are no entries of the
rail's navigation: they are Deployment's tabs and the Repositories tab's (`pages/README.md`).

**On a phone** (`../draft/surfaces.md`, section 10; the whole story's
`phone-home.html`, `phone-need.html` and `phone-tell.html`), at 760 px and
under, the rail is a top bar showing the switcher alone, and the tab bar closes
the page: in the browser, a navigation named "Sections" after the main, drawn
at phone width alone (`web/look.md`). Its tabs are one pure function of the
domain's, `temper_client_domain::pages::rail::tabs(&Parts) -> Tabs`, over the
rail's parts, which the browser draws from and decides nothing again:

| Tab | Shows when | Current on |
|---|---|---|
| "Home" | always | as the rail's "Home" |
| "Needs", with its count after it in the signal colour, "N+" where cut, named by its words, "Needs 2" | the rail's "Needs you" shows: its count while more than zero | as the rail's "Needs you" |
| "Work" | the rail's "Work" shows | as the rail's "Work" |
| "Ask", named "Ask temper" | the rail's "Ask temper" shows | as the rail's "Ask temper": Ask temper's page, an Ask temper chat and `Tell` |
| "Deployment" | To begin's cut holds, the rail's "Work" not shown | as the rail's "Deployment" |

Each tab goes where its rail entry goes, so on a page across projects "Home"
opens `AllHome` and "Needs" *Needs you* across them, with the rail's count
(*All projects,* below).
So four tabs in four columns as wide as their words, or To begin's three,
"Home", "Ask" and "Deployment", equal, and one fewer while `me` offers no Ask
temper. Team, How work runs and Deployment are reached by the switcher's "Team
for PROJECT", "How work runs for PROJECT" and "Deployment" (below), the phone
drawing no rail's foot and no person's menu. A chat's layout draws no tab bar
on a phone, but Ask temper's page and an Ask temper chat, the Ask tab's, whose
tabs close it under the composer (`pages/README.md`). The Away switch is a chip in the top
bar, on the rail's terms. Every target the top bar and the tab bar draw, the
switcher, the Away chip, the night switch and each tab, is at least 44 pixels
tall to a press (`web/look.md`, *What a phone presses*). The terminal has
no tab bar.

**No navigation outside a project** (1; the look request
`outside-pages`): a page that is no project's is drawn under the top
bar, which offers none; with no project yet, the column's acts are the
way on (`pages/welcome.md`, 1, *With no project yet*).

**No switcher outside a project** (`../draft/first-run-projects.md`,
7; the look request `outside-pages`): the top bar names no project;
`me` read with no project, the column's "New project" makes the first
(`pages/welcome.md`, 1, *With no project yet*), and once one is open the rail's switcher
lists them.

**Projects in the rail:** the same switcher, a bordered box with its
chevron, its menu a panel under it: "All projects", while the rail's parts
show it (*All projects,* below); each project, the open one first
and pressed, then, after a hairline, "Rename PROJECT" where the
rename is offered, the form "New project", "How work runs for PROJECT",
to the open project's `Settings`, and "Deployment", to `Accounts`. Each project's row says, in small
type at its right, what it holds, from the `forges` watch's rows
(`forge/records.md`), the playground's project from the playground's read (`forge/family.md`, 7): its
one repository, "work/hello"; with several, its home then how many
more, "infra/deploy +2"; with none, "no repository"; and nothing
while the watch is refused, ended or not delivered. A long name is cut
with an ellipsis before its count, never the count. `me` does not say
which repository is a project's home (`forge/family.md`, 4), so its first adopted, the
first its rows list on the forge numbered lowest, stands in for it.
The repositories in a row follow the forge's repository ids ascending,
not the time they were adopted.
In the browser the row is two siblings, since a link can't sit inside a
button: the project's name, the button as above, and the small words. A
repository with an address (`forge/deployment.md`, *A repository's link*) is
the link "work/hello ↗" there, in the same place, size and colour,
underlined on hover, and opens the repository's page in a new tab; the
project's name alone switches project. "+2" stays outside the link, which
goes to the repository named. "no repository", the playground's and one
with no address stay plain words.
With no home known, "3 repositories" is the row's words; the stand-in
leaves that state unreached until the client is given the home. In the
terminal a row reads "PROJECT · work/hello", one run of words. Choosing
a project keeps the page where it can (`pages/README.md`): on `Home`, `Work`, keeping its
filter, `Goals`, its filter All, `Changes`, `Settings`, `Models`, `Builders`,
`Repositories` and `Ask`, that project's page; on `Forges`, `Accounts`,
`Members` and `Limits`, the same page, the words naming the project changed.

**All projects** (`../draft/surfaces.md`, section 2): the switcher's first
row while the person is in two projects or more, a button named "All
projects", in small type at its right how many, "4 projects". Pressed, it
goes to a page across projects: on `Inbox`, its filter with the project
taken out, where a sign-in lands, `/`, having none; anywhere else
`AllHome` (`pages/home.md`, 1, *Home on All projects*). While the page is one across
projects, `AllHome` or `Inbox` with no project, the switcher's summary
names "All projects" and its row is the one pressed. "Home" and "Needs you"
stay across projects there (`../draft/surfaces.md`, sections 1 and 2): "Home",
temper's name and the phone's "Home" go to `AllHome`, current on it; "Needs
you" and the phone's "Needs" go to `Inbox` with no project, current on it,
their count across the person's projects (`pages/needs-you.md`, 1, *What the
client keeps*). The rail's other entries keep the open project, the one last
open, so that "Work", "Team" and "How work runs" open on it, and the switcher
names it again there; "Team" shows by that project's parts. Whether the page
is across projects is one function,
`temper_client_domain::pages::rail::across(root)`, which the rail's facts and
both shells' entries read.

The acts of both, each one act and one request (`README.md`, 3.6):

- **Rename,** after the rows in the rail, offered while `me`'s role on
  that project is owner, the role jig gives, which the context line
  reads too (`pages/welcome.md`, 2); jig judges the request (jig's `domain/people/acts.md`,
  1). Pressed, the open project's row is the form, "Rename PROJECT",
  and the rail's "Rename PROJECT"
  leaves its acts while the form is open. The form holds the name's
  field, filling the row, its value the project's name and the focus on
  it, then "Save", primary and small. The field's Enter or "Save" sends
  jig's `Rename { project, name }`, its name without the spaces at its
  ends, the form's acts disabled while it is out; Escape closes the form
  and sends nothing: the form's "Cancel" names it (skein's `ui/README.md`,
  6.1), which the terminal shows and the browser's sheet hides. An empty
  name, or the name unchanged, sends nothing and closes the form,
  keeping the old name. Answered renamed, the form closes, and `me`
  brings the new name wherever the project's name shows; refused (the
  name taken, its shape, the role), the form stays and says why under
  its field. A closed form leaves the focus on the switcher's summary
  (skein's `ui/web/README.md`, 5), or on a page's own field where the page asks
  it (a new chat, Ask temper's page, a composer after a send, `pages/README.md`).
- **A new project:** the list ends with a form, "New project", holding
  a name's field, "+ New project" its placeholder, and "Create".
  "Create", or the field's Enter, sends jig's `MakeProject { name,
  policy: 1 }` in the deployment's scope, on the stock policy
  (`../protocol/configuration/stock.md`); an empty name sends nothing.
  Answered made, the field empties and the client opens the project
  once `me` lists it, on its `Home`, with no repository (`pages/README.md`).
  Offered whatever the person's role, as above: jig judges it (an owner
  of the deployment), and a refusal says why under the field.
- **Its state:** the rename's form, at most one, with its project and
  words; the new project's words; each within jig's name bound; at most
  one request out of either, with its key; and the refusal, with the
  form it answers. Choosing another project, or a page of no project,
  closes the rename's form.

## 4. Limits

The view's limits are skein-ui's (skein's `ui/README.md`, section 13), derived
from the domain's: each page's nodes at its conversation's bytes (`pages/chat/conversation.md`, 2)
and its other windows' most rows, the
forge's form and its outcome's lines, the kinds' list at its longest,
and one paragraph per part not read. Their text across a tree,
`text_bytes`, is the nodes outside a window's turns at a node's most
text, and each of a window's lines at what its reader may make of its
turn's bytes (skein's `ui/markdown.md`) and its disclosures' names:
a lower cap would be a guess, which skein-ui's builder turns into a
panic on the page past it. Their patches are four a node, the most a diff emits (skein's
`ui/README.md`, section 7). skein's text shell counts its worst-case print from these limits,
and reserves for each print only what that page's tree may print
(`line.md`, 8).

A note's body and a task's words are `long_text`, 131,072 on each of temper's
clients (the browser tier's bundle may name less, with its window: `web/testing.md`,
3), past a field's `text` (jig's `client/domain/client/README.md`, section 11), and
a chat's document's section is read within the view's `text_bytes`, which prices one.

A line of a conversation holds words the store took, and they are `long_text` too:
a person's message or reply, up to `people.words` (8,192 on `temper-box init`'s box),
and an agent's message, question or answer and a result, up to
`tasks.message_bytes` and `tasks.result_bytes` (10,660 on it). Each draws whole in
the browser and the terminal, never cut (jig's `client/domain/client/README.md`, section
13). The window's bytes do not move: a line weighs what it holds within
`document_bytes`, including `LINE_ROOM` for each line and its parts, so a window
holds fewer lines the longer they are. The view prices the bytes at Markdown's
maximum nodes and text, plus ten structural nodes per `LINE_ROOM`, with room for
their names and text. A person's words waiting to be read total at
most `document_bytes` too (jig's `client/domain/client/conversations.md`), and the
view prices them so. A line past `long_text`, or a page of lines past
`document_bytes`, has its page refused and its read ended (`pages/chat/conversation.md`, 2): no box `temper-box
init` makes reaches one (`README.md`, section 5).

A composer, a chat's message, a new chat's first words, an agent's or a goal's
message or a question's reply, takes what a person types or pastes up to what one
message holds: `Me`'s `words_bytes`, the engine's `people.words`, within `long_text`
(jig's `client/domain/contract/documents.md`, 1). Past that nothing is cut, dropped or kept
stale: the words stay as typed, Send is disabled, Enter makes a new line, and a line
under the composer says how far over they are, counting the characters past the
bound: "That's 3,200 characters over what one message holds." The client holds an
edit's first `long_text` bytes and counts the characters past them in that line. A
browser's field keeps them, since a field's value is written only when its owner's
count moves (skein's `ui/README.md`, 3.4), and a composer drawn anew shows what the
client holds. A suggestion's words and a choice's stay within `CUE_BYTES`, as a
cue's and a button's. The view prices one composer's value at `long_text`.

A chat's first message reads back whole. Past the engine's `documents.text_bytes`
(512 on `temper-box init`'s box) its spec comes marked cut, and the chat's page reads
the rest through `TaskText`, as a goal's page does (`pages/goal.md`, 2), drawing what it has, ending
in `…`, until it has it all or the read ended. The view prices that line at
`long_text`. It prices a question read whole (`pages/needs-you.md`, 1, *Whole*) the same, one to a page,
with at most 16 choices (`question::CHOICES`).

temper's client takes every document within the bounds the engine's
configuration gives its documents. A path's segments have a bound of
their own, 9, apart from the windows' rows (jig's
`client/domain/client/README.md`, section 11), at least the most the engine
admits in a name (`../protocol/configuration/limits.md`), so a page
of the forge's work is never refused for a path. A path longer than a
node's text shows its last segments after the mark `…` (as jig's
`client/domain/view.md`, section 7 cuts one), which adds nothing to the
view's limits. An authority's entries have a bound of their own, 16,
apart from the windows' rows (jig's `client/domain/client/README.md`, section
11), at least the most the engine admits in an authority's list
(`../protocol/configuration/limits.md`), so a task's page is never
refused for its authority, nor an amendment drafted from part of one.
Each of an authority's lists shows at most the rows per window, then
jig's mark (jig's `client/domain/view.md`, section 7), which adds
nothing to the view's limits.

Both presets receive the dark theme's turn and the CSV chat's turns whole:
three rows (`rows`, which bounds a turn's messages and a message's blocks
as well as a page's lines), 512 bytes of field text (`text`), and 131,072 bytes
of a turn (`long_text`): the turn being written (jig's thread's `live`), and a
committed turn's shared text (`turn_text`, which jig's transcript prices its
calls' names first: jig's `client/domain/client/README.md`, section 11). The CSV
chat's first turn fits with its
`delegate` input of about 1.2 KB, so its card appears at that call (5).
A conversation's window is bounded by `document_bytes`, 1,048,576, not
16 lines (`pages/chat/conversation.md`, 2). jig's thread charges the bytes and `LINE_ROOM`, 64 bytes,
for each line, message, block, object and part. Its `lines` stays for jig's
reference UI and tools; temper sets it to `document_bytes / LINE_ROOM`,
16,384, which the bytes reach first. The view prices its window from those
bytes, including Markdown and ten structural nodes per room. Earlier and
later reads let whole lines go at the opposite end, and read them again
when that edge nears the view; nothing says how many left. No second page
is read merely because the window has room: a chat's opening reads earlier
only until its history's end or a screen and a half held (`pages/chat/conversation.md`, 2, *Opening
history*), and from there the browser measures the log and the terminal
waits for `up`. An empty page with an older cursor
reads on in either shell. The engine's pages keep their own rows (its
`documents.rows`, at most the client's). Committed turns on both the chat
and agent task pages are priced by `turn_text`; cards and other fields
retain `text`.

## 5. A delegation's card

A chat's delegation shows as the card of the task it made, at the call
that made it, or else at the result it sent back: a delegation as the
chips of the tasks it made, and their results (jig's
`client/domain/objects.md`, section 10), in one card per task, as
the mockups' first run shows it. For a change, it is the first form of
the change's chip in a chat (`forge/README.md`, 2.4 and 2.5).

- **Which tasks.** A `delegate` call that did not fail names the tasks
  it made: by the call's `objects`, each `Task(N)`, when jig's thread
  keys them there, and otherwise by its acknowledgement, temper's own
  words ("Delegated tasks: 2, 3", `temper-engine-smith`'s answers),
  which the domain reads N from (`README.md`, section 5). The chat's task document
  is not enough: it lists the chat's delegates while they live only.
- **Where it shows.** Once, at the first of its anchors the
  conversation's window holds: its call, or else a result block that
  carries its result, whose summary has a line "Result from task N:"
  after the inbox entry's "M: " (temper's inbox words,
  `temper-engine-domain`'s `inbox_words`); or else the chat's heard
  result from the task (its activity's `Heard`, a result from task N),
  which comes before the chat's resumed turn carries a result block and
  is never drawn as words of its own; or else, while the task
  lives, after the window's last line, as one of the chat's live
  delegates (its document's `delegates`). The chat's window holds a
  small conversation whole, its turns with their calls'
  acknowledgements (4), so the first run's change shows at the call
  that made it; in a longer conversation the call may leave the
  window, and the card shows at its result, or after the window while
  the task lives. Those words name a person's number as well as a task's: a
  result shows a card only once its task's document says this chat
  requested it (its `requester`), and keeps its work summary
  otherwise; such a number still takes one of the watches while the
  thread names it.
- **What it reads.** The chat page watches each such task's document
  (`Subject::Task`), as the task's own page does: its chip, its result
  contract, its hold and its result, which stay readable once it has
  ended. When the chat hears a delegate's result, it reads that task's
  document again once, so a cached running card takes its durable result
  without waiting for the chat's resumed turn or the watch's end.
  It watches those its thread's calls, results and heard results name, the
  newest first, then its live delegates, up to the domain's `reads`;
  one past them, or that neither names any longer, is unwatched, and
  leaving the page closes them all. A delegation whose document has
  not arrived, or past the bound, shows as its anchor's work summary,
  as any call does.
- **Its parts,** in the mockup's order, a region named "Task N":
  - a line with its number and kind, "TN · CHANGE", and its state; the
    kind is its result contract's: CHANGE, REPORT, VERDICT, APPROVAL;
  - its title, the chip's (the spec's first line), as a heading;
  - its result's words, once it has ended;
  - for a change, its steps, a list named "Steps": "Branch pushed",
    "Pull request opened", "CI passed", "Landed";
  - a link to its page, named "Task N" and reading "Task N ↗" (`pages/README.md`, 2, "Links are by
    number").
- **Its state,** in temper's words, from its document: "Running" while
  it lives, "Held" while held (why, on its page and in the inbox);
  once ended, "Landed" for a change done with a change's result,
  "Completed" for any other done (jig's word, as the task's page says
  it), "Failed", "Cancelled" (cancelled or withdrawn), and "Ended"
  otherwise.
- **Its result's words:** a landed change's, "CI passed. Landed ·
  merge commit ABCDEFG", ABCDEFG the first seven characters of the
  result's words' first line, which is the merge commit's id in hex (the
  engine's landing words, whose second line names the pull request,
  `../domain/root.md`, section 5); any other result's words as they are; a failed
  task's reason.
- **The steps are a stand-in:** none ticked while the change lives, all
  four once it has landed, none otherwise. A ticked step is skein-ui's
  `completed` item, its tick the look's (STEP_DONE): a screen reader
  reads "Branch pushed completed" and the terminal prints "- Branch
  pushed (completed)" (skein's `ui/README.md`, 3.2), so nothing is said by
  style alone (1). The change's live stages are later (`README.md`, section 5).
- **What it stands for:** the call's work summary, its result's, and
  the line that heard the task's result, whose words are the card's: a
  chat that shows a task's card shows none of them.
- **In the browser it is one line** (the greenfield mock's `.turn-tool`),
  the view and the terminal holding every part above:
  - *a delegated change:* the pen icon, "Change", its title in `--gf-ink-2`,
    its state in lower case, "held" in `--gf-signal` while held, its result's
    words once it has ended ("CI passed. Landed · merge commit 1a2b3c4"),
    then a link to its page, underlined as `.turn-tool button`. Its line of
    number and kind and its steps are hidden in the browser. The link's name
    stays "Task N"; its text is "Open" in the browser, "Task N ↗" in the
    terminal;
  - *a goal,* a delegate that is tracked, from its `goal` call or its
    document: the same line with "Started goal", its title in `.file` mono,
    its state and "Open".
- **The look:** the card's tokens (CARD and TASK_CARD, META, TITLE,
  DETAILS, a status's) and three of the steps: STEPS, STEP and
  STEP_DONE (`web/look.md`); on a chat's page these draw the line
  above, and elsewhere the mockup's `.steps`, `.step` and `.step.done`.
- **Where it is decided:** the domain's chat page holds each delegation
  (its task and its handle), and a pure function of the domain derives
  a card's facts from a task's document: its kind, state, title, words
  and steps ticked. The view lays them out with the phrasebook's words.
- **The first run:** the chat is task 1 and the change it delegates
  task 2, so the person reads "T2 · CHANGE", Running, then Landed
  (`../draft/archive/first-run.md`, 2.1), and after the card the chat's next
  call, folded, "1 tool call", described "wait" (`pages/README.md`).

## 6. Times

- **The view's clock is the shell's.** The view builds at the wall
  time of the shell's iteration (`line.md`, 2; `web/README.md`, 2.1) and says each
  time the domain's documents carry against it, both nanoseconds since
  the epoch. The same documents at the same time give the same words.
- **Five forms, the mockups' own** (`web/mockups/pages.js`, and the TUI's,
  `tui/whole-story/`). For a time T,
  E is the whole minutes from T to now, and 0 when T is ahead of now,
  as a machine's clock behind the engine's makes it:
  - *ago,* when something last happened: "just now" while E is 0;
    under 12 hours, E as a span with " ago" ("12 min ago", "1 h 28 min
    ago"); then by the calendar's day, "at HH:MM" on now's day,
    "yesterday" on the day before, the weekday ("Tue") within the six
    days before that, and earlier its date ("20 October"), with its
    year when not now's;
  - *span,* how long something has waited: "under 1 min" while E is
    0; "M min" under an hour; "H h M min" under a day, "H h" on the
    hour; then "N d", in whole days;
  - *clock,* when a moment of a conversation was: "HH:MM" on now's
    day, the weekday and "HH:MM" ("Tue 07:20") within the six days
    before it, and earlier its date and "HH:MM" ("20 October 07:20"),
    with its year when not now's;
  - *dated,* when a conversation was last active, said with its day:
    "today HH:MM" on now's day, else as *clock* ("Fri 18:40", "20
    October 07:20"), `TimeForm::Dated`;
  - *the day's rule,* a line before the first turn of each day a
    conversation's window holds, whose turns then say "HH:MM" alone:
    "Today · Thursday 22 October", "Yesterday · Wednesday 21 October",
    or the day's name and date ("Tuesday 20 October"), with its year
    when not now's.
- **The person's zone** sets the calendar's days and the hours the
  forms say: the browser's, in minutes east of universal time, an input
  of the shell's (`web/README.md`, 2.1), read at start, so that a change of
  daylight saving shows at the next load. The terminal has none: universal
  time. *ago* and *span* are differences, and it moves neither. The words
  name no zone, as the mockups' don't.
- **The tick.** With each build, the view keeps the instant at which a
  word it drew next changes: the next whole minute of an ago under 12
  hours or of a span under a day, a longer span's next whole day, and
  the next midnight while it drew a word of the calendar's (a later
  ago, a clock, a dated, a day's rule). The shell wakes at that instant, and the
  view builds again although the domain's shown count has not moved
  (`line.md`, 2; `web/README.md`, 2.1). A page that draws no time asks for no wake.
- **Where they are said:** a chats line's last activity or end, an ago
  (`pages/README.md`); a chat's turns and its end, clocks under the day's rules
  (`pages/README.md`); a card's wait, a span, in the inbox and on a chat's page; an
  update's arrival in the inbox, an ago (`pages/README.md`); the approval page's ask,
  an ago (`pages/README.md`, 2, *The approval page*); a remembered line's change, the
  day's rule's words (`pages/remembered.md`); on the TUI's launchpad a
  left-off row's last activity, dated, and a need's wait, an ago
  (`tui/pages/launchpad.md`, 4 and 5). The task pages and the
  accounts page say their moments as their sections do (`pages/accounts.md`, 2, `pages/goal.md`, 2, `pages/task.md`, 2
  ), in universal time, and so does a full deployment's earliest end
  (1, the pages not about the work).
- **Fixed in the tests.** The view world, the terminal's goldens and
  the gallery build at a fixed time: the fixtures' `NOW`, and each of
  the gallery's worlds its own (`web/gallery.md`, 2). A test of a form
  steps that time to either side of the bound it reads (`testing.md`).

## 7. Open questions

- **A line past `long_text`** (4): `standard` with no override takes a message of
  262,144 bytes (`../protocol/configuration/settings.md`, 7), past `long_text`, and a page
  of its `documents.rows` lines can pass `document_bytes`. Its read ends and says so
  (`pages/chat/conversation.md`, 2), where a line would otherwise draw whole. Only a deployment with larger
  models reaches it. Default: so; or `long_text` and `document_bytes` grow with such
  a profile, or the engine stops a page at a client's bound.
- **A new chat's first words** (4): the engine takes a task's spec up to
  `tasks.spec_bytes`, 131,072, and its composer stops at `words_bytes`, 8,192, which
  jig's `client/domain/contract/documents.md`, 1 gives for a spec, a message and an answer
  alike. Default: one bound for every composer.
- **An edit past `long_text`** (4): the client keeps its prefix and a count of
  the characters beyond it; the browser keeps the whole value only while that
  field stays mounted and its value count stays unchanged. Keeping the whole
  edit across a new drawing, or in the terminal, needs room beyond that bound.
  Default: the bounded prefix and count, with Send disabled while any lies beyond.
- **A project's home in the switcher** (`view.md`, 3, `forge/family.md`, 4): `me` does not say
  which repository is a project's home, so the first listed stands in
  for it, and a row with several repositories never reads "3
  repositories", the state `project-home`'s hand-made page draws. The
  project's own document does say its home, per project; reading it for
  each row is the candidate, or a fact of `me`'s, which is a protocol
  change and the user's. Default: the stand-in.
- **The rail's "N+"** (`pages/needs-you.md`, 1, *What the client keeps*): the standing document's
  `next` says more follows of needs or of updates alike, since a page holds
  every need and then the updates (`../../help/inbox.md`), so that a long list
  of updates, which no page of the browser's reads, says "N+" of a whole list of
  needs. It never says less than there is. Default: "N+", as that request has it.
- **The rename's opening and closing, as the mock draws them** (3; the request
  `ui-rename-focus` of 2026-11-06): the mock selects the name as the form opens,
  folds the menu as it closes and says "Renamed" in a toast. A view can neither
  select a field's words nor fold a disclosure (skein's `ui/web/README.md`, 12), and that
  request names none of the three. Default: the name unselected, the menu open with
  the focus on its summary, and no toast.
- **A time's moment on demand:** an ago ("12 min ago") has no title
  or hover saying its moment. Default: none; the view can give the time
  node its moment as a hint (`skein/docs/design/ui/README.md`, 3.2).
- **The switcher's name** (1): the look requests name it "Project:
  NAME" apart from its summary's words, NAME; skein's disclosure names
  its summary by its words alone (skein's `ui/web/elements.md`).
  Default: named NAME, until skein's disclosure carries a label of its
  own.
