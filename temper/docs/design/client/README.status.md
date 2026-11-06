# README: status

2026-11-06. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 3.5, *the turn being written*, *While it works*: the turn being written is read as closed text under "Writing...", with no "▍".
- **Design departs:** 3.5, *Starting a goal from a chat's document*, and 5.14, from
  the request `ui-chat-draft` of 2026-11-05, decided by the answers that day to its
  tasks `goal-sheet-state` and `goal-sheet`: one sheet, `ui-goal-page`'s, where the
  request draws a chat's own. So the sheet has no Budget field, its budget the
  creating bound's. It has no "Plan, then wait for me" of stock template 2:
  `shapes::set_agreed_goal_from` and `sheet::WayKind::Agreed` stand built, and nothing
  sends them, `sheet::snapshot`
  taking the task-by-task words alone. Its ways are `goals::ways`', the project's
  templates by number with their own names and lines, where the request has three in
  its own order and words. It has "When it needs you", which the request leaves out
  until the away switch exists, as it now does. Its words close with `Draft:` and
  `Chat:`, which the request doesn't name and the goal's page reads for its link back.
  A proposed goal is 5.14's toast, where the request has a line above the composer:
  `Chat.proposed` is set (`temper-client-domain`'s `pages/start_goal.rs`, `told`) and
  read by nothing. No plan holds these; section 11 keeps them.
- **Design departs:** 3.5, *A chat's document*, from that request's decision 7,
  decided by the answer that day to `goal-sheet-state`: `drafts_offered` reads kind
  7's `drafts`, false in the stock (5.13), where the request has it true while `me`
  lists a tracked template with no conversation flag. So a fresh project offers no "+
  Draft", Drafts filter, "Start goal…" or rail's "Goal" until "Chats can write drafts"
  is turned on, as the journey `chat-draft` does first.
- **Departs:** 5.14: a party whose `me` gives the project no creating bound still
  sees a chat's "Start goal…" (`temper-client-view`'s `pages/chat_sheet.rs`, `bar`),
  and a press opens nothing (`pages/start_goal.rs`, `open`;
  `no_creating_or_empty_document_opens_no_sheet`), where the section has `NotOffered`
  said in jig's words. No reason is recorded.
- **Departs:** 3.5, *The Chats page*: the document read stops at
  `limits.client.domain.rows` notes, so a chat beyond those notes has no
  "writes" line and is absent under Drafts. Goal chips cover only the newest
  `rows` chats across the live and closed lists, with ties by the lower chat
  number; a row past that window has no "started goal" line. The answer of
  2026-11-05 to `ui-chat-draft/chats-drafts` keeps this one chips window,
  without raising jig's bound.
- **Design departs:** 5.3, from the greenfield target, by the request
  `ui-rename-focus` of 2026-11-06: closing the rename's form leaves the project
  menu open with the focus on its summary, where the mock draws the menu closed,
  since a disclosure's fold is skein's page part's (`ui-web.md`, 7). The field does
  not select its name on opening, and no toast says "Renamed": the request names
  neither.
- **Design departs:** 3.5, 5.3, 5.8, 5.13, from the greenfield target:
  the switcher keeps its inline new-project form; a chat keeps its status,
  Stop and Close, and offers Change model only with supplied choices;
  a goal uses supplied budget units, not dollars, and offers neither the
  mock's budget field nor its earlier reports. Settings adds Configure
  temper and keeps unsupported status, funds and notification controls
  disabled with their explanation. The welcome's Claude card says
  "a config of its own" rather than the mock's "an environment of its
  own" (3.5). Home and Changes omit the mock's "main green at" time:
  their landing panel supplies a red flag, not that timestamp. These
  sections retain the client's
  available acts and supplied values rather than the mock's local-only
  operations. Their extra rows and controls change the pages' lengths at
  laptop and phone widths; they are not stylesheet differences.
- **Departs:** 5.1: at 760 px and under, `temper.css` hides the accounts' and the
  playground's banners on every chat's page and `NewChat`
  (`#app:has(> .temper-session) > .temper-accounts`), where the section draws
  them there, because the request `ui-phone` of 2026-11-05 has those pages whole
  at `100dvh`, which a banner above them would cut. With no usable account "No
  model yet · Connect a model" still stands above the composer, but for a
  configuring chat (`temper-client-view`'s `pages/chat.rs`, `main`), where nothing
  then says why Send stays grey; and no line says there that the playground is
  running or leaving. Seen by reading; no test holds it.
- **Not yet:** 4.7: the form draws no read-only line on an adopted repository (`temper-client-view/src/forge/page.rs`).
- **Not yet:** 3.5, *Home on any project*: on the stock policy the status agent
  is left no funding. Its roles' pools are as large as the project's period
  (`temper-service`'s `stock.rs`, `project_policy`), so once a person's pool has
  opened in a period, nothing of it is left: `jig-domain-core`'s
  `connector_repair_authority` gives no seed, the run is refused
  (`../domain/forge.md`, 5.7) and Home keeps the computed sentence, with no lede
  or byline, at each sitting of that period. The request `ui-home-status` of 2026-11-05 has the
  agent always run for now. Both the root's status world and the fake box's walk
  shrink the owner's pool to leave funding, the box's by `project fund`
  (`../domain/forge.md`, 20, holds the question).
- **Not yet:** 10: the walk that request asks of a box: land a change, come back
  past 30 minutes, see the count, the agent's words and the byline. A real box
  has no fake clock, and the 31 minutes do not fit its tier, so
  `temper/crates/temper/tests/fake_backends/walk/status.rs` stops inside a
  sitting: the first look, then the count across a client's and a box's restart,
  with the byline asserted absent and the charter's scripted words read from the
  model's document alone. Walked by hand on a fake box, its owner's pool as the
  stock leaves it, at 1440 and 420 (the request `ui-final-walk` of 2026-11-05): the
  landing stands in Changes, and past 31 minutes Home says "you last looked 31
  minutes ago" over the computed sentence, with no brief and nothing saying why
  (section 11, *A brief refused, on Home*). The period's 8,117,760 stood all
  reserved, so the cause above is read from those balances, not proved. Each half is
  shown apart: a new sitting on the client world's fake clock
  (`temper/tests/client/tests/suite/status.rs`), the agent's brief in the root's
  document (`temper/tests/engine/domain/tests/suite/forge_routes/status.rs`), and
  the lede and byline from a hand-made document (`temper-client-view`'s
  `tests/home.rs`, the gallery's `home-morning`); no test draws a brief an agent
  wrote.
- **Not yet:** 10: a walk from the sheet through planning, a live report and
  delegate tasks, Pause, Resume, a message and Stop, which the request
  `ui-goal-page` of 2026-11-05 asks of the browser tier and of a box. The
  browser tier walks Goals' link to `/chats` and back alone
  (`start_goal_opens_chats_for_a_draft`), and the fake box's walk reaches an
  ended goal's report from its Goals row, in the terminal's client
  (`a_chats_goal_on_the_fake_box_shows_its_report_in_the_goals_list`). The fake
  box's model (`temper-fake/src/backends.rs`, `model_document`) writes a report
  only for the first chat's goal, task 2, and has it ask, wait or finish with no
  delegates, so it can show no planning and no delegate task: the journey
  `chat-draft` walks a chat's sheet to that goal's page and back to the chat alone.
  The request `ui-final-walk` of 2026-11-05 walked a live report, Pause, Resume and
  Stop there by hand, which no test keeps. That model has no script of a configuring
  agent's document or proposal and no hold of temper's own, so the gallery alone
  draws those states.
- **Not yet:** 3.5, *Needs you*, 5.13: a proposal of settings has its inbox row say
  project 0, so *Needs you* by the open project and the rail's count
  (`temper-client-domain`'s `pages/needs.rs`, `count`) leave it out, whichever
  project's kinds its items change, where the greenfield mock, the target, has its
  card and its count. No live walk shows that card: the gallery's
  `configure-chat-proposal` draws it in a chat, and `needs-proposal` is a goal's.
  Section 11 holds the decision.
- **Not yet:** 3.5, *Claude*, 5.5: the command is the same words whatever the box's
  directories (`temper-client-view`'s `pages/connect_model.rs`, `command`), so on a
  box given its own it refuses, where the answer of 2026-11-05 to the `ui-final-walk`
  task `fix-flow` has it run as shown. The pinned fetch has run by hand only with the
  three directories exported, to Claude Code's first sign-in prompt. Section 11
  holds it, because it needs a fact no client is given.
- **Not yet:** 3.5, *What the client keeps*, its held send: once, on a fake box at
  1440, a choice pressed just after Undo left the card, the count and the engine's
  question as they were after six seconds, and a repeat on a fresh box and the walk
  at 420 answered. The cause is unknown; no test holds it (section 11).
- **Not yet:** 5.3, from the answer of 2026-11-06 to the `ui-final-walk` task `fix-look`: in
  the rail the rename's form stands after the rows, the row kept, with a secondary
  "Save", a visible "Cancel" and a field 25 pixels high (`temper-client-view`'s
  `frame.rs`, `rename_form`), and closing it gives no focus back
  (`temper-client-domain`'s `pages/switcher.rs`, `close`): the walk at both widths
  found Escape on the new project's field and "Cancel" on main. That plan fixed the
  answer's ring alone and left the rest for its size; section 11 holds it.
- **Not yet:** 3.5, *Away*, 4.6: with the `away` watch refused, ended or not
  delivered yet the switch draws off, but its act is still offered, where 4.6
  has it not offered: `temper-client-view`'s `frame.rs`, `away`, and
  `pages/settings.rs`, `away_settings`, draw their controls enabled, and
  `temper-client-domain`'s `pages/away.rs`, `act`, then sends nothing and says
  nothing, so a press does nothing. Seen by reading; no test holds it and no
  reason is recorded.
- **Not yet:** 10: the renders the request `ui-away` of 2026-11-05 asks,
  `home.html`'s away state beside the client's at 1440, 1000 and 420 wide, the
  switch on and off at each and toasts open. They are saved on a builder, outside
  the repository, so what they show is unknown here; the gallery holds their
  states (`browser.md`, 7.2).
- **Departs:** 3.5, *Away*, its *Overturn*: an `Overturn` whose outcome is no
  answer, unknown after a restart or its link lost, toasts "That was overturned
  already." and sends no message (`temper-client-domain`'s `forge/decided.rs`,
  `overturned`), where the section has those words for a refusal: if it
  committed, the row stands marked and its task has heard nothing. A message that
  fails for any reason, a lost link's too, toasts "It has ended; a new goal can
  undo it." (`pages/away.rs`, `overturn_told`), where the section has them for a
  task that ended: a live task then hears nothing, and its marked row says
  "Overturned: it asks you again" and offers no second Overturn. Seen by reading;
  no test holds either and no reason is recorded. The message's question is cut
  at a character boundary to fit 512 bytes (`forge/decided.rs`, `overturn_words`),
  where the section has it whole, because the smallest deployed composer admits
  that (5.4).
- **Departs:** 3.5, *Goals*, 5.14: Goals' "Start one from
  a draft" is a link to `Chats { filter: All }`, `/chats`
  (`temper-client-view`'s `pages/goals.rs`), where those sections have it open
  the sheet with its Draft field. A chat's "Start goal…" opens the shared
  sheet with its document's snapshot.
  `temper-client-domain`'s `pages/start_goal.rs`, `open`, ignores one without
  both a draft and a chat, so "Start a goal" and the Draft field are drawn by
  nothing. The rulings of 2026-11-05 on that request (the answers to its tasks
  `start-domain`, `start-sheet` and `goal-browser`) keep no typed draft path: the
  link goes to Chats' Drafts filter and the sheet opens from a chat. The entry
  still goes to All, where the rulings name Drafts, the filter being there only
  while `drafts_offered`.
- **Design departs:** 5.8, from the request `ui-goal-page` of 2026-11-05, decided
  by its plan's cut and its proof. A goal's bar is `figure.bar` in the browser
  and `Role::Image` in the terminal, where the request has `div.bar` and
  `Role::Image` for both: skein-ui's `Role::Image` is an image, with no children.
  The planning dots are spans in a `Figure`, not `.typing`'s `<i>`s. A live goal
  with no tasks yet headlines as planning, where the request's words give "0 of 0
  tasks landed." A dependency outside the goal's rows says "waits on task N",
  where the request has its title alone, and the row has none. A goal's counts
  search at most 8 pages for its own line and are then what the page holds, where
  the request has them true past one page (section 11, *A goal's own line*).
- **Not yet:** 3.4, 5.5: the composer's
  supplied default, jig's
  `resolved_default`, which neither jig's `Me` nor temper carries. 3.5:
  `Board { project }` with its reordering: no
  address in `temper-client-domain`'s `address.rs`.
  5.13: a person with a project's role alone can read kinds 6 and 7,
  but cannot change them on the tab: jig's settings inventory requires
  a deployment role (`jig-domain-core`'s `documents/settings.rs`, `make`),
  so `temper-client-domain`'s `forge/ways.rs`, `act`, has no newest
  revision and sends nothing. The controls appear enabled and give no
  refusal, where the request of 2026-11-05 has a person who is no owner refused
  in jig's words, as one with a deployment role is. With no document to say a
  setting changed, that person reads them again only as the project opens or
  the link comes back. A project-scoped settings inventory needs a contract in
  jig before the tab can use it.
  5.13: Home draws neither "Turn on drafts and goals" nor "See the settings", so
  nothing opens the tab by them.
  5.3: the banner's "Changes", on a page that is no project's:
  the request of 2026-11-05 asked for the rail, and its mock has no banner.
  3.5, *Goals*: a project whose `me` lists no goal template opens no watch
  (`temper-client-domain`'s `pages/goals.rs`, `query`), so its page draws the
  kicker and the lede alone, with no heading and no filters, where the section
  says "No goals yet."; a project with no tracked template has a lede that
  stops at "landed work.", which the section has no words for.
  3.5, 4.6, 8: Forget, on the Forges page and as the FORGE
  request `Forget`: no `forget` in `temper-client-protocol-wire`'s schema
  or in `temper-service`'s routing. 3.5: Ready's model line and the Model step's
  status name no email for a ChatGPT sign-in, whose account's display is empty, since
  the engine keeps no email: it waits on
  `jig/docs/design/draft/account-facts.md`, 6, its second step.
  3.5, *Ready*: two endings leave nothing under "Model none yet"
  (`temper-client-view`'s `pages/connect_model.rs`, `welcome_end`), and no
  test holds either: a sign-in `Made` or `Renewed`, not declined, with no
  account usable, since 5.5 has no words for it and no walk has ended so
  (section 11); and any ending where `me` offers no ChatGPT plan's sign-in,
  since the function finds that way before it draws the ending's words.
  4.4: the picker, which the adoption page does not draw
  (`temper-client-view`'s `forge/page.rs`), so Home's GitHub step's
  "Adopt" opens the form with no picker, and its line counts no
  repositories, where the mock's does.
  5.5: a plan's first use, its card in the conversation. 3.5, 3.6, 10:
  Delete on a project's settings, confirmed by its name, and jig's
  `RemoveProject` by `shapes::remove_project`, with the chat
  template's grace: no such act in `temper-client-domain`. 3.5, The welcome:
  for a person whose role does not hold the deployment's forges, the model
  alone deciding;
  `temper-client-domain`'s `pages/welcome.rs` `decision` reads a
  refused `forges` watch as no forge, so a usable model opens Forge.
  FORGE in "FORGE · not
  finished" as the forge's name; `forge/connect.rs` `unfinished` prints
  "FORGE" itself. "Go on" after a restart: the row's `app` is the forge
  domain's memory alone (`temper-engine-domain-forge`'s `domain.rs`
  `apps`), so a GitHub forge found unfinished after a restart offers
  "Start over" alone.
  3.5, With no project
  yet, where the document shows the welcome: its heading, words and acts; `pages/chats.rs` draws its page
  alone. There too, with a welcome or without one, `NewChat` at its address
  outside the playground's start: `temper-client-view`'s `pages/new_chat.rs`
  draws its layout where the section has it give way, its line calling the
  missing project "Project".
  4.7: the "not installed" outcome, "temper's App is not installed on
  OWNER.", with "Install on another account": `temper-service/src/family.rs`
  maps `forge::AdoptionRefusal::NotInstalled` to `ForgeReason::NoAccess`;
  the client wire has no not-installed reason, pinned as a known gap by
  `temper-service/src/tests/setup_github.rs`,
  `unaccepted_and_other_account_adoptions_keep_the_wire_reason_gap`;
  no reason is recorded.
- **Departs:** 3.5, Settings: "Configure temper" chooses among the rows
  `temper-client-domain`'s `Chats::open_configuring` is delivered, whose
  live watch holds a window of `limits.client.domain.rows` by latest
  activity, so a configuring chat not closed but past that window isn't
  seen and the link opens `Configure { project }`; `configure-temper-name`'s
  landing accepted it, raising no bound for one link.
  3.5, With no project yet, and 5.3: where the document
  shows the welcome, with no project, the banner's place for it reads "No project yet" followed by the new
  project's form, "New project" and "Create" (`temper-client-view`'s
  `frame.rs`, `switcher`), where 5.3 ends the switcher's list with that
  form and 3.5 leaves the first project to the playground's start or
  the welcome's Ready. The code records no reason; it lets a person make
  the project Ready would offer before Ready, the case of the draft's open question
  (`../draft/first-run-projects.status.md`).
  3.5, 4.6, Changes: the queue's first says "checking" at whichever step of
  its turn, and "checks started" gives when that step began
  (`temper-engine-domain-forge`'s `views/changes.rs`, `Change.since`), where
  the request of 2026-11-05 has when its checks began: nothing keeps that time.
  4.6: `changes_view` reads every `ChangeRow`'s repository with `?`, and no
  step erases a closed change's row (`../domain/after-landing.status.md`), so
  once a forge's removal erases a repository with a closed change, no project
  has the document; seen by reading, no test holds it and no reason is recorded.
  3.5, *Goals*: `goals::row` is `pub(crate)` (`temper-client-view`'s
  `pages/goals.rs`), where this section has it public, because its `Bindings`
  is the view's own and Home reads it in the same crate.
  3.5, *Needs you*: a filter's `kind` and `goal` narrow the open cards alone:
  `temper-client-view`'s `pages/needs.rs`, `main`, asks `Session::answered` by
  project, and an `Answered` keeps neither, so under such a filter the answered
  lines and the head's count hold answers the filter leaves out; no reason is
  recorded.
  3.5, *A new chat* and *A project with no repository*: `NewChat`'s line says
  the project has no repository for every `Repository` but `In`
  (`temper-client-view`'s `pages/new_chat.rs`, `main`), so also with the
  `forges` watch refused, ended or not delivered yet, where the section has
  nothing said. The request of 2026-11-05 gives that line two cases and no
  words for not knowing; no test holds it and no reason is recorded.
  3.5, *The rows*: an account's sources are a list under their level-3
  heading, each with its state and acts, and a sign-in under way a paragraph
  (`temper-client-view`'s `pages/accounts/rows.rs`, `draw`), where the section
  has a row's facts as `.setting` rows, because the rows keep their sources,
  observations and forms in document order.
- **Design departs:** 3.5, *Away*, 5.8, 5.12 and 5.13, from the request `ui-away`
  of 2026-11-05, each decided by its plan's cut, for the user to see:
  - 5.12: every clock in the browser reads in the person's zone, where the request
    asks it of Away's 08:00, because `TimeForm::Clock` is one form and read
    universal it would say "Away until 06:00". The offset is read once, at start,
    so a change of daylight saving shows at the next load, and
    `forge::away::next_eight` takes that one offset, so the night the clocks change
    its 08:00 can be an hour off. The terminal stays universal (section 11).
  - 3.5: the chip is a second button in the rail, the phone's top bar, where the
    request has it in the banner: a project's page has no banner (5.3). The
    terminal's is the rail's button, where the request has its `navigation`, which
    a project's page does not draw.
  - 3.5: "Limits" focuses the section's heading, by a count in `Pages`, and a
    browser brings it into view. `#away` is no anchor: `Address` holds none, so no
    address reaches the section.
  - 3.5: the toast names the project, where the request's words have "fabrig’s".
    "Away: agents decide within PROJECT’s limits.", "Away didn’t change. Try
    again." and "That was overturned already." are words the request does not name.
  - 3.5: a row whose task has ended says "landed…" or "ended…", overturned or not,
    its took still struck, where the request's list has "Overturned: it asks you
    again" first: a refused message leaves the row marked, and its task won't ask.
  - 3.5: Home's away form is not drawn while To begin shows, where the request has
    it with any row showing.
  - 5.8: "follows the away switch" stands on every goal whose words hold no line
    `Asks: always`, where the request has it where its requester has a switch: no
    read says that, and a requester with none has its goal told to ask
    (`../domain/forge.md`, 5.9) under those words.
  - 5.13: the tab's "Away switch" row says "Hidden" with kind 7's `away` false, and
    a project with a goal, or the party's row on, shows the switch all the same,
    since the request has any of the three show it.
- **Design departs:** 3.5, *A chat's conversation*, its cards, from the request
  `ui-chats` of 2026-11-05, which names a proposal, a held task and a question's
  answer as cards in the conversation: a chat's page holds proposals alone
  (`temper-client-domain`'s `pages/chat.rs`, `cards`), since its held delegate
  is its one line (5.7), and the answer the one line under "You" the same
  section keeps. No reason is recorded.
- **Design departs:** 5.9, and `browser.md`, 2.5, from the request `ui-chats` of
  2026-11-05, decided by its plan's cut: the request retires `look::CHAT` and
  `CHATBAR`, and they stand renamed `AGENT_PAGE` and `TASKBAR`, with `ENTRY` and
  `COMPOSER_AREA` kept, because an agent's task's page still draws them and the
  request keeps to chats' pages.
- **Design departs:** 3.5, *Needs you*, *What the client keeps*, and 5.3, from
  the request of 2026-11-05 for them, decided by its plan's cut, the request
  leaving that judgement to it. The rail's watch opens while a project's frame shows, not once signed
  in: the rail alone reads it, and opened at sign-in it would stale all three
  journeys' recordings. Bare `/inbox` is `/`, the Inbox's unfiltered address
  (`browser.md`, 2.3). The dots are a figure, not `role=img`: skein-ui's
  `Role::Image` is an image, with no children. Money reads "S of L used", not "$S
  of $L": the contract's units aren't dollars, and the request also asks the task
  page's form, which has none. Leave held takes its reason in a disclosure, as
  Reject does, where the request names no text area: `Decision::LeftHeld` needs
  one. A `Choice:` line past `CUE_BYTES` is no choice (`../domain/agent.md`, 8.2),
  where the request counts every such line, since its button could send nothing.
  And " · later" shows on *Needs you*'s mono line as on Home's, where the request
  and the mock have it on Home's card alone: the one function draws both, and no
  reason is recorded.
- **Design departs:** 3.5, *The flows' paths* and *Builders*, from the request
  `ui-settings-tabs` of 2026-11-05, decided by its plan's cut. The terminal
  draws no path, where the request has each flow add one: the same request
  keeps the terminal's prints' meaning, a path would shift every number on the
  flows, and the terminal's rail holds "Settings". With hosts kept and none
  connected, `.empty` stands above their "Not connected" lines, where the
  request gives none connected `.empty` and no words for that case, so that no
  state is dropped.
- **Design departs:** 3.5, *Goals*, from the request of 2026-11-05: "6 / 9" is
  on the task line (jig's `client/domain/contract.md`, 5.1), where the request
  offered the goal's document or the tree's read, and a row reads its report
  alone, not also its tasks by `parent`: each place needed the same stored
  counter, and the line needs no read per row. The lede's ways run in template
  number order, as the request's words have it, so the reference's read "task
  by task, plan, tasks, land whole", where its mock, the target, draws them the
  other way round: number order is what a project's settings decide.
- **Design departs:** 3.5, *Settings*, 5.3 and 5.13, from the request `ui-settings`
  of 2026-11-05, decided by its plan's cut. "Sign out did not reach the engine."
  stands on the rail's foot as well as the `Accounts` tab, where the request moves
  it there: a person with a lost sign-out may be on any page (6.6). Kind 7's
  controls are disabled together while one is out, where the request disables the
  one pressed: they write one record, so two at once would lose one. And a
  proposal's card names kinds 6 and 7 as `temper ctl` names them for a person with
  no role in the deployment, where the request has their values for everyone: a
  proposal's items span scopes, so their read keeps that role
  (`../protocol/configuration.md`, 13.2).
- **Design departs:** 3.5, *The welcome* and *Home*, from the request `ui-welcome`
  of 2026-11-05, decided by its plan's cut. A plan's way at its address, with no
  sign-in asked, keeps its card in the panel, where the request draws a way's page
  for the key, the paste and Claude alone: that card is how the welcome reaches
  the paste. A repository Home's sheet asked leaves the step's lines once in and
  reads as any in the project, "ORG/NAME is in PROJECT" and "Chats can change its
  code now.", where the request has each copy ticked and then "Chats can change its
  code now…": section 11 holds it. The sheet alerts past 16 addresses, where the request
  names no bound: the wire and the store need one (`../protocol/setup.md`, 6.7).
