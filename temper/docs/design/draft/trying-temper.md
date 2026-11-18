# Trying temper: the first ten minutes

Draft, 2026-11-18, with the user. The first journey of the UX draft
(J0, "the try"), step by step: from a person typing `./temper box`
after the download to their first change landed on their own machine,
in the terminal and the browser at once. Rewritten the same day: the
user set the playground aside, so the try is the set up, and chose to
start from Forgejo on this machine, with the welcome choosing by what
the person wants, in the words section 4 proposes, and fewer choices
(the user, 2026-11-18). GitHub's way is the second path, designed next
(section 6). On paper first; the mockups follow (section 8).

## 1. In one page

- **What the try is for:** a person who has just downloaded temper sees,
  on their own machine and with their own model, a change written by an
  agent, checked by CI, reviewed by an agent, and merged at the commit
  that was checked, with no merge an agent's. Then they bring their own
  code, or say how they work.
- **One press to start:** the welcome asks what the person wants, "Try
  temper here" or "Work on your GitHub repositories", each with what it
  costs: the time, what leaves the machine, what gets access. Trying
  here starts a Forgejo on this machine with a small sample project, at
  once, with no sheet and no form (4.1).
- **Three steps, not four:** Start, Model, Ready. Ready makes the
  project by itself once the forge and the model are; the person names
  nothing (4.2, 4.3).
- **A first change worth making:** the sample is a small program with a
  test its CI runs, so the check is real; Home offers three first
  changes for it (4.4, 4.5).
- **The card says the line:** a change's card in the chat reads
  "Written by an agent", "Checked by CI", "Reviewed by an agent",
  "Landed", each with its fact, the same commit in the last three
  (4.6). temper says it, not the model.
- **Claude Code said first:** Claude's card says whether Claude Code is
  on the machine before it is pressed (4.7).
- **About seven minutes** to the first change landed, none of it
  measured (section 3).

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
1  Ask for a first change
   An agent writes it, CI checks it, another agent reviews it,
   and temper merges it.
   [ Add a --name option ]  [ Greet in Italian with --lang it ]
   [ Add a --shout option ]
2  Tell temper how you work                         Ask temper ›
3  Bring your own code                Copy a repository · GitHub ›
```

"Open Forgejo ↗" shows the sample on a real Forgejo: a short `hello`
script, its test, and a CI file that runs it. Small enough to read in
ten seconds, so the change to come can be judged.

**3:00, the first change.** "Add a --name option". A chat opens: their
model, in its own words, says it will make the change in work/hello,
and under it the change's card:

```
T2 · CHANGE                                                Running
Add a --name option to hello
  ✓ Written by an agent        task 3 · 2 files · View on Forgejo ↗
  ▸ Checked by CI              running at 3f2a1c9
  · Reviewed by an agent
  · Landed
```

The terminal, on Hello's launchpad, says the same chat is working.

**3:00 to 6:00, the wait.** The longest watched wait of the try, and
something new every half minute: the pull request on Forgejo, its diff
readable there; CI's run, on this machine's runner; the review's words.
If CI fails, the card says so and that an agent is repairing it, "1 of
3": the try shows the line holding, not breaking.

**6:00, landed.**

```
T2 · CHANGE                                                 Landed
Add a --name option to hello
  ✓ Written by an agent        task 3 · 2 files
  ✓ Checked by CI              passed at 3f2a1c9
  ✓ Reviewed by an agent       task 4 approved 3f2a1c9: "Adds --name, with a test."
  ✓ Landed                     merged 3f2a1c9 into main, by temper
```

The same commit three times: what was checked is what was reviewed is
what was merged, and the last line is temper's, not an agent's. On
Forgejo, `main` holds the merge. That is the line, seen once, on their
own machine.

**6:30, what next.** To begin's first step is ticked. The next two say
where the climb goes: "Tell temper how you work", the way this change
ran, Agents review, being one they can change; and "Bring your own
code". Ctrl+C in the terminal asks "Stop the box?" while anything runs,
and `./temper box` tomorrow opens on Hello, signed in, with nothing to
click.

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
| 7. Home | "Hello is ready.", To begin, New: its first step (4.5) | Hello's launchpad, "● Live · Hello · Claude Opus 5.5 · high" | the adoption runs on | Home and To begin are built; the first step New |
| 8. The first change | the chat, the card New (4.6), the task's page, "View on forge ↗" | the launchpad, "Hello  Add a --name option, a chat · Working" | the chat's agent delegates a change: an agent writes it, the worker pushes it, the pull request opens, the box's runner runs CI, gate 1 an agent's review, the merge at that head (`../protocol/configuration/stock.md`, kind 2) | the change's way is built; not recorded on this path |
| 9. Landed | the card's four facts | the card's line, "Landed" | the chat wakes on the change's result | yes, but for the card's words |

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
- **The playground** keeps a quiet link under the cards, or none: its
  place is open (section 9).

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

### 4.5 Home's first step: ask for a first change

While the project's repository is the sample, To begin opens with "Ask
for a first change", "An agent writes it, CI checks it, another agent
reviews it, and temper merges it.", and three requests fit to the
sample, each starting a chat with its words: "Add a --name option",
"Greet in Italian with --lang it", "Add a --shout option". Its other
steps follow: "Tell temper how you work", and "Bring your own code",
to a copy or GitHub. To begin then shows three steps, where it shows
two at most today (`../client/pages/home.md`, 1).

### 4.6 The card says the line

A change's card in a chat (`../client/view.md`, 5) gets four steps in
place of "Branch pushed", "Pull request opened", "CI passed" and
"Landed": "Written by an agent", "Checked by CI", "Reviewed by an agent"
and "Landed", each with its fact once done: the task that wrote it and
its files, with "View on Forgejo ↗" once its pull request is open; CI's
verdict at the head; each review gate's verdict and its words at the
head; the merge commit. The last three name the same commit. A change
with no review gate has no "Reviewed" step; a failed check reads
"failed at ABCDEFG; an agent is repairing it, 1 of 3". The card reads
the change's panel, as the task's page does (`../client/pages/task.md`,
3), not the chip alone. Medium.

### 4.7 Claude's card knows Claude Code

Claude's way runs the person's `claude`, which temper does not install
(`../release.md`, 4). The card says, before it is pressed, whether it is
on this machine: found, as today; not found, "Needs Claude Code on this
machine: install it, then come back", a link to its install page, and
ChatGPT's card the default. The engine says whether it found it, beside
the `accounts` document's `lendable`. Medium.

## 5. Where it may still feel wrong

- **Forgejo's fetch on a slow line,** the try's first long wait if the
  sign-in is quick: Ready's bar and "opens by itself" let the person
  look away. Its size and time are not measured.
- **The sign-in's three windows:** the browser, Anthropic's tab and the
  terminal. Each says where to look; the mockups will show whether
  that is enough.
- **The first change's minutes:** the card's steps and the pull request
  on Forgejo keep it moving; a change that takes longer than about five
  minutes would want the card to say so.
- **A red in the first change:** shown as a repair under way, not a
  failure; the try's words for a change held for good are open.
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

## 7. The playground, set aside

The user set it aside on 2026-11-18. This draft's first version walked
it step by step and found its way out said two ways (the scripted
answers and the model picker send the person to Accounts, where nothing
can be connected), its reset taking Demo at once with no question, and
the terminal offering none of its requests. Those stand for whenever it
is taken up again; its own status records the first
(`../protocol/setup/playground.status.md`).

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
| 8 | the chat and its card, checking | the launchpad, working |
| 9 | the card, landed | the chat, landed |

The recorded frames stand for steps 1, 2 and the Model step's first
frame. A recording of the whole try, once its changes land, is a walk
after landing, and measures section 3's times.

## 9. Open questions

- **The sample's language:** POSIX `sh`, which needs nothing more, or a
  language a developer would recognise as real, which needs its
  toolchain on the machine. Default: `sh`.
- **The playground's place on the welcome:** a quiet link under the two
  cards, or nowhere until it is taken up again. Default: a quiet link,
  "Or look around a scripted playground first ›".
- **Ready with no press:** some people may want a moment to name the
  project. Default: no press; the switcher renames it.
- **To begin's third step,** "Bring your own code": on Home, or only in
  the rail's switcher and How work runs. Default: on Home, while the
  project holds only the sample.
