# Triage

Provisional, 2026-11-12, revised 2026-11-18, from
`../../draft/landed-qa.md`, section 4, milestones Q1 and Q6 of
`../../draft/landed-qa.execution.md`. How each landing
is judged for what a person now meets: an agent session per landing, in
main's order, answering no QA, cases or gaps through the ledger's commands.

## 1. In one page

- **One landing at a time, in main's order** (section 2): the loop
  starts a session for the next landing when none runs (`ledger.md`,
  4.1).
- **Given its inputs** (section 3): the landing's drafts, diff and stat,
  its commits' `QA:` trailers, the surfaces' pages, the setups'
  catalogue and the cases queued, in a brief on its stdin and a tree of
  the merge to read.
- **It answers** (section 4) through `temper-qa triage none WHY` or
  `temper-qa triage case FILE` or `temper-qa triage gap KIND WHY`
  (`gaps.md`, 2), each recorded at once; a case for a page
  a queued case covers joins it.
- **Cheap** (section 5): the agent's command line at triage's effort,
  within `bounds.triage`; it reads, and starts no box.
- **Its end** (section 6) is its process's: the landing is triaged if it
  answered; a landing whose triage recorded nothing twice is a line.

## 2. A session's start

The loop's second step (`README.md`, 4), when no triage session runs and
there is a next landing:

1. **Its inputs read from git** at the landing's merge, MERGE, and its
   base, BASE (`landings.md`, 4; section 3), through the git face
   (`landings.md`, 6).
2. **Recorded:** a session in the ledger, `triage`, its landing, its
   effort (`agent.triage`), its start, and the pages and setups its
   commands check against (`ledger.md`, 2.5).
3. **Its directory,** `sessions/N/` in the home: `brief` (section 3)
   and `diff.patch`, `git diff BASE MERGE`. Its tree is the kept
   `trees/triage`, under the home's private `trees/` directory, for the
   agent to read the whole tree at the merge. If absent, made with
   `git worktree add --detach trees/triage MERGE`; if present, moved
   with `git checkout --force --detach MERGE` then `git clean -ffdxq`
   in that tree. Thus each session starts with exactly the merge's
   files, whatever its predecessor left. If either move command fails,
   the tree is removed with `git worktree remove --force`, the worktree
   registrations pruned and the tree made again. A failed add is cleaned
   up too. The brief names this kept tree.
4. **The agent started** (section 5).

A start that fails before the agent runs (git refusing, the command not
found) ends the session `unstarted`: it counts for no landing's
`sessions`, its kept tree stays for the next start, and the failure is
the loop's step's
(`README.md`, 4), tried again at the next tick.

## 3. Its brief

`brief`, given on the agent's stdin, written by code from these, in
this order:

- **What triage is,** from `../../draft/landed-qa.md`, 4: what a person
  now sees or does through the configured surfaces alone
  (`README.md`, 3.2), named in the opening words: with `["ctl"]`,
  through ctl; with all three, through ctl, the web client or the TUI.
  Ctl comes first where configured:
  a landing's ctl cases written before its web and TUI ones, the
  person's acts and temper's account as `temper ctl` gives them, each
  case on a setup of either kind, a box's or a deployment's, which take
  every surface alike. QA judges the happy path alone: v0.1.0's stories
  and each part's bar, used as intended (`../../draft/v0.1.0.md`, 4).
  Edge cases are off it: malformed, out-of-range or oversized input,
  limits and bounds, refusals of bad input and their exit codes, errors,
  races and recovery. Write no case, step or gap for one, and answer no
  QA for a landing whose only change a person meets is one, saying so.
  A security hole or lost data still counts wherever it is. Each case's
  expectations from the draft's promises, written before anything is
  tried; no QA, with a line why, when the landing changes nothing a
  person meets there; the fakes
  alone, the person's forge the box's fake Forgejo; what a person
  meets that these cannot show is a gap (`../../draft/landed-qa.md`,
  6 and 12); reading alone, no box started, within its bound.
- **The landing:** its number, goal, pull request, merge commit and
  title.
- **Its drafts,** each a path in the tree: every draft the merge's
  message names by its path, a Markdown file directly in a tree's
  `docs/design/draft/`, an execution or status file named read as its
  draft, each once, those present at MERGE; with none named, the drafts
  whose file, status file or execution file the diff touches, but the
  drafts' notes and `archive/`; with none either, a line saying no
  draft is named and the request was the title's words.
- **Its diff:** its stat in the brief (below), and the whole diff's
  path, `sessions/N/diff.patch`.
- **Its stat,** written by the tool from the paths `git diff --numstat
  BASE MERGE` gives with their added and deleted lines, BASE the landing's
  base (`landings.md`, 4), so that a
  landing of goldens stays cheap to read (`../../draft/landed-qa.md`, 4):
  - **a generated directory** is a path's leading part up to and
    including its first segment named `generated` or `golden`
    (`crates/X/src/generated/`, `crates/X/golden/`), or
    `temper/docs/help/reference/`, whose pages are rendered from the
    tools' tables (`../../protocol/documentation.md`, 8);
  - **one line per changed path outside them,** in path order,
    `PATH | +A -D`, or `PATH | binary` where git counts no lines;
  - **one line per generated directory** for every changed path under
    it, at the place of its first path in path order, `DIR | N files, +A
    -D`, DIR ending in `/`, its binary files counting no lines;
  - **then the total,** `N files changed, +A -D`, every path counted
    once.

  So a landing that rewrites a thousand goldens takes a line for their
  directory, never a thousand.
- **Its commits' `QA:` trailers:** each value of a `QA:` trailer among
  `git log BASE..MERGE`, each once, in order; none, said.
- **The configured surfaces alone,** ctl's first, with each configured
  surface's entries and design below; an absent surface has neither:
  - `ctl`: each group of ctl's tools, from the pages of
    `temper/docs/help/reference/` at MERGE whose names start `ctl-`,
    each group named by the rest of its page's stem
    (`temper/docs/help/reference/ctl-task.md` is `task`), with that
    page's path; and ctl's design,
    `temper/docs/design/client/ctl/README.md` and `tools.md`;
  - `web`: each page of `temper/docs/design/client/pages/` at MERGE, its
    name the stem of each Markdown file there but the README and the status files,
    and each subdirectory's name (`chat`), with its design's path; and
    the journeys, `temper/docs/design/client/web/journeys.md`;
  - `tui`: the same of `temper/docs/design/client/tui/pages/`, and
    `temper/docs/design/client/tui/journeys.md`.

  These names are the session's `pages`.
- **The setups' catalogue,** a line each, with its kind
  (`../../draft/landed-qa.md`, 4): `fresh (box)`, a fresh box at its
  welcome with nothing set up; then each directory under
  `temper/docs/development/qa/setups/` at MERGE, its name, its kind in
  parentheses, `(box)` or `(deployment)`, and its `README.md`'s first
  paragraph (`../../draft/landed-qa.md`, 5). A recipe's kind is its
  chain's root's (`../../client/recipe.md`, 2): the tool follows each
  `recipe`'s `after` line to the root, through at most eight parents,
  and the root holding `config/` is a deployment's; a chain it cannot
  follow is a box's. When neither `web` nor `tui` is configured, the
  catalogue offers `fresh` and only the recipes whose whole chain,
  the recipe and its `after` ancestors, holds no `web` or `tui` line:
  these walk in a ctl session without a browser
  (`../../client/recipe.md`, 5.1). A missing recipe or parent, a cycle,
  or more than eight parents leaves that setup out. With `web` or
  `tui` configured, the catalogue offers every setup. Either way it
  offers the setups of the kinds `qa.json`'s places run alone
  (`README.md`, 3.2; `kinds.md`, 6): `fresh` and the box's setups only
  where a box's place is configured, a deployment's only where a
  deployment's place is.
  These names are the session's `setups`, those of
  a deployment's kind its `deployments` (`ledger.md`, 2.5); with no
  recipe yet, `fresh` alone, and a case says in `adds` what it needs.
- **The cases queued:** each case's number, surface, page and landings,
  and each ask's setup, adds and steps with their expectations; a case
  for one of their pages joins it.
- **The commands** (section 4), the case file's form, and that each is
  recorded at once, so a session ended early loses nothing recorded.
  Where ctl is configured, the case form says do is ctl or forge drive
  lines, or `temper help PAGE`. In its pass, a `temper help PAGE` step is
  read with the place's own program, with no box; its words compared as a `see` is (`passes.md`,
  4). These pages walk v0.1.0's rung A (`../../draft/v0.1.0.md`, 5,
  *Documentation for agents*). The brief gives the drive lines' byte
  rule in one line: quoted bytes take
  printable ASCII alone, other bytes as `\xHH`, with `\"` and `\\` for
  quote and backslash (`../../client/drive.md`, 3.1).
  Where `web` is configured, the case form lets a page whose draft
  promises a request budget, so many requests in so many waves on
  opening it (`../../draft/page-round-trips.md`, 2), take a step whose
  do is `web requests`, after a step that opens the page cold, its see
  that line's last line (`../../client/drive.md`, 3.2), in these words:
  "For a web page whose draft promises a request budget, a step's do may
  be `web requests`, after a step that opens the page cold, such as a
  reload of its path; its see is that line's last line as the budget
  counts it, `N requests in W waves`: `2 requests in 2 waves` for two
  waves of two requests." Where `web` is not configured, the brief has
  no such words.

After the commands the brief says: "A landing a person would meet
that no case can judge yet, because ctl offers no way to do or see it,
the fakes show no such behaviour, or QA's lines and setups can't reach
it, is a gap: `temper-qa triage gap KIND WHY`, never none."

When a landing closes its gap, its earlier gap and session brief stay
beside the new triage (`gaps.md`, 7). Its diff and drafts are the
original landing's; its surfaces, setups and kept tree are the newest
kept build holding the closing landing's (`ledger.md`, 4.18).

## 4. Its answers

- **`temper-qa triage none WHY`:** WHY one argument, the line saying why
  (`ledger.md`, 4.2); prints `landing L: no QA`.
- **`temper-qa triage case FILE`:** FILE a path, or `-` for stdin, holding
  one JSON object:

  `surface` is one of the configured surfaces, `ctl`, `web` or `tui`;
  another refuses, naming the surfaces triage takes. For `ctl`, `page` is a group of
  ctl's tools (`landing`), each step's `do` a ctl act or wait as the
  person types it, and `see` words ctl prints. For `web`, a step's `do`
  may be `web requests`, its `see` that line's last line, for a page
  whose draft promises a request budget (section 3):

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

  `setup` must be among the session's catalogue. When neither `web`
  nor `tui` is configured, a setup outside it refuses with the reason:
  ctl-only triage takes `fresh` and recipe chains with no `web` or
  `tui` lines. The page must be among that surface's session pages.
  `adds` and each `see` optional, the rest required; checked, merged and
  answered as `ledger.md`, 4.3 says, or refused as its section 5 words it.
- **`temper-qa triage gap KIND WHY`:** KIND `ctl`, `fake` or `qa`, WHY
  one nonempty argument: what a person would do or see and what is
  missing to judge it. Prints `gap N`, answering `gap` unless cases
  accompany it (`gaps.md`, 2).
- **All** run in the session's environment (section 5) and record for
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
  and time (`ledger.md`, 4.4); the kept tree, `brief`, `diff.patch` and
  `output` stay.
- **Lost:** a session still running when the loop starts is ended `lost`
  (`README.md`, 4) and counts as any end does.
- **Then** the next session may start in the same step.

## 7. Later

- **A security hole or lost data** seen in triage is a finding
  wherever it is (`../../draft/landed-qa.md`, 4), once findings are
  recorded, in Q3.
- **Sessions' directories** are kept; their retention waits for a need.
