# browser-budget: how a story is measured

## One run

From the slot's root, one story by its exact name, in a fresh private directory:

```sh
DIR=target/browser-budget/<label>-<story>-<n>
cat /proc/loadavg
heavy python3 -I -B tools/tiers/run --cpu "$DIR" temper browser -- -E 'test(=<story>)'
cat /proc/loadavg
tools/tiers/cpu-summary "$DIR" --json
```

- **Check the count:** the summary must hold exactly one test. A run that selected nothing proves nothing.
- **Three runs of a story, for a figure,** unless your spec says otherwise. Report each and their median: the same story varies by about a third with the load. To compare two arms, alternate them (A B A B A B), so that the load falls on both.
- **A failed run stays in the table,** with its log's path and what it showed. Don't leave out a measurement because it's unflattering, and don't rerun unchanged (`AGENTS.md`, 5), except once after exit 137. A run past the tier's wall limit is **starved**, a warning: mark it, and don't count its CPU as the story's.
- **Delete `target/browser-budget/`** when you've written your figures: it's yours, and it isn't committed.

## Where the CPU goes

`cpu-summary` gives a story's whole tree. Where your spec asks for the split, sample it with a throwaway script, untracked and gone before the push: read `/proc/<pid>/stat` (never `environ`) for the story's process tree, and group user plus system time by family: Chromium's renderer, Chromium's other processes, `temper` serving, `temper ctl` and other children (the agents among them), and the test binary. Say how much of the story's total the split accounts for.

A count of the harness's accessibility queries takes a throwaway counter in `BrowserFace::ask`, removed before the push.

## What's written down

Write your figures to `temper/docs/plan/browser-budget/figures/<your task>.md`, a file of its own so that parallel tasks never meet in one, and commit it:

```markdown
# <task>: <what these runs measure>

<date -u>, commit <short sha>, <nproc> CPUs, <lscpu's model name>.

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---:|---:|---|

**Medians:** ...
```

Name the machine by its CPU count and model only, never by its host name. `budget-close` carries these into `temper/docs/development/browser-budget.md`.
