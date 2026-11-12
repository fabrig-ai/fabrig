# Triage

Provisional, 2026-11-12, from `../../draft/landed-qa.md`, section 4,
milestone Q1 of `../../draft/landed-qa.execution.md`. How each landing
is judged for what a person now meets: an agent session per landing, in
main's order, answering no QA or cases through the ledger's commands.

## 1. In one page

- **One landing at a time, in main's order** (section 2): the loop
  starts a session for the next landing when none runs (`ledger.md`,
  4.1).
- **Given its inputs** (section 3): the landing's drafts, diff and stat,
  its commits' `QA:` trailers, the surfaces' pages, the setups'
  catalogue and the cases queued, in a brief on its stdin and a tree of
  the merge to read.
- **It answers** (section 4) through `temper-qa triage none WHY` or
  `temper-qa triage case FILE`, each recorded at once; a case for a page
  a queued case covers joins it.
- **Cheap** (section 5): the agent's command line at triage's effort,
  within `bounds.triage`; it reads, and starts no box.
- **Its end** (section 6) is its process's: the landing is triaged if it
  answered; a landing whose triage recorded nothing twice is a line.

## 2. A session's start

The loop's second step (`README.md`, 4), when no triage session runs and
there is a next landing:

1. **Its inputs read from git** at the landing's merge, MERGE, and its
   first parent, MERGE^1 (section 3), through the git face
   (`landings.md`, 6).
2. **Recorded:** a session in the ledger, `triage`, its landing, its
   effort (`agent.triage`), its start, and the pages and setups its
   commands check against (`ledger.md`, 2.5).
3. **Its directory,** `sessions/N/` in the home: `brief` (section 3),
   `diff.patch`, `git diff MERGE^1 MERGE`, and `tree/`, `git -C
   DIRECTORY worktree add --detach sessions/N/tree MERGE`, for the agent
   to read the whole tree at the merge.
4. **The agent started** (section 5).

A start that fails before the agent runs (git refusing, the command not
found) ends the session `unstarted`: it counts for no landing's
`sessions`, its tree is removed, and the failure is the loop's step's
(`README.md`, 4), tried again at the next tick.

## 3. Its brief

`brief`, given on the agent's stdin, written by code from these, in
this order:

- **What triage is,** from `../../draft/landed-qa.md`, 4: what a person
  now sees or does on the web client or the TUI, the green path only, as
  `../../draft/v0.1.0.md`, 4 weighs it; each case's expectations from the
  draft's promises, written before anything is tried; no QA, with a line
  why, when the landing changes nothing a person meets there; reading
  alone, no box started, within its bound.
- **The landing:** its number, goal, pull request, merge commit and
  title.
- **Its drafts,** each a path in the tree: every draft the merge's
  message names by its path, a Markdown file directly in a tree's
  `docs/design/draft/`, an execution or status file named read as its
  draft, each once, those present at MERGE; with none named, the drafts
  whose file, status file or execution file the diff touches, but the
  drafts' notes and `archive/`; with none either, a line saying no
  draft is named and the request was the title's words.
- **Its diff:** `git diff --stat MERGE^1 MERGE` in the brief, and the
  whole diff's path, `sessions/N/diff.patch`.
- **Its commits' `QA:` trailers:** each value of a `QA:` trailer among
  `git log MERGE^1..MERGE`, each once, in order; none, said.
- **The surfaces:**
  - `web`: each page of `temper/docs/design/client/pages/` at MERGE, its
    name the stem of each Markdown file there but the README and the status files,
    and each subdirectory's name (`chat`), with its design's path; and
    the journeys, `temper/docs/design/client/web/journeys.md`;
  - `tui`: the same of `temper/docs/design/client/tui/pages/`, and
    `temper/docs/design/client/tui/journeys.md`.

  These names are the session's `pages`.
- **The setups' catalogue,** a line each: `fresh`, a fresh box at its
  welcome with nothing set up; then each directory under
  `temper/docs/development/qa/setups/` at MERGE, its name and its
  `README.md`'s first paragraph (`../../draft/landed-qa.md`, 5). These
  names are the session's `setups`; with no recipe yet, `fresh` alone,
  and a case says in `adds` what it needs.
- **The cases queued:** each case's number, surface, page and landings,
  and each ask's setup, adds and steps with their expectations; a case
  for one of their pages joins it.
- **The commands** (section 4), the case file's form, and that each is
  recorded at once, so a session ended early loses nothing recorded.

## 4. Its answers

- **`temper-qa triage none WHY`:** WHY one argument, the line saying why
  (`ledger.md`, 4.2); prints `landing L: no QA`.
- **`temper-qa triage case FILE`:** FILE a path, or `-` for stdin, holding
  one JSON object:

  ```json
  {
    "surface": "web",
    "page": "work",
    "setup": "fresh",
    "adds": "a project whose repository holds one draft marked ready",
    "steps": [
      { "do": "open Work", "expect": "the draft is listed under Drafts by its title", "see": "Drafts" },
      { "do": "press the draft's Start", "expect": "a goal starts on it at the project's way" }
    ]
  }
  ```

  `adds` and each `see` optional, the rest required; checked, merged and
  answered as `ledger.md`, 4.3 says, or refused as its section 5 words it.
- **Both** run in the session's environment (section 5) and record for
  its landing alone.

## 5. The agent's run

- **Its command:** `agent.command` (`README.md`, 3.2), each `{effort}`
  replaced by `agent.triage`; its working directory the session's tree;
  its stdin `brief`; its stdout and stderr both to
  `sessions/N/output`; its environment the tool's captured one with
  `TEMPER_INTERNAL_QA_HOME` and `TEMPER_INTERNAL_QA_SESSION=N` set; a
  process group of its
  own.
- **Its bound:** `bounds.triage` seconds from its start; then its group
  is sent SIGTERM, and SIGKILL ten seconds later, and its exit is
  `bound`.
- **Its tokens,** where its command reports them: the last nonempty line
  of `output`, when it is `tokens N`, N decimal. The deployment's
  command line bounds them (`README.md`, 3.2).
- **Its world:** an agents face of the crate's, to start a session's
  command, ask whether it has ended and stop it; the tests' fake records
  through the ledger's operations as the commands would, or records
  nothing (`README.md`, 6).

## 6. Its end

- **At the loop's second step,** a session whose process has exited, or
  whose bound has passed, is ended in the ledger with its exit, tokens
  and time (`ledger.md`, 4.4), and its tree removed (`git worktree
  remove --force`); `brief`, `diff.patch` and `output` stay.
- **Lost:** a session still running when the loop starts is ended `lost`
  (`README.md`, 4) and counts as any end does.
- **Then** the next session may start in the same step.

## 7. Later

- **A security hole or lost data** seen in triage is a finding
  wherever it is (`../../draft/landed-qa.md`, 4), once findings are
  recorded, in Q3.
- **Sessions' directories** are kept; their retention waits for a need.
