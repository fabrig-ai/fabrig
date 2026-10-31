# Removable forges and projects, and the playground's reset

Draft, 2026-10-31, from a session with the user (10-28 to 10-31).
Replaces the earlier `playground-apart.md`, which kept the playground on
a store of its own in memory and held a set up in memory until it stood
whole: both worked around two missing things, forges kept in the store
and projects that can be deleted, which temper wants anyway (the user,
10-31). This draft builds those two, then puts the playground and the
set up on them, with no workaround left. Section 2 is today, sections 3
to 6 the change, section 7 its order. Nothing here is adopted.

## 1. In one page

- **Forges are records in the store,** made by the acts that set them
  up or connect them, numbered at runtime and never reused, and no
  longer named by `temper.json` (section 3). The document keeps what is
  the machine's: the listeners and the ports a box's forge and fakes
  bind.
- **A forge can be removed,** whole: ended at once, as the playground's
  forge is ended today, then its live work cancelled, its repositories
  out of their projects, its secrets and records erased, and its box
  group stopped and its directory removed (section 4).
- **A project can be deleted** (jig's): closed to new work, its tasks
  ended and erased at once as retention erases an ended task, then its
  own rows (section 5).
- **The playground is ordinary records:** a fake forge, a scripted
  account and the playground's project, made when the person chooses it
  (6.1). **Its reset** is the banner's "set up temper": the project
  deleted, the fake forge removed, the account removed, and the person
  on the welcome's Forge step as if they had never tried it (6.2).
- **A set up writes as it goes, as today, and starts over by removal:**
  an unfinished forge found later is removed and set up again,
  Forgejo's binaries kept in the cache (6.4). No staging in memory, no
  pending file, no commit across two stores.
- **The store is empty, unfinished or whole.** The playground is
  offered while no real forge is kept, and never again once one is
  (6.3).
- **What stays in the document** after this draft: the LLM providers,
  each a design of its own later (`configuring.md`, section 10). The
  scripted provider can become the binary's stock first (6.1).
- **Projects as they are** (the user, 10-28): the seeded project, the
  welcome's Default, the playground's: made as today. This draft only
  adds their deletion.

## 2. Today

From the code at `main` 3714194a9, with 41b98d50d's welcome:

- **Forges and providers are in the document.** `temper init` writes
  forge 1 (the playground's fake), 2 (GitHub) and 3 (the owned
  Forgejo, asked), their `setup` entries, `application.demo`, and
  `engine.llm` with the demo provider, 3, at the fake LLM's port
  (`crates/temper-init/template/temper.json`). The store holds the
  forges' secrets, the accounts and the `Demo` record.
- **Nothing is ever removed.** jig has no act that deletes a project or
  a task; an ended task is erased only by its retention (jig's
  `domain/tasks.md`, its retention class). temper's forge ends only for
  the playground's, which then leaves every list while its rows stay
  readable (`../domain/forge.md`, 4.1). Accounts are removed by jig's
  `RemoveAccount`.
- **So the playground can only end, never leave:** when another forge
  connects, `Demo` ended, its chats closed "Ended with the playground"
  and left readable, its account removed (`../protocol/setup.md`, 8.4),
  and the client's model's and forge's halves judged apart
  (`../client/README.md`, 4.8).
- **A set up writes as it goes:** the forge's secrets at reconcile, the
  project and adoption once it connects, each account when added. A
  restart before the first repository loses its intent, held in memory
  (`../protocol/setup.md`, 6.7), and no test walks a set up cut short.
- **The document sized memory from its forges** until 272c945cc made
  `memory_bytes` a budget metered at runtime (programming-model.md,
  6.3): what was the first reason forges stayed in the document is
  gone.

## 3. Forges in the store

- **A forge is a record of the forge connector,** as its repositories
  are: its kind (GitHub; Forgejo connected, managed or owned; fake), its
  addresses, its bot's git identity, its setup entry (`setup.md`,
  section 8, as the document has it today) and, for a first set up,
  the first repository's intent. The acts that set a forge up or
  connect it make it: `SetUp` and `Connect` carry what the document
  carried, and the forge is numbered next, never reused, since its
  number keys its records (`../protocol/credentials.md`, the records).
- **The intent is kept,** so a restart before the first repository
  makes it at the next connect, instead of losing it
  (`../protocol/setup.md`, 6.7).
- **The document keeps the machine's:** the clients' and webhooks'
  listeners, a box's owned Forgejo's port and the fakes' ports, which
  `temper init` chooses and the box holds from its start
  (`../protocol/setup.md`, 7.1); the limits. A first run's document
  names no forge.
- **The box knows kinds, not forges.** Under the box, serve writes a
  group's settings, the forge's setup entry and its port, to
  `<data>/box/groups/N` and asks `start N` on the pipe, as it asks a
  dormant group today. The box builds the group's steps by its kind,
  an owned Forgejo's or the fakes' (`temper-forge-setup`'s and
  `temper-fake`'s steps, today read from the document), and at a later
  start runs each group whose `started` record exists from its file.
  It stays wiring only.
- **Limits are counts:** `forge.forges`, the forges kept at most, and
  `forge.repositories` and its kin as the deployment's, not a sum over
  the forges configured. Memory is metered as any held bytes are.
- **Its client:** Forges lists the records; Connect a forge and the
  welcome make one. Nothing changes for a person, but that a fresh
  deployment lists no GitHub or Forgejo slot until one is chosen.

## 4. Removing a forge

- **`RemoveForge { forge }`,** of the deployment's scope, its owners'
  act, and the deployment's own when granted by `engine.acts`, as the
  demo's end's acts are today (jig's `domain/people.md`, 5.3). In one
  decision the forge is ended (below); the rest follows as kept acts.
- **Ended at once,** as `../domain/forge.md` 4.1 ends the playground's
  forge today, for any forge: no list shows it, no call is served, its
  hints are dropped, the setup host sets it up no more, and its
  repositories leave their projects.
- **Its live work cancelled:** each change on its repositories, by the
  deployment's compound cancel (jig's `domain/tasks.md`, the compound
  request), so its tasks end as a cancel ends them. A chat that worked
  on it stays open; its next delegation to the forge is refused as on a
  forge not connected.
- **Its box group stopped and removed:** `remove N` on the pipe; the
  box stops the group, removes `<data>/box/groups/N`, its `started`
  record and the forge's directory (`<data>/forge-N/`, or a fake's),
  and keeps the cache, Forgejo's binaries with it.
- **Its records erased:** its secrets at once, the admin's token, the
  bot's tokens, the webhooks' secret and the App's key among them; its
  writer and its flights at once; its repositories' rows once no kept
  task names them, so that a chat that worked on one still reads its
  name until the chat itself is erased.
- **On the forge itself,** nothing is undone but what temper ran: an
  owned Forgejo goes with its directory; a managed Forgejo's bot and
  hooks stay, and the page says so; a GitHub App stays the person's,
  and the page links to its settings to delete it.
- **Kept until whole:** each act after the decision is a row kept under
  its key and sent again at each start until answered, as the demo's
  end's `DemoAct` rows are today (`../protocol/setup.md`, 8.4), so a
  removal a restart cut ends whole.
- **Its client:** Remove on the forge's row in Forges, confirmed, its
  words naming what goes and what stays on the forge.

## 5. Deleting a project (jig's)

- **`RemoveProject { project }`,** the project's owners' act, and the
  deployment's own when granted by `engine.acts` (jig's
  `domain/people.md`, 5.3), which the playground's reset uses (6.2).
- **Closed at once:** in its decision, the project takes no new task,
  message, adoption or settings change, each refused as on a project
  that is gone, and leaves every list.
- **Its tasks ended, then erased:** each live task cancelled as a
  conversation's close cancels it, with its grace (jig's
  `domain/tasks.md`, the compound request); once each has ended, it is
  erased whole at once, as its retention would erase it at the horizon
  (jig's `domain/tasks.md`, its retention class; `protocol/store.md`,
  11.1), its transcript's file with it.
- **Then its own rows:** its policy's copy, its roles and parties, its
  pools in the current period, the settings of its scope (the connector
  owning each kind told, temper erasing its kinds 2, 4 and 5:
  `../protocol/configuration.md`, 13.2), and its adoptions released,
  each connector told (temper's repository rows for it, section 4's
  rule). Its name is free again; its number is never reused.
- **What outlives it:** spend, in the deployment's and the parties'
  totals for the period, never per task; an act's retained answer that
  named it, which answers unknown, as one past its retention does. Each
  party's inbox and read positions drop what named its tasks.
- **Kept until whole:** the deletion is a row, resumed at each start
  until its last step, so a deletion a restart cut ends whole and no
  start serves a half-deleted project.
- **Tested in jig's conformance world:** each family of jig's store with
  a case, and the referee checking at the end that no row names the
  project.
- **Its client:** Delete on the project's settings, confirmed by its
  name, for its owners.

## 6. The playground and the set up on them

### 6.1 The playground, made when chosen

- **"Try the playground"** sends `StartPlayground` (today `StartDemo`),
  the deployment's acts, as the demo's start makes them today
  (`../protocol/setup.md`, 8.4): a `Playground` row, running, naming
  what it makes; the fake forge's record, its setup entry the
  playground's; the box asked for its group (section 3); the scripted
  account, the deployment's, its uses naming the playground's project
  alone (jig's `domain/accounts.md`, 4.2); and, once the fake forge
  connects, `playground/widget` adopted into the playground's project.
- **The scripted model, the playground's alone:** the playground's
  project allows only the scripted model, and no other project allows
  it, so a person's own account added before (6.3) never runs the
  playground's scripts, nor the scripted account a real project's work.
- **The scripted provider** stays in `engine.llm` until providers move
  (section 1), or becomes the binary's stock first; its endpoint is the
  fake LLM's port, the machine's.
- **Its pages:** Forges lists the playground forge's row and Accounts
  the Scripted model, neither offering anything to add while the
  playground runs; every page's banner is one line: "This is the
  playground: a scripted model and in-memory forge. Use the wizard to
  [set up temper]". The setup strip and the demo banner's Connect
  links go.

### 6.2 The reset

- **"set up temper"** on the banner sends `EndPlayground`: in one
  decision the `Playground` row reads resetting and the forge connector
  keeps the reset's acts, in order: the playground's project deleted
  (section 5), its chats cancelled with it; the fake forge removed
  (section 4), its group stopped; the scripted account removed by
  `RemoveAccount`; then the row erased, none again.
- **The page** shows "Leaving the playground…" until the reset answers,
  then opens the welcome's Forge step, `/welcome?at=forge`
  (`../client/journeys/welcome-set-up/02.html`): the store as a person
  who pressed "Set up temper" from the welcome finds it.
- **Cut by a restart,** the kept acts are sent again at the next start
  (sections 4 and 5), and the page shows the reset under way until the
  row is none.

### 6.3 Empty, unfinished, whole

- **Empty:** no real forge and no account of a real provider. The
  welcome offers "Try the playground" and "Set up temper".
- **Unfinished:** a real forge kept, but not connected with its first
  repository adopted, or no real account usable. The welcome opens at
  the step left undone (6.4).
- **Whole:** a real forge connected with its first repository, and an
  account serving a model. The welcome moves on to the chats page.
- **The playground is offered** while no real forge is kept, and refused
  *not offered* once one is: an unfinished forge is started over first
  (6.4), and a whole one never. A real account alone does not end it
  (6.1).
- **What goes:** `Demo`'s ended and its end on another forge's
  connect, the client's model's and forge's halves, the closed chats
  "Ended with the playground", and `engine.acts`' `close`, replaced by
  the grants of the reset's acts.

### 6.4 A set up cut short

- **It writes as it goes,** as today, now with the forge's record and
  the first repository's intent kept (section 3).
- **Found later, it starts over:** the welcome's Forge step shows the
  unfinished forge, "FORGE · not finished", with Start over, which
  removes it (section 4) and opens the step's choices again. An owned
  Forgejo then sets up afresh: its binaries from the cache
  (`../protocol/setup.md`, 6.6), its directory prepared anew.
- **GitHub goes on by default:** its App's records are kept as they
  come, so the step offers Go on, to the App's installation, before
  Start over, which removes the forge and names the App on GitHub as
  the person's to delete (section 4).
- **An account added before** stays: it is the person's, not the set
  up's, and the Model step shows it ready.

## 7. Order

1. **Forges in the store** (section 3): the records, the box's groups by
   kind, the limits as counts, `temper init` naming no forge.
2. **Removing a forge** (section 4), on section 3 and the forge's end;
   with it, a set up started over (6.4).
3. **Deleting a project** (section 5), jig's, on its retention's
   erasure.
4. **The playground on them** (6.1 to 6.3): its records, its reset, and
   what goes.
5. **The LLM providers,** later, their own design.

## 8. Testing

- **The process tier:** a first run into the playground and out by its
  reset, the store then read as empty, the welcome at Forge; the box
  killed during a reset, which ends whole at the next start; a set up
  killed at each step, found unfinished and started over, Forgejo's
  binaries taken from the cache; GitHub gone on after a restart, on
  `temper box --fake-backends`' fake GitHub (`fake-backends.md`).
- **The engine's world:** a removal and a deletion cut by a crash after
  each kept act, resumed to the same end; nothing of the forge or the
  project read afterwards but what 4 and 5 keep.
- **jig's conformance world:** section 5's case per family, and the
  referee's check.
- **The client:** the welcome's three states, the playground's pages
  with nothing to add, the banner's link, Remove and Delete confirmed.

## 9. Open questions

- **A box restarted in the playground.** The fake forge keeps no state
  across its starts, so the engine's records of the branches it lost
  outlive it (`../protocol/setup.md`, 8.4). Default: the start resets
  the playground, as the banner does, and the person meets the welcome.
- **A removed forge's history:** its repositories' names kept while a
  task names them (section 4), or erased at once. Default: kept.
- **Who deletes a project:** its owners, and the deployment for the
  playground. Default: as said; the deployment's owners over any
  project, later.
- **A deleted project's spend:** kept in the totals (section 5).
  Default: as said.
- **The welcome's Default project** after a start over: the project a
  removed forge's first repository was to go into, made already, stays
  empty. Default: reused by the next set up's first repository.

## 10. What it amends, once adopted

`../protocol/configuration.md` (the forges' members leave the
document), `../protocol/setup.md` 7.1 (the pipe's `remove` and the
groups' files), 8.3 (the default document names no forge) and 8.4 (the
demo's start and end, replaced by 6.1 and 6.2); `../domain/forge.md`,
4.1 (any forge ended, then removed); `../client/README.md`, 3.5 (the
welcome's states) and 4.8 (the demo's halves); `../client/forge.md`
(Remove); `first-run.md`, 2.3 (the demo's end) and 2.4 (the
playground's project, deleted at its reset); `configuring.md`,
section 10 (forges' addresses no longer later); jig's
`domain/people.md` (`RemoveProject`) and `domain/tasks.md` (erasure
at once).
