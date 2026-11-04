# live-smith: the forced grant refresh story still red after landing (11-02)

Plans live-smith-reds and live-smith-reds-2 brought smith's live pair to green but one story
(besides the API-key story, manual by the user's decision). At main df41529d3, after-landing run
`~/src/fabrig/fab/after-landing/df41529d3ee1-20261102T183621700067Z/` on hetzner8 (its
`live-smith-*.log`), `smith::live a_run_refreshes_its_grant_at_the_issuer` panics at
smith/tests/end-to-end/live.rs ~47, on all three attempts:

> the issuer refresh was kept; record before: generation=9 refreshable=true expires_in=2698s
> earliest_in=2336s; after: generation=9 refreshable=true expires_in=2698s earliest_in=2336s

The run itself completes (5 turns, status Accepted, exit 0); the login record is unchanged: no
refresh happened, or one happened and was not kept. Note: fab refreshes a lent ChatGPT record's
access token before lending it for live pairs (its after_landing.py "Kinds whose tests read the
access token and never refresh it: fab refreshes it first"), so the record arrives fresh, with
`earliest_in` well above zero.

## What to do

Find why the story's forced refresh leaves the record's generation unchanged: whether smith's
forced refresh is skipped while `earliest_in` is above zero (then the story's "forced" does not
force, or the story must ask for what smith's design calls a forced refresh), or whether the
refresh is made but not kept in the record. Reproduce it in smith's lowest world (its login and
refresh fakes, domain tests), repair it there with a test red before and green after, and bring the
live story's expectation to the design (smith's login design names when a forced refresh runs and
what it keeps). If the design says a refresh must not run before `earliest`, the story must set up
the condition it tests (or read a refresh it can force), and say so.

## Acceptance

`live-smith`'s after-landing pair runs this story green on a main holding the fix, or it is moved
or ignored with a reason its design names. A fix is checked by the after-landing pair, not inside
the implementer's run (the live logins are lent to the pair only).

## Out of scope

The API-key story; fab's after-landing code and its pre-lending refresh; the shelf's logins.
