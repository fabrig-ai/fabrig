# address-contracts: status

2026-11-22. Where the tree's code lags or departs from [address-contracts.md](address-contracts.md).

- **Not yet:** `Work { project, filter }`'s row: the subject `drafts` is
  not read, and no landed draft's card starts a goal [in flight: change 664].
- **Departs:** *A chat's terminal*: the quiet line says "N spent · of B",
  because `temper-client-view`'s `pages/terminal.rs`, `under`, reuses
  `Word::ChatbarSpend`, including the budget, rather than own spend alone.
  Temper/ui owns this remaining words gap.

- **Departs:** *A chat's terminal*: a settings proposal's compact line
  keeps "Accept" and "Change something", with Reject on Needs you, rather
  than the terminal paragraph's "Reject…". `cards/proposal.rs`, `settings`,
  shares `settings.md`, section 2's controls; the departure from `ui-chat-accept` is
  recorded under `settings.md`, 2 below. Temper/ui owns their reconciliation.

- **Not yet:** *The `Models` tab*: the omitted-count delivery gap remains `jig/docs/design/domain/engine/settings-catalogue.status.md`, 2. The view draws what it receives.

- **Design departs:** *The `Models` tab*, from the look request `model-picker`'s 07, for
  now and on purpose: an address under the project, `/projects/<project>/settings/models`, as
  `me`'s offer is a project's, where 07 draws `/models`; and "the default" on one row, as `me`
  names one model for a template, where 07 also draws "the default without Claude".
  The lede describes the offer and picker rather than 07's automatic-read
  explanation; the provider's source clause and Read again carry the read's
  state. The subscription's clause is "These come with temper.", rather than
  "Claude Code has no list to read: these come with temper.", as the design
  specifies. The placeholder is `model-name` rather
  than 07's hand-made `claude-haiku-5-5`, since the selected provider can differ.
  The final line keeps `.temper-settings-facts` with `.temper-models-ask`,
  rather than 07's `.temper-settings-detail`, as the design specifies; its
  link, "ask temper.", includes the final period, where 07's leaves it outside.

- **Not yet:** *A person's pause*: no browser test walks a Pause, a Resume or a message that resumes. The client's
  world holds them on the engine's root, with no browser (`a_chat_pauses_resumes_and_a_message_resumes_it`,
  `a_delegate_paused_from_its_page_is_held_then_resumed`), and the check that request asks on a fake box leaves its
  proofs on a builder, outside the repository, so no record here shows it ran. A refused message is held on an answered
  refusal in the domain (`a_refused_message_to_a_paused_chat_stays_in_the_composer_s_refusal`) and by jig's all-or-none:
  no world moves a hold's revision between the read and the send. "Paused" on Home is seen by reading: the view's tests
  draw it on a Chats row (`a_paused_chat_s_row_says_paused`) and none on Home.

- **Not yet:** `ConnectModel`, *The sign-in's tab*: only Chromium has run it, and against skein's fake issuer
  alone. Firefox's and Safari's gesture is read and not run (skein's `ui/web/testing.md`, 3), and no walk on record has a
  provider's own page in the tab: the request `sign-in-new-tab` of 2026-11-06 leaves a real box to the user.

- **Not yet:** `ConnectModel`, *Its end*: no test has a slow sign-in move on once a hidden tab's stream is opened
  again: `skein-ui-web`'s `tests/hidden.rs` holds the letting go natively, and each look of the browser tier's brings
  the page to the front within the minute (skein's `testing/browser-operations.md`, section 1). On the welcome, no test has a paste's end
  move on: `watch_success_replaces_a_pending_paste_with_ready` runs on `ConnectModel`, and `route.rs`'s
  `welcome_model_progress` reads the same for both. Seen by reading.

- **Design departs:** *The sign-in under way* and `ConnectModel`, *While it waits*, from the request
  `sign-in-new-tab` of 2026-11-06, by its plan's cut and its account. The terminal draws the note's plain title,
  its muted line and "Open the sign-in page" above the folded paste, where the request has the terminal as today:
  one function draws the page for both shells. `ConnectModel`'s disclosure takes auto side margins, where the
  request's rule has none, so that it keeps its note's edges under that page's width. The note's "‹ Back" goes to
  Choose, where the request's page, made by hand from Choose's, goes to Forge.

- **Departs:** `ConnectModel`, *The paste*: a paste answered *finishing* keeps the note's "Waiting for
  ChatGPT’s sign-in, in the new tab" and its link, and reads "Finishing your sign-in…" in the disclosure, where the
  welcome's note reads those words alone (*Finishing a paste*): `temper-client-view`'s `pages/connect_model.rs`,
  `body`, draws `sign_in_wait::waiting` while the sign-in runs, whatever its paste. No reason is recorded.

- **Not yet:** *while it runs*, which the request `chat-reading-b` asks of a
  terminal and a browser both: no test takes the call under way from the engine, or
  draws it running in a browser or a real terminal. The client's world writes the
  run's `call` into the delivery the client hears, its joint world's decoder being
  unable to leave a call unanswered (`temper/tests/client/tests/suite/call_line.rs`,
  `outstanding`). Chromium's story opens a committed line alone, since the demo's fake
  LLM holds text pieces, not calls, and a held text ends a run
  (`temper/tests/client-browser/tests/browser_all/browser_pages.rs`, its history's test), and the joint world's
  terminal reads a committed line alone (`temper/tests/client/tests/suite/call_line.rs`,
  `a_person_at_the_terminal_reads_a_run_of_calls`). A browser's running line is seen in
  the gallery's `chat-work-running`; a terminal's in the view's tests and that world.

- **Departs:** *while it runs*: a call under way with none ended draws a
  description too, its words twice ("Checking template 2", "checking template 2"),
  where the section has the line alone: `temper-client-view`'s `pages/chat_line.rs`,
  `Run::flush`, takes the description from any call under way, and the client's world
  (`an_agents_page_says_the_call_under_way`) holds it. No reason is recorded.

- **Departs:** *a failed call* and *while it runs*: a run still out with a failed
  call keeps `temper-work-failed`, so its call under way stands in the signal's colour
  after the sheet's comma, and the line says no "1 failed" until the run ends, its row
  alone saying so (`pages/chat_line.rs`, `Run::flush`; the colour seen by reading the
  sheet). No reason is recorded.

- **Not yet:** *the words being written* and *While it works*: no browser test
  holds the writing mark's placement for Markdown beyond paragraphs. The
  `ui-pgup` render proof at `e7bd0179d9` holds paragraphs through streaming and
  commitment at 1440 and 420 px, with focus, draft and caret kept; it does not
  settle the fence and open-marker cases below. The request `chat-reading-a`
  has the user read them on a real chat.

- **Departs:** *the words being written*, and `chat/conversation.md`, 2: a call or card drawn
  at position zero of a committed turn takes the provisional text's key,
  giving the later text another and replacing its nodes
  (`pages/chat.rs`, `committed`, `block_key`). The plan `ui-pgup` left this
  case outside its fix unless its reproduction saw it; its view test
  `a_committed_redraw_keeps_the_nodes_of_the_words_being_read` commits a text
  first. No render proof records the other order.

- **Design departs:** *Starting a goal from a chat's document*, and `goal.md`, 3, from
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
  A proposed goal is `goal.md`, 3's toast, where the request has a line above the composer:
  `Chat.proposed` is set (`temper-client-domain`'s `pages/start_goal.rs`, `told`) and
  read by nothing. No plan holds these; `../README.md`, section 5 keeps them.

- **Design departs:** *A chat's document*, from that request's decision 7,
  decided by the answer that day to `goal-sheet-state`: `drafts_offered` reads kind
  7's `drafts`, true in the stock (`settings.md`, 2), where the request has it true while `me`
  lists a tracked template with no conversation flag. So a project can turn the offer off.

- **Departs:**  and `../view-navigation.md`, 1, from the mock (`greenfield/`):
  Ask temper has its own project page, rail entry and phone tab, and
  Settings has no configuring offer. The user's draft, 2026-11-07
  (`../../draft/ask-temper.md`, section 9).

- **Design departs:** *A chat's document*, `settings.md`, 2, from the mock (`greenfield/`), whose
  fresh projects have `drafts` false (`data.js`) and whose "+ Draft" follows it on every
  chat (`app.js`): a fresh project's chats write drafts, and an Ask temper chat offers
  "+ Draft" whatever `drafts` says. The user's word, 2026-11-07 (the request
  `ui-drafts-default`).

- **Design departs:** *A chat's conversation*, from the look request `chat-work` of
  2026-11-06. A failed call's "1 failed" shares the description with the run's time, both
  in the signal's colour, where the request adds ", 1 failed" alone in it, and a document's
  write ends a run (`../../draft/chat-reading.status.md`, 2): each decided by that plan's cut. The words being
  written end in a faint, inline "▍" that does not blink, where the request's overlay draws
  a block 7 px by 15 that blinks: the plan `chat-reading-a` set the mark, this request's
  landing took it at a catch-up, and no reason is recorded. Its pages' line says "Read 7
  docs, checked settings 3×" of calls that start with `settings`, where a phrase follows the
  first call, so the gallery's reads "Checked settings 3×, read 7 docs"; and its 01 gives
  the call under way an outcome, which a client is not given.

- **Departs:** *The Chats page*: the document read stops at
  `limits.client.domain.rows` notes, so a chat beyond those notes has no
  "writes" line and is absent under Drafts. Goal chips cover only the newest
  `rows` chats across the live and closed lists, with ties by the lower chat
  number; a row past that window has no "started goal" line. The answer of
  2026-11-05 to `ui-chat-draft/chats-drafts` keeps this one chips window,
  without raising jig's bound.

- **Design departs:** Home, from the greenfield target, by the request `ui-small-1` of 2026-11-06: the mock's empty *Needs you* repeats the headline's "Nothing needs you."; here it says that once, in the headline.

- **Design departs:** *Ask temper's page*, from the request
  `configure-temper-own-way` of 2026-11-06, by the user's word that day and that
  request's plan's cut:
  - the thread's padding of 28 and 8 reaches the opening alone, at wide widths, where the request's
    rule reaches every chat; a phone keeps its 20 and 16;
  - a phone hides the composer's hint, by the rule every session has there, where the request's page
    shows it.

- **Not yet:** *The try*: the gallery has no trial past the host's check on
  Ready, and no `accounts-try-stopped` sample (`../web/gallery.md`, 2). The view's
  tests draw those states; `home-try-stopped` draws Home's failed line.

- **Departs:** *The try*, `accounts-words.md`, 1: the concurrent-trial and wait-reason gaps remain `../../domain/trial.status.md`, 6.

- **Design departs:** *The try*, from the look request `model-check-pane` of 2026-11-06,
  each by the cut of its landing: the steps' list has hidden state words and its accessible name,
  as Forge's has, where the request's pages have the same words, because the terminal reads
  them. A line with nothing yet keeps its "Forge" or "Model" label, as the request's words say.

- **Not yet:** *A chat's bar*: on that walk "Start goal…" went once from a first
  chat's restored document, at 420, after a round trip through working notes, with no
  error and no setting, repository or goal changed in between. Eight other fresh boxes
  kept it, at each width, and `a_page_of_working_notes_does_not_hide_a_later_document`
  and `start_goal_shows_when_it_may` pass, so the cause is unknown and nothing is fixed.

- **Not yet:** *Needs you*: an accepted proposal of settings stays the flow's
  card, the rail's count gone, where an answered card leaves the open ones and the
  mock moves it to its answered stack (`temper-client-view`'s `pages/needs.rs`). Seen
  on the walk of the request `ui-walk-large` of 2026-11-06; no test holds it.

- **Design departs:** *Where the focus stands on a chat*, from the request `ui-composer-focus` of 2026-11-06,
  decided by its plan's cut: a suggested request's press asks for the composer's focus on a phone too,
  where the request has no press raise a keyboard the person did not ask for, because the tree has no width and telling
  them apart takes a kit mechanism (`../README.md`, section 5). Whether a keyboard rises is unknown, for it and for the request's own
  answers, up after a send and down on a chat opened from a list: no run on record has a real phone, and headless
  Chromium draws no keyboard.

- **Design departs:** *A chat's conversation*, its cards, from the request
  `ui-chats` of 2026-11-05, which names a proposal, a held task and a question's
  answer as cards in the conversation: a chat's page holds proposals alone
  (`temper-client-domain`'s `pages/chat.rs`, `cards`), since its held delegate
  is its one line (`../view.md`, 5), and the answer the one line under "You" the same
  section keeps. No reason is recorded.

- **Design departs:** *Claude*, `../forge/records.md`, from the request `ui-walk-2` of 2026-11-06,
  decided by its plan's cut. `program_environment` answers the owner at any time,
  where the request has it during set up, because Connect a model draws the same
  command after set up.

- **Not yet:** The welcome's Model step, *The command's Copy*: no browser test
  presses it, since no story in `temper/tests/client-browser` reaches the Claude panel,
  and no walk on record has. skein's `a_clipboard_write_keeps_the_press_and_reports_once`
  holds the part, and `host_tests_copy.rs` holds that the press's own report asks the
  write; no test runs the one line between them, in `carry`
  (`temper-client-shell-web/src/lib.rs`). Seen by reading. Nor does a browser show its
  word change: `the_button_reads_each_label_once_in_the_browser` holds the three names in
  the view's tree, and that a name's patch rewrites the button's text
  (`skein-ui-web/src/page/web.rs`, `refresh` and `content`) is seen by reading too.

- **Not yet:** *Team*, *The project's triager*: no setting names a
  triager (`../../domain/landing-settings.status.md`, 1.1; `../../domain/findings.status.md`),
  so no read gives `team::Facts`' `triager`, `team::parts` never shows its
  line, and the view has no words for it (`temper-client-view`'s
  `pages/team.rs`).

- **Design departs:** *Needs you*, `.rec`, from the request `ui-copy-once` of
  2026-11-06, decided by its plan's cut. The request has no button, tab or option with
  both a name and a text child in a browser, and a recommended choice's button has its
  name and the tag "recommended", which this section draws: those are other words, not
  the name twice. `no_named_control_draws_a_text_child_in_any_state`
  (`temper/tools/gallery/src/tests.rs`) excepts that word alone.

- **Design departs:** *The flows' paths* and *Workers*, from the request
  `ui-settings-tabs` of 2026-11-05, decided by its plan's cut. The terminal
  draws no path, where the request has each flow add one: the same request
  keeps the terminal's prints' meaning, a path would shift every number on the
  flows, and the terminal's rail holds "Deployment". With hosts kept and none
  connected, `.empty` stands above their "Not connected" lines, where the
  request gives none connected `.empty` and no words for that case, so that no
  state is dropped.

- **Design departs:** *Goals*, from the request of 2026-11-05: "6 / 9" is
  on the task line (jig's `client/domain/contract/documents.md`, 1), where the request
  offered the goal's document or the tree's read, and a row reads its report
  alone, not also its tasks by `parent`: each place needed the same stored
  counter, and the line needs no read per row. The lede's ways run in template
  number order, as the request's words have it, so the reference's read "task
  by task, plan, tasks, land whole", where its mock, the target, draws them the
  other way round: number order is what a project's settings decide.

- **Design departs:** *Sign out*, and `../view-navigation.md`, 1, from the request `ui-owner-menu` of
  2026-11-06, decided by its plan's cut. A press outside that took the focus to
  another control leaves it there, where the request returns the focus to the row:
  the row would take back what that press landed on. With no project, "Sign out" is
  the top bar's, where the request keeps it at the `Accounts` tab's foot: the later
  request `outside-pages` draws the column "No project yet" in that page's place
  (`README.md`, *With no project yet*; `../view.md`, 1). Over 760 px the tab's whole row leaves, "Signed in
  as NAME" and the login with "Sign out", where the request moves "Sign out" alone:
  one form holds them, and the sheet hides it, since the view knows no width (`../README.md`, section
  5, *The person's login over 760 px*). The rail's new box's menu closes on a
  press outside too, which the request doesn't name: skein's page part gives it to
  every menu.

- **Design departs:** *Settings and Deployment*, `../view-navigation.md`, 1, and `settings.md`, 2, from the request `ui-settings`
  of 2026-11-05, decided by its plan's cut. "Sign out did not reach the engine."
  stands on the rail's foot as well as the `Accounts` tab, where the request moves
  it there: a person with a lost sign-out may be on any page (`../line.md`, 6). Kind 7's
  controls are disabled together while one is out, where the request disables the
  one pressed: they write one record, so two at once would lose one. And a
  proposal's card has no application rows for a person with no review,
  including one with no role in the deployment, where the request has their
  values for everyone: the heads omit application items, and the review and
  proposal-value read keep the deployment role because items span scopes
  (`../../protocol/configuration/settings-kinds.md`, 1).

- **Design departs:** *The welcome* and *Home*, from the request `ui-welcome`
  of 2026-11-05, decided by its plan's cut. A plan's way at its address, with no
  sign-in asked, keeps its card in the panel, where the request draws a way's page
  for the key, the paste and Claude alone: the request `sign-in-new-tab` took the
  paste's page and the card's way to it, the reason that cut gave, and no plan has
  weighed the card since. A repository Home's sheet asked leaves the step's lines once in and
  reads as any in the project, "ORG/NAME is in PROJECT" and "Chats can change its
  code now.", where the request has each copy ticked and then "Chats can change its
  code now…": `../README.md`, section 5 holds it. The sheet alerts past 16 addresses, where the request
  names no bound: the wire and the store need one (`../../protocol/setup/owned-repositories.md`, 1).

- **Not yet:** the whole document: the real-provider picker walk remains `accounts-words.status.md`, 1.

`README.md`'s Models tab, switches, add-by-name form, source, estimated
prices, omitted count and All models' link to the project's Models tab
have no known implementation gap apart from the count's delivery below.
Provider, the name and Add share one
unwrapped row with the field's label visually hidden; its accessible name
remains. The Provider and Add section headings have 6 px bottom margins.
The client world holds switches and adding across reloads. Page 07's
render comparison at 1440, 1280, 1000 and 420 px belongs to temper/ui after
landing; it and browser and real-provider
checks remain `../README.md`, section 5's deferred work, with no outcome supplied here.
`README.md`'s provider-list source clauses, Read again, Reading and read
refusals, and the expiring new labels on the Models tab and All models have no
known implementation gap. View tables and gallery states cover these
words and expiry; the client world carries reads, failures, retries and
retained names through the engine. They establish no rendered equality.
The one-process world's ModelsList row carries the key read through a
worker and hosted agent and checks one request, the new name in `me`,
and unchanged accounting. No real-provider result is supplied;
`../README.md`, section 5 keeps that observation and the page comparison after landing.

- **Not yet:** the whole document: the first-usable-model real-box walk remains `accounts-words.status.md`, 1.
- **Not yet:** the whole document:
  the Board address and reordering remains `../engine.status.md` (Not yet).
   the project-only settings-role gap remains `settings.status.md`, 2.
   the Home settings-tab links remain `settings.status.md`, 2.
  `README.md`, *Goals*: the empty Goals-page gap remains `work.status.md`, 1.
  the whole document: the Forget request remains `../forge/records.status.md` (Not yet).
  `README.md`, *Ready*: the unhandled Ready endings remain `welcome.status.md`, 1.
   the plan's first-use card remains `accounts-words.status.md`, 1.
  `README.md`, The welcome:
  the welcome role-scope gap remains `welcome.status.md`, 1.
  the unfinished-forge name remains `../forge/records.status.md` (Not yet). the restarted GitHub setup continuation remains `../forge/records.status.md` (Not yet).
   the App-not-installed reason remains `../forge/family.status.md`, 6.
- **Design departs:** the whole document, `goal.md`, 2, from `ui-goal-asks-2`'s whole-question
  ruling, by its cut: only the first question on a goal's page draws whole;
  later bands keep the bounded first line and Answer, to price at most one
  `long_text` question per page (`../view.md`, 4). Needs you draws the selected question
  whole. `../README.md`, Section 5 keeps extending the bands open.

The document and `accounts-words.md`, section 1 and `task.md`, section 2 have no known implementation gap in the closed
chat's header, closed agent's taskbar or the composer's named template
default and effort. The view table covers both shells, including a turn
with unknown effort, an unoffered model and the unread tail. The joined
client story carries smith's turn bytes through decoding to `T1 · model`
in the terminal's closed header. Browser and real-model checks, the goal
page's line and latest-run metadata beyond readable turns are `../README.md`, section 5's
later work; the phone keeps its hidden byline.

- **Design departs:** the whole document, `goal.md`, 2, `task.md`, 4, from the greenfield target: "Pause", "Resume" and "Close" on a chat, a goal and an
  agent's task, where its goal says "Stop", which meant a goal's end and a chat's hold. The user's word, 2026-11-06, the
  request `ui-pause-close`.

- **Not yet:** the whole document:
  the Board address and reordering remains `../engine.status.md` (Not yet).
   the project-only settings-role gap remains `settings.status.md`, 2.
   the Home settings-tab links remain `settings.status.md`, 2.
  `README.md`, *Goals*: the empty Goals-page gap remains `work.status.md`, 1.
   the Forget request remains `../forge/records.status.md` (Not yet).
  `README.md`, *Ready*: the unhandled Ready endings remain `welcome.status.md`, 1.
   the plan's first-use card remains `accounts-words.status.md`, 1. `../root.md`, 1.6:
  `README.md`, The welcome:
  the welcome role-scope gap remains `welcome.status.md`, 1.
  the unfinished-forge name remains `../forge/records.status.md` (Not yet). the restarted GitHub setup continuation remains `../forge/records.status.md` (Not yet).
   the App-not-installed reason remains `../forge/family.status.md`, 6.
