# The mockups' status

Revised 2026-11-11. How far temper's client is from its mockups
(`README.md`), a group of pages at a time. A group under the rules has
a row for each thing its main drawing shows that the client doesn't
draw yet (a lag), and for each place the client does otherwise (a
divergence); the rest of its main drawing is built. Its future pieces
are marked in the drawing, each with its need, and not repeated here.
Section 2 lists the pages not under the rules yet, with the notes the
README held for them.

The look request `outside-pages` draws every page outside a project in
the welcome's look (`../../view.md`, 1; `../look.md`), so the
mockups' banner over those pages is not the client's.

The `connect-flows` request supersedes Settings’ model and forge connection
drawings: shared welcome cards and panels, no steps bar or Ready page,
completion on Accounts or Forges, and no repeated summary facts in an open
account. Settings’ adoption drawing is now “Add repositories”, with Adopt,
Make empty and Copy public, search and a list that reads on, checkboxes,
Not listed and More. Copies and failed copies remain in Repositories.
The hand-made request’s 01–23 are not committed mockups or journeys;
`../journeys.md`, 9, names their gallery coverage. These are code and print
checks, with no new visual audit: the retained browser stories share
setups within twelve tests, and no passing result for their repaired
versions is supplied here. temper/ui’s comparison follows landing
(`../look-requests.md`, 4). Home keeps its own empty-or-copy sheet,
as the user asked (`../../forge/family.md`, 3; the draft status's departure,
`../../../draft/repository-copies.status.md`). The two entry paths into
“Add repositories” that the request left undrawn, To begin's repository
step and a project with no repository's Chats band, are the gallery's
`sheet-from-home` and `sheet-from-chats` (`../gallery.md`, 2).

## 1. Under the rules

### 1.1 The welcome journey

`welcome.html` and `github.html`, at the states their worlds list:
`world-first-run.js` at step 0, `world-first-run-real.js`,
`world-first-run-github.js` and `world-first-run-elsewhere.js`. Audited
2026-10-25, at temper 3273eae0d (first-run-ux-6's walk of its 64 rows,
desktop and phone, and its fixes); under the rules since 2026-10-25,
with the user. The Chats, Forges, Accounts and
Connect pages the journey passes through are other groups'.

What the client doesn't draw yet:

- **A sign-in's port taken on Ready:** declined, held and misconfigured
  now draw their words and acts. "Try later" still needs the listener's
  port-taken fact in the client (M-j); `OwnerAccounts.last_end` does
  not carry it.

Where the client does otherwise:

- **Claude's host check** (the client README's 3.5, *The try*): "I’ve
  signed in" goes on to Ready at once, and the check shows as Model's line in
  Ready's pane, its first step under way. Not signed in, Ready reads Almost
  ready, the line marks that first step failed and says why, with "Try
  again" and "Sign in again". The mockup's checking note and separate not-yet
  alert are not drawn. Once usable, the account starts a trial through the
  worker and model; Model's line draws its later steps, answer or stop
  (`../../../domain/trial.md`, 6.1 to 6.4), also on Accounts and, failed, on Home.
  The mockup is not edited.

- **Model's forge pane** (the client README's 3.5; `../look.md`;
  the user, 2026-11-04): its named Forge region follows the choices and
  precedes the foot, beside the column from 1280 px and under the
  choices otherwise. Its name and muted details stack under a mark,
  a green tick once connected; the repository precedes the connection
  under way and failed. While setup runs, a named indeterminate bar
  follows the status before "Each step", moving unless reduced motion
  is requested; done, failed and GitHub's installation waits have none.
  The terminal says the reordered line with no bar. "Each step" stays
  in the pane while unfinished. The mockup is not edited.

- **Ready adopts, and waits for nothing** (the client README's 3.5
  and 5.6; `../../../draft/first-run-projects.md`, 4): Forgejo on this
  machine's Details have no "Into a new project, named" and GitHub's
  step no "Choose a repository to start in"; the forge's line names no
  project; Ready's act makes the first project, adopts the first
  repository into it and opens its Home without waiting, where the
  mockup's Ready says "ORG/NAME is ready" and opens it once made. The
  mockup is not edited.
- **The project's frame and Home** (the client README's 3.5, 5.1 and
  5.3; `../look.md`): every project page has the rail of the
  user's greenfield mock's `home.html`, which `chats.html`, `chat.html`,
  `inbox.html`, `accounts.html`, `forges.html` and the first run's pages
  do not draw, under their top bar; the project's first page is Home.
  Once the project's facts and the person's dismissals are delivered,
  with a step left and no skip, Home says "PROJECT is ready." above
  To begin, a card headed "To begin" beside "Optional", with "Skip"
  at the right. Its note counts the steps left and says whether their
  acts stay in Settings, Ask temper, or both. Its two rows keep their
  order: the repository, then "Tell temper how you work", with each act
  at the right while left. Done, a number becomes a green tick and the
  act goes; "Tell temper how you work" keeps its title alone, the
  repository its line. While it shows, the rail holds Home and Ask temper,
  with Settings at its foot, and a phone's tab bar Home, Ask and Settings
  (the look request `to-begin-two-steps`). An adoption out
  names the repository and project with no act; refused, its words
  precede "Adopt" again. The terminal keeps the heading and says the
  numbers and "Done.". Connected is absent on every Home; rename stays
  in the switcher; Forges, Accounts and Models are in Settings. Models has
  the offered list, picker switches and adding by name (`../../pages/README.md`).
  The empty Needs you and Chats sections are left out while To begin shows;
  a need or chat appears under it once present. This remains a gap from
  `to-begin-two-steps`, which asks both sections to stay out even with
  content (`../../pages/README.status.md`). Skip hides the card at once and is
  kept per person and project across devices; refused or lost, the
  card returns with "temper couldn't keep that. Try Skip again."
  below the note. Skipped or with every step done, Home's headline
  counts the project's needs, its first need is a card
  and the others are rows, and Chats has at most five rows with each
  chat's state and age. "All N" reads "All N+" where the read is cut;
  empty sections say "Nothing needs you. Work carries on, and this
  page tells you what changed when you come back." and "No chats
  yet.", with no "All 0". The phone replaces the card with a pale
  signal block and "Answer", or "Answer, one at a time" for more; that
  link and the wide "Answer in a row"
  open the project's *Needs you*. The rail and phone tabs colour the
  open project's needs count, as Home's section does. The kicker shows the
  current day and time and "you last looked", the headline what landed,
  and the lede and byline the status agent's words, with no prose of the
  client's where it has written none; "Landing: …", including "Landing:
  nothing has landed yet" after Skip, closes the page, and Goals takes
  Chats' place with a goal in the project (`../../pages/README.md`, *Home on
  any project*). Context, choices,
  "recommended", "Later", the answer's green line and "Undo" are the
  greenfield mock's (`../../pages/README.md`). A chat's "landed" waits on
  `../../README.md`, 5, *What a chat did, on its line*. An adopted repository's To begin line says temper can change its code,
  without claiming a creation age or first CI result
  (`../../README.md`, 5). Landing-by-turn wording waits on that
  section's *How a project lands, on To begin's repository step*: the
  client has no project's `by`, so no project's line names how it
  lands. The switcher shows
  repository names and counts, the first listed standing in for the home; "3 repositories"
  without that stand-in is unreached (`../../view.md`, 3). Home's
  GitHub picker and reached count and the model account's email wait
  as `../../pages/README.status.md` and `../../forge/family.status.md`, 4, records. The gallery draws
  Home's states, including Claude alone, both plans and an API key.
  While To begin shows, the shorter lede names the first usable account
  (a key as "an API key") and the connected forge; the Accounts link
  names what the person does not hold yet (`../../pages/README.md`).
  The recorded project landing still prints Welcome
  (`../journeys.status.md`, 9).

  | Page | Where the client does otherwise, or what is left |
  |---|---|
  | Ask temper's page (`chat.html?id=new&configure=1`) | In the project's frame, through no Settings, at `/projects/<project>/ask` (`../README.md`, 2.3), the rail's "Ask temper" and the tab bar's "Ask" current (the look request `ask-temper-place`); an Ask temper chat keeps that layout at `/chats/<task>` (`../../pages/README.md`). The opening draws the lede "Pick a way for PROJECT, or ask temper to build your own.", the ways' tabs and axis, the shown way's line and path from the `ways` read, a pressed stop's card, the legend and "Use this" or "Keep it" where offered (the look request `ask-temper-ways`). It has no heading, starters or temper's own way's card, and no reference beside the page. On a phone the path runs down the page; while To begin is left, its three tabs stay under the composer. The opening, turns and composer share a column at most 720 px wide, with both scrollbar gutters on a wide page and 16 px in on a phone. Typing, sending, refusal and reopening keep their words and controls. The rail's "Ask temper", the tab bar's "Ask" and To begin's "Start" open the newest Ask temper chat not closed in their delivered rows, or this opening; Settings offers none. Chats tags it "Ask temper"; To begin keeps "Tell temper how you work". Render comparison follows landing, owned by temper/ui (`../../README.md`, section 5). |

  The model line names the first usable account's kind and the first
  model `me` serves for the project (`../../pages/README.md`), rather than
  claiming Ask temper's charter's first model: a template's `llm` may
  be its charter's default, which the client is not given. The gallery
  draws `gpt-6.1-sol` where its facts serve it. Opening focus is built
  in both shells; the static gallery omits focus by its contract
  (`../gallery.md`, 1). The first-chat selection gap remains `../../forge/records.status.md` (Departs, The first chat). A proposal
  review's "Change
  something" awaits the client's proposal view
  (`../../pages/README.status.md` and `../../pages/settings.status.md`, 2).
  With a document it shares every chat's Chat, Split and Draft controls and
  Markdown pane, and has "Start over" in place of "Start goal…". Split
  folds the rail to a 56 px strip; "Show the rail" opens it over the page.
  On a phone Split displays as Draft and only Chat and Draft are shown.
  The document's path line says not proposed, waiting for an owner, or
  accepted, from the proposal's state. The newest changed section is tinted;
  Show focuses and flashes it. The request `configure-chat-draft`'s drawing
  and figure wait on `skein/docs/design/draft/diagrams.md`, whose decision
  remains the user's. Its hand-made `configure-chat/02.html` through
  `04.html` and README are request sources and are not committed prints
  (`../journeys.md`, 7).
  Ordinary chats share this pane, add an empty draft, and open the shared
  Start goal sheet; Chats' Drafts filter lists their documents. The sheet's
  Budget and agreed-way gaps stay in `../../pages/README.status.md`.
  The mockups are not edited.
- **The welcome's greenfield pass** (the client README's 3.5 and 5.6;
  `../look.md`, The welcome's look; the user, 2026-11-04): the
  frame holds temper's mark, the steps and the link region alone, no
  person or "Sign out" (`welcome-top`); temper box's line is a paragraph
  under the Welcome step's cards' note, not in the bar, and Forge, Model
  and Ready draw none. "Set up temper" is the first card and the default;
  Forge's Forgejo card comes first with "Run on this machine" alone,
  no "Or connect your deployment", which opens Forgejo's sheet, what
  it starts with, an empty repository by default, as the user's
  greenfield mock draws it; Model's cards hold the plans alone,
  no key; the forge's ticked line says who temper is on it and that
  its first repository is made; the foot has no "Later";
  Ready is the greenfield mock's, its pane, the request
  `model-check-pane`'s in the checklist's place, and its first
  project's form, the first repository's name in capital or "My
  project" until the person names it otherwise, and
  "Create your project"; and the words of the cards and leads are the
  client's. The mockup is not edited.

Not yet known: whether GitHub keeps the state through its
installation's Configure and Save, and whether the picker reads the
repositories again on the way back (`../../forge/served.md`, 1).

## 2. Not under the rules yet

Every page. What one draws may need what the engine lacks, marked
`ahead` or `gap` or not at all, and the client documents' passages
comparing a page with its mockup are disagreements still to settle.
The notes below are the README's, as they stood on 2026-10-24, until
their group comes under the rules.

### 2.1 Where each value comes from

What a task waits on is jig's, computed by the engine: a party, its delegates, news. Where it is news, the page narrows it from the task's wake policy, mechanically: *your words* when a party's words wake a task that party requested (a chat at rest), *T12's end* for a subscription to another task's state, *until 09:05* for a timer alone. *Waiting on your words* is not *needs you*: nothing is held up, and only an entry (a question, a proposal, an escalation, a person task) is in the inbox, where its task waits on *an answer from you*. temper's client words these statuses its own way ("Waiting for you", "Parked"; the client README, 3.5).

Anything else on a page is the UI's own words, and those name no workflow: the director, an owner or a check appear only in what was written or configured. The inbox, for example, says "Nothing needs you." and stops; "Next check at 09:10" appears only where the director wrote it, in its reply. Its updates are derived, as jig's inbox derives them: replies in the person's chats, and results of the tasks the person requested.

### 2.2 Drawn ahead of temper

What temper does not have yet, drawn ahead of its design and marked with its gap's label while **Annotations** is on:

- **6.3, plan usage:** a program's 5-hour and weekly windows on the accounts page, the week with its pace; the director's check reads them. A ChatGPT plan shows none: its usage is ChatGPT's settings' to show (jig's `domain/providers.md`, 2.6), so its row shows its limit's state and "Manage usage ↗", which opens them.
- **6.4, worker health:** the ops strip on the workers page, an ops application's report of load and disk.
- **6.5, open findings:** a finding on a goal's page, the board and the changes page, where a failure that matches an open finding attaches to it and leaves the queue running.
- **6.7, fleet occupancy:** a worker's free slots, and the director's read of them.

Marked "ahead" (slate, with the annotations): what the owned Forgejo’s rows and the first run draw before temper has planned its implementation. **First repository:** the set-up’s first repository, and "New repository", made by temper with the admin’s token once temper-15 moves that token into the engine. **demo’s end:** retirement after the first personal forge connects, with no implementation planned. **sign-in:** "Open Forgejo" signing the person in through temper, as Forgejo’s OpenID provider (`../../../protocol/openid-provider.md`); until then, the person signs in with the password the preparation wrote. **display:** an account's email, "ChatGPT · sam@example.com"; jig carries no display yet, and the client README's 5.5 drops it from the heading until it does. **words:** a sign-in's endings that 5.5 has no words for: a subject another person holds, and the port taken. **Google:** signing in to temper with Google, through jig's `Mint`, and the return route's pages (`../../../protocol/sign-in.md`, section 4). **provider:** temper as Forgejo's OpenID provider: its authorization, and "Open Forgejo" on a managed Forgejo that signs in through temper (`../../../protocol/openid-provider.md`). **sign-out:** signing out of temper leaves Forgejo signed in (9.6). **sketch:** Claude's subscription in Connect a model, in outline. **limited chat:** where a chat shows its account's limit.

Two pages follow jig's pages, which temper's client leaves out (the client README, 5.3 and 11): **Tasks**, every task of the project as one tree (jig's `client/domain/pages.md`, 6.3), and **Workers**, the deployment's hosts (7).

Not drawn: two ideas that would change temper's engine, not only its pages. A charter or template would name the pages where its tasks lead, so the board could open with the director's line; an agent would set a short status line on its task with a tool, shown as written beside its phase. They wait for a design of their own.

### 2.3 The first run

`world-first-run.js` is the deployment `temper-box init --box` writes (`../../../protocol/setup/init.md`, 2 and 3): its project 1, "Project 1", which View → Future draws as no project, made by the playground's start or the first repository (`../../../draft/archive/first-run.md`, 2.4), the charters `chat` and `coding`, one worker in temper's process, three forges (the demo's fake, GitHub not connected, a Forgejo here not set up), and no account, repository or forge started yet. Every word an agent says is the demo's script (8.4). [first-run.html](first-run.html), the address the documents cite, opens it at its first step. Its steps:

- **0. The link opened:** the welcome offers Try the playground or Set up temper (section 1.1). Chats behind it has the setup strip, no model and a disabled send button.
- **1. The demo chosen:** the engine asks temper box for the demo’s fakes, makes the project Playground, adds the Scripted model’s account and adopts playground/widget into it. The chats page is the demo's start (client README, 5.6): the banner under the navigation, as on every page; a chatbar with no task and no number; the context line; the three suggested requests.
- **2. A request sent:** "Add CSV export to widget" starts the chat T1; its agent delegates the change T2, whose agent T3 edits `src/main.rs`. T2's card says Running with no step ticked (5.7).
- **3. The change landed:** T2's card says Landed, every step ticked, "CI passed. Landed · merge commit e3b8d41"; T1 says so in its script's third turn.
- **4. A text-only request:** "Add a dark theme to widget", the chat T4, answered in words; its script's `finish` ends only the turn (../../../protocol/setup/playground.md), so T4 waits at rest for Sam's next words, its composer open, as T1 does.
- **4a. Connect a model, from another computer:** the mockup's `step=model-choose` still draws Choose, with Continue with ChatGPT and the secondary button “This browser isn't on temper's computer ›”. The client has no secondary button or separate entry for another computer (`../../pages/README.md`, *The sign-in under way*).
- **4b. Signing in, with the browser elsewhere:** the mockup's `step=model-elsewhere`, `way=chatgpt`, still draws Signing in, its external sign-in link in another tab and the paste unfolded. Accounts draws that sign-in with the paste folded. The client's welcome and Connect a model share the waiting note, “Open it again ›” and the folded disclosure “Signed in, but the page couldn’t be reached?”; opening it offers the paste, its field the note's width (`../../pages/README.md`). The greenfield's `app.js` still says “in this tab”: its README's rule puts look requests' changes on top of that drawing (`../greenfield/README.md`). The new-tab behaviour waits on `../../README.status.md`'s first **Not yet**; the mockups stay as drawn.
- **4c. The whole address pasted:** `step=model-pasted`, `way=chatgpt`, draws the secret field written, hidden, and Finish connecting enabled; the button leads to step 5. Accounts has the same paste, folded. No address is sent or stored by a mockup.
- **5. A model connected:** Sam follows the banner's Connect a model, chooses ChatGPT and finishes by pasting from the browser on another computer; the flow ends on Ready, with Start a chat and Manage usage. The Scripted model leaves Accounts, where ChatGPT · sam@example.com is the one row. The model's half of the demo ends (4.8): the banner says a playground forge alone, the chats page has no demo's start, and the composers' pickers name the next run's model and effort. The galleries below Connect a model, Accounts and Chats draw the rest of the ChatGPT experience (below).
- **6. Setting up a Forgejo:** chosen in the flow, its defaults kept (work/hello, a new project hello); the box has fetched and is preparing. The flow shows each step, and the banner says where it is.
- **7. A Forgejo connected:** temper connected as its bot, made work/hello and adopted it into the new project hello; its first CI run passed. The flow ends on the new project, opened with no chat yet. The demo is retired: forge 1 stops and disappears, T1 and T4 end at 21:19 with the reason, and playground/widget leaves Playground’s repository lists. Its landing and transcripts stay readable; Recent landings names it “playground/widget · the playground’s, ended”. The banner is gone.

### 2.4 The first run's other notes

The welcome journey's own notes are its drawing's, under the rules (section 1.1). What stays here is the first run's, beyond it.

The person meets the demo as **the playground** (with the user, 2026-10-23): "Try the playground", the banner "This is the playground: a scripted model and a playground forge.", Scripted model, Playground forge and playground/widget; the scripts say "Playground forge" and "a scripted change". The design keeps the demo as its concept's name (`application.demo`, the demo's halves); ../../../protocol/setup/playground.md holds the scripts' words, and the client README 5.6 the page's (`../../../draft/archive/first-run.md`, 2.3).

The demo’s explicit states are `not started`, `running`, and `retired`; an absent state retains the older worlds’ running behavior. Its model and forge halves and context draw only while running. The first usable account from another provider ends the model half and removes the Scripted model’s account; the first connected forge of the person’s retires the demo’s forge and repository and ends its open chats, keeping history.

While the demo is not started and a forge or model is missing, the setup strip takes the demo strip’s place. It states what is missing, shows any forge setup status with View, and links to Continue setting up and (until a personal forge connects) Try the playground. Welcome, Accounts, Forges, Connect a forge and Connect a model omit it. With no account, a composer says “No model yet · Connect a model” and cannot send.

Where the documents disagree, or leave a word open, the mockup takes one side and says so here:

- **T1's next run takes the new account** (step 5), where temper's stays on the demo model. The draft says the demo model "stays the demo project's" (`../../../draft/archive/first-run.md`, 2.2). ../../../protocol/setup/init.md 3 says a task with no model chosen takes the first of its charter's models that has a usable account, and the demo model is last in both charters. jig resolves that default once, at the task's Create, and records it, so a new chat's first run takes gpt-6.1-sol on the ChatGPT plan while T1, created before it, keeps the demo model. The mockup draws T1 on gpt-6.1-sol too: re-resolving a task's default for each run is jig's to decide.
- **The coding agent's title is the change's** (T3, "Export widget's table as CSV"): ../../../protocol/setup/playground.md makes the change's title its agent's brief, and a chip is the spec's first line. The other worlds title that agent "Produce TN's branch", a stand-in for a title temper's procedure does not word yet.
- **The dark theme's words** are a stand-in. Their content is the demo's follow-up (`../../../draft/archive/first-run.md`, section 8). The empty search and the fallback are not drawn: they take step 4's shape.
- **The forges page** shows only the person’s forges (connecting, setting up, set up failed, connected, stopped), each folded until opened, and the playground’s. A forge not acted on is a choice in the “Connect a forge” flow: Choose → Details → Setting up → Ready for a Forgejo, and Choose → Connecting → Ready for GitHub (below), ending on the new project. The banner shows the set-up’s status and a link to it; the Forges nav item’s dot is amber while setting up or connecting, red when a forge needs the person. The page and the flow are ../../forge/deployment.md's. Mockup-only galleries on both pages compare the flow’s other paths and the other forge rows, including managed and connected Forgejo. The flow’s “A Forgejo elsewhere” includes the setup entry that connecting as its admin needs.
- **Accounts and Connect a model** follow the forges' shape (with the user, 2026-10-23). The page stays Accounts; the flow and every entry point say "Connect a model": the banner, the composer's "No model yet", the page's "+ Connect a model". The mockup's flow is Choose → Signing in → Ready: Continue with ChatGPT opens its sign-in in the same tab, and the engine's listener sends the browser back to the page it left; the secondary button keeps Signing in here for a browser on another computer (B, below). A sign-in drawn under way offers the paste, folded on the engine’s machine and in Accounts, unfolded after the secondary button. The client's waiting note and shared folded paste differ as first run's 4a and 4b record (section 2.3). A key has Details in Signing in’s place. ChatGPT is drawn whole; an API key's form and Claude's subscription are there for the context, not designed. Each account is a folded row: its name and holder, its deciding source's state, and a line of its models, what it pays for and its tasks; opened, the state's words and acts from 5.5's table, its sources in order, Pays for, Used by, Waiting for it, and its acts. The Accounts nav item's dot is amber while an account is limited, red while one needs the person. Mockup-only galleries draw, below Connect a model, each other ending, back in temper; below Accounts, the ChatGPT row in each state of 5.5's table, and a key's and a Claude subscription's rows; below Chats, a plan's first-use card and a chat whose plan is limited.
- **A plan pays for its holder's own work, and for projects where the holder is the only member;** a key or a program's login, with no uses given, for its holder's own work. A limited plan's tasks wait for it; where the account also holds a key, "Use the API key first" reorders its sources, and never moves a task to another account.
- **B is built:** a browser on another computer finishes by pasting the whole address of the page that could not be reached into temper; the tunnel still works (`ssh -L 1455:127.0.0.1:1455 HOST`, `../../../../help/box.md`, section 5). The first run’s 4a–4c draw it; Connect a model’s gallery draws every paste state (below), and Accounts draws the same paste folded. The mockup keeps its separate entry and unfolded paste; the client offers the folded paste under the waiting note (steps 4a and 4b in section 2.3).
- **A key reads Ready until a run is refused:** nothing checks it when it is added, so a wrong key shows only at its first request. The key's form says so, as a note.

The paste’s gallery is below [Connect a model](connect-model.html?world=first-run&step=5#the-paste). Its slate headings link to each state as the main card: `connect-model.html?world=first-run&step=5&way=chatgpt&paste=STATE&elsewhere=1`. The query names a mock state only, never a pasted address. Omit `elsewhere=1` for the folded paste under a local sign-in; `accounts.html?world=first-run&step=5&paste=waiting` shows it in the account’s row, also drawn in Accounts’ gallery at `#other-states-signing`. The external sign-in links use a fake provider, never ChatGPT’s live sign-in.

| State (`paste=STATE`) | Gallery anchor | What is drawn |
|---|---|---|
| `waiting` | `#paste-waiting`; `#paste-local` without `elsewhere=1` | The paste unfolded or folded; empty secret field, Finish connecting disabled |
| `written` | `#paste-written` | Hidden written field; Finish connecting enabled |
| `sending` | `#paste-sending` | Field disabled, button “Finishing…” |
| `finishing` | `#paste-finishing` | “Finishing your sign-in…”; field and finish button gone |
| `address`, `state` | `#paste-address`, `#paste-state` | Refusal above the empty field |
| `too-many` | `#paste-too-many` | Refusal and Continue with ChatGPT; field gone, as built |
| `too-long` | `#paste-too-long` | Bound’s refusal above the field; Finish connecting disabled |
| `not-waiting`, `expired` | `#paste-not-waiting`, `#paste-expired` | “This sign-in has ended.” and Continue with ChatGPT |
| `lost`, `unsent` | `#paste-lost`, `#paste-unsent` | “That address wasn't sent. Paste it again.”; empty field |
| `done` | `#paste-done` | Ready, “Signed in to ChatGPT”, using the existing return’s drawing |

The earlier first-run mockup, a story of its own, also drew "Reset demo", a turn's "Just now", the composer's placeholder, "Landed into main" and the pull request's link on the card. The client leaves each out (`../../README.md`, section 5), and so does this world.

### 2.5 Chats, a chat, Accounts and the inbox

`chats.html`, `chat.html`, `inbox.html` and `accounts.html`, surveyed
2026-10-25 at temper ff1dac865c (the branch's base), then compared
with the gallery at 390 and 1440 px. The branch builds their composer,
chat rows and ended bars, inbox menus and cards, and account rows.
The two additional gallery states are the first plan's Chats at step 5
and the retired playground's T1 at step 7. This group remains here:
its drawings still include the future pieces and missing facts below,
unmarked in their main drawing.

The `chat-work` conversation is drawn through the view in the gallery's
`chat-work-*` states (`../gallery.md`, 2): a folded run of calls and its
rows, an answer's end mark, Latest while reading back, and the loading
status at the head. Reading follows the log's edges with no Older control
(`../../pages/chat/conversation.md`, 2). What remains of the live text and the terminal and
browser coverage is in `../../../draft/chat-reading.status.md`, sections 3
and 5, and of the line in its sections 2 and 7; a numbered list's look is in
`../look.status.md`.

The terminal is a browser's other look of a chat, with its own top line,
commands and document pane (`../../pages/README.md`, *A chat's terminal*).

Where the client's recorded picks stand against the drawing:

- **The chats page's pane** (the person's look requests, 11-03;
  `../../pages/README.md` and `../../view.md`, 1; `../journeys.md`, 7): a left pane holds
  the new chat's bar, "Chats" with its filters and list, and the
  playground's banner; the prompt stands alone beside it, the
  composer's line under it. The banner names the open project alone,
  with its switcher, and leads its corner with "Set up temper →" while
  the playground runs. The playground's start has no invitation, "What
  shall we change?" and its words; its suggested requests stay, under
  the composer's line (`../../pages/welcome.md`, 2, and `../../README.md`, 5). `chats.html` and
  `first-run.html` still draw one column under a banner of every
  project, the invitation in the playground's start.
- **Accept**, not “Accept goal”. The chatbar carries status and own
  spend, “N spent · of B”, and no model, template, charter, host or
  budget mode; the picker under the composer names the next run's model
  and effort (`../../pages/README.md`, *The work's account and model*).
  A stored picker amendment can still leave the next provider request on
  the previous model; `../../pages/README.status.md` records the proof and
  the `ui-model-next-turn` repair owner.
- **Closed** for every ended chat's bar and list row, with no clock in
  the bar. A requester-closed chat ends its conversation with "Closed
  at CLOCK" between two hairlines (`../../README.md`, 3.6). The conversation's
  Failed and Cancelled words and reasons stay. Chats is one list, newest first;
  a row's spend is its own, with live delegates counted separately.
- **Confirmation sheets** follow `../../README.md`, 3.6: a sentence title,
  only what changes, then the keep and named primary act. A close at rest
  has no impact list; a live run, delegates, tasks below or proposals add
  their nonzero lines under a hairline. An uncertain release's two choices
  wrap in their own group above the acts. The browser's native modal supplies
  the backdrop; its choices and confirmation acts keep the 44 px phone target
  (`../look.md`).
- **Acts beside each account and source**, including the move
  controls. A person's own account shows neither Used by nor Waiting
  for it; the deployment's accounts can show both. No Tasks or
  Workers in the navigation.
- **Mixed key and program offers** do not establish MODEL's endpoint.
  The composer's picker names the next run's model and effort without
  an account; Manage usage stays on Accounts and an agent's task's line.
  Known provider and plan names elsewhere use OpenAI/ChatGPT and
  Anthropic/Claude.

What is not built here, though the main drawings show it:

- A chats row's or update's reply preview; a chats row's ended
  delegates; fractional spend.
- A program's usage windows (6.3); an account's email display.
- A proposal's template and charter names (`me` has no charter names,
  and a proposed task names no template); a card's lineage beyond
  its own task (the inbox's `Card` carries no ancestry).
- **Board** in the navigation: `../../pages/README.md` and `../../view.md`, 3 have the
  project's Goals in its place, drawn from the greenfield mock.

Other facts the comparison found absent:

- **A timer's next clock or interval.** The chat's wake policy says
  timers can wake it, but carries neither “until 09:10” nor “every
  2 h”. The client keeps its existing news-wait words.
- **The request's own timestamp.** The task gives its specification
  but no separate request clock. The conversation uses its first
  activity line's clock for the request, as client README 3.5 says;
  that differs from mockups which timestamp the request separately.
- **An update's title before its task document arrives.** The entry
  has its task number and kind but no title. Its heading-link says
  “Task N” until the task document supplies the title, preserving
  the same OpenResult/Go act and seen behavior.
- **A plan's first-use card.** No delivered account or task fact says
  whether the person has seen that explanation; the mockup-only
  sample below Chats is not drawn as part of the client's page.
- **Older transcript and calls beyond the receiving bounds.** The
  reference T1 drawing has 142 older turns and nine calls; the fake
  client's delivered window has no such total and carries three
  calls. The gallery keeps the browser's shipped bounds (../gallery.md
  2), says nothing of omitted lines, and binds the log's edges to read
  earlier or later pages where supplied (`../../pages/chat/conversation.md`, 2), rather
  than inventing history or widening limits.

Where a word or shape was otherwise open, the client keeps its
existing pattern: jig's executor and specification words and resource
and grant words in proposals (numeric charters until names arrive);
Kind's folded disclosure for all inbox menus; the folded rejection
editor for “Answer in words”; the forge row's native disclosure and
summary for account rows; the existing dashed empty block for “No
model of yours yet.”. Confirmation keeps each supplied funding fact
in its Budget row, as the existing decision referee reads it. After
retirement the banner selects the surviving project, hello, as client
README 4.8 requires; the history remains the retired task's. Other-state
galleries below mockup pages remain
examples of separate states, not extra content appended to a page.

### 2.6 Settings, Models

Settings has Models between Accounts and Builders, following `model-picker`'s
hand-made page 07 at `47c23ff6ed` (`../journeys.md`, 9). Its offered rows,
what pays, default effort, picker switches, adding by name and the link to
Ask temper are built; unknown prices read "estimated prices". A provider
without a usable account has its Connect or fix line and no model rows.

Where the client does otherwise: the address is under the project; one
row says "the default", with no "the default without Claude"; the lede
explains the available models and picker without promising list reads;
and the placeholder is `model-name`, independent of the selected provider.
`../../pages/README.status.md` records the reasons and the after-landing render
check, owned by temper/ui, at 1440, 1280, 1000 and 420 px. No geometry or
screenshot proof is supplied by this plan.

What the client doesn't draw yet: "Read again", "Couldn't read OpenAI's
list · 2 hours ago", "new" and "Read from OpenAI 2 hours ago" belong to
`model-catalogue-3` (`../../../draft/model-catalogue.status.md`, 5, 7). Separate
unusable-account rows beside a usable account of the same provider remain
that draft's section 3 gap.
