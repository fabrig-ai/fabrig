# Rung B1, the repair: B0, with a gate the first push fails, and the change's repair

With `kit.md` beside this file (the machine, the login's move, the configuration, findings and
the report). Backends: **ChatGPT with an owned Forgejo** (the first pair).

## What this is

Rung B1 of temper's ladder to v0.1.0 (`temper/docs/design/draft/v0.1.0.md`, 2.1 and 3, read it
first): rung B0's story, the green light, with a gate the change's first push fails for real,
and the change's repair (`temper/docs/design/draft/reference-case.md`, 4.3; mapped in
`temper/docs/design/domain/testing.md`, 2.7: B1 walks today's landing, CI on the pull request's
head, until the third edition's checks before ready land). The gate is the person's: before
the request, the person adds to `main` a rule their CI checks, that each pull request adds
`changes/<its number>.md`, and name when its pull request was opened, which no push can meet before its pull request exists. Its fake
twin is temper's repair story (testing.md 2.4); the repair's design is
`temper/docs/design/domain/forge.md` 8.4. This brief is the rung's acceptance: it is rerun
unchanged until it passes.

## The scenario

1. **The box,** as rung 0 walks it: configured, the four checks `true`, started as a user unit,
   signed in, the budget raised before any forge is set up.
2. **The playground's repository on the owned Forgejo,** made through ctl as the help names it,
   adopted as a project's home. Read it through the API with the read header: its files and
   `main`'s head.
3. **The person's gate:** as the person, with the write header, commit to `main` through
   Forgejo's API (`PUT /repos/playground/widget/contents/PATH`, or `POST .../contents` for
   several files) the two files in the appendix, exactly: the workflow
   `.forgejo/workflows/ci.yml` replaced and `CONTRIBUTING.md` added, one commit, message "CI:
   each pull request adds changes/<its number>.md". Wait until the push's CI run on `main` has
   passed (the combined status at `main`'s head), and say what ctl showed of it, if anything.
4. **The login in:** ctl's account import of `W/account.json`, `W/give-back` written at once,
   the account usable.
5. **The request,** as the person asks it, through ctl's chat in the widget's project, these
   words exactly:

   > Add CSV export to widget. Before anything is changed, have the work tell me its plan and
   > wait for my go-ahead.

   Then, through ctl alone, follow it: the chat's reply; its tree; the person's inbox, waiting
   on it rather than reading it again and again. Expected: the chat delegates the work, the work
   asks its question (its plan, and whether to go ahead), and the question waits in the
   person's inbox, its words readable through ctl.
6. **Before answering,** check both sides: no change in the chat's tree (ctl), and on the forge
   no branch but `main` and no pull request (the API). Wait 2 minutes and check again: nothing
   moved.
7. **The answer,** from the person's inbox through ctl, these words exactly:

   > Go ahead. The CSV's first line must be the header fruit,count.

   Then follow the change through ctl (the change's panel, its waits, the chat's own answers)
   and the forge through its API. Expected: its pull request opened; CI at its first head
   **failed**, the `changes` job naming the missing or incomplete `changes/<number>.md`; temper's change
   showing the failure and a repair, through ctl; a new head on the same pull request adding
   that file; CI passing there; an agent's review passing; the change landed on `main`. Read on
   the forge: the pull request's commits, the runs at each head and their jobs, and each failed
   job's log (qa.md 5). Read `src/main.rs` and `changes/` at `main`'s new head. Time each phase
   from the request: the question in the inbox, the answer sent, the change made, the pull
   request, the first CI's failure, the repair's start and its push, CI passed, the review,
   landed, the chat's last word.
8. **Clean up:** close the chat, `W/give-back` and its check, stop the box. Keep all of `W`: the
   box's directories hold the run's transcripts, which become the rung's fake twin (testing.md
   4.1).

## What passes

All of these, read on both sides: the work's question about its plan reached the person's inbox
before anything changed (whether the work asked it, or the chat asked it for the work), and the
person's answer reached the work before its change was made; nothing pushed and no change made
before the answer; CI at the change's first head red on the box's runner, its `changes` job red
for the missing or incomplete `changes/<number>.md` alone; the repair temper's, delegated by the change and not by the chat or
the person, its brief's failure the one CI showed; both jobs green at the repaired head, on the
same pull request; `main` holding `changes/<number>.md` and the header `fruit,count`; an agent's
review passed at the landed head; the chat saying it landed; the login exported back to
`W/account.json` (non-empty, 0600) before the box stopped; temper's account and the forge's
agreeing at each step. A run where the work went ahead without asking walked neither B0 nor B1;
a run whose first head passed CI did not walk B1: its verdict says so, and it is no pass.

## Appendix: the person's gate (edition 2, 11-01)

Edition 2 (11-01): edition 1's rule, `changes/<its number>.md`, could be met by a first
push that guessed its pull request's number (on an owned Forgejo, issues and pull requests share
one numbering, and a goal's tracking issue takes #1), so a careful model's first head passed and
the repair was never walked (walks qlc-b1-1, qlc-b2-1). The note must now also name when its pull
request was opened, which no push can know before the pull request exists; CI prints it when the
note is missing or wrong. A run's red log ends with the one line `Missing changes/<number>.md with
the line: opened <time>`. A run whose log ends `No pull request time in the event` is a setup to
redo (the forge's event lacks the time), never a finding.

`.forgejo/workflows/ci.yml`:

```yaml
on:
  push:
    branches: [main]
  pull_request:
jobs:
  check:
    runs-on: temper-host
    steps:
      - run: echo "temper's starter check passed; replace this step with your own checks"
  changes:
    if: ${{ github.event_name == 'pull_request' }}
    runs-on: temper-host
    steps:
      - name: Each pull request adds changes/<its number>.md, naming when it was opened
        run: |
          n='${{ github.event.pull_request.number }}'
          opened='${{ github.event.pull_request.created_at }}'
          echo "pull_request.number=$n opened=$opened"
          test -n "$opened" || { echo 'No pull request time in the event'; exit 2; }
          auth=$(printf 'x-access-token:%s' "${{ github.token }}" | base64 -w0)
          git -c "http.extraHeader=Authorization: basic $auth" clone --quiet --no-checkout "${{ github.server_url }}/${{ github.repository }}.git" repo
          git -C repo checkout --quiet '${{ github.event.pull_request.head.sha }}'
          f="repo/changes/$n.md"
          test -f "$f" && grep -qxF "opened $opened" "$f" || { echo "Missing changes/$n.md with the line: opened $opened"; exit 1; }
```

`CONTRIBUTING.md`:

```markdown
# Contributing

Each pull request adds `changes/<its number>.md`: one line saying what it
changes, for the next release's notes, then the line `opened <when the pull
request was opened>`, exactly as CI prints it. CI checks it.
```
