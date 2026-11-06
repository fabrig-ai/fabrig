# README: status

2026-11-06. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 3.5, The welcome's Model step, *The sign-in under way*, and `ConnectModel`, *While it waits* and
  *From another computer*: the card's line and the note say a new tab, and nothing opens one: a press still
  sends a `Return` and the page's own tab leaves, until `../draft/sign-in-new-tab.md` is built. So the note shows
  for a moment before the tab goes, and again on a page come back while the sign-in waits, and a browser on
  another computer presses, leaves, comes back by hand and finds the note's folded paste. The view has no fact of a tab refused:
  `sign_in_wait::tab_refused` is false, the one function that draft feeds. The new tab's two lines, its closing on
  a refusal at the press, and a reload's same page are that draft's. A page come back holds no address, so
  it draws no "Open it again". `Act::SignInElsewhere` and `ConnectModel::elsewhere` stay in the domain with no
  entry in the view, the paste's tests and gallery states reaching the page through them, until that draft
  makes every sign-in hold its page.
- **Design departs:** 3.5, *The sign-in under way* and `ConnectModel`, *While it waits*, from the request
  `sign-in-new-tab` of 2026-11-06, by its plan's cut and its account. The terminal draws the note's plain title,
  its muted line and "Open the sign-in page" above the folded paste, where the request has the terminal as today:
  one function draws the page for both shells. `ConnectModel`'s disclosure takes auto side margins, where the
  request's rule has none, so that it keeps its note's edges under that page's width. The note's "‹ Back" goes to
  Choose, where the request's page, made by hand from Choose's, goes to Forge.
- **Departs:** 3.5, `ConnectModel`, *The paste*: a paste answered *finishing* keeps the note's "Waiting for
  ChatGPT’s sign-in, in the new tab" and its link, and reads "Finishing your sign-in…" in the disclosure, where the
  welcome's note reads those words alone (*Finishing a paste*): `temper-client-view`'s `pages/connect_model.rs`,
  `body`, draws `sign_in_wait::waiting` while the sign-in runs, whatever its paste. No reason is recorded.
- **Not yet:** 3.5, *while it runs*, which the request `chat-reading-b` asks of a
  terminal and a browser both: no test takes the call under way from the engine, or
  draws it running in a browser or a real terminal. The client's world writes the
  run's `call` into the delivery the client hears, its joint world's decoder being
  unable to leave a call unanswered (`temper/tests/client/tests/suite/call_line.rs`,
  `outstanding`). Chromium's story opens a committed line alone, since the demo's fake
  LLM holds text pieces, not calls, and a held text ends a run
  (`temper/tests/client-browser/tests/browser_all/browser_call_line.rs`), and the real
  terminal's reads a committed line alone (`temper/tests/real-loop/tests/suite/terminal.rs`,
  `a_person_at_the_terminal_reads_a_run_of_calls`). A browser's running line is seen in
  the gallery's `chat-work-running`; a terminal's in the view's tests and that world.
- **Not yet:** 5.9, *The conversation*: no browser test draws an agent's running line
  or presses its "Latest". The gallery's `agent-work` draws both, and the domain's and
  the view's tests hold what a press reads and when it binds.
- **Departs:** 3.5, *while it runs*: a call under way with none ended draws a
  description too, its words twice ("Checking template 2", "checking template 2"),
  where the section has the line alone: `temper-client-view`'s `pages/chat_line.rs`,
  `Run::flush`, takes the description from any call under way, and the client's world
  (`an_agents_page_says_the_call_under_way`) holds it. No reason is recorded.
- **Departs:** 3.5, *a failed call* and *while it runs*: a run still out with a failed
  call keeps `temper-work-failed`, so its call under way stands in the signal's colour
  after the sheet's comma, and the line says no "1 failed" until the run ends, its row
  alone saying so (`pages/chat_line.rs`, `Run::flush`; the colour seen by reading the
  sheet). No reason is recorded.
- **Not yet:** 3.5, *the words being written* and *While it works*: no browser test
  draws the words being written, so the "▍" faint and in the line of the last word,
  and the place held as the turn commits, are seen by reading the sheet's
  `.temper-writing-end`. The gallery's `chat-work-writing` draws them and holds nothing
  of the mark. The request `chat-reading-a` has the user read them on a real chat.
- **Departs:** 3.5, *the words being written*. Before its first word the page and the
  dots ask only whether the text is empty (`temper-client-view`'s `pages/chat.rs`,
  `conversation_with_times`, and `pages/chats.rs`, `working`), where `read_growing`
  holds back a marker being written: a text so far of `##`, `**` or blank lines draws
  its message with nothing in it and no "▍", and the three dots go, until its next
  word. A closed fence's code keeps its last newline (`skein-ui`'s `markdown.rs`,
  `line`), and a `<pre>` shows it, so a text ending in one has its "▍" on a line of its
  own, and the block loses that line as the turn commits: the look request `chat-work`
  left this to `read_growing`, which sets the end inside the code, after that newline.
  A last block holding a run still open, or a last line of digits or of a marker alone,
  `read` shows as the characters it is once committed, so that block changes at the
  commit, where only the "▍" should go. No reason is recorded for any; seen by reading,
  and no test reads them.
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
- **Design departs:** 3.5, *A chat's conversation*, from the look request `chat-work` of
  2026-11-06. A failed call's "1 failed" shares the description with the run's time, both
  in the signal's colour, where the request adds ", 1 failed" alone in it, and a document's
  write ends a run (`../draft/chat-reading.status.md`, 2); "Older" and its rule stay at both
  conversations' heads, where the request removes them, as the one way to earlier lines
  (`../draft/chat-reading.status.md`, 5): each decided by that plan's cut. The words being
  written end in a faint, inline "▍" that does not blink, where the request's overlay draws
  a block 7 px by 15 that blinks: the plan `chat-reading-a` set the mark, this request's
  landing took it at a catch-up, and no reason is recorded. Its pages' line says "Read 7
  docs, checked settings 3×" of calls that start with `settings`, where a phrase follows the
  first call, so the gallery's reads "Checked settings 3×, read 7 docs"; and its 01 gives
  the call under way an outcome, which a client is not given.
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
- **Not yet:** 3.5, *The Forge step*, and 5.3, *Projects in the rail*, a repository's link, and Settings'
  Repositories' (the request `ui-repo-link`): a repository is plain words in all three.
- **Design departs:** 5.3, from the greenfield target, by the request `ui-repo-link` of 2026-11-06: the mock's
  switcher row is one button holding the name and the small words; here the small words are a sibling of the
  button, since a link can't sit inside one, and they are a link with its "↗" where the repository has an address.
- **Design departs:** 5.3, from the greenfield target, by the request
  `ui-rename-focus` of 2026-11-06: closing the rename's form leaves the project
  menu open with the focus on its summary, where the mock draws the menu closed,
  since a disclosure's fold is skein's page part's (`ui-web.md`, 7). The field does
  not select its name on opening, and no toast says "Renamed": the request names
  neither, and section 11 holds them. On a new chat or Configure the closed form
  leaves the focus on the page's own field, where the request has the opener or the
  summary, because that plan's cut keeps a page's own request. The field's hidden
  label says "Rename", where the mock's says "Project name": no one sees it, and
  that plan's cut took no new word for it.
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
  model yet · Connect a model" still stands above the composer; no line says there
  that the playground is running or leaving. Seen by reading; no test holds it.
- **Departs:** 3.6, *Closed ends on one line*: a requester-closed chat hides
  every "Decision made" activity line, where the section and the request
  `close-chat` of 2026-11-06 hide the closing's alone, because `Activity::Decided`
  carries only a revision and cannot identify the closing decision (section 11).
  This keeps the quiet end line, at the cost of hiding earlier decision markers too.
- **Design departs:** 3.6, *The sheet*, from the user's request of 2026-11-06 to land
  `close-chat`, which asks no change of behaviour: a Release pressed on an uncertain
  effect before either choice closes the sheet and sends nothing, by the section's
  rule, which that plan's cut wrote (`temper-client-domain`'s `acts.rs`, `send`;
  `a_press_that_sends_nothing_closes_the_sheet`). The view draws both choices alike,
  reading no `again`, so nothing says one was pressed. A Reject pressed with no reason
  keeps its sheet open and says nothing, where `close-chat` has a confirmation that
  fails say why above its acts: no ask went, and the reason's box is on the card behind
  the sheet.
- **Not yet:** 3.6, *The sheet*: that request has each confirmation walked. A browser
  on a fake box pressed Close, a goal's Stop and a Release both ways, a goal's Pause,
  and Accept on a configuring chat's proposal from *Needs you*, by hand at each width,
  which no test keeps; a press while the page updates, on a Release alone. A task's Cancel,
  an agent's task's Stop, Reject and Start over are drawn at each width by their
  gallery states, and no walk on record presses them.
- **Not yet:** 3.5, *temper's own way, beside the page*: no walk on record presses "See how it
  works" or "Start from it", and no journey records them: the view's tests and the gallery's
  `configure-reference` and `configure-reference-draft` alone draw them, since the plan that built
  them took no walk.
- **Design departs:** 3.5, *The configuring chat's page* and *temper's own way, beside the page*,
  from the request `configure-temper-own-way` of 2026-11-06, by the user's word that day and that
  request's plan's cut. The picture, the figures and the table's Workers and QA stand in
  `../draft/own-way.status.md`. Beside them:
  - the reference's title is level 2 at 28 px over level 3 sections at 17 px, as a chat's document
    has them, where the request's page has a small level 3 title over large level 2 sections;
  - its section 3 says of a goal's branch "whose tests run along the way", where the request has
    it checked "by CI and a review on the way": the reference's goal branches have neither
    (`../draft/reference-code-production.md`, sections 6 and 10);
  - a table's row name is bold in a cell, where the request's rule is `th[scope=row]`, since
    skein's reader draws no row header; a figure's term and value are `role=term` and
    `role=definition`, where its rules are `dt` and `dd`;
  - the thread's padding of 28 and 8 reaches the opening alone, at wide widths, where the request's
    rule reaches every chat; a phone keeps its 20 and 16;
  - the card's steps are a list with no marker and the pane a region of slots, where the request's
    page has an `ol`, an `article` and `section`s;
  - a phone hides the composer's hint, by the rule every session has there, where the request's page
    shows it.
- **Not yet:** 4.7: the form draws no read-only line on an adopted repository (`temper-client-view/src/forge/page.rs`).
- **Not yet:** 3.5, *The try*: the view has no fact of a trial, so `model_check::trial`,
  the one function `../draft/model-check.md`, 2 to 4, feeds, reads the host's check alone:
  under way is the first step Running, and settled with no usable source the first step
  Stopped, its why the host's; a usable account is its ticked line with no "answered in N s".
  No page reaches Running past its first step, Answered, a sign-in's or a key's steps, a stop
  at a later step, a stop with the account usable, or a reason with no act: each is drawn and
  tested (`temper-client-view`'s `tests/model_check.rs`), with no gallery state. "Try again"
  checks the host again. Home's line after a try that stopped and the same lines on an account's
  row of the accounts page are not drawn: nothing reports a try once the person has left Ready.
- **Design departs:** 3.5, *The try*, from the look request `model-check-pane` of 2026-11-06,
  each by the cut of its landing: the steps' list has hidden state words and its accessible name,
  as Forge's has, where the request's pages have the same words, because the terminal reads
  them. A line with nothing yet keeps its "Forge" or "Model" label, as the request's words say.
- **Not yet:** 10: a walk from the sheet through planning, a live report and
  delegate tasks, Pause, Resume, a message and Stop, which the request
  `ui-goal-page` of 2026-11-05 asks of the browser tier and of a box. The
  view holds Goals' link to `/chats` (`the_link_shows_with_a_tracked_template`),
  and the fake box's walk reaches an
  ended goal's report from its Goals row, in the terminal's client
  (`a_chats_goal_on_the_fake_box_shows_its_report_in_the_goals_list`). The fake
  box's model (`temper-fake/src/backends.rs`, `model_document`) writes a report
  only for the first chat's goal, task 2, and has it ask, wait or finish with no
  delegates, so it can show no planning and no delegate task: the journey
  `chat-draft` walks a chat's sheet to that goal's page and back to the chat alone.
  The request `ui-final-walk` of 2026-11-05 walked a live report, Pause, Resume and
  Stop there by hand, which no test keeps. Of a configuring agent, that model scripts
  one proposal, the cue "Show drafts and goals for this project.", and no document,
  and it has no hold of temper's own, so the gallery alone draws those states.
- **Not yet:** 5.13, *The card in a chat*: a live proposal of temper's kinds draws its
  reason's lines and none of its items: jig's document heads leave out every
  application's item and project's policy (`jig-domain-core`'s `settings.rs`,
  `proposal_heads`), so `temper-client-domain`'s `forge/ways.rs` asks no values for it
  and the card has no "Drafts: on". The person accepts on the agent's words. The
  gallery's `configure-chat-proposal` rows alone draw those lines, from a hand-made
  document.
- **Not yet:** 3.5, *Needs you*: an accepted proposal of settings stays the flow's
  card, the rail's count gone, where an answered card leaves the open ones and the
  mock moves it to its answered stack (`temper-client-view`'s `pages/needs.rs`). Seen
  on the walk of the request `ui-walk-large` of 2026-11-06; no test holds it.
- **Not yet:** 5.13: a chat's conversation says "Decision made" above a proposal of
  settings that still waits (`temper-client-view`'s `pages/task.rs`, `owner_activity`,
  `Activity::Decided`), seen on that walk at both widths. Which decision the line
  records is unknown; no test holds it.
- **Not yet:** 5.3: the rename's form asks the focus, and so does an open reply:
  `temper-client-view`'s `pages/ask.rs`, `reply_form`, calls `b.focus(0)` with no
  guard, where `pages/chat.rs`, `document.rs`, `settings.rs`, `new_chat.rs` and
  `configure.rs` ask only while no rename is open. So an owner who presses "Rename
  PROJECT" with a reply open on Home or Needs you meets skein-ui's
  `Builder::focus`, at most one per tree, which asserts. The `ui-rename-focus` task
  `rename-row` has opening the form on such a page not panic; no reason is recorded
  for this site. Seen by reading; no test holds it.
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
- **Not yet:** 3.5, 4.6, 5.5: the walk the request `first-usable-model` of
  2026-11-06 asks: none on record has a real box with one account, unedited, start
  a chat that answers, or a box with no usable account say why it waits. A real
  `temper program-login` needs a person's sign-in, so such a walk starts from a login
  already signed in, or a key, and pastes the printed command only as far as the
  program's prompt. No test runs that command through `temper program-login`:
  `temper`'s `program_login/tests.rs` runs it by `sh -c` against a stub, so that it
  reaches the sign-in for that box's `programs/NAME` is seen by reading.
- **Design departs:** 5.5, *A run's wait*, from the request `first-usable-model` of
  2026-11-06, decided by its plan's cut: a Chats row says "Starting" for a chat
  that can't start, where the request has a run that isn't about to start say why,
  because a row holds no run wait and its query carries no run's check per row
  (jig's `client/domain/client.md`, 19.1). The chat's own page says why.
- **Not yet:** 3.4, 5.5: the composer's
  supplied default, jig's
  `resolved_default`, which neither jig's `Me` nor temper carries. Until it
  does, a new chat's line and the `Configure` page's take the first model of
  the catalogue whose provider is usable (3.5), in place of "Using the default
  model". That is what the run takes on the stock settings, whose charters keep
  the catalogue's order; it can differ where a configured charter orders its
  models otherwise, a catalogue is clipped, or a configured template 3 names a
  model. 3.5:
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
  the client wire has no not-installed reason, so ctl offers repository access
  and Forgejo collaborator-admin recovery rather than installation recovery.
  This gap is pinned by
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
- **Design departs:** 3.5, *Claude*, 4.6, from the request `ui-walk-2` of 2026-11-06,
  decided by its plan's cut. `program_environment` answers the owner at any time,
  where the request has it during set up, because Connect a model draws the same
  command after set up.
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
- **Design departs:** 5.3, from the greenfield target, by the request
  `ui-owner-menu` of 2026-11-06, at the user's word: the rail's foot row is the
  button of a menu with "Accounts" and "Sign out", where the mock draws the
  person's name as plain text, and a browser over 760 px signs out there rather
  than on the `Accounts` tab. `greenfield/README.md` notes it.
- **Design departs:** 3.5, 5.13 and 10, from the request `ui-walk-large` of 2026-11-06:
  - 5.13: that request has the chat's card stay. It stays for the session, from the
    proposals *What the client keeps* holds; a reload or another device loses it, an
    ended proposal asking no card (section 11). Built so by its task, for the user to see.
  - 3.5: a proposal of settings shows in the Needs you of a project's owner who also
    owns the deployment, those who can decide it, by that plan's cut: a project's owner
    alone has no row of it (jig's `domain/engine.md`, 17.6, and its status).
  - 10: that request's browser walk was made by hand, at 1440 and 420 on fresh fake
    boxes, its driver thrown away as that plan's cut has it, so no test keeps it. The
    client's world holds the card, count and Accept
    (`a_configuring_proposal_of_a_projects_ways_reaches_its_needs_and_its_rail`), and
    the process tier a person's plan's return
    (`a_plans_project_has_its_brief_on_a_return_past_the_sitting`), past a sitting a
    test's `forge.sitting_ms` sets to seconds, where that request's return is past 31
    minutes, because a real box has no fake clock.
- **Design departs:** 3.5, *Sign out*, and 5.3, from the request `ui-owner-menu` of
  2026-11-06, decided by its plan's cut. A press outside that took the focus to
  another control leaves it there, where the request returns the focus to the row:
  the row would take back what that press landed on. A project-less `Accounts` page
  keeps its "Sign out" in the banner, where the request keeps it at the tab's foot:
  that page has no such row (5.1). Over 760 px the tab's whole row leaves, "Signed in
  as NAME" and the login with "Sign out", where the request moves "Sign out" alone:
  one form holds them, and the sheet hides it, since the view knows no width (section
  11, *The person's login over 760 px*). The rail's "+ New chat" menu closes on a
  press outside too, which the request doesn't name: skein's page part gives it to
  every menu.
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
  for the key, the paste and Claude alone: the request `sign-in-new-tab` took the
  paste's page and the card's way to it, the reason that cut gave, and no plan has
  weighed the card since. A repository Home's sheet asked leaves the step's lines once in and
  reads as any in the project, "ORG/NAME is in PROJECT" and "Chats can change its
  code now.", where the request has each copy ticked and then "Chats can change its
  code now…": section 11 holds it. The sheet alerts past 16 addresses, where the request
  names no bound: the wire and the store need one (`../protocol/setup.md`, 6.7).
