# What a result looks like

## 1. Text

A new settings proposal outcome names the owners of the submitted items:
`to the deployment's owners` for deployment items, `to project P's owners`
for a project's. Multiple scopes are listed once each, the deployment first
then projects by number, followed by `; one person must own every affected
scope`. Other new owner proposal outcomes print the original returned role
in the request's project. A restored kept line prints its identity alone;
read the proposal for its present recipient. Replaying the same key retains the original outcome;
`settings propose` forwards an optional reason, and `proposal decide`
on a settings proposal's acceptance sends `--running`'s choice, or stays
when it is absent, live or repeated, since jig refuses an acceptance of
settings without one; a reject or a pass-up sends none, which jig
refuses there (QA 20261027-1434-temper finding 1).

The default: plain UTF-8 lines, no colour, no wrapping. A document
prints its header line, its fields, its acts and, for a watch's
subject, its digest (`tools.md`, 4). Account lists use area 5's number-first account
and source lines, with program observations indented by host; model lists
keep their field lines. A met text inbox wait uses the plain inbox
read, including question details and continuation calls, then prints
`met`. A cut inbox question's `read whole: task activity TASK` names the
asked task whose activity holds its words, the same TASK as its
`answer: task answer TASK N TEXT` call; JSON's `read_whole` names it too.
A goal waiting on the agent:

```
task 41 · goal · Running · shop · priority 2
asked: Add a /health endpoint that reports the store's state
requester: you · spent 120 of 500 · deadline 2026-10-24 17:00 UTC
instructions: Answer on port 8080 unless the store's file names another one, and keep the /hea… (412 bytes; --json prints them whole)
waiting on: you: question 3 of task 44 · 1 waiting
plan: 4 live (2 running, 1 waiting, 1 held below)
acts:
  task answer 44 3 TEXT
  task stop 41
  task cancel 41 REASON
digest: task 7c1e09a2b44f0d13
```

- **Spend in the deployment's units,** as `me`, the settings and the
  stock state budgets (`../../protocol/configuration/stock.md`): a task's
  spent and budget print as those numbers, never scaled.
- **A duration in whole seconds,** `N seconds`, a remainder below a
  second dropped, never scaled to minutes, hours or days: the form help
  prints a bound in (`at most 3600 seconds`). `me`'s retention prints
  `retention: 86400 seconds` for a day. Under `--json`, jig's integers:
  nanoseconds (2).
- **The header** is the task's number, its kind, its phase in jig's
  words, its project and its tracking. A task `waiting on:` prints as waiting
  for `an account` (below) has `Waiting for an account` in place of `Due to
  run`. Its kind is `chat` for a
  conversation, `goal` for a goal (the task its own goal), and
  otherwise its result contract's kind: `approval`, `report`, `verdict`,
  or, for Change, `change` when its connector is the forge's (1) and
  its kind is 1. Another kind on the forge's connector is `change, `
  followed by that kind's words (`../forge/family.md`, 1); any other connector
  is `change, connector C kind K`. These are never jig's label as a
  number, which jig gives no meaning (jig's `domain/tasks/README.md`, section
  3) and temper numbers its delegates by.
  Its tracking is `priority P` while it is tracked, and `untracked`
  otherwise. A change delegate, ended: `task 2 · change · Ended ·
  project 1 · untracked`.
- **`waiting on:`** prints jig's word for what the task waits on (jig's
  `client/domain/client/helpers.md`), or the party and what it must answer. A
  held task that awaits nothing else prints its hold, where `Nothing
  awaited` would read as if nothing were wrong; one that awaits anything
  else prints that. The hold prints as `held` alone (the user, 2026-11-06):
  the document's `held:` and `way out:` lines carry its reason and what to
  do. A task that awaits nothing else, holds no hold, and whose next run no
  account serves prints jig's `an account`, where `Nothing awaited` would
  say the opposite of the lines above it. The header, this line and `--json`
  tell that one story, from one check: `--json`'s `waits_on` is `"account"`.
  Otherwise `--json` keeps `waits_on` as jig gives it, a held task's
  `nothing` among them. Its `chip.phase` stays `due` and its `run_wait` as jig
  gives it, a wait's `phase:due` still holds, and `task list`'s lines, which
  carry no next-run wait, keep their word.
- **A move** concerns who requests a task, not what it waits on: `acts:` and
  `--json` offer it on a task waiting for an account as on any other. On a
  task whose requester is already the party it isn't offered (jig's
  `domain/people/acts.md`, 1), and moving gives no task a run: an account does.
- **A hold for failed runs** has its class and no failure's text: `held:`
  prints `out of tries: run failures`, and `way out:` starts `the engine's log
  names each failure (temper help hold), then`. `temper help hold` says where
  that log is, under `temper box` and under `temper-box serve`
  (`../../domain/worker.md`, 12, on keeping the text with the hold).
- **`next run:`** prints what a run that can't start waits for, then the
  lines that say why and what to run (`tools.md`, 2). When no account serves the
  work, an account wait with neither an account nor a source and the why
  `Empty`, the label stands alone and those lines speak:

  ```
  next run:
    no account serving this work yet
  ```

  Every other wait keeps its words, an account wait that names its
  account among them: `next run: account 43 source unknown: empty`.
  `--json` keeps `run_wait` as jig gives it, a refusal's `figures` as
  `{"required":N,"offered":N}` or `null` and a bounds refusal as
  `{"refused":{"bounds":{"kind":…,"why":…,"figures":…}}}`.

  Every command those lines name is a ctl call that parses, or `temper
  help NAME` with NAME a page, or the program login's command (the
  request `ui-wait-words`, 2026-11-07). A host's wait names none: it starts
  by itself.

  ```
  next run: host
    no host has room, or none is connected; it starts when one does
  ```

  A run no worker can ever take (jig's `Refused(Bounds)`, `jig/docs/design/client/domain/contract/documents.md`,
  1) says which bound keeps it out in the client's form
  (`../pages/accounts.md`, 2), lower case, a count as digits, a duration in
  milliseconds as `N ms`:

  ```
  next run: refused, no worker takes it: agent turns too few for charter 11
    no worker can take this: its charter asks 100000 turns, and the largest worker takes 800
  ```

  With no bound named, `next run: refused, no worker takes it` and `no
  worker can take this: each falls short of another bound its charter
  asks`. A refused hello with figures keeps its code line and says the
  same of its worker, `no worker can take this: charter 11 asks 100000
  turns, and the worker refused at 10:42 takes 800` (its UTC clock), in
  place of `its host refused this start: …`; every other refusal keeps
  that line. A shortfall of turns, time or waiting names its charter as
  checks does, `agent turns too few for charter 11`, and the checks'
  pointer is `see temper help settings for held revisions`.
- **The deadline** has its date and its zone: `deadline 2026-10-24
  17:00 UTC`, or `no deadline`.
- **Its instructions,** when the document carries them, on a line of
  their own: whole when they are one line of at most 80 characters;
  otherwise their first line cut at 80 characters, then `…` and how
  many bytes they hold and where to read them whole, `(N bytes; --json
  prints them whole)`. They are the task's instructions in force, its
  charter's or an amendment's, and the document does not say which
  (`README.md`, section 12). With none, no line. A core-cut instruction preview names the
  held byte count; JSON also holds only that prefix. A cut specification or
  instruction prints `read specification:` or `read instructions:` and its
  `task text TASK --field FIELD --from 0` call. Task JSON carries these calls
  under `text_reads`. `task activity` prints the two source reads before its
  events: the brief is outside the activity log.
- **Its settings revision,** for an agent task, after its instructions:
  `settings: revision 7`, the revision its runs read their charter
  from, jig's `Task.settings.task` (jig's `domain/engine/settings.md`, 3);
  then ` · its run on 6` while a live run was prepared under another,
  and ` · newest 9` when the newest is another, which `revision advance
  7` moves it to if the newest offers its charter and allows its LLM
  choice (17.4). A procedure's or a person's task prints no line.
- **Its dependencies,** when it has any, after its instructions: a
  line `dependencies:`, then one indented line per dependency, in the
  document's order: `task M`, its state as a tree's chip says it, or an
  ended one's supplied status (`Completed`, `Failed`, `Cancelled`), or
  none; then, for one added by an amendment (its `dependency_facts`,
  jig's `Dependency.added`), ` · added by WHO at 2026-10-22 05:10 UTC`,
  WHO as the requester is said (`you`, `party 3`, `task 7`,
  `deployment 1`); then, while it keeps the task (jig's `keeps`, jig's
  `domain/tasks/lifecycle.md`, 8), ` · keeps task N from ending`:

  ```
  dependencies:
    task 43 · Completed
    task 44 · Running · added by you at 2026-10-22 05:10 UTC · keeps task 41 from ending
  ```
- **A resource** in the forge's words (`../forge/family.md`, 1): `resource:
  branch temper/1/c2 of repository 1 on forge 1`, never its bytes.
- **A result** in temper's words, never its record: `result:`, its
  state as the card says it (`../view.md`, 5: `Landed`, `Completed`,
  `Failed`, `Cancelled` (cancelled or withdrawn), `Ended`), then ` · ` and its
  words. A landed change's words are its merge commit, whole, as the
  engine's landing words give it (the card shows seven characters; an
  agent may need the id), after its pull request, the result's
  resource: `result: Landed · pull request #7 · merge commit 3f2a9c1…`
  (forty characters), the pull request by its number, as the result's
  settled resource gives it (its repository and host too where the
  resource is a path, `../forge/family.md`, 1), then the call that reads its
  panel, `change show TASK`; the id is the words' first line. The
  pull request line, after any verification words, with its address
  (`../../domain/root.md`, section 5), prints as `pull request: ADDRESS` when the task's document
  carries it whole: a line the document's text window cut prints
  nothing, and `change show TASK` has the address. Verification words
  immediately after the id remain readable: `Cleared by TIER run N.`,
  or why it is uncleared. Within ctl's text budget, their evidence takes
  room before the optional pull address. Retained results with the pull
  line immediately after the id and verification after the landing line
  render alike when the document carries those lines. A change's words that
  are not an id print as they are. A report's words, a verdict's, an approval's and a choice's print
  as they are; a connector's its summary and its part in the forge's
  words; a failed or cancelled task's its reason, if it gave one. Words
  of more than one line go on the lines after it, each indented two
  spaces. The result's details and acts are under `--json` only. An
  ended task's document, a read of an ended task and a wait's ended
  task print it alike.
- **An activity line** is its position, then what happened. A
  milestone's, `phase changed`: the activity names only the position of
  the task's history row that says which phase, and no tool reads that
  history (`README.md`, section 12).
  An invalid run's end prints `run N ended: its result broke its
  contract (KIND)`, where KIND is form, verdict, words, change or
  followups. An invalid end with no recorded contract kind prints
  `run N ended: its result couldn't be read`. JSON preserves the cause
  as the failed end's optional `invalid_reason`, with `unreadable result`
  for the latter. Other failure classes omit the cause. These words do
  not change the class or the retry counts (jig's `protocol/client/wire.md`, 1).
  A heard `QuestionAnswered` notice prints `POSITION · You answered
  question N in your inbox`, including when its words are empty.
- **Acts as calls,** without the `temper ctl DIRECTORY` prefix; a keyed
  act's call ends with `--key NAME`. A proposable act says to whom; a
  refused one says why, and prints no call. The host gives the
  rendering how a call is printed. A bounded act prints its bounds after
  its call, but for `task amend`, which prints its call alone: its whole
  authority, over a kilobyte, would bury the words above it, and what the
  bounds decide is the tool's when it sends (`areas.md`, area 2). What lies
  beyond them goes with them, since `proposed to` beside a bare call
  would say the whole act asks another party first. `--json` prints the
  admission whole.
- **An offered act with no tool** (`tools.md`, 3 has none for it) prints its name
  in the tools' words and ` (no tool)`, in the document's order, with no
  call and no admission: the agent learns the act is there and may hand
  it to a person, and no call names a tool that does not exist. Under
  `--json`, the document's offers print as they are.
- **jig's words and the forge's** (`README.md`, section 4): phases, holds, what a
  task waits on and refusals in the words a person reads.
- **Lists** print one line per row, its number first, then the cursor
  for the next page, or that there is none.
- **A tree** prints `tree:`, then its root and each row its window
  holds, a chip a line. The engine answers the tools' window within its
  own rows (jig's `client/domain/contract/README.md`, 5.2), so a tree larger
  than either folds the rest. Its text prints one total line after the
  rows: `cut: 3 more not shown (1 Waiting, 2 Running)`, the folded tasks'
  count, then each phase that has any, in a chip's words and jig's order
  of phases. Then one `read on with: task list --project 1 --parent 41`
  line per distinct fold parent lists that parent's delegates, with the
  project's number from the root's document. A tree's window opens at
  its start only (jig's wire carries no cursor for it), so read on with
  the folds' parents' delegates, a page at a time by `--after`, and each
  one's tree by `task show TASK --tree`. With nothing folded, no cut or
  read-on lines. Under `--json`, `more` and `folded` as the document
  carries them.
- **An activity page** cut before its first row (jig's
  `protocol/store/loads.md`, 3) prints no rows and its `read on with:` call,
  never `no more`; under `--json`, `"cut":true` and the position to read
  on from.
- **An act's outcome** prints what was made or decided and its numbers:
  `done: task 41, chat`; `proposed: p3/17, to maintainers`; `decided
  first: by ada, accepted, 14:02`; `removed: project 3, fabrig`; for
  `repository adopt`, `../forge/family.md`, 6's words. A removal kept
  from an earlier call, which knows no name, prints `kept: remove-project,
  removed: project 3`, its number from the answer.
- **What a call sends first** (`README.md`, 6.2, step 4) prints on stderr, a line
  each: `kept: `, the request's operation, jig's or the forge's (`stop`,
  `adopt`), then its outcome as an act's prints, which names what it
  made or decided. No name and no key: a request's record keeps the
  engine's key, drawn or a name's digest, and the client's, the
  runner's (jig's `protocol/client/README.md`, section 7), never the name,
  and neither key is of use to an agent.
- **A restored proposal's `kept:` line** prints its identity alone,
  `kept: chat, proposed: t3/17`, with no recipient: jig keeps none with
  the request, so the line is never stale, and the agent reads the
  proposal for its present one. A new call's outcome keeps `, to
  <recipient>`.
- **`adopt`'s `kept:` line** has two forms: `kept: adopt WORDS`, with no
  comma, for an answer in `../forge/family.md`, 6's words, unknown among them,
  or an answer that could not be decoded; and `kept: adopt, REASON`, as
  every other operation's, for the client's own answers: held, the
  limit, unconfirmed, not kept.
- **What it does not send** prints a line on stderr too: another
  party's kept requests, `kept: N for party P, unsent while it is not
  signed in here` (`README.md`, 6.2); the records the client dropped at start,
  `dropped: N kept records these tools cannot read` (`tools.md`, 5).

## 2. JSON

`--json` prints the result record instead: one JSON object on stdout,
written with `skein-json`'s writer, the result's fields by their names
in the table, in snake case, numbers as jig's integers, no floats. The
JSON is the contract for scripts; the text follows temper's words, and
may change with them.

- **`chat show` leaves tool payload fields null:** a call's `input` and
  `outcome`, and a result's `summary`. Its conversation words retain the
  turn part records and their continuations. `task turn` serves payloads
  with the same part numbers.

- **A task's document** prints its fields as jig's client gives them:
  its `dependencies` as links, and beside them its `dependency_facts`,
  one object per dependency, `number`, `pending`, `added` (`null`, or
  `at`, a wall in nanoseconds, and `by`, the requester as `requester`
  prints it), `keeps` and `status`.
- **`help --json`** prints one JSON array, whatever it names: with no
  group, each group and each of the client's tools by its name and line
  (its whole records are `help NAME --json`'s); with GROUP,
  every record of the group; with GROUP VERB, every record of that name
  (`settings show` has two). Each record is whole: its name, line,
  every form with every argument, result, keyedness and notes.
- **`--call`'s arguments** are named as the table names them: `proposal
  decide`'s are `proposal`, and `task decide`'s `task` and `hold`, then
  `outcome` and `reason`.
- **The `kept:` lines,** under `--json`, are one JSON object a line on
  stderr: `operation` and `outcome`, as the act's; another party's,
  `party` and `unsent`; the dropped, `dropped`.
- **A project made public,** under `--json`, prints `tools.md`, 3's words on stderr
  as one JSON object a line, `may_show`, beside its record on stdout.

## 3. Exit status

| Status | Meaning | Prints |
|---|---|---|
| 0 | a read answered, an act done or proposed, a wait's condition met | the result |
| 1 | the call failed: usage, the directory refused, its store unreadable (`tools.md`, 5), or busy past the lock's wait (`README.md`, 6.3), the engine's address not on loopback, nothing listening at the engine's address, the engine unreachable or the link down past its deadline, signed out, a request past the client's limits, a name the client holds already (`Held`, `README.md`, section 5), unknown past the retention, not kept (DIRECTORY's store not written); the client's own answers alike, a read past its limits among them; a wait's watch ended by the sign-in's end | one line on stderr, `temper ctl: ...` |
| 2 | refused by the engine, and only by it: a key conflict among them (`README.md`, section 5); a wait's subject no longer visible to the party | jig's refusal, typed in JSON, in words in text; the act's alternatives where the refusal names them (read again, propose instead) |
| 3 | a wait's timeout, or its subject's watch ended, its condition not met | the current document, or the last, and its digest |
| 128 + N | ended by signal N, `SIGINT` or `SIGTERM`: 130 or 143 (`README.md`, 6.2) | nothing more |

- **An argument refused before a call** names the tool, the argument
  and what it takes from the same specification as help, including its
  bound: `temper ctl: task activity: --limit COUNT takes a count from 1 to 32; temper ctl help task activity`.
  The line is at most 240 bytes, ends with the tool's help call, and is
  the same for argv and `--call`. Missing, repeated and excessive
  arguments name the rule they broke. A call that names no tool keeps
  the usage line; an unknown group or tool names it and where help
  lists it.
- **2 is the engine's answer alone:** what it decided about the
  request, with the alternatives it names. The client's own answers
  are 1. Two of them never reach a call: unconfirmed, since the runner
  sends nothing before `me` is live (`README.md`, 6.2, step 3), and the client's key
  conflict, since each call's keys are its own (`README.md`, section 4). Either is
  the tools' bug: it exits 1, says so, and the page `ctl` says to report
  it as a finding (`testing.md`, 5).
- **A task wait's final read** follows `tools.md`, 4 on both an `Unknown`
  opening and `SubjectEnded`: status 0 requires N and a terminal phase,
  for both `Historical` and live-form records. Unknown or hidden
  history remains status 2; a nonterminal or wrong-number result fails
  visibly, never reports completion. Busy, sign-out and other read
  failures keep their ordinary statuses.
- **An act refused for want of a retained answer's place** (jig's busy,
  *answers full*, with the table's size and its oldest answer's expiry:
  jig's `domain/people/acts.md`, 1) prints `temper ctl: the engine holds its
  N answers of the last day; room again at TIME` and exits 2, never
  retrying past its patience when TIME is past it; nothing was made,
  and the same key sends it again once there is room. The carried
  export path (an account's give-back, `areas.md`) says the same, never a bare
  `unsent` (QA 20261027-1202-temper finding 3).
- **A call cut by a signal** ends at once and exits 128 plus the
  signal's number, never 0 (`README.md`, 6.2).
- **A call whose link stays down** past its deadline exits 1, and what
  it sent stays in the small store, for the next call that sends (`README.md`, 6.2).
- **Nothing listening says so at once.** A call whose connection to the
  engine's address is refused, before the engine has answered anything in
  this call, exits 1 then, with no deadline waited out: `temper ctl:
  nothing listens at ADDRESS; is temper running?`. What it sent stays in
  the small store, as above. The sign-in's call is such a call. Once this
  call has heard the engine, a refused connection is the link down, and
  its deadline holds, so that a wait outlives the engine's restart. An
  engine that takes the connection and stays quiet takes the deadline too,
  and the line says what it waited for (below).
- **The engine's watches all in use say so.** A call whose client still
  holds a watch refused busy at its deadline (jig's
  `client/domain/client/watches.md`, 2) exits 1 with `temper ctl: all the
  engine's watches are in use, by open pages and waits; close one, or
  wait and call again`. It comes after the busy sign-in's and the full
  streams' lines, which are nearer the cause, and before any other line
  of the deadline. A call that gets its answer first prints no such
  line, since a closing page gives its room back
  (`../../protocol/configuration/limits.md`).
- **The deadline says what it waited for.** A call past the link's
  deadline (`README.md`, section 9) exits 1 with one stderr line, at most 240 bytes,
  which names what was outstanding and holds no request's words and no
  secret. *Busy*, *streams full* (`README.md`, 6.2) and all watches in use keep their
  own lines, ahead of these. A watch refused not ready also takes
  precedence: its line says the engine is not ready and points to
  `temper-box serve`'s log for why (`../../protocol/README.md`, 6.2), rather
  than claiming all watches are in use. A sign-in's or a sign-out's call
  says
  `temper ctl: the engine at ADDRESS did not answer within 30 s`. Any
  other says, from the link's
  state (jig's `client/domain/client/watches.md`, 2):

  | Outstanding | When | The line, after `temper ctl: ` |
  |---|---|---|
  | the link | no connection taken in this call, or the link *offline* | `the link to the engine at ADDRESS did not come up within 30 s` |
  | the stream's first answer | a connection taken, *starting*, nothing heard | `the engine at ADDRESS took the connection but its stream did not answer within 30 s` |
  | a read's answer | *starting* with the engine heard, or *live* in a read or a wait | `the engine at ADDRESS did not answer a read within 30 s` |
  | an act's outcome | *live* in an act that has no answer yet | `the engine at ADDRESS gave no outcome for this act within 30 s` |
  | an act's outcome, the engine having answered it *busy* | as above, its last retry's why (jig's `client/domain/client/requests.md`) | `the engine at ADDRESS last answered this act busy; no outcome within 30 s` |
  | an act's outcome, *not ready* | as above | `the engine at ADDRESS last answered this act not ready; no outcome within 30 s` |
  | an act's outcome, its answer lost or undecodable | as above | `the last answer to this act from the engine at ADDRESS was lost; no outcome within 30 s` |

  A call that acts adds what is true of its act, on any of those lines,
  until the act has answered; a first repository's watch (`tools.md`, 4) has its
  answer, and adds nothing:

  | The act at the deadline | Added |
  |---|---|
  | not sent: not yet entered, or its record not yet stored | `; it was not sent` |
  | kept, under a `--key` the call names | `; it is kept, and the same --key may be tried again` |
  | kept, under a name the table drew | `; it is kept, and the next call that sends tries it again` |
  | sent once and never kept (`README.md`, section 5), under a `--key` the call names | `; it is not kept, and the same --key and file may be tried again` |
  | sent once and never kept, under a name the table drew | `; it is not kept, and calling again is a new request` |

  A same-key retry is safe by `README.md`, section 5, which is why these lines may
  offer it. The deadline stays the user's 30 s. A line naming a retry's
  why is the call's explicit pending result: its request is kept, and it
  was still being tried (`README.md`, 6.2).
- **An account's move** (`areas.md`, area 5) keeps the rule: 2 is the
  engine's refusal alone, "not signed in", "not ours", "held",
  "moving" and "cancelled"; FILE's faults are the tool's own, exit 1,
  nothing sent: `account export`'s FILE already there, `account
  import`'s FILE not private, a link, empty, past its bound or of a
  newer form than it reads; so are a record not
  written and an answer lost or unsent, exit 1, each line saying what
  the engine does with the account.
- **Only `sign-in` creates DIRECTORY.** Three of 1's lines, word for
  word: `temper ctl: DIRECTORY does not exist`, for any other call on a
  DIRECTORY that is not there; `temper ctl: DIRECTORY is not signed
  in`, for a call that needs the token when DIRECTORY holds none; and
  the deadline's lines (above), past the link's deadline (`README.md`, section 9),
  ADDRESS as `address` holds it.
- **`repository adopt`:** adopted exits 0, with branch protection
  unknown or not (`../forge/family.md`, 6, prints "temper's forge user may not
  read the branch protection: the forge shows it to repository
  administrators only." then); unknown past the retention exits 1; each
  other outcome there, not found, no access and refused among them,
  exits 2, in its words.

## 4. Bounded

A verdict result in `task show` names `verdict CODE` with its reason.
`task activity` likewise names the verdict code in heard and notified results,
including when the reason is empty. A numbered person-task answer reaches its
requester with that code in both starting-run and live-message text. The host's
`read_task` result names `Verdict: CODE` separately from the reason's byte window;
its `from` and `next` offsets count reason bytes alone.

- **Every result has a ceiling,** in rows and bytes: a list's page, the
  activity lines `task show` adds, a conversation's page, a turn's part. A
  text cut prints how much was left and the call that reads the rest.
- **`me` prints a `cut:` line** in text and a `cut` field in JSON. It is
  true for supplied document loss, omitted provider/program rows, a cut
  provider name, and any free text lost earlier in the document. The
  provider offers include their own `providers_cut`, `name.cut` and
  `programs_cut` facts; the document flag combines these losses.
- **Program identifiers and manage URLs are whole or omitted.** If one
  cannot fit the text budget, JSON prints null and text omits its bytes;
  the document's cut flag is true. A displayed prefix is never a target.
- **Text `task show`, including waits, fills its `asked:` words** with
  sequential specification reads when the overview is cut, up to the smaller
  of the part and text bounds and the result byte ceiling. Rendering shares the result's text
  budget; any undisplayed source ends with a `task text TASK --field
  specification --from OFFSET` continuation. A refused or nonadvancing
  chunk stops retrieval and keeps a continuation. A wait retains its validated
  terminal task and result, or its meeting delivery, while retrieving these
  words; its status and `met` envelope stay those of that completion. JSON keeps
  the bounded overview and its digest; supplemental text never changes the digest.
- **Long texts are paged, never whole by default:** `chat show`, `task
  activity`, `task turn` and `task text` take a cursor or an offset.
  `task text` reads the source rather than the overview's prefix. It prints
  the source chunk, its starting byte offset and total size; a cut ends with
  `N bytes left; read on with: task text TASK --field FIELD --from OFFSET`.
  JSON carries `task`, `field`, `from`, `total`, `text` and `next` (null at
  the end). `--field result` retrieves the complete kept terminal report,
  including one retained with cancellation, without expanding task overviews.
  A cut report in text `task show` prints `read result: task text TASK --field
  result --from 0`; JSON names the same call under `text_reads.result`, beside
  its bounded typed result. Follow continuations until `no more`; a task without
  a kept report is unknown. Offsets keep characters whole. Instructions are the task's
  override or its charter at the task's recorded settings revision; unavailable
  instructions are unknown. Reads reflect current state; restart at zero after
  an amendment or settings move. An agent's
  context is the
  budget the defaults are chosen for.
- **A cut page says so first.** A paged read is a read whose own call
  takes `--from`: `task text`, and `task turn` with `--part`. Its page
  is cut when it starts past byte 0 or leaves bytes after it. A cut
  page prints its position as its first line, before anything else the
  read prints, as well as at its end: `FIELD page N of M holds B bytes;
  L bytes left; read on with: CALL` for `task text`, and `part P page N
  of M holds B bytes; L bytes left; read on with: CALL` for `task turn
  TASK TURN --part P`, ahead of its `turn A.T` line. CALL is the call
  the page's last line prints. On the last page `no more` takes the
  place of the bytes left and the call. B is the source bytes the page
  holds and L those after it. N is ⌈from ÷ C⌉ + 1 and M is N + ⌈L ÷ C⌉,
  where C is the most source bytes the read's page may hold, at least
  1: for `task text`, `README.md`, section 9's page, within a result's text less
  512 bytes and within a result's bytes less 512 at six bytes each;
  for a part, the smaller of the page and the result's text room when
  the read starts. A page cut short, to keep a character whole or
  because retrieval stopped, is counted the same way, so a later page
  may count one more. An uncut text, a whole source from byte 0 or a
  whole part, prints no such line. An 84,336-byte specification reads
  in two pages:

      specification page 1 of 2 holds 65536 bytes; 18800 bytes left; read on with: task text 240 --field specification --from 65536
      specification from 0 of 84336 bytes:
      ...
      18800 bytes left; read on with: task text 240 --field specification --from 65536

      specification page 2 of 2 holds 18800 bytes; no more
      specification from 65536 of 84336 bytes:
      ...
      no more

  These are the tools' words, the same for agents and people. JSON is
  unchanged: `from`, `total` and `next`, and a part's `from`,
  `bytes_left` and `read_on`, carry the position. A read that prints a
  cut text inside a larger answer, `task show`'s asked words, `task
  turn` without `--part`, `task activity` or `chat show`, is no paged
  read: each cut keeps its end line alone, whose call is a paged read.
- **The numbers** are `README.md`, section 9's. A path is never cut: the tools
  print it whole, within the client's bound on a path's segments
  (`jig-client-domain`'s `Limits`, `segments`); nor is an authority,
  within its bound on an authority's entries.
