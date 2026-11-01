# Rung A, the playground: its three requests on a copy of playground/widget on an owned Forgejo, through ctl

With `kit.md` beside this file (the machine, the login's move, the configuration, findings and
the report). Backends: **ChatGPT with an owned Forgejo** (the first pair).

## What this is

Rung A of temper's ladder to v0.1.0 (`temper/docs/design/draft/v0.1.0.md`, 2.1 and 3, read it
first): "the playground's three suggested requests (`../protocol/setup.md`, 8.4) on a copy of
`playground/widget` on the box's forge: each answered, and 'Add CSV export to widget'
delegated, pushed, passed by CI and an agent's review, and landed". It stands on rung 0's box.
This brief is the rung's acceptance: it is rerun unchanged until it passes.

## The scenario

1. **The box,** as rung 0 walks it (`temper help box` steps 1 to 3, kit "The box's
   configuration"): configured, the four checks `true`, started as a user unit, signed in, the
   budget raised before any forge is set up.
2. **The playground's repository on the owned Forgejo:** a copy of `playground/widget` made
   through ctl, adopted as a project's home, as the help names it (`temper help forge`,
   `temper ctl help forge`). Read it through the forge's API with the read header: its files
   and `main`'s head. Find the three suggested requests from temper itself (its help, ctl, the
   client's suggestions as ctl shows them) and say where you found them.
3. **The login in:** ctl's account import of `W/account.json`, `W/give-back` written at once,
   the account usable.
4. **The three requests,** each in its own chat in the widget's project, as a person asks it,
   in the words temper suggests, nothing appended. Each answered, in words or by delegating a
   change; a change delegated for the other two goes on (it may land; say so). "Add CSV export
   to widget" must be delegated, its change pushed, its pull request opened, CI passing on the
   box's runner, an agent's review passing, and the change landed on `main`. Follow each
   through ctl (the chat's replies, its tree, a change's panel and its waits) and the forge
   through its API (the pull request, its files and reviews, CI at its head, `main`'s head and
   the landed `src/main.rs`). Time each phase: the reply, the change made, the pull request,
   CI passed, the review, landed, the chat's last word. Close each chat once answered.
5. **The login out and the stop:** `W/give-back` and its check, then stop the box (its last
   line `Stopped`). Keep all of `W`.

## What passes

All of these, read on both sides: the three requests each answered by the model through ctl,
truthfully (an answer that says something is so is checked on the forge); the CSV change
delegated, its pull request opened, CI green at its head on the box's runner, an agent's
review passed, landed on `main`, and `main`'s new head holding the CSV export; temper's account
(the change's step, its pull request and merge commit, the chat's words) and the forge's
agreeing at each step; the chat saying it landed; the login exported back to `W/account.json`
(non-empty, 0600) before the box stopped. Every step reached from `temper help` and `temper ctl
help` alone, or each place they did not get you there filed as a finding.
