# Projects at the first run

Draft, 2026-11-03, from a session with the user that day, whose
decisions it records. What projects a deployment has from its first
start through its first run's welcome, and how a person sees, names and
makes them. It carries out `archive/first-run.md`, 2.4 ("No project at
the first start"), whose default project stayed only until jig made
and named projects at runtime, which it now does, and changes what the
welcome does with a forge's repositories. The look of the pages it
names is tuned on the person's `look-requests` branch
(`../client/journeys.md`, 7), landed apart from it.

## 1. In one page

- **The first start makes no project.** `temper init` and the fake
  box seed none; the first owners are the deployment's alone, as jig's
  seed already allows (section 2).
- **The playground makes "Demo",** an ordinary project, shown by its
  stored name like any other; nothing in the client names the
  playground's project in fixed words (section 3).
- **Leaving the playground reverts the box** to the state a person who
  pressed "Set up temper" at the welcome finds: Demo, the fake forge
  and the scripted account go, as the playground's reset already does
  (section 3).
- **One wizard for every forge:** Forge, Model, Ready, on GitHub as on
  Forgejo. It adopts no repository and asks no project name; its last
  act makes the project, "Default", which the person can rename later
  (section 4).
- **A project with no repository is highlighted,** and offers to adopt
  one; it is fully usable meanwhile, since a chat needs no repository
  (section 5).
- **The first chat is the configuring chat,** "Tell temper how you
  work", which names no repository (section 6).
- **The banner names the open project alone,** with a switcher that
  lists the others, renames the open one in place and makes a new one
  (section 7).

## 2. No project at the first start

- **The default document seeds no project:** its `projects` is `[]`.
  jig makes each first owner a deployment owner with no project
  configured (jig's `domain/people.md`, 3), and temper's engine, the
  playground and the client already run a deployment with no project.
- **Policy 1 stays,** a stock setting that projects are made from, not
  a project's.
- **A deployment made before this keeps its project 1:** nothing
  removes or renames it.

## 3. The playground's project

- **"Try the playground" makes a project named "Demo",** as the
  playground's start makes its project today, with its name changed;
  `playground/widget` is adopted into it as now.
- **The client honours the stored name:** no fixed word labels the
  playground's project, in the banner, the chats, a task, the inbox or
  the welcome. Renamed, it shows its new name.
- **Leaving it is the playground's reset,** unchanged: "set up temper"
  sends `EndPlayground`, which deletes Demo, removes the fake forge
  and the scripted account, and opens the welcome's Forge step. The
  person finds the store as one who pressed "Set up temper" at the
  welcome would. What outlives it does so by design: the numbers used,
  never reused, and the spend in closed history.
- **Nothing is added while it runs:** no LLM account and no forge can
  be connected from the playground (`../client/README.md`, 4.8); its
  pages offer no "Connect a model" and no forge to add.
- **The way out is prominent:** while the playground runs, the banner
  offers "Set up temper →", the same act as the pages' line.

## 4. Setting temper up

- **The same wizard on GitHub and on Forgejo:** Welcome, Forge, Model,
  Ready. The Forge step connects a forge and nothing more: it adopts
  no repository and asks no project name.
- **Ready makes the project:** once a real forge is connected and a
  model usable, its last act makes the project "Default", one
  `MakeProject` on the stock policy, and opens it. A failure leaves
  nothing half made; the act is offered again.
- **The welcome is whole** once a real forge is connected, a model is
  usable and the person has a project; no adoption is waited for.
- **Forgejo on this machine** keeps making its sample repository,
  `work/hello`, and leaves it unadopted, so that the first adoption is
  one choice away. Its setup asks no project: the request's project
  goes.
- **On GitHub,** connecting temper's App is the Forge step; the
  repositories its installations reach are offered at adoption.
- **The name is the person's later:** "Default" is renamed from the
  switcher (section 7), never asked in the wizard.

## 5. A project with no repository

- **Usable:** chats start and run in it, a chat that names no
  repository running with no workspace (`../domain/agent.md`, 4.2).
- **Highlighted** wherever the project's repository would show, on the
  chats page's side pane above all, with "Adopt a repository", to the
  adoption form into this project (`../client/README.md`, 4.4).

## 6. The first chat

- **The configuring chat** (`configuring.md`, 5.1): after Ready, the
  project's chats page offers "Tell temper how you work", a chat from
  the configuring agent's stock template, which names no repository.
- **Until that template is built,** the offer starts the stock chat
  with no repository, and moves to the configuring template when it
  lands, the welcome unchanged.

## 7. Projects in the banner

- **The open project alone,** its name and a chevron; pressed, the
  person's other projects, the open one marked. On a narrow screen the
  switcher is the menu panel's first row.
- **Rename in place:** the open project's row offers "Rename" to its
  owners; pressed, the row becomes its name's field, Enter or "Save"
  sends jig's `Rename`, Escape cancels.
- **A new project:** the list ends with "+ New project", a name's
  field whose "Create" makes the project on the stock policy and opens
  it, with no repository (section 5).

## 8. Its order

1. Sections 2 and 3: no seeded project; Demo, its stored name honoured.
2. Sections 4 and 5: the wizard without adoption, Ready making
   "Default", the highlighted project with no repository.
3. Section 7: rename and a new project from the switcher.
4. Section 6: the configuring chat as the first chat, its interim
   first.

## 9. Open questions

- **A taken "Default":** a person who made "Default" before finishing
  the wizard; Ready then opens it, or makes "Default 2"?
- **Demo after a reset:** a second playground run makes "Demo" again,
  the first one being deleted; nothing is reused but the name.
