# The tools' reads

This document holds the tools' reads, split from [`tools.md`](tools.md).

## 1. Reads

A read answers with a document, or a page and its cursor, and, for a
watch's subject, its digest (4). It needs no key.

| Tool | What it answers | jig's |
|---|---|---|
| `me` | the party, when its sign-in expires, its projects with their roles and pools, what waits for it counted, the bounds of what it may create, configured provider offers and key-byte ceilings, the retention | `me` |
| `inbox [--project PROJECT] [--kind questions\|proposals\|escalations\|person-tasks\|results\|replies] [--root TASK] [--after CURSOR]` | what needs the party, oldest first, each need with its *since*, its bound and whether it is overdue (`results.md`, 1, *`inbox`'s needs*), then updates, each with its acts | `inbox` |
| `waits --project PROJECT [--kind party\|question\|dependency\|resource\|account\|host\|delegate\|news\|step\|room\|turn\|sign-off] [--party PARTY] [--due DURATION] [--after CURSOR]` | every live wait in the project, a line each: its task, its kind and object, *since*, bound and what happens at it, and who hears; overdue first, then by bound, then by task, a page and its cursor (`results.md`, 1, *`waits`*); `--kind` takes a kind as `task show` prints it, PARTY a party's number, DURATION a duration as `--timeout` takes one, at most 30 days | `waits{project, kind, party, due}` (jig's `client/domain/contract/reads.md`, 1.2) |
| `project show PROJECT` | the project: its name, whether it is public and, if so, its public page's path, and its homes, its roles, each with its parties and its spend per period, and its policy's spend per period; its open period, its number, start and next start (`../../domain/routing.md`, 7), its budget and what is reserved, spent and left of it; and each party's pool in it, with the same numbers. A deleted project, its read answered gone (jig's `client/domain/contract/reads.md`, section 1), prints `project 3 was deleted` on stderr, exit 2, where one unknown prints `unknown` | a project, and its spend's first page |
| `task list --project PROJECT [--parent TASK] [--label chat\|goal] [--tracked] [--mine] [--ended] [--order activity\|priority] [--after CURSOR]` | task lines of one group: the live tasks, or with `--ended` the ended; with `--parent`, that task's delegates alone | `tasks{query}` (jig's `client/domain/contract/reads.md`, 1.1, its parent key) |
| `task show TASK [--tree] [--activity COUNT] [--run]` | a task's document, with its acts, its text saying under `next run:` why a run that can't start waits and what to run; with its live tree, its COUNT latest activity lines, its live run, in one answer | `task{n}`, `tree{root}`, `activity{n}`, `run{n}` |
| `role list --project PROJECT [--after CURSOR]` | the project's live standing roles, label 3 (`../../domain/agent/standing.md`, section 6), a line each: its number, its charter, its state, what its period's budget has left, and its Now's first line, or `no Now yet` | `tasks{query}` by label, and the notes read of each role's scope by the description `now` (jig's `client/domain/contract/reads.md`, section 1) |
| `role show TASK [--report]` | a standing role: its task's document, as `task show` prints it, then, while a talk is open, `talking with PARTY since TIME` (jig's `domain/tasks/standing.md`, 5.6), then its Now whole, its note's name, revision and when it was written, or `no Now yet`; with `--report`, its weekly report whole in the Now's place, the note described `report` read and printed as the Now is, or `no report yet` (`../../domain/agent/standing.md`, section 8) | `task{n}`, and the notes read of its scope by the description `now`, or `report` |
| `landing show LANDING` | a landing: its members, each candidate with its base, each gate's key with its receipt or run, the proof's state, the turn, its stale proofs and faults (`../../domain/landing.md`, section 14) | the forge's family |
| `landing list --project PROJECT [--ended] [--newer-than TIME] [--after CURSOR]` | the project's landings, newest first, with their phases, the turn's holder first, a page and its cursor; with `--newer-than`, every landing on its repositories' landing branches the engine saw after TIME (as `task amend --deadline` takes it, UTC to the minute), with its verdicts and its trailers, from the landing log (`../../domain/after-landing.md`, section 8, and `../../domain/after-landing-storage.md`, section 1) | the forge's family |
| `landing history --project PROJECT [--change TASK \| --landing TASK] [--since TIME] [--until TIME] [--after CURSOR]` | the project's landing history, a page of at most 32 events in its order, or a change's or a landing's, each event a line: its number, time, kind, change, landing, round and unit, head and base, and its kind's fields; the upper bound the first page fixed, and from when the project's changes are whole; a cursor whose event has gone refused, naming the first that remains (`../../domain/landing-history.md`, 5.1) | the forge's read `History` |
| `landing times --project PROJECT [--since TIME] [--until TIME] [--after CURSOR]` | each change's span split into states, 32 a page, then their landings; running totals (`../../domain/landing-times.md`, section 5) | the forge's read `Times` |
| `repository receipts REPOSITORY [--gate GATE]` | a repository's receipts, newest first: gate, key, verdict, run, candidate, time (`../../domain/landing-proof.md`, 1.2) | the forge's family |
| `draft list --project PROJECT` | the project's landed drafts, newest first, each its repository and path, ready, not ready or its goal, its commit, branch and time, its chat and its title (`areas.md`, area 7) | the forge's family, the subject `drafts` |
| `qa marks --project PROJECT` | the project's marks: landing, words, chosen, waiting or tested (`../../domain/findings.md`, section 8) | the forge's family |
| `follow list --project PROJECT` | the project's follows, one line each: source and target with their branches, the lag and the parity, in `../../domain/forge/follows.md`, section 7's words; then its dependencies, one line each, "dependency 1: fabrig depends on vsr-rs" (`../../domain/forge/dependencies.md`, section 4) | the forge's family |
| `follow show --project PROJECT FOLLOW` | one follow whole: its setting, its task, its lag's entries oldest first, each with its change and state, its skips with their reasons and its parity (`../../domain/forge/follows.md`, section 7) | the forge's family |
| `qa passes --project PROJECT` | the project's QA passes: the cleared main, the marks covered, the tester, its outcome and findings | the forge's family |
| `hold list --project PROJECT` | the project's placed holds: owner, mode, scope, reason, what lifts it, the tasks it holds and exempts, and whether the party may lift it | `holds{project}` |
| `chat show TASK [--before CURSOR]` | a conversation: turns, words and, where the run's host gives it, provisional text, in order, a page from its end | `activity{n}`, `run{n}`, through `thread` |
| `task activity TASK [--before CURSOR] [--limit COUNT]` | a task's activity, a page from its end | `activity{n}` |
| `task turn TASK TURN [--part PART] [--from OFFSET]` | one turn of the task, TURN as `A.T`, attempt A, turn T, whole: its text, each call and its result | `turn{n, attempt, turn}` (jig's `client/domain/contract/reads.md`, section 1) |
| `task notes TASK --project PROJECT [--after CURSOR]` | a page of the notes of the task's own scope, each whole, as a chat's document pane reads them: number, revision, description, body | `notes{project, task}` (jig's `client/domain/contract/reads.md`, section 1) |
| `task text TASK [--field specification\|instructions\|result] [--from OFFSET]` | one bounded UTF-8 chunk from the stored specification words (default), effective instructions or kept report, joining sequential backend pages within the tools’ window, with total bytes and continuation | `task_text{n, field, from}` |
| `task tree TASK` | a task's live tree: its root and each row its window holds, a chip a line, and what it folds (`results.md`, 1) | `tree{root}` |
| `task run TASK` | a task's live run: its attempt, its provisional text where its host gives it, its current call | `run{n}` |
| `settings show` | the settings: the newest revision's number; each setting, by kind and number with its name, following its default or a change and the revision that last changed it; each revision held, with who decided it, when, the proposal it came from and how many live tasks hold it; the pending proposals of settings | `settings` |
| `settings show charter NUMBER [--revision REVISION]`, `settings show template\|policy NUMBER`, `settings show rules` | one setting's value whole, as the newest reads it, or a charter as held REVISION reads it; with `--json`, the value in the form `settings apply` takes. A REVISION no longer held exits 2, `temper ctl: revision R is no longer held; its record is in temper ctl revision show R, and the held ones in temper ctl settings show` | `setting{kind, number, revision}` |
| `settings show project-ways PROJECT [--revision REVISION]`, `settings show project-questions PROJECT [--revision REVISION]`, `settings show project-classes PROJECT [--revision REVISION]` | a project's ways, its questions and its classes (`../../protocol/configuration/settings-kinds.md`, 1): the value stored, or the stock's, marked so, with none | `forge`, kinds 7, 6 and 8 |
| `settings show charter-words NUMBER [--revision REVISION]`, `settings show project-words PROJECT [--revision REVISION]` | a charter's words or a project's (`../../protocol/configuration/settings-kinds.md`, 1, kinds 9 and 10): the value stored; with none, a charter's the stock's, marked so, or that it has none, and a project's that it has none | `forge`, kinds 9 and 10 |
| `settings show landing-rule NUMBER [--project PROJECT] [--revision REVISION]`, `settings show forge-charter NUMBER [--revision REVISION]` | one of temper's settings (`../../protocol/configuration/settings-kinds.md`, 1): a landing rule of the deployment's, or of PROJECT's, or one of the forge's charters, 1 the change's and 2 its reviews'; its state from jig's `settings`, its value from the forge's read; with `--revision`, the value that revision set, while it remains in the history; with `--json`, the value in the form `settings apply` takes | the forge's `Setting { kind, scope, number }` (`../forge/served.md`) |
| `settings show template-goal NUMBER [--revision REVISION]` | template NUMBER's work (`../../protocol/configuration/settings-kinds.md`, 1, kind 11): `design` and `close_out`, the value stored, or, with none, neither, marked so; with `--revision`, the value that revision set, while it remains in the history; with `--json`, the value in the form `settings apply` takes | the forge's `Setting { kind: 11, scope: 0, number }` (`../forge/served.md`) |
| `settings show project-landing PROJECT [--revision REVISION]`, `settings show project-gates PROJECT [--revision REVISION]` | a project's landing and its gates (`../../domain/landing-settings.md`, 1.1, kinds 4 and 5): the value stored, in the names and forms `settings apply` takes for them, or that it has none; with `--revision`, the value that revision set, while it remains in the history; with `--json`, the value in the form `settings apply` takes; the line a too-large revision item prints names this call | the forge's `Setting { kind: 4 or 5, scope: PROJECT, number: 1 }` (`../forge/served.md`) |
| `revision show REVISION` | a revision: the revision it was made against, when, who decided it (a party, the release or the first start), the proposal it came from, whether it moved running work, and its items, each set with its value whole (or its size, too large, to read with `settings show`), removed or reset, and what it replaced (`was:` its value whole or its size, its default, removed, or not offered), a project's policy item with the policy it set and the one it replaced, temper's kinds' set items read at that revision from the forge and printed whole in text and JSON, their replaced explicit values read at the preceding set found in retained history, with size metadata when unavailable or too large, the forge keeping those values for a restore (jig's `domain/engine/settings.md`, 10); a revision forgotten past the history names the oldest kept | `revision{n}`, the forge’s `RevisionSetting { kind, scope, number, revision }` |
| `proposal show PROPOSAL` | a proposal of any kind, including its settings review: current values, origins, proposed values and the newest revision, each value whole: one past the review document's bound is read whole with `setting{…, proposal}` (jig's `domain/engine/settings.md`, 10) and the current one with `setting{…}`, so a review never shows a value as too large; a retained decided or withdrawn proposal keeps its state; an automatically replaced task proposal is unknown (7) | `proposal{proposer, number}`, `settings`, `setting{…}` |
| `revision list [--before REVISION]` | the history a page, newest first: each revision's record and its items without values, cut at the page's bound with how many more; the last page says whether older ones were forgotten | `revisions` |
| `push list` | the party's devices that pushes go to: each its number, label, when it was added and its push service's origin, never its endpoint whole or its keys (`../../domain/forge/attention.md`, 7.1) | the forge's `devices` |
| `deployment census` | deployment document's store census (jig's `client/domain/contract/documents.md`, 1), printed by `results.md`, 1 and 2 | `deployment` |

- **The findings' reads,** `finding list` and `finding show`, and a
  goal's reds after landing, `goal show`, are `findings.md`'s, section 1,
  as the milestones' are `milestones.md`'s.
- **A settings proposal's review** derives its groups from the delivered
  settings values, independently of the proposer's reason: *temper
  enforces* for the settings other than a charter's instructions and
  documents, *your agents follow* for those instructions and documents.
  A charter changing both appears in both groups; each group's entries
  name their fields and refer to the whole before and after values.
  Its *Effect* reads `settings` for live-task counts by held revision,
  explains new work on acceptance, no restarts, stays or moves, conversations
  at their next run and incompatible tasks retaining their revision.
  Authority remains current even on a retained charter revision (jig's
  `domain/engine/settings.md`, 4). Counts of tasks eligible to move are not
  available (`../README.md`, section 5). JSON retains the whole values and
  adds `review.groups` with setting and field references and `review.effect`
  with held revisions and their live-task counts, the running choices and
  their default, compatibility conditions and the same explanation. A
  failed inventory read reports unavailable counts, never a count of zero.

- **Temper's revision values** come from a revision-qualified forge read,
  never from the current setting. A value past its share of the output
  bound prints its size and `settings show landing-rule NUMBER
  [--project PROJECT] --revision REVISION` or `settings show forge-charter
  NUMBER --revision REVISION`; JSON includes the same call as `read`.
  That read returns the value set by this revision, even after later
  accepted changes overwrite it. Removed and reset edits retain their
  words, and `was:` retains the replaced-state metadata.
- **A revision's value too large to print** says `too large, N bytes;
  read with settings show charter NUMBER --revision REVISION` for a
  charter, and `too large, N bytes; read current value with settings
  show KIND [NUMBER]` for a template, policy or rules. Among these jig kinds, only a charter
  takes `--revision`; the other reads serve the current value, including
  when the hint appears in a historical revision.
- **`task list` reads one group a call.** jig answers task queries in
  two groups, live then ended, each with its own cursor (jig's
  `client/domain/contract/reads.md`, 1.1). Without `--ended`, the call
  reads live tasks; with it, ended tasks alone, as jig's view does.
  `--after CURSOR` takes that group's cursor, printed unchanged from jig.
  Reading on keeps `--ended`, whose help reads "Only ended tasks".
  To read both groups, an agent makes two calls.
- **`task list`'s order:** by number, ascending, without `--order`, with
  `--ended` or without; by activity or priority with it. The tools never
  send jig's other orders, `Changed` and `Ended`: no command line makes
  a call with either, and such a call is refused a command line, never
  given a guessed one, so that a call's command line parses back to
  that call (`results.md`, 1).
- **`inbox` reads needs, then updates,** in the one call. jig reads the
  inbox a group a call, needs or updates, each with its own cursor, and
  each kind in one group. A call reads needs first; while needs has
  more, its page is needs alone. Once needs ends on a page, updates'
  first page follows it when both fit in a page's rows (`bounds.md`, section 1);
  when they do not, the call that reads on starts updates. No page
  passes the rows, and no update is read twice.
- **The inbox's printed cursor** is one group byte, 0 for needs and 1
  for updates, then jig's cursor, whole, or none when the call that
  reads on starts updates; to the agent it stays opaque. The call that
  reads on stays in needs while needs has more, then moves to updates.
  `--after CURSOR` reads CURSOR's group; with `--kind`, a CURSOR of the
  other group is a usage error, exit 1.
- **A page cut by `--limit COUNT`** reads on with the same read without
  `--limit`, `task activity TASK`, or `task activity TASK --before
  CURSOR` with the CURSOR it was given, which prints the page whole, its
  COUNT rows again, then reads on at jig's `before`. `task show TASK
  --activity COUNT` that cuts a page prints `read with: task activity
  TASK`. The printed cursor stays jig's: at most a page's rows repeat,
  and no row is skipped.
- **Provisional text is the run's host's.** An agent process gives none
  (`../../protocol/llm.md`, section 7), and `chat show` and `task run` then
  print none: a run shows by its activity, and its words as each turn
  commits.
- **A conversation keeps its words readable.** `chat show` retains tool
  identities and failures, but leaves call inputs, outcomes and result
  summaries to `task turn TASK TURN`. Those payloads consume no conversation
  text budget. Conversational text shares the bounded window in order;
  an oversized reply keeps its archived part number and continuation.
- **A turn too large for a page** comes in a page by its size in place
  of its bytes (jig's `client/domain/contract/documents.md`, 1). `task
  activity`, `chat show` and `task show --activity` print it as one
  line, its number, its size and the call that reads it: `turn 3.7 ·
  412 KB · read with: task turn 41 3.7`.
- **`task turn`** prints each part of the turn cut to the tools' window,
  each cut with the call that reads on (`--part PART --from OFFSET`). A
  turn past the engine's bound on one read, or the client's limit for
  one document, is answered with its size and that it is past the limit,
  exit 0; the agent reports it as a finding (`bounds.md`, section 1).
- **The forge's changes are read on the forge,** with whatever the agent
  chooses, its API, `git` or a command line of the forge's, as a person
  reads them there: temper is agnostic about how (the user, 10-26). The
  tools give what temper is doing, why, and where it is on the forge:
  each part that has a place there names it by its ids and its address.
  A part of the forge's that the engine does not serve prints "Shown on
  the forge.", with the resource in the forge's words (`../forge/family.md`, 1
  and 2). What temper owns there, its branches, pull requests and
  landings, changes only through temper; what a person does there, a
  review or an approval at a head, they do on the forge as themselves,
  and temper reads it (`../forge/README.md`, sections 4 and 5). Of the forge's
  read family (`../forge/served.md`), the deployment's part is a tool,
  `forge list`, and a change's panel another, `change show` (`areas.md`, area
  6); the others join the table as tools of their own (`README.md`, section 11).

## 2. Open questions

- **Whole reads saying a cut first** (`results.md`, 4): `task turn` without
  `--part`, `task activity` and `chat show` could print a cut part's
  position before its text, and `task show` its asked words'. Default:
  their end line alone; the paged read each names says so first.
