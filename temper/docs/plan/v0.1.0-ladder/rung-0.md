# Rung 0, the ground: a box, its owned Forgejo, the held ChatGPT login in, one chat turn, the login out

With `kit.md` beside this file (the machine, the login's move, the configuration, findings and
the report). Backends: **ChatGPT with an owned Forgejo** (the first pair).

## What this is

Rung 0 of temper's ladder to v0.1.0 (`temper/docs/design/draft/v0.1.0.md`, 2.1): "a box whose
owned Forgejo is set up through `ctl`; the held ChatGPT account moved in, one chat turn, the
account moved out". It is the ground every other rung stands on: temper's own walk `box`
(`temper help box`) on real backends, from empty directories to a stopped box, with the model's
login back where it was. This brief is the rung's acceptance: it is rerun unchanged until it
passes.

## The scenario

1. **Configure** (`temper help box` step 1, kit "The box's configuration"): `temper init` into
   empty `W/box/{config,data,cache}`, the model and room set, the four checks `true`.
2. **Start the box** as a user unit (kit "The machine"), and **sign in** with the link it
   prints, on stdin (step 3). Raise the budget (kit).
3. **Set up Forgejo on this machine** through ctl, as the help's step 4 says (the help's
   own names: org `work`, repository `hello`, the project `me` lists). Read on the forge, with
   the read header the box names: the repository, its files and `main`'s head, and the
   runner's starter workflow.
4. **Move the login in** (step 5): ctl's account import of `W/account.json`; at once write
   `W/give-back` (kit); then wait until the account is usable.
5. **One chat turn,** in the project, these words exactly:

   > What does this project's repository hold? Answer in two sentences.

   Wait for the reply through ctl and read it whole; read the chat's task (its tree, its
   activity). Expected: a model's answer (not a fake's), naming what the repository holds as
   the forge shows it; no change started. Close the chat.
6. **Move the login out** (step 8): `W/give-back`, its `stat` check; then the account list
   shows the account no longer signed in. Then stop the box; its last line ends `Stopped`.
7. **Clean up:** keep all of `W` (the box's directories, its log, your notes and evidence).

## What passes

All of these: the four checks `true` before the start; the box started and signed in; the
owned Forgejo made through ctl alone, temper's account of it (the forge, its repository, the
project's home) agreeing with the forge's API; the login imported and usable; the chat's reply a
real model's, true to what the forge shows; no change made; the login exported back to
`W/account.json` (a non-empty regular file, 0600) before the box stopped; the box's last line
`Stopped`. Every step reached from `temper help` and `temper ctl help` alone, or each place they
did not get you there filed as a finding.
