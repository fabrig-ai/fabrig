# Mockups of temper's client

These mockups are reference material for temper's client (`../README.md`): its look and the order of its pages' parts, not its contract. Where they and the client's documents differ, the documents hold.

Serve this directory and open the contents page:

```sh
python3 -m http.server 8765 --directory docs/design/client/mockups
# http://localhost:8765/index.html
```

Each file also opens directly in a browser, with no build step; serving it keeps the chosen world, step and switches from page to page. The only external asset is the optional Google Fonts stylesheet; system fonts are used if it cannot load.

## The pages

| Page | File | Client README, 3.5 |
|---|---|---|
| The contents: the worlds, the step, where to look | `index.html` | not a page of the client |
| What needs the person, then updates | `inbox.html` | `Inbox` |
| The person's chats | `chats.html` | `Chats` |
| A chat: its chatbar, conversation, delegates' cards, proposals, composer (`?task=T1`) | `chat.html` | `Chat { task }` |
| The project's goals | `board.html` | `Board { project }` |
| Every task, as one tree | `tasks.html` | none: jig's `client/domain/pages.md`, 6.3 |
| A task: a goal's plan, an agent's transcript, a change's stages (`?task=T12`) | `task.html` | `Task { task }` |
| The changes of each repository, with their queues and the open findings | `changes.html` | `Changes { project }` |
| The person’s forges: each one’s connection, access and load, and its acts, each folded; the playground’s | `forges.html` | `Forges` |
| Connecting a forge: choose, the details, setting up, ready (?forge=N) | `connect-forge.html` | `ConnectForge` |
| The workers, their slots and health | `workers.html` | none: jig's `client/domain/pages.md`, 7 |
| The person’s LLM accounts: each one’s sources in order, what it pays for, who uses it or waits for it, and its acts, each folded; the playground’s | `accounts.html` | `Accounts` |
| Connecting a model: choose, ChatGPT’s sign-in in the same tab or from another computer with the paste (B), a key’s details, ready; the paste’s states below (`?way=chatgpt\|key`, `?paste=`, below) | `connect-model.html` | `ConnectModel` |
| The welcome: try the playground, or set up temper | `welcome.html` | `Welcome`, drawn ahead (below) |
| The first run, at its first step: the link `temper box` prints | `first-run.html` | opens `welcome.html` at step 0 in the first run |
| The stock front page, the same for every deployment, signed out, with Continue with Google; Forgejo's authorization through temper; each page Google's return route answers instead of signing in (`?from=forgejo`, `?state=`) | `sign-in.html` | the browser's signed-out main (5.1), drawn ahead (below) |
| GitHub's own pages, as connecting GitHub meets them: the App's form, where to install, an account's repositories, an installation's; and the page temper's listener answers where GitHub's way can't go on (`?at=create\|install\|account\|installation\|refused`) | `github.html` | none: GitHub's, and the listener's routes (`../forge.md`, 9.1) |

"Adopt a repository" is in the navigation, in the order of the client README's 5.3, but has no mockup (4.4).

## A world, at a step

- **The worlds.** `world-reference.js` holds the reference case (`../../draft/reference-case.md`): a director, owners on two machines, pools, findings, an ops application, at hour 0, with the steps that move it on. `world-solo.js` holds a workflow unlike it: one developer, one repository, one laptop, chats and a goal set by hand, no coordinator, timers, pools or ops application. `world-first-run.js` holds a person's first run after `temper box`, from the welcome through the demo and its end (below). `world-first-run-real.js` follows the welcome’s real setup, without starting the demo, on a Forgejo here; `world-first-run-github.js` follows it on GitHub, with GitHub's own pages. `world-google-sign-in.js` holds a deployment at a name of its own (`../../draft/sign-in.md`): `temper serve` behind a TLS proxy, its people signing in with Google, and its managed Forgejo signing them in through temper. A page that reads wrong in one world assumed another's workflow.
- **The step.** `pages.js` draws every page from the world at the step in the address (`?world=`, `?step=`), so the pages agree at each step. The dark bar above each page is the mockups' own: ‹ and › move the step, **Steps** lists the whole case, the step's title opens what it shows and where to look, and **View** switches the world and what is marked.
- **What is drawn:** the reference case's hour 0, the solo world's one step, and the first run’s eleven, the real setup’s four, the setup on GitHub's six and Google sign-in's two. The case's later steps are listed, and are added one act at a time.
- **Acts:** a button or link for an act shows a note instead of acting, unless a later step draws what it does: then it leads to that step (the first run's suggested requests). Nothing is sent anywhere.
- **For a capture:** `?bare` draws the page alone, without the bar and the marks, and every link keeps it (`chat.html?world=solo&task=T10&bare`). A page's state is its address: the world, the step and the task. `?project=` shows another project; the chats, tasks, board and changes pages follow it.

## Where each value comes from

temper is a generic engine, so its pages may not assume a workflow. Every value a page shows comes from a named source, and `pages.js` draws each through one helper per source:

| Source | What it is | Examples |
|---|---|---|
| Engine fact | what jig or temper keeps or computes | a phase; what a task waits on; a wake policy's next time; a change's stage; slots in use |
| Configuration | a name the deployment's configuration gives | a template (`chat`, `goal`); a charter (`Coordinator`); a host kind; the period and allotment |
| As written | words a person or an agent wrote, shown as written, with the writer | a title; a brief; a reply, rendered from Markdown; a note; a reason in a task's history |
| Ops application | what a later application reports, beside temper's facts and never as them | a worker's load and disk |

What a task waits on is jig's, computed by the engine: a party, its delegates, news. Where it is news, the page narrows it from the task's wake policy, mechanically: *your words* when a party's words wake a task that party requested (a chat at rest), *T12's end* for a subscription to another task's state, *until 09:05* for a timer alone. *Waiting on your words* is not *needs you*: nothing is held up, and only an entry (a question, a proposal, an escalation, a person task) is in the inbox, where its task waits on *an answer from you*. temper's client words these statuses its own way ("Waiting for you", "Parked"; the client README, 3.5).

Anything else on a page is the UI's own words, and those name no workflow: the director, an owner or a check appear only in what was written or configured. The inbox, for example, says "Nothing needs you." and stops; "Next check at 09:10" appears only where the director wrote it, in its reply. Its updates are derived, as jig's inbox derives them: replies in the person's chats, and results of the tasks the person requested.

**View** has two switches. **Annotations** (on by default) shows the case's names beside temper's numbers (`B`, `f1`) and the gaps of temper-on-temper.md's section 6 that a page draws ahead of temper (`gap 6.4`), all in slate, never the product's palette. **Sources** (off by default) underlines each value in its source's colour, and hovering one names where it comes from.

## Drawn ahead of temper

What temper does not have yet, drawn as temper-on-temper.md's section 6 wants it and marked with its gap while **Annotations** is on:

- **6.3, plan usage:** a program's 5-hour and weekly windows on the accounts page, the week with its pace; the director's check reads them. A ChatGPT plan shows none: its usage is ChatGPT's settings' to show (jig's `domain/providers.md`, 2.6), so its row shows its limit's state and "Manage usage ↗", which opens them.
- **6.4, worker health:** the ops strip on the workers page, an ops application's report of load and disk.
- **6.5, open findings:** a finding on a goal's page, the board and the changes page, where a failure that matches an open finding attaches to it and leaves the queue running.
- **6.7, fleet occupancy:** a worker's free slots, and the director's read of them.

Marked "ahead" (slate, with the annotations): what the owned Forgejo’s rows and the first run draw before temper has a mission for it. **B4 M4:** the set-up’s first repository, and "New repository", made by temper with the admin’s token once temper-15 moves that token into the engine. **jig:** a project made after the first start; jig makes projects only at a deployment’s first start. **welcome:** the first run’s welcome and an explicitly chosen demo, with no mission yet. **demo’s end:** retirement after the first personal forge connects, with no mission yet. **sign-in:** "Open Forgejo" signing the person in through temper, as Forgejo’s OpenID provider (`../../draft/sign-in.md`, section 5); until then, the person signs in with the password the preparation wrote. **display:** an account's email, "ChatGPT · sam@example.com"; jig carries no display yet, and the client README's 5.5 drops it from the heading until it does. **return:** the engine's loopback listener answering ChatGPT's return with a redirect to the client's page that started the sign-in; jig and skein's `oauth.md` draw no answer. **words:** a sign-in's endings that 5.5 has no words for: a subject another person holds, and the port taken. **Google:** signing in to temper with Google, through jig's `Mint`, and the return route's pages (`../../draft/sign-in.md`, section 4). **provider:** temper as Forgejo's OpenID provider: its authorization, and "Open Forgejo" on a managed Forgejo that signs in through temper (section 5). **sign-out:** signing out of temper leaves Forgejo signed in (section 6). **sketch:** Claude's subscription in Connect a model, in outline. **limited chat:** where a chat shows its account's limit.

Two pages follow jig's pages, which temper's client leaves out (the client README, 5.3 and 11): **Tasks**, every task of the project as one tree (jig's `client/domain/pages.md`, 6.3), and **Workers**, the deployment's hosts (7).

Not drawn: two ideas that would change temper's engine, not only its pages. A charter or template would name the pages where its tasks lead, so the board could open with the director's line; an agent would set a short status line on its task with a tool, shown as written beside its phase. They wait for a design of their own.

## The first run

`world-first-run.js` is the deployment `temper init` writes (`../../protocol/setup.md`, 8.2 and 8.3): no project, which the playground's start or the first repository makes (`../../draft/first-run.md`, 2.4), the charters `chat` and `coding`, one worker in temper's process, three forges (the demo's fake, GitHub not connected, a Forgejo here not set up), and no account, repository or forge started yet. Every word an agent says is the demo's script (8.4). [first-run.html](first-run.html), the address the documents cite, opens it at its first step. Its steps:

- **0. The link opened:** the welcome offers Try the playground or Set up temper. Chats behind it has the setup strip, no model and a disabled send button.
- **1. The demo chosen:** the engine asks temper box for the demo’s fakes, makes the project Playground, adds the Scripted model’s account and adopts playground/widget into it. The chats page is the demo's start (client README, 5.6): the banner under the navigation, as on every page; a chatbar with no task and no number; the context line; the three suggested requests.
- **2. A request sent:** "Add CSV export to widget" starts the chat T1; its agent delegates the change T2, whose agent T3 edits `src/main.rs`. T2's card says Running with no step ticked (5.7).
- **3. The change landed:** T2's card says Landed, every step ticked, "CI passed. Landed · merge commit e3b8d41"; T1 says so in its script's third turn.
- **4. A text-only request:** "Add a dark theme to widget", the chat T4, answered in words; its script's `finish` ends only the turn (setup.md 8.4), so T4 waits at rest for Sam's next words, its composer open, as T1 does.
- **4a. Connect a model, from another computer:** `step=model-choose` draws Choose, with Continue with ChatGPT and the secondary button “This browser isn't on temper's computer ›”.
- **4b. Signing in, with the browser elsewhere:** `step=model-elsewhere`, `way=chatgpt`, draws Signing in, its external sign-in link in another tab and the paste unfolded. Accounts draws that sign-in with the paste folded.
- **4c. The whole address pasted:** `step=model-pasted`, `way=chatgpt`, draws the secret field written, hidden, and Finish connecting enabled; the button leads to step 5. Accounts has the same paste, folded. No address is sent or stored by a mockup.
- **5. A model connected:** Sam follows the banner's Connect a model, chooses ChatGPT and finishes by pasting from the browser on another computer; the flow ends on Ready, with Start a chat and Manage usage. The Scripted model leaves Accounts, where ChatGPT · sam@example.com is the one row. The model's half of the demo ends (4.8): the banner says a playground forge alone, the chats page has no demo's start, and the composers' line reads "Using ChatGPT plan · gpt-6-luna · Manage usage". The galleries below Connect a model, Accounts and Chats draw the rest of the ChatGPT experience (below).
- **6. Setting up a Forgejo:** chosen in the flow, its defaults kept (work/hello, a new project hello); the box has fetched and is preparing. The flow shows each step, and the banner says where it is.
- **7. A Forgejo connected:** temper connected as its bot, made work/hello and adopted it into the new project hello; its first CI run passed. The flow ends on the new project, opened with no chat yet. The demo is retired: forge 1 stops and disappears, T1 and T4 end at 21:19 with the reason, and playground/widget leaves Playground’s repository lists. Its landing and transcripts stay readable; Recent landings names it “playground/widget · the playground’s, ended”. The banner is gone.

## The first run, set up

`welcome.html` has a bare topbar (temper, Sam, Sign out), and Welcome → Forge → Model → Ready. Its default view follows the deployment: the welcome before any forge is acted on, Model while a forge sets up without a model, then Ready. Once the demo runs it shows a mock-note linking to Chats. `?at=welcome|forge|model|ready` selects a view; `?forge=2|3|elsewhere` keeps the forge’s details inside the wizard. The forge cards and setup pieces are shared with `connect-forge.html`. Later opens Chats; trying the demo explicitly switches to first-run step 1.

`world-first-run-real.js` reuses the first-run world’s fresh `initial`:

1. **The link opened:** Sam chooses Set up temper, then Forgejo on this machine, with its defaults.
2. **Setting up a Forgejo:** the box fetches Forgejo and its runner; Model opens while that proceeds, with an Each step disclosure: ChatGPT or an OpenAI API key, as the default document offers them. The gallery sketches Claude's subscription, where a configuration offers it.
3. **Signed in:** Continue with ChatGPT opens ChatGPT's sign-in in the same tab; Sam approves, and the engine's listener sends the browser back to the wizard. ChatGPT · sam@example.com is the only account, and Almost ready waits while the forge starts. Open hello is disabled. Had the forge connected while Sam was at ChatGPT, the same view would read work/hello is ready, as at step 4.
4. **Ready:** the forge is connected as temper; work/hello is adopted into hello with its first CI passed. Open hello opens that project with no chat yet. The demo, never started, is retired and nothing of it shows.

The person meets the demo as **the playground** (with the user, 2026-10-23): "Try the playground", the banner "This is the playground: a scripted model and a playground forge.", Scripted model, Playground forge and playground/widget; the scripts say "Playground forge" and "a scripted change". The design keeps the demo as its concept's name (`application.demo`, the demo's halves); setup.md 8.4 holds the scripts' words, and the client README 5.6 the page's (`../../draft/first-run.md`, 2.3).

The demo’s explicit states are `not started`, `running`, and `retired`; an absent state retains the older worlds’ running behavior. Its model and forge halves and context draw only while running. The first usable account from another provider ends the model half and removes the Scripted model’s account; the first connected forge of the person’s retires the demo’s forge and repository and ends its open chats, keeping history.

While the demo is not started and a forge or model is missing, the setup strip takes the demo strip’s place. It states what is missing, shows any forge setup status with View, and links to Continue setting up and (until a personal forge connects) Try the playground. Welcome, Accounts, Forges, Connect a forge and Connect a model omit it. With no account, a composer says “No model yet · Connect a model” and cannot send.

Where the documents disagree, or leave a word open, the mockup takes one side and says so here:

- **T1's next run takes the new account** (step 5), where temper's stays on the demo model. The draft says the demo model "stays the demo project's" (`../../draft/first-run.md`, 2.2). setup.md 8.3 says a task with no model chosen takes the first of its charter's models that has a usable account, and the demo model is last in both charters. jig resolves that default once, at the task's Create, and records it, so a new chat's first run takes gpt-6-luna on the ChatGPT plan while T1, created before it, keeps the demo model. The mockup draws T1 on gpt-6-luna too: re-resolving a task's default for each run is jig's to decide.
- **The coding agent's title is the change's** (T3, "Export widget's table as CSV"): setup.md 8.4 makes the change's title its agent's brief, and a chip is the spec's first line. The other worlds title that agent "Produce TN's branch", a stand-in for a title temper's procedure does not word yet.
- **The dark theme's words** are a stand-in. Their content is the demo's follow-up (`../../draft/first-run.md`, section 8). The empty search and the fallback are not drawn: they take step 4's shape.
- **"Connect a forge" is a link,** to `connect-forge.html`. The client has no `ConnectForge` yet, so until it does the button is disabled (client README, 5.6).
- **The forges page** shows only the person’s forges (connecting, setting up, set up failed, connected, stopped), each folded until opened, and the playground’s. A forge not acted on is a choice in the “Connect a forge” flow: Choose → Details → Setting up → Ready for a Forgejo, and Choose → Connecting → Ready for GitHub (below), ending on the new project. The banner shows the set-up’s status and a link to it; the Forges nav item’s dot is amber while setting up or connecting, red when a forge needs the person. The page and the flow are forge.md 5.4's. Mockup-only galleries on both pages compare the flow’s other paths and the other forge rows, including managed and connected Forgejo. The flow’s “A Forgejo elsewhere” includes the setup entry that connecting as its admin needs.
- **Accounts and Connect a model** follow the forges' shape (with the user, 2026-10-23). The page stays Accounts; the flow and every entry point say "Connect a model": the banner, the composer's "No model yet", the page's "+ Connect a model". The flow is Choose → Signing in → Ready: Continue with ChatGPT opens its sign-in in the same tab, and the engine's listener sends the browser back to the page it left; the secondary button keeps Signing in here for a browser on another computer (B, below). A sign-in drawn under way offers the paste, folded on the engine’s machine and in Accounts, unfolded after the secondary button. A key has Details in Signing in’s place. The welcome's Model reuses its Choose, and the sign-in returns to Almost ready. ChatGPT is drawn whole; an API key's form and Claude's subscription are there for the context, not designed. Each account is a folded row: its name and holder, its deciding source's state, and a line of its models, what it pays for and its tasks; opened, the state's words and acts from 5.5's table, its sources in order, Pays for, Used by, Waiting for it, and its acts. The Accounts nav item's dot is amber while an account is limited, red while one needs the person. Mockup-only galleries draw, below Connect a model, each other ending, back in temper; below Accounts, the ChatGPT row in each state of 5.5's table, and a key's and a Claude subscription's rows; below Chats, a plan's first-use card and a chat whose plan is limited.
- **A plan pays for its holder's own work, and for projects where the holder is the only member;** a key or a program's login, with no uses given, for its holder's own work. A limited plan's tasks wait for it; where the account also holds a key, "Use the API key first" reorders its sources, and never moves a task to another account.
- **B is built:** a browser on another computer finishes by pasting the whole address of the page that could not be reached into temper; the tunnel still works (`ssh -L 1455:127.0.0.1:1455 HOST`, `../../../guide/box.md`, step 4). The first run’s 4a–4c draw it; Connect a model’s gallery draws every paste state (below), and Accounts draws the same paste folded. Words and controls follow temper-client-view's `accounts/words.rs`, `pages/connect_model.rs` and `pages/accounts.rs` as built.
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

The earlier first-run mockup, a story of its own, also drew "Reset demo", a turn's "Just now", the composer's placeholder, "Landed into main" and the pull request's link on the card. The client leaves each out (client README, section 11), and so does this world.

## The first run, on GitHub

`world-first-run-github.js` is the real setup on GitHub (with the user, 2026-10-23), for one person: the same fresh deployment, Set up temper, then GitHub's card. Its steps:

1. **The link opened:** Connect a forge's GitHub card is one button, Connect GitHub, and a quiet "Use an App of your own ›", which opens the App's Details and is not designed further. The same card is on Connect a forge, from Forges. Nothing asks for an organization.
2. **On GitHub, its App:** Connect GitHub sends the tab to temper's manifest page, on the clients' listener, which posts temper's manifest to GitHub as it loads. GitHub's form makes the App under Sam's own account, always. Nothing in temper shows a connect under way: the browser's back, or Connect GitHub again, starts afresh.
3. **On GitHub, where to install:** temper's redirect kept the App's key and sent the tab to the App's install page. The App is public, so GitHub lists Sam's account and every organization Sam belongs to: the organization menu is GitHub's own. Sam's account and an organization Sam owns offer Install; one Sam is only a member of offers Request. Had Sam come back to temper here, Connect a forge would show Connecting: the App made and installed nowhere, and Install on GitHub.
4. **Installed:** GitHub's own choice of repositories, all of them (its default) or those selected. Install sends the tab to temper's setup route, which takes the installation and sends the tab back where Connect GitHub started: the wizard, Connect a forge or Forges. Ready lists the installation's repositories to start in; "Choose its repositories on GitHub" opens the installation's page, whose Save comes back the same way. The playground, never started, is retired.
5. **A repository adopted:** the one Sam chose, into a new project named after it (marked ahead · jig).
6. **Ready:** a ChatGPT plan, as in the real setup.

`github.html` draws GitHub's pages in GitHub's look, each under a slate strip with its address and how the tab got there, and with a slate link to temper as it would be had Sam left there. `?from=welcome|connect-forge|forges` says where the tab goes back. GitHub's choices are carried in the address, so every page after them agrees: `granted=ACCOUNT:all` or `granted=ACCOUNT:NAME,NAME`, the installation's account and repositories, and `pick=NAME`, the repository chosen at Ready.

Where GitHub's way can't go on, temper's listener answers its own short page, saying why, with Back to temper, in place of sending the tab back. Two are drawn, below Connect a forge in this world and at `github.html?at=refused&why=request|ended`:

- **A request, not an installation:** an organization's member who isn't its owner can only request the App, and GitHub comes back with no installation. temper can't wait for an owner's approval yet, since an installation approved later isn't used: install it on your own account, or on an organization you own.
- **A link that has ended:** the state was used, expired, or temper restarted since. Start again from temper.

## The files

- `style.css`, `terminal-chat.css` and `terminal-pages.css`: the product's look, from which `temper.css` takes its rules (`../browser.md`, 2.5).
- `mockup.css`: the mockups' own marks and bar, never the product.
- `pages.js`, `world-reference.js`, `world-solo.js`, `world-first-run.js`, `world-first-run-real.js` and `world-first-run-github.js`: every page and its worlds; each HTML file is a shell that loads them, `first-run.html` opens the first run, and `github.html` draws GitHub's side.
