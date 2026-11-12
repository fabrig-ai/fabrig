# The ledger

Provisional, 2026-11-12, from `../../draft/landed-qa.md`, sections 4
and 9, milestone Q1 of `../../draft/landed-qa.execution.md`. The one
record of landed QA's state for a project: its landings, cases, builds,
agent sessions and lines, kept by code, written through its operations
alone, by the loop and by the agents' commands. Q3 adds passes and
findings.

## 1. In one page

- **One document,** `ledger.json` in the home (`README.md`, 3.1), read
  and written whole under a lock (section 3).
- **Its records** (section 2): main's last head, landings in main's
  order, cases with the landings that asked for them, kept builds,
  agent sessions with their time and tokens, and lines.
- **Its rules** (section 4) hold whoever writes: triage in main's order,
  one answer kind per landing, a case merged into the queued one for its
  page, a build deleted only once no pass can need it, and a case
  waiting too long for a build said once.
- **Its refusals** (section 5) are the agents' only feedback: each names
  what it refused, and nothing is recorded.
- **`show` and `lines`** (section 6) print it for a person.

## 2. The document

One JSON object. Times are whole seconds since the Unix epoch; numbers
of landings, cases, sessions and lines count from 1 in the order they
are recorded and are never reused. Before v0.1.0 it changes in place, as
every stored format does: a later milestone adds records and fields, and
a document a newer build cannot read is refused by name.

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
- `state`: `queued` in Q1; Q3 adds the pass's states;
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
- `held`: absent in Q1; the pass holding it, from Q3.

### 2.5 Sessions

`sessions`. Each:

- `number`; `kind`, `triage` in Q1; `landing`, for triage; `effort`;
- `started`; `ended`, absent while it runs; `exit`, how it ended: its
  code, its signal, `bound`, `lost` or `unstarted` (`triage.md`, 2 and
  6); `tokens`, where its command reported them (`triage.md`, 5);
- what its commands check against, given at its start: `pages`, the
  pages of each surface, and `setups`, the catalogue's names
  (`triage.md`, 3).

A session's time is `ended` less `started`.

### 2.6 Lines

`lines`. Each: `number`, `time`, `area`, `words`, and `given`, true once
given to `notify` (`README.md`, 5).

## 3. Its lock and writes

- **Every operation that records** takes an exclusive lock on
  `ledger.lock` (`std::fs::File::lock`), reads `ledger.json`, applies
  itself, writes the whole document to a temporary file in the home,
  renames it over `ledger.json` and releases the lock. A refusal writes
  nothing. A read alone (`show`, `lines`) takes the lock too, so it
  never reads half a write.
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
- **Merged:** where a case in state `queued` has the same surface and
  page, the new ask joins it, and the case keeps every landing that
  asked for it (`../../draft/landed-qa.md`, 4). Otherwise it is a new
  case, `queued`, at the ask's time.
- **Its answer:** the command prints `case N queued` or `case N joined:
  landings L1, L2`, the case's landings in order.

### 4.4 A session's end

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

## 5. Its refusals

Printed `temper-qa: WORDS`, exit 1 (`README.md`, 2), nothing recorded:

| What | WORDS |
|---|---|
| no session | `TEMPER_QA_SESSION is not set: triage commands run in a triage session` |
| a session not running | `session N is not running` |
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

## 6. For a person

- **`temper-qa show`:** the ledger as text, a block per kind (landings,
  cases, builds, sessions), one line per record, oldest first, times as
  UTC to the minute (`2026-11-12T10:04Z`): a landing's number, short
  commit, goal, title and state; a case's number, surface, page, state
  and landings; a build's short commit, state and held landings; a
  session's number, kind, landing, effort, time and exit.
- **`temper-qa lines`:** each line, `N TIME AREA: WORDS`, oldest first.

## 7. Later

- **Passes and findings** (`../../draft/landed-qa.md`, 9): a case's
  states past `queued`, a pass's records, a build's `held`, findings and
  their rounds, from Q3.
