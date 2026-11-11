# A task's page

## 1. The page

- **A task's page** is of its kind, as `task.html` lays each out: a
  tracked task's, a goal's (`goal.md`, 2); an agent's task's, for an agent's
  task with no priority (2). A task with the chat label opens as
  `Chat { task }`. A procedure's task shows a tracked task's header
  and plan, with its goal and its plan as "Its tasks", its ended tasks
  among them, and a change's task its panel (3); a person
  task shows a tracked task's header and plan only (`../README.md`, section 5).

## 2. An agent's task's page

An agent's task's page (`README.md`) is that of an agent's task with no
priority and not the chat label: a goal's delegate, the agent a
change's procedure makes, a task an agent's task delegated. It shows
its task document, its conversation as a chat's page holds one, and its
owner tree's held cards, as `mockups/task.html` draws its agent kind,
in the chat's look. It draws each amount and each state as the
documents supply them (jig's `client/domain/contract/documents.md`, 1), and
acts on it as its document offers (4).

- **Its parts, in order:** the lineage bar; the taskbar, with its acts
  (4); "Task brief"; the run's heading, then its own held card while
  it is held (4); the earlier read's loading or failure status (`chat/conversation.md`, 2);
  the conversation, read by its log's edges with no "Older" button;
  its acts' outcome; "Result", once it has ended; "Plan",
  while its tree holds a delegate or a held task's card; the composer
  while Message is offered (4); then the confirmation while one is
  open (`../README.md`, 3.6). At phone width the order is
  the same: the lineage bar keeps one line, its titles cut short, and
  the taskbar's parts wrap.
- **What scrolls,** in a browser: the log alone, as on a chat's page, so
  that PgUp and PgDn from the message box, "Latest" and the kept place
  (`chat/conversation.md`, 2) act on it as there. From "Task brief" to "Plan", the parts above
  stand in their order inside the log, around its lines, and "Latest"
  beside it, after it, as a chat's. The log follows its end, so that the
  page opens there and the brief is read back to (`../README.md`, section 5). The
  amendment's editor (4) stands above the log, where it is seen when
  it opens, since it takes no focus; it scrolls itself where a short
  window can't hold it. A terminal scrolls nothing of its own: it prints
  those parts around its log, in the same order.
- **The lineage bar:** its parent, the last of its lineage
  (`lineage_chips`, root first), as its "Task N" link and its title,
  after an arrow; then the task itself, "Task M" and its title. With no
  lineage supplied, the parent is its requester's "Task N" link when a
  task asked it, and there is none otherwise. The arrow and the slash
  between them are the look's.
- **The taskbar:** its level-1 heading "TN ·
  TITLE", as a chat's; its state, in `goal.md`, 2's words, from the same
  function (`pages::task::header_state`), with its status token; then
  its executor, "Agent", and its own spend, "N spent", its `budget.spent`
  as a plan row's (`goal.md`, 2); on a closed task, then what it ran on (`chat/composer.md`, 1, *The
  work's account and model*).
- **"Task brief":** a region, the mockup's "TASK BRIEF", whose capitals
  are the look's: its specification as written (`spec`, read as Markdown); who asked, in `goal.md`, 2's words ("Asked by
  task N"); then its dependency line, as a plan row's (`goal.md`, 2): the list
  "Dependencies of task N", one item per dependency of its document
  (`dependencies`), in order, keyed by its task, with its "Task M"
  link and its state, and an added one's mark (`goal.md`, 2), from the
  document's `dependency_facts`. The document's `cut` does not say
  which of its lists was cut, so the line has no last item (`../README.md`, section 5
  ). The mockup's worlds declare no dependency: the line is
  temper's, for rung B2 (`../../testing/live-tier.md`).
- **The run's heading,** one, for its current or its last run, from its
  chip's owner phase and its `attempt` (jig's `client/domain/objects.md`,
  section 3):
  - "Run N · live now" while a run is out: a claimed or a running
    run, or a run closing, N its attempt;
  - "Run N · waiting" while the task lives and no run is out, N its
    last run's attempt;
  - "Run N · ended" once the task has ended, N its last run's attempt;
  - "No run now" while it lives and has had no run, and no heading
    once it has ended without one.

  Under a live or a waiting run's heading, "Started MM-DD HH:MMZ" when
  its chip supplies `run_since`, when the run was placed, in universal
  time. A run's earlier runs are the
  conversation's "Run N began" and "Run N ended": the page has no run
  groups. An invalid run's end adds ": its result broke its contract
  (KIND)", naming form, verdict, words, change or followups from its
  recorded cause. An invalid end without a contract kind adds ": its
  result couldn't be read" (jig's `protocol/client/wire.md`, section 1).
  Other failure classes add no invalid-result cause.
- **The conversation,** from the task's bounded thread (jig-client's
  `thread`), read as the chat's page reads it, is drawn as work, not as
  a chat's (`README.md`): its latest window; committed turns, the agent's
  "Temper" over its texts; its calls in runs, as a chat's (`README.md`), one
  line a run, running while `task::run` says a run is out and the words
  being written haven't begun, a `delegate` call's a row too, since a
  delegation's card is the chat's (`../view.md`, 5), and a run's results its rows'
  outcomes, a message that holds only those drawing no "Completed"; "Run N began" and
  "Run N ended"; and a live run's words being written, as the chat's.
  No three dots: its run's heading says it is live. "Latest" stands over
  the log's foot while its end is out of view, as a chat's (`README.md`).
- **Its result,** once it has ended: a region "Result" after the
  conversation, its state and its words as a goal's "Result" says them
  (`goal.md`, 2), from the same function (`pages::delegation::facts`):
  "Completed", "Landed" (done with a change's result, as `goal.md`, 2 says),
  "Failed", "Cancelled" or "Ended", and the report's words, or a
  failed task's reason. The mockup's "RESULT" capitals are
  the look's. A task that has not ended has no such region.
- **Its plan,** while its owner tree holds a delegate, or a held task's
  card, its own or a delegate's: "Plan", by `goal.md`, 2's rules for a plan, its rows,
  their dependency lines and held tasks' cards. The mockup draws none on an
  agent's page: its held cards are what this page may act on.
- **Against its mockup,** the page's way:
  - links and the lineage name a task "Task N", the mockup's "T12"
    (`README.md`, 2, "Links are by number"); the executor is "Agent", where the
    mockup names the charter (`README.md`);
  - the playground's banner stays where the frame draws it, before the page,
    where the mockup draws it under the taskbar (`../view.md`, 1);
  - its calls in runs, as a chat's (`README.md`), where the mockup folds them
    by message ("7 tool calls", then "read ×6 · recall"), and its results
    its rows' outcomes;
  - not drawn, since no document supplies them: the charter's, model's,
    account's, worker's and pool's names ("Implementer · Sonnet ·
    medium", "on s1", "holds a slot of T12's pool"); whose
    specification it took ("from T3's spec"); a run's own spend ("N
    spent so far"); and a turn's time, which a chat's page alone says
    (`../view.md`, 6). Nor its context (`Task.context`), which
    is optional;
  - its acts are 4's, with their own departures there.
- **Where it is decided:** the domain's task page has the shape
  `Agent`: an agent's executor, no priority, not the chat label, which
  opens as `Chat { task }` (`README.md`). It holds the task's thread as the
  chat's page holds one: made with the chat's bounds, its latest page
  read as the page opens, read again on each change of the document,
  given the run's live text while the `run` watch is open, and its run
  ended as the chat's is when no run is out. A pure function of the
  domain derives the run's heading from the chip (`pages::task::run`);
  the state and the result are `goal.md`, 2's. The view lays them out with the
  phrasebook's words.
- **The look:** its own, MAIN and AGENT_PAGE on its main, TASKBAR on its
  taskbar, THREAD on what follows it and the conversation's tokens, so
  `temper.css`'s rules for this page draw them; and AGENT_LINEAGE
  on the lineage bar (a node
  carries two tokens at most, and its main has MAIN and AGENT_PAGE),
  AGENT_BRIEF on the brief, AGENT_RUN on the run's heading and its
  start, AGENT_RESULT on its result, and AGENT_FOOT on the keyed slot
  holding the acts' outcome, result and plan after the lines. `task-pages.css`
  draws them as the mockup's
  `.agent-lineage`, `.agent-brief`, `.agent-run-heading`,
  `.agent-run-meta` and `.agent-result` (`../web/look.md`).

## 3. A procedure's task's page

A procedure's task, a change among them (`../../domain/forge/change/README.md`
), shows its header, "Task details", and its plan (`goal.md`, 2's rules for a plan), as a
person task does, with its goal and "Its tasks". A forge procedure's task, whose
panel the forge's side watches while its page is open (`../forge/served.md`,
2), draws its panel as `task.html`'s change does: its steps, its
hold, its gates and its bounds. Each fact is drawn as the panel
supplies it: the page computes none (jig's `client/domain/contract/documents.md`,
1).

- **Its parts, in order:** the bar, a change's opening with the "Changes"
  breadcrumb, to its project's `Changes`; the header; "View on forge ↗"; the
  change's card, holding its steps, its step's line, its hold, "Its
  tasks", "Gates at its head" and its bounds. At phone width the order
  is the same, and the steps take three columns. Until its panel is
  shown, and for any other procedure, the page is the bar, the header
  and "Its tasks".
- **Its goal:** in the header's first line, after its project and its
  procedure's part, a slot of "Goal" and the "Task G" link to the goal
  it serves, the document's `goal` (the nearest tracked task at or
  above it, itself included: jig's `domain/model.md`, 6.2), when that is
  another task. A tracked procedure's task serves itself and one under
  a chat serves none; neither shows the slot.
- **The header, its panel shown:** its first line begins with the tag
  "Change · Task N" where `goal.md`, 2's says "Task N" and its executor, then
  its project, then its goal's slot; the procedure's part is the panel
  itself, so the line has no "Shown on the forge.". Its title follows,
  its state and who asked, as `goal.md`, 2's; then a line naming its pull
  request, "PR #N · OWNER/NAME · BRANCH → LANDING", or "No pull
  request yet · OWNER/NAME · BRANCH → LANDING" before one is opened,
  the repository, its branch and its landing branch as supplied.
- **The header, its panel not shown:** `goal.md`, 2's, its procedure's part
  saying where the panel is (jig's `client/domain/root.md`, section 6):
  "Loading the change from the forge." until its first delivery; "The
  forge knows no change for this task." when the forge answers unknown
  (a task that is no change, or one whose subject left the engine);
  "The forge cannot show this change." when its watch ends for any
  other reason. A procedure the forge does not run says "Shown on the
  forge." as before (`../forge/family.md`, 2).
- **"View on forge ↗":** an external link (`../line.md`, 5) to its pull request's
  address on the forge, in a toolbar after the header, only when the
  panel supplies an address (`../forge/served.md`, 2); a forge with no address
  gives none, as the playground's fake forge does.
- **Its steps:** a list of six, as the mockup's: "Producing",
  "Opening", "Checking", "Gating", "Queued", "Landed". The one its step
  is at is current, those before it done, and all six done once it has
  landed. The steps 2.1 draws beside the way show at the one they
  serve: recreating and reopening at "Opening"; a repair of CI at
  "Checking", of a gate at "Gating", of a semantic conflict at
  "Queued"; first, updating and resolving at "Queued"; landing at
  "Landed", current. "Queued" reads "Queued · Nth" while it has a place
  in its queue, "Queued · 1st" when first, the place an English
  ordinal ("1st", "2nd", "3rd", "4th", "11th", "12th", "13th",
  "21st").
- **Its step's line,** after the steps, then the "Task N" link of the
  task its step runs, when the panel names one (`runs`):

  | Step | Its line |
  |---|---|
  | producing | "Producing its branch." |
  | opening | "Opening its pull request." |
  | recreating | "Making its branch again at the head temper last knew." |
  | reopening | "Reopening its pull request." |
  | checking | "Waiting for CI at its head." |
  | gating | "Waiting for its gates at its head." |
  | queued | "Nth in the queue for LANDING, after Task A, Task B and N more.": the tasks ahead it names, nearest first, as links; "and N more" for those it does not name; "after N tasks" when it names none; and no "after" when nothing is ahead |
  | first | "First in the queue for LANDING." |
  | updating | "Updating from LANDING." |
  | resolving | "Resolving its conflicts with LANDING." |
  | repairing | "Repairing: CI failed at its head.", "Repairing: gate N asked for changes.", for a person's request on the forge "Repairing: a person asked for changes on the forge." (`../../domain/forge/change/gates.md`, 1) or, for a semantic conflict, "Repairing: CI failed at its updated head.", which blames neither the change nor what it was updated with |
  | landing | "Merging at its head." |
  | landed | "Landed · merge commit ABCDEFG", its first seven digits, as a delegation's card says it (`../view.md`, 5) |

- **Its hold,** while it is held, a notice after its step's line: "Held
  while STEP: REASON.", STEP the step it was held at in lower case as
  the table names it ("first in the queue" for first), and REASON:

  | Hold | Its reason |
  |---|---|
  | stalled | "its wait stalled" |
  | repairs | "its repairs are spent" |
  | resolutions | "its resolutions are spent" |
  | updates | "its updates are spent" |
  | moved | "its branch was moved on the forge" |
  | missing | "its branch is missing on the forge" |
  | closed | "its pull request was closed on the forge" |
  | retargeted | "its pull request was retargeted on the forge" |
  | failed | "a delegate or a write failed for good" |
  | rejected | "a person rejected a gate" |

  The reasons are `../line.md`, 1's and `../line.md`, 2's, in the panel's measure (`../forge/README.md`,
  section 5): the held card says the rest.
- **Its tasks:** a region and tree named "Its tasks", the tasks its
  procedure made (the agent producing a change's branch, its repairs
  and reviews), where a goal's and a person task's say "Plan". Its
  live tasks are `goal.md`, 2's plan rows. Its ended ones are `goal.md`, 2's ended
  rows, read once from the engine, its children in the order they
  were made (`Read::Tasks`, a `TaskQuery` by `parent` and the ended
  phase, `QueryOrder::Ended`, by number), and read again when its tree
  or its document changes, one read out at a time; they are drawn in
  `goal.md`, 2's rows' order. It has no filter and no counts, as its mockup
  draws none. Until that read answers, an empty tree says
  "No live tasks." as `goal.md`, 2's; after it, a region with no row says "None
  yet." while the task lives, "None." once it has ended. When the
  read's page holds less than all of them, a last line: "More of its
  ended tasks are not shown." The page reads no older page (`../README.md`, section 5
  ). Every procedure's task reads them, its panel shown or not.
- **Gates at its head:** a region of rows, each a name and its state.
  First "CI at its head": "Not yet" while CI is unknown, "Running"
  while pending, "Passed", "Failed". On a landed change, unknown CI is
  "Not read since the restart": a landed change keeps the verdict last
  read at its head with its record, through a restart too, so its CI is
  unknown only when no step read one at that head between the engine's
  last start and the landing, and no step reads it after
  (`../../domain/forge/change/README.md`, 5). Then each gate, in the supplied
  order: "Gate N · review", "· approval" or "· check", then "·
  blocking" or "· advisory", and its verdict: "Not yet" with none,
  "Waiting" pending, "Approved", "Changes asked". With no gate, a last
  row, "No other gate."
- **Its bounds,** the card's last line: "Repairs N of M · resolutions N
  of M · updates N of M", each used of its limit (2.2).
- **Against its mockup,** the page's way:
  - links name a task "Task N", the tag "Change · Task N", the mockup's
    "T28" and "Change · T28" (`README.md`, 2, "Links are by number");
  - not drawn, since the panel does not carry them (`../forge/served.md`, 2):
    the "Fixes finding" notice (findings are a gap,
    `../web/mockups/README.md`); the head's requirements, its heads in order,
    CI's contexts, the pull request's title and mergeability, how long
    a step has waited and when a stall began;
  - not drawn: the mockup's case name beside the tag, a reference
    world's;
  - not drawn: the bar naming its way as `../view.md`, 1's does; "Its tasks"'s
    subtitle ("made by the change procedure…"); and "its landing rules
    add none" after "No other gate.", which the panel cannot tell from
    a rule not read;
  - its tasks' rows are `goal.md`, 2's plan rows, with their executor, state
    and spend, where the mockup draws each as a gate's row with its
    kind and phase; a gate is named by its number and kind, where the
    mockup names it by the landing rule that adds it;
  - its hold, the step's line and its bounds are temper's: the mockup
    draws none.
- **Where it is decided:** the domain's task page holds the forge's
  side's panel (`forge::panel`, its state loading, shown, unknown or
  ended) and its ended children's page, its one read and whether it is
  stale; pure functions of the domain's forge side read the panel: the
  step's slot and the steps done (`forge::change::slot`), the line's
  facts and the hold's step. The view lays them out with the forge's
  words (`../forge/family.md`, 2).
- **Its words and look:** "Goal", the existing word; "Its tasks", "None
  yet.", "None." and the more line, the page's; the change's words, a
  forge table beside its parts' (`forge::words`). CHANGE on the card,
  CHANGE_TAG on the header's tag, STEPS, STEP and STEP_DONE as a
  delegation's card's (`../view.md`, 5) and STEP_CURRENT on the current step,
  CHANGE_HOLD on the hold, GATE on each row of "Gates at its head",
  META on the pull request's line and the bounds, TASK_ACTS on the
  toolbar; `task-pages.css` draws them as the mockup's `.tag.accent`,
  `.steps`, `.step.current`, `.notice` and `.gate` (`../web/look.md`),
  the steps in six columns, three at phone width.

## 4. A task's page's acts

A goal's page (`goal.md`, 2) and an agent's task's page (2) act on their task
as a chat's page acts on its chat (`README.md`): through the acts its document
offers (`Task.acts`, read through `acts::offers`), each sent as `../README.md`, 3.6
says. An act the document does not offer is not drawn, not even
disabled. One it offers is drawn by its admission (jig's
`client/domain/contract/documents.md`, 3), as a held task's card draws its own (`../README.md`, 3.6):

- *allowed* or *bounded:* its button;
- *proposable:* its name, then the card's "This needs a proposal to"
  and to whom, and no button: the page proposes none of these acts;
- *refused:* its name, then the refusal in jig's words, and no button:
  the page sends nothing.

Its rules on a goal's page:

- **Where they are:** `.acts` in the header (`goal.md`, 2), "Amend", "Pause" or
  "Resume" (`.btn.small`), "Close" (`.btn.quiet.small`), keeping `.btn.small`'s
  height at phone width, wrapping as drawn. Ended, none of the three.
- **Pause** sends `Ask::Stop` at once, no confirmation. A person's Stop is a
  placed hold over the goal's subtree. Its toast: "Paused: running tasks stop,
  nothing new starts."
- **Resume,** on a goal in state paused alone: `Ask::Release` naming the hold's
  revision, at once, no confirmation. A subtree hold's root released lifts it
  whole. Its toast: "Resumed." A goal held for any other cause has no Resume:
  its band has the need.
- **Close** opens the confirmation (`confirm.rs`, `Role::Dialog`, `../README.md`, 3.6, *The sheet*):
  "Close this goal?", then "You can still read it, but it can’t be reopened." and
  "What landed on its branch stays there.", "Keep it", "Close goal". It sends
  `Ask::Cancel` with the reason "Closed from its page". A goal branch nothing
  landed from stays (`../../domain/forge/branches.md`, 1), so the sentence holds. The
  acts stay drawn behind the dialog.
- **Amend** opens the amendment's editor below, between the header and the
  bands, in the page's look: `.field`, `.text`, `.btn`.
- **The composer,** while live and Message offered: the mock's `.composer`, as
  the chat's: its placeholder and name "Tell TITLE something", Send disabled
  while empty, Enter sends, Shift+Enter a new line, the focus in it after a
  send (`README.md`), growing to 220 px; then
  `.composer-hint` "Its agent reads it at once. A change of plan becomes an
  amendment it shows you first." While Message is refused as `NotWaiting`, the
  composer's place holds nothing; ended, no composer.
- **The toast** is the frame's (`needs-you.md`, 1: one at a time, one optional
  act). It says Pause's and Resume's done words, a Pause's, Resume's, Close's or
  Message's refusal in jig's words, with "Look again" after a moved one, which
  closes it, and "Unknown" past the engine's retention. Nothing is shown done
  before its answer; a pending request's double press joins it (`../README.md`, 3.6).
- **No own held card, no held rows:** a goal's page reads no card of its own and
  none for a delegate: a held delegate's Release is on its own page, and a
  need of the reader's is a band.

Its rules on an agent's task's page:

- **Where they are.** In its taskbar after its spend, as a chat's chatbar
  holds its acts: "Amend", "Pause" or "Resume", "Close"; and its composer last
  (below).
  The taskbar's acts wrap as a chat's do. While the page's confirmation is
  open, the taskbar draws no act: the dialog is the one place to answer.
- **Its own held card.** While the page's task is held for any cause but
  the person's own pause, its held task's card (`../README.md`, 3.6), with Release and its
  confirmation, follows the agent's run heading. The task is no row of its
  own plan, so its plan never draws it; a delegate's held card stays in its
  row. Release names the hold's revision, and a moved one's refusal asks the
  person to look again (`../README.md`, 3.6).
- **The composer,** while Message is offered: the chat's composer (`README.md`, `../view.md`, 1),
  with its model choice line, its text area "Message", its hint and "Send",
  preceded by the note "Steer this task · words reach its live run at the next
  turn." A message sent shows in the conversation as a chat's person's words
  do, "Being sent", then "Sent". The engine offers Message on every live
  agent's task and refuses it while the task waits to start (`NotWaiting`): the
  composer's place then holds "Message", then a note in the steer note's look,
  temper's "Messages reach it once it runs." (any other refusal: its supplied
  words, in the same look, since why Message isn't offered is no outcome), and
  its model choice line alone.
- **Pause and Resume** are a chat's (`chat/README.md`, 1, *A person's pause*): `Ask::Stop` at once,
  and on the person's own pause "Resume", `Ask::Release` naming the hold's revision,
  at once. The page's state says "Paused"; neither draws a done word, and a refusal
  says why as any of its acts' does.
- **Close,** confirmed (`../README.md`, 3.6, *The sheet*): "Close this task?" saying "You can still
  read it, but it can’t be reopened." and what it changes, then "Keep it" and "Close
  task". It sends `Ask::Cancel` with the reason "Closed from its page".

On both:

- **Amend** opens the amendment's editor, laid out where the page says (above):
  a region "Amend task N". The draft
  is the amendment editor's (`../README.md`, 3.7), `amend::Draft::of` the task's
  document as read when it opens, and the page lays out two of its
  edits, no other:
  - "Specification", a text area holding the spec's words, its edit
    `words`; where the read cut the spec (`Cut::Spec`), the words read
    only and "This task's specification is too long to edit here: use
    `temper ctl task amend`";
  - on a goal, "Tracked", a check, and "Priority", a number, its edit
    `track`: `Tracked { priority }` or, unchecked, `Untracked`.

  Then "Send amendment" and "Discard". "Discard" closes the editor and
  drops its draft. "Send amendment" with no change (`amendment()` is
  none) sends nothing and says "No change". While a document read since
  has another spec revision (`Draft::moved`), the editor says "This task
  changed since the editor opened" and draws no "Send amendment": the
  person discards and opens it again on the new document. Otherwise it
  sends `Ask::Amend` with the drafted change, the drafted revision
  (none for a tracked task's priority alone, `../README.md`, 3.7) and the reason
  "Amended from its page"; a refusal as moved in between says so with
  "Look again" (below). Its outcome is the editor's: pending, then the
  answer in jig's words in the region, a refusal as moved with "Look
  again". A done amendment leaves the region holding its outcome alone,
  where it was made, until "Amend" opens another draft. Amend is drawn
  by its admission as every act is: proposable, the page proposes
  nothing. The model choice for the next runs stays the choice line's
  (`README.md`), its own amendment, not repeated in the editor. Instructions,
  dependencies, the deadline, wake and an addressee are `temper ctl
  task amend`'s alone (`../ctl/areas.md`).
- **Their outcome.** One request per kind of act at a time; a double
  press joins the pending one (`../README.md`, 3.6). On an agent's task's page: pending, its
  progress as a card says it; answered, the outcome in jig's words, or the
  refusal in jig's words, with "Look again" after a moved one; past the
  engine's retention, "Unknown". It follows the conversation, as a chat's. On a
  goal's page, the toast (above).
- **Reading the conversation,** on an agent's task's page, is the chat's
  (`chat/conversation.md`, 2): the log's head asks for earlier activity and its foot for later
  activity, through the same thread, with no "Older" button. Latest returns
  to the window's foot and asks for later activity while the end is missing (`chat/conversation.md`, 2).
- **Against the mockup,** an agent's task's page's way:
  - the composer's text area is named "Message", the chat's, where the
    mockup's is "Write to agent task" with a placeholder "Write to
    T12…" (`../README.md`, section 5, the agent's placeholder); its hint is the
    chat's;
  - the composer carries the chat's model choice line, which the
    mockup's agent page does not draw (`README.md`);
  - the composer shows while Message is offered, the mockup's while the
    task lives, and a refused Message says why in its place (`../README.md`, 3.6);
  - each confirmation, and an act's outcome and refusal, are `../README.md`, 3.6's: the
    mockup's buttons do nothing;
  - its own held card, which no mockup world draws;
  - the acts hidden while the confirmation is open.
- **Where it is decided:** the domain's task page holds the composer's
  words, its pending requests by kind and the last outcome, as a chat's
  page does, and the chat's acts (`Act::MessageWords`, `Send`,
  `Older`, `Newer`, `Latest`) are routed to whichever of the two pages is open; a goal's and an
  agent's task's Pause and Resume (`Act::Pause`, `Act::Resume`), and a chat's
  (`Act::Stop`, `Act::Resume`), are sent at once; an agent's
  task's Close and a goal's Close (`Act::Close`) open their confirmation first, as
  `Confirm` with the page's task as its object, and `Act::Confirm` sends them.
  An agent's task's page reads a card for its own task when its document is
  held, as it reads one for a held row. The view draws each act by
  `acts::offers` and `acts::allowed`.
- **The look:** on a goal, `../web/look.md`, *A goal's look*. On an agent's
  task's page, AGENT_STEER on the composer's note, the mockup's
  `.agent-steer-note`; TASK_AMEND on the editor's region. The taskbar's
  acts and the composer keep the agent's page's tokens (2). `task-pages.css` draws
  them (`../web/look.md`).

## 5. Open questions

- **An agent's page opening at its end** (`task.md`, 2, `chat/conversation.md`, 2): a long conversation's
  page opens below its brief and, on a held task, below its own held card;
  the taskbar says the state. Earlier lines load as the log's head nears, so
  a brief past a screen and a half above a cut window lets a reader pass the
  oldest held line before the read starts; the lines then arrive under the
  brief, the view kept. Default: so, until temper/qa's walk after landing
  says otherwise. Otherwise the card moves to the log's foot, or the brief
  draws only once the window reaches the start, as a chat's context does.

- **The agent-page keys after landing** (`task.md`, 2, `chat/conversation.md`, 2; the request
  `ui-agent-keys`): temper/qa reads back from Message with PgUp and PgDn
  on an agent's page and a goal's untracked page, while text streams and
  after the turn ends, wide and at 420 px. The check includes a screen per
  press, focus kept in Message, the kept place as words arrive, Latest,
  long briefs and results, the plan and held cards, and opening the amendment's
  editor in a short window. Default: the focused stylesheet and view-order
  tables; no new browser story. The keys and layout are checked after landing.

- **A PgUp while earlier lines are being read** (`chat/conversation.md`, 2; the request
  `ui-agent-keys`): at the log's head a press has no room to move and is
  not kept for the arriving lines. Reads chain from the log's measures,
  not the presses. Default: no queued press. Keeping a bounded page request
  and replaying it after the view is restored is a candidate in skein's page
  part; it needs per-log state and a bound on repeated presses, and does not
  itself shorten the read's wait.

- **An agent's page going to its end** (`chat/conversation.md`, 2; `../../draft/chat-reading.md`, 5): its
  log has no end request, so a person sending from far back stays where they read,
  and Latest reaches the end a page at a time. A chat's `to_end` and
  `Thread::latest` on a task's page would close both. Default: as it stands.
- **An agent's conversation drawn as a chat's** (2): the agent's
  page draws runs as a chat does, and keeps the run markers
  as work lines, which a chat hides (`README.md`).
  Default: as work; the chat's drawing is the candidate, the task
  pages'.
- **An added dependency** on a task's page (`goal.md`, 2, `task.md`, 2):
  `Dependency.added` says when and by whom a dependency was added.
  Default: a mark in its item of the dependency line, who added it and
  whether it keeps the task, which keeps the line's shape; when, in
  `ctl task show TASK` alone.
- **A message to a paused goal or agent's task** (`chat/README.md`, 1, *A person's pause*): the request
  `ui-pause-close` makes sending resume a chat alone. Default: words alone, which wait for
  Resume, since a goal's release lifts its whole subtree's hold; an agent's task that resumes
  on a message is the candidate.
- **An agent's task's page against its mockup** (2), each a default
  the user may change:
  - its dependencies, which the mockup draws on no agent's page: the
    dependency line under its brief;
  - a cut document's dependency line: no last item, since the
    document's `cut` does not say which list was cut; a cut of its own
    on the dependencies, as a plan row's, is the candidate (a jig
    change);
  - its calls: runs as a chat's, one line a run (`README.md`), a row a call,
    a result its row's outcome, which the mockup doesn't draw;
  - its plan: drawn while its tree holds a delegate or a held card,
    where the mockup draws none;
  - its acts are 4's.
- **A task's page's acts** (4), each a default the user may change:
  - an agent's task's Pause sent at once, as a chat's and a goal's, though that
    page acts on work the person may not be watching: Resume undoes it (`../README.md`, 3.6). A
    sheet on it is the candidate;
  - Close on a goal's and an agent's task's page alone; a procedure's Cancel, which
    no page draws, is reached from `temper ctl`;
  - the amendment's editor on the pages: the specification's words
    and, on a goal, its tracking and priority, the model choice being
    the choice line's; the rest of an amendment is `temper ctl
    task amend`'s. A draft whose task moved is discarded and opened again,
    not carried onto the new revision: the person rereads what
    changed.
- **A change's page against its mockup** (3), each a default the
  user may change:
  - "Its tasks"'s rows: `goal.md`, 2's plan rows, with their executor, state and
    spend, where the mockup draws each as a gate's row, its kind and
    its phase, and heads the region "made by the change procedure…";
  - a gate named "Gate N · kind", where the mockup names the landing
    rule that adds it, and "No other gate." with no "its landing rules
    add none", since the panel carries the gates, not the rules;
  - its step's line, its hold and its bounds, which the mockup does
    not draw;
- **Older ended delegates and a goal's older activity** (`goal.md`, 2, `task.md`, 3):
  a goal's tasks and a procedure's "Its tasks" read one page of their
  ended delegates, the first made, and say "More of its tasks are not
  shown." or "More of its ended tasks are not shown." past it; a goal's
  page reads its latest activity's page for what the person sent, not the
  older ones. Default: one page of each, since a procedure makes a handful
  and a goal's page is read for what is under way. Reading at a list's edge
  is the candidate, as an agent's task's conversation does (`chat/conversation.md`, 2): the
  ended page's `after` as the next query's
  start, and the activity's older pages by their `before`; with the
  view's words and tests and the client world's story, about two hours
  of work.
