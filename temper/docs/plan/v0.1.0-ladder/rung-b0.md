# Rung B0, the green light: rung A's CSV change, with the work's question and the person's answer

With `kit.md` beside this file (the machine, the login's move, the configuration, findings and
the report). Backends: **ChatGPT with an owned Forgejo** (the first pair).

## What this is

Rung B0 of temper's ladder to v0.1.0 (`temper/docs/design/draft/v0.1.0.md`, 2.1 and 3, read it
first): rung A's delegated change, "Add CSV export to widget" on a copy of `playground/widget`,
with a question the work asks before it changes anything, and the person's answer
(`temper/docs/design/draft/reference-case.md`, 4.2 and 7.2; mapped in
`temper/docs/design/domain/testing.md`, 2.7: until the third edition's route lands, B0 walks
today's, its pass read from what the person sees: the question in their inbox, their answer
reaching the work before any change). Its fake twin is temper's green-light story
(testing.md 2.4). This brief is the rung's acceptance: it is rerun unchanged until it passes.

## The scenario

1. **The box,** as rung 0 walks it: configured, the four checks `true`, started as a user unit,
   signed in, the budget raised before any forge is set up.
2. **The playground's repository on the owned Forgejo,** made through ctl as the help names it,
   adopted as a project's home. Read it through the API with the read header: its files and
   `main`'s head.
3. **The login in:** ctl's account import of `W/account.json`, `W/give-back` written at once,
   the account usable.
4. **The request,** as the person asks it, through ctl's chat in the widget's project, these
   words exactly:

   > Add CSV export to widget. Before anything is changed, have the work tell me its plan and
   > wait for my go-ahead.

   Then, through ctl alone, follow it: the chat's reply; its tree; the person's inbox, waiting
   on it rather than reading it again and again. Expected: the chat delegates the work, the work
   asks its question (its plan, and whether to go ahead), and the question waits in the
   person's inbox, its words readable through ctl.
5. **Before answering,** check both sides: no change in the chat's tree (ctl), and on the forge
   no branch but `main` and no pull request (the API). Wait 2 minutes and check again: nothing
   moved.
6. **The answer,** from the person's inbox through ctl, these words exactly:

   > Go ahead. The CSV's first line must be the header fruit,count.

   Then follow the change as rung A does, through ctl (the change's panel, its waits, the chat's
   own answers) and the forge through its API: its pull request, CI passing on the box's runner,
   an agent's review passing, the change landed on `main`. Read `src/main.rs` at `main`'s new
   head: the CSV's header is `fruit,count`. Time each phase from the request: the question in
   the inbox, the answer sent, the change made, the pull request, CI passed, the review, landed,
   the chat's last word.
7. **Clean up:** close the chat, `W/give-back` and its check, stop the box. Keep all of `W`: the
   box's directories hold the run's transcripts, which become the rung's fake twin (testing.md
   4.1).

## What passes

All of these, read on both sides: the work's question about its plan reached the person's inbox
before anything changed (whether the work asked it, or the chat asked it for the work), and the
person's answer reached the work before its change was made; nothing pushed and no change made
before the answer; the change landed, with CI green at its head and an agent's review passed;
`main`'s `src/main.rs` holding the header `fruit,count`; the chat saying it landed; the login
exported back to `W/account.json` (non-empty, 0600) before the box stopped; temper's account
and the forge's agreeing at each step. A run where the question is asked only in a chat reply,
never in the inbox, or where the work goes ahead without asking, did not walk B0: its verdict
says so, and it is no pass.
