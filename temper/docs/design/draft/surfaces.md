# The surfaces

Draft, 2026-11-08, from a session with the user that day. The web, the
phone and the terminal reshaped for the whole story (`whole-story.md`):
what each page and screen holds, in words, with the whole story's
mockups as their look (section 13). It is drawn green field, on the
user's word: "take it as greenfield, doesn't matter what we have now,
let's design optimizing for the vision". It revises the greenfield
mock's rail (`../client/web/greenfield/README.md`), the client's rail, tab
bar and Settings (`../client/view.md`, 3 and `../client/pages/README.md`, *Settings*), and
the terminal's direction and what it leaves after its first pass
(`tui.md`, sections 3, 6 and 7): the launchpad as drawn here, the side
pane, needs from elsewhere and talking with a role become v0.1.0's
through `surfaces.md`, section 11. It wins where they differ
until its plans fold it in. temper/ui carries out the web, the phone and
the drafter's instructions, which are a chat's; temper/tui carries out
the terminal. Reasons: `whole-story.md` and `notes.md`.

## 1. In one page

- **Two surfaces, three acts.** The web is where a person decides and
  knows; the terminal is where they direct, working with agents at
  length (`whole-story.md`, section 2; `tui.md`, section 2). The phone is
  the web at a glance.
- **A rail of five, then the process.** Home, Needs you, Work, Team and
  Ask temper; then How work runs; Away, Deployment and the person at its
  foot (section 2).
- **Across projects where the person is.** Home and Needs you read every
  project; Work, Team and How work runs are a project's.
- **Chats leave the rail.** A conversation is filed with what it is
  about, a goal's or a role's, and is had in the terminal. Work lists the
  chats that started work, and Ask temper keeps its entry.
- **Settings splits.** How work runs is the project's process, changed
  by proposal (section 7); Deployment keeps accounts, forges, models,
  workers, members and limits (section 8).
- **Team is new:** each area's owner, the person or a standing role, and
  a role's card (section 6).
- **Every need says why it reached you,** and offers to stop asking
  (section 4).
- **Tell temper** files what's off as a finding for its area's owner
  (section 9).
- **The terminal** opens on a launchpad, holds design sessions with the
  draft beside them, and talks with roles (section 11).
- **One set of pages, filled as the person hands over more** (section
  12).
- **The mockups are the look,** and win where they differ from the
  greenfield mock (section 13).

## 2. The rail

- **Its order:** temper's mark, a link to Home; the switcher; "Home";
  "Needs you", with the count across what it shows in the signal colour,
  left out at zero; "Work"; "Team", while the project shows it (section
  12); "Ask temper"; a hairline; "How work runs"; then "+ New draft or
  change". At its foot: the Away switch, while it shows; on All
  projects it sets every project the person is in, and reads "Away on 2
  of 4" while they differ; "Deployment"; the person's menu; the night switch, as today (`../client/view.md`,
  3).
- **All projects** is the switcher's first row, once the person has a
  second project. On Home and Needs you it is where a sign-in lands. Work,
  Team and How work runs are a project's: chosen under All projects, they
  open on the project last open, the switcher saying which.
- **Chats leave the rail.** `Chats` keeps its address, reached from
  Work's Chats filter (section 5) and from a chat's bar. A conversation is
  filed with what it is about: a goal's on its goal's page, a role's on
  Team (section 6), Ask temper's under its own entry.
- **Goals and Changes fold into Work** (section 5). Their addresses stay,
  as Work's filters, so that links and tests hold.
- **Settings splits.** The rail's "Settings" goes: its How work runs and
  Repositories tabs become How work runs (section 7), and its Forges,
  Accounts, Models and Builders tabs become Deployment (section 8).
- **While To begin shows** (`../client/pages/README.md`): "Home", "Ask
  temper", a hairline and "How work runs", with "Deployment", the
  person's menu and the night switch at the foot, today's "Settings"
  being those two. A new project's Home asks for one deliberate choice,
  as today. Its mockup: `pair/first-run.html`.
- **"+ New draft or change"** takes `div.temper-rail-new`'s place: a
  dashed box whose menu offers "Draft", "A design session that ends in a
  goal", best had in the terminal; "Change", "Ask for one change"; and
  "Ask temper". A project at Pair (section 12) shows "+ New change" alone.
- **On a phone** the rail is the top bar, the switcher and the Away chip,
  and a tab bar of four closes the page (section 10). The terminal's line
  shell keeps no tab bar, as today.

## 3. Home

- **Across projects or one.** On All projects, Home is the deployment's
  for the person; on a project, that project's, as
  `../client/pages/README.md`, *Home on any project*, has it, with this
  section's parts added. The mockups draw All projects.
- **The intro:** the kicker, "Sunday 08:04 · you last looked 9 hours ago
  · away 23:10 to 08:00", the Away span while one ended since the last
  look; the headline, what landed across the projects and the needs
  sentence, "31 changes landed across four projects. Two things need
  you."; the brief, the status agent's lede; and its byline, on All
  projects "Each project's brief, its first line · the newest, vsr's, by
  the status agent at 07:52".
- **The brief rolls up the owners.** The status agent writes it from
  each area's owner's Now and report (`standing-roles.md`, section 7) and
  what the engine keeps, on the triggers it has today, a look with
  something new and none over an hour old (`../domain/forge/attention.md`, 3). Its
  instructions and its brief are temper/ui's alone, as the status agent's
  are.
- **On All projects,** one brief a project, as today, until a brief per
  person is designed: the headline is code's, counted across the person's
  projects; the lede joins each project's brief's first sentence, newest
  first, at most four; the byline names the newest. One brief across
  projects, written per person, is section 15's open question: which
  project's period funds it, which project its task belongs to, and
  whether it may read other projects' Nows.
- **Needs you,** at most three cards, oldest first: the project's chip,
  who asks and when; the question; one line of why it reached the person
  (`what-reaches-you.md`, section 5); the recommended choice first, dark,
  then the others; "Tell it something else", to Needs you at that card.
  "All needs" at the head's right.
- **Decided for you,** a row each, while it shows: the project, who
  decided and when; the question; what was taken, in weight; why it was
  allowed; "Overturn" (`what-reaches-you.md`, section 6). "What reaches
  you" at the head's right, to How work runs.
- **Projects,** on All projects: a row each, its name, its Now in a line,
  rolled from its owners', and its health in chips: main, tiers, a
  follow's parity, a red with "owned" while an owner holds it.
- **What follows what:** each follow and each dependency between the
  person's projects, a box each with who acts and the lag and parity in
  words (`follows.md`, section 5).
- **An Ask temper line** closes the page: a box whose words open Ask
  temper with them, on the project in view.
- **Its mockups:** `home.html`, and `phone-home.html` on a phone.

## 4. Needs you

- **Across projects by default,** filtered by project from a project's
  pages, at its addresses as today (`../client/pages/README.md`, *Needs
  you*). On a wide page it is a list, oldest first, the first card open;
  on a phone, one card at a time, as today.
- **The intro** says what reaches the person, from their projects'
  settings: "What reaches you: changes to vsr's protocol core, money past
  a budget, anything that can't be undone, and departures from your
  rules. Agents decide the rest and list it here for a day."
  (`what-reaches-you.md`, section 2), with a link to How work runs.
- **A need's card:**
  - *the from line:* the project's chip, the goal or the role, the task,
    who asks and how long ago;
  - *the question,* then its context in a few lines and what each choice
    does;
  - *why it reached you,* on the signal ground: the rule that sent it, in
    words, with "Keep asking" and "Stop asking", which offers a narrower
    rule: "Stop asking: let goals decide fixes that keep the wire format".
    "Stop asking" opens the proposal of that rule, accepted in place by an
    owner (`what-reaches-you.md`, section 5);
  - *the choices,* the recommended one first; then "Or tell it something
    else", a field whose words go to the asker as its next message.
- **Its kinds:** a question, money past a budget, a sign-off a rule by
  path asks, a proposal of settings, a held task: today's cards, each
  with its why.
- **An approval or a sign-off is the person's review on the forge:** its
  card leads with "Review on GitHub ↗" or "Sign off on Forgejo ↗", one tap
  to the forge's review page, as `forge-identity.md`, section 8, has it,
  and the mockups draw it. Approving in place, temper posting the review
  as the person, is later (section 16).
- **Only you:** what no one else can do, a sign-in or a login its
  account reads as expiring, with its act alone (`what-reaches-you.md`,
  section 8). Work goes on around it.
- **Decided for you:** the rows Home has, for a day; then they live in
  each task's history.
- **Tell temper** at its foot (section 9).
- **Being told:** what waits on the person reaches them as
  `what-reaches-you.md`, section 9, says, and opens here.
- **Its mockups:** `needs.html`, and `phone-need.html` on a phone.

## 5. Work

- **One list of a project's work,** under way and lately done: goals,
  each with its state, its report's headline written by its agent, its
  tasks done of its total and its spend of its budget, as the greenfield
  mock's Goals rows have it; changes outside a goal, on their way to
  landing or landed, as Changes' rows; chats that started work, each with
  what its delegates did; drafts being written in a design session, and
  landed drafts ready to start a goal.
- **Its head** keeps Changes' landing line: "3 queued · last landed 6 min
  ago · main green".
- **Its filters:** All, Drafts, Goals, Changes, Chats. `Goals { project,
  filter }` and `Changes { project }` are its filters' addresses
  (`../client/pages/README.md`, *Goals* and *Changes*).
- **What it holds is what exists,** never the way's name: each chat that
  started work, change, goal and draft shows by its own facts, under what
  it belongs to, a goal's changes under the goal and a chat's under the
  chat, the latest activity first. Under You approve the list fills with
  chats and their changes because that is what the way makes; under
  Draft first, with drafts and goals. A filter with nothing under it
  isn't drawn (section 12).
- **A row opens its page:** a goal's and a task's pages stay as they are
  (`../client/pages/goal.md`, 2, and `../client/pages/task.md`, 2), and a goal's conversation opens
  from its page.
- **Not drawn:** Work is the greenfield mock's `goals.html` renamed, its
  rows widened to changes, chats and drafts.

## 6. Team

- **A project's areas and their owners.** The intro: "Who owns what on
  PROJECT", and a lede saying that the project's triager hands a finding or
  a red to its area's owner, who asks the person as How work runs says.
- **The areas,** a table: the area's name; what it covers, in words; its
  owner, the person, a member or a standing role, as a chip in who-acts
  colours; the owner's Now in a line; this week's spend of the owner's
  budget. What fits no area has an owner too, the project's triager
  (`../domain/findings.md`, section 6). "+ Hand an area to a role" at the
  head's right proposes a role, from the stock or written with Ask temper.
- **Who routes,** under the table: the project's triager, as a chip, which
  reads what comes in against the areas' words (`standing-roles.md`,
  section 4), changed by proposal.
- **A person's line** is what waits on them: "Two questions wait on
  you." A person writes no Now.
- **A role's card,** opened from its row (`standing-roles.md`, section 2):
  - *its head:* its name; "A standing role · owns AREA since DATE ·
    started by WHO"; "Talk to it", which gives the terminal's command
    and opens the role's conversation on the web; "Pause", no wakes
    until resumed, what comes meanwhile kept; "Give the area back", the
    area's owner becoming the person, by proposal;
  - *its Now,* "written at 06:58, when its last session ended", in its
    own words;
  - *its rules,* its charter's words, with "change by proposal";
  - *what wakes it:* each subscription and timer in words, with the
    class code gives it, "at once", "every 4 hours", "Mondays 06:00";
  - *its sessions this week:* each one's start, length, what it did and
    what it cost; then a line saying that each session starts fresh from
    its rules, its Now and what woke it, and ends by writing its Now;
  - *its weekly report,* linked (`standing-roles.md`, section 7).
- **Shown** once the project has a role or a second member (section 12).
- **Its mockup:** `team.html`.

## 7. How work runs

- **A project's process on one page.** It takes Settings' How work runs
  tab's place (`../client/pages/settings.md`, 2) and carries what that page's
  fuller form deferred: roles, ways to start work, rules, running work
  and history (`../client/README.md`, section 5).
- **The intro:** a headline naming the way and its chief departure,
  "Draft first, with the protocol core asking you"; a lede; "Ask temper
  to change this", opening Ask temper on the project; "History · N
  changes", the settings' revisions.
- **The path of a change,** the one drawing Ask temper's ways use
  (`../client/pages/chat/conversation.md`, 3): a column a step, the stops of the
  project's derived path (`stock-ways.md`, section 4), a person's and an
  agent's above the authority line and code's below it, each by its mark,
  each opening its card (`stock-ways.md`, section 5); the line "no merge
  tool above", as fabrig.ai draws it. The ways stay its tabs, over the
  drawing, with "Use this" and "Keep it" as `stock-ways.md`, section 5,
  has them, and "Yours", current, for a project that runs no stock way, as
  a project with rules by path does. What each stop does in full is its
  card's. A stop of a rule by path says so in its words, as "you, if
  core" does (`what-reaches-you.md`, section 7).
- **What reaches you:** three columns, "Agents decide, then tell you",
  "Asks you" and "Always waits for you", each item in words, with what
  Away changes (`what-reaches-you.md`, sections 2 and 3).
- **Rules by path:** a row a rule, its paths, who reviews where the
  rule asks it, who signs off, and when it lands (`what-reaches-you.md`).
- **What follows a landing:** each follow the project starts
  (`follows.md`), and its tiers on main, each with its who-acts chips:
  "code hears the landing", "an agent ports", "code checks".
- **Roles,** every agent the project's work runs, whether or not the
  path draws it: each charter its ways, templates, follows and standing
  roles start, with the chat's and Ask temper's, but not the deployment's
  model trial. A row each: its name, what it does and when, and the model
  a run of it would take now; opened, what it may do, what it is asked
  and its instructions (`../client/pages/settings.md`, 2, *Roles*). The path
  draws a typical run, and this is where the rest is, the foreman and the
  status agent among them. What it does is words saved with its charter
  (`whole-story.execution.md`, P3e).
- **Budgets,** per week: each role's, and what a goal is given when it
  starts.
- **At its foot:** the project's repositories, today's Repositories tab,
  with "Adopt a repository"; and what the project shows, today's
  switches (`../client/pages/settings.md`, 2), as overrides of what its way shows (`surfaces.md`, section 12).
- **Every change is a proposal an owner accepts,** or an owner's own
  change of one item, as `../client/pages/settings.md`, 2, has it.
- **Its mockup:** `how.html`.

## 8. Deployment

- **The deployment's own, from the rail's foot:** its tabs Accounts,
  Forges, Models, Workers, Members and Limits.
- **Today's pages in a new frame:** Accounts, Forges, Models and Builders,
  renamed Workers, keep their addresses and acts (`../client/pages/README.md`
  , *Settings*), drawn in Deployment's frame rather than a project's
  Settings, so that To begin's links and a connect's way back hold.
- **Members:** who may do what, in the deployment and in each project
  (jig's `domain/people/README.md`). Ownership of an area is Team's.
- **Limits:** the deployment's limits, read-only but for an owner
  (`../protocol/configuration/limits.md`).
- **Not drawn:** its pages are today's.

## 9. Ask temper and Tell temper

- **Ask temper answers** (`ask-temper.md`): its entry in the rail, the
  phone's Ask tab, and every page's Ask line, which opens it with the
  page in view as its context. It changes nothing but settings, by
  proposal.
- **Tell temper files.** A person's words about what's off become a
  finding: its project and area found from the page they were on and the
  words, its owner the area's (`standing-roles.md`, section 2;
  `../domain/findings.md`). Before it is filed the card says "About" and
  "Goes to"; "File it" files it; "Ask temper instead" is the other
  choice, for a question. The owner answers on the finding, and what
  becomes of it returns in the brief.
- **Where:** Needs you's foot, the phone's Ask tab, and `/tell` in the
  terminal (11.4).
- **Its mockups:** `needs.html`, its foot, and `phone-tell.html`.

## 10. The phone

- **Four tabs:** Home, Needs (with its count), Work, Ask. The top bar
  holds the switcher, All projects first, and the Away chip.
- **One tap:** a need's recommended choice is a full-width button on
  Home's card and on the need's page; "More" opens the need. An approval's
  or a sign-off's is "Review on GitHub ↗" or "Sign off on Forgejo ↗",
  opening the forge's review page (section 4).
- **A need, one at a time:** "1 of 2"; the question; its context in a few
  lines; why it reached you, with "Stop asking"; the choices stacked,
  the recommended first; a field for words.
- **Decided for you** is one row on Home, opening its list.
- **Tell temper** is the Ask tab's second act.
- **Team and How work runs** are reached from Home's project rows and
  the switcher's menu; Deployment from the person's menu.
- **Touch:** every target at least 44 pixels tall.
- **Its mockups:** `phone-home.html`, `phone-need.html` and
  `phone-tell.html`.

## 11. The terminal

This section revises `tui.md`, section 3, and the launchpad of
`../client/tui/pages/README.md`. The terminal stays where a person works with
agents, one thing in focus (`tui.md`, section 2); watching running work
stays the web's.

### 11.1 The launchpad

- **The brief,** framed: the status agent's, across the person's
  projects, with its byline and `/brief`.
- **Where you left off:** the conversations the person can go back to,
  newest first, each its project, what it is (a design session, a talk
  with a role, a chat), when, and what is open in it, numbered.
- **Needs you:** a line each, marked "?" for a question, "$" for money,
  "✓" for a sign-off; `n` walks them, one prompt after another, as
  `tui.md`, section 3, has needs from elsewhere.
- **Team on PROJECT:** each role, its area and its Now in a line.
- **Start:** `/draft`, a design session that ends in a draft and a goal;
  `/change`, one change, run as the project's way says; `/talk`, a role,
  about its area; `/ask`, Ask temper.
- **The box:** "Draft, ask, or talk to a role. / for commands". The quiet
  line: the project, the model and effort, the link's state, and "N need
  you · n to walk".
- **Its mockup:** `launchpad.html`.

### 11.2 A design session and the drafter

- **`/draft` starts a design session** with the drafter: a chat charter
  of its own, whose instructions temper/ui writes as a chat's. It reads
  the project's code through the forge, surveys with sub-agents where its
  model has them, and writes the draft as the chat's document.
- **The draft beside it:** the side pane, folded, half the screen or all
  of it, Ctrl-P cycling them (`tui.md`, section 3); what the last turn
  changed marked "new" at its right. No diffs.
- **One question at a time,** as a prompt with numbered choices, the
  recommended first with its reason, "Tell it something else" last.
- **Sittings:** a session is kept, and one opened again resumes from its
  draft and its conversation, listed on the launchpad.
- **`/goal` starts a goal from the draft,** its choices the project's
  ways of working, kind 7's tracked templates, as the web's "Start goal…"
  offers them (`../client/pages/README.md`, *Ways*, and 5.14).
- **On the web,** the same session reads as a chat with its draft in
  Split (`../client/pages/README.md`).
- **Its mockup:** `design-session.html`.

### 11.3 Talking with a role

- **`/talk NAME`** opens a session of the role with the person present
  (`standing-roles.md`, section 6). Its head: the role, its area and this
  week's spend; then its Now, dim.
- **It answers from its Now and its area,** and acts within its rules.
- **A standing instruction becomes a rule,** not a line of its Now: "From
  now on…" is answered with "Add it to NAME's rules?", numbered: "Yes, add
  it", a proposal an owner accepts in place; "Just for this port";
  "Tell it something else".
- **Leaving,** by Esc or `/exit`, ends the session, and the role writes
  its Now. The quiet line says so, with the session's length and cost.
- **Its mockup:** `role.html`.

### 11.4 Commands

- **Today's commands stay** (`../client/pages/README.md`, *A chat's
  terminal*, and 6.14), `/chats` among them.
- **Added:** `/draft`, a design session, or in one, the draft shown or
  hidden as today; `/change`; `/talk NAME`; `/ask`; `/brief`, the brief
  whole; `/needs`, walking what needs the person, as `n`; `/tell`, Tell
  temper; `/open`, the object in focus on the web (`tui.md`, section 7).

### 11.5 Views in the terminal

- **Printed, not drawn** (the user, 2026-11-08): a view in a reply or a
  chat's document (`../client/pages/chat/conversation.md`, 3) prints in the
  conversation as lines of the terminal's own text. No picture for
  v0.1.0.
- **A path:** a stop a line, in the derivation's order: the mark the web
  draws for its actor, a ring for a person, a four-point star for an
  agent, a small square for code, then its words. Code's stops are
  indented, as they stand below the line on the web. A branch's bar
  closes its part, with the branch's name.
- **The other views:** a way prints its name and line over its path; a
  proposal its two paths, under "Before" and "After"; a charter its
  `who`, `name` and `facts`, as terms; a `not_drawn` its words.
- **Nothing to press:** a stop's card stays the web's.

## 12. Density

What the pages show follows what a project's settings say and what
exists, part by part. No page is drawn per step of the spectrum
(`whole-story.md`, section 3), so a project between two steps, or mixing
them, shows exactly its parts, and moving along it never flips a page.

- **Each part has one rule,** over the project's settings, its facts or
  the person's, decided in code by one pure function per page, which the
  web and the terminal share:

| Part | Shows when |
|---|---|
| Home's brief | the status agent has written one (`../domain/forge/attention.md`, 3) |
| All projects, and Home's projects | the person is in two projects or more |
| Needs you's approvals and sign-offs | a change waits for the person's review under a landing rule |
| Needs you's questions, money, Only you | such an entry waits for the person |
| Decided for you | a row within its day exists (`what-reaches-you.md`, section 6) |
| Away | the project's classes hold an "asks you" item Away may decide (`what-reaches-you.md`, section 3) |
| Work's chats, changes, goals and drafts | one of that kind exists, live or lately done |
| Team, in the rail | the project has a second member or a standing role |
| Team's role cards; Home's owners' lines | a standing role exists |
| What follows what; lag and parity | a follow exists (`follows.md`, section 5) |
| How work runs' rules by path, follows, budgets | one is set |
| How work runs, Work, Ask temper | always |

- **The steps are illustrations,** not views; under the stock ways they
  come out so:

| Step | The rail | Home | Needs you | Work |
|---|---|---|---|---|
| Pair | Home, Needs you, Work, Ask temper, How work runs | the brief, what landed, what needs you | approvals, each a review on the forge, and questions | chats and their changes |
| Team | Team, for people; Away | who can still sign; Decided for you | sign-offs, each with who can still sign | changes |
| Stock | Away, without Team | the brief; Decided for you | questions alone | changes |
| At scale | as Stock | as Stock | questions and money | drafts and goals first |
| Standing roles | Team, for standing roles beside people | the brief from owners' Nows; What follows what | what passes what reaches you | goals, and follows' changes |

- **Mixed is ordinary:** a You approve project with one standing role
  owning its tests shows Team for that role and approvals for the rest;
  a Draft first project whose `main` still needs the person's approval
  shows drafts, goals and approvals together.
- **A move along the spectrum** is a change of settings, proposed and
  accepted (`configuring.md`, section 4). New work follows it, running
  work keeps what it started under, and each item shows by its own
  facts, so an approval asked before the move stays in Needs you beside
  the questions asked after it, until it is done.
- **How work runs suggests the next step** when the facts earn it, such
  as "let agents land test-only changes" after a run of approvals given
  unchanged: a fixed set of suggestions, each a rule over counts code
  keeps, offered as a proposal the person may make, changing nothing on
  its own.
- **This replaces** today's `Domain::away_shown` (`../client/pages/README.md`
  , *Away*) and the greenfield mock's per-project `parts` switches.
- **Overrides:** today's "What the project shows" switches
  (`../client/pages/settings.md`, 2) stay as overrides at How work runs'
  foot, hiding a part its rule would show, never showing one it would
  hide.

## 13. The mockups and their precedence

- **The files,** the whole story's mockups, which join the design in
  `whole-story.execution.md`'s first milestone, M0 (`whole-story.md`,
  section 9):
  - at scale, with standing roles, in
    `temper/docs/design/client/web/whole-story/`: `index.html`, the
    shape; `home.html`; `needs.html`; `work.html`; `team.html`;
    `how.html`; `phone-home.html`; `phone-need.html`; `phone-tell.html`;
    `whole-story.css`; `whole-story.js`; and their README; and in
    `temper/docs/design/client/tui/whole-story/`: `launchpad.html`,
    `design-session.html`, `role.html` and their README;
  - Pair, one person approving every change, in `pair/` beside each:
    `first-run.html`, `home.html`, `needs.html`, `work.html`, `how.html`
    and `phone-home.html` on the web, `launchpad.html` in the terminal;
  - Team, three people with sign-offs by path, in `team/` beside each:
    `home.html`, `needs.html`, `work.html`, `team.html`, `how.html` and
    `phone-home.html` on the web, `launchpad.html` in the terminal.
- **They win where they differ** from the greenfield mock: the rail,
  Home across projects, Needs you's list and its why, Work, Team, How
  work runs, Deployment, the phone's tabs and the terminal's screens.
  The greenfield mock still leads where these draw nothing, a goal's page
  and a chat, and leads `../client/mockups/` as it does today
  (`../client/web/mockups/README.md`, section 1).
- **Their data is illustrative:** at scale, one Sunday morning on one
  deployment with four projects, vsr, vsr-rs, cowsql and fabrig; Pair,
  one maintainer's project, pocket; Team, three people's project, shop,
  on a deployment of their own. Their words,
  parts, order and look bind; their figures don't.
- **Their look is the greenfield mock's,** paper and ink with one signal
  colour, and night as the client has it (`../client/view.md`, 3). Who
  acts is a mark, those of the client's path (`../client/pages/chat/conversation.md`,
  3), on its stops and on the chips: a person an orange ring, an agent
  an ink star, code a grey square. Orange is the person's alone, so
  How work runs' milestone, P3, turns the client's agent star, and the
  line's rise to it, from the signal colour to ink, in Ask temper's ways
  too. The terminal's frames are the dark theme's, 110 columns wide.
- **Refined after they're built,** on their journeys, by look requests
  (`look-requests.md`).

## 14. Its acceptance

Focused and fuzzy tests only; a walk and a render check follow landing.

- **The web's gallery states** (`../client/web/gallery.md`): Home
  on All projects, with needs and decisions, with nothing new, with a red
  owned; a project's Home at each step of `surfaces.md`, section 12; Needs you, a need
  with its why and its offer, Only you, Decided for you, Tell temper's
  card; Work at Pair and with drafts first; Team with areas, a role's
  card, and none; How work runs under each stock way, with a rule by
  path and a follow; Deployment's tabs; the phone's tabs and a need at
  390 pixels wide.
- **Step tests of the acts:** answering with the recommended choice;
  "Stop asking" and its narrower rule's proposal, and "Keep asking";
  an approval's card opening the forge's review page; Overturn; filing a
  finding; handing an area to a role; pausing a role; the rail and tab
  bar under each step of section 12.
- **The terminal's golden frames** (`../client/tui/journeys.md`): the
  launchpad, a design session with the pane folded, half and full, and
  talking with a role up to the rule's proposal, at 80 by 24 and wider,
  in both themes.
- **Journeys,** recorded as `../client/web/journeys.md` and
  `../client/tui/journeys.md` say: Home to a need, answered; Team to a
  role, then Talk; a design session to `/goal`; a talk with a role that
  adds a rule. The user takes their pages and frames.
- **A fuzz test of section 12's rules,** over random settings revisions
  and facts, on each page's model: a part shows exactly when its rule
  holds; no part is drawn empty; every live item is placed exactly once;
  and a move of the settings never hides a live item, such as an
  approval asked under You approve, still shown once the project has
  moved to Agents review.

## 15. Open questions

- **`/chats` or `/resume N`** for Where you left off. Default: `/chats`,
  the user's request of 2026-11-08 (`tui.status.md`), its list numbered
  as the launchpad numbers them, as the mockups draw it.
- **Needs you on a wide page:** a list with the first card open, or
  today's one card at a time. Default: the list, one at a time on a
  phone.
- **Repositories:** at How work runs' foot, or in Deployment. Default:
  How work runs, since a project's repositories are its own.
- **`Chats`:** kept at its address and reached from Work, or folded
  away. Default: kept.
- **The drafter on the web:** the same session read in a chat's Split,
  or the terminal's alone. Default: both, as a chat is.
- **One brief across projects,** written per person: which project's
  period funds it, which project its task belongs to, whether it may read
  other projects' Nows. Default: section 3's one brief a project, joined
  on All projects.

## 16. Later

- **Talking a need through in the terminal:** a need's card naming a
  terminal command that opens a conversation with whoever asked. No such
  command is designed: 11.4's `/talk` is for talking with a role.
- **Approving in place:** an approval or a sign-off given from Needs you and posted on
  the forge as the person, with a forge credential they signed in with,
  rather than one tap to the forge's own review page (temper's
  `../README.md`, section 8). The forge's page is v0.1.0's (the user,
  2026-11-08).
