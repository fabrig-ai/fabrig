# Rung M8, tiers and verdicts: chat fixes into main verified by a tier, green clearing, a red rerun, blamed and bisected

With `kit.md` beside this file (the machine, the login's move, the configuration, findings and
the report). Backends: **ChatGPT with an owned Forgejo**.

## What this is

The walk of the reference case's milestone M8, tiers and verdicts on main
(`temper/docs/design/draft/reference-case.execution.md`; the design: `temper/docs/design/domain/after-landing.md`,
1 to 6; the story: `temper/docs/design/draft/reference-landing.md`, section 7): chat fixes go into
main the way every change does, and a tier the person configures verifies them after landing. Green
clears a landing; a red reruns its failed tests alone, then is blamed on the landing that broke it,
found by bisection where a run covers more than one. `temper help tier` is the person's guide. This
brief is the rung's acceptance: it is rerun unchanged until it passes.

## The scenario

1. **The box,** as rung 0 walks it: configured, the four checks `true`, started as a user unit,
   signed in, the budget raised before any forge is set up.
2. **The playground's repository on the owned Forgejo,** made through ctl as the help names it,
   adopted as a project's home. Read it through the API with the read header: its files and
   `main`'s head.
3. **The person's tier:** as the person, with the write header, commit to `main` through
   Forgejo's API, in one commit with the message "A slow tier: the README checks", the two files of
   the appendix, exactly: `.forgejo/workflows/slow.yml` and `tests/slow.sh` (mode 0755 if the API
   lets you set it, else the workflow runs it with `sh`). Then configure the tier as `temper help
   tier` says: the repository's `landing` member in CONFIG gets `"tiers": [{"name": "slow",
   "workflow": "slow.yml", "ignored": [], "artifact": "report", "format": "ids"}]` (or the format
   name the help gives for one test id per line), beside its gates, and the box restarted. Say what
   ctl shows of the tier, if anything.
4. **The login in:** ctl's account import of `W/account.json`, `W/give-back` written at once,
   the account usable.
5. **The first fix,** as the person asks it, through ctl's chat in the widget's project, these
   words exactly:

   > Add the line "Tier walk: first." to the end of README.md in widget.

   Follow it as rung A does (the change's panel, its waits, the forge through its API): its pull
   request, CI passing on the box's runner, an agent's review, landed on `main`. Then
   `change show CHANGE --until cleared`: the change `verifying`, the tier's run task (`task show`)
   and its dispatched run on the forge (on its branch `temper/tier/slow.N`, its `report`
   artifact), green, and the change `cleared`, its result saying `Cleared by slow run N.`.
6. **Two fixes in one run's range:** while the tier's first run is still running (its full run
   sleeps ten minutes), start two more chats, one after the other, these words exactly:

   > Add the line "BREAK the slow check" to the end of README.md in widget.

   > Add the line "Tier walk: third." to the end of README.md in widget.

   Follow both to landed. The tier's next run starts at the newest tip once the first ends, so it
   covers both landings. Expected: that run red on `no-break` alone; the failed test rerun alone at
   the same tip, failing again (not a flake); then bisected: `no-break` run alone at the second
   fix's commit, failing, so the red is blamed on the second fix; the third fix cleared. Read on
   both sides: the change panels (`red` naming the run and the failing test; `cleared`), the run
   tasks, the dispatched runs and their inputs (`temper_commit`, `temper_tests`) on the forge, the
   artifacts, and that each `temper/tier/slow.N` branch is gone when its run ends. If the two
   landings did not fall in one range (the second run covered one landing), say so: the blame is
   then direct and the bisection not walked.
7. **The chats' words:** what each chat told the person about its change after landing: that it
   landed, and whether it says cleared or red; say what it said.
8. **Clean up:** close the chats, `W/give-back` and its check, stop the box. Keep all of `W`.

Time each phase from the first request: the landings, each tier run's start and end, the rerun,
the bisection's runs, each change's `cleared` or `red`.

## What passes

All of these, read on both sides: the tier configured by the person through CONFIG and the help
alone; the first fix landed, `verifying`, then `cleared` by the tier's green run, its result saying
so; the second and third fixes landed in one run's range; that run red on `no-break`, its failed
test rerun alone at the same tip and failing again; bisection running it alone at the second fix's
commit and blaming the red on the second fix alone; the third fix cleared; each run on its own
`temper/tier/slow.N` branch, deleted when it ends; temper's account (panels, run tasks, results)
and the forge's (dispatched runs, inputs, artifacts) agreeing at each step; the login exported
back to `W/account.json` (non-empty, 0600) before the box stopped. Every step reached from `temper
help` and `temper ctl help` alone, or each place they did not get you there filed as a finding. A
run where the tier never ran, or ran but temper never judged it, did not walk M8: its verdict says
so, and it is no pass.

## Appendix: the person's tier

`.forgejo/workflows/slow.yml`:

```yaml
on:
  workflow_dispatch:
    inputs:
      temper_commit:
        required: true
      temper_key:
        required: true
      temper_tests:
        required: false
        default: ""
jobs:
  slow:
    runs-on: temper-host
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ inputs.temper_commit }}
      - name: The README checks
        env:
          TEMPER_TESTS: ${{ inputs.temper_tests }}
        run: sh tests/slow.sh
      - uses: actions/upload-artifact@v3
        if: always()
        with:
          name: report
          path: report/failed.txt
```

`tests/slow.sh`:

```sh
#!/bin/sh
# Two README checks; with TEMPER_TESTS (one id per line) only those, at once; else all, slowly.
set -u
mkdir -p report
: > report/failed.txt
tests=${TEMPER_TESTS:-}
if [ -z "$tests" ]; then
    tests="readme-heading
no-break"
    sleep 600
fi
status=0
for t in $tests; do
    case $t in
        readme-heading) head -n 1 README.md | grep -q '^# ' ;;
        no-break) ! grep -q 'BREAK' README.md ;;
        *) false ;;
    esac || { echo "$t" >> report/failed.txt; status=1; }
done
exit $status
```
