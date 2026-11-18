# Trying temper: the first ten minutes

Draft, 2026-11-18, with the user. The first journey of the UX draft
(J0, "the try"), step by step: from a person typing `./temper box`
after the download to their first project open, in the terminal and
the browser at once. First on paper, here: each step, what the person
does, what each screen says, what temper does behind it, how long it
takes and how far it is built. The mockups come next, a web frame and
a terminal frame for each step (section 5). Where the journey trips,
section 4 proposes a change; the user's own changes are to come.

## 1. In one page

- **What the try is for:** a person who has just downloaded temper sees,
  in about ten minutes and connecting nothing, the line held on a
  change: written by an agent, checked by code, reviewed by an agent,
  and merged at the commit that was checked, with no merge an agent's.
  Then they set temper up for real.
- **Where it ends:** their first project, Hello, open on its Home in the
  browser and on its launchpad in the terminal, with `work/hello` on a
  Forgejo of their own and their model signed in. The first day with
  their own code starts there.
- **Three parts:** the start (steps 1 to 3, a minute), the playground
  (4 to 9, about three minutes) and the set up (10 to 13, about five).
- **Two screens, one door each:** the browser acts and the terminal
  follows it, saying where the set up stands. The terminal acts once:
  Claude Code's sign-in, which it lends the terminal for (step 11).
- **How far it is built:** the start, the welcome, the playground's
  start and the set up as far as the Model step are built and recorded
  (`../client/web/journeys/welcome-playground/`,
  `../client/web/journeys/welcome-set-up/`). From the playground's first
  request to its landing, and from Model to Ready, is built and played
  by the real loop's stories, the process tier and the client's world,
  but no journey records it.
  No time below is measured; each is an estimate from what runs.
- **What trips it** (section 4): the line the playground holds is
  nowhere said where the person looks; no code that changed can be
  seen; its way out is worded two ways; and Claude needs Claude Code
  installed, which nothing says before it fails.

## 2. The steps

Each step says: **You**, what the person does; **Terminal** and
**Browser**, what each shows, in its words; **Behind**, what temper
does; **Time**; and **Built**, how far. Task numbers are illustrative:
the chat is task 1 on a fresh box, its change task 2.

### Step 1. `./temper box`

- **You:** in a terminal, in the directory the download landed in,
  `./temper box`.
- **Terminal:** the TUI takes the whole terminal at once, before the box
  runs (`../client/tui/box.md`, *The box's stages*):

  ```
    ▄█▀▀   temper 0.0.0 (b948f40d6df1)
   ▄█▄█▄█  temper box · a whole deployment on this machine

   Welcome. Set temper up in your browser: open this link on this machine.

   The link comes here once the box has started.

   It signs you in as this temper's owner, once, within 2 hours.

   In the browser  1 Welcome · 2 Forge · 3 Model · 4 Ready
  ```

  with, on the quiet line, "● Connecting" and the stage on its right:
  "Downloading temper-box, once for this build: 12.3 of 24.1 MB", then
  "Starting the box…", then the box's last log line.
- **Browser:** nothing yet.
- **Behind:** `temper` checks the machine (git 2.41 or later,
  `/usr/bin/rg`, `/bin/sh`: `../release.md`, 4), downloads `temper-box`
  of its own build from fabrig.ai once, checked against the site's
  sums (`../client/box-binary.md`, 3), and starts it behind the TUI. The
  box binds its port first, draws the link's secret, prints it to the
  TUI, then starts serve, a worker and its steps
  (`box-link-first.md`).
- **Time:** the download, a few seconds on a good line; the start,
  about 4 s.
- **Built:** yes; the stage's line on the quiet line is in flight
  (`../client/tui/box.status.md`).

### Step 2. The link

- **You:** open the link in a browser on this machine: a click where the
  terminal makes links clickable, else a copy.
- **Terminal:** the link in place of "The link comes here…", whole and
  wrapped: `http://127.0.0.1:8367/jig/v1/sign-in/link?secret=…`. The
  TUI never opens it.
- **Browser:** the link signs the person in as the box's owner and opens
  the welcome.
- **Behind:** jig's sign-in spends the link: one live at a time, two
  hours; opened again once spent, a fresh one is printed.
- **Time:** seconds.
- **Built:** yes, recorded (`welcome-playground`, 01).

### Step 3. The welcome

- **You:** read; choose between setting up and the playground.
- **Browser:** the welcome's first step (`../client/pages/welcome.md`,
  1), under a top bar with temper's mark and the four steps, Welcome,
  Forge, Model, Ready:
  - "Welcome to temper", "Reliable, scalable and flexible software
    factory. You describe what you want and how you want it done, and
    it happens.";
  - the card "Set up temper", first and the default: "Your forge: a
    Forgejo temper runs for you, or GitHub. Your model: ChatGPT or
    Claude.", "About five minutes", the button "Set up temper";
  - the card "Try the playground": "A scripted model and a forge, built
    in. A taste of how temper feels, with nothing to connect.", "Fully
    offline · Ready at once", the button "Try the playground";
  - "Tried the playground first? Set up whenever you like; the
    playground then steps aside.";
  - "temper box · on this machine, to try it · more later, from
    Settings · Run temper on a server ↗".
- **Terminal:** "Setting temper up in your browser", then "✓ Signed in
  as the owner", "▸ 1 Welcome", "· 2 Forge", "· 3 Model", "· 4 Ready",
  and "The steps are the browser's; this screen follows them."
- **Time:** as long as the person reads.
- **Built:** yes, recorded (`welcome-playground`, 01; `welcome-set-up`,
  01 and 02).

### Step 4. "Try the playground"

- **You:** press "Try the playground".
- **Browser:** a new chat in the project Demo
  (`../client/pages/welcome.md`, 2):
  - the banner above it: "This is the playground: a scripted model and
    in-memory forge. Use the wizard to set up temper", its last words a
    link;
  - the rail: Demo and its repository, `playground/widget`; Home, Needs
    you, Work, Ask temper, How work runs; "New draft or change"; at its
    foot "Set up temper";
  - the chat's head: "Starting", then "Ready"; `playground/widget`;
    "Say what you want; the agent works in playground/widget.";
  - three suggested requests: "Add CSV export to widget", "Add a dark
    theme to widget", "Fix widget's empty search state";
  - the composer, its model "Scripted model".
- **Terminal:** Demo's launchpad, its headline "This is the playground:
  a scripted model and in-memory forge. Set temper up in your browser
  when you're ready to leave it.", and on the quiet line "● Live · Demo
  · Scripted model" and "playground".
- **Behind:** the box starts its playground group, a fake model and a
  fake Forgejo. The engine makes the fake forge's record, the scripted
  account and the project Demo, and adopts `playground/widget` into it
  (`../protocol/setup/playground.md`, *Its start*). The repository is a
  small Rust program whose `main` prints a table of names and counts.
- **Time:** a few seconds.
- **Built:** yes, recorded (`welcome-playground`, 02).

### Step 5. "Add CSV export to widget"

- **You:** press the first suggestion.
- **Browser:** the chat opens in place of the new chat, its first turn:
  "I'll delegate this change in playground/widget. It will push a
  branch, open a pull request, check CI and land into main on
  Playground forge." Under it the change's card (`../client/view.md`,
  5): "T2 · CHANGE", "Running", its title "Export widget's table as
  CSV", its steps "Branch pushed", "Pull request opened", "CI passed",
  "Landed", none done yet, and "Task 2 ↗".
- **Terminal:** the launchpad's "Where you left off" gains the chat,
  "Demo  Add CSV export to widget, a chat · just now · Working". Opened
  there, the chat shows the same turn and a line for the card, "✦ T2 ·
  CHANGE Export widget's table as CSV · Running".
- **Behind:** the scripted model plays the chat's script: the text, then
  a delegation to the change procedure on `playground/widget`, landing on
  `main`, then a wait.
- **Time:** a second or two.
- **Built:** yes, played by the real loop's playground stories; not
  recorded.

### Step 6. The change runs

- **You:** watch the card's steps tick, or open "Task 2 ↗".
- **Browser, the card:** "Branch pushed", "Pull request opened", "CI
  passed" and "Landed" tick in turn.
- **Browser, the task's page** (`../client/pages/task.md`, 3): "Change
  · Task 2 · Demo", its title, "Running", "PR #1 · playground/widget ·
  BRANCH → main"; six steps, "Producing", "Opening", "Checking",
  "Gating", "Queued", "Landed", the current one's line under them
  ("Producing its branch.", "Waiting for CI at its head.", "Waiting for
  its gates at its head.", "Merging at its head."); "Its tasks", task 3
  writing it and task 4 reviewing it; "Gates at its head", "CI at its
  head · Passed" and "Gate 1 · review · blocking · Approved"; its
  bounds. No "View on forge ↗": the fake forge has no pages.
- **Terminal:** the chat's card line, its state as it changes.
- **Behind,** each step code, but for the agents' two:
  1. *Producing:* an agent, task 3, reads `src/main.rs`, edits it so
     that `widget --csv` prints a header line `name,count` and a line a
     row, and finishes the change "Add CSV export to widget". The
     worker, not the agent, commits and pushes the branch.
  2. *Opening:* the procedure opens the pull request on the fake forge.
  3. *Checking:* the fake forge's CI, one check, passes at once at the
     branch's head.
  4. *Gating:* gate 1, the stock rule's agent review, blocking: an
     agent, task 4, reviews the change at that head and approves it,
     "Checked widget CSV export." (the stock rule:
     `../protocol/configuration/stock.md`, kind 2).
  5. *Queued, then landing:* first in `main`'s queue, the procedure
     merges at the head CI and the review saw; neither agent holds a
     tool that merges.
- **Time:** not measured; the model answers at once, so the agents'
  start-up and the engine's steps make it, perhaps half a minute.
- **Built:** yes, played; not recorded.

### Step 7. Landed

- **You:** read the chat's last turn.
- **Browser:** the card "Landed", its four steps ticked, the change's
  words, "widget --csv prints the table as CSV: a header line, then a
  line for each row. This was a scripted change."; and the chat's
  turn: "CSV export is available from widget's table. The change
  landed on Playground forge. This was a scripted change." The task's
  page reads "Landed · merge commit ABCDEFG".
- **Terminal:** the card's line, "Landed"; the launchpad's row, its
  state as Home words it.
- **Behind:** the change's result wakes the chat, which plays its third
  turn and waits again: the chat stays open.
- **Built:** yes, played; not recorded.

### Step 8. The other two requests, if the person likes

- **You:** "Add a dark theme to widget" or "Fix widget's empty search
  state", or anything typed.
- **Browser and terminal:** one text each, saying what it would change
  and why, then "I am the scripted model and play this from a script,
  so nothing is changed: connect a model from Accounts to have the work
  done." Anything else is answered that the scripted model answers only
  the suggested requests, and to "connect a model from Accounts".
- **Built:** yes; the way out it names is wrong (4.3).

### Step 9. "Set up temper": leaving the playground

- **You:** the banner's "set up temper", or the rail's "Set up temper".
- **Browser:** "Leaving the playground…" in the banner's place, then the
  welcome's Forge step.
- **Terminal:** the welcome again, at "▸ 1 Welcome" while the browser
  shows Forge: the terminal follows the store, empty again, not the tab
  (`../client/tui/box.md`, *The welcome*).
- **Behind:** the reset, at once and with no question: Demo deleted
  with its chats and tasks, the fake forge removed, the scripted
  account removed (`../protocol/setup/playground.md`, *Its reset*).
- **Time:** a few seconds.
- **Built:** yes; not recorded.

### Step 10. Forge: Forgejo on this machine

- **You:** on "Forgejo", "Run on this machine"; in its sheet, "An empty
  repository" (the default, `work/hello`), then "Run on this machine".
- **Browser:** two cards, Forgejo's first, "A forge of your own: temper
  starts it on this machine and manages it. You can import existing
  repositories into it, from GitHub or elsewhere.", then GitHub's. The
  sheet "Forgejo on this machine", "temper starts it here and connects
  to it; you connect your model meanwhile.", and "Start it with": "An
  empty repository" ("A README and a starter CI check."), "Copies of
  public repositories", "Nothing for now". Run moves on to Model at
  once.
- **Terminal:** "▸ 2 Forge · fetching Forgejo…", then "setting up
  Forgejo…", "making its first repository…", then "✓ 2 Forge".
- **Behind:** the box fetches Forgejo and its runner, pinned and
  checked, starts them, sets temper up as their admin and makes
  `work/hello`, unadopted until Ready.
- **Time:** the fetch, the longest wait of the try, runs on under the
  Model step: not measured.
- **Built:** yes, recorded to Model (`welcome-set-up`, 03 to 07).

### Step 11. Model: the person's own plan

- **You,** with Claude, the default: "Sign in to Claude Code"; in the
  terminal, Claude Code's own sign-in, which opens Anthropic's page in a
  tab; approve there; come back to the browser and press "Continue".
  With ChatGPT: "Continue with ChatGPT", sign in in a new tab, and the
  page moves on by itself.
- **Browser:** "Connect a model", "temper's agents run on a plan you
  sign in to."; the Claude card, "Your Claude subscription, through a
  Claude Code temper keeps apart from your own.", "Signs in from the
  terminal where temper runs"; the ChatGPT card. While Claude Code signs
  in, the note "Claude Code is signing in, in the terminal where temper
  runs" and Cancel. After: "✓ Claude subscription", "through Claude Code,
  signed in", "Continue". Beside it all, the pane "Forge and model",
  Forgejo's set up line with its bar, and the model's.
- **Terminal:** cleared and lent to Claude Code
  (`../client/tui/box.md`, *Lending its terminal*): "temper: Claude
  Code signs in here. Approve it on Anthropic's page, in the tab it
  opens; if that page shows a code, paste it here. temper comes back
  when it's done, or when you press Cancel in the browser.", then
  Claude Code's own screens. Back, the notice "Claude Code signed in"
  and "✓ 3 Model · Claude subscription, through Claude Code".
- **Behind:** temper runs Claude Code in an environment of its own,
  apart from the person's, and never reads its login; once usable, it
  tries the account with one short turn, the try, whose line says
  "Trying: Claude answers", then "answered in N s".
- **Time:** a minute or two, the person's.
- **Built:** yes, but not recorded past the Model step's first frame;
  the lent terminal's blank rows are in flight.

### Step 12. Ready: the first project

- **You:** keep the offered name, Hello, or write another; "Create your
  project".
- **Browser:** "Ready", "Everything is connected. Create your first
  project, with work/hello in it, and start working."; the field "Your
  first project", "Hello"; "Create your project"; "Rename it whenever
  you like."; the pane: "✓ Forgejo · connected as temper · work/hello
  made · Open Forgejo ↗" and "✓ Claude subscription · through Claude
  Code, signed in · answered in N s".
- **Terminal:** "▸ 4 Ready".
- **Behind:** the project is made on the stock way and `work/hello`
  adopted into it, the adoption running on (`first-run-projects.md`,
  4).
- **Built:** yes; not recorded.

### Step 13. The end: Hello

- **Browser:** Hello's Home: "Hello is ready.", "Your agents run on your
  Claude subscription, and Forgejo on this machine.", and the card "To
  begin", "Optional", its steps: `work/hello` in Hello ("Adopting
  work/hello into Hello…", then "temper can change its code now."), and
  "Tell temper how you work", to Ask temper (`../client/pages/home.md`,
  1).
- **Terminal:** Hello's launchpad, the quiet line "● Live · Hello ·
  Claude Opus 5.5 · high".
- **Built:** yes; not recorded.

### After the try: closing and coming back

- **Ctrl+C** in the terminal asks "Stop the box?" while work runs, and
  stops it at once otherwise; the browser goes offline with it.
- **`./temper box` again** shows "Starting temper" and its four steps,
  the box, temper, the sign-in and the projects, then Hello's launchpad,
  signed in with nothing to open: the link is for a first run alone
  (`../client/tui/box.md`, *The starting screen*). Whether the
  browser's tab, offline meanwhile, comes back by itself is not checked.

## 3. The ten minutes

| Steps | Part | Time | Waits on |
|---|---|---|---|
| 1 to 3 | the start | under a minute | the download of `temper-box` |
| 4 to 8 | the playground | two to three minutes | the agents' start-up |
| 9 | leaving it | seconds | |
| 10 to 12 | the set up | about five minutes | Forgejo's fetch, under the person's sign-in |
| 13 | the end | | |

None of it measured: the try's first recording measures it (5).

## 4. Where the try trips

Each with what happens today and a proposal, the default until the
user says otherwise. The first two are about the try's point.

### 4.1 The line is held but never said

- **Today:** the chat's card shows "Branch pushed", "Pull request
  opened", "CI passed" and "Landed": no review. The review is on the
  change's page, one click away, as "Gate 1 · review · blocking ·
  Approved", and its words on task 4's page, two; the commit CI and the
  review saw,
  and the commit merged, are said nowhere side by side. The chat's last
  words are "CSV export is available from widget's table." A person who
  does not open the task's page leaves with "an agent changed some
  code", the line unseen.
- **Proposal:** the card's steps gain "Reviewed", between "CI passed" and
  "Landed", for any change with a review gate; and its landed state names
  the commit, "Landed at ABCDEFG, the commit checked and reviewed". The
  playground's last turn says the line in four short lines, each a
  link: written by an agent (task 3), checked by CI at ABCDEFG, reviewed
  by an agent at ABCDEFG (task 4, "Checked widget CSV export."), merged
  at ABCDEFG by temper, no agent holding a merge.

### 4.2 No code to see

- **Today:** the fake forge has no pages, so no "View on forge ↗", and
  nothing in temper shows a change's diff. The person never sees what
  the agent wrote.
- **Proposal:** for the try, the scripted change's result says its diff,
  as a short code block in the chat, cheap and scripted. Later, and for
  every change, a "What changed" on the task's page, its files and
  lines, read from the forge's git at the change's head: a feature of
  its own, past the try.

### 4.3 The playground's way out, said two ways

- **Today:** the banner and the rail's foot say "Set up temper"; the
  scripted answers say "connect a model from Accounts", and the model
  picker in the playground offers "Connect a plan or key ›" for OpenAI
  and Claude (recorded, `welcome-playground`, 02), though nothing can be
  connected there (`first-run-projects.md`, 3). Both are known
  (`../protocol/setup/playground.status.md`). The banner says "Use the
  wizard", a word the welcome never uses.
- **Proposal:** one way out, said once: the scripted answers end "Set up
  temper, from the banner, to have it done for real."; the playground's
  picker offers no connect; the banner reads "This is the playground: a
  scripted model and an in-memory forge. Set up temper when you're
  ready; the playground goes with it."

### 4.4 Claude needs Claude Code, unsaid

- **Today:** Claude's way runs the person's `claude`, which temper does
  not install (`../release.md`, 4). The welcome's Claude card, the
  default, does not say it is needed; what a press does without it is
  not checked.
- **Proposal:** the Claude card says, before it is pressed, whether
  Claude Code is on this machine: with none, its muted line "Needs
  Claude Code on this machine: install it, then press here" and ChatGPT's
  card the default.

### 4.5 Leaving the playground takes it all, at once

- **Today:** one press deletes Demo, its chats and its change, with no
  question; the person may still be reading.
- **Proposal:** keep the one press, and say what goes before it: the
  rail's "Set up temper" and the banner's link open a short confirmation,
  "Demo, its chats and its change go; your set up starts", "Set up
  temper", "Stay".

### 4.6 The terminal shows the playground, but offers nothing in it

- **Today:** the TUI shows Demo's launchpad; its suggested requests are
  the browser's alone, so the terminal, the second door, sits idle.
- **Proposal:** the launchpad's Start lists the three requests while
  the playground runs, each sent as its words: the same chat, from the
  terminal.

### 4.7 "Say how you work" is missing from the try

- **Today:** the try shows "Agents do the rest" and "Code holds the
  line"; nothing points at the way the change just followed, Agents
  review, which How work runs holds.
- **Proposal:** the playground's last turn ends with one line, "It ran
  as Demo's way says: CI and an agent's review before landing. How work
  runs ›". Later, a fourth suggestion that changes the way ("Ask me
  before landing") is its own design.

## 5. The mockups to make

Next, once the user has read this. One frame a step, web and terminal
side by side, in the greenfield mock's look (`../client/web/greenfield/`)
and the terminal's (`../client/tui/journeys/theme.css`), each step's
words as section 2 has them, and the proposals of section 4 drawn under
a switch, so that each can be seen with it and without:

| Step | Web | Terminal |
|---|---|---|
| 1 | | the welcome, downloading |
| 2 | | the welcome, its link |
| 3 | the welcome | signed in, step 1 |
| 4 | Demo's new chat | Demo's launchpad |
| 5 | the chat and its card, running | the chat |
| 6 | the task's page, gating | the launchpad, working |
| 7 | the chat, landed (and 4.1, 4.2, 4.7) | the chat, landed |
| 9 | leaving (and 4.5) | the welcome, step 2 |
| 10 | Forge, its sheet | step 2, fetching |
| 11 | Model, Claude Code signing in | lent to Claude Code |
| 12 | Ready | step 4 |
| 13 | Hello's Home | Hello's launchpad |

Recorded frames stand for steps 1 to 4 and 10 where they exist; a
recording of the whole try, once its changes land, is a walk after
landing, and measures section 3.

## 6. Open questions

- **The try's order:** the welcome's default is "Set up temper", the
  playground second (the user, 2026-11-04). The try assumes the person
  picks the playground. Default: as it stands; the try is one of two
  first runs.
- **A machine without a browser,** a box over SSH: the link is on
  loopback. Default: out of the try; the second door from another
  machine is its own design.
