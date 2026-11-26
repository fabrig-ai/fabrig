# Milestones in `temper ctl`

Provisional, 2026-11-16, from `../../draft/milestones.md`, section 6. A
person or their scripts make milestones through `temper ctl` alone, for
now (`../../domain/milestones.md`). A script that reads a draft makes
its milestone, then every goal in order with its `--after`s, then says
whole, and jig starts each goal as what it waits on is done. Whoever runs
it starts the goals, with the authority they hold, so the approval is
theirs. The tools are records of the one table (`tools.md`, section 1),
the acts keyed as `goal start` is (`requests.md`, section 1), their text
and JSON as `results.md` says.

## 1. In one page

| Tool | What it does | jig's | K |
|---|---|---|---|
| `milestone new --project PROJECT [--words TEXT] [--priority PRIORITY] [--budget AMOUNT \| --as-allowed] [--deadline TIME] NAME` | makes a milestone from template 8 (2) | `shapes`' fill of template 8 with the gathering's executor, a `Create` | yes |
| `goal start ... [--milestone TASK] [--after TASK]... [--no-deadline] TEXT` | starts a goal, in a milestone or not, waiting on tasks or not (3) | a `Delegate` beneath the milestone, or a `Create` | yes |
| `milestone whole TASK` | says every goal is in (4) | `Conclude` | yes |
| `milestone list --project PROJECT [--ended] [--after CURSOR]` | one group of the project's milestones (4) | `tasks{query}` by template 8's label | |
| `milestone show TASK` | a milestone's facts, totals and goals (4) | `Read::Task`, `tasks{query}` by parent, the forge's `Milestone` | |

Pause, Resume and Close are `task`'s, as for any task.

## 2. `milestone new`

- **NAME** is the milestone's name, 1 to 60 characters on one line
  (`../../domain/milestones.md`, section 3); another is usage, exit 1,
  nothing sent: `milestone new: NAME is 1 to 60 characters on one line`.
  **`--words TEXT`**, optional, its words, TEXT a string or `-` for
  stdin, as `goal start`'s TEXT is. The spec's words are NAME, or NAME, a
  blank line and TEXT.
- **The rest is `goal start`'s** (`areas.md`, area 2): template 8's
  required controls collected before usage, its budget `--budget` or
  `--as-allowed` (template 8's is given, so one of them is required:
  `template 8 needs: --budget AMOUNT or --as-allowed; help milestone
  new`), `--deadline TIME` optional (template 8 has none), `--priority`
  replacing its priority, a budget or deadline past the offered ceiling
  diagnosed as `goal start`'s is, and the fill and authority's or
  proposal's judgement jig's.
- **Its executor** is the gathering's, `Procedure { connector: RECURRING,
  code: GATHERING }`, which the tool fills, as `role start` fills charter
  13: no `--charter`.
- **It prints** `done: task N, milestone`, or `proposed: PROPOSAL, to
  RECIPIENT`, as `goal start` prints; under `--json`, the record
  `milestone_create`.

## 3. `goal start`'s flags

- **`--milestone TASK`** starts the goal in that milestone: it reads
  the task's document first and refuses, exit 1, nothing sent, one that
  is not a milestone of the project (`task N is not a milestone of
  project P`) or one already whole (`milestone N is whole since TIME`);
  then fills template 2 as without it, its authority narrowed to what
  the milestone has left (`../../domain/milestones.md`, section 4), its
  tasks from the document it read, `tasks_left` less one, and
  sends a `Delegate { project, task, tasks }` with the one goal. A
  refusal at admission, a whole milestone's among them (*concluded*,
  naming when), prints as `goal start`'s refusals do, exit 2, but for
  one lacking budget: the milestone funds the goal, not the party's
  pool, so it names the milestone, `goal start: refused: milestone M had
  LEFT left, NEEDED needed; give each goal in it --budget AMOUNT`, from
  jig's `Budget { needed, left }`, a second `--as-allowed` goal after a
  first took all among them (`../../domain/milestones.md`, section 4).
- **`--after TASK`**, repeatable, adds each task to the goal's
  dependencies, in order, once each, `TaskRef::Task`: a goal or a person
  task, in a milestone or not. jig judges each at admission: one done
  adds no wait; one failed or cancelled refuses the start, naming it and
  how it ended, printed as jig's refusal, exit 2. The tool judges none.
- **`--no-deadline`** gives the goal no deadline: the alternative to
  `--deadline TIME`, and so meets template 2's given deadline among the
  required controls. It is usage, exit 1, nothing sent, where the
  offered bounds need a deadline (`goal start: --no-deadline: the
  project's role ends every task by its ceiling; give --deadline`), or,
  with `--milestone`, where the milestone has one (`goal start:
  --no-deadline: milestone N ends by TIME; give --deadline`).
- **Its output** is `goal start`'s, `done: task N, goal`, record
  `goal_create`, whether it is in a milestone or not.
- **`help goal start`** says each flag in a line, and in its notes that
  a goal waiting on tasks counts its deadline from when it was made,
  and, after that line:

  ```
  With --milestone, --as-allowed takes all the milestone has left, so
  a second --as-allowed goal in it is refused for its budget: give each
  goal in a milestone --budget AMOUNT.
  ```

## 4. Whole, list and show

- **`milestone whole TASK`** sends `Conclude { project, task }`, the
  project the task's, read as `task cancel` reads it, and prints `done:
  task N, whole`; record `milestone_whole`. A second prints the same
  (jig answers done and keeps the first). A task that is not a gathering
  prints jig's refusal, exit 2, as one that has ended does, saying how
  it ended.
- **`milestone list`** prints one group, the live or with `--ended` the
  ended, as `task list` does (`tools-reads.md`, 1), a line a milestone:
  its number, its name, its count and its state:

  ```
  812  Store shape  7 / 8 done  live
  790  Search       5 / 5 done  complete
  ```

  The count is its task line's `delegates_done` and `delegates_total`
  (`../../domain/milestones.md`, section 6). The state of a live one is
  `live`; an ended one's `complete`, `ended` (its report's failed count
  above zero) or `closed` (cancelled). Under `--json`, the record
  `milestone_list`, its task lines as `task list` prints them, each with
  its `name` and `state`.
- **`milestone show TASK`** prints its facts, its totals and its goals:

  ```
  milestone 812 · Store shape · whole since 2026-11-16T10:02Z
  The store's new shape, from its draft
  made by person 4, Sam · 2026-11-16T09:12Z
  goals: 7 / 8 done · 0 failed · 1 closed · 1 live · 2 reds owed
  spent: 2.12 of 4.00
  last landing: 2026-11-16T21:38Z, S7 history in task logs (task 820)
  goals:
    813  S1 Draft the layout       done
    820  S7 history in task logs   live
  ```

  Its facts are `milestones::facts` (`../../domain/milestones.md`,
  section 3): `open` while not whole; the words line absent with none;
  the maker by number and member name, as a task's requester prints.
  `ended: TIME, complete` replaces `last landing`'s place after it once
  ended, with `ended` or `closed`. Its totals are the forge's
  `Milestone`, `spent:` the forge's spend of the milestone's budget,
  both in ctl's amount, as `task show` prints the milestone's
  (`results.md`, 1); where it answers `not_milestone` the tool prints jig's
  facts alone and `totals: none`. The goals' line ends with its totals'
  `reds`, the reds after landing its goals owe, ` · R reds owed`, ` · 1
  red owed` for one, zero included (`../../domain/red-findings.md`,
  section 10). Its goals are its delegates, live and
  ended, merged in the order they joined (`../../domain/milestones.md`,
  section 7), each its number, title and state, `live`, `done`, `failed`
  or `closed`; a list past one page prints `read on with:` as `task
  list` does. A live goal whose line names a first wait (jig's
  `domain/engine/waits.md`, 4) goes on with ` · ` and that wait as
  `task show`'s `waits:` prints one, `KIND OBJECT · since WALL · BOUND`,
  with the wait's own bound (`../../draft/bounded-waits.md`, 9), led by
  `task T: ` where it waits at a task under the goal:
  `820  S7 history in task logs   live · task 833: party question 3 at
  you · since 2026-11-16 10:02 UTC · passes up at 2026-11-16 18:02 UTC`,
  its walls as `waits:` prints them. Under `--json`, the record `milestone_show`:
  `facts`, `totals` (the answer's fields, or null), `goals` (task lines,
  which carry `wait` and `nearest`, `results.md`, 2).

## 5. Its tests

In the tools' step harness (`temper/crates/temper-client-tools`): each
tool parses and prints back; `help milestone`'s golden files; `milestone
new` filling template 8 with the gathering, its name refused past 60
characters or on two lines, its required controls; `goal start
--milestone` sending a `Delegate` with the narrowed authority, its
tasks from the milestone's `tasks_left`, refused
locally for a task that is not a milestone or is whole; `--after`'s
dependencies in order; `--no-deadline` sent with none and refused under
a deadline's bound; `milestone whole` a `Conclude`; `list` and `show`
printing from fixtures, `show` with the reds its goals owe. In the joined client world (`temper/tests/client`):
a milestone made, two goals started in it, the second `--after` the
first, said whole, and `milestone show` printing it; and, under a
delegation deep and wide enough that a goal in a milestone gets tasks of
its own, two goals started in one milestone, each admitted, the second
with an allowance one less than the first; and a milestone made
`--as-allowed`, a goal started in it `--as-allowed`, and a second so
refused for the milestone's budget, naming the milestone, exit 2.

## 6. Open questions

- **The pages' words in `show`:** the draft's state words and headline
  (its section 4) are the pages', milestone M3. Default: `show` prints
  facts and totals as above until M3 gives one function both use.
