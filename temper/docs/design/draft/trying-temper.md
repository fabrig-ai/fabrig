# Trying temper: the first ten minutes

Draft, 2026-11-18, with the user. The first journey of the UX draft
(J0, "the try"), step by step: from a person typing `./temper box`
after the download to their first change landed on their own machine,
in the terminal and the browser at once. Rewritten the same day: the
try is the set up, starting from Forgejo on this machine, with the
welcome choosing by what the person wants, in the words section 4
proposes, and fewer choices (the user, 2026-11-18). Then the user added
two things: the playground stays, to become a showcase of every way of
working, offline (section 7); and a rung below Pair, a person working
with one agent in its own copy of the repository, as with a coding
agent, without a chat that delegates to another (4.8). The try's first
change is worked that way. GitHub's way is the second path
(section 6). On paper first; the mockups follow (section 8). On
2026-11-19 the user took the paper as sound, settled that Pair names the
new rung, rung 0, and asked how the try fits the bigger picture
(section 10); the mockups' first pass is `trying-temper/index.html`.
Overnight, on the user's word for the night, GitHub's path was
designed (section 6).

## 1. In one page

- **What the try is for:** a person who has just downloaded temper sees,
  on their own machine and with their own model, a change made with an
  agent, checked by CI, reviewed by another agent, and merged at the
  commit that was checked, with no merge an agent's. Then they hand the
  next one over, or say how they work.
- **One press to start:** the welcome asks what the person wants, "Try
  temper here" or "Work on your GitHub repositories", each with what it
  costs: the time, what leaves the machine, what gets access. Trying
  here starts a Forgejo on this machine with a small sample project, at
  once, with no sheet and no form (4.1). The playground stays, behind a
  quiet link (section 7).
- **Three steps, not four:** Start, Model, Ready. Ready makes the
  project by itself once the forge and the model are; the person names
  nothing (4.2, 4.3).
- **A first change worth making:** the sample is a small program with a
  test its CI runs, so the check is real; Home offers three first
  changes for it (4.4, 4.5).
- **The first change is worked with an agent:** one agent, in its own
  copy of the sample, edits and runs the test in front of the person,
  who presses "Land this" when it looks right. From there temper's line
  runs: CI, another agent's review, the merge. This is Pair, rung 0
  (4.8), a design of its own.
- **The card says the line:** a change's card reads "Written with you",
  "Checked by CI", "Reviewed by an agent", "Landed", each with its fact,
  the same commit in the last three (4.6). temper says it, not the
  model.
- **The climb is offered next:** once the first change has landed, the
  next is handed over and lands by itself while the person watches.
- **Claude Code said first:** Claude's card says whether Claude Code is
  on the machine before it is pressed (4.7).
- **About six minutes** to the first change landed, none of it measured
  (section 3).
- **GitHub's path** (section 6): the same welcome and model; Ready
  starts with the repository installed, on a press; a card says whether
  a first change can land before one is made; the first change is the
  person's own, from their words or an issue; their review is on
  GitHub, which temper asks for, so that it reaches their phone.

## 2. How it feels

A developer on their laptop has downloaded `temper` from the site,
which said "try it in ten minutes". They are curious and a little
wary: another tool that wants their accounts?

**0:00, the terminal.** `./temper box`. The terminal turns into temper
at once: its mark, "temper box · a whole deployment on this machine",
and "Welcome. Set temper up in your browser: open this link on this
machine." Under it, for a few seconds, "Downloading temper-box, once
for this build: 12.3 of 24.1 MB". Nothing asks anything yet. Then the
link appears where it said it would.

**0:15, the link.** A click, or a copy. The browser opens on the
welcome; the terminal ticks "✓ Signed in as the owner" by itself. The
two are one thing, and the terminal is following.

**0:20, the choice.** Two cards, and the first answers the wary
question before it is asked:

```
Try temper here                          Work on your GitHub repositories
A forge on this machine, with a          temper's GitHub App, on the
small sample project to change.          repositories you choose.

Nothing leaves this machine.             Needs your GitHub account.
Nothing gets access to your accounts.    temper's changes land in them.
About five minutes.                      About ten minutes.

[ Start ]                                [ Connect GitHub ]
Or start from a copy of a public repository ›

Or look around a scripted playground first, with nothing to set up ›
```

They wanted to see it on their own code, but not to hand an unknown
tool their GitHub. "Nothing gets access" settles it, and the copy line
says their own public code can come along. They press Start. Nothing
else is asked.

**0:25, the model.** "Connect a model: temper's agents run on a plan you
sign in to." Two cards, Claude's first: "Signs in from the terminal
where temper runs". Beside them, a pane already moving: "Forgejo ·
work/hello · Fetching Forgejo, 18%". temper is working while they
choose; that is the first good feeling.

**0:40, the sign-in.** "Sign in to Claude Code". The browser says
"Claude Code is signing in, in the terminal where temper runs". The
terminal has cleared: "temper: Claude Code signs in here. Approve it on
Anthropic's page, in the tab it opens…", then Claude Code's own screen,
and a tab opens on Anthropic's page. Three windows at once: this is the
try's most confusing minute, and every one of them says where to look
next. Approve; the terminal comes back with "Claude Code signed in"; the
browser reads "✓ Claude subscription, through Claude Code, signed in",
then "answered in 3 s": temper has tried it, and it works.

**2:00, ready.** "Continue". If Forgejo is still starting, "Almost
ready: Forgejo is setting up, 80%. Your project opens by itself." It
does, a few seconds later. No name to choose: the project is Hello,
after its repository, renamed whenever they like.

**2:30, Home.** "Hello is ready. Your agents run on your Claude
subscription, and Forgejo on this machine." Then "To begin":

```
1  Make a first change with an agent
   It works in its own copy of work/hello, in front of you;
   when you say so, CI checks it, another agent reviews it,
   and temper merges it.
   [ Add a --name option ]  [ Greet in Italian with --lang it ]
   [ Add a --shout option ]
2  Hand the next one over                    after your first change
3  Tell temper how you work                              Ask temper ›

Your own code: copy a public repository, or connect GitHub ›
```

The repository's link shows the sample on a real Forgejo: a short
`hello` script, its test, and a CI file that runs it. Small enough to
read in ten seconds, so the change to come can be judged.

**3:00, the first change.** "Add a --name option". A chat opens with an
agent in its own copy of work/hello, and they watch it work, as with
any coding agent:

```
▸ read hello
▸ edit hello                      +6 −1
▸ edit test.sh                    +4
▸ sh test.sh                      passed
✦ hello --name Ada now prints "Hello, Ada!", and the test covers it.

  This chat's changes · 2 files · +10 −1 · Show the diff
  [ Land this ]
```

The diff is one click away, in the chat. They ask for one more thing,
"keep 'world' as the default", and the agent edits again. Nothing has
reached the forge yet: the copy is the agent's, and so far it is just
the two of them.

**3:40, "Land this".** The change's card takes the place of the button:

```
T2 · CHANGE                                                Running
Add a --name option to hello
  ✓ Written with you           this chat · 2 files · View on Forgejo ↗
  ▸ Checked by CI              running at 3f2a1c9
  · Reviewed by an agent
  · Landed
```

The agent they worked with cannot merge, and neither can the one that
reviews it: temper merges, if the line holds. In the terminal, Hello's
launchpad says the change is in its way.

**3:40 to 5:30, the wait.** CI on this machine's runner, then the review's
words. If CI fails, it comes back into the same chat, "CI failed at
3f2a1c9: test.sh, line 7", and the agent fixes it with them; "Land
this" again. The try shows the line holding, not breaking.

**5:30, landed.**

```
T2 · CHANGE                                                 Landed
Add a --name option to hello
  ✓ Written with you           this chat · 2 files
  ✓ Checked by CI              passed at 3f2a1c9
  ✓ Reviewed by an agent       task 3 approved 3f2a1c9: "Adds --name, with a test."
  ✓ Landed                     merged 3f2a1c9 into main, by temper
```

The same commit three times: what was checked is what was reviewed is
what was merged, and the last line is temper's, not an agent's. On
Forgejo, `main` holds the merge. That is the line, seen once, on their
own machine.

**5:45, the climb, offered.** To begin's second step opens: "Hand the
next one over: an agent writes it, checks it and lands it while you do
something else." with "Greet in Italian with --lang it". Pressed, it
runs on its own: no chat to steer, the card ticking from "Written by an
agent" to "Landed" by itself. The person sees the rung above the one
they started on, without reading about it. "Tell temper how you work"
follows: the way both changes ran, Agents review, is one they can
change.

**The terminal, all along.** Everything after the sign-in can be done
there too, and a coding agent's user may prefer it: the launchpad offers
the same first changes; the agent's edits show as diffs in the
conversation, `/diff` shows the chat's whole change, and `/land` lands
it. Ctrl+C asks "Stop the box?" while anything runs, and `./temper box`
tomorrow opens on Hello, signed in, with nothing to click.

## 3. The steps

Each with what the screens say beyond section 2, what temper does
behind it, and how far it is built. "New" marks what section 4 adds.

| Step | Browser | Terminal | Behind | Built |
|---|---|---|---|---|
| 1. `./temper box` | nothing | the welcome, "Downloading temper-box…", then "Starting the box…" | checks git 2.41, `/usr/bin/rg`, `/bin/sh` (`../release.md`, 4); fetches `temper-box` once, checked (`../client/box-binary.md`); the box binds its port, draws the link, starts serve (`box-link-first.md`) | yes, recorded |
| 2. The link | signed in as the owner | "✓ Signed in as the owner" | jig spends the link: one live, two hours | yes, recorded |
| 3. The choice | New: the two cards of section 2 | "▸ 1 Start" | none until a press | the welcome is built, its words and cards New |
| 4. Start | Model, the pane's Forgejo line | "▸ 2 Model", "✓ 1 Start · fetching Forgejo…" | New: one `SetUp`, Forgejo on this machine, its first repository the sample (4.4) | the set up is built, recorded to Model; the sample New |
| 5. Model | Claude's and ChatGPT's cards; New: Claude Code found or not (4.7) | lent to Claude Code, then "✓ 2 Model · Claude subscription, through Claude Code" | Claude Code in an environment of its own; the try, one short turn (`../client/pages/welcome.md`, 1, *The try*) | yes, not recorded past Model's first frame |
| 6. Ready | "Almost ready", the forge's bar, then Hello's Home | "▸ 3 Ready", then Hello's launchpad | New: the project made by itself, named after the repository, which is adopted into it (4.3) | Ready's form is built; New without it |
| 7. Home | "Hello is ready.", To begin, New: its steps (4.5) | Hello's launchpad, "● Live · Hello · Claude Opus 5.5 · high" | the adoption runs on | Home and To begin are built; the steps New |
| 8. The first change | the chat with its agent's edits, its diff, "Land this" | the same chat, its edits as diffs, `/land` | New: a chat whose agent works in its own copy of the repository (4.8) | New |
| 9. Land this | the card New (4.6), "View on Forgejo ↗" | the card's line | the worker pushes the chat's branch and temper opens a change from it, as `temper ctl change open` does for a pushed branch (`../domain/forge/change/opened.md`); CI on the box's runner, gate 1 an agent's review, the merge at that head (`../protocol/configuration/stock.md`, kind 2) | the change's way and the opening from a branch are built; from a chat New |
| 10. Landed | the card's four facts | "Landed" | the chat hears the change's end | yes, but for the card's words |
| 11. The next, handed over | a chat that delegates, its card | the launchpad, working | the chat's agent delegates a change, as a change chat does today | yes |

## 4. The changes it needs

Each small unless it says otherwise, and each the client's unless it
names another part.

### 4.1 The welcome chooses by what the person wants

- **Two cards,** replacing "Set up temper" and the Forge step's choice:
  "Try temper here", the default, its button "Start" primary; "Work on
  your GitHub repositories", its button "Connect GitHub". Each says what
  it costs in three short facts: the time, what leaves the machine,
  what gets access (section 2's words).
- **Start sends the set up at once:** Forgejo on this machine, its first
  repository the sample, `work/hello`; no sheet. The sheet's other two
  ways stay, behind quiet links under the card: "Or start from a copy
  of a public repository ›", its addresses' field in place, and "or
  with no repository ›".
- **The Forge step leaves the Forgejo path;** "Connect GitHub" opens
  GitHub's way (section 6). A forge cut short still offers "Start over".
- **The playground,** a quiet link under the cards: "Or look around a
  scripted playground first, with nothing to set up ›" (section 7).

### 4.2 Three steps: Start, Model, Ready

The top bar's steps, and the terminal's "In the browser  1 Start · 2
Model · 3 Ready". Start is done once a forge is kept, as Forge is now.

### 4.3 Ready makes the project by itself

Once a real forge is connected and a model usable, Ready makes the
person's first project, named after its first repository, "Hello", and
adopts the repository into it, as its form's act does today
(`first-run-projects.md`, 4), then opens its Home. No field and no
press: the name is renamed from the switcher. Until then Ready says
what it waits for, "Almost ready: Forgejo is setting up, 80%. Your
project opens by itself.", or "Almost ready: connect a model." A name
already taken opens that project, as today.

### 4.4 A sample worth changing

The empty repository becomes the sample, `work/hello`: a short POSIX
`sh` script, `hello`, that prints a greeting; its test, `test.sh`; and
the starter workflow running `sh test.sh`, which needs nothing the
machine lacks, since temper already needs `/bin/sh`. A change that
breaks the greeting fails CI, so the check is real. It crosses into the
set up's first repository (`../protocol/setup/owned.md`, 7) and skein's
`Create`, whose files are bounded at two of 4 KiB; the sample needs
four, Forgejo's README with them. Medium.

### 4.5 Home's To begin, for the try

While the project's repository is the sample, To begin's steps are:

1. "Make a first change with an agent", "It works in its own copy of
   work/hello, in front of you; when you say so, CI checks it, another
   agent reviews it, and temper merges it.", and three requests fit to
   the sample, each opening a chat with an agent in its own copy (4.8)
   with its words: "Add a --name option", "Greet in Italian with --lang
   it", "Add a --shout option".
2. "Hand the next one over", "An agent writes it, checks it and lands it
   while you do something else.", open once a change has landed, its
   requests the two the person did not take, each starting a chat that
   delegates the change, as a change chat does today.
3. "Tell temper how you work", to Ask temper, as today.

Under them, "Your own code: copy a public repository, or connect
GitHub ›". To begin then shows three steps, where it shows two at most
today (`../client/pages/home.md`, 1).

### 4.6 The card says the line

A change's card in a chat (`../client/view.md`, 5) gets four steps in
place of "Branch pushed", "Pull request opened", "CI passed" and
"Landed": "Written with you" for a change landed from the chat's own
copy, "Written by an agent" for one it delegated; "Checked by CI";
"Reviewed by an agent"; and "Landed". Each has its fact once done: the
chat or the task that wrote it and its files, with "View on Forgejo ↗"
once its pull request is open; CI's verdict at the head; each review
gate's verdict and its words at the head; the merge commit. The last
three name the same commit. A change with no review gate has no
"Reviewed" step; a failed check reads "failed at ABCDEFG", then who
repairs it: "back in this chat" for the chat's own, "an agent is
repairing it, 1 of 3" for a delegated one. The card reads the change's
panel, as the task's page does (`../client/pages/task.md`, 3), not the
chip alone. Medium.

### 4.7 Claude's card knows Claude Code

Claude's way runs the person's `claude`, which temper does not install
(`../release.md`, 4). The card says, before it is pressed, whether it is
on this machine: found, as today; not found, "Needs Claude Code on this
machine: install it, then come back", a link to its install page, and
ChatGPT's card the default. The engine says whether it found it, beside
the `accounts` document's `lendable`. Medium.

### 4.8 Working with an agent in its own copy

The user's word (2026-11-18): a rung below Pair, where the person works
"quite like a coding agent", with one agent that "has its own repo
checkout/workspace and works directly in it". Today a change chat runs
with no way to write (stock charter 1: the inspect and agents families,
no modify, no shell: `../protocol/configuration/stock.md`) and delegates
the change to another agent, which writes it unattended: two agents and
a hand-off, where the person wanted one. Large: a draft of its own,
which this one only sketches, so that the try can use it.

- **One agent, its own copy:** a chat on a project's repository whose
  agent may read, edit and run commands in a working copy of its own, on
  a branch of its own from the landing branch, kept across the chat's
  turns. The person's own checkout is never touched.
- **In front of the person:** its edits and commands show as it makes
  them, the browser's and the terminal's conversation alike, with the
  chat's whole change one press away ("Show the diff", `/diff`).
- **"Land this," the person's act:** the worker commits what the copy
  holds and pushes the chat's branch; temper opens a change from it, as
  it opens one from a pushed branch today
  (`../domain/forge/change/opened.md`); from there it is any change: CI,
  the gates, the queue, the merge at the checked head. The chat's agent
  holds no tool that pushes or merges.
- **Repairs come back to the chat while it is open:** a CI failure or a
  review asking for changes reaches the chat as words, and "Land this"
  again updates the change. A chat closed hands them to the change's own
  repair, as a change opened from a branch does today.
- **After landing,** the copy moves to the new head of the landing
  branch, and the chat can make the next change. One change at a time.
- **And delegation stays:** "hand it over" in the same chat starts a
  change another agent writes, unattended, as today: the rung above.
- **One chat or two kinds:** whether every chat on a repository works
  this way, delegating when asked, or it is a kind of chat of its own,
  is the draft's first question. Default: every chat on a repository,
  so that a person meets one thing, a chat, and its agent decides
  nothing about where it works.

## 5. Where it may still feel wrong

- **Forgejo's fetch on a slow line,** the try's first long wait if the
  sign-in is quick: Ready's bar and "opens by itself" let the person
  look away. Its size and time are not measured.
- **The sign-in's three windows:** the browser, Anthropic's tab and the
  terminal. Each says where to look; the mockups will show whether
  that is enough.
- **The first change's minutes:** shorter now that its writing is seen;
  the wait is CI and the review, which the card's steps keep moving.
- **A red in the first change:** shown as a repair under way, back in
  the chat, not a failure; the try's words for a change held for good
  are open.
- **The person's plan is spent:** a first change costs a few turns of
  their subscription. Home's lede says what the agents run on; whether
  the try says more is open.

## 6. GitHub, the second path

Designed on 2026-11-19, overnight, under the user's word for the night
("take reasonable decisions, so nothing blocks on unimportant
details"), from a survey of what main builds; for their read, its
mockups after it. It is the UX draft's first day on GitHub (its 5.2, on
its own branch), made short. The welcome, the Model step and the first
change worked with an agent (4.8) are the Forgejo path's. What differs
is the App, Ready, what the first change must pass, and the person's
own review.

### 6.1 How it feels

The same developer, whose code is on GitHub, at the welcome's choice,
at 0:20 in section 2. They press "Connect GitHub" on the second card.

**0:20, before leaving.** Under the button, what GitHub will show (6.3):
"On GitHub: create temper's App under your account, then install it on
the repositories it may use; one is enough to start. It pushes branches
under temper/, opens and merges pull requests, and reads your
repositories' settings and checks." And, for an organization: "On an
organization, its owner installs it."

**0:30, GitHub.** The same tab moves to GitHub's page for a new App,
filled in by temper: its name, temper-3fa9c1, and its permissions. They
create it, and GitHub's install page follows: their account, "Only
select repositories", one, pocket. Back on temper, Model, with "✓ Start
· temper's App on ada · 1 repository"; the terminal ticks the same
line. Nothing was copied or pasted: the manifest flow
(`../protocol/setup/github.md`, 2).

**1:00, the model.** As on Forgejo, from 0:25 to 2:00 in section 2.

**2:30, Ready asks one thing,** not a name but where to start (6.4):

```
Ready
Start with ada/pocket
  lands into main · merge commits · CI: ci.yml
  [ Start with pocket ]
Another repository ›
```

One press adopts it, and the project is named after it, pocket. Its
landing branch is GitHub's default branch, read, not typed.

**2:40, Home: can a first change land?** Before any model time is
spent, To begin's first step is temper's read of the repository, the UX
draft's readiness card (its J1, step 3), from what the App already
reads (6.5):

```
1  Can a first change land in ada/pocket?
   ✓ CI on pull requests     ci.yml · main passed 2 h ago, in 6 min
   ✓ Merging                 a merge commit, by temper's App
   ✓ Your review counts      main asks for 1 approval; yours counts
   · Checks before CI        none: pocket has no .temper/pre-pr
   ? The agent's tools       known once it runs them
   A first change can land; CI takes about 6 minutes here.
2  Make a first change with an agent
3  Tell temper how you work                              Ask temper ›
```

A line that isn't ready says what to do: "CI runs nothing on pull
requests, so a change would wait for checks that never come." A read
that failed is "?", never "✓".

**3:00, the first change,** worked with an agent in its own copy of
pocket (4.8), as on Forgejo, but from their words: temper doesn't know
pocket well enough to suggest changes. The step offers a box, "What
should change in pocket?", and up to three open issues as one-press
starts, those labelled "good first issue" first (6.6). They pick "#12
--json drops null fields".

**6:00, "Land this".** The App pushes the chat's branch under `temper/`
and opens a pull request, as temper-3fa9c1[bot]. On GitHub the card has
a fifth step, the person's review, since a repository with code starts
at You approve (the UX draft's section 13) (6.7):

```
T2 · CHANGE                                                Running
Fix #12: --json drops null fields
  ✓ Written with you      this chat · 2 files · pull request #48 ↗
  ▸ Checked by CI         ci.yml running at 9c0e7d2 · about 6 min
  · Reviewed by an agent
  · Approved by you       on GitHub
  · Landed
```

**12:30, their review.** "Approved by you: waiting for you · Review on
GitHub ↗". temper has asked GitHub for their review, so GitHub's own
mail and app tell them, on their phone too (6.8). They read the pull
request there and approve. temper hears of it within seconds, through
the App's delivery log, since GitHub can't reach the laptop
(`../protocol/forge/README.md`, 4.1), and merges at the head CI checked
and the agent reviewed.

**13:00, landed.** "✓ Landed · merged 9c0e7d2 into main, by temper's
App", and the pull request on GitHub says the same. The climb follows
as on Forgejo: the next one, handed over.

**About ten minutes, and their CI's time,** none of it measured. Their
CI is the longest wait and theirs; the readiness card says how long it
takes before the first change starts, so the wait is expected. The
terminal follows as on Forgejo: `/land` opens the pull request, and the
card's link is its.

### 6.2 The steps that differ

| Step | GitHub's | Behind | Built |
|---|---|---|---|
| 3. The choice | New: what GitHub will show (6.3) | none until a press | the GitHub card is built, "about two minutes" |
| 4. Start | GitHub's App page and install page, then Model | the manifest posted, its code converted at the redirect, the installation accepted at the return (`../client/forge/served.md`, 1) | yes, not recorded; an organization's request ends there today |
| 6. Ready | New: "Start with ada/pocket" (6.4) | the adoption, into GitHub's default branch | Ready's form and the picker are built; on GitHub Ready adopts nothing, and the picker's Landing branch is empty, under More, and required (`../client/forge/family.md`, 4) |
| 7. Home | New: the readiness card (6.5) | the App's reads: workflows, main's last run, protection, merge styles, `.temper/` | adoption reads protection and workflow names (`../domain/forge/adoption.md`, 4 and 7); the card New |
| 8. The first change | New: their issues as starts (6.6) | as on Forgejo (4.8), in a copy of their repository | New |
| 9. Land this | the pull request, by the App | their Actions; the head's checks and statuses, folded (`../domain/forge/change/gates.md`, 2) | yes, against a fake GitHub only |
| 10. Approved by you | New: the card's step, and GitHub asked for the review (6.7, 6.8) | the person's approval counted at the head, a request for changes repaired with its remarks (`../domain/forge/change/gates.md`, 2) | counting and repair built; the request to GitHub New |

### 6.3 The welcome says what GitHub will show

Small. Under the GitHub card's button, before the tab leaves: what the
App may do, in the person's words; that one repository is enough to
start; that on an organization its owner installs it. GitHub's pages
list the permissions too, but temper says it first, to section 2's wary
person. The card's time (4.1's "About ten minutes") adds "and your
CI's".

### 6.4 Ready starts with a repository

Small. On GitHub, Ready offers the repository to start with and adopts
it on a press: the one installed, or the most recently pushed of
several, "Another repository" opening the picker. Its landing branch is
the repository's default branch as GitHub reads it, its merge style the
first GitHub allows (`../domain/forge/adoption.md`, 4 and 8). The picker, too,
fills Landing branch from a pick's default branch, so that a pick never
meets a required field it can't see. Unlike Forgejo's Ready (4.3), a
press: adopting a person's repository is theirs to do.

### 6.5 Can a first change land

Medium: reads in the engine, a card in the client. The UX draft's
readiness card (its J1, step 3), as To begin's first step on GitHub.
Each line is ✓, · (not there, saying what that means) or ? (not read),
with what to do:

- **CI on pull requests:** the workflows that run on a pull request,
  which needs each workflow's triggers read, not only its name (an open
  question of `../domain/forge/adoption.md`, 9); main's last run, its
  verdict and how long it took.
- **Merging:** the merge styles GitHub allows, and protection that
  would refuse temper's merge (`../domain/forge/adoption.md`, 4).
- **Your review counts:** the approvals protection asks for, and
  whether temper knows the person's GitHub account, which a manifest's
  App gives: its owner.
- **Checks before CI:** whether `.temper/pre-pr` is there
  (`../domain/agent/charter.md`, 4).
- **The agent's tools:** unknown until it runs them, and said so.

A repository whose CI runs nothing on pull requests hears it here,
before a change waits for checks that never come.

### 6.6 A first change from their words or their issues

Small. On GitHub To begin's second step suggests no changes, since
temper doesn't know the repository: a box, "What should change in
NAME?", and up to three of its open issues as one-press starts, "good
first issue" first, none when it has none. The issue's number goes in
the change's title, so that GitHub links the pull request to it.

### 6.7 The card's fifth step

Small, beside 4.6. Where a rule asks for the person's approval, the card
shows "Approved by you" between the agent's review and the merge: "on
GitHub" while it waits, the head it approved once given. On Forgejo,
with You approve, it is the same step, its link Forgejo's.

### 6.8 temper asks for the review on GitHub

Small, the engine's. When a change waits for the person's approval, the
App requests their review on the pull request, from their known GitHub
account, so that GitHub's mail and app tell them, as the UX draft's J1
expects (its step 7). temper's GitHub adapter has the call; nothing in
temper makes it today. temper's own pages stay on the laptop; the phone
reaches GitHub.

### 6.9 Where it may still feel wrong

- **An organization:** a member's installation becomes a request, and
  temper can't wait for an owner's approval today. The welcome's line
  says so first; waiting for an owner is not the try's.
- **Their CI:** long, needing secrets, or red on main. The card says
  main's last run. A red that isn't the change's goes to repair today,
  and rerunning a runner's fault needs the App to write Actions, which
  it doesn't ask for (`../protocol/setup/github.md`, 1).
- **The laptop asleep:** the box stops while it sleeps; a change
  waiting on CI catches up when it wakes, and a long sleep may hold it.
- **The App after the try:** it stays on GitHub; removing the forge
  says "The GitHub App stays yours: delete it in its settings."
- **What it spends:** a few turns of their model and one run of their
  CI.
- **Not recorded:** no journey walks GitHub's path yet; its parts are
  tested against a fake GitHub only.

## 7. The playground

The user's word (2026-11-18): the playground stays, "a semi 'live' /
'real-feel' show case of all the rungs", where a person can "see how
each 'way of work' could feel like without having to setup anything and
completely offline", and "how a fully automated factory would work".
It is designed once every rung is, so its words and its scripts follow
theirs.

- **Until then,** it stays as it is, behind the welcome's quiet link
  (4.1).
- **What this draft's first version found in it,** to fix when it is
  taken up: its way out said two ways (the scripted answers and the
  model picker send the person to Accounts, where nothing can be
  connected: `../protocol/setup/playground.status.md`), its reset taking
  Demo at once with no question, and the terminal offering none of its
  requests.

## 8. The mockups to make

One frame a step, web and terminal side by side, in the greenfield
mock's look (`../client/web/greenfield/`) and the terminal's
(`../client/tui/journeys/theme.css`), with section 2's words, each
change of section 4 under a switch so that each can be seen with it and
without:

| Step | Web | Terminal |
|---|---|---|
| 1 | | the welcome, downloading |
| 2 | | the welcome, its link |
| 3 | the choice, two cards | signed in, "▸ 1 Start" |
| 4 | Model, Forgejo fetching in the pane | "▸ 2 Model", Forgejo fetching |
| 5 | Claude Code signing in; Claude Code not found | lent to Claude Code; back |
| 6 | Almost ready | "▸ 3 Ready" |
| 7 | Hello's Home, To begin | Hello's launchpad |
| 8 | the chat, the agent's edits, its diff, "Land this" | the same chat, edits as diffs |
| 9 | the card, checking | the card's line, `/land` |
| 10 | the card, landed | the chat, landed |
| 11 | To begin's second step, a delegated card | the launchpad, working |

The recorded frames stand for steps 1, 2 and the Model step's first
frame. A recording of the whole try, once its changes land, is a walk
after landing, and measures section 3's times.

**The first pass,** 2026-11-19: `trying-temper/index.html`, the eleven
steps in order, sixteen browser pages and twelve terminal frames, each
change of section 4 under its switch (`trying-temper/README.md`). What
drawing them showed:

- **Today's Home offers no first change:** while To begin shows, the
  rail keeps Home and Ask temper, and To begin's steps are the
  repository, already ticked, and "Tell temper how you work"
  (`../client/pages/home.md`, 1). A person who came to see a change made
  finds it only in the rail's "+ New change". 4.5 is the try's turn
  from setting up to working, not a nicety.
- **The terminal needs To begin too:** the launchpad offers `/change`
  and `/ask` (`../client/tui/pages/launchpad.md`, 7), and nothing fit to
  the sample. Drawn as a section of its own, "To begin", the browser's
  three steps with their requests as `/change` lines: 4.5 for both.
- **A chat in its own copy has verbs of its own:** Diff in its switch
  beside Chat and Split, and "Land this" at its top right, where a
  draft chat has "Start goal…"; in the terminal, `/diff` and `/land` in
  the box's hint, and `/land` asking first. Section 10.
- **A change waiting on its chat:** when the review asks for a change
  of a change landed from a chat, the change waits for the chat's next
  "Land this". The card says "Waiting", a state today's cards lack
  (`../client/view.md`, 5), its step "back in this chat". 4.6 and 4.8.

## 9. Open questions

- **The sample's language:** POSIX `sh`, which needs nothing more, or a
  language a developer would recognise as real, which needs its
  toolchain on the machine. Default: `sh`.
- **Ready with no press:** some people may want a moment to name the
  project. Default: no press; the switcher renames it.
- **The rung's name:** settled, the user's (2026-11-19): working with an
  agent in its own copy is Pair, rung 0, below the UX draft's rung 1.
  Rung 1, where the person approves each change a delegated agent made,
  takes another name, the user's to give. Proposed: "Review", since the
  person's part there is reviewing each change.
- **GitHub's first change:** the person's words or an issue (6.6), or
  "Teach temper this repository", the UX draft's harmless first change
  that writes only `.temper/` (its J1, step 4). Default: their words;
  teaching after v0.1.0, as the UX draft has it.
- **The first change's rung:** worked with an agent (4.8), or handed
  over as today, the try then needing nothing of 4.8. Default: worked
  with an agent, the try's first change waiting for it; until it lands,
  the try's first change is a delegated one, its card saying "Written by
  an agent".

## 10. The bigger picture

The user's word (2026-11-19): the UX draft lets a person move between
rungs freely, the verbs staying the same and only the noun changing;
"each rung needs some 'sub verbs' of its own (/land, /diff) not
applicable to others, so probably both the TUI and the web UI will need
to be a bit 'contextual'". And: "One exercise we'll have to do is to
design how each rung feels and works in a 'real' / day-2 deployment
(be it temper box, which is totally adequate and just differs in the
scale of what it can do, limited by hw, or a full fledge temper-engine
+ temper-workers deployment)." Baby steps first: the try's mockups.

- **The five verbs stay:** Start, Answer, Look, Tell and Tune, the UX
  draft's, its section 3.2, on its own branch for now, on every rung.
- **A rung's own verbs are those five at its size,** shown where its
  work is: a chat says which rung it works at, and its page offers that
  rung's forms and no other's. As drawn for the try (section 8), a chat
  in its own copy, rung 0, has "Land this", its Start, and Diff, its
  Look (`/land`, `/diff`); a draft chat has "Start goal…" and its draft
  (`/goal`, `/draft`); a chat that delegates has the card it waits on,
  and nothing to land. The places are fixed, the chat's switch, its top
  right and the box's hint; what stands in them is the rung's.
- **The day-2 exercise:** each rung designed again at day 2, on a box
  and on an engine with its workers: what the person opens in the
  morning, how a rung's work is found again, and what differs only in
  scale. After the try's mockups and GitHub's path, one rung at a time,
  each its own draft, as 4.8's is.
