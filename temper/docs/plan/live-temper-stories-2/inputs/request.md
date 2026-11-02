# live-temper: both stories land, then fail the after-restart CI line on the change page (11-02)

Plan live-temper-stories (landed 11-02 20:29Z, main 6ce795f9d) fixed the account import and the
delegate check. In fab's after-landing pair at 6ce795f9d (hetzner8
`~/src/fabrig/fab/after-landing/6ce795f9da61-20261102T202920809314Z/live-temper.log`) both shelf-login
stories (`temper::live a_person_asks_chatgpt_to_land_a_change_on_the_owned_forge`, 102 s, and
`a_person_asks_claude_code_to_land_a_change_on_the_owned_forge`, 66 s) now run the whole story on
real models: the chat's change lands (PR #1, `Landed · merge commit …`, CI passed, gate 1 review
approved, the login back: "LOGIN BACK: regular file mode=600"), the box restarts ("CHANGE ENDED after
restart after 0s"), and then both fail at temper/crates/temper/tests/live.rs ~35 (`ci_shown`):

> CI on temper's change page: … CI at its head / Passed / Gate 1 · review · blocking / Approved …

After the restart the story expects `CI at its head\nNot read since the restart` (its comment: "after
the restart, which forgets it, not read again on the landed change (domain/forge.md, 8.8)"); the page
shows `Passed`.

## What to do

Settle which is right, from temper/docs/design/domain/forge.md 8.8 and the change page's design: if
a restart now keeps a landed change's CI (the design moved, or a later change made it durable), bring
the story's expectation and its comment to the design; if the design still says a restart forgets it
and the page shows a remembered `Passed`, repair the page or the state behind it in the lowest world
that shows it (a restart world with a landed change), red before, green after. Say which in the
story's design text.

## Acceptance

fab's after-landing pair runs both stories green on a main holding the fix (the stories need the
lent logins: run the worlds and harness tests in the slot).

## Out of scope

fab's after-landing pair; the other live stories.
