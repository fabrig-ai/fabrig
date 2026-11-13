# temper's pages

Before its document arrives, a task page says “Reading task N…”, or one
sentence explaining a lasting refusal; after a transient read end it asks
again while open, waiting one second, doubling to at most thirty seconds.
A conversation's activity reads, a chat's and an agent's task's, follow
the same rule: a read that ends transiently (busy) is asked again from
the position it asked for while the page is open, on the same waits, so
a committed reply always arrives without a further delivery to prompt it
(jig's `client/domain/client/README.md`: a read is one attempt, and its caller
asks again).

No page claims empty until its read has answered. "Nothing needs you.",
"No chats yet.", "None.", "No updates." and "No goals yet." stand only on a
list whose read has answered with its value. A read that ends without one
(busy, unreachable, refused, past a bound) is no answer, and neither is a
watch that has delivered nothing, whether it is opening or Behind. While a
read hasn't answered, on first load or after the link comes back, a page
keeps what it last knew, rows and words alike, through every refresh; with
nothing known, a section draws its title with no line under it.

Each page has an address, and watches what it shows while it is open;
leaving it closes its handles. Its mockup designs what temper does not
draw yet: its look, its parts and their order, its words and where its
acts lead (`../web/mockups/README.md`, section 1). Lists of tasks are
task queries (jig's `client/domain/contract/README.md`, 4.1): the chat and
goal templates' labels are jig's defaults, which temper's configuration
keeps (`../../protocol/configuration/README.md`, section 2).

| Address | Mockup | Shows | Watches and reads | Acts |
|---|---|---|---|---|
| `Home { project }` | the greenfield mock's `home.html` | a project's first page: its intro, "To begin" while it shows, what needs the person, its chats (`home.md`, 1) | `me`; the `forges` watch; the person's dismissals (`../forge/records.md`); the project's chats' `tasks{query}`, as `Chats` reads them; `inbox` for that project, as `Inbox` reads it; `settings`, to say whether a way of working is configured | open Ask temper (`ask-temper.md`, 1, *The first chat*); adopt the first repository again, or open the adoption form; skip "To begin"; a question's choice, reply and Later, and a card's acts, in place |
| `Settings { project }` | the whole story's `web/whole-story/how.html`; the greenfield mock's `settings.html` for its foot | How work runs, the first tab of a project's settings: the way the project runs, the path of a change by who acts, the ways as tabs, what reaches the person, a goal's budget, and at its foot what the project shows and its ways of working (`settings.md`, 2, *The page*) | `me`; the session's `settings` watch and the project's kinds 6 and 7 (`settings.md`, 2); the project's `ways` (`../forge/records.md`); the project's chats' `tasks{query}`, as `Chats` reads them | jig's `ChangeSettings` of one item (`settings.md`, 2); show a way's tab and a stop's card in place; "Use this" or "Keep it" (`settings.md`, 2, *A way's acts*) |
| `Repositories { project }` | the greenfield mock's `settings.html`, its tab, and `chats.html`'s band with no repository | the second tab: the project's repositories, or that it has none, and "Adopt a repository" (`settings.md`, 1) | `me`; the `forges` watch; the playground's read (`../forge/family.md`, 7) | go to `Adopt { project }` |
| `Models { project }` | the look request `model-picker`'s `journeys/model-picker/07.html` | Deployment's third tab: what the picker offers, by provider, an owner turning a model on or off and adding one by name (`settings.md`, 1) | `me`; the accounts document; jig's settings, for an owner's act | none |
| `Builders { project }` | none: the greenfield mock's tabs say "not drawn in this experiment", so its How work runs tab's language | Deployment's fourth tab, Workers: the deployment's hosts, each with its slots and its connection, read-only (`settings.md`, 1) | `me`; jig's `deployment` document, for a person with a role in the deployment | none |
| `Role { project, charter }` | none: the look request `ask-temper-ways`'s card names the role it opens | one role of the project, a charter: its name, who it is and its facts, as Ask temper's `charter N` view draws it (`settings.md`, 1, *A role's or a rule's page*) | `me`; the forge's read `View` of `charter N` in the project (`../forge/records.md`) | none |
| `Rule { project, kind, number }` | none, as `Role`'s | one landing rule, of the deployment (kind 1) or the project (kind 2): what it covers and asks, as the view `rule K/N` draws it (`settings.md`, 1, *A role's or a rule's page*) | `me`; the forge's read `View` of `rule K/N` in the project (`../forge/records.md`) | none |
| `Ask { project }` | the greenfield mock's `chat.html?id=new&configure=1`, out of Settings, as the look request `ask-temper-place` places it | Ask temper before its first words, in the project's frame, through no Settings: the ways to pick, drawn (`ask-temper.md`, 1, *Ask temper's page*) | `me`; the project's chats' `tasks{query}`, as `Chats` reads them; To begin's facts, as Home reads them, for the rail (`home.md`, 1); the project's `ways` (`../forge/records.md`) | start an Ask temper chat from its composer; show a way's tab and a stop's card in place; "Use this" or "Keep it", jig's `ChangeSettings` (`settings.md`, 2) |
| `Team { project, role }` | the whole story's `web/whole-story/team.html`, and `team/team.html` | a project's areas and their owners, each owner's Now and spend, and Who routes; its standing roles that own no area and one role's card, its head, Now, rules, what wakes it, its sessions this week and its weekly report; a project's people, each with their role, their areas and what waits on them (`team.md`, 1) | `me`; jig's `Read::Project { project }`; the forge's `findings`, for its areas and triager; the roles' watch, a `tasks{query}` of label 3; each role's Now, `Read::Notes`; the card's role, `Subject::Task`, its charter's `View`, its `Read::Activity` and its `report` note; the standing inbox watch and the person's approvals, as Needs you's; a `tasks{query}` watch for each other person read; kind 14's setting, when an act sends | "+ Hand an area to a role" and "Give the area back", kind 14 by `ChangeSettings` or a proposal, and a stock role's start; a role's "Talk to it", a link to its page; a role's Pause and Resume, `Ask::Stop` and `Ask::Release` (`task.md`, 4) |
| `Talk { project, task }` | the whole story's `tui/whole-story/role.html` | a person's talk with a standing role: the role's chat page in its talk, its conversation from the talk's opening, its name, areas, period's spend and Now (`talk.md`, 1); drawn in the terminal (`../tui/pages/talk.md`), and on the web as a chat until Team's "Talk to it" draws it (`talk.md`, 4) | as `Chat { task }` reads, of the role's task; its Now, `Read::Notes`; the project's findings document, as Needs you's; the project's reads, as `Team`'s | talk, `Ask::Talk`; leave, `Ask::Leave`, on leaving the page; accept a proposal of the talk in place, running work moved |
| `Inbox` (the root) | the greenfield mock's `needs.html`; the terminal's, `inbox.html` | in the browser, *Needs you* (`needs-you.md`, 1): what needs the person, one card at a time, across projects or filtered by project, kind and goal (its root), a proposal of settings in each project it shows in (jig's `domain/engine/settings.md`, 6), its card, count and Accept as in its chat. In the terminal, the list: proposals, questions, held tasks, person tasks; then updates (results, replies); its filters; "+ New chat", a link to `NewChat` (`/chats/new`) | `inbox` with its filters; the `proposal` or `task` an entry's card needs | a question's choice, reply and Later (`needs-you.md`, 1, *What the client keeps*); accept ("Accept"), reject with a reason, release, leave held, take, hand back; in the terminal, answer in words |
| `Chats { filter }` | the greenfield mock's `chats.html` | a page of the frame (`chats.md`, 1): its head; while the project has no repository, its band; the list, under All, Live, Drafts or Closed, with "+ New chat"; "More closed chats" | `tasks{query}`: requester the person, the chat label, its live group by last activity, its ended pages by when each ended; the project's documents, as it opens (`chat/document.md`, 1); the person's facts (`../forge/records.md`) | choose the list's filter; more closed chats |
| `NewChat { draft }` | the greenfield mock's `chat.html?id=new`, and `&draft=` for a draft; the playground's start at the first run's first step | a new chat in the open project, on the layout every chat takes (`chat/README.md`, 2), in the playground's project the playground's start (`../forge/family.md`, 7, `welcome.md`, 2); with `draft`, the empty document beside it (`chat/document.md`, 1) | as `Chats`; the playground, from the forge's side (`../forge/family.md`, 7) | start a chat, from the composer or a suggested request; "+ Draft" |
| `Chat { task }` | the greenfield mock's `chat.html` | the layout every chat takes (`chat/README.md`, 1, *One layout for every chat*): its bar, with its path, its heading, its number, its status, Pause or Resume, and Close, and its spend in the view alone; the context line, only in the playground's project while the playground runs (`../forge/family.md`, 7, `welcome.md`, 2); the conversation (`chat/conversation.md`, 1): its request, the agent's words with its calls folded, the person's words, provisional text, and its end once it has ended; each task it delegated as that task's card, at the call that made it (a `delegate`, a `change` or a `goal`), or else at the result it sent back, or else after the conversation while it runs (`../view.md`, 5); the proposals waiting in its subtree (jig's `client/domain/objects.md`, section 10) and the decided proposals of settings its activity says it made, each at the call that made it where its turn names it, and otherwise after the turn that was running when it was made, so that what comes later comes after it (`settings.md`, 2); the composer, while the chat lives: the chooser's feedback above it, its text area "Message" with the placeholder "Reply to T6", T6 the chat's number, shown while it is empty and never its name, and "Send", the hint "Enter sends · Shift+Enter for a new line", or "Sending resumes this chat." on a chat the person paused (`chat/README.md`, 1, *A person's pause*), then a message's refusal or failure, if any: a message or the chat's start accepted draws no status of its own, neither at the conversation's end nor in the composer, since the conversation shows its words, "Being sent", then "Sent" (`chat/conversation.md`, 1); while Message is not offered, the feedback alone; an ended chat draws no composer | `task`, `activity` a page at a time, earlier and later pages at the conversation's log edges (`chat/conversation.md`, 2), `run` while one is live; each `proposal` its `task` names, or its activity's lines say it made; each delegated `task` its calls, results and live delegates name, up to the domain's `reads` (`../view.md`, 5); its document, read again when its run writes it (`chat/document.md`, 1); the person's facts (`../forge/records.md`) | message, pause, resume, close, change the next run's model, accept or reject a proposal in place; choose Chat, Split or Draft; "+ Draft"; start a goal from its document; on an Ask temper chat, start over |
| `Goals { project, filter }` | the greenfield mock's `goals.html` | Work under its Goals filter: Work's head and filters (`work.md`, 1, *Work*), then the project's goals, their filters, a row a goal, each its state and its report's headline (`work.md`, 1, *Goals*) | `me`; `tasks{query}`: the project, the goal template's label, tracked, by priority, its live group watched, its ended pages by when each ended; a row's report, one read at a time; `inbox`'s needs for that project | choose the filter; more goals |
| `Work { project, filter }` | the whole story's `web/whole-story/work.html` | the project's work, under way and lately done: Work's head and filters, then, under All, In flight, Drafts and Landed this week, or, under Drafts, the drafts alone (`work.md`, 1, *Work*) | `me`; the goals' reads, as `Goals` reads them; the subject `changes` and a `task` per row shown, as `Changes` reads them; the person's chats' `tasks{query}` and their documents' notes, as `Chats` reads them; the subject `drafts`, the project's landed drafts (`../forge/records.md`) | choose the filter; each card and row links to its page; a landed draft's card starts a goal from it, or opens the start sheet for it (`goal.md`, 3) |
| `Task { task }` | the greenfield mock's `goal.html`, for a goal; `task.html` otherwise | a goal: its header, its needs as bands, its report, its tasks, landing, how it runs and what it decided, what the person sent (`goal.md`, 2), with Amend, Pause or Resume, Close and its composer (`task.md`, 4); a change: its header and "View on forge ↗", its steps, its step's line, its hold, its tasks live and ended, its gates at its head and its bounds (`task.md`, 3); an agent's task: its lineage, its brief, its run's heading, its conversation as a chat's, its result once ended and its plan's held cards (`task.md`, 2), with Amend, Pause or Resume, Close and its composer (`task.md`, 4), and its own held card while it is held for any cause but the person's own pause | `task`, `tree`, `activity` a page at a time, earlier and later pages at an agent's conversation's log edges (`chat/conversation.md`, 2); `run` while one is live; a change's panel, through the forge's side (`../forge/served.md`, 2); a goal's or a procedure's ended delegates, a page read once and again on a change; a goal's report, its own line and its needs (`goal.md`, 2) | as its document offers them (`task.md`, 4): on a goal amend, pause, resume, close, message, and answer its needs; on an agent's task message, amend, pause, resume, close, release its own held task or one of its plan; on a standing role's, talk in place of message, and leave its talk (`task.md`, 4, *A standing role's talk*) |
| `Changes { project }` | the greenfield mock's `changes.html` | Work under its Changes filter: Work's head and filters (`work.md`, 1, *Work*), then what is on the way to the project's landing branches and what last landed there (`../forge/pages.md`, *Changes*; `../forge/README.md`, section 3) | the subject `changes`, through the forge's side (`../forge/family.md`, 2, `../forge/records.md`); a `task` per row shown, for where it came from | none: each row links to its change's `Task` |
| `Adopt { project }` | none: `../forge/family.md`, 4 | under its path (`settings.md`, 1), the adoption form, into `project` (`../forge/family.md`, 4), with the picker of the repositories temper can reach on the chosen forge | `me`; the chosen forge's deployment part, for its kind, mode, user and installations (`../forge/deployment.md`); the picker's page, a `Reachable` request (`../engine.md`) | adopt a repository (`../forge/family.md`, 4) |
| `Forges` | `forges.html` | inside Deployment's frame while a project is open (`settings.md`, 1): the person's forges, each row folded until opened: its connection, its mode and access, on GitHub temper's App and its installations, a connect under way and how the last ended, its load; then the playground's forge, folded, while the playground is read (`../forge/family.md`, 7); "+ Connect a forge", to `ConnectForge`, but while the playground runs (`../forge/deployment.md`) | the forge's, through the forge's side (`../forge/family.md`, 2) | connect a stopped forge; replace a key or tokens; install on another account; new repository; forget the admin's token; set up again; remove a forge, confirmed (`../engine.md`) |
| `ConnectForge { forge }` | `connect-forge.html` | under its path (`settings.md`, 1), connecting one more forge, `forge` the choice it is at: the shared choice cards or a way’s panel, then the open row on Forges, adopting nothing (`../forge/deployment.md`) | the forge's, through the forge's side (`../forge/family.md`, 2) | connect; connect as its admin; set up a Forgejo here (`../engine.md`) |
| `Welcome { step }` | `welcome.html`; the first run's step 0 | the first run's welcome, by the store's state, empty, unfinished or whole (`welcome.md`, 1): "Try the playground" or "Set up temper", the latter a wizard of Welcome, Forge, Model and Ready, its `step` (`welcome.md`, 2; `../../draft/archive/first-run.md`, 2.3) | the playground's read (`../forge/family.md`, 7); `accounts`; the `forges` watch, for a person whose role holds the deployment's forges; the forge's, as `ConnectForge` reads it | start the playground; start a forge cut short over, or go on with it; `ConnectForge`'s and `ConnectModel`'s acts |
| `Accounts` | `accounts.html` | inside Deployment's frame while a project is open (`settings.md`, 1), and there "Sign out" at its foot, in a browser at a phone's width alone: the person's LLM accounts, then, for an owner, the deployment's, each row folded until opened: its LLM provider and display name, its holder, its deciding source's state and its line (`accounts.md`, 2, An account's line); opened, that state's words and acts, its trial's line under the head and before the sources (`../../domain/trial.md`, 6.2), its sources in order, each with its state in words and, for a program's, on which hosts (`accounts.md`, 2), what it pays for, the runs using it and the tasks waiting for it (facts jig gives the deployment's accounts alone for now, so a person's own leave them out: `accounts.md`, 2) and its acts; the checks under way, and how the last ended; then the playground's account, folded, while the playground is read (`../forge/family.md`, 7); "+ Connect a model", to `ConnectModel`, but while the playground is read | `accounts` (jig's `client/domain/contract/README.md`, 5.1); `me`'s LLM providers | sign in again or add a key from a state's acts, as `ConnectModel`'s ways; sign a source out; add a key to an account; replace a key; remove a source or an account; check a program's login again; move an account, or a source, up or down; resume a limited source |
| `ConnectModel { way }` | `connect-model.html` | under its path (`settings.md`, 1), connecting one more LLM account, `way` the way it is at, or none at the choices: the welcome’s cards (`accounts.md`, 2, The ways), then a sign-in’s wait or a key’s or program’s panel; success returns to Accounts with a confirmation line, and other endings keep their words and acts (`accounts.md`, 2, A sign-in) | `accounts`; `me`'s LLM providers | sign in with an LLM provider; add a key as an account of its own; declare a program's login |
| `Members` | none: `../../draft/surfaces.md`, section 8 | Deployment's fifth tab: each project's members, each with the role it holds there, and the person's role in the deployment (`settings.md`, 1) | `me`; jig's `Read::Project` for each project `me` lists, one at a time | none |
| `Limits` | none: `../../draft/surfaces.md`, section 8 | Deployment's sixth tab: the deployment's capacities as its engine reports them, read-only (`settings.md`, 1) | `me`; jig's `deployment` document, watched as `Builders` watches it | none |
| `Remembered` | none: `../../draft/ask-temper.md`, section 7, drawn in Deployment's language | "What temper remembers about you": the lines Ask temper keeps about the person across their chats, each with the chat that kept it and when, or that none are kept (`remembered.md`) | `me`; jig's `Notes` read of the person's own party scope (jig's `client/domain/contract/README.md`, section 4) | change a line; delete a line (`remembered.md`) |
| `Told` | none: `../../draft/what-reaches-you.md`, section 9, drawn in Deployment's language | "Telling you": whether this browser is pushed to, with its act, and the person's devices, each with Remove, or that the deployment doesn't push (`told.md`) | the deployment's read's `push`; the forge's `devices` subject; the browser's push state, from the shell (`../web/README.md`, 2.7) | turn this browser on or off; remove a device (`told.md`) |
| `Proposal { proposal }`, later (`settings.md`, 2) | none: the user's (`settings.md`, 2) | a proposal of settings: temper enforces and your agents follow, its effect on running work, the settings folded; withdrawn when stale; accepted (`settings.md`, 2) | the `proposal`; `settings`; each item's setting before | accept, choosing whether running work moves; reject with a reason; change something, in the chat |

- **Words and facts:** a chat's status from jig's facts (jig's
  `client/domain/client/helpers.md`), in jig's words, with temper's
  own in places ("Waiting for you", "Parked"); a conversation from
  jig's thread model; jig's enums and a decision's facts in jig's
  words, under temper's phrasebook (`../view.md`, 1);
  each card temper's own (`../README.md`, section 1).

## 1. A chat

## 2. Across pages

- **Links are by number:** a chip, an entry or a line naming a task
  opens its page, from any page (jig's `client/domain/pages.md`,
  section 2).
- **Away** (the greenfield mock's `home.html`, `phone.html` and
  `settings.html`'s `#away`; `../../domain/forge/attention.md`, 4 and 6;
  `../../draft/what-reaches-you.md`, section 3): a person's switch on a
  project, while which agents decide for that person what the project's
  classes let Away decide, each listed under *Decided for you* (`home.md`, 1). The
  classes are the project's, the switch each person's. It is the rail's and
  Settings'.
  - *Shown* while either holds, one function's answer (`Domain::away_shown`),
    so the rail and Settings agree: the project's classes, kind 8 as `settings.md`, 2
    reads it, hold an item Away may decide (`Classes::away_may_decide`,
    `../../domain/forge/attention.md`, 6), or the party's row there is on, so that a
    switch left on can be turned off. A project on You approve's table has no
    switch, and one with no value of kind 8 has Agents review's table and so
    has one. Until kind 8 is read, or after a lasting refusal, only an on row
    shows it.
  - *The control,* in the rail's foot above Settings, a button, `State::Pressed` while on,
    in `temper-away`, the mock's `.away`: 8 below it, full
    width, 8 10 padding, 1px `--gf-line`, radius 8, `--gf-surface`, 13.5 px
    muted, a 30 by 18 track holding a 14 px knob, by style; on, the ink ground,
    white words, the knob at the right. Its words: "Away" off; "Away until
    08:00" on, `until` by `../view.md`, 6's *clock*; "Away" alone on with `until` none.
    On a phone the foot is hidden: a second button, `temper-away-chip`, the
    mock's `.away-chip`, at the top bar's right (`../view.md`, 3) beside the project switcher,
    min-height 36, radius 18: "Away: off", "Away until 08:00", "Away" alone
    on with `until` none; the browser's alone. In the terminal the rail's
    button, numbered as any. While its request is out it is disabled.
  - *Its act* sends `SetAway` (`../forge/records.md`): to off, `on` false and the row's
    `until`; to on, `until` none where the row's is none, otherwise the next
    08:00: the first wall after now whose hours in the person's zone (`../view.md`, 6)
    read 08:00, computed by one pure function, `forge::away::next_eight`. A
    row's `until` once past means nothing but that the choice is 08:00. What
    Away decides is the classes', never the act's.
  - *Past `until`* agents are told to ask (`../../domain/forge/attention.md`, 4) and
    the control draws off, with no write: the build asks to wake at
    `until`. `Domain::away_on` holds the rule, the row and the wall.
  - *Its toasts,* Needs' toast slot (`needs-you.md`, 1, *What the client keeps*), on
    the request's answer: on, "Away until 08:00: agents decide within
    PROJECT’s limits." with the act "Limits", to Settings; "Away: agents
    decide within PROJECT’s limits." with `until` none; off, "Away is off:
    questions wait for you again."; refused, "Away didn’t change. Try again."
  - *On All projects* (`../../draft/surfaces.md`, section 2), while the page
    is one across projects (`pages::rail::across`), the switch and the chip
    are the person's projects' together: M, the projects `me` lists, and
    N, those whose row is on at the wall (`Domain::away_on`), counted with
    the earliest `until` of the rows on by one pure function,
    `pages::away_all::across`, which the rail and the chip read:
    - *Shown* while this section's *Shown* holds for any of them, the
      rail's fact on All projects (`../view.md`, 3).
    - *Its words:* with N 0, a project's off words, not pressed; with N
      M, a project's on words with that earliest `until`, "Away" alone
      where no row on has one, pressed; between, "Away on 2 of 4", N of
      M, not pressed, so that the state says what a press does. The build
      asks to wake at that earliest `until`.
    - *Its act,* `away::Act::ToggleAll`: with N M, every project off;
      otherwise every project on. Each project not already so is sent its
      own `SetAway`, the request *Its act* above sends for that project's
      switch, one decision each, sent one after another, each once the one
      before is answered: the client's exchanges in flight are bounded
      (jig's `protocol/client/README.md`, section 10), and may be fewer than the
      person's projects. One already so is sent nothing. While any is out
      or waits to go, every Away control is disabled and a project's act
      waits, as while a project's request is out.
    - *Its toast,* once every answer is in, says how many changed: "Away
      turned on for 3 projects: agents decide within their limits.";
      "Away turned off for 3 projects: questions wait for you again.";
      "1 project" for one; with some refused or unknown, "Away changed for
      2 of 3 projects. Try again.", of those sent; with none changed, "Away
      didn’t change. Try again." No "Limits": limits are a project's.
  - *On How work runs,* `settings.md`, 2's *What reaches you* draws the classes, what
    Away changes among them and "Away until". "Limits", on the toast and on
    Home, opens How work runs with that section's heading
    focused, so a browser brings it into view: `Address` holds no anchor, and
    the page takes the request from `Pages` once.
- **The approval page** (`../../draft/tui.md`, 4.2; jig's
  `protocol/client/sign-in.md`, *A sign-in approved by a browser*):
  where a person signed in to the browser approves a terminal's sign-in,
  in the browser alone. Its terminal's side is 7.1.
  - *A place of its own,* `Place::Approve { code }`, at `/terminal`
    (`../web/README.md`, 2.3), beside `Authorize`: no page's address, so the
    pages and the forge's side are left as at any place that isn't
    theirs (`../README.md`, 3.3), and the frame is the top bar alone (`../view.md`, 1). `code` is an
    `ApprovalCode` (jig's wire), or none. Its state is the root's, in a
    module of its own, `approve`, beside `authorize`: made when the place
    is gone to, dropped when it is left, and kept nowhere, the small
    store included. `Domain::approval()` gives it to the view, and the
    root's shown count moves with it.
  - *Opening it approves nothing.* With a code, the page reads the
    approval, `jig-client`'s `ReadApproval { code }`, once the link is
    live and an exchange is free, as the `Authorize` place sends its
    own; until then, and while it is read, it says "Reading CODE…".
  - *With no code,* or one the address carried that doesn't read
    (`../web/README.md`, 2.3), it asks for one: a field named "Code" and
    "Check". The field keeps what is typed up to jig's bound on a typed
    code (`CODE_TYPED_BYTES`, 32 bytes), and marks a field typed past
    it; "Check", or Enter, reads it as the address's code is read,
    `ApprovalCode::parse`: letters of the code's alphabet in either
    case, hyphens and spaces ignored. One that reads is read as above;
    one that doesn't, or a field typed past the bound, stays in the
    field with the alert below. A typed code changes no address. The page never
    lists the approvals that wait: the engine reads one by its code
    alone.
  - *Its states,* each in the column (`../view.md`, 1; `../web/look.md`, *Outside
    a project*), its level-1 heading, its words, then its acts, the
    first filled and the others outlined, CODE the code as shown
    (`WDJB-MJHT`), large, in mono, on the surface; "Check" filled beside
    the field, the field named "Code" above them; while deciding,
    "Approving…" or "Refusing…" stands in the acts' place:

    | State | Heading | The column | Its acts |
    |---|---|---|---|
    | asking for a code | "Sign in a terminal?" | "Type the code your terminal shows.", the field | "Check" |
    | a typed code that doesn't read | "Sign in a terminal?" | the same, the field as typed, and the alert "That isn't a terminal's code. Type the eight letters your terminal shows." | "Check" |
    | reading | "Sign in a terminal?" | "Reading CODE…" | none |
    | the ask, the approval waiting | "Sign in a terminal?" | CODE, large; "Asked AGO" (`../view.md`, 6's *ago*, from the read's wall time); "Approve only a code your own terminal shows." | "Approve", "Refuse" |
    | deciding | "Sign in a terminal?" | the ask's, "Approving…" or "Refusing…" in its buttons' place | none |
    | approved, waiting for the terminal's take | "Approved" | CODE; "Waiting for your terminal to sign in…" | none |
    | signed in, taken | "Terminal signed in" | "Signed in. You can close this tab." | none |
    | refused | "Sign-in refused" | "This terminal's sign-in was refused. Nothing was signed in." | none |
    | expired | "Code expired" | "This code has expired. Ask again in your terminal." | none |
    | used | "Code already used" | "This code was already approved or refused. If your terminal isn't signed in, ask again there." | none |
    | unknown | "Unknown code" | "temper doesn't know this code. Check the code your terminal shows, or ask again there." | "Type a code" |
    | signed out | "Sign in first" | "This browser isn't signed in to temper. Open the sign-in link temper printed first, then open this page's address again: your terminal shows it." | none |
    | signed out, Google's way given (the view's `Ways`, `google` and an origin) | "Sign in first" | "This browser isn't signed in to temper. Sign in, and this page comes back with your terminal's code." | "Continue with Google", to `ORIGIN/sign-in/google?then=PATH`, PATH the approval's printed address with its code, `/terminal?code=CODE` (`/terminal` with none), percent-encoded as the sign-in page's (`../../protocol/sign-in.md`, section 7) |
    | busy, not ready or lost | "temper couldn't answer just now" | "It is starting or stopping, busy, or out of reach. Try again in a moment." | "Try again" |

  - *Deciding:* "Approve" and "Refuse" each send `DecideApproval {
    code, approve }` once, when the link is live and an exchange is
    free, and the page is deciding until it is answered: approved,
    refused, used, expired or unknown as the table says. Nothing sends
    a decision by itself, and none is sent again.
  - *Approved:* the page reads the approval again each second
    (`approve::AGAIN`) while it stays approved, from the answer's
    turn, to learn the terminal's take: taken is signed in, and the
    reading ends. The approval's own expiry ends it otherwise, answered
    expired (jig's `protocol/client/sign-in.md`, *Never taken,
    nothing made*), so a page left open reads at most for the
    approval's ten minutes. A read answered busy, not ready or lost
    meanwhile keeps the page approved and reads again a second later.
  - *Ended:* signed in, refused, expired, used and unknown read no more.
    "Type a code" returns to the field, empty, its code dropped.
  - *Signed out:* the link signed out, before a read or after, or a read
    or a decision answered signed out (which signs the client out,
    jig's `client/domain/client/README.md`, section 9), draws "Sign in first"
    in place of the pages' signed-out paragraph and sign-in page (`../view.md`, 1).
    An answer of not a browser draws it too: the page's exchanges carry
    the cookie (`../web/README.md`, 2.4), so the answer says they reached the
    engine without it, and the remedy is the same. The printed link's
    sign-in lands on `/welcome` (`../web/README.md`, 2.4), so the person comes
    back by the address their terminal shows; a Google deployment prints
    no link, and its "Continue with Google" returns to the approval with
    its code, so a person signing a terminal in there is never stuck
    (the look request `outside-pages`). The page's codes, acts and
    `then` are the TUI's too (`../README.md`, 4.1): only its states' look and words are
    the browser's.
  - *Busy:* a read or a decision answered busy, not ready or lost.
    "Try again" reads the approval again, from its code, so that a
    decision lost on its way back shows as it stands.

## 3. Open questions

- **The two-step Home and night look after landing** (`README.md`, `../view.md`, 3): temper/ui
  compares `ask-temper-start`, `to-begin-two-steps` and `night-mode` with
  main at 1440, 1000 and 420 px, with menus and disclosures open and
  each state their messages name (`../web/look-requests.md`, 4).
  Include both steps left, each alone, done and skipped, Home with a need
  and a chat left out while either step remains, and their return after
  the last step or a skip. Check the restricted navigation on Home
  and Ask temper, its three tabs on a phone and composer above them.
  Check the night switch at every width, in the rail folded beside a
  Split, open and closed, and at a phone's chat bar, reload, the three
  night pages, their shadows, and readability
  elsewhere. The night palette is the user's first pass
  to refine; system preference and the placeholder's night look remain
  `../web/README.md`, section 3's choices. Default: focused tables, gallery
  states and replayed prints establish no rendered equality or browser
  reload result.

- **Account email after landing** (`README.md`, `accounts.md`, 2; the request
  `ready-account-email`): temper/ui checks a real ChatGPT sign-in and a
  real Claude Code login on Ready, the Model step, the model-check pane,
  the picker and Accounts, including the terminal's words. The check
  establishes whether the real provider's claims yield an email and
  compares the rendered look. Default: focused view and integration
  checks with fake verified claims and program statuses; browser and
  process stories and the real-login render check follow landing.

- **The connect-flows comparison** (`README.md`, `../forge/family.md`, 4, `../view.md`, 2): temper/ui compares
  request pages 01–23 with main, including named states and open menus
  and disclosures at 1440, 1000 and 420 px, with screenshots also at
  1280 px (`../web/look-requests.md`, 4). Default: the source and print
  account is distinct from rendered equality; differences return as
  findings after landing. Compare the request-owned parts, including
  Settings' select chevrons, the sheet's chooser, captions, list and empty
  search, More's summary, connection actions and each model way's path;
  the shared H1 and open project menu row keep their frame's design.
- **The trial's later choices** (`welcome.md`, 1, *The try*, `accounts.md`, 2): whether Settings
  offers "Try it", whether deployment accounts are tried, how a second
  account is tested while selection stays a new chat's, and retaining a
  failed line across a restart are `../../domain/trial.md`, section 8.
  Its project-only key limit stands there too. Default: the account that
  a new chat selects, personal accounts alone, and no retained line or
  additional Settings act.

- **The rail, Work and Deployment after landing** (`work.md`, 1, *Work*;
  `settings.md`, 1; `../view.md`, 3): temper/ui compares main with the whole
  story's mockups, `web/whole-story/work.html`, `pair/work.html`,
  `team/work.html` and each page's `nav.rail`, at 1440, 1000 and 420 px, by
  day and by night, with the new box's menu, the switcher's All projects
  and the person's menu open, Work under each filter and Deployment's six
  tabs; and walks them on a box (`../../draft/whole-story.execution.md`,
  section 2). Owner: temper/ui. Default: each difference a finding filed
  back to the plan's repair.
- **Home and Needs you across projects after landing** (`home.md`, 1, *Home*,
  *Home on All projects*; `needs-you.md`, 1): temper/ui compares main with
  the whole story's `web/whole-story/home.html`, `needs.html`,
  `phone-home.html` and `phone-need.html`, and `pair/`'s and `team/`'s
  Home and Needs you, at 1440, 1000 and 390 px, by day and by night, with
  Only you's region against `needs.html`'s through `../web/gallery.md`, 2's
  `needs-only-you` and `night-needs-only-you`, and Accounts'
  `accounts-expiring`; walks them on a box across two projects, a
  sign-in signed out reaching Accounts from Only you and renewed there;
  and records the journey of Home to a
  need, answered (`../../draft/surfaces.md`, 14). Owner: temper/ui. Default:
  each difference a finding filed back to the plan's repair.
- **Away on All projects after landing** (2, *Away*, *On All
  projects*): no mockup draws "Away on 2 of 4", so temper/ui renders
  `../web/gallery.md`, 2's `all-home-away` and `all-home-away-on` at 1440, 1000
  and 390 px, by day and by night, the words in the switch and the chip
  where the whole story's `web/whole-story/home.html` and
  `phone-home.html` draw them, and walks a press on a box across two
  projects. Owner: temper/ui. Default: each difference a
  finding filed back as a repair.
- **The phone after landing** (`needs-you.md`, 1, *On a phone*; `ask-temper.md`, 1, *Ask
  temper's page*; `../view.md`, 3, *On a phone*): temper/ui compares main with the
  whole story's `web/whole-story/phone-home.html` and `phone-need.html`,
  and the Ask tab with `phone-tell.html`'s top bar and tabs, at 390 px, by
  day and by night, through `../web/gallery.md`, 2's `all-home`, `needs-list`
  and `ask-chat`; presses each target of the top bar, the tab bar, Home,
  Needs you, Work and a chat's bar and composer, measuring its 44 px; and
  walks a need answered from Home and back on a phone-sized page. Owner:
  temper/ui. Default: each difference a finding filed back as a repair.
- **Home's and Needs you's later parts** (`home.md`, 1, *Home on All projects* and
  *A page's parts*; `needs-you.md`, 1; `../../draft/surfaces.md`, sections 3, 4,
  12 and 15): a project's Now on its row, rolled from its owners', its
  tiers, a follow's parity and "owned" (R1 to R3, F1); who decided, on a
  Decided for you row, and the intro's What
  reaches you; on an Only you item, what waits for it ("Two agents on vsr
  wait for this login.", `../../draft/what-reaches-you.md`, section 8), once
  jig's `accounts` document names the tasks waiting on each of a person's
  own accounts, as the `deployment` document does the deployment's (jig's
  `client/domain/contract/documents.md`, 1); Only you's other kinds, a forge's
  token to replace, and a person task asking an act rather than a choice,
  which stays among the cards meanwhile; Only you in the terminal's inbox
  (`../tui/pages/README.md`); a need's goal, role and asker on its from
  line; one brief across projects written per person,
  `../../draft/surfaces.md`, 15's open question; and more than 16 projects
  read. Each joins *A page's parts* with the read that gives it. Default:
  none is drawn until it is built, the brief is one a project, joined, and
  the first 16 projects in `me`'s order are read.
- **What a real box's walk has not shown** (`README.md`; `../../protocol/setup/owned.md`, 6; the
  requests `ui-final-walk` of 2026-11-05 and `ui-walk-3` of 2026-11-06): the pinned
  Claude Code fetch, from a box with directories of its own up to the sign-in prompt
  and reused on a second run, and a real browser's Sign out ran by hand, on fresh
  loopback deployments, and no test keeps them. Not shown: a provider's
  sign-in or a paid call; TLS, a proxy or a browser on another machine; Accounts
  with a project or an account on it; the small engine profile whole, whose client
  limits alone ran; another platform's build, a failed or corrupt download, two
  fetches at once; a copy from a public host. Default: none, until a plan walks them.
- **Pages the mockups do not show:** a project's page with its
  resources, findings, notes and settings (policy, people and roles,
  spend; jig's `client/domain/pages.md`, section 6, and the forge's
  parts in `../forge/repositories.md`, 3, and `../forge/README.md`, 6.3), but the settings' Delete (`README.md`); the deployment's page but its LLM
  accounts, which the accounts page shows an owner, and its forges,
  which the forges' page shows (`README.md`); the "Tasks" list of a project,
  which the mockups draw, after jig's pages (`jig/docs/design/client/domain/pages.md`, 6.3
  and 7); of the deployment's hosts, which the mockups draw as "Workers",
  all but what Settings' Builders tab reads (`README.md`): its capacity and its runs;
  a procedure's task beyond `task.md`, 3's page, and a person task beyond a
  tracked task's header and plan; making a service and showing its
  credential once; another party's kept work. Default: not in temper's
  client; the person acts on them through the engine's configuration,
  or not at all.
- **The Away switch and quiet hours** (2, *Away*): "08:00" is the mock's
  end of its quiet hours, a fixed hour here. Quiet hours and notifications
  are later parts, and a setting of the hour would come with them. The
  terminal reads no zone (`../view.md`, 6), so its 08:00 is universal time's.
- **The playground's and the agent's placeholders** ("Try a suggested
  request…"; "Write to T12…", the
  agent's task's composer, `task.md`, 4): skein-ui's text area takes a
  placeholder, which a chat's composer and `NewChat`'s, "What do you want
  done?" in every project, the playground's start among them, draw
  (`README.md`). Default: neither drawn, the agent's waiting for its page's
  owner.

- **A page refused busy.** How a page says its rows have not arrived,
  rather than calling them absent, belongs to the client's presentation
  (`../README.md`), outside watch sizing. Default: keep the handle's
  behind state and its retry (jig's `client/domain/client/watches.md`, 1).
