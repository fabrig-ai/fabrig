# The tools

## 1. One table

- **A tool is a record of the table:** its name; a line saying what it
  does; its arguments, each a name, a type (a number, a word from a set,
  a text, a file, a duration) with its bound where it has one, whether
  it is required, and a line; its result, a typed record, and a line
  saying what it prints; whether it is keyed (`README.md`, section 5); its area
  (`areas.md`); and its notes, at most 1,024 bytes, what this tool alone does
  beyond its line: a file's life, a refusal's words and what mends it,
  a step that follows.
- **Everything else is derived from it:** the parser and `temper ctl
  help [GROUP [VERB]]`, the JSON of arguments and results, and the tool
  names the pages may use (`testing.md`, 5). A test checks that each derivation
  names every tool. Its lines and notes are documentation, in the voice
  of `../../protocol/documentation.md`, section 2.3.
- **A call is a value:** a tool's name and its arguments, decoded from
  argv or from JSON and checked against the table before anything is
  sent. The runner takes a call and gives back its result; the host
  interprets neither.
- **A tool's name is its words:** a group, one of the map's nouns
  (`task`, `chat`, `goal`, `change`,
  `proposal`, `settings`, `revision`, `account`, `model`, `forge`,
  `repository`, `playground`, `hold`, `landing`, `finding`, `qa`, and
  area 3's `project`, `party` and `deployment`; `../../protocol/documentation.md`,
  2.4), then a verb of the closed vocabulary, `task show`; or
  one of the six names of the client and the party, `me`, `inbox`,
  `sign-in`, `sign-out`, `pending` and `help`. A read of a part of the
  group's noun is the part's name: `task tree`, `task run`, `task
  activity`, `task turn`, `forge repositories`, `repository receipts`,
  `qa marks`, `qa passes`. `help GROUP` lists a
  group's rows; `--call`'s `tool` is the name's words, one space
  between (`README.md`, 6.1).
- **The table's vocabulary of placeholders** shows the arguments in
  the usage; written pages use `../../protocol/documentation.md`, 2.1.
  The table's `shown` names are `TASK`,
  `PROJECT`, `FORGE`, `INSTALLATION`, `ACCOUNT`, `PROVIDER`, `ADAPTER`,
  `SOURCE`, `POSITION`, `NAME` (a program source's), `HOLD` (a placed
  hold's number), `LANDING`, `FINDING`, `REVISION` (a
  settings revision), `REV` (a revision a read printed of the object
  acted on: a spec's, a hold's, a person task's), `PROPOSAL`, `QUESTION`,
  `CHOICE` (a person contract's code), `STATE` (its approval state), `NUMBER` (a
  setting's), `TEMPLATE`, `CHARTER`, `CURSOR`, `COUNT`, `PART`, `OFFSET`, `PAGE`,
  `TURN`, `DIGEST`, `CONDITION`, `DURATION`, `TIME`, `PRIORITY`,
  `MODEL`, `EFFORT`, `AMOUNT`, `OWNER`, `ORG`, `REPOSITORY`, `BRANCH`,
  `PREFIX`, `GATE`, `LABEL`, `IDS`, `APP`, `ROLE`, `FILE`, `TEXT`,
  `REASON`, `KEY`. A step test refuses any other `shown` name.
- **Projects by name or number;** a name is resolved through `me`. A
  name is never decimal digits alone, and no two projects share one
  (jig's `domain/model.md`, 6.1), so an argument is a number or a name,
  never both, and names one project.

## 2. Reads

A read answers with a document, or a page and its cursor, and, for a
watch's subject, its digest (4). It needs no key.

| Tool | What it answers | jig's |
|---|---|---|
| `me` | the party, its projects with their roles and pools, what waits for it counted, the bounds of what it may create, configured provider offers and key-byte ceilings, the retention | `me` |
| `inbox [--project PROJECT] [--kind questions\|proposals\|escalations\|person-tasks\|results\|replies] [--root TASK] [--after CURSOR]` | what needs the party, then updates, each with its acts | `inbox` |
| `project show PROJECT` | the project: its name, whether it is public and, if so, its public page's path, and its homes, its roles, each with its parties and its spend per period, and its policy's spend per period; its open period, its number, start and next start (`../../domain/routing.md`, 7), its budget and what is reserved, spent and left of it; and each party's pool in it, with the same numbers | a project, and its spend's first page |
| `task list --project PROJECT [--parent TASK] [--label chat\|goal] [--tracked] [--mine] [--ended] [--order activity\|priority] [--after CURSOR]` | task lines of one group: the live tasks, or with `--ended` the ended; with `--parent`, that task's delegates alone | `tasks{query}` (jig's `client/domain/contract/README.md`, 4.1, its parent key) |
| `task show TASK [--tree] [--activity COUNT] [--run]` | a task's document, with its acts, its text saying under `next run:` why a run that can't start waits and what to run; with its live tree, its COUNT latest activity lines, its live run, in one answer | `task{n}`, `tree{root}`, `activity{n}`, `run{n}` |
| `role list --project PROJECT [--after CURSOR]` | the project's live standing roles, label 3 (`../../domain/agent/standing.md`, section 6), a line each: its number, its charter, its state, what its period's budget has left, and its Now's first line, or `no Now yet` | `tasks{query}` by label, and the notes read of each role's scope by the description `now` (jig's `client/domain/contract/README.md`, section 4) |
| `role show TASK` | a standing role: its task's document, as `task show` prints it, then its Now whole, its note's name, revision and when it was written, or `no Now yet` | `task{n}`, and the notes read of its scope by the description `now` |
| `landing show LANDING` | a landing: its members, each candidate with its base, each gate's key with its receipt or run, the proof's state, the turn, its stale proofs and faults (`../../domain/landing.md`, section 14) | the forge's family |
| `landing list --project PROJECT [--ended] [--newer-than TIME] [--after CURSOR]` | the project's landings, newest first, with their phases, the turn's holder first, a page and its cursor; with `--newer-than`, every landing on its repositories' landing branches the engine saw after TIME (as `task amend --deadline` takes it, UTC to the minute), with its verdicts and its trailers, from the landing log (`../../domain/after-landing.md`, sections 8 and 10) | the forge's family |
| `repository receipts REPOSITORY [--gate GATE]` | a repository's receipts, newest first: gate, key, verdict, run, candidate, time (`../../domain/landing.md`, 6.2) | the forge's family |
| `finding list --project PROJECT [--state new\|routed\|backlog\|dropped]` | the project's findings: issue, kind, severity, area, state, signature, times seen, goal (`../../domain/findings.md`, section 2) | the forge's family |
| `finding show FINDING` | one finding whole, its state as its word set | the forge's family |
| `qa marks --project PROJECT` | the project's marks: landing, words, chosen, waiting or tested (`../../domain/findings.md`, section 8) | the forge's family |
| `follow list --project PROJECT` | the project's follows, one line each: source and target with their branches, the lag and the parity, in `../../domain/forge/follows.md`, section 7's words | the forge's family |
| `follow show --project PROJECT FOLLOW` | one follow whole: its setting, its task, its lag's entries oldest first, each with its change and state, its skips with their reasons and its parity (`../../domain/forge/follows.md`, section 7) | the forge's family |
| `qa passes --project PROJECT` | the project's QA passes: the cleared main, the marks covered, the tester, its outcome and findings | the forge's family |
| `hold list --project PROJECT` | the project's placed holds: owner, mode, scope, reason, what lifts it, the tasks it holds and exempts, and whether the party may lift it | `holds{project}` |
| `chat show TASK [--before CURSOR]` | a conversation: turns, words and, where the run's host gives it, provisional text, in order, a page from its end | `activity{n}`, `run{n}`, through `thread` |
| `task activity TASK [--before CURSOR] [--limit COUNT]` | a task's activity, a page from its end | `activity{n}` |
| `task turn TASK TURN [--part PART] [--from OFFSET]` | one turn of the task, TURN as `A.T`, attempt A, turn T, whole: its text, each call and its result | `turn{n, attempt, turn}` (jig's `client/domain/contract/README.md`, section 4) |
| `task notes TASK --project PROJECT [--after CURSOR]` | a page of the notes of the task's own scope, each whole, as a chat's document pane reads them: number, revision, description, body | `notes{project, task}` (jig's `client/domain/contract/README.md`, section 4) |
| `task text TASK [--field specification\|instructions\|result] [--from OFFSET]` | one bounded UTF-8 chunk from the stored specification words (default), effective instructions or kept report, joining sequential backend pages within the tools’ window, with total bytes and continuation | `task_text{n, field, from}` |
| `task tree TASK` | a task's live tree: its root and each row its window holds, a chip a line, and what it folds (`results.md`, 1) | `tree{root}` |
| `task run TASK` | a task's live run: its attempt, its provisional text where its host gives it, its current call | `run{n}` |
| `settings show` | the settings: the newest revision's number; each setting, by kind and number with its name, following its default or a change and the revision that last changed it; each revision held, with who decided it, when, the proposal it came from and how many live tasks hold it; the pending proposals of settings | `settings` |
| `settings show charter NUMBER [--revision REVISION]`, `settings show template\|policy NUMBER`, `settings show rules` | one setting's value whole, as the newest reads it, or a charter as held REVISION reads it; with `--json`, the value in the form `settings apply` takes. A REVISION no longer held exits 2, `temper ctl: revision R is no longer held; its record is in temper ctl revision show R, and the held ones in temper ctl settings show` | `setting{kind, number, revision}` |
| `settings show project-ways PROJECT [--revision REVISION]`, `settings show project-questions PROJECT [--revision REVISION]`, `settings show project-classes PROJECT [--revision REVISION]` | a project's ways, its questions and its classes (`../../protocol/configuration/settings.md`, 2): the value stored, or the stock's, marked so, with none | `forge`, kinds 7, 6 and 8 |
| `settings show charter-words NUMBER [--revision REVISION]`, `settings show project-words PROJECT [--revision REVISION]` | a charter's words or a project's (`../../protocol/configuration/settings.md`, 2, kinds 9 and 10): the value stored; with none, a charter's the stock's, marked so, or that it has none, and a project's that it has none | `forge`, kinds 9 and 10 |
| `settings show landing-rule NUMBER [--project PROJECT] [--revision REVISION]`, `settings show forge-charter NUMBER [--revision REVISION]` | one of temper's settings (`../../protocol/configuration/settings.md`, 2): a landing rule of the deployment's, or of PROJECT's, or one of the forge's charters, 1 the change's and 2 its reviews'; its state from jig's `settings`, its value from the forge's read; with `--revision`, the value that revision set, while it remains in the history; with `--json`, the value in the form `settings apply` takes | the forge's `Setting { kind, scope, number }` (`../forge/served.md`) |
| `settings show template-goal NUMBER [--revision REVISION]` | template NUMBER's work (`../../protocol/configuration/settings.md`, 2, kind 11): `design` and `close_out`, the value stored, or, with none, neither, marked so; with `--revision`, the value that revision set, while it remains in the history; with `--json`, the value in the form `settings apply` takes | the forge's `Setting { kind: 11, scope: 0, number }` (`../forge/served.md`) |
| `revision show REVISION` | a revision: the revision it was made against, when, who decided it (a party, the release or the first start), the proposal it came from, whether it moved running work, and its items, each set with its value whole (or its size, too large, to read with `settings show`), removed or reset, and what it replaced (`was:` its value whole or its size, its default, removed, or not offered), a project's policy item with the policy it set and the one it replaced, temper's kinds' set items read at that revision from the forge and printed whole in text and JSON, their replaced explicit values read at the preceding set found in retained history, with size metadata when unavailable or too large, the forge keeping those values for a restore (jig's `domain/engine/settings.md`, 10); a revision forgotten past the history names the oldest kept | `revision{n}`, the forge’s `RevisionSetting { kind, scope, number, revision }` |
| `proposal show PROPOSAL` | a proposal of any kind, including its settings review: current values, origins, proposed values and the newest revision, each value whole: one past the review document's bound is read whole with `setting{…, proposal}` (jig's `domain/engine/settings.md`, 10) and the current one with `setting{…}`, so a review never shows a value as too large; a retained decided or withdrawn proposal keeps its state; an automatically replaced task proposal is unknown (7) | `proposal{proposer, number}`, `settings`, `setting{…}` |
| `revision list [--before REVISION]` | the history a page, newest first: each revision's record and its items without values, cut at the page's bound with how many more; the last page says whether older ones were forgotten | `revisions` |

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
- **`task list` reads one group a call.** jig answers a task query in
  two groups, live then ended, each with its own cursor (jig's
  `client/domain/contract/README.md`, 4.1), and a printed cursor is one
  group's. Without `--ended`, `task list` reads the live group, `--after
  CURSOR` its cursor; with `--ended`, the ended group alone, as jig's
  view reads it, `--after CURSOR` the ended group's. The call that reads
  on keeps `--ended`, whose help reads "Only ended tasks". An agent that
  wants both groups makes two calls; the printed cursor is jig's, with
  nothing added.
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
  first page follows it when both fit in a page's rows (`README.md`, section 9);
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
  exit 0; the agent reports it as a finding (`README.md`, section 9).
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

## 3. Acts

Each act is one keyed request through `jig-client`'s request table,
answered once durable (`README.md`, section 5). `K` marks the tools whose key the
agent must name.

| Tool | What it does | jig's | K |
|---|---|---|---|
| `chat start --project PROJECT [--template TEMPLATE] TEXT` | starts a chat from TEMPLATE, the project's chat template when none is named: template 5 is Ask temper (`../../domain/agent/ask-temper.md`) and template 6 a design session (`../../domain/agent/agents.md`, 4); a template that is no chat's, or not offered, is refused naming the templates `me` offers | `shapes::chat`, a `Create` | yes |
| `role start --project PROJECT --budget AMOUNT [--charter CHARTER] TEXT` | starts a standing role from template 7 (`../../domain/agent/standing.md`, section 2): TEXT its spec's words, what it covers and how; AMOUNT its budget each period; run by CHARTER, the stock's charter 13 when none is named; prints the task it made, or the proposal a batch beyond the person's role became | `shapes`' fill of template 7, a `Create` | yes |
| `task message TASK TEXT` | words to a task | `Message` | yes |
| `chat close TASK [--move TASK]... [--reason TEXT]` | closes a chat: its live delegates cancelled, or moved to the party where named; the reason empty unless given, as jig's reference UI sends it | `shapes::close_chat`, a compound request | yes |
| `repository adopt --project PROJECT [--forge FORGE] --owner OWNER --name REPOSITORY [--ids IDS] --landing BRANCH [--role owned\|fork\|context] [--prefix PREFIX] [--no-home] [--no-ci] [--check GATE]...` | adopts a repository | the forge family's request (`../forge/family.md`, 4) | yes |
| `repository make --project PROJECT [--forge FORGE] ORG/NAME [--no-home] [--until adopted] [--timeout DURATION]` | makes an empty repository, its starter workflow in it, and has it adopted into the project: owned, landing on its default branch, with CI, the project's home unless `--no-home` (`../../domain/forge/adoption.md`, 5) | the forge family's `MakeRepository` (`../forge/records.md`) | yes |
| `repository copy --project PROJECT [--forge FORGE] ADDRESS [--as ORG/NAME] [--no-home] [--until adopted] [--timeout DURATION]` | copies a public repository, `HOST/OWNER/NAME`, into `OWNER/NAME` on the forge, or `--as` where given, and has it adopted as `repository make` does; the address refused, naming the form to write, before anything is sent | the forge family's `MakeRepository` | yes |
| `task answer TASK QUESTION TEXT` | answers question QUESTION of the task | `Answer` | |
| `proposal decide PROPOSAL accept\|reject\|pass-up [REASON] [--running stays\|moves]` | decides a proposal: PROPOSAL is `tN/K` for task N's, `pN/K` for party N's | `Decide` | |
| `task decide TASK --hold REV release\|leave\|pass-up [REASON]` | decides an escalation | `Decide` | |
| `settings propose FILE [REASON] --against REVISION` | proposes the whole settings set to the owners of its items, the deployment's and each project's it names; FILE or `-` shares the format and bounds of `settings apply`; the optional reason is forwarded unchanged | `Propose` settings | yes |
| `settings apply FILE --against REVISION [--running stays\|moves]` | changes the settings in one decision: FILE (or `-`) holds the items; REVISION is the newest revision the caller read; running work stays unless `--running moves`; answered the new revision's number | `ChangeSettings` | yes |
| `revision advance REVISION` | moves every live agent task on held REVISION that may move to the newest, each at its next run; prints jig's answer, `moved 3 tasks to revision 9; 1 could not, and finishes on 7` | `MoveToNewest` | yes |
| `revision restore REVISION [REASON]` | proposes the settings as REVISION left them, against the newest, to the owners of its items: jig's kinds, temper's (`../../protocol/configuration/settings.md`, 2) and each project's policy a later revision set; REASON follows jig's "restores revision R"; prints the proposal's number and what it leaves out, a deleted project's settings and policy, by kind, project and number; `proposal show` then shows its items, and an owner decides it with `proposal decide`. A revision forgotten past the history exits 2 naming the oldest kept, `temper ctl: revision R is forgotten; the oldest kept is revision O`, and one too large to restore says so, exit 2, a later revision restorable instead | `Restore` (jig's `domain/engine/settings.md`, 10) | yes |
| `task release TASK --hold REV [--again]` | lifts a hold | `Release` | |
| `task stop TASK` | cancels a task's run, the task held | `Stop` | |
| `task move TASK --under TASK` | makes the second task the first's requester (jig's `domain/tasks/changes.md`) | `Move` | yes |
| `task succeed TASK --by TASK` | names a live task the successor of an ended one: its open questions, subscriptions and kept news go to it | `Succeed` | yes |
| `finding route FINDING --goal TASK` | routes a finding to a goal: news to it, `state:routed` (`../../domain/findings.md`, section 6) | the forge's request | yes |
| `follow start --project PROJECT FOLLOW COMMIT` | gives the owner's word to the follow change for the landing COMMIT, lifts its owner's hold, or makes it again after an end without landing (`../../domain/forge/follows.md`, section 5) | the forge's request | yes |
| `follow hold --project PROJECT FOLLOW COMMIT --reason TEXT` | keeps the follow change for COMMIT from producing, its reason in the lag | the forge's request | yes |
| `follow skip --project PROJECT FOLLOW COMMIT --reason TEXT` | counts the landing COMMIT followed with no change, cancelling its follow change and keeping the chain whole | the forge's request | yes |
| `follow merge --project PROJECT FOLLOW COMMIT COMMIT...` | makes the first follow change of consecutive landings follow them all, cancelling the others' and keeping the chain whole | the forge's request | yes |
| `finding backlog FINDING` | puts a finding in the backlog, `state:backlog` | the forge's request | yes |
| `finding drop FINDING REASON` | drops a finding: `state:dropped`, its issue closed with the reason | the forge's request | yes |
| `qa choose LANDING` | chooses a landing's mark for the next QA pass (`../../domain/findings.md`, section 8) | the forge's request | yes |
| `task hold TASK --stop\|--drain REASON` | places a hold of the party's over TASK's subtree: stop cancels its runs at once, drain lets them finish their turns; a party's hold lifts by `hold lift` alone (jig's `domain/tasks/lifecycle.md`, 10) | `Hold` | yes |
| `project hold PROJECT --stop\|--drain REASON` | places a hold of the party's over the whole project, the party's chats exempt | `Hold` | yes |
| `project make NAME` | makes a project from a copy of policy 1, as the client switcher does; prints its number, kept on key replay | `MakeProject { name, policy: 1 }` | yes |
| `project amend PROJECT --visibility public\|private` | makes the project readable by anyone without signing in, or private again (jig's `domain/people/public-reading.md`, 2), by its owner; prints `project 1: public` and, made public, the public page's path from `me`'s public identifier, `/public/projects/3f2a9c01d4e5b6a7`, and on stderr, as its help does, "Anything said in this project's chats may show on its public pages, in the titles of its goals and tasks. On a public repository, a change's pull request shows what its agent wrote too." (jig's `domain/people/public-reading.md`, 3); project 0 and a member are refused, exit 2 | `SetVisibility` | yes |
| `project remove PROJECT` | removes the party's own project, by its owner (jig's `domain/people/acts.md`, 2), a fresh store's default project among them: closed at once and deleted whole after, each of its live chats asked to finish within its chat template's grace as `me` gives it, and none where the project has no chat template; prints `removed: project 3, fabrig`, the answer's number and the name `me` gave PROJECT at the call (the number alone where `me` gave none), the same on key replay while the deletion lasts. A project not in the party's `me` sends nothing, exit 2, as any unknown project; a member who is not its owner, or a project already being deleted, is refused, exit 2. No confirmation, as no act asks one (below) | `shapes::remove_project`, `RemoveProject { project, grace }` | yes |
| `project fund PROJECT AMOUNT [--role ROLE]` | sets what the project's period may spend, its spend per period, to AMOUNT, in the deployment's unit, or with `--role` that role's; either resizes what the open period gave (jig's `domain/authority.md`, section 7), so a raise funds the next chat at once, and each period the calendar opens takes it (`../../domain/routing.md`, 7). An owner's chats are funded from their pool, their role's, and pools reserve from the project's period, so a raise for them is two calls, the project's first, then the role's; a role's raise past the period's room is refused, exit 2, naming what the period has left. A cut goes the other way, the role's first, since a project's spend below a role's is refused; it keeps each budget at what it has used. A refusal past a ceiling (jig's typed ceiling, its holder, dimension and, for a scalar, its value) exits 2 with one line naming it and the next step: `project fund: past the rules' period spend, V; raise the rules first with temper ctl settings apply`; with `--role`, `past the project's period spend, V; fund the project first: temper ctl project fund P AMOUNT`; a cut, `below role R's period spend, V; lower the role first: temper ctl project fund P AMOUNT --role R`; an authority over its ceiling when rechecked, `the project's authority is past the rules' DIMENSION[, V]; raise the rules with temper ctl settings apply, or narrow the project's policy` (for a role, `role R's authority is past the project's DIMENSION[, V]`), a list's dimension (tools, executors, grants, notes) named without its contents, read whole with `settings show`. Its answer is the edit, `project 1: spend per period 449696` or `role 0: spend per period 415104`, the same under one key by construction (`README.md`, section 5); then a read marked as such, `now: period 1: budget 449696, left 449590`, and with `--role` each of the role's pools now: a replay prints the edit as first answered and today's numbers after it | the project's policy edit, `ProjectSpend`, or with `--role` `RoleSpend` | yes |
| `hold lift HOLD --rev REV [--with-tasks]` | ends a placed hold; with `--with-tasks`, releases every task it held that nothing else holds | `Lift` | yes |
| `task cancel TASK [--grace DURATION] REASON` | cancels a task and its tree; a task that has ended refuses a cancel (exit 2), saying how it ended, and the same cancel under its key gets its first answer | `Cancel` | |

- **Settings FILE is JSON:** `{"items": [ITEM, ...]}`, at most one item per
  setting, each `{"setting": SETTING, "set": VALUE}`,
  `{"setting": SETTING, "edit": "removed"}` or
  `{"setting": SETTING, "edit": "reset"}`. SETTING is
  `{"kind": "charter"|"template"|"policy", "number": N}`,
  `{"kind": "rules"}`, `{"kind": "landing-rule", "number": N}` for the
  deployment's, `{"kind": "landing-rule", "project": P, "number": N}`
  for project P's, `{"kind": "forge-charter", "number": N}`, N 1
  the change's charter and 2 its reviews', `{"kind": "charter-words",
  "number": N}`, N a charter's number, `{"kind": "project-ways",
  "project": P}`, `{"kind": "project-questions", "project": P}`,
  `{"kind": "project-classes", "project": P}`,
  `{"kind": "project-words", "project": P}`, or `{"kind":
  "project-policy", "project": P}`, project P's own policy (jig's
  `domain/engine/settings.md`, 2), set alone and at most once in a file; VALUE is exactly the
  `value` member `settings show … --json` prints: for a landing rule, `README.md`, section 3
  's rule member but `connector` and `kind`, its `pattern` in `FIELDS`'
  `landing-pattern` form, its segments as hex
  (`../../protocol/configuration/settings.md`, 2), for the forge's charter a charter's
  number, for a project's ways `{"drafts": BOOL, "goals": BOOL, "changes": BOOL,
  "way": N}`, for a project's questions `"none"`, `"foreman"` or `"up"`,
  for a project's classes `{"items": [ITEM, ...], "sets": [SET, ...]}`,
  each ITEM `{"kind": KIND, "words": TEXT, "class": CLASS}` or `{"set": N,
  "words": TEXT, "class": CLASS}`, `words` optional and empty by default,
  KIND one of the ten kinds' names and CLASS `"decide"`, `"ask"` or
  `"ask-away"`, each SET `{"number": N, "name": TEXT, "patterns": [TEXT,
  ...]}` (`../../domain/forge/attention.md`, 6), refused as `Classes::check` refuses,
  for a charter's words its line as a string, for a project's words
  `{"headline": STRING, "lede": STRING}`,
  and for a project's policy a policy in the form `settings show
  policy NUMBER --json` prints, the stock's policy 1 a project's starting
  point. Decoded bounded (`results.md`, 4): the file's size, the item count and every
  value's bounds are jig's (`ChangeSettings`' own, and the limits the engine's
  `me` gives where it gives them). Past a bound it is refused before anything
  is sent, naming the item and the bound. Unknown members are refused,
  naming them. A member `FIELDS` marks optional may be left out and
  takes the default its row names: a template's model choice is
  `{"main": {"model": M, "effort": E}}` alone, its `account` none and
  its `subagents` empty; a template's `resources`, the symbols a task
  filled from it holds, is a list of `{"connector": C, "symbol": S}` as
  jig's format has it (jig's `protocol/configuration.md`, 3.3), left
  out `[]`, so the reference's goal template names the forge's *its own
  goal branch* as `{"connector": F, "symbol": 1}`, F the forge's
  connector (`../../domain/forge/README.md`, section 2); a required
  member left out is refused naming its path, `llm.main`. Every refusal a person reads numbers items from 1, the
  decoder's and the engine's alike (jig's index plus one), and names a
  member by its path in the file, `charter.alternatives`, never by the
  wire's limit's name. `null` is refused where a member takes a value:
  a charter's `model` is a model and its `alternatives` a list, and an
  authority's models are `"all"` or a list of model names. A project's
  policy `removed` or `reset`, or a second one, is refused before
  anything is sent, naming the item. So is a template's or charter's effort
  that its model doesn't offer, where `me` shows that model in any of the
  party's projects: exit 1, nothing sent, `settings file item 1: model M offers
  no effort xhigh; it offers high, medium`, or `it offers none`. A model
  `me` doesn't show goes to the engine, which refuses it by name (jig's
  `client/domain/contract/README.md`, 3). jig's kinds are the deployment's; a landing rule names
  its scope, the deployment's or a project's, as a project's policy
  names its project, and a set that touches a project's needs an owner
  of it, one that touches the deployment's too one person who owns both
  (jig's `domain/people/acts.md`, 1); `settings propose` sends such a set
  to them, and `settings apply` from one who is not both is refused
  naming the item, exit 2.
  `proposal show` prints the values of temper's items from the forge's
  `ProposalItems` read, and a project's policy as it is now and as it
  would be, each whole, from jig's review; `settings show` lists
  temper's settings with jig's, by the names above, and no project's
  policy, which is no setting (`project show` prints its roles and
  spends).
- **A goal template's work** (`../../protocol/configuration/settings.md`, 2, kind
  11): its SETTING `{"kind": "template-goal", "number": N}`, N the
  template's, the deployment's as a template is, and its VALUE
  `{"design": BOOL, "close_out": BOOL}`, both members required, as
  `settings show template-goal N --json` prints it; a proposal card
  names it `template-goal N`.
- **A charter's parts and a later choice** (`../../draft/per-wake.md`,
  sections 3 and 5; jig's `protocol/configuration.md`, sections 3 and
  3.1). A charter's `parts`, which `FIELDS` marks optional, left out
  `[]`, is a list, in order, of `{"on": [REASON, ...], "instructions":
  TEXT}`: each `on` a nonempty list of `first` and the kinds that
  choose parts, by jig's store names (jig's
  `client/domain/contract/requests.md`, 2), each named once, and its
  instructions text, bounded as the charter's own are. A run reads the
  charter's `instructions`, then each part whose `on` meets why it runs.
  A template's model choice may add `"later": {"main": {"model": M,
  "effort": E}, "subagents": [...]}`, its `subagents` left out empty:
  the choice every run after the task's first activation's takes (jig's
  `domain/tasks/README.md`, section 3), its efforts checked against `me` as
  the first choice's are; left out, it has none. `settings show … --json`
  prints `parts` only for a charter that has some and `later` only for
  a choice that has one, so a value without them prints as before. The
  decoder refuses, before anything is sent, naming the item and the
  member's path, its indices from 0 as a document's are: a name that is
  no reason, or one every run is offered, `settings file item 1:
  charter.parts[1].on[0]: "answer" goes to every run and chooses no
  part`; a name twice; an empty `on`; more
  parts than `tasks.charter_parts`; and parts on a charter whose runs
  resume, `charter.parts: a charter that resumes takes no parts`. jig's
  refusal of a change names the charter and no part
  (`../../draft/per-wake.status.md`, 3), so a malformed part prints as
  `settings apply: out of shape; item 1; charter 5: shape`.
  A charter's stored policy, its parts with it, past
  `root.journal.run_bytes` also names its measured bytes and the bound:
  `settings apply: out of shape; item 1; charter 5: shape; stored policy: supplied 70112; maximum root.journal.run_bytes 65536`.
  The run's settings tool uses the same words; JSON names the member
  `stored_policy` (`../../domain/root.md`, section 9).
  In text, `settings show charter N` prints,
  after the value, a line per part, by its path, with its `on` and its
  instructions' bytes, `parts[0]: on first; 4210 bytes`,
  and `settings show template N` a line for a later choice, `later:
  claude-opus-5-5 high`.
- **The form's fields are one table,** `FIELDS`, beside the form's
  decoder in the crate ctl and the runs' `propose_settings` share
  (`temper-settings-form`): a row per member of a shape, by the shape's
  name and the member's (`authority.spend`, `model.name`), with its JSON
  type, its bound where the settings' limits give one, whether it may be
  left out and its default then, and a line saying what it does to
  work. A shape that recurs (an authority in a policy's
  ceiling, its roles and the rules; a model in a charter's model and its
  alternatives) has its rows once. `resolve` walks a path from a kind
  through the shapes (`policy.ceiling.spend`, `charter.model.name`) to
  its row. The reference page `settings-fields` is rendered from it, a
  section per shape (`../../protocol/documentation.md`, section 6), and a
  step test holds it to the form: every member that `settings show …
  --json` prints for each stock value, and every member the decoder takes,
  resolves to a row, and every row is reached. temper's own kinds'
  rows come with their decoder.
- **The arguments are jig's,** named as its operations name them, with
  the revisions they need (jig's `client/domain/contract/documents.md`, 2): a
  proposal by its proposer and number, `t41/3` for task 41's third,
  `p3/17` for party 3's seventeenth, as every result prints it, an
  escalation and a release by the hold's revision. An agent copies them
  from the acts a result printed (`results.md`, 1); it never has to know which an
  act needs.
- **`repository adopt` takes the adoption form's fields and defaults**
  (`../forge/family.md`, 4): forge 1, role `owned`, prefix `temper/`, the home
  and CI on. `--landing` names the landing branch. A context repository
  is never the home: `--role context` turns the home off. `--no-ci`
  needs at least one `--check`, a gate number, each number once. `--ids
  IDS` is a picker row's repository id and owner id, `R/U`, as `forge
  repositories` prints them (`areas.md`, area 6), carried with that row's
  `--owner` and `--name`; without it the request carries no ids
  (`../forge/records.md`). The form's checks are the parser's: a call that
  fails one exits 1 before anything is sent.
- **No confirmation.** The person's client says what an act will do
  before sending it (`../README.md`, 3.6); an agent reads it in the
  document's acts and their effects, and the revision is its promise.
- **A draft beyond the party's bounds** is compared with the bounds the
  document gives, as the person's client does (jig's
  `client/domain/contract/documents.md`, 3): within, sent; between the bounds,
  sent as `Propose` where it is proposable, and the result says so; past
  them, refused by the client, nothing sent. Where `me` gives no bound
  for a draft, it is sent and the engine's answer printed.

`repository make` and `repository copy` answer once the request is kept.
`--until adopted` then follows the `forges` watch's row: gone with its forge
still listed, it is adopted, status 0; failed, its step's words and status 1;
past `--timeout`, the row's state and status 2, as a read's timeout is, the
copy going on. These two are the only acts that take `--until`, and a
`--timeout` without it is refused, as on a read.

A charter's `tools` in a settings file is an integer mask in the same bits
as an authority's tools. Left out, it is 1,023, every jig family but
settings, so a settings file written without that member does not offer the
settings family to its runs (2026-11-04).

## 4. Waiting

```
READ [--since DIGEST] [--until CONDITION] [--timeout DURATION]
```

- **A watched read** is one whose arguments are a watch's subject (jig's
  `client/domain/contract/README.md`, 5.1): `me`; `inbox` with no flag, the
  inbox's watch being the party's whole inbox; `task show TASK` with no
  flag; `task tree TASK`; `task run TASK`; `task list` with what its
  watch takes alone, without `--after` or `--ended`, `tasks{query}`
  following live tasks; `account list` (area 5 of `areas.md`); `forge list`
  and `change show TASK` (area 6). Given `--since` or `--until`, it
  waits; given neither, it reads at once and prints its digest.
  `--timeout` alone, with neither, and a wait's flag on any other read,
  `task show TASK --tree` among them, are usage errors, exit 1. There is
  no `wait` tool.
- **A digest** is the first 8 bytes of the SHA-256 (skein's `runtime/lib.md`,
  6.2) of the subject's document as `--json` prints it (`results.md`, 2), without
  its `digest` field, printed as 16 hex digits. A read and a wait digest
  one form, the watch's document: for `inbox` and `task list`, whose
  read is a page and whose watch a window, the rows the watch's first
  window holds, in its order, and whether more follow, within the tools'
  page of rows (`README.md`, section 9). `task list`'s read makes that form from its
  own rows; an unfiltered `inbox` read opens the inbox's watch once for
  it, since a needs page leaves unfetched the updates the watch's window
  holds. Each prints its cursor beside the digest, outside it. A read's digest is
  therefore a wait's first delivery's while nothing moved between them,
  so a wait `--since` a read's digest never answers at once. Two calls
  of the same tools, under the same limits, print the same digest for
  the same document. jig's revisions move only with what a party decides
  about (jig's `client/domain/contract/documents.md`, 2), so they cannot say
  that a document changed; the digest can, and needs nothing of jig. It
  is not the mark the engine keeps per watch (jig's
  `client/domain/contract/README.md`, 5.2), which no tool prints.
- **Which reads print one:** every watched read, and every wait's
  answer. `task show TASK --tree --run` prints one for each of its
  parts, `task show`, `task tree` and `task run`, each over its own part
  of the result. `--activity`'s lines, `chat show`, `task activity` and
  `task turn` are pages, not subjects, and print none.
- **`--since DIGEST`:** answered at the first delivery whose digest
  differs. **`--until changed`,** a condition of every watched read:
  answered at the first change after the watch's first delivery.
- **`--until CONDITION`:** answered at the first delivery that meets it,
  the first delivery included; with `--since`, at the first delivery
  whose digest differs and that meets it. Conditions are named facts
  over the document:

  | Read | Conditions, beside `changed` |
  |---|---|
  | `me` | `waiting` (something waits for the party) |
  | `inbox` | `waiting`; `updates` (an unread update) |
  | `task show TASK` | `ended`; `waiting-on-me`; `held`; `replied` (a chat's run done for now); `phase:<phase>` |
  | `task tree TASK` | `ended` |
  | `task run TASK` | `ended` |
  | `task list` | none |
  | `account list` | `usable` (an account the party's work may use has a source usable on a host); `settled` (no sign-in or check the party asked is in progress) |
  | `forge list` | `settled` (no forge setting up or connecting: no set up, connect or install the deployment asked is in progress); `connected:FORGE` (forge FORGE is connected); `made:FORGE` (forge FORGE's first repository is made, `../../protocol/setup/owned.md`, 7) |
  | `change show TASK` | `pull` (its pull request opened: the panel names its number, which it keeps after the pull request closes or lands); `ci:pending`, `ci:passed`, `ci:failed` (CI at its head, as the panel says it); `held` (it is held); `landed` (its step is landed, or one after it: verifying, red, cleared); `cleared` (its step is cleared, or it ended landed and uncleared, as the panel says) |

- **A change's wait outlives its panel's watch,** as the change's page
  does (`../forge/served.md`, 2, **Its subject**): when the panel's
  watch opens `Unknown` or ends, the wait reads task N; while the task
  is live, it watches the panel again at the task's next document (a
  change the forge has not registered yet); once the task has ended
  with a change result, it opens the panel once more and judges the
  condition on that last panel, its digest the last panel's. Any other
  end answers not met, exit 3, with the last panel and its digest, and
  a task with no panel ever delivered prints `no change panel for task N`,
  exit 2. A watch the engine let go as it stopped is no end
  (below): the wait goes on through jig's client's reopening, and its
  read of task N, like every read a wait makes, waits out a link down
  within the link's deadline (`README.md`, section 9) and fails only past it, as
  the link down's line (`results.md`, 3), never as a task it could not read.
- **A change's wait follows its task,** since the panel of a change
  that ended without landing keeps its last step (`../../domain/forge/change/README.md`,
  5): the wait watches task N beside the panel, and when the task
  ends with no change result (successful no-change completion, cancelled,
  failed) it answers at once, not met, exit 3, naming how the task ended:
  `ended`, `ended: cancelled` or `ended: failed`, with the panel's last
  delivery and its digest. A successful report result establishes no-change
  completion (`../../domain/forge/change/README.md`, 2): both a read and a wait print
  `result: completed, no change` and `next: temper ctl DIR task show N`
  to reach its explanation directly, even when its parent chat's tree no
  longer lists N. JSON includes `"result":"completed, no change"`; the
  digest includes this completion detail. A panel whose step is ended
  also points to `task show N`. One with no panel ever delivered prints
  `no change panel for task N`, exit 2.
- **A task wait fills its text document before printing** through the same
  bounded specification reads as plain `task show` (`results.md`, 4). These supplemental
  words leave its completion, terminal result and digest unchanged.
- **`replied`** holds while a chat's status (jig's
  `client/domain/client/helpers.md`, its facts) is parked, waiting
  for you, waiting for them, held, closing or ended. A chat wakes on its
  person's words (jig's `domain/tasks/messages.md`, 3), so a wait begun once
  `chat start` or `task message` has answered is not met by the reply
  before them: `task message 41 TEXT --key k`, then `task show 41
  --until replied`.
- **`waiting-on-me`** covers task N's tree: what a task waits on is
  computed over its tree (jig's `domain/tasks/README.md`, section 3), so a
  question or a proposal below for the party meets it on `task show
  TASK`, and `task tree TASK` needs no condition of its own.
- **`phase:<phase>`** names the client's phase in lower case, hyphenated:
  `waiting`, `idle`, `due`, `preparing`, `claimed`, `running`,
  `backing-off`, `active`, `closing`, `held`. The word parses in any
  ASCII case (`Running`, `RUNNING`), and `backing-off` also as
  `BackingOff` and as `--json`'s `backing_off`; refusals and echoes print
  the lower-case word, and `help task show` lists the words. The ended
  phases, `ended`, `done`, `failed` and `cancelled`, are never conditions: the engine
  ends a task's watch as the task ends (jig's
  `client/domain/contract/README.md`, 5.2), so a delivery may never show one.
  `--until phase:done` and its kin are usage errors, exit 1, naming
  `--until ended`.
- **`settled`** follows a sign-in or a check, or a forge's connect or
  install, to its end. A sign-in is in
  progress from its request's answer until it ends: made, renewed,
  refused, or past the sign-in's deadline (skein's `oauth/README.md`, section
  3); a check, until each host it asks has reported (jig's
  `domain/accounts.md`, 3.4). So a wait begun once `account connect` or
  `account verify` has answered is not met before them: `account
  connect 1 --key chatgpt`, then `account list --until settled`, then
  the account's state in its answer. On `forge list`, a connect or an
  install is in progress from its request's answer until its run ends:
  connected, refused, lacking, or failed, or its link's state past its
  life (`setup.state_life_ms`, `../../protocol/configuration/README.md`); `forge
  connect 1 github --key gh`, then `forge list --until settled`, then
  the forge's row in its answer. A set up is in progress from its
  request's answer, setting up, until the forge's row leaves setting up
  and connecting: set up failed, connected, stopped, or not connected;
  `forge setup 3`, then `forge list --until settled`. `connected:FORGE` is
  met by a delivery where forge F is connected, whoever connected it.
- **`usable`** counts the party's own accounts, those whose uses name
  the party, and the deployment's (jig's `domain/accounts.md`, 4.3), on
  any host: the box's harness waits on it once it has added the
  deployment's key (`README.md`, section 8).
- **`ended`** is met by a delivery that shows the subject ended, or by
  its watch's end because it ended. For `task show`, completion is checked
  by the task's number and terminal phase as below; its answer says
  how it ended.
- **A task wait whose condition a terminal task can hold watches first.**
  These conditions are `ended` and `replied`; terminal `phase:<phase>`
  conditions are usage errors as above. A watch on a subject
  that has ended is refused `Unknown` (jig's
  `client/domain/contract/README.md`, sections 4 and 5.2). On that opening
  refusal or `SubjectEnded`, the tool reads N through the existing
  authorized task read. On both paths, a `Historical` record or a
  live-form task establishes completion only when its returned number
  is N and its phase is terminal. `Unknown` alone never means success.
  A nonterminal or wrong-number result fails visibly; it cannot succeed or
  leave the wait hanging. A valid terminal uses the ordinary result
  renderer and meets the wait when its condition holds on that task;
  otherwise it answers not met, `ended`, exit 3, with the final document
  and its digest. Unknown, hidden or forgotten history keeps status 2;
  Busy, sign-out and read failures keep their ordinary statuses (`results.md`, 3).
  The read follows the watch, so the engine's retained-load durability
  barrier leaves no gap at live retirement. Conditions no terminal task
  holds (`held`, `waiting-on-me` and live phases) take no terminal read:
  `SubjectEnded` answers not met with the last document and its digest,
  and an opening `Unknown` keeps status 2. Run/tree waits retain their
  watch-based completion; there is no additional retry or timer policy.
  A tree completing from `SubjectEnded` prints `tree ended (watch)`
  and labels its retained rows `last observed snapshot` before the tree.
  Their chip phases remain as delivered, even when live: the watch
  establishes tree completion, not terminal chip facts or a task result.
  JSON retains the tree and its snapshot digest unchanged and adds
  `"completion":{"source":"watch","ended":true}` and
  `"snapshot":"last_observed"` outside it. The digest describes the
  last observed snapshot, never a reconstructed final tree. A terminal
  delivery uses the ordinary tree form. Without any delivery, a
  run/tree watch end refuses `gone`, status 2.
- **`--timeout`:** 300 seconds by default, at most 3600; answered with
  the current document, its digest, and that the condition is not met
  (`results.md`, 3).
- **A met text inbox wait** prints the plain read's inbox (`results.md`):
  the question's words, its `read whole` call when cut, its answer line
  and the read-on call, then `met`.
- **Wait timers** are bounded by `Deadlines::with_capacity(rows)` in
  `temper-client-tools`' `jig/wait.rs`, one per admitted call. Arming a
  new timer past that capacity refuses; admitted calls keep within it.
  The tools do not price the timer table.
- **A watch that ends under a wait** (jig's `client/domain/contract/README.md`,
  5.2) answers it: met where the condition is `ended` and the subject
  ended. For `task show`, `ended` and `replied` first require the matching
  terminal read above and answer according to the condition on that task;
  refused where the party may no longer see the subject; failed
  where the sign-in ended; otherwise, not met, with the last document
  and its digest (`results.md`, 3). An engine that stops ends no watch: jig's
  client opens again a watch the engine let go as it stopped (jig's
  `client/domain/client/watches.md`, 1), so the wait goes on while the
  engine comes back, and fails as a link down past the link's deadline
  (`results.md`, 3, `README.md`, section 9).
- **Level-triggered, as the watch is** (jig's
  `client/domain/contract/README.md`, 5.2): deliveries between are coalesced,
  and the answer is the document as it is when answered. A change before
  the wait began is in its first delivery, which `--since` compares.
- **The loop:** `me --until waiting`; `inbox`; act; again. An agent
  following one chat: `task show TASK --until waiting-on-me`, or `task
  show TASK --until ended`.
- **On a box,** the forge's webhooks make the engine's documents move
  soon after the forge does; with polling alone, the same waits answer
  later (`../../protocol/setup/README.md`, section 6).

## 5. The host's tools

| Tool | What it does |
|---|---|
| `sign-in FILE\|-` | signs in by the link `temper-box serve` printed, read from FILE, or from stdin for `-`; writes `token` and `address` (`README.md`, 6.1) |
| `sign-out` | ends the sign-in and leaves `token` empty, zero bytes, read as signed out (`README.md`, 6.1) |
| `pending` | lists what DIRECTORY's store keeps: each request's operation, its party and when it was first sent, and how many records these tools cannot read |
| `help [GROUP [VERB]]` | the page `ctl`, then the tools (`testing.md`, 5); with GROUP, its rows; with GROUP VERB, its record of the table |

- **`pending` reads the store's file only:** it takes no lock, starts
  no client and sends nothing (`README.md`, 6.3).
- **A `pending` row** is a line each: the request's operation, as a
  `kept:` line names it (`results.md`, 1); its party, `party N`; and when it was
  first sent, `first sent HH:MM UTC`, the documents' words for a wall
  time (skein's `runtime/lib.md`, section 12), or `not sent`. No name and no
  key, as a `kept:` line prints none. Under `--json`: `pending`, rows
  of `operation`, `party` and `first_sent`, a wall time as jig's
  integer, or `null`; and `unread`.
- **A record these tools cannot read,** of a store version, or of a
  family's owner or version, that they do not declare, is counted, never
  listed: jig's client reads nothing of such a record, drops it at start
  and says how many (jig's `protocol/client/README.md`, section 7), and
  `pending` says what a call's client will. Its line, `N kept records
  these tools cannot read; a call that sends drops them`, follows the
  rows, and the call exits 0. A file that is not a small store's at all
  exits 1, "DIRECTORY's store cannot be read".
- **A TEXT of `-`** is read from stdin, to its end. A call takes at
  most one `-`, and none with `--call -`; otherwise it is a usage
  error.
