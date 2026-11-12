# temper ctl draft

Kind: reference

Landed drafts: say ready or not ready, list them.

## draft ready

```
draft ready --project PROJECT [--owner OWNER --name NAME] [--chat TASK] PATH --key KEY
  --project PROJECT: a project's name or number; required. The project
  --owner OWNER: text, at most 1048576 bytes; optional. The repository's owner; give --name too
  --name NAME: text, at most 1048576 bytes; optional. The repository's name; give --owner too
  --chat TASK: a number; optional. The chat it was written in
  PATH (path): text, at most 1048576 bytes; required. The landed draft's relative Markdown path
Say a landed draft is ready to start a goal
Prints: `ready: OWNER/NAME PATH at COMMIT on BRANCH`, then its title
--json: the record draft.
Key: --key KEY, required.
Give --owner and --name together, or neither for the project's home.
Reads PATH at the head of the repository's landing branch.
COMMIT is its first 12 digits; --json keeps the whole row.
```

## draft not-ready

```
draft not-ready --project PROJECT [--owner OWNER --name NAME] PATH --key KEY
  --project PROJECT: a project's name or number; required. The project
  --owner OWNER: text, at most 1048576 bytes; optional. The repository's owner; give --name too
  --name NAME: text, at most 1048576 bytes; optional. The repository's name; give --owner too
  PATH (path): text, at most 1048576 bytes; required. The landed draft's relative Markdown path
Say a landed draft is no longer ready
Prints: `not ready: OWNER/NAME PATH`
--json: the record draft.
Key: --key KEY, required.
Give --owner and --name together, or neither for the project's home.
Work stops listing the draft. --json keeps the answered row.
```

## draft list

```
draft list --project PROJECT [--until CONDITION] [--timeout DURATION]
  --project PROJECT: a project's name or number; required. The project
  --until CONDITION: one of changed; optional. Wait until the drafts change
  --timeout DURATION: a duration: seconds, or a number with s, m or h, at most 3600 seconds; optional. How long a wait lasts; 300 seconds when absent, at most 3600
List a project's landed drafts, newest first
Prints: A line per draft: repository, path, readiness or goal, commit, branch, time, chat and title; `No landed drafts.` with none
--json: the record drafts.
Key: none.
Lists ready and not-ready drafts, including those with a goal.
--json prints the rows with each commit whole.
```
