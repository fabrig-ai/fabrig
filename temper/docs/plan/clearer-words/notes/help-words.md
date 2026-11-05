# clearer-words: the help's words

The exact words of the task `help-words`, checked against the trunk when this cut was written: applied in order, `change.md` is 8,186 bytes and `task.md` 8,168, within the concept bound of 8,192. Every word removed is a fact the page says elsewhere, or a line end.

## The merge command's section

The section `configuration` closes with, rendered by `temper-service/src/startup/reference.rs`. `{BYTES}`, `{SECONDS}` and `{PATH}` come from `limits.rs`, in the task's Steps; everything else is literal, in this order, with a blank line after the heading and after the first paragraph, and the page ending in one line end. The word `restart` stays out of it: `the_pages_configuration_are_what_the_tables_render` allows one per page.

````markdown
## The merge command

`merge_job` names this command, a path in the repository being merged. It settles the conflicts its project's rules know, with no model, before an agent is asked. temper takes it from the merge in progress: a command file left in conflict, or absent from the merge, is its failure.

- **What it is given:** its arguments, `merge` and the path of `request.json`, which holds `{"version":1,"head":"HEAD","base":"BASE"}`: the head merged into, and the commit merged in. Its environment is `PATH` (`{PATH}`), `LANG` (`C.UTF-8`), and `HOME` and `TEMPER_JOB`, both the job's directory, which is its working directory too. No credential reaches it. It has {SECONDS} seconds.
- **Where it works:** the repository is the directory of its own name under `TEMPER_JOB`, at the head with the base merged in and not committed; the conflicting files are marked and unmerged in the index (`git diff --name-only --diff-filter=U`).
- **What it must do:** edit the conflicting files it knows and `git add` each it settles, without committing, aborting the merge or writing another branch; then write `result.json` in `TEMPER_JOB`, at most {BYTES} bytes. Where the paths do not fit, it lists those that do, in index order, and adds `"more":N`, the count left out. The same head and base must give the same tree.
- **Exit 0:** `{"settled":[PATH]}`, no file left unmerged: temper commits the tree as the merge, with both parents, and pushes it. A file left unmerged or still marked fails the job.
- **Exit 3:** `{"settled":[PATH],"left":[PATH]}`, the conflicts it could not settle: nothing is committed or pushed, and an agent resolves them in a merge made afresh, this result in its words.
- **Any other exit,** or a result missing or past its bound, is the command's failure. A job that fails for good holds the change, as a resolution that fails does.
````

## `merge_job`'s line

````
The merge command’s path from the repository root; see The merge command, below.
````

## `temper/docs/help/change.md`

Each old text occurs once; replace it with the new. A fence's last line end is the fence's, so text after an old text on its line stays; where the new is empty, the old's whole line goes.

### change 1

Old:

````
  only those words (temper help task).
  With control: `landing` (`goal`, a branch, or the default), `branch` (`mine`: its whole goal branch),
  `after` (task numbers or names from the same call it waits for), `merge` (a catch-up), `repairs` (the red landing
  commit from news: temper help tier), `changes` (one batch), `producer` (the charter number for its coder,
  repairs and resolutions; default: the forge's change charter) and
  `llm` (`producer` and `review` model and effort names; default: its charter's),
  and `questions`
  (`foreman`, `up`, or `none`, the default: how its producer's questions go).
````

New:

````
  only those words (temper help task). With control: `landing` (`goal`, a branch, or the default), `branch` (`mine`:
  its whole goal branch), `after` (task numbers or names from the same call it waits for), `merge` (a catch-up),
  `repairs` (the red landing commit from news: temper help tier), `changes` (one batch), `producer` (the charter
  number for its coder, repairs and resolutions; default: the forge's change charter), `llm` (`producer` and `review`
  model and effort names; default: its charter's) and `questions` (`foreman`, `up`, or `none`, the default: how its
  producer's questions go).
````

### change 2

Old:

````
cut at a UTF-8 boundary with “…” where needed.
````

New:

````
cut at a UTF-8 boundary with “…”.
````

### change 3

Old:

````
- **Its panel:** task, repository, branch, base, pull URL, step, CI,
  gates, repair/update/resolution limits; `way: turn · landing task N`;
  questions' route, count, words and standing. `read_task` gives its answer
  number. `CI not required` means its repository or rules ask for none.
  JSON separates
````

New:

````
- **Its panel:** task, repository, branch, base, pull URL, step, CI, gates, repair/update/resolution limits;
  `way: turn · landing task N`; questions' route, count, words and standing. `read_task` gives its answer number.
  `CI not required` means its repository or rules ask for none. JSON separates
````

### change 4

Old:

````
  `Cleared by TIER run N.`, or, after a red landing, `Cleared by TIER
  run N, which passed the failures blamed on it; repaired by COMMIT.`
  where a change naming it repaired it, or `Not cleared: no verdict
  within the verification bound.` Verdicts
````

New:

````
  `Cleared by TIER run N.`, or, after a red landing, `Cleared by TIER run N, which passed the failures blamed on it; repaired by COMMIT.`
  where a change naming it repaired it, or `Not cleared: no verdict within the verification bound.` Verdicts
````

### change 5

Old:

````
  `task show TASK`. Or it fails or is cancelled. No change applies: ends
  successfully with why, no pull request or landing. Failure holds it at
  `producing` (below).
````

New:

````
  `task show TASK`. Or it fails or is cancelled. No change applies: ends
  successfully with why, no pull request or landing.
````

### change 6

Old:

````
- `producing`: the coder writes the change; temper pushes its branch
  once it finishes. Failure holds it here with the coder's words;
  an explicit conclusion that no change applies ends it successfully;
````

New:

````
- `producing`: the coder writes the change; temper pushes its branch
  once it finishes. Failure holds it here. With dependencies pending,
  it may start early from their pushed heads. A head that does not merge
  ends that producer `Cancelled · its early start conflicts with SHA`;
  nothing holds: once they are done, the change goes on with a new producer;
````

### change 7

Old:

````
- `gating`: its blocking gates give their verdicts (temper help
  landing-rule);
````

New:

````
- `gating`: its blocking gates give their verdicts (temper help landing-rule);
````

### change 8

Old:

````
- `resolving`: the merge job, then an agent, settles a conflict with
  the base and pushes the merge commit;
````

New:

````
- `resolving`: the merge job (its command: temper help configuration),
  then an agent, settles a conflict with the base and pushes the merge
  commit;
````

### change 9

Old:

````
  (`Repair CI at HEAD: CONTEXT`,
  `Repair gate N (KIND) at HEAD: changes requested`,
  `Repair the base's break at HEAD: BASE`),
  so `task show` names it
````

New:

````
  (`Repair CI at HEAD: CONTEXT`, `Repair gate N (KIND) at HEAD: changes requested`, `Repair the base's break at HEAD: BASE`),
  so `task show` names it
````

### change 10

Old:

````
- `ready`: its gates hold at its head, and its landing, `way: turn ·
  landing task N`, proves its candidate and waits for the turn;
````

New:

````
- `ready`: its gates hold at its head, and its landing proves its
  candidate and waits for the turn;
````

### change 11

Old:

````
- **Bounded work:** it holds past repair, resolution or update limits.
````

New:

````
````

### change 12

Old:

````
- **Approvals:** rule, role, freshness, required/remaining counts, people
  and heads; unread and incomplete differ from zero. Unmapped accounts
  do not count. An approval
````

New:

````
- **Approvals:** rule, role, freshness, required/remaining counts, people and heads; unread and incomplete differ from zero. Unmapped accounts do not count. An approval
````

### change 13

Old:

````
- **Gates are temper's:** a change's gates run in temper, and `change
  show` prints each, `gate 1 · review · blocking · approved`; temper
  posts none
````

New:

````
- **Gates are temper's:** a change's gates run in temper, and `change show` prints each, `gate 1 · review · blocking · approved`; temper posts none
````

### change 14

Old:

````
- **Landed as checked:** temper merges the exact head after its required
  CI and gates pass
````

New:

````
- **Landed as checked:** temper merges the exact head after its required CI and gates pass
````

### change 15

Old:

````
- `queued`: it waits its turn in its landing branch's queue, `queued:
  place N`;
````

New:

````
- `queued`: it waits its turn in its landing branch's queue, `queued: place N`;
````

## `temper/docs/help/task.md`

Each old text occurs once; replace it with the new. A fence's last line end is the fence's, so text after an old text on its line stays; where the new is empty, the old's whole line goes.

### task 1

Old:

````
  for each sub-agent choice in order. A model without an effort prints
  just its name. A task without a choice prints no model lines.
````

New:

````
  for each sub-agent choice in order. A model without an effort prints
  just its name.
````

### task 2

Old:

````
  awaited`, or a party and what it must answer: `you: question 3 of
  task 44 · 1 waiting`.
````

New:

````
  awaited`, or a party and what it must answer: `you: question 3 of task 44 · 1 waiting`.
````

### task 3

Old:

````
  leaving it held, or passing it up to the next holder of authority,
  are the other decisions. Only an uncertain effect asks whether to
  repeat it. `task show` prints `held:` and its reason, then `way
  out:`, what to mend and the call that releases it: `task release
  TASK --hold REV`. An effect's
````

New:

````
  leaving it held, or passing it up to the next holder of authority,
  are the other decisions. `task show` prints `held:` and its reason, then `way out:`, what to mend and the call that releases it: `task release TASK --hold REV`. An effect's
````

### task 4

Old:

````
  in its connector's words. An uncertain effect is repeated only by a
  release with `--again`, once you have checked whether it happened.
````

New:

````
  in its connector's words. Only an uncertain effect asks whether to
  repeat it: a release with `--again`, once you have checked whether it happened.
````

### task 5

Old:

````
  cancel again, under its key, gets its first answer.
````

New:

````
  cancel again, under its key, gets its first answer. A change's early
  start may end its producer `Cancelled · its early start conflicts with
  SHA`: a dependency's head did not merge. The change goes on with a new
  producer (temper help change).
````

### task 6

Old:

````
  kind, or failed or cancelled; a delegate's result reaches its
  parent.
````

New:

````
  kind, or failed or cancelled.
````

### task 7

Old:

````
- **Words between turns:** a message reaches the agent at a turn's
  boundary and steers its next turn, never the one in progress.
````

New:

````
- **Words between turns:** a message reaches the agent at a turn's
  boundary and steers its next turn.
````

### task 8

Old:

````
  one; `task tree`, `task
  activity`, `task turn` and
````

New:

````
  one; `task tree`, `task activity`, `task turn` and
````

### task 9

Old:

````
  `task message`, `task answer`, `task amend`, `task stop`, `task
  cancel`, `task release`, `hold lift`, `task decide` and `proposal
  decide`; an act marked `(no tool)` has no ctl tool yet.
````

New:

````
  `task message`, `task answer`, `task amend`, `task stop`, `task cancel`, `task release`, `hold lift`, `task decide` and `proposal decide`.
````

### task 10

Old:

````
  another party, reorder it or mark it seen: those acts print `(no
  tool)`.
````

New:

````
  another party, reorder it or mark it seen: those acts print `(no tool)`.
````
