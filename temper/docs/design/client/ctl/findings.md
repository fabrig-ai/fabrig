# Findings in `temper ctl`

Provisional, 2026-11-19, from `../../draft/after-landing-repairs.md`,
section 6, and `../../domain/findings.md`, section 9. A person or their
scripts read a project's findings, one finding whole, and the reds after
landing blamed on a goal (`../../domain/red-findings.md`, section 10).
The tools are records of the one table (`tools.md`, section 1), reads
with no key, their text and JSON as `results.md` says. The finding acts,
`finding file`, `hand`, `route`, `backlog` and `drop`, are `tools.md`,
section 3's.

## 1. In one page

| Tool | What it answers | jig's |
|---|---|---|
| `finding list --project PROJECT [--state new\|routed\|backlog\|dropped] [--mine] [--goal TASK \| --milestone TASK]` | the project's areas and triager, then its findings newest first, a line each; with a filter, those it keeps (2) | the forge's subject `findings`, read once (`../forge/records.md`) |
| `finding show FINDING` | one finding whole, its blame and its clearing among it (3) | the forge's read `Finding` |
| `goal show TASK` | a goal: its task's document, as `task show` prints it, then its reds after landing (4) | `task{n}`, and the forge's subject `findings` filtered by the goal, read once |

The group `goal`'s line: "Goals: start one from a project template, or
read one and its reds after landing".

## 2. `finding list`

- **Its filters:** `--state` and `--mine` are the subject's `triage`
  filter, a state and the reader's alone (`../../domain/areas.md`,
  sections 4 and 5); `--goal TASK` and `--milestone TASK` its `goal`
  and `milestone` filters, the findings blamed on that goal or on a goal
  of that milestone, in every state, cleared or not
  (`../../domain/red-findings.md`, section 10). `--goal` and
  `--milestone` take neither each other nor `--state` or `--mine`: a
  usage error, exit 1, naming the flags.
- **Its text:** the project's areas, a line each, `NAME owner OWNER,
  held by HOLDER: WORDS`, and its triager's line, then its findings
  newest first, a line each: `P/N`, state, kind, severity, area or `-`,
  holder, title, ` · ` between them; a cleared finding's line then
  ends ` · cleared by TIER run N`. `[cut]` follows where the document
  was cut.
- **Its JSON,** the record `findings`: as before, each row with its
  `goal` and `reason`, null where it has none, and `cleared`, `{tier,
  run, at}` or null.

## 3. `finding show`

- **Its text:** its line, its filer, when, the last hand-off's why, its
  goal and a drop's reason, as before; then, where it has a blame,
  `blame: STRENGTH`, `owned`, `suspected` or `none`, and a line for each
  landing it names, oldest first, indented two spaces: `landing COMMIT:
  `, the commit's first twelve hex digits, then each goal, `goal G`,
  then ` in milestone M` and `, person P` where it has them, `; `
  between goals, or `no goal`; then, where cleared, `cleared: TIER run N
  at TIME`, UTC to the minute; then its body.

  ```
  blame: owned
    landing 1a2b3c4d5e6f: goal 820 in milestone 812, person 4
  cleared: nightly run 412 at 2026-11-19T09:40Z
  ```

- **Its JSON,** the record `finding`: as before, with `blame`,
  `{strength, landings: [{commit, goals: [{goal, milestone, person}]}]}`
  or null, `commit` its whole hex, and its row's `cleared`.

## 4. `goal show`

- **What it prints:** the task's document as `task show TASK` prints
  it (`tools.md`, section 2), then `reds after landing:` and the
  findings blamed on it, the uncleared first, each group newest first,
  a line each, indented two spaces: `P/N TITLE · WHERE`, WHERE where it
  stands in `../../domain/red-findings.md`, section 10's words, from the
  client domain's `findings::stands`; with none, `reds after landing:
  none` on one line; `[cut]` last where the document was cut.

  ```
  reds after landing:
    2/14 3 tests fail in nightly since S7 landed · being repaired by goal 830
    2/9 1 test fails in nightly since S7 landed · cleared by nightly run 412
  ```

- **What it reads:** the task's document, then, in its project, the
  forge's subject `findings` with the filter `goal`, TASK, read once as
  `finding list` reads it: watched, its first delivery printed, then
  unwatched. A task that is not a goal has no reds and prints `none`.
  It is not a watched read: `--since` and `--until` are usage errors
  (`tools.md`, section 4).
- **Its JSON,** the record `goal_show`: `task`, as `task show --json`
  prints it; `reds`, the rows in the text's order, each as `finding
  list --json` prints a row, with `stands`, `{kind, goal, reason, tier,
  run}`, `kind` one of `cleared`, `waiting`, `repairing`, `backlog` or
  `dropped` and the rest present where its kind has them; and `cut`.

## 5. Its tests

In the tools' step harness (`temper/crates/temper-client-tools`), from
fixtures, each failing first: each tool parses and prints back, its
help's golden files and its group's reference page
(`temper/docs/help/reference/`); `finding list --goal` and
`--milestone` sending the subject with that filter, refused with each
other, `--state` or `--mine`; a cleared row's ending; `finding show`
printing an owned blame naming its goal, milestone and person, a
suspected one of two landings, one with no goal, and none; `goal show`
printing the document, then its reds uncleared first, each of the five
places a red stands, `none` and `[cut]`; and their JSON. The engine's
side is the root's world's (`../../domain/red-findings.md`, section 11).

## 6. Open questions

- **A goal's reds in one state,** `--goal` with `--state`. Default:
  refused; a goal's reds are few, and `goal show` says where each
  stands.
