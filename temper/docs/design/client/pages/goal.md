# A goal's page

## 1. The page

- **A goal's report** is the note `../../domain/agent/agents.md`, 3 shapes. `report::parse`
  in the client's domain, pure, reads a note as `Report { headline, paragraphs,
  left, risk, decided, at }`, a line's start alone making `Left:`, `Risk:` or
  `Decided:`; the read is `Read::Notes { scope: NoteScope::Task { project, task
  }, description: Some("report: "), after: None }`, the newest by `changed`
  whose description opens so, since jig's filter finds its words anywhere. Notes
  have no watch: a page reads again when the goal's `last_activity` moves. No
  note yet is a normal state, never an error.

## 2. A goal's page

A goal is a tracked agent's task (`README.md`). Its page is the greenfield mock's
`goal.html` (`../web/greenfield/README.md`), drawn by the view's `pages/goal.rs`,
where `pages/task.rs`'s `main` hands over for `Shape::Tracked`. An agent's
task's page (`task.md`, 2) and a change's or a person's (`task.md`, 3) are other pages, and
they are where a goal's task rows lead: the mock has no task page.
`temper ctl task show` keeps the facts this page doesn't say (`../ctl/results.md`,
1): what was asked whole, its dependencies, its activity, its budget and
authority.

The page is `.col.wide`, in the order below. Its words are exact. Its state,
`S`, is `goals::state` (`work.md`, 1, *Goals*), of its chip, whether the reader has
an open need on it, whether it has a report, and its direct delegates' total.

- **What it reads,** beside the task's document, its tree, its latest
  activity page and its ended delegates' page, as a procedure's page reads
  them (`task.md`, 3):
  - its report (1, *A goal's report*), read again when the document's
    `last_activity` moves;
  - its own task line, for its counts: `Read::Tasks` by `TaskQuery { project,
    requester, label, tracked, by number }`, all four from its document,
    following the live group's and then the ended group's cursor until a row
    is the goal, at most 8 pages, and read again when the document's
    `changed` or `last_activity` moves. Until it is read, and when no page
    held it, the counts are what the page holds: the delegates' total its
    chip's `live_delegates` and the ended page's rows, cancelled ones left
    out; done, the ended rows that ended done;
  - its needs: the inbox's `Inbox`, `Filter { project, goal: Some(task) }`,
    opened once the document says the task is tracked and running inside the
    goal's page, its cards the page's as Needs you's are (`../README.md`, 3.6).
- **The header,** `header.goal-head`:
  - `p.meta.mono`, items separated by `<span>·</span>`: `<b>TITLE</b>`; `S`'s
    word in its `state-*` class, as a goal's row says it (`README.md`); "6 of 9 tasks"
    when there are tasks; "31 of 40 spent", the budget's numbers as supplied,
    no currency; "since " and `made` in the clock form (`../view.md`, 6).
  - `<h1>`: the report's headline. With none: planning, or live with no tasks yet, "Planning from NAME:
    reading it and the designs it cites.", NAME the draft's file from the
    goal's words (below), or "Planning: reading what was asked." without one;
    closed, "Closed by you. What landed on the goal branch stays there.";
    landed, the result's first line, or "Landed."; failed, "Failed: " and the
    failure's reason; otherwise "N of M tasks done.", as a Goals row's (`README.md`).
  - `p.byline`: "Written by its agent at " and the note's `changed` in the
    clock form. No byline with no report.
  - `.acts`: "Draft" (`.btn.small`), when the goal's words name a draft and a
    chat: to that `Chat`. "Chat T2", when the requester is a task: to that
    chat. Neither otherwise. While live: "Amend", "Pause" or "Resume", "Close",
    `task.md`, 4's. Ended, none of the three.
- **Needs you bands,** one `section.band` named "Needs you" per open need of
  the goal, in the inbox's order, between the header and the report:
  `<p><span class=tag>Needs you · KIND</span>QUESTION</p>`, KIND and QUESTION
  as Needs you's card reads them (`needs-you.md`, 1): "a question", "a
  proposal", "held", "a person task", "funds". Then:
  - the page's first question whole, as Needs you's card draws it (`needs-you.md`, 1, *Whole*,
    *Cut*): its context, "What the agent saw" and its cut line, in `.ctx`. A later
    question on the page draws its QUESTION from the document's bounded words and
    "Answer", so that one page holds one whole question, as `../view.md`, 4 prices it;
  - the parsed choices of a question or a person task in words, in
    `.choices`, answering as Needs you does, with no Reply and no Later;
  - no choices: `.btn.primary` "Answer" for a question or a person task,
    "Open" otherwise, to `Inbox` filtered by this goal;
  - a funds card, with no "Add $10" and no "Stop the goal": its Release and
    Leave held as Needs you draws them (`../README.md`, 3.6), or "Open" when neither is
    offered;
  - an approval, a change under the goal waiting on a review on the forge
    (`../forge/README.md`, 4.1), KIND "an approval": its review link as `.btn.primary`
    where the card offers one, else "Open", to the change's `Task { task }`.
- **`.two`,** the report and the side, wrapping by `.two`'s own flex basis, not
  by a media rule.
- **`article.report`,** heading "Report" (`.section-title`):
  - each paragraph as `<p>`;
  - planning: `.typing`, three dots, a `Figure` named "Planning", under the
    words "Reading NAME and the designs it cites, to propose its tasks.", or
    "Reading what was asked, to propose its tasks." with no draft; no Left or
    Risk;
  - `<p><strong>Left:</strong> …</p>`, `<p><strong>Risk:</strong> …</p>`, when
    the report has them;
  - a report-less ended goal: the result's words, or a failure's reason, as
    paragraphs;
  - `p.byline`: "By its agent at TIME". The mock's "earlier reports" is not
    drawn: one note is revised in place.
- **`aside.side`,** its parts in order:
  - **Tasks** (`.section-title`), the goal's direct delegates alone: its tree's
    depth-1 rows and the ended page, merged by number, a delegate in both
    drawn once as the tree's; a delegate's own delegates aren't drawn; cancelled
    ones are left out.
    - `figure.bar` (`Role::Figure` in the browser, `Role::Image` in the terminal),
      named "N landed of M", a `<span>` per task in
      order, class `landed` for a landing, `running` for a running one,
      none otherwise. A *landing* is a delegate ended done with a change's
      result that landed (`ResultValue::Change { kind: 1, .. }`, as a delegation's card
      reads it, `../view.md`, 5); a person task's answer, an advisor's report
      or another result is done, and not landed.
    - Each task not landed, `p.task-line`, with class `running` when
      running: the title, a link to its `Task`, then its state: "done" and its end
      in the clock form; "running";
      "waits on TITLE" for one dependency, or "waits on task N" where the title
      is not among these rows; "waits on N tasks" for several; "waits on you"
      when an open need of the goal's is that task's; "waits on its resources";
      "held", in `state-needs`; "failed"; otherwise "waits".
    - `Role::Disclosure` "N landed" (`.link`, 13px, `margin-top: 6px`), folded,
      holding each landed task's `p.task-line`: its link, then its end in the
      clock form.
    - No tasks: `p.facts` "Proposing tasks from the draft…" planning, or "No
      tasks.".
    - While the ended page has more after it or was cut, "More of its tasks are
      not shown." as `.facts`.
  - **Landing,** `p.facts`: landed, "Landed at 01:12." in the clock form;
    closed, "Closed before it landed."; failed, "Failed before it landed.";
    live, holding a goal branch, "Lands whole once all nine are checked.", the
    delegates' total in words through nine, then a digit, or "Lands whole once
    all its tasks are checked." with no tasks yet; live with no branch, "Each
    task lands on its own once checked." A goal branch is a resource of the task
    whose path ends in the name `temper/<root>/g<root>`, `../../domain/forge/branches.md`,
    1's. The mock's "last updated from main" is not drawn: no read says it.
  - **How it runs,** `p.facts`: the way, "Plan, tasks, land whole" with a goal
    branch, "Task by task" without, then " · always asks you" when the
    goal’s words hold the exact line `Asks: always`, otherwise " · asks as the
    project says", by its classes, Away included (`../../domain/forge/attention.md`, 6).
    One pure domain function, `goal::always_asks(words) -> bool`,
    reads that line; a phrase inside a sentence does not count.
  - **Decided by the goal,** only with a report's `Decided:` line: `p.facts`,
    its items as written, joined by " · ".
- **`.sent`,** when the person has sent words: each of the goal's
  `Activity::Words` by the person in the latest activity page, `p.you`, oldest
  first, then the pending one, "Being sent". No "Its agent:" line: a goal's
  answer is its next report.
- **The composer,** `task.md`, 4's, last, under a `max-width: 720px` box.
- **The goal's words,** which Start goal writes (3) and the page reads back
  by a line's start, in `goal::draft_of(words) -> (Option<path>, Option<chat>)`,
  pure and the domain's: `Draft: PATH` and `Chat: T4`.
- **Where it is decided:** the domain's `pages/goal.rs` holds what the page
  reads and decides, pure where it can be: its counts, its headline's case, a
  task's line, its landing's case, `draft_of`. The view words and lays them out.
- **The terminal:** the view is shared and the page keeps its meaning. The
  regions print in order: the header, the needs, "Report", "Tasks", "Landing",
  "How it runs", "Decided by the goal", the composer. The bar prints by its
  name, "6 landed of 9"; the disclosure's rows print under it.
- **Not drawn,** the mock's: "earlier reports"; the goal's budget as money;
  reordering a goal; and every part of the page as it was ("Task details", the
  pagebar, "What was asked", "Result", "Plan" and its filter, dependency lines,
  "Latest activity", "Budget and authority", the model choice).
- **The look:** `../web/look.md`, *A goal's look*.
A plan on the other pages, an agent's task's (`task.md`, 2) and a change's or a person's
(`task.md`, 3), keeps its tree of rows, by these rules:

- **The header,** on a change's or a person's page: "Task details", then the
  task's number, its executor and its project; its state follows the title,
  then who asked.
- **A task's state,** on those pages, in the mockup's words, in lower case:
  - in the header, from what the engine computes the task waits on
    (`Task.waits_on`): on its dependencies, "waiting on task M" for
    one, "waiting on N tasks" for several, N their supplied `count`;
    on its delegates, "waiting on N delegates" ("1 delegate"), N its
    chip's `live_delegates`; on a party, "waiting on an answer from
    you" when the person is that party or holds that role, "waiting
    on an answer from person N" or "from role N" otherwise; on news,
    "waiting on news". Waiting on nothing, it says what a row would;
  - on a plan row, which has no `waits_on`, from its chip's owner phase
    (jig's `client/domain/objects.md`, section 3): waiting on its
    dependencies (the phase's `dependencies`, those not yet ended),
    "waiting on task M" on one, "waiting on N tasks" on several, N
    those listed, and "waiting on more than N tasks" when the list was
    cut (`more`); waiting on no dependency but on its holdings
    (`holds`), "waiting on its resources";
  - running, in both, with the work its owner observed
    (`observation`): "running · writing", or "running · a tool ·
    NAME", the tool's name as supplied; "running" alone when no
    observation is supplied, which jig allows, or for any other work;
  - ended, with its owner's ending: "Completed", "Failed" or
    "Cancelled", as `../view.md`, 5 words them;
  - otherwise, jig's word for its phase in lower case: "held",
    "retrying", "waiting for news". A held task's card and a
    delegation's card keep theirs (`../README.md`, 3.6, `../view.md`, 5).

  Each carries its phase's status token: a waiting state waiting
  (`STATUS_WAITING`, the mockup's amber), a cancelled one quiet, a
  running one active, a completed one done, a failed one held.
- **A plan's rows' order,** a tree of its delegates: the roots by number, the
  order they were made.
  An ended delegate is drawn before the first live root with a higher
  number, or after the last live root when none has one; a live root's
  own delegates stay under it, in the tree's order. A delegate in both
  the tree and the page, one that ended after its tree was delivered,
  is drawn once, as the tree's row. An ended delegate's own delegates
  are not read: its page shows them (`../README.md`, section 5).
- **An ended row** is a plan row (below) whose state is its ending,
  "Completed", "Failed" or "Cancelled", with its spend and with no
  dependency line or held card; its "Task N" link is muted, as the
  mockup's done row.
- **A plan row's cells:** the "Task N" link and the task's title; its
  executor; its state; its spend, "N spent", what the task spent
  itself (its chip's `budget.spent`; not the row's `spent`, which adds
  what the ended tasks it funded spent), in the deployment's unit, a
  whole number. The cells have no heads, since tree items are not a
  grid's cells; the mockup heads its columns "Task", "Kind", "Model",
  "On", "State" and "Spent" (a default the user may change, `../README.md`, section 5
  ). A procedure's part follows on a line of its own, as before. At
  phone width the name is on a first line, and the executor, state
  and spend on a second; no cell is hidden.
- **The dependency line:** under a row whose task has dependencies
  (its `dependencies`), a list named "Dependencies of task N", one item
  per dependency in the supplied order, keyed by its task as an object,
  holding its "Task M" link and its state: a live dependency's as
  above, an ended one's supplied status ("Completed", "Failed",
  "Cancelled"), and none when none was supplied. When the row's list
  was cut (its `cut`), a last item, with no key: "More dependencies
  not shown." The browser shows the list's name before its items. In a
  row's tree item come the row, its dependency line, its held task's
  card (`../README.md`, 3.6), then its children.
- **An added dependency's mark:** in its item of the dependency line,
  after its link and its state, read only, for a dependency an
  amendment added (the row's chip's `dependencies`, jig's
  `Dependency.added`, matched by number): a paragraph, "Added by you"
  when the person signed in added it, "Added by party P", "Added by
  task M" or "Added by the deployment" otherwise; then, while it keeps
  the row's task (jig's `keeps`, jig's `domain/tasks/lifecycle.md`, 8), " ·
  keeps task N from ending". No time is drawn: `ctl task show TASK`
  prints when (`../ctl/results.md`, 1). A dependency the facts
  don't name, or one not added, has no mark; the line keeps its shape
  (`../README.md`, section 5).

## 3. Starting a goal

The sheet, the mock's `startSheet` (`greenfield/`), `pages/start_goal.rs` in the
view: a `Role::Dialog` over the page (`dialog.sheet`, `::backdrop`), opened by
`Act::OpenStart { draft: Option<Box<[u8]>>, chat: Option<u64> }`, closed by
Cancel or Escape. A domain struct on `Pages` holds its fields, so that any page
can open it. `draft` is the draft's path in the repository
(`docs/design/draft/away-mode.md`), and its FILE the path's last segment.

- **A form,** in order:
  - `h2` "Start a goal from FILE", then `.muted` 14px "The goal takes the draft
    as it is now. Later edits reach it through Amend." With no draft given: `h2`
    "Start a goal", and a `.field` "Draft", a text field, placeholder
    "docs/design/draft/name.md", which Start requires;
  - `.field` "Name": `input.text`, FILE without `.md`, following the Draft field
    until the person edits it;
  - `fieldset`, legend "How it runs": a `ChoiceGroup` of `label.option` per way
    (the project's tracked templates in number order, `work.md`, 1, *Ways*):
    `<b>WAY</b><small>TEXT</small>`. The default adds " PROJECT’s default." to
    its text and is chosen: the way kind 7 names (`settings.md`, 2). After them, `.link` 13px muted
    "The project’s ways of working", to `Settings { project }`;
  - `fieldset`, legend "When it needs you": a `ChoiceGroup`, drawn as the ways:
    "As the project says", chosen by default, small "Decides what the
    project’s settings let agents decide, Away included, and tells you; asks
    you the rest."; "Always ask", small "Asks you every question it can’t
    answer from the plan, even while you’re away." A goal of either is judged
    by the project's classes, "Always ask" lifting every class below
    `ask_away` to it (`../../domain/forge/attention.md`, 6). `Act::StartAsks { always: bool }` changes this choice, as
    `Act::StartWay` changes the way. Both choices are disabled while a request
    is out.
  - `.foot`: `.btn.quiet` "Cancel", `.btn.primary` "Start goal", disabled while
    Name or Draft is empty or a request is out.
- **Not drawn:** "Budget": nothing stands behind it yet (`../README.md`, section 5).
- **With nothing to fund it:** while the project's first creating bound gives no
  budget (`creating.within.budget` 0) and the way's budget isn't a fixed amount,
  `.sheet-error` above the foot, and "Start goal" is disabled; `Me` is watched, so
  it clears when a chat closes. While the party's pool has reservations: "Nothing
  is left in your pool to fund a goal: your open chats hold it. Close one you’ve
  finished with, and you can start this."; with none, "Nothing is left in your pool
  to fund a goal." Funding's rules are as ever. Stock reaches it: a role 2 member's
  pool is one chat's spend, so one open chat takes it, and an owner's is eighty's
  (`../../../help/budget.md`).
- **Start** calls `shapes::set_goal_from(me, project, number, GoalDraft { words,
  executor, budget, deadline: None, priority: None, llm: None }, "Started from
  its draft")`: `executor` the template's charter, else `Agent { charter: 1 }`;
  `budget` the first creating bound's (jig's `client/domain/contract/documents.md`, 3).
- **The goal's words,** which the goal's page reads back (2):

  ```
  NAME

  Build docs/design/draft/FILE.
  Draft: docs/design/draft/FILE
  Chat: T4
  ```

  The first line is the task's title, as jig takes it. `Chat:` only when a chat
  opened the sheet, whose document's snapshot then stands in the "Build" line's
  place (`README.md`). "Always ask" appends the exact line `Asks: always`;
  "As the project says" adds no asking line. Nothing stores the choice
  apart from these words.
- **Its ends:** `Judged::Create`: sent, and on its answer to the goal's page, which
  shows planning. `Judged::Propose`: sent, the sheet closes, toast "Proposed to
  WHOM: it starts once accepted." `Judged::Refused`, `NotOffered` or a refused
  answer: `.sheet-error` in the sheet, in jig's words, and it stays open.
- **Where it opens:** on a chat, its "Start goal…" (`README.md`); on Goals, the `.link`
  "Start one from a draft" at the section head (`work.md`, 1, *Goals*), with no draft, shown
  while the project has a tracked template: `Act::OpenStart { draft: None, chat:
  None }`, a button in the link's look, as it moves nowhere, for the project in the
  address; on Work, a landed draft's card (*From a landed draft*, below); a draft
  without a chat but a landed draft's, or a chat without a draft, opens nothing. A goal
  started this way asks its person as a chat's does (`../../domain/agent/agents.md`, 2); its
  words have no `Chat:` line. The rail's "Goal" stays what `../view.md`, 3 has it, a new chat for
  a new draft.
- **From a landed draft** (`work.md`, 1, *Work*;
  `../../domain/forge/drafts.md`): Work's card opens the sheet for the
  project in view with `draft` the row's path and no chat, the sheet holding
  the row's repository, its owner and name, and its path. Its form is the one
  with a draft given, "Start a goal from FILE", Name FILE without `.md`; its
  words are the path's form, "Build PATH." and `Draft: PATH`, with no `Chat:`
  line whatever chat the row names, since the goal builds the file as it
  landed. The card's "Start a goal: WAY" opens it with WAY chosen and presses
  "Start goal" at once, which sends only where "Start goal" is enabled; the
  sheet then shows its request out, and stays open only to show what the sheet
  shows of a refusal or an empty pool. On `Judged::Create` the client sends
  `SetDraft { project, repository: { owner, name }, path, change: started {
  goal } }` (`../forge/records.md`), keyed and at once, before or while the
  goal's page opens; a refusal or a loss of it is drawn nowhere, the goal
  having started, as a `Dismiss` of `GoalFrom` refused is drawn nowhere
  (`../forge/records.md`, *`Dismiss`*), and the draft stays listed. On `Judged::Propose` nothing is sent, and the draft stays listed
  (`../../domain/forge/drafts.md`, 4).
- **The chat's thread:** a person's Create leaves no turn, so its "Started goal"
  line is drawn from the fact `GoalFrom` (`README.md`).
- **The look:** `../web/look.md`, *A goal's look*.

## 4. Open questions

- **The person-started question flow after landing** (`README.md`, `goal.md`, 2; the
  requests `ui-goal-asks` and `ui-goal-asks-2`): check a question past the
  document's text bound, its context, evidence and choice buttons under
  Needs you and on the goal's band; starting from Goals by a typed Draft
  path; the empty-pool and open-chat funding explanations; and person and
  advisor tasks finishing without a landing. Browser, walk and render
  checks follow landing. Default: focused sheet-to-answer stories from a
  chat and Goals, and shared domain and view tables, with no new browser test.
- **More than one whole question on a goal** (`README.md`, `../view.md`, 4, `goal.md`, 2): the page's
  price includes one `long_text` question. Default: the first whole,
  later bands' bounded first line and Answer to Needs you; extending the
  price or drawing a question's whole words on demand is later work.
- **A goal's model, effort and account** (`README.md`): the goal page has no such
  line. A place for those words and their amendment remains to be designed.
  Default: no line; a closed agent's task uses its taskbar (`task.md`, 2).
- **The rest of a goal's authority:** its tools, grants, executor kinds
  and note scopes, which jig supplies as codes and patterns, not
  words. Default: not on the goal's page.
- **A goal's page against its mock** (2), each a default the user may change:
  - task rows are links, the one departure: a held delegate's Release is on its
    own page;
  - no "earlier reports" and no reorder; no budget field and no "Add $10": budgets
    matter least;
  - no drain: Pause is jig's Stop, so its toast says running tasks stop, where the
    mock's say they finish. jig's Drain and a party's `Hold` are not built;
  - "Close" is `Ask::Cancel`, where the mock says "Stop": Pause, Resume and Close are
    one vocabulary across a chat, a goal and an agent's task.
- **A goal's own line** (2): no key selects one task, so a goal's counts follow
  `Read::Tasks` by its project, requester, label and tracked for at most 8 pages,
  and a party with more goals than that sees what the page holds. Default: so,
  since the request `ui-goal-page` keeps jig's part to `made`; a key selecting one
  task is the candidate (a jig change).
