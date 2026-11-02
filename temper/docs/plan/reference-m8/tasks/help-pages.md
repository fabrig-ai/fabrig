+++
id = "help-pages"
part = "help"
kind = "docs"
needs = ["panel-steps", "tiers-root"]
files = ["temper/docs/help/tier.md", "temper/docs/help/change.md", "temper/docs/help/ctl.md", "temper/docs/help/index.md", "temper/crates/temper-docs/src/lib.rs", "temper/crates/temper-client-tools/src/tests/golden/help_groups.txt"]
effort = "medium"
minutes = 45
+++

# Help: the tier page, and a change living to cleared

## Goal

temper's help says what tiers are and how a change lives to cleared, in the text below, which
the lead wrote (`temper/AGENTS.md`: help pages are the lead's): place it exactly, register the
new page, and run the help tests. This task departs from the implementer's text in one way,
stated: it writes `temper/docs/help/`, only as given here.

Acceptance:

- `temper/docs/help/tier.md` is the page below, byte for byte; `temper-docs`' `PAGES` gains its
  entry after `template`'s (`name: "tier"`, `kind: Kind::Concept`, `title: "Tier"`, `summary`:
  the page's head line, `text: include_str!("../../../docs/help/tier.md")`), the array's length
  54.
- `change.md`, `ctl.md` and `index.md` carry the edits below, each replacing exactly the text
  quoted.
- Pass: `heavy cargo nextest run -p temper-docs`; `-p temper --lib -E 'test(/help|commands/)'`;
  `-p temper-client-tools --lib -E 'test(/help/)'`.

## Inputs

- `temper/docs/plan/reference-m8/notes/common.md`.
- The help's rules: `temper/crates/temper-docs/src/tests/rules.rs` (`the_table_is_the_directory`,
  `each_page_keeps_its_head_template_links_and_voice`,
  `the_index_names_every_concept_walk_and_recipe_once`); `temper/crates/temper/src/tests/help.rs`
  (`every_command_line_in_the_pages_parses`,
  `box_and_change_help_locate_temper_review_gates_in_change_show`).

## The text

`temper/docs/help/tier.md`, whole:

````markdown
# Tier

Kind: concept

A tier is a slow check of a branch after changes land on it: each run covers every landing since the last, and its verdict clears them or blames the one whose commit broke a test.

## What it is

- **A tier** is a workflow too slow for a change's CI, such as a
  browser suite or an exhaustive profile. A repository names its tiers
  beside its gates, and each tier names the branch it follows: the
  repository's landing branch, its main, unless it names another. A
  landing on a branch no tier follows, a goal's own branch for one, is
  cleared as it lands.
- **A run** covers every landing since the tier's last judged run. One
  run per tier goes at a time, and the next starts at the newest tip
  once the last ends. temper dispatches it on a branch of its own,
  `temper/tier/NAME.N`, made at the commit it runs, and deletes the
  branch when the run ends.
- **Its report:** the run uploads an artifact, named in the tier's
  settings, holding a JUnit file or one test id per line, and temper
  reads the failing tests' ids from it. A failed run that names no
  test, or whose artifact is absent or unreadable, is a runner's fault,
  never a red.
- **A tip already judged:** where the files the tier reads at a new tip
  are those of a tip it already ran, that run's receipt judges it with
  no new run, so a landing of documentation every tier ignores clears
  at once.

## How it changes

- **Green** clears every landing in the run's range.
- **Red** runs the failed tests once more, alone, at the same tip.
  Those that pass are a flake: counted on the tier, blamed on no one.
  Those that failed in the tier's previous run too are inherited,
  blamed on no new landing. A new failure belongs to the one landing in
  the range, or to the first landing whose commit fails it, found by
  running the failing tests alone at each landing's commit, oldest
  first; a landing that changes nothing the tier reads is skipped.
- **A landing's verdict is its own:** red only by the failures blamed
  on it, until a later run passes them, which clears it.
- **A runner's fault,** a lost run or a failed setup, runs the part
  again, up to the project's `tier_retries`. Past them, the tier's run
  task is held, `Tier NAME run N held after K runner faults, last:
  KIND`, until a person releases it, which runs it again.

## What temper guarantees

- **A change lives to cleared:** landed, then verifying, until every
  tier following its branch has cleared its landing, and its result
  comes then (temper help change). A green tip never hides a red
  landing behind it.
- **Bounded:** past the project's verification bound, two hours unless
  set, with no verdict, the change ends landed and uncleared, saying
  so; a later verdict still counts on the tier.
- **Only temper's runs judge:** a run of the workflow that temper did
  not dispatch is not read.

## Acting on it

- **The configuration:** a repository's tiers go in the `landing`
  member, beside its gates, and the project's `tier_retries` and
  `verification_bound_ms` beside its `by`; then restart. Main's tiers,
  the slow suites, follow main:

  ```json
  {"forge": 1, "repository": 7, "gates": [],
   "tiers": [{"name": "browser", "workflow": "browser.yml",
              "ignored": ["docs/"], "artifact": "report", "format": "junit"}]}
  ```

- **The workflow** is dispatch-only and takes the inputs
  `temper_commit` and `temper_key`, and `temper_tests`, the ids to run
  alone, one per line, when temper gives them: it runs only those then.
  It uploads its report as the named artifact.
- **ctl:** `change show TASK --until cleared` waits for a landed
  change's verdict, its step `verifying`, `red` or `cleared` naming the
  tiers and their runs; `task show` reads a tier's run task, and `task
  release` releases a held one.

## Limits

- A repository has at most two tiers in the small profile. A run names
  at most 16 failing tests, each id at most 256 bytes; past them its
  failures are many, blamed by landing alone.
- A tier follows one branch: following two takes two tiers.
- temper neither tells a goal of a red after landing nor posts a
  verdict to the forge.
````

`temper/docs/help/change.md`, three edits:

1. Replace

   ```
     long. It names no check or test the change didn't see. Or it fails,
   ```

   with

   ```
     long. It names no check or test the change didn't see. Where tiers
     follow its landing branch (temper help tier), the result waits for
     their verdicts and adds a line, `Cleared by TIER run N.`, or `Not
     cleared: no verdict within the verification bound.` Or it fails,
   ```

2. Replace

   ```
   - `landing`, then `landed`: temper merges it, and `change show` names
     the merge commit.
   ```

   with

   ```
   - `landing`, then `landed`: temper merges it, and `change show` names
     the merge commit; a change no tier follows ends there;
   - `verifying`: landed, and a tier following its branch has not judged
     it yet (temper help tier);
   - `red`: a tier's verdict blames its landing, naming the tier, the
     run and the failing tests; it stays red until a later run passes
     them;
   - `cleared`: every tier following its branch cleared it, and it ends
     with its result; or `landed, uncleared`, past the verification
     bound with no verdict.
   ```

3. Replace `- **The parent hears its result,** landed or not.` with
   `- **The parent hears its result,** landed, and cleared where tiers follow its branch, or not.`

`temper/docs/help/ctl.md`, one edit: replace

```
- `change show TASK`: `pull`, `ci:pending`, `ci:passed`, `ci:failed`,
  `held`, `landed`.
```

with

```
- `change show TASK`: `pull`, `ci:pending`, `ci:passed`, `ci:failed`,
  `held`, `landed`, `cleared`.
```

`temper/docs/help/index.md`, one edit: after the two lines

```
- temper help landing: how a project that lands by turn lands a
  change: its candidate, its gates proved by receipts, the turn.
```

insert

```
- temper help tier: the slow checks after landing: runs on a branch's
  tip whose verdicts clear landings or blame one.
```

## Owns and may change

- The files in the header, as given. If a help test refuses a line of this text (a voice rule,
  a link, a command line that does not parse), change only that line, to the nearest form the
  test accepts, and quote both in the report; nothing else of the text.

## Steps

1. Place the text and the `PAGES` entry.
2. The tests in Goal, counts reported.
3. Commit (why: help says what tiers do and when a change ends); push your branch.

## Report

As the implementer's text says, and also: any line a test made you change, before and after.

## Answers

2026-11-02.

- **The change page past its 8,192-byte bound, and `help_groups.txt` outside the files: may the
  three new state bullets be shortened and the golden fixture updated?** Yes, both.
  1. In `change.md`, edit 2's three new bullets (`verifying`, `red`, `cleared`) are replaced by
     these four lines, the rest of edit 2 and edits 1 and 3 unchanged; the page is then 8,184
     bytes on the pushed branch's text (the result paragraph of edit 1 keeps the link to
     `temper help tier`):

     ```
     - `verifying`: awaiting a tier's verdict;
     - `red`: blamed by a tier, naming the run and failing tests, until a
       later run passes them;
     - `cleared`: all its tiers cleared it, and its result comes; with no
       verdict by the verification bound, it ends `landed, uncleared`.
     ```

     If a help test still refuses it, the line rule in Owns applies; no bound is raised.
  2. `temper/crates/temper-client-tools/src/tests/golden/help_groups.txt` joins this task's files:
     the task's own acceptance (`-p temper-client-tools --lib -E 'test(/help/)'`) cannot pass
     without it. Regenerate it by its route, `SKEIN_CODEGEN_WRITE=1` with
     `tests::words::help_lists_groups_then_a_group_then_a_tool`, and check that its diff is the
     `cleared` of `ctl.md`'s edit alone.
  Continue from the pushed branch `reference-m8/help-pages`.

2026-11-02, second.

- **`ctl.md` is 8,195 bytes, three over its bound: may its last line be shortened, with the
  matching extra line in `help_groups.txt`?** Yes. In `ctl.md`, replace `- ctl opens no browser
  and runs no program.` with `- ctl opens no browser, runs no program.` (the line the bound test
  names, under Owns' line rule; the page is then 8,192 bytes). Regenerate `help_groups.txt` by its
  route; its diff from main is then the `cleared` of `ctl.md`'s edit and this line, nothing else.
  Then run the refused rule and Goal's acceptance whole. No bound is raised. Continue from the
  pushed branch `reference-m8/help-pages`.
