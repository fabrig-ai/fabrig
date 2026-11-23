# Settings and Deployment

## 1. The page

- **Settings and Deployment** (`../../draft/surfaces.md`, sections 7 and 8):
  two frames of tabs, each tab a page in the rail's frame, in one `.col` as
  Home's (720 wide, gap 60; 32 on a phone). *How work runs*, the project's
  process, holds the tabs How work runs and Repositories, "How work runs"
  current in the rail; *Deployment*, the deployment's own, holds Accounts,
  Forges, Models, Workers, Members and Limits, "Deployment" current at the
  rail's foot (`../view-navigation.md`, 1). Builders is Workers on the page; its address and the
  code's names stay.
  - *Its head:* the kicker, "PROJECT · how work runs" on How work runs' tabs
    and "Deployment" on Deployment's, then, by tab, the level-1 heading and
    the lede:

    | Tab | Heading | Lede |
    |---|---|---|
    | How work runs | the way the project runs (2, *The page*) | that way's line, then how to change it (2, *The page*) |
    | Repositories | What PROJECT works on. | The repositories chats and goals in PROJECT read and change. |
    | Forges | Where your code lives. | Each forge temper is connected to, or setting up. Add one to bring its repositories to temper. |
    | Accounts | What agents run on. | The LLM accounts temper’s agents run on for you. A task keeps its account: while the account is limited, the task waits for it. |
    | Models | What the picker offers. | The models your accounts can run, by what pays for them. Each model can be left off the picker; a task keeps the model it has. |
    | Workers | Where agents work. | The machines temper’s agents run on, each with its places and whether it’s up. An owner adds one by its secret’s digest. |
    | Members | Who may do what. | Each project’s people and services, and the role each holds there. Who owns an area of a project is its Team’s. |
    | Limits | What this deployment holds. | The capacities temper’s engine runs within, as it reports them. Its owner sets them in its configuration. |

  - *Its tabs:* a navigation, links, the current one current, wrapping on a
    phone. How work runs' is named "How work runs": "How work runs", to
    `Settings { project }`, at `/projects/<project>/settings`; and
    "Repositories", to `Repositories { project }`, at
    `/projects/<project>/settings/repositories`. Deployment's is named
    "Deployment": "Accounts", to `Accounts`; "Forges", to `Forges`, these two
    with the marks `../view-navigation.md`, 1 gives them; "Models", to `Models { project }`, at
    `/projects/<project>/settings/models`; "Workers", to `Builders { project
    }`, at `/projects/<project>/settings/builders`; "Members", to `Members`,
    at `/deployment/members`; and "Limits", to `Limits`, at
    `/deployment/limits`. Models and Workers name the open project, as the
    rail does.
  - *`Forges` and `Accounts`* keep their addresses and paths, so that
    To begin's links and their tests hold, and so do
    `Models` and `Builders`. Whenever the
    page has a project, as the rail does (`../view-navigation.md`, 1), they draw inside Deployment's frame,
    with no level-1 heading of their own and no page bar; their rows follow
    the tabs. With no project they take the same column, sections and rows
    under their own level-1 heading, "Forges" or "Accounts", and that tab's
    lede, with no kicker and no tabs. Choosing a project on them changes
    nothing but the words naming the project.
  - *Sign out* is the last row of the `Accounts` tab: "Signed in as NAME",
    its login in small type, "What temper remembers about you", a link to
    `Remembered` (`remembered.md`), "Telling you", a link to `Told`
    (`told.md`), and "Sign out"; with "Sign out did not reach
    the engine." beside it as `../line.md`, 6 says. A browser draws this row at 760 px and
    under alone, where the rail has no foot; wider, "Sign out" is the person's
    menu's (`../view-navigation.md`, 1). With no project, the top bar has "Sign out" (`../view.md`, 1).
  - *How work runs* is 5.13.
  - *A repository being made,* on the `Repositories` tab, is a row among them,
    by the forge's row's `makings` whose `into` names this project (`../forge/records.md`), the
    request `connect-flows`: ORG/NAME, then "Copying · FORGE", or "Making ·
    FORGE" for an empty one. A failed one: "Copy failed · FORGE", or "Make failed
    · FORGE", then its own line, "Forgejo said: “WORDS”", with "Copy again", which
    opens the sheet (`../forge/family.md`, 4) on Copy public with its address filled, or, for a name
    taken, "Adopt it", the sheet on Adopt with its row picked, and "Dismiss"
    (`DismissMaking`, `../forge/records.md`).
  - *Repositories:* a section whose head holds the title "Repositories · N"
    and "Add repositories", to `Adopt { project }` (`../forge/family.md`, 4); then a line per
    repository of the project, from the `forges` watch's rows whose
    `project` matches: `owner/name`, in the browser the repository's link
    where it has an address (`../forge/deployment.md`, *A repository's link*), and in
    the line's time place its forge's name. The line itself is no link. The
    playground's project shows its repository from the playground's read
    (`../forge/family.md`, 7), plain. With none, a quiet band:
    the tag "No repository yet", "Chats in PROJECT talk and plan, but change
    no code until a repository is in.", and "Add repositories".
  - *The rows,* on `Forges` and `Accounts`, are the greenfield mock's language
    applied to what the client has, since its tabs draw neither: every fact,
    act, form and state of `../forge/family.md`, 7, `accounts-words.md`, 1 and `../forge/deployment.md` stays, and only its
    look is this. Each is a folded disclosure in `.lines`' place: a hairline
    under it, 14 px 4 px, no card and no shadow. Its summary is its name in
    ink, 15 px, its second line in `small`, 13.5 px, muted, and "⌄" at the far
    right, faint, as `.switcher > summary::after`. Opened, its content is 14 px
    under the summary, indented nothing, its facts `.setting` rows. A section's
    head is as Repositories': `.section-title`, and the section's act at its
    right, a `.btn.small`. Buttons are `.btn`, `.btn.primary`, `.btn.quiet` and
    `.btn.small`; fields `.text`. The tabs' marks stay.
  - *The `Forges` tab:* a section, "Your forges · N", with "+ Connect a forge" in
    its head, not while the playground runs. Until the watch answers, "Your
    forges" alone, claiming nothing. With none, and the playground not running, a
    quiet band: the tag "No forge yet", "No forge of yours yet.", and the
    primary "Connect a forge". Then a row per forge (`../forge/deployment.md`). While the
    playground is read, a section of its own holding its forge's row.
  - *The `Accounts` tab:* a section, "Your accounts · N", with "+ Connect a
    model" in its head, as `accounts.md`, 1's accounts page has its act and condition. Until
    the document is read, "Your accounts" alone. With none, a quiet band: the
    tag "No account yet", "No model of yours yet.", and, not while the playground is read, the
    primary "Connect a model". Then a row per account, its summary its line (`accounts-words.md`, 1, An account's
    line). "Deployment accounts", for an owner, is a section of the same rows;
    the playground's account, a section of its own; then *Sign out*.
  - *The `Models` tab* (the look request `model-picker`, its 07;
    `../../draft/model-catalogue.md`, sections 3, 4 and 6) draws the project's
    offer in `me` and works nothing out, so that it holds no model the offer
    leaves out:
    - *A section per provider* `me` offers, in `me`'s order, its title the
      provider's word, "Claude" for `anthropic`, as the picker's headings say it.
      Under it, in `small`, what pays and where its models came from: "Your Claude
      subscription.", "Your ChatGPT plan." or "Your API key.", from the person's
      usable accounts of that provider in the accounts document, one clause each,
      then one clause on its list, from those accounts' `list`s (jig's
      `client/domain/contract/documents.md`, 1), PROVIDER its word as the section's title:
      - where one's last attempt failed, "Couldn't read PROVIDER's list · 2 hours ago.",
        by the newest `tried` of those failed;
      - else where one has a good read, "Read from PROVIDER 2 hours ago.", by the
        newest `read`;
      - else "These come with temper."

      Where any of them has a `list`, "Read again" follows the clause, in
      `.temper-settings-small`: a press asks jig's `ReadLists` with the section's
      provider, says "Reading…" and is off until that answer and the `accounts` watch
      has moved past what it showed, and a refusal shows under the clause in the
      words of an account act's refusal.
    - *A provider with no model offered* has its section's title and its line alone,
      as the picker's: "Connect a plan or key ›", to `ConnectModel`, with no account,
      or the account's own fix, such as "Sign in again ›", to `Accounts`. It has no
      row and no switch, the stock's models included. So with one account the page
      holds that provider's models alone.
    - *A row per offered model,* `.setting`, in the offer's order: its shown name;
      in `small`, "NAME · EFFORT", its name and default effort, with no effort for
      a model that declares none, then " · the default" on the model the chat
      template's `resolved_default` names, " · added by you" on one a person
      added, " · estimated prices" on one whose prices are estimated, and " · new" on
      one whose `new_until` is later than now; at its
      right, "On" or "Off", whether the picker shows it. An owner in the deployment
      has a switch beside the word, named "SHOWN in the picker": a press asks jig's
      `ChangeModel` (jig's `client/domain/contract/requests.md`, 2), the switch holding
      its old state until the answer, and a refusal shows under the row in the
      words of a change's refusal. Anyone else has the word alone.
    - *How many were left out,* in `.facts` under the sections, where `me` says
      the catalogue left any out: "N more models don't fit the catalogue." (jig's
      `client/domain/contract/documents.md`, 1).
    - *"Add a model by name",* a section, for an owner in the deployment: a
      provider's select, of the providers `me` offers but the playground's; a
      field whose accessible name is "The model’s name", drawn by its placeholder
      "model-name" alone; "Add", the three on one row at every width; and, in
      `small`, "For a model the list doesn’t show. It runs through the account you
      have with that provider." The name's shape is checked here as the engine
      checks it, 1 to 128 bytes of letters, digits, `-`, `.`, `_` and `:`, and "Add"
      is off until it holds; a name of that provider's in the offer says
      "PROVIDER already has that model." A press asks `ChangeModel`. The engine
      makes the model (jig's `domain/engine/settings-catalogue.md`, 1), and the field empties;
      its row then reads "added by you". A refusal shows under the form.
    - *The last line,* in `.facts`: "To change a default or a role’s models, ask
      temper.", its last two words a link to `Ask { project }`.
    - *In the terminal,* the same sections as lines, a row's word and name on one line,
      and the switch and the form as the terminal has them on How work runs (2).
  - *Workers,* `Builders { project }`, is the workers' settings, each with
    its places and whether it is up, and an owner's acts on them
    (`workers.md`). There is no capacity graph.
  - *Members* (jig's `domain/people/README.md`, section 4, and `jig/docs/design/domain/people/acts.md`, section 2) reads jig's
    `Read::Project` for each project `me` lists, in `me`'s order, one read at a
    time, while the tab is open, and again when `me`'s projects change. A
    section a project, "PROJECT · N", N its members as the document lists
    them; then a line per member, as Repositories' are, no link and no act: its
    name, or its login with none, or "Party N" with neither; in `small`,
    "person" or "service", by its `kind`; at the line's `.when`, its role's
    word, "owner", "maintainer", "member", "observer", or "role N" for a
    policy role. A cut document says so, as `Accounts` does. Until a project's
    read answers, its title alone, claiming nothing; a refused read, its title
    and "Read by the project’s members.". Before the projects, a section "The
    deployment", one line by `me`'s `deployment_role`: "You are its owner.";
    "Your deployment role: ROLE.", ROLE its role's word as above, for another;
    "You hold no role in it." with none. Under them, `.facts`: "Who owns an
    area is Team’s, on each project.". The deployment's own members are
    `../README.md`, section 5's.
  - *Limits* (`../../protocol/configuration/limits.md`) reads jig's
    `deployment` document, watched as Workers watches it, by a person with a
    role in the deployment. A section, "Capacities", a line each, as
    Repositories' are, no link and no act, its name and at its `.when` "USED
    of LIMIT": "Tasks", "Workers", "Attempts", with " · N waiting" while
    `waiting_attempts` is more than zero, "Proposals", and "The store", its
    fill against its mark in megabytes, one decimal under ten, "4.1 MB of 64 MB"; then
    a section "By project", a line per `projects` row, "PROJECT" and "USED
    of LIMIT tasks", the name from `me`, "Project N" for one `me` doesn't
    list. Under them, `.facts`: "The deployment’s owner sets these in its
    configuration; temper reads them at start.". Then, once the document
    carries the store's census (jig's `client/domain/contract/documents.md`,
    1), a section "What the store holds", its lede "As of commit N; writes
    since commit S, the store's last snapshot.", and a line per keyspace,
    as Capacities' are, most rows first, then by prefix: its name, or with
    none its prefix's bytes in hex, two digits each, space-separated
    ("02 0c"), and at its `.when` "ROWS rows · SIZE · WRITES writes",
    ROWS and WRITES (its puts and erases together) in digits grouped by
    thousands ("993,441"), SIZE its keys' and values' bytes together in
    megabytes as the fill's are, "under 0.1 MB" below that; then
    " · layout H" for its one head, or " · layouts H, H" for several, in
    ascending order, "none" in place of H for the head of values with no
    `u16` (" · layouts none, 3"), and nothing for a keyspace with no head.
    With no census, no such section. With no role, one line,
    "Limits are seen by the deployment’s owners.", and no section; until the
    document is read, the title "Limits" alone. Read-only for everyone; the
    configuration's own limits and an owner's change of them are `../README.md`, section 5
    's.
  - *The flows' paths:* `ConnectForge` and `ConnectModel` share the
    welcome’s cards and panels in Deployment’s column. With a project open,
    the browser draws Deployment / Forges / Connect a forge, or
    Deployment / Accounts / Connect a model. Links go to Deployment's first
    tab, `Accounts`, and the tab; the last part is plain. Inside
    a way, the path also links back to the choices, with the way’s name
    last. The quiet Other ways or Other forges stays beside the main act.
    On a phone the path’s “‹” links back to the choices inside a way, or
    to the tab at the choices. With no project there is no path, and the
    terminal uses its rail’s Deployment. `Adopt` draws the Add repositories
    sheet over Repositories, whose path remains the tab’s (`../forge/family.md`, 4).
    A flow’s completion returns to its tab; Start over on a forge cut
    short returns to Forges.
  - "How work runs" is current in the rail on its two tabs and on `Adopt`;
    "Deployment" on its six tabs, on `ConnectForge` and on `ConnectModel`;
    neither on `Ask` or an Ask temper chat. A project's Delete, when it is
    built, is on How work runs (`../testing.md`).
- **A role's or a rule's page:** `Role { project, charter }` and `Rule {
  project, kind, number }`, what a stop's card opens, each a page in
  How work runs' frame with its How work runs tab current, reading the forge's
  `View` (`../forge/records.md`) of `charter N` or `rule K/N` in the project, the same
  read and the same card a reply's view draws (`chat/conversation.md`, 3): its heading, level
  1, the card's `name` with a capital ("The reviewer", "The project's
  rule 1"); its `who` under it, muted; then its `facts`, each label
  small, faint and in mono over against its value, as a stop's card's
  are. On a phone, "‹", a link named "How work runs" to `Settings {
  project }`, alone before the heading. Until the read answers the page
  says "Reading NAME…" (NAME "role N" or "rule K/N"); a `not_drawn`
  answer, a refusal or a failed read says, muted, in a line, that it
  can't be read now, and the address. It has no act: a role's or a
  rule's change is a proposal, Ask temper's. The terminal prints its
  heading, `who` and facts, as any page.
- **Deleting a project** (jig's `domain/people/acts.md`, 2): How work
  runs (`Settings { project }`) ends, after its other parts, with
  "Delete project", a plain button, drawn while `me`'s role on the
  project is owner, the role Rename reads (`../view-navigation.md`, 1), and for no
  one else. Pressed, it opens the sheet that names the project
  (`../root.md`, 1.6, *The sheet*), and sends nothing; the sheet's
  "Delete project" sends jig's `RemoveProject` by
  `shapes::remove_project` (jig's `client/domain/client/helpers.md`;
  `jig/docs/design/client/domain/contract/requests.md`, 2), one keyed request. Answered
  removed, the sheet goes, the project leaves `me`, and with it the
  navigation and every list (`../view-navigation.md`, 1), and the client goes to
  `AllHome`, All projects (`home.md`, 1), where, with no project left,
  the column "No project yet" draws (`welcome.md`, 1). Refused, the sheet
  stays, saying why above its acts in jig's words: a project with live
  tasks, "This project has live tasks; end them first: T12, T15", each
  number the task's, and ", and 2 more" past jig's 32; a refusal of the
  role, which a person who was owner when the page drew can meet. Its
  work is jig's to erase; its repositories stay on their forges, released.
  The line terminal draws the same button and sheet, as it draws this
  page; the TUI offers no Delete. The settings' other parts are
  `../README.md`, section 5's.

## 2. How work runs, and a proposal of settings

### 2.1 How work runs

Moved to [`settings-workflow.md`](settings-workflow.md), section 1.

### 2.2 Proposals of settings

Moved to [`settings-proposals.md`](settings-proposals.md), section 1.

## 3. Open questions

- **Settings, Models after landing** (`README.md`): temper/ui compares page 07 of
  `model-picker` with the shipped page at 1440, 1280, 1000 and 420 px,
  including the named states and screenshots (`../web/look-requests.md`,
  section 4). Default: the client world's switches and adding across
  reloads, reads, failures, retries, retained names and unchanged reads;
  view tables and gallery states for the read ages and expiring new tags;
  a fake-provider row through a worker and agent. Browser and real-provider
  checks remain after landing.
  The comparison includes Provider's chevron, the single-row add form
  and its section headings. Settings' shared H1 and the open project
  menu row keep their frame's design, as the repair request leaves them.

- **Deployment's later parts** (1;
  `../../draft/surfaces.md`, section 8): Members' deployment roster, which no
  jig read lists (jig's `domain/people/README.md`, section 3); and Limits read
  from the configuration's profile and overrides, an owner changing them
  (`../../protocol/configuration/limits.md`), which have no read and no
  act. Default: Members lists each project's members and the person's own
  deployment role; Limits draws the `deployment` document's capacities,
  read-only for everyone.
- **The person's login over 760 px** (1, *Sign out*; `../view-navigation.md`, 1; the request
  `ui-owner-menu`): the sheet hides the `Accounts` tab's row whole, so a browser
  that wide reads the person's name on the rail's foot, and their login on no tab
  of `Settings`. Candidate: the sheet hides the row's button alone, its words
  staying. Default: the whole row.
