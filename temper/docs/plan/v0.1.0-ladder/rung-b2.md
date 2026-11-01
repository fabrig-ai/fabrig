# Rung B2, two delegates: B1, with an advisor's report and two delegates, one depending on the other

With `kit.md` beside this file (the machine, the login's move, the configuration, findings and
the report). Backends: **ChatGPT with an owned Forgejo** (the first pair).

## What this is

Rung B2 of temper's ladder to v0.1.0 (`temper/docs/design/draft/v0.1.0.md`, 2.1 and 3, read it
first): rung B1's story, the green light and the person's gate, with an advisor's report and two
delegates, one depending on the other (`temper/docs/design/draft/reference-case.md`, 4.2 and
6.2; mapped in `temper/docs/design/domain/testing.md`, 2.7: B2's advisor reports before the
plan): the work surveyed by an advisor that changes nothing, its report delivered before the plan
is agreed, then two changes, the second started only once the first has landed, each through the
person's gate and its repair, and the work's spend read. The gate is B1's. This brief is the
rung's acceptance: it is rerun unchanged until it passes.

## The scenario

1. **The box,** as rung 0 walks it: configured, the four checks `true`, started as a user unit,
   signed in, the budget raised before any forge is set up.
2. **The playground's repository on the owned Forgejo,** made through ctl as the help names it,
   adopted as a project's home. Read it through the API with the read header: its files and
   `main`'s head.
3. **The person's gate,** as B1's step 3: the appendix's two files committed to `main` as the
   person through the API, one commit, message "CI: each pull request adds changes/<its
   number>.md"; the push's CI passed at `main`'s head; what ctl showed of it, if anything.
4. **The login in:** ctl's account import of `W/account.json`, `W/give-back` written at once,
   the account usable.
5. **The request,** as the person asks it, through ctl's chat in the widget's project, these
   words exactly:

   > Add CSV and JSON export to widget. First have an advisor read widget and report how the
   > two exports should share the table's rows, without changing anything. Then tell me the
   > plan and wait for my go-ahead. Make CSV first, and JSON only after CSV has landed, reusing
   > what CSV adds.

   Then, through ctl alone, follow it: the chat's reply; its tree; the advisor's task and its
   result; the person's inbox, waiting on it rather than reading it again and again. Expected:
   the chat delegates the work, the work delegates an advisor, the advisor reports, and only
   then the work asks its question, which waits in the person's inbox, its words readable
   through ctl and naming what the advisor found.
6. **Before answering,** check both sides: no change in the chat's tree (ctl); on the forge no
   branch but `main` and no pull request (the API); the advisor's task ended with its report,
   and nothing on the forge made by it. Wait 2 minutes and check again: nothing moved.
7. **The answer,** from the person's inbox through ctl, these words exactly:

   > Go ahead. CSV's first line is the header fruit,count. JSON is an array of objects with the
   > keys fruit and count.

   Then follow both changes through ctl (the changes' panels, their waits, the tree, the chat's
   own answers) and the forge through its API. Expected: the CSV change made first, its pull
   request opened, CI at its first head **failed** for its missing or incomplete `changes/<number>.md`, its
   repair, a green run, an agent's review passed, landed; the JSON change made only after the
   CSV change landed (its task's start after CSV's merge, read on both sides), the same way
   through its own red run, repair, green run and review, landed after it; the work's report,
   then the chat's. Read on the forge each pull request's commits, the runs at each head, each
   failed job's log (qa.md 5), and at `main`'s new head `src/main.rs` and `changes/`. Read
   through ctl the work's spend (its task's document) and say what it was. Time each phase from
   the request: the advisor's start and report, the question in the inbox, the answer sent,
   each change made, each pull request, each first CI's failure, each repair's push, each green
   run, each review, each landing, the chat's last word.
8. **Clean up:** close the chat, `W/give-back` and its check, stop the box. Keep all of `W`: the
   box's directories hold the run's transcripts, which become the rung's fake twin (testing.md
   4.1).

## What passes

A deep task's page (a change's producer, repairer or reviewer, three ancestors down) may show
its lineage cut at the box's lines and say so: expected on the small profile, not a finding, as
long as its own row reads whole.

All of these, read on both sides: the advisor's report delivered before the question, the
advisor making no branch, pull request or change; the work's question about its plan in the
person's inbox (asked by the work, or by the chat for it), and the person's answer reaching the
work before its change; nothing pushed and no change made before the answer; two changes,
JSON's made only after CSV's landed, each with CI red at its first head for the missing
`changes/<number>.md` (missing or incomplete) alone and its repair temper's, then green at the repaired head and an
agent's review passed; both landed in that order; `main` holding the CSV header `fruit,count`,
the JSON export, and a `changes/<number>.md` for each pull request; the work's spend read through
ctl; the chat saying both landed; the login exported back to `W/account.json` (non-empty, 0600)
before the box stopped; temper's account and the forge's agreeing at each step. A run that
changed anything before the answer, or started JSON before CSV landed, is no pass; one whose
first heads passed CI did not walk B2's gate: its verdict says so, and it is no pass.

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
