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
change is worked that way. GitHub's way is the second path, designed
next (section 6). On paper first; the mockups follow (section 8).

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
  runs: CI, another agent's review, the merge. This is the rung below
  Pair (4.8), a design of its own.
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

To be designed next, from the same welcome. What differs:

- **Connect GitHub:** temper's App made from a manifest on GitHub, then
  installed on the repositories the person chooses
  (`../client/forge/deployment.md`). The box is on loopback, so
  GitHub's webhooks cannot reach it: temper reads the App's delivery log
  instead.
- **Ready adopts nothing:** the person chooses a repository on Home.
- **The first change runs on their own CI and protection,** the readiness
  gap of the UX draft: what a first change needs to pass is unknown
  until it runs.
- **Their review is GitHub's:** a change waiting for them reaches their
  phone through GitHub's app, the UX draft's other gap.
- **About ten minutes,** none measured.

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

## 9. Open questions

- **The sample's language:** POSIX `sh`, which needs nothing more, or a
  language a developer would recognise as real, which needs its
  toolchain on the machine. Default: `sh`.
- **Ready with no press:** some people may want a moment to name the
  project. Default: no press; the switcher renames it.
- **The rung's name:** working with an agent in its own copy is what
  "pair programming" names, so "Pair" fits it better than the rung
  above, where the person approves each change a delegated agent made
  (the UX draft's rung 1). Default: none chosen here; the names are the
  user's, in 4.8's own draft.
- **The first change's rung:** worked with an agent (4.8), or handed
  over as today, the try then needing nothing of 4.8. Default: worked
  with an agent, the try's first change waiting for it; until it lands,
  the try's first change is a delegated one, its card saying "Written by
  an agent".
