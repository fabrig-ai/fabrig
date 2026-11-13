# Settings and Deployment

## 1. The page

- **Settings and Deployment** (`../../draft/surfaces.md`, sections 7 and 8):
  two frames of tabs, each tab a page in the rail's frame, in one `.col` as
  Home's (720 wide, gap 60; 32 on a phone). *How work runs*, the project's
  process, holds the tabs How work runs and Repositories, "How work runs"
  current in the rail; *Deployment*, the deployment's own, holds Accounts,
  Forges, Models, Workers, Members and Limits, "Deployment" current at the
  rail's foot (`../view.md`, 3). Builders is Workers on the page; its address and the
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
    | Workers | Where agents work. | The machines temper’s agents run on, each with its slots and how many are in use. |
    | Members | Who may do what. | Each project’s people and services, and the role each holds there. Who owns an area of a project is its Team’s. |
    | Limits | What this deployment holds. | The capacities temper’s engine runs within, as it reports them. Its owner sets them in its configuration. |

  - *Its tabs:* a navigation, links, the current one current, wrapping on a
    phone. How work runs' is named "How work runs": "How work runs", to
    `Settings { project }`, at `/projects/<project>/settings`; and
    "Repositories", to `Repositories { project }`, at
    `/projects/<project>/settings/repositories`. Deployment's is named
    "Deployment": "Accounts", to `Accounts`; "Forges", to `Forges`, these two
    with the marks `../view.md`, 3 gives them; "Models", to `Models { project }`, at
    `/projects/<project>/settings/models`; "Workers", to `Builders { project
    }`, at `/projects/<project>/settings/builders`; "Members", to `Members`,
    at `/deployment/members`; and "Limits", to `Limits`, at
    `/deployment/limits`. Models and Workers name the open project, as the
    rail does.
  - *`Forges` and `Accounts`* keep their addresses and paths, so that a
    connect's `back`, To begin's links and their tests hold, and so do
    `Models` and `Builders`. Whenever the
    page has a project, as the rail does (`../view.md`, 3), they draw inside Deployment's frame,
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
    menu's (`../view.md`, 3). With no project, the top bar has "Sign out" (`../view.md`, 1).
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
    act, form and state of `../forge/family.md`, 7, `accounts.md`, 2 and `../forge/deployment.md` stays, and only its
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
    primary "Connect a model". Then a row per account, its summary its line (`accounts.md`, 2, An account's
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
      makes the model (jig's `domain/engine/settings.md`, 11), and the field empties;
      its row then reads "added by you". A refusal shows under the form.
    - *The last line,* in `.facts`: "To change a default or a role’s models, ask
      temper.", its last two words a link to `Ask { project }`.
    - *In the terminal,* the same sections as lines, a row's word and name on one line,
      and the switch and the form as the terminal has them on How work runs (2).
  - *Workers,* `Builders { project }`, reads jig's `deployment` document, watched while the tab is open
    by a person with a role in the deployment, as `Accounts` watches it for an
    owner. It draws `Deployment.hosts`, as far as the document's configured rows
    hold them. A section, "Workers · N", N its rows; then a line per host, as
    Repositories' are, no link and no act: the host's name, or "Worker
    CHANNEL" with none; in `small`, "2 slots · 1 in use" ("1 slot" for one); in
    the line's `.when`, "Not connected" while it is not connected, else
    "Draining" while it drains, else "Connected". Under them, `.facts`: "Each
    agent’s role names which workers it runs on. Ask temper to change it."
    With no host connected, `.empty`, "No worker is connected.",
    above any lines left. A cut document says so, as `Accounts` does. With no
    role in the deployment, one line, "Workers are seen by the deployment’s
    owners.", and no section. Until the document is read, the title "Workers"
    alone, claiming nothing. There is no capacity graph. The word on the page
    is "worker"; the code's stays host, and the page's `Builders`.
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
    ascending order, "none" for no head. With no census, no such section. With no role, one line,
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
  project is owner, the role Rename reads (`../view.md`, 3), and for no
  one else. Pressed, it opens the sheet that names the project
  (`../README.md`, 3.6, *The sheet*), and sends nothing; the sheet's
  "Delete project" sends jig's `RemoveProject` by
  `shapes::remove_project` (jig's `client/domain/client/helpers.md`;
  `jig/docs/design/client/domain/contract/requests.md`, 2), one keyed request. Answered
  removed, the sheet goes, the project leaves `me`, and with it the
  navigation and every list (`../view.md`, 3), and the client goes to
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

The settings, their revisions and their proposals are
`../../protocol/configuration/settings.md`, and jig's
`domain/engine/settings.md`; the draft is `../../draft/configuring.md`,
section 5.

Ask temper is reached from the rail and tab bar (`../view.md`, 3), and from this page's
"Ask temper to change this".

- **The page,** `Settings { project }` (1), is How work runs,
  the project's process (`../../draft/surfaces.md`, section 7), its look the
  whole story's `web/whole-story/how.html` and `pair/how.html`
  (`../web/look.md`, *How work runs' look*). It draws what the engine
  holds (`../../draft/whole-story.execution.md`, section 2). Under the frame's kicker, in
  order, its sections 44 apart, each a section title and its rows:
  - *Its heading and lede,* in the frame's place for them (1),
    the words saved with the settings
    (`../../draft/whole-story.execution.md`, P3e): the heading is the `ways`
    read's `headline` of the way the project runs ("You approve every
    change"); where it runs no stock way, the project's words' `headline`
    the read carries (`../../protocol/configuration/settings.md`, 2, kind 10),
    else "A way of its own". The project's words name the way and its
    chief departure, as a rule by path makes one, "Draft first, with the
    protocol core asking you": Ask temper writes them with the proposal
    that leaves the project on no stock way (`../../domain/agent/ask-temper.md`,
    2, *Words with the settings*), and the page derives none. The lede is "PROJECT runs the stock way
    NAME." and that way's `line`; where it runs none, the project's words'
    `lede`, else "PROJECT runs settings of its own, drawn below."; then
    "Ask temper to change any of it, in words; an owner accepts its
    proposal, and new work follows it." Until the read answers, and once
    its watch has ended, the heading is "How work runs in PROJECT." and
    the lede that last sentence alone. A way's words and the project's are
    the read's, never written here.
  - *"Ask temper to change this,"* under the lede, a primary link to `Ask {
    project }`, while `me` lists Ask temper's template among the project's
    (`ask-temper.md`, 1, *Ask temper's page*); then the frame's tabs.
  - *The path of a change,* a section: its title, and at its head's right,
    small and muted, "the way: NAME", NAME the current way's, or "its own"
    where it runs none. Under it the ways' tabs as Ask temper's page draws
    them (`README.md`): a tab a way of the read, in its order, then "Yours" where
    the project runs no stock way, the current one's "now", the shown one
    selected, and the line "From more of you to more of your agents"; the
    shown way's `line`; its path, the drawing Ask temper's page draws
    (`chat/conversation.md`, 3), its stops pressed, laid out by the box's width, the legend
    under it as Ask temper's page has it; while a stop is open, its card,
    as Ask temper's page draws a stop's card, its caret across; then
    the shown way's act at the section's right, "Use this" or "Keep it"
    (*A way's acts,* below), and a refusal under it. The shown tab and the
    open stop are the page's state, as on Ask temper's page. Until the read
    answers the section holds its title alone; once its watch has ended,
    "The ways can't be read now.", muted.
  - *What reaches you,* the project's classes, kind 8, in words
    (`../../draft/surfaces.md`, section 7; `../../draft/what-reaches-you.md`,
    sections 2 and 3; `../../domain/forge/attention.md`, 6), the section "Limits"
    focuses (`README.md`, 2, *Away*): a section "What reaches you", at its head's
    right, small and muted, "on PROJECT", or "on PROJECT; each project has
    its own" while `me` lists two projects or more. Under its head, once
    kind 8 is read, its columns side by side, a box each, in this order;
    each opens with its chip, its words and who acts' mark as the path
    draws it (`chat/conversation.md`, 3), then a line a subject, then a small muted line:
    - "Agents decide, then tell you", an agent's star: the subjects at
      `decide`; "Listed in Decided for you for 24 hours, to overturn.";
    - "Asks you", a person's ring: the subjects that ask even when away,
      then those Away may decide; then what Away changes: "While you’re
      away, they are decided with the recommended choice and listed." where
      Away may decide every one, "They ask even while you’re away." where
      it decides none, and otherwise "While you’re away, all but OBJECTS
      are decided with the recommended choice and listed.", OBJECTS those
      that ask even when away, each as Stop asking names it (`needs-you.md`, 1),
      joined by ", ";
    - "Always waits for you", a person's ring: the fixed kinds, in the
      kit's order (`Kind::ALL`); "Away or not."

    A subject's line is its words: `wording` "wording and names inside a
    spec", `retry` "retrying or splitting a task", `implementation`
    "implementation choices a spec leaves open", `tests` "test-only
    changes", `dependency` "a new dependency", `departure` "a departure
    from a landed design", a set "a change to NAME", its `name`; `money`
    "money past a budget", `irreversible` "what can’t be undone: rewriting
    history, a force-push, deleting unlanded work", `rules` "changes to
    this page", `other` "anything an agent can’t place"; an item with
    words, its subject's line then " under “WORDS”". Within a column the
    subjects stand in the settings order, the proposal card's (*The card
    in a chat,* below): the movable kinds in the kit's order, each at its
    own item's class, or asking even when away with none; then the
    table's items with words and every item of a set, in the table's
    order; then each set with no item of its own, which asks even when
    away. So each subject has one line, in one column, and a column with
    no line isn't drawn: on You approve's table "Agents decide, then tell
    you" isn't. Until kind 8 is read, and after a lasting refusal, the
    section holds its head and its rows alone. Then its rows (*Its rows,*
    below): "Questions a goal can answer from its plan", kind 6, and
    "Away until", while the switch is shown (`README.md`, 2, *Away*). The section has
    no control of kind 8: a class changes by proposal, Ask temper's from
    "Ask temper to change this", or a need's Stop asking, which an owner
    may apply (`needs-you.md`, 1).
  - *Rules by path,* while the `ways` read gives a row of them
    (`../../draft/surfaces.md`, section 7; `../../draft/what-reaches-you.md`,
    section 7): a section "Rules by path", at its head's right "+ A rule",
    a link to `Ask { project }` while "Ask temper to change this" is drawn,
    since a rule changes by proposal as every setting does; then a table,
    its rows the read's `rules` (`../../domain/forge/change/branch-path.md`,
    *Rules by path*), in order, under a header row "Paths", "Review",
    "Sign-off" and "Lands", "Review" left out where no row has one. A
    row's paths in the code face; its review in words; its sign-off a
    chip in a person's ring, as What reaches you's are, its words the
    row's; an empty review or sign-off "–", muted; its lands in words.
    Under the table the read's `note`, small and muted, where it has one.
    The rows' words are the read's, never composed here. On a phone the
    table keeps its columns and scrolls across within the column. Until
    the read answers, once its watch has ended and while it gives no row,
    the section isn't drawn.
  - *What follows a landing,* the project's follows and its tiers after
    landing (`../../draft/surfaces.md`, section 7;
    `../../domain/forge/follows.md`, sections 2 and 7;
    `../../domain/after-landing.md`, sections 2 and 7), after What reaches
    you and after Rules by path once it is drawn: a section "What follows a
    landing", at its head's right, small and muted, "code triggers, agents
    judge"; under it its boxes side by side, a box a follow, then the
    tiers' box, each its title, its words, small, and its chips, a chip
    who acts' mark and its words, as What reaches you's chips are, code's
    mark the path's (`chat/conversation.md`, 3):
    - *A follow,* each of the forge's `follows` document's, in its order
      (`../../domain/forge/follows.md`, section 7): the title "TARGET
      follows SOURCE", the repositories as the document names them; the
      words "Each landing on SOURCE's BRANCH that touches PATHS makes a
      change in TARGET's TBRANCH, owned by OWNER.", the branches as the
      document names them, PATHS the follow's paths joined by ", " and a
      last " or ", and " that touches PATHS" left out where it names
      none; then "Each waits for OWNER's word." under `start` `owner`, or
      "Each starts once the one before it has landed." under `code`; then
      "Its check: WORKFLOW.", its check's workflow. OWNER is the owner's
      name among the project's members, else their login, as Team names a
      person (`team.md`, 1), and "you" and "your word" where the owner is
      the reader; for a standing role, its owner's `task`, the role's
      name as Team's Standing roles names it, its task's title from the
      roles' watch (`team.md`, 1), else "task N"; where the members
      aren't read or don't hold the owner, ", owned by OWNER" is left out
      and the start's sentence says "its owner's word". Then a line, small and muted, "LAG · PARITY", as
      `temper ctl follow list` words them (`../../domain/forge/follows.md`,
      section 7; the client's `forge::follows`): "caught up" or "K
      behind", then "check passes at AFTER12", "K failing at AFTER12",
      "many failing at AFTER12" or "no check yet"; and where the follow
      is stopped, "Stopped: WORDS." under it, its words as the document
      gives them. Its chips: "code hears the landing", code's mark; "an
      agent ports", an agent's star; "code checks", code's mark.
    - *The tiers,* while the path the project runs, the current way's or,
      where it runs none, the `ways` read's `own`, holds a stop that opens a
      tier and verifies what landed (`../../domain/forge/change/branch-path.md`;
      the path's `verify` stops): the title "Tiers on BRANCH", BRANCH the
      path's branch; the words "Code runs each after landings on BRANCH:
      TIERS. A red reaches the goals whose changes landed in it, and their
      agents repair it.", TIERS each such stop's tier, in the path's order,
      by its card's name, then " (WORDS)" where its card says what it
      checks ("Checks"), joined by "; "; its chips: "code runs", code's
      mark; "an agent repairs", an agent's star.

    Until the `follows` document answers, and once its watch has ended or
    been refused, no follow's box is drawn; until the `ways` read answers,
    no tiers' box. A section with no box isn't drawn. Nothing in it is a
    control: a follow changes by a proposal of kind 12, or by its owner's
    acts in `temper ctl` (`../../domain/forge/follows.md`, section 5).
  - *Roles,* every agent the project's work runs
    (`../../draft/surfaces.md`, section 7): a section "Roles", at its head's
    right, small and muted, "every agent PROJECT runs"; a row a role of
    the view `roles` (`chat/conversation.md`, 3; `../../domain/forge/change/branch-path.md`, *The project's
    roles*), in its order: a disclosure (`confirm.rs`' and the frame's
    kind) whose summary is the role's `name`, its `who` in small muted
    words, when it runs and what it does as the view says them ("For
    each chat: talks with you and asks for the changes"), and at its
    right its "Model" fact's value; opened, its "Can"
    and "Asked to" facts as terms, then "Read its instructions ›", a link
    to the role's page, `Role { project, charter }` (`README.md`). Every row
    closed as the page opens. The page reads the view as the role's page
    reads `charter N`: once as How work runs opens, again when the
    session's `settings` watch names a newer revision; until it answers,
    or where it is refused or `not_drawn`, the section isn't drawn. What
    a role does is its charter's words
    (`../../protocol/configuration/settings.md`, 2, kind 9), or its
    instructions' first sentence where none are kept, which the root
    puts in `who`: the page draws `who` as it comes and composes
    nothing.
  - *Budgets,* while one is set (`../../draft/surfaces.md`, sections 7 and
    12): each of the project's standing roles' budget per period
    (`../../domain/agent/standing.md`, section 5; jig's
    `domain/tasks/standing.md`, section 4), then what a goal is given when
    it starts. A section "Budgets", at its head's right, small and muted,
    the project's open period as the project's document gives it (its
    `current_period`, jig's `client/domain/contract/documents.md`, 1):
    "a week" where its `next` is seven days after its `began`, else "each
    period"; then " · renews DAY" where `next` is known, DAY the day of the
    week `next` falls on in the reader's time, as the client says a day
    ("Tuesday"); nothing at its right while the document isn't read or
    holds no open period. Then its rows:
    - *a standing role's,* for each live task of the project labelled 3
      (`../../domain/agent/standing.md`, section 2), in number order, at
      most the watch's window: its title, its spec's first line as jig's
      chip gives it; in small type "a standing role · N left", N what its
      budget has left (`left`); and at its right its budget, the amount it
      was made with or an amendment set (`numbers.budget`), a whole number
      as the Goals row says one;
    - *"Goals,"* while kind 7 has Goals shown and its goal's template, the
      one `way` names or else the first tracked one in `me`, gives an
      amount or a cap (its `budget`): in small type "each given its own
      when it starts", and at its right the amount, a whole number as a
      chat's bar says a budget, or "up to N" for a cap. A budget a goal's
      starter gives, or one only its policy bounds, has no figure, and so
      no row.

    No unit follows a figure: the deployment's unit has no word yet (jig's
    `domain/authority.md`, section 12).
  - *At its foot,* *What the project shows* and, while Goals is shown,
    *Ways of working*, below: the switches stay as overrides of what each
    page's own rule shows (`../../draft/surfaces.md`, section 12).

  A row is a label, small words and a control at its right, a phone
  stacking them. On a phone the path runs down the page, as on Ask
  temper's page (`chat/conversation.md`, 3, *Its layout*). Nothing disabled is drawn: a
  mechanism not built has no row until it is.
- **Its parts** are one pure function, `pages::how::parts`, of the facts
  `pages::how::facts` gathers (Ask temper's template in `me`, the `ways`
  read's state, current way and rules by path, kind 7's Goals and its goal template's
  budget, kind 8 read, `me` listing two projects or more, the project's
  open period from its document, and its live standing roles), which the
  web and the terminal share (`../../draft/surfaces.md`, section 12): the
  heading's way, "Ask temper to change this", the path section's state,
  What reaches you's columns and its head's words, whether Rules by path is drawn and with "+ A rule" and its "Review" column, Roles' state, and
  Budgets' head and rows. What reaches you's lines are a second pure function,
  `pages::how::reaches`, of the project's table: its three columns, each
  subject placed once (*What reaches you,* above). What follows a
  landing's boxes are a third, `pages::how::follows`, of the `follows`
  document as read, the path the project runs, the project's members and
  the reader: a box a follow, in the document's order, then the tiers'
  box, or none (*What follows a landing,* above).
- **Its rows:**
  - *What the project shows,* live, kind 7: a row each, the state's word then
    a switch:

    | Label | Small | Words | Member |
    |---|---|---|---|
    | Chats can write drafts | A chat keeps a document beside it, in docs/design/draft/, that a goal can start from. | On / Off | `drafts` |
    | Goals | Bigger work from a draft: a plan, tasks, landing. | Shown / Hidden | `goals` |
    | Changes | The landing queue, as Work's filter. | Shown / Hidden | `changes` |

    The Away switch is no row of it: the classes show it (`README.md`, 2, *Away*).

  - *Ways of working,* live, kind 7's `way`, only while Goals is shown: a row
    per template of the project with `tracked` set, in number order, named and
    worded as the Goals page words them, one table for both: its name; its line, then " Used by
    N." or " Used by none yet." where a goal's template is recorded; a radio,
    " The default". A stored `way` of 0, or of a template no longer offered,
    reads as the first. A goal's start preselects it.
  - *What reaches you's rows,* under its columns, each a `.setting`:
    - "Questions a goal can answer from its plan", a select, live, kind 6:
      "Asked of me too" (`up`), "Not asked: each agent decides alone"
      (`none`), with no foreman (`../../domain/forge/change/questions.md`, 1);
    - "Away until", a select, small "When it isn’t turned off by hand.",
      options "08:00, the end of quiet hours", "I turn it off", while the
      switch is shown (`README.md`, 2, *Away*): each writes through `SetAway`, `on` as
      the row stands, the first sending the next 08:00 and the second none.
      They work while off: the row keeps them. With no row it reads 08:00.
- **What the client holds:** jig's `settings` watch, for the session, as the
  `dismissed` and `forges` watches are held, Home reading the same one. It reads
  kinds 7 and 8 of the open project as the project opens, kind 8 also of the
  card *Needs you* shows where that card's project is another, and kind 6 on
  the tab, through the forge's reads in that project's scope (`../forge/served.md`
  ), and again whenever the document's line for that setting says it
  changed and when the link comes back. Kind 8 decodes to the kit's table
  (`temper-protocol-reaches`, `classes_from`); one `Classes::check` refuses
  reads as a lasting refusal. A transient read failure (busy, unreachable, local
  read capacity, or sign-in still settling) retries after one second, doubling
  to at most 30 seconds; a successful answer resets the wait. Kinds 7 and 8
  retry while their project is open or its card shown, kind 6 while the tab is
  open; leaving that scope drops its retry. Until first read, or after a lasting refusal, the optional entries
  read as hidden, with `../view.md`, 3's exceptions for a current page and a project holding a
  goal; `drafts_offered` is false, with an Ask temper chat's exception (`README.md`).
  The tab's live controls draw the stock, disabled; a value of none reads as
  the stock (`../../protocol/configuration/stock.md`).
  How work runs also holds, while it is open: the forge's `follows`
  document of the project (`../../domain/forge/follows.md`, section 7;
  `forge::follows::watch`), watched as the `ways` read is and dropped on
  leaving; and a watch of the project's live standing roles, jig's task
  query of the project's tasks labelled 3 in number order within the
  client's window of rows (jig's `client/domain/contract/README.md`, 4.1),
  as Chats and Goals watch theirs, dropped on leaving. Its members and
  open period are the project's document that every project page reads
  (`team.md`, 1; jig's `Read::Project`), read again on each `me`
  delivery; How work runs reads no other copy of it.
- **Its acts:** each live control sends at once jig's `ChangeSettings` with one
  item, running work staying, against the newest revision the person has, the
  one exception to the pages sending proposals alone, beside Stop asking's
  "Apply" (`needs-you.md`, 1), which sends kind 8, one table, the same way. The
  control is disabled while it is out, and so are the others of its kind, since
  kind 7 is one record; the value asked shows meanwhile. A refusal is said under its row in jig's
  words, `full` and a person who is no owner too, the control back at its
  value. A `stale` refusal is sent once more against the revision it names.
  Answered, the client takes the value at once and reads it again.
- **A way's acts** (`../../draft/stock-ways.md`, section 5;
  `../../protocol/configuration/settings.md`, 8): Ask temper's page and How work
  runs each watch the project's `ways` (`../forge/records.md`) while open, its state the
  page's. *"Use this"* on a
  way the project doesn't run sends at once jig's `ChangeSettings` of that
  way's `items`, running work staying, against the newest revision the person
  has, one decision, as a control on this tab does; where the way's
  `delegation` holds, the project's policy joins the items, read whole and
  set with charter 5, role 0 and the merge job added to its ceiling's and
  role 0's delegation. *"Keep it"* does the same for the way the project
  runs, while To begin's "Tell temper how you work" is left (`home.md`, 1, *To
  begin*), so that its values become the project's own and that change
  marks the step done; once the step is done the current way has no act.
  A project that runs no stock way has a fourth, *"Yours"*, current, its
  path the read's `own`, with no act. Neither act shows where `may_change`
  is false. While one is out both are disabled; a `stale` refusal is sent
  once more against the revision it names; a refusal is said in jig's
  words with the forge's reason, as an approval too few people can give
  names the forge, the role, how many it asks and how many can
  (`../../protocol/configuration/settings.md`, 2); answered, a toast names the way
  and the revision, "You approve is this project's way now, as revision
  12." or, kept, "Agents review is now this project's own, as revision
  12.", and the subject's next document shows the way current. The tab
  shown and the stop whose card is open are the page's state, in no
  address: the current way's tab as the page opens; kept by the way's
  place across documents while it stands, else the current one again;
  the card closed by another tab or a change of the shown path. Ask
  temper's page draws them as `ask-temper.md`, 1, *Ask temper's page*, says, and How work
  runs as *The page* does, above. Ask's row also holds "How does temper
  work?" before the way's act, alone where the way has none; How work
  runs never offers that button (`ask-temper.md`, 1).
- **The terminal:** the switches as checks, the selects as selects, a disabled
  one marked so; What reaches you's columns one after another, each its chip's
  words, its lines and its small line; What follows a landing's boxes one
  after another, each its title, its words, its lines and its chips'
  words; it all prints.
- **What the settings change:** the rail and the tab bar (`../view.md`, 3), `goals` hiding
  the offer of Goals and not a goal the project holds; `drafts`,
  `pages::chats::drafts_offered`, the chats' offer of a draft, an Ask temper
  chat's apart (`README.md`); kind 8, the Away switch, a reason's set names, Stop
  asking's offers (`README.md`, 2, *Away*; `needs-you.md`, 1) and What reaches you's columns;
  and nothing else;
  and these open the tab, `Address` having no anchors: Home's "Show goals",
  "See the settings" and "Limits".
- **The card in a chat:** a proposal of settings is a card at the call that
  made it where its turn names it, and otherwise after the turn that was running
  when it was made, as every proposal is (`README.md`; *Where a card stands,* below): the title "How temper will run
  PROJECT", PROJECT its `projects`' names joined by ", " and " and ", or, with
  none, the deployment's, "How temper will run your projects"; the proposal's reason, in the person's words, its Markdown drawn as the Accept sheet draws it (`../README.md`, 3.6);
  then each item, small and faint in the card's type, in the words of `temper ctl` but
  for temper's kinds of a project: kind 7 as "Drafts: on", "Goals: shown",
  "Changes page: shown" and "Way: NAME", kind 6 as
  "Questions: answered by the goal", "Questions: asked of me too" or "Questions:
  not asked", kind 8 as "What reaches you:" and a line a class the proposed
  table gives any subject, in the classes' order, each its subjects as
  Settings' checks name them, lowercased and joined by ", ": "Agents decide,
  then tell you: wording choices, retries and splits, test-only changes",
  "Asks you; Away may decide: implementation choices", "Asks you, even when
  away: changes to vsr’s protocol core", then a line a set, "NAME:
  PATTERNS"; a template by its name, as "Way: NAME" names one, or "template 4"
  where the person's `me` names none; any other as "charter 2", "landing rule 1".
  A proposal's heads name jig's own settings alone, so a live card takes its
  application's items and project's policies from the proposal's review, and
  those values from the forge's read of a proposal's items under the review's
  `key`, which takes a role in the deployment
  (`../../protocol/configuration/settings.md`, 2). Until those values arrive, kinds
  6, 7 and 8 are named as `temper ctl` names them. With no review, no application
  or project-policy rows can be recovered from the heads. A decided card has
  no review: it keeps the kind 6, 7 and 8 values it read in this session, and
  otherwise draws its heads alone. Then
  "Accept", primary, jig's `Decide` with running work staying, and "Change
  something", quiet: it focuses the composer, brings it into view and sets its
  placeholder to "What should change?", until something is sent or the proposal
  is decided, when the placeholder goes back. A decision goes under the proposal's
  own `project`, never the chat's: a proposal of settings is the deployment's,
  whichever chat made it, and jig knows it under no other project. The card has
  no "Reject": the person rejects it on its card in *Needs you* (`README.md`). Saying
  what to change here has the agent propose again, whose fresh proposal replaces
  the one that waits (`../../domain/agent/agents.md`, 1): the new card stands below. An
  "Accept" pressed on a replaced card, in a tab that hadn't heard, is refused and changes
  nothing, and the refusal ends the card as withdrawn: "Replaced by the proposal below",
  or "Withdrawn · nothing changes" until it has read the later one. Decided, it ends in a muted state line in place of its
  acts, however the decision was made (*Needs you*, another tab, `temper ctl`), the
  chat's task document telling it with no reload: accepted, "✓ Accepted · new work
  follows it"; rejected, "Rejected · nothing changes"; withdrawn, "Withdrawn · nothing
  changes", or, a later proposal of settings of the chat's having replaced it, dimmed,
  "Replaced by the proposal below". While it is pending, a person who may not accept it
  sees neither act: "Waiting for an owner", the parties it waits for as jig's proposal
  names them, "Owners of this deployment" for a holder that is the deployment's policy.
  Before its document is read it says none of these. Nothing of it
  says which items temper enforces and which agents follow: that is the review
  page's, below, once it is built. In the inbox, its card shows the same lines
  (`README.md`).
- **What a rule on a box holds against:** where the forge's read of the
  proposal's items answers `owned_approval` (`../forge/records.md`), a landing rule of it with an
  enforcing approval covering a repository on an owned Forgejo, its card says,
  after its items and in their look, "On this machine temper's agents and CI
  jobs run as you, so this stops their mistakes, not one trying to get past
  it." (`../../domain/forge/change/questions.md`, 2; `../../draft/forge-identity.md`, section 6),
  in a chat and in *Needs you* alike, and its review page says it under its
  plain words once that page is built (below). Until the read answers, and
  for a decided card with no review, it says nothing of it.
- **Where a card stands:** jig's activity says when a proposal was made: a
  `Decided` line naming it, with `made` (jig's `client/domain/contract/documents.md`, 1).
  With no call naming it, its card stands after the first committed turn after that
  line, and with no such turn yet, its turn still running, at the conversation's end.
  So a new pending card is the last thing in the chat, the chat stays at its end as
  usual, and the reply after an Accept follows the card. A pending card whose line
  isn't in the window (a delegate's has none) goes by its waiting time: after the first
  turn committed at or after it, or at the end. A card whose place is past the window's
  end isn't drawn until the window gets there. Cards at one place stand in the order
  they were made.
- **A decided card after a reload:** that same line brings its card back, read as a
  pending one is: jig answers an ended proposal with its reason, items and `projects`.
  It stands at its place, through a reload and paging back, with its state line. Past
  the bound of what jig retains (`inbox_messages`, its read answering `Gone`), no card
  stands. A proposal that isn't of settings draws no card once decided.
- **"Decision made":** a line of a proposal's making draws no row, being no
  decision; and where a card stands, its proposal's lines draw none. A card and a row
  never both say it. A proposal with no card standing keeps its row.

**Later: the fuller page.** Roles as settings, ways to start work, rules, running work and
history join the page, the review page opens, and the card gains "Review". The
pages then read jig's `settings` document and each setting they show whole
(`setting`, jig's `client/domain/contract/documents.md`, 1, and `jig/docs/design/client/domain/contract/README.md`, 4), again as the document
says it changed; temper's kinds' values, and a proposal's items of them, from
the forge's reads (`../forge/served.md`). A party sees the deployment's settings
with a role in the deployment, and a project's with a role in that project, as
the document gives them; the client hides nothing of its own.

- **More of the page:** the newest revision, "Revision 8, accepted by Pat today
  at 14:03" (who decided it and when, `../view.md`, 6; "temper's defaults" for the
  first start's and "temper 0.4's defaults" for a release's); each
  pending proposal of settings as a line, "Proposal 13 waits for an
  owner. Lee asked for a Reviewer role, from "Reviews"", with "Review".
  Then five sections, each a level-2 heading with its count and a line
  of what it holds, its rows folded until opened, as the accounts' are
  (`accounts.md`, 2):
  - *Roles,* the charters: each its name, "a role · temper's stock" or
    "added in revision 6", a tag, and its line, a sentence of its
    instructions' first paragraph; opened, its instructions (temper's
    for a stock one, its own otherwise), its models, where it runs, how
    it waits and its budget, and how many tasks run it, by revision;
    the forge's charter says "Made by temper when a change needs a fix,
    a rebase or a gate".
  - *Ways to start work,* the templates: each its name and line;
    opened, its role, its budget and deadline, and how it closes.
  - *Rules,* what temper holds work to: the deployment's landing rules
    and, for the project chosen, its own, each its branches and what
    it needs ("Needs CI passed · up to date · gate passed · 1
    approval by an owner") and who decides it ("Decided by acme/app's
    owners: Pat, Kim"); the policy new projects start with; and jig's
    rules, "Everywhere, always". A project's people and its own policy
    are its own page's, not this one's.
  - *Running work,* the revisions held: the newest, "Newest · 14 tasks
    · new work takes it", and each older one, its tasks' count, and
    "Move them" (jig's `MoveToNewest`), confirmed (`../README.md`, 3.6) as "Move 9
    tasks to revision 8 at their next run? A task whose role revision 8
    removed, or whose model it no longer allows, finishes on revision
    7."; the answer's count of those that could not move is said after
    it.
  - *History,* the revision records the store keeps, newest first:
    each its number, what it did in a line (a proposal's title, or "a
    release's", "the first start's"), who decided it and when, "Show",
    and "Restore…", which proposes the settings as that revision left
    them, against the newest (jig's `domain/engine/settings.md`, 10), saying
    which of temper's kinds' settings it leaves out, their past values
    not kept, and opens that proposal's review.
- **Its tags:** "Default", a setting no change is over; "Changed", one
  a change is over, with "changed in revision N"; "Added", one no
  default has.
- **Proposals, but for the tab's own rows** (`../../draft/configuring.md`, section
  10; jig keeps the owner's direct change in its API, and the pages send
  `ChangeSettings` only for a row of the tab that is one setting, above). A later row's act is "Change it in a chat", which
  starts an Ask temper chat with the row named in its first message.
  "Reset to default…" on a changed default, "Remove…" on an added
  setting and "Restore…" in the history are the person's own proposals
  (jig's `Propose`), each confirmed (`../README.md`, 3.6) and opening its review page.
- **The card, with the review page:** at most five lines and "and N more",
  then "Review", to its review page; nothing is accepted from it once the
  review page reviews it.
- **The review page:** the level-1 heading "Proposal 12 · from "How we
  work" · against revision 7"; "Made by T21 How we work, for Pat, today
  at 13:56"; then two blocks:
  - *temper enforces,* "temper holds work to these, whatever an agent
    does.": the changes of every setting but a charter's instructions,
    grouped as How work runs groups them (roles, ways to start work,
    rules), each before and after in words, with "new" and "removed";
  - *your agents follow,* "Agents read these at each run and are
    trusted to follow them; temper does not check.": the changes of a
    charter's instructions, each as the role and what it now reads.
  Then *Effect*: "New work follows this from the moment it is accepted.
  Nothing restarts."; "9 running tasks are on revision 7:" with two
  choices, "Let them finish on revision 7", chosen, and "Move them at
  their next run"; and "A task whose role this removes, or whose model
  it no longer allows, finishes on revision 7." Then "Show the
  settings", folded, the items as the settings name them (charter 4,
  its models, before and after), and the acts: "Reject…", a reason,
  sent as jig's `Decide`, which the agent hears and revises from;
  "Change something", which returns to the chat with the person's
  words; and "Accept", jig's `Decide` with the
  choice of running work, naming the proposal by its proposer and
  number (jig's `client/domain/contract/requests.md`, 2, and `jig/docs/design/client/domain/contract/documents.md`, 2); the revision
  it was made against is its own, and one made against an older is
  withdrawn, never accepted (jig's `domain/engine/settings.md`, 6).
- **What the review page says plainly:** a landing rule naming a role
  nobody holds cannot be accepted: the refusal names the affected
  project, role, required and eligible forge-person counts, and asks
  for known forge people with the role or removal of the approval
  (`../../domain/forge/change/gates.md`, 3). An owner login or display name supplies
  no forge identity. A change of jig's rules that removes the
  landing's requirements (`../../protocol/configuration/settings.md`, 3),
  "temper no longer holds landing to its rules: the change procedure
  still waits for them."; a role removed while tasks run it, "2 tasks
  hold it on revision 7"; a landing rule with an enforcing approval on an
  owned Forgejo, the card's sentence for it (above).
- **Stale:** a proposal made against an older revision is withdrawn by
  the decision that made the newer (jig's `domain/engine/settings.md`, 6). Its
  review page keeps its items, dimmed, under the banner "Withdrawn:
  made against revision 7; now 8.", what revision 8 changed in a line
  and who made it, and "T21 How we work is redoing it against revision
  8; its new proposal comes to your inbox."; it offers no Accept, only
  "Open the chat". In the chat the card says "Withdrawn: made against
  revision 7; now 8", and the agent, told so, proposes again against
  the newest, a new card after it.
- **Accepted:** the review page says "Accepted by Pat at 14:03:
  revision 8. New work takes it now.", with what became of running
  work ("9 tasks finish on revision 7", or "moved"), and "How work
  runs"; in the chat, the card's line "Accepted at 14:03: new work
  takes it now." A refusal of Accept (out of shape, judged, full) is
  said under the acts in jig's words, the forge's for a landing rule
  it refused (`../../protocol/configuration/settings.md`, 2), and the proposal
  stays pending; full names the oldest revision held and offers "Move
  them" there.
- **Drawn ahead of jig:** the names of a revision's tasks, "Show them"
  and how many of them could not move are not in jig's facts (`../README.md`, section 5
  ); the page draws counts alone until they are. A charter's
  documents, "Owners also read acme/handbook process.md", wait on jig
  (`../../protocol/configuration/README.md`, section 9).

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
- **What reaches you after landing** (2, *What reaches you*; `README.md`, 2,
  *Away*): temper/ui compares main with the section of the whole story's
  `web/whole-story/how.html`, `pair/how.html` and `team/how.html`, through
  `../web/gallery.md`, 2's `how-reaches`, `night-how-reaches`, `how` and
  `how-approve`, at 1440, 1000 and 390 px, by day and by night, the
  agent's star and the person's ring on the chips; and walks on a box a
  need's Stop asking applied by an owner, its subject moving from "Asks
  you" to "Agents decide, then tell you", and "Limits" bringing the
  section into view. Owner: temper/ui. Default: each difference a
  finding filed back as a repair.
- **What follows a landing and Budgets after landing** (2, *What follows a
  landing* and *Budgets*): temper/ui compares main with those sections of
  the whole story's `web/whole-story/how.html`, through `../web/gallery.md`,
  2's `how-follows` and `night-how-follows`, at 1440, 1000 and 390 px, by
  day and by night, code's mark on the chips beside the agent's star; and
  walks on a box a follow's box on the target project's How work runs, its
  lag rising with a source landing and back to "caught up" once its change
  lands, and a standing role's row once `temper ctl role start` has made
  one. Owner: temper/ui. Default: each difference a finding filed back as
  a repair.
- **How work runs' fuller page** (2, *Later: the fuller page*): roles as settings, ways to
  start work, rules, running work and history on the page; `Proposal { proposal
  }`; and the card's "Review", with the grouping of what temper enforces and what
  agents follow that the mock's `.proposal-split` draws. Default: the page's parts
  and the card alone.
- **How work runs' later parts** (`../../draft/surfaces.md`, sections 7 and 12;
  `../../draft/whole-story.execution.md`, section 3):
  "History · N changes" beside "Ask temper to change this", once a page lists
  the settings' revisions (*Later: the fuller page*); the next step suggested
  when the facts earn it; and the project's repositories at the page's foot
  rather than its frame's second tab. Default: none of them is drawn until it
  is built, the heading is the way's headline or the project's words,
  and Repositories stays the second tab.
- **How many goals take a way** (2): " Used by N." and " Used by none yet."
  need a goal's template, and jig's `TaskLine`, what a client is given of a goal,
  records none. Default: the sentence is left out.
- **Application rows after a decision** (2): retaining every application
  item and project-policy row, and restoring kind 6 and 7 values after a
  reload, needs more than the session's cached values. A person without a
  review has no such rows to read. Default: a live card uses its review; an
  ended card uses its heads and the kind 6 and 7 values already read in this
  session, with no new proposal-value request.
- **The person's login over 760 px** (1, *Sign out*; `../view.md`, 3; the request
  `ui-owner-menu`): the sheet hides the `Accounts` tab's row whole, so a browser
  that wide reads the person's name on the rail's foot, and their login on no tab
  of `Settings`. Candidate: the sheet hides the row's button alone, its words
  staying. Default: the whole row.
- **Which tasks hold a revision, and which could move** (2). jig's
  `settings` document counts the live tasks holding each revision, but
  a task query has no revision among its keys (jig's
  `client/domain/contract/README.md`, 4.1), and nothing says before a move or
  an acceptance how many tasks could not move. Default, asked of jig:
  a task query keyed by revision, its live group only (a scan, which
  costs live tasks no more than any key), and a proposal's or a move's
  count of tasks that could not move; without it, "Running work" and the
  review page draw counts alone and say in words which tasks stay.
