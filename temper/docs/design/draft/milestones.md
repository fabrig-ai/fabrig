# Milestones

Draft, 2026-11-15, from a session with the user that day. A milestone is
a set of goals, as a goal is a set of changes. The user: "a large draft
is a milestone, with several goals to reach it, the same as 'a goal is a
set of changes'. So in the UI I'd see cards for 'change' (plain), 'goal'
(several changes), 'milestone' (several goals)". Each of its goals lands
on its own: "each goal in the milestone should independently land on main
when done. The milestone completes when everything is landed. Otherwise
we accumulate too much drift, and also landing by goal makes it possible
to start QA'ing what's done". temper offers the primitive and nothing
around it: "we offer the milestone primitive and every user can use it
as fit". Making milestones from a draft or a list is the person's own
plumbing, through `temper ctl`. This first pass designs what the pages
show, and the contracts underneath that they need (sections 7 and 8).
How a person makes a milestone in the client comes later (section 11).
It adds to `surfaces.md` and to the pages' designs, and wins where they
differ until its plans fold it in.

## 1. In one page

- **Three sizes of work, in the forge's words:** a change is a pull
  request, a goal is an issue, and a milestone groups goals as the
  forge's milestones group issues (section 2).
- **A milestone is a primitive:** a name, the goals in it, and done once
  every one of them is done. temper doesn't know where it came from
  (section 3).
- **Each goal lands on its own,** by its own way, as soon as it's done.
  Nothing waits for the milestone, and nothing lands as it.
- **A goal may wait on others,** in a milestone or not: jig's
  dependencies, given at its start. A goal that waits says what it waits
  on (section 4).
- **No new page in the rail.** Home's Goals and Work's In flight draw a
  milestone as one row or card, with a strip of its goals. Needs you
  names it on its goals' cards. Its page borrows the goal page's layout
  (section 5).
- **Made through `temper ctl`** for now, by a person or their scripts
  (section 6).

## 2. A change, a goal, a milestone

| | A change | A goal | A milestone |
|---|---|---|---|
| Is | one piece of work: one pull request | a set of changes, which its agent plans | a set of goals |
| Made by | a chat, the rail's "Change", a goal | a chat's or a landed draft, a finding, `temper ctl goal start` | `temper ctl`, by a person or their scripts |
| Runs | its producer, CI, its reviews, its repairs, the queue | a coordinator that wakes for judgement alone (`goal-runs-itself.md`, section 1) | nothing of its own: its goals run |
| Lands | on its own | each change on its own, or whole from its branch, by its way | each goal on its own, by that goal's way |
| Done | once landed | once it has ended done (section 3) | once no more goals will join and every goal in it is done |
| On the forge | a pull request | an issue (`../domain/forge/issues.md`) | nothing yet: its goals' issues alone (section 11) |

- **Each size is optional.** A project that makes no milestones never
  sees one, since no part is drawn empty (`surfaces.md`, section 12). A
  project with milestones keeps plain goals and plain changes: a chat
  still asks for one change, a draft still starts one goal, and a
  finding is still routed to a goal.
- **Each shows under what it belongs to,** as Work already places a
  change under its goal (`../client/pages/work.md`, 1, *What goes
  where*): a change under its goal, a goal under its milestone, and
  anything that belongs to nothing on its own.
- **On the spectrum** (`whole-story.md`, section 3), a milestone is the
  step after At scale: "big work is a milestone, and each of its goals
  lands as soon as it's done".

## 3. A milestone

- **What it holds:**
  - *its project* and *its number,* as a task's;
  - *its name,* for a person, 1 to 60 characters: "Store shape";
  - *its words,* optional, what it is for, which its page shows: "The
    store's new shape, from its draft";
  - *its goals,* in the order they joined;
  - *open or whole:* open while more goals may join, whole once its
    maker says every goal is in. Whole can't be undone. A goal joins only
    while the milestone is open and live;
  - *its maker,* the party that made it, whom its goals ask (below);
  - *its budget,* given when it is made, which funds its goals as a
    goal's funds its changes (section 7).
- **A goal joins at its start,** and belongs to one milestone at most. It
  asks, decides, lands and reports as any goal of its way does. Its
  questions go to the milestone's maker, as a goal a person started asks
  that person (`../domain/agent/agents.md`, section 2), never to the
  milestone, which has no agent to judge them. Its Decided for you rows
  reach the person as any goal's.
- **A goal is done** once it has ended done, jig's success (jig's
  `domain/tasks/lifecycle.md`, section 7). By temper's goal ways, its
  work is then on the default branch, or it had nothing to land: a goal
  of "Task by task" ends done once its changes have landed, and one of
  "Plan, tasks, land whole" once its branch has landed or held nothing
  new (`../domain/forge/branches.md`, section 2). The pages say "done"
  for a goal, and "landed" only beside a landing they read, with its
  time.
- **Its order is its goals' dependencies,** not the milestone's. A goal
  made to wait on another waits until that one is done; one made to wait
  on a person task waits for the person's answer, which reaches them as
  a card like any. Neither is a milestone's: a plain goal may wait on
  another just the same. Dependencies give a partial order, so goals with
  nothing between them run side by side.
- **Its outcome,** once whole and every goal in it has ended: *complete*
  if every goal is done, *ended* if some failed. Closed goals are left
  out, as a goal's count leaves out its cancelled tasks
  (`../client/pages/work.md`, 1, *Goals*), so a goal the person closed
  to replace it never keeps the milestone from completing. *Closed* is
  the milestone's own end when the person closes it. A milestone
  completes when its last goal ends or its maker says whole, whichever
  comes later: that time is its completion, distinct from its last
  landing.
- **A goal that fails or is closed** doesn't stop the others. The goals
  waiting on it are held (jig's `domain/tasks/lifecycle.md`, section 6),
  each a held card in Needs you. Release alone holds it again, since the
  dependency still stands: to go on, the person or the maker removes or
  replaces the dependency and releases it, or closes it (jig's
  `client/domain/objects.md`, section 5).
- **Pause, Resume and Close** are a task's (`../client/pages/task.md`,
  section 4). Pause stops every goal under the milestone, and those made
  later, at once, cancelling their runs (jig's
  `domain/tasks/lifecycle.md`, section 10); Resume lifts it. Close
  cancels every goal in it that hasn't ended. What landed stays.

## 4. States and words

- **A goal that waits,** in a milestone or not, gains two states in
  `goals::state` (`../client/pages/work.md`, 1, *Goals*), after needs
  and paused and before planning; both count as under way, in the Goals
  filter's "Under way" and its intro's count:

| State | When | Word |
|---|---|---|
| waits | live, waiting on a dependency that hasn't ended | "waits on TITLE" where the title is read, else "waits on task N", "waits on 3 tasks"; "waits for you" where it is a person task addressed to the reader |
| waits for room | live, its dependencies done, waiting on a hold of a capacity the project names (section 11) | "waits for room"; "waits for its resources" for any other hold |

  A goal that waits has no report yet, so its headline is its wait in a
  sentence, "Waits on S6.", on every page that draws its row.
- **A milestone's state,** one pure function, the first that holds:

| State | When | Word |
|---|---|---|
| closed | the person closed it | "closed" |
| complete | whole, every goal ended, every one not closed done | "complete" |
| ended | whole, every goal ended, some failed | "ended" |
| needs | a need of the reader's is under it | "needs you" |
| paused | paused by the person | "paused by you" |
| running | a goal in it is live and started | "running" |
| waits | goals in it are live and none has started | "waits" |
| open | open, and no goal in it is live | "open" |

  A whole milestone with no goals is complete, with no landing; an open
  one with none says "No goals yet."
- **Its count,** "7 / 8": its goals done, then its goals, the closed
  ones left out of both.
- **Its headline,** composed by code: the count in words, "7 of 8 goals
  done.", then the headline of its newest active goal, its report's or
  its row's fallback (`../client/pages/work.md`, 1, *Goals*), after that
  goal's title, and ", and one more running" where there are more.
  Ended: "7 of 8 goals done; S5 failed." Complete: "All 8 goals done."
  Open with nothing live: "6 goals done so far; more may join."
- **Its strip:** one segment per goal it counts, in the order they
  joined, its id under it where the goal's title begins with one: done
  in ink, live and started in grey, needing the person in the signal's
  colour, waiting outlined, failed struck. It is named "7 done of 8".

## 5. The pages

### 5.1 Home

- **Goals** (`../client/pages/home.md`, 1, *Goals*) becomes a grouped
  list: a plain goal is a row, as today, and a milestone is one row for
  all its goals, which have no rows of their own. A milestone's row shows
  while it is live, open or not, and for seven days after it ends.
  - *Its order:* live rows first, by last activity, a milestone's being
    the newest of its goals' and its own, the lower task number first on
    a tie; then ended rows, newest end first.
  - *Its limit:* the larger of 5 and the live rows, counted after
    grouping.
  - *Its head:* "All N goals", N the goals read, each counted once,
    milestones not counted, "N+" where the read is cut; to Goals, which
    counts the same.
  - *The region* is drawn in Chats' place while the project has a goal
    or a milestone (`Domain::has_goals` reading both).
- **A milestone's row:** its top line, the tag "milestone" and its name
  at the left, "7 / 8 done · " and its state's word at the right; then
  its strip; then its headline.

  ```
  [milestone] Store shape                        7 / 8 done · running
  ■■■■■■▒■
  S1 S2 S3 S4 S5 S6 S7 S8
  7 of 8 goals done. S7: 5 of its 7 changes landed; the other 2 are building.
  ```

- **Needs you:** a card's from line keeps what it has, and names the
  milestone and the goal before the card's own title (5.3).
- **Decided for you:** a row of a goal in a milestone reads "Store shape
  › TITLE · 01:05", with its project's chip before it on Home on All
  projects.
- **The brief:** the status agent names a goal by its milestone where it
  has one, "Store shape is complete: its last goal landed at 21:40.",
  from the facts section 8 adds to its input.
- **The headline** doesn't change: it still counts changes and needs.

### 5.2 Work

- **In flight:** each live milestone is one card, placed once, newest
  activity first by Home's key. What goes where becomes recursive
  (`../client/pages/work.md`, 1, *What goes where*): a goal in a
  milestone is placed on its milestone's card alone, and a change of
  that goal under the goal's line there, so every live goal and change is
  still placed exactly once.
  - *its from line:* the tag "milestone"; its name, a link to its page;
    "8 goals · whole" or "6 goals so far"; and its spend (section 7);
  - *its headline and its strip,* as on Home;
  - *a line per goal started and live,* in the order they joined: its
    title, a link to its goal, its state's word and its count, "5 / 7",
    with its changes on their way under it, as a goal's card has them;
  - *the goals waiting,* in a disclosure under them, "3 wait", each a
    line with what it waits on;
  - *the goals ended,* folded into one line with their counts, "6 done ·
    1 failed", opening to their lines.

  ```
  [milestone] Store shape · 8 goals · whole · 212 spent of 400
  7 of 8 goals done. S7: 5 of its 7 changes landed; the other 2 are building.
  ■■■■■■▒■
  ▸ S7 history in task logs                           running · 5 / 7
      · Move the turns' reader to the task log            queued 2nd
  ✓ 7 done, the last S8 at Sat 21:40
  ```

- **Landed this week** keeps its rows, its order, its counts, its commits
  and sign-offs as they are: a goal in a milestone is its row as any
  goal's is, with " · in Store shape" in the byline's style. A
  milestone that completed this week, by its completion's time, makes
  that byline " · in Store shape, now complete" on the row of its last
  landing, where that landing is in the week. A milestone is never a row
  of its own.
- **Its head:** a milestone card counts once in "in flight".
- **Its filters:** none of their own. Under Goals, a milestone is a group
  holding its goals, and a goal the filter matches keeps its milestone
  around it; the filter counts goals, never milestones.

### 5.3 Needs you

- **No new kind.** The cards are today's: a goal's questions, a held goal,
  a person task, an approval.
- **The from line** (`../client/pages/needs-you.md`, 1) keeps the
  project's chip on the unfiltered page, then the milestone and the goal
  a card comes from, then the card's own title, link, kind and age, with
  the proposal's and the person task's own rules unchanged: "fabrig ·
  Store shape › S7 history in task logs · a question · 16 min".
- **Filtered by milestone:** the inbox's goal filter takes a root
  (jig's `client/domain/pages.md`, section 3), and the milestone is its
  goals' root, so the filter holds every need under it. Its menu names
  it "Milestone · Store shape". The forge's approvals join by the same
  ancestry: an approval of a change under a goal in the milestone is in.
- **A held goal** waiting on one that failed or was closed reads as any
  held task does, its words saying which and what releasing needs: "S6
  is held: S5, which it waits on, failed. Drop or replace that
  dependency, then release it."

### 5.4 A milestone's page

`Task { task }` of the milestone, a page shape of its own borrowing the
goal page's header and two columns (`../client/pages/goal.md`, section
2), dispatched by the milestone's identity (section 7):

- **The header:** "NAME · STATE · 7 of 8 goals done · 212 spent of 400 ·
  since Mon 09:12"; its headline as the level-1 heading; and its acts
  while live, Pause or Resume, and Close.
- **Needs you bands:** every need under it, as a goal's page draws its
  own, the first question whole and the rest by their first line.
- **Goals,** in the report's place: a row per goal in the order they
  joined, its title a link to its page, its state's word, and its
  headline. A waiting goal says what it waits on. Ended goals fold into
  "N done", "N failed" and "N closed", as a goal's page folds its landed
  tasks, read from its ended goals even after the milestone has ended.
- **The side:** its strip; its words, if it has any; "Landing: each goal
  lands on its own, once done."; "Open: more goals may join." or "Whole:
  all its goals are in."; and "Made by NAME".
- **No report byline, no composer and no Chat link:** a milestone runs
  no agent, so words go to one of its goals.

### 5.5 A goal in a milestone

A goal's page (`../client/pages/goal.md`, section 2). Where its requester
is a milestone, the header's act is "in Store shape", a link to the
milestone's page, in place of the "Chat T2" a task requester gives
today; only a chat requester gives "Chat TN". While it waits, its
headline is its wait, "Waits on S6.", by section 4's rule.

### 5.6 The phone and the terminal

- **On a phone,** Home's row keeps its top line and its strip.
- **In the terminal,** the same parts print as lines, the strip as
  "■■■■■■▒■ 7 done of 8".

## 6. Made through `temper ctl`

The first pass makes milestones through `temper ctl` alone, by a person
or their scripts:

- `milestone new --project P NAME`, with `--words TEXT` and a budget as
  `goal start` takes one (`--budget N` or `--as-allowed`), prints the
  milestone's number;
- `goal start --milestone M` starts a goal in the milestone, as `goal
  start` starts any goal;
- `goal start --after TASK`, on any goal, in a milestone or not, makes it
  wait on that task, a goal or a person task, once for each. A task
  already done adds no wait; one that failed or was cancelled refuses
  the start, naming how it ended; a task ending as the start is admitted
  is judged at admission, never by an earlier read;
- `goal start --no-deadline`, where the template and the milestone's own
  bounds allow it, since a goal that waits counts its deadline from when
  it was made;
- `milestone whole M` says every goal is in;
- `milestone list --project P` and `milestone show M` print them as the
  pages draw them;
- Pause, Resume and Close are `task`'s, as for any task.

A script that reads a draft makes its milestone, then every goal in order
with its `--after`s, then says whole, and jig starts each goal as what it
waits on is done. Whoever runs it starts the goals, with the authority
they hold, so the approval is theirs. Every goal made at once reserves
its budget at once (section 7), so the milestone's budget must hold them
all.

## 7. What it is underneath

- **A tracked task of its own identity:** a milestone is a tracked task
  under a template and label of its own, so that its tree, its root, its
  pool, Pause and Close are jig's. Nested tracking needs nothing new:
  any task may be tracked, and a goal under it keeps its own goal
  identity (jig's `domain/model.md`, 6.2). Its identity keeps it out of
  every goal list, count and way (`../client/pages/work.md`, 1, *Ways*),
  dispatches its page (5.4), and keeps it from a forge issue of its own,
  while its goals keep theirs. It takes one of jig's live tracked tasks,
  as a goal does (jig's `domain/tasks/README.md`, section 6).
- **No model.** Its executor is a procedure of temper's (jig's
  `domain/tasks/lifecycle.md`, section 3) that does nothing but wait. It
  ends once its maker has said whole and every goal in it has ended,
  done with the outcome of section 3, and so never ends early while a
  script is still adding goals.
- **Its goals are its delegates,** made by a party through `temper ctl`:
  a new act, since a person's `Create` makes roots alone (jig's
  `domain/tasks/README.md`, section 4). The act makes a goal beneath a
  milestone the party may act on, admitted against both the party's
  authority and the milestone's remaining delegation and budget; the
  milestone is the goal's requester and funder, and its maker is the
  goal's person, whom it asks (section 3). A goal's `--after` may name
  any live task the party may reach, a sibling or not.
- **Funding:** the milestone's budget is carved from its maker's, as a
  goal's is; each goal's is carved from the milestone's when the goal is
  made, waiting or not (jig's `domain/authority.md`, section 7). Its
  spend is what it and its goals have spent, each task counted once:
  its own, its ended goals' settled spend, and its live goals' own, never
  their reservations.
- **It lands nothing:** its goals land by their ways, and the milestone
  holds no branch.

## 8. Reads

What the pages read beside a task's own reads, each new where it says
so:

- **The milestones:** a task query of the project by the milestone's
  label, its live group watched and its ended group read by when each
  ended, as Goals reads goals.
- **A milestone's facts,** new: its name, words, maker, open or whole
  and when it was made whole, as its task's document carries them.
- **Its goals:** its live delegates and its ended ones, merged in the
  order they joined, the ended page read again on a change and after
  the milestone has ended; each goal's report as Goals reads it.
- **A goal's milestone:** for a goal in any list, the milestone it
  belongs to, new, so that a row or a card can be grouped and a from
  line named without reading the tree.
- **Its totals,** new: its goals done, failed and closed, its last
  landing and its completion, and its spend as section 7 sums it, so
  that no page claims complete from a partial list. Until they are read,
  a page says what it holds and never "complete".
- **The status agent's input** gains the live and recently ended
  milestones with these facts (`../domain/forge/attention.md`, section
  3).

## 9. Out

- Making a milestone from a draft in temper: the person's own plumbing,
  through `temper ctl`.
- An agent that plans a milestone's goals, one level up from a goal's
  coordinator.
- Milestones across projects, and milestones within milestones.
- Due dates and calendars: a goal starts when what it waits on is done,
  never on a date.
- Drawing the dependencies as a graph: the pages draw order, not edges.

## 10. Reasons

- **Why "milestone":** it is the user's word, and the forge's. GitHub's
  and Forgejo's milestones group issues and count those done, and
  temper's goal is an issue on the forge (`../domain/forge/issues.md`),
  so a milestone groups goals as the forge's groups issues.
- **Why each goal lands on its own:** the user's reason, quoted above:
  less drift, and what landed can be checked at once.
- **Why "done" and not "landed" in counts:** a goal ends done with
  nothing to land as well as after landing, so a count of landings would
  have to read the forge, while jig's ending is what dependencies wait
  on. The pages say "landed" where a landing is read.
- **Why no draft in the primitive:** how a plan becomes goals differs
  from one team to another, and a script through `temper ctl` serves
  each. temper keeps the mechanism, and the person keeps the plumbing.
- **Why jig's dependencies:** they order any goal, in a milestone or not,
  so a milestone adds grouping and nothing else.

## 11. Open questions

- **Making a milestone in the client,** from a landed draft's card or a
  chat: its sheet, and who approves its goals. Default: `temper ctl`
  alone (section 6).
- **A milestone on the forge:** a forge milestone holding its goals'
  issues. Default: none; the milestone has no issue, its goals keep
  theirs.
- **A goal joining after its start,** or moving between milestones.
  Default: neither; a goal joins at its start alone, and jig's Move,
  when built, refuses to move a goal into or out of a milestone.
- **A waiting goal's deadline** (section 6): a deadline counts from when
  the goal was made, waiting included, and a delegate's can't outlive its
  milestone's. Default: `--no-deadline` at start where allowed, else a
  deadline long enough for the wait.
- **Room:** a project's bound on goals in flight, as a capacity hold
  every goal takes, which "waits for room" names. Default: none; jig's
  admission limits and the budgets bound goals as today, and the workers
  bound runs.
- **The word in the design:** jig's `Milestone` record is a task's phase
  change (jig's `domain/tasks/lifecycle.md`, 12.4), and a goal's issue
  posts its "milestones" as comments (`../domain/forge/issues.md`).
  Neither is the person's milestone. Default: both keep their names,
  being internal, and a later move renames them if they confuse.
- **`temper ctl`'s words** (section 6). Default: as written.
