# The ledger

Provisional, 2026-11-12, from `../../draft/landed-qa.md`, sections 4, 7,
8 and 9, milestones Q1 and Q3 of `../../draft/landed-qa.execution.md`.
The one record of landed QA's state for a project: its landings, cases,
builds, agent sessions, passes, findings, notes and lines, kept by code,
written through its operations alone, by the loop and by the agents'
commands.

## 1. In one page

- **One document,** `ledger.json` in the home (`README.md`, 3.1), read
  and written whole under a lock (section 3).
- **Its records** (section 2): main's last head, landings in main's
  order, cases with the landings that asked for them and their outcomes,
  kept builds and the pass holding each, agent sessions with their time
  and tokens, passes with their requests, findings with their rounds,
  the agents' notes, and lines.
- **Its rules** (section 4) hold whoever writes: triage in main's order,
  one answer kind per landing, a case merged into the queued one for its
  page, a build deleted only once no pass can need it, a case in one
  pass at a time, a session's end deciding each case's state, a pass
  resumed on a build holding its goal's landing, and a finding handed on
  after two goals.
- **Its refusals** (section 5) are the agents' only feedback: each names
  what it refused, and nothing is recorded.
- **`show`, `lines` and `report`** (section 6) print it for a person.

## 2. The document

One JSON object. Times are whole seconds since the Unix epoch; numbers
of landings, cases, sessions, passes, findings, notes and lines count
from 1 in the order they are recorded and are never reused. Before
v0.1.0 it changes in place, as every stored format does: a later
milestone adds records and fields, and a document a newer build cannot
read is refused by name. Every field Q3 adds is absent or empty in a
document Q1 wrote, so such a document reads as it is.

### 2.1 The head

`head`: the last commit of main the watcher read (`landings.md`, 3),
absent before its first read.

### 2.2 Landings

`landings`, in the order the watcher recorded them, main's order
(`landings.md`, 3). Each:

- `number`; `commit`, its merge's; `goal`, the goal's task number;
  `pull`, its pull request's number where the merge names one; `title`;
  `seen`, the time it was recorded;
- `answer`: absent until triage answers, then `none` with its `why`, or
  `cases`;
- `triaged`: absent, or the time it was triaged (4.4), with `unanswered`
  true where its triage recorded nothing twice;
- `sessions`: how many triage sessions ended for it with no answer.

### 2.3 Cases

`cases`. Each:

- `number`; `surface`, `web` or `tui`; `page`, a page of that surface
  (`triage.md`, 3);
- `asks`, at least one, in the order recorded: each the `landing` that
  asked, its `setup`, what it `adds` to the setup (possibly empty), and
  its `steps`, each a `do` with its `expect` and, where the page shows it
  word for word, `see` (`triage.md`, 4);
- `state`: `queued`; `assigned`, taken by a pass that has not yet ended
  it; `passed`; `failed`; or `blocked`, blocked twice and out of the
  queue (4.12);
- `pass`: the pass it is assigned to, failed in or passed in, absent
  while `queued`;
- `blocks`: how many pass sessions ended it blocked, absent for none;
- `outcomes`: each outcome recorded for it, in order, each its
  `session`, `time`, `result` (`passed`, `failed` or `blocked`),
  `notes` and `findings`, the numbers it names, possibly none (4.10);
- `queued`, the time of its first ask; `line`, true once its wait for a
  build has been a line (4.7).

A case's landings are its asks' landings, each once.

### 2.4 Builds

`builds`, in the order kept. Each:

- `commit`; `kept`, the time; `state`, `kept` or `deleted`, with
  `deleted`, the time;
- `holds`: absent until the watcher has read a head that holds the
  commit, then the numbers of the landings whose merges the commit holds
  (`landings.md`, 5); `order`, set with it, the number of commits the
  commit holds, so that of two kept builds the newer has the greater;
- `held`: absent, or the number of the pass holding it (4.8, 4.12 and
  4.13).

A deleted build's record keeps its `holds`, so that the landings since a
copy's build stay known (`places.md`, 7).

### 2.5 Sessions

`sessions`. Each:

- `number`; `kind`, `triage` or `pass`; `landing`, for triage; `pass`,
  for a pass; `effort`;
- `started`; `ended`, absent while it runs; `exit`, how it ended: its
  code, its signal, `bound`, `lost` or `unstarted` (`triage.md`, 2 and
  6; `passes.md`, 3 and 6); `tokens`, where its command reported them
  (`triage.md`, 5);
- for triage, what its commands check against, given at its start:
  `pages`, the pages of each surface, and `setups`, the catalogue's
  names (`triage.md`, 3).

A session's time is `ended` less `started`. A pass session has recorded
something once an outcome, a finding, a check or a note names it.

### 2.6 Lines

`lines`. Each: `number`, `time`, `area`, `words`, and `given`, true once
given to `notify` (`README.md`, 5).

### 2.7 Passes

`passes`. Each:

- `number`; `place`, the place's directory as `qa.json` writes it
  (`places.md`, 2); `build`, the commit of the build it holds or last
  held; `state`, `running`, `waiting` or `ended`; `started`; `ended`,
  the time it ended;
- `cases`: the numbers of every case it took, in the order taken, at its
  start, at a resume, by a join (4.15) or moved to it (4.12);
- `sessions`: its sessions' numbers, in order; its first is its start's,
  each later one a resume's;
- `requests`: in order, each made at a session's end (4.16):
  - `number`, from 1 within the pass; `time`, when made;
  - `findings`: the open findings it carries, possibly none;
  - `recipes`: the names of the setups it carries, each added or changed
    in the pass's setups since its build's commit (`passes.md`, 6),
    possibly none;
  - `filed`: absent until the deployment's command put it on main, then
    the time (`findings.md`, 5);
  - `goal`: the task number of the goal started on it, absent for a
    request with no finding and until filed;
  - `ended`: absent, or the time the deployment said its goal ended
    without landing (`findings.md`, 6).

A pass's goal is its last request's.

### 2.8 Findings

`findings`. Each:

- `number`; `pass`; `case`; `session`; `time`;
- what its agent gave (`findings.md`, 2): `title`; `severity`, `low`,
  `medium` or `high`; `area`; `setup`; `lines`, the drive lines from
  that setup that reproduce it; `expected`; `seen`; `evidence`, the file
  names kept under `evidence/N/` in the home;
- from its case, when recorded: `surface`, `page` and `landings`;
- `state`: `open`; `linked`, with `to`, the open finding it matched
  (4.11); `closed`, with `closed`, the commit of the build it no longer
  held on, and `closed_time`; or `handed`, handed on after two goals
  (4.14);
- `goals`: the goals of the filed requests that carried it, in order;
- `checks`: each check of a resumed session, in order: `session`,
  `time`, `commit`, the build it ran on, `holds`, true or false, and
  `notes`.

### 2.9 Notes

`notes`, what a pass's agent noticed off the green path
(`../../draft/landed-qa.md`, 8). Each: `number`, `pass`, `session`,
`time`, `words`.

## 3. Its lock and writes

- **Every operation that records** takes an exclusive lock on
  `ledger.lock` (`std::fs::File::lock`), reads `ledger.json`, applies
  itself, writes the whole document to a temporary file in the home,
  renames it over `ledger.json` and releases the lock. A refusal writes
  nothing. A read alone (`show`, `lines`, `report`, a holder's watch)
  takes the lock too, so it never reads half a write.
- **An absent `ledger.json`** is an empty ledger; one that does not
  parse is refused, `ledger.json: WHY`, and nothing is written over it.

## 4. Its rules

### 4.1 Triage's order

- **The next landing** is the first landing in main's order with no
  `triaged`. A triage session starts only for it, and only when no
  triage session is running.
- **A session's commands** are refused unless the session is running
  and its landing is still the next one, so a landing's cases are
  recorded only once the one before it is triaged.

### 4.2 Answers

- **No QA** (`triage none WHY`): `why` nonempty; refused where the
  landing has a case.
- **A case** (`triage case FILE`): refused where the landing is answered
  no QA, or where the case does not fit (section 5).
- **Repeated:** a second no QA replaces the first's `why`; cases add up.

### 4.3 A case and its merge

- **Checked** against the session's `pages` and `setups` (2.5): its
  surface `web` or `tui`; its page one of that surface's; its setup one
  of the catalogue's; at least one step, every step a nonempty `do` and
  a nonempty `expect`.
- **Joined to a waiting pass** where 4.15 says, before any merge.
- **Merged:** where a case in state `queued` has the same surface and
  page, the new ask joins it, and the case keeps every landing that
  asked for it (`../../draft/landed-qa.md`, 4). Otherwise it is a new
  case, `queued`, at the ask's time.
- **Its answer:** the command prints `case N queued`, `case N joined:
  landings L1, L2`, the case's landings in order, or 4.15's.

### 4.4 A triage session's end

Ended by the loop (`triage.md`, 6), with its exit, tokens and time:

- **Its landing answered** (no QA or a case): the landing is triaged at
  that time.
- **Not answered:** the landing's `sessions` count rises. At two, the
  landing is triaged as `unanswered` and a line says so: `landing L
  (goal G, COMMIT12): triage recorded nothing in two sessions; triaged
  as unanswered`, COMMIT12 the merge's first twelve hexadecimal digits.
  Below two it stays the next landing, and the loop starts another
  session.
- **Unstarted** (`triage.md`, 2): the landing is left as it was, its
  count unchanged.

What a session recorded stands, however it ended.

### 4.5 Triage has reached a build

A kept build is reached once its `holds` is known and every landing it
holds is triaged (`../../draft/landed-qa.md`, 4).

### 4.6 A build's deletion

A kept build may be deleted when no pass holds it, triage has reached
it, and either a newer build (greater `order`) that triage has reached
is kept, or none of its held landings is a landing of a `queued` case
(`../../draft/landed-qa.md`, 3). Nothing else deletes one. So what is
kept is the newest reached build while cases wait for it, the builds
passes hold, and the builds triage has not reached.

### 4.7 A case waiting for a build

A `queued` case waits for a build while no kept build that triage has
reached holds every one of its landings. One that has waited
`bounds.build_wait` seconds since `queued` is a line, once: `case N (PAGE
on SURFACE; landings L1, L2) has waited past S seconds for a kept build
holding its landings`, S the bound.

### 4.8 A pass's start

The loop's third step (`passes.md`, 3), for a free place, once no
waiting pass resumes there (4.13), with `qa.json`'s `surfaces`:

- **A build it may take:** kept, reached by triage (4.5), held by no
  pass, and holding every landing of at least one `queued` case of a
  surface in `surfaces`.
- **It takes,** in one write: the one with the greatest `order`; a new
  pass, `running` at the place, its build that one, which it holds
  (`held`); every `queued` case of a surface in `surfaces` whose
  landings the build holds, `assigned` to the pass, in case order; and
  its first session, `running`. So no case is in two passes
  (`../../draft/landed-qa.md`, 9).
- **None:** no pass starts, and nothing is written.

### 4.9 A pass session's commands

- **Refused** unless the session is running, is a pass session, and its
  pass is `running`.
- **An outcome or a finding** only for a case `assigned` to the
  session's pass; **a check** only for an `open` finding of the
  session's pass, in a session that is not the pass's first.
- **Each recorded at once,** so a session ended by its bound or a crash
  loses nothing it recorded.

### 4.10 Outcomes

`pass outcome CASE RESULT NOTES [--finding N]...` (`passes.md`, 5):

- **RESULT** `passed`, `failed` or `blocked`; NOTES one argument,
  nonempty for `blocked`, which says why, and possibly empty otherwise.
- **Findings named** must each be the case's: recorded for it, by any
  session of the pass. `failed` names at least one; `passed` and
  `blocked` may name some, a finding beside the case
  (`../../draft/landed-qa.md`, 13).
- **Appended** to the case's `outcomes`; the case's state changes only
  at the session's end (4.12), from the last outcome the session
  recorded for it.

### 4.11 A finding and its match

`pass finding FILE` (`findings.md`, 2 and 3):

- **Checked:** its fields as section 5 refuses them; its setup `fresh`
  or a recipe of the pass's setups.
- **Matched** against every `open` finding, of any pass: the same
  `setup` and the same `lines`, or the same `surface`, `page` and
  `expected`, the last compared with surrounding whitespace trimmed.
  Matching one, it is recorded `linked`, `to` that finding (the oldest,
  where several match); else `open`.
- **Its answer:** `finding N recorded`, or `finding N linked to open
  finding M of pass P`.

### 4.12 A pass session's end

Ended by the loop (`passes.md`, 6), with its exit, tokens and time, in
this order, in one write:

1. **Recorded nothing** (2.5): every case assigned to the pass with no
   outcome in this session is taken as blocked by it, notes `the pass's
   session recorded nothing`, appended to its outcomes; and a line:
   `pass P (PLACE, build COMMIT12): session N recorded nothing; its
   cases count as blocked`.
2. **Each case assigned to the pass,** by the last outcome this session
   recorded for it:
   - `passed`: `passed`.
   - `failed`: `failed`, in the pass, where one of its findings is the
     pass's own and `open`; else, where its findings are `linked` to
     open findings of one other pass that is not `ended`, `failed` and
     moved to the pass holding the oldest of them, joining its `cases`,
     to run at that pass's resume; else `queued`, `pass` absent,
     uncounted.
   - `blocked`: its `blocks` raised; at two, `blocked`, out of the
     queue, and a line: `case N (PAGE on SURFACE; landings L1, L2)
     blocked twice: NOTES`; below two, `queued`, `pass` absent.
   - **none:** as it was before the session, so `failed` in the pass
     where it was a resumed pass's failed case and one of its findings
     is still open; otherwise `queued`, `pass` absent, uncounted
     (`../../draft/landed-qa.md`, 9).
3. **Findings' rounds,** in a session that is not the pass's first, as
   4.14 has them.
4. **The pass,** then:
   - **with an open finding of its own:** `waiting`, still holding its
     build; a request made (4.16) of all its open findings and the
     setups it changed;
   - **without:** `ended` at that time, its build released (`held`
     absent); a request made of the setups it changed alone, where it
     changed any.

   Either way the place is free once its holder has gone
   (`places.md`, 5).

**Unstarted** (`passes.md`, 3): a start's pass is `ended` at once, its
cases `queued` again, uncounted, and its build released; a resume's
pass is `waiting` again, its cases as they were before the resume and
its build's hold with them. Nothing else of this section applies.

### 4.13 A waiting pass's resume

The loop's third step (`passes.md`, 7), for a free place, before any
start, waiting passes oldest first. A `waiting` pass whose last request
is filed may resume when:

- **its goal has landed:** a landing whose `goal` is the request's
  goal, held by a kept build that triage has reached: it resumes on the
  one with the greatest `order` of those, not necessarily at the
  landing's own commit; or
- **its goal ended without landing:** the request's `ended` is set and
  no landing of its goal is recorded: it resumes on the reached kept
  build with the greatest `order`.

Resuming, in one write: the pass `running` at the place; the new build
held and the old one released, unless they are one; its `failed` cases
`assigned` again; its cases joined while it waited (4.15) still
`assigned`; every `queued` case of a surface in `surfaces` whose
landings the new build holds `assigned` to it; and a new session. A case
queued after this write is not the pass's ("none later",
`../../draft/landed-qa.md`, 9).

### 4.14 Findings' rounds

At the end of a pass session that is not its first:

- **Checked:** each open finding of the pass by the last check the
  session recorded: `holds` false, `closed` at the pass's build's
  commit, now; `holds` true, still `open`. An unchecked one stays
  `open`.
- **Two rounds at most:** a finding still `open` whose `goals` hold two
  goals is `handed`, out of the pass's requests, and a line for its
  `area`: `finding N (TITLE; setup S) still holds after goals G1 and
  G2; handed on` (`../../draft/landed-qa.md`, 9).

A `linked` finding follows the one it is linked to: it is never filed,
checked or handed on itself.

### 4.15 A waiting pass keeps its page

At `triage case`, where a `waiting` pass has an `open` finding on the
ask's surface, page and setup, the ask is a new case, `assigned` to that
pass, joining its `cases`, to run at its resume; the oldest such pass
where several have one. The command prints `case N joined pass P,
waiting on finding F`.

### 4.16 Requests

- **Made** at a pass session's end (4.12), unfiled.
- **Filed** by the loop (`findings.md`, 5): `filed` set when the
  deployment's command succeeds, with `goal` for a request with
  findings; each finding it carries gains that goal in its `goals`.
- **Its goal's end** without landing recorded as `ended`
  (`findings.md`, 6). A landing of its goal outranks it: 4.13 reads the
  landing first.

## 5. Its refusals

Printed `temper-qa: WORDS`, exit 1 (`README.md`, 2), nothing recorded:

| What | WORDS |
|---|---|
| no session, for triage | `TEMPER_QA_SESSION is not set: triage commands run in a triage session` |
| no session, for a pass | `TEMPER_QA_SESSION is not set: pass commands run in a pass session` |
| a session not running | `session N is not running` |
| a session of the other kind | `session N is a pass session` or `session N is a triage session` |
| a pass not running | `pass P is STATE` |
| out of order | `session N triages landing L, which is not next: landing K is not triaged` |
| a case after no QA | `landing L is answered no QA; a case is refused` |
| no QA after a case | `landing L has cases; no QA is refused` |
| no QA without why | `no QA needs a line saying why` |
| a case file unread | `case refused: FILE: WHY` |
| no surface | `case refused: no surface` |
| another surface | `case refused: surface S is neither web nor tui` |
| no page | `case refused: no page` |
| a page not the surface's | `case refused: P is no SURFACE page; pages: P1, P2, ...` |
| no setup | `case refused: no setup` |
| a setup not in the catalogue | `case refused: setup S is not in the catalogue; setups: S1, S2, ...` |
| no step | `case refused: no step` |
| a step without its act | `case refused: step N says nothing to do` |
| a step without expectation | `case refused: step N has no expectation` |
| a field it does not know | `case refused: unknown field F` |
| a case not the pass's | `case N is not assigned to pass P` |
| another result | `outcome refused: RESULT is none of passed, failed, blocked` |
| blocked without why | `outcome refused: a blocked case says why` |
| failed without a finding | `outcome refused: a failed case names a finding of its own` |
| a finding not the case's | `outcome refused: finding F is not case N's` |
| a finding file unread | `finding refused: FILE: WHY` |
| a finding field missing | `finding refused: no F`, F the first of `case`, `title`, `severity`, `setup`, `lines`, `expected`, `seen` missing or empty |
| another severity | `finding refused: severity S is none of low, medium, high` |
| a setup not the pass's | `finding refused: setup S is not in pass P's setups; setups: S1, S2, ...` |
| a bad line | `finding refused: line N is empty or spans lines` |
| evidence unread | `finding refused: evidence PATH: WHY` |
| a field it does not know | `finding refused: unknown field F` |
| a check in a first session | `check refused: pass P's first session has no finding to check` |
| a finding not open in the pass | `check refused: finding F is not open in pass P` |
| another check result | `check refused: RESULT is neither holds nor gone` |
| a note without words | `note refused: no words` |

## 6. For a person

- **`temper-qa show`:** the ledger as text, a block per kind (landings,
  cases, builds, sessions, passes, findings, notes), one line per
  record, oldest first, times as UTC to the minute
  (`2026-11-12T10:04Z`): a landing's number, short commit, goal, title
  and state; a case's number, surface, page, state, pass and landings;
  a build's short commit, state, held landings and holding pass; a
  session's number, kind, landing or pass, effort, time and exit; a
  pass's number, place, short build, state, cases and goal; a finding's
  number, pass, case, severity, state and title; a note's number, pass
  and words.
- **`temper-qa lines`:** each line, `N TIME AREA: WORDS`, oldest first.
- **`temper-qa report PASS`:** the pass's report (`passes.md`, 8).

## 7. Later

- **Findings from triage:** a security hole or lost data triage sees
  while reading (`triage.md`, 7).
