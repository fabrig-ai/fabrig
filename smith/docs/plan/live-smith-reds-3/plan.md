# live-smith-reds-3: the plan

One `live-smith` story still red after plan live-smith-reds-2 landed (request:
`inputs/request.md`), read from the after-landing pair of main df41529d3ee1 on hetzner8
(`~/src/fabrig/fab/after-landing/df41529d3ee1-20261102T183621700067Z/`: the whole run
`live-smith.log`, the rerun `live-smith-1.log`). Every other live story passed there.

| Test (`smith::live`) | Where it failed | Cause | Tasks |
|---|---|---|---|
| `a_run_refreshes_its_grant_at_the_issuer` | `live.rs:47`, both runs, after an accepted run; whole run: `generation=9 refreshable=true expires_in=2800s earliest_in=2438s`, before and after alike; rerun: `2698s`, `2336s` | the story, not the product: OpenAI's issuer sets an earliest refresh time on every token, 362 s before its expiry in both runs, and no refresh is asked before it whatever the lead (skein `oauth.md`, 2 and 6.3). The pairs run minutes apart (18:03, 18:14, 18:25, 18:36) and a ChatGPT run refreshes the record once it expires, so the story meets a fresh record | `refresh-story` |

What the request asked, answered:

- **Skipped or not kept?** Skipped, by design: `skein-oauth-accounts` `refresh_ready` holds every
  refresh until the record's earliest refresh time, and the raised lead does not override it.
  smith's design has no other forced refresh (a rejected token's one refresh is also held).
- **The lowest world** already shows it, green: `smith/tests/end-to-end/credentials.rs`,
  `an_exec_before_the_earliest_refresh_time_keeps_its_record` (raised lead, earliest refresh time
  ahead: no refresh, generation kept; from plan live-smith-reds-2), and skein's
  `no_refresh_starts_before_the_earliest_refresh_time`. No product fix: the product meets the
  design. The task runs the twin once on its base.
- **The record's freshness** is not fab's pre-lending refresh: `smith-live` is a fixed shelf
  kind, never refreshed by fab (`fab/after_landing.py`, `REFRESHED = ("codex", "claude")`).
  It is the pairs' own ChatGPT runs that refresh it, once it expires.
- **The story** now sets up the condition it checks, from the record's facts read before its run
  (`smith/docs/design/testing.md`, 2.3): no earliest refresh time, or one past, expects the
  refresh; one more than five minutes ahead expects the record kept and the run completed on the
  held token; one nearer is waited out (at most five minutes, within the profile's 20-minute
  global timeout), then the refresh is expected; no refresh token fails by name. It is renamed
  `a_run_refreshes_its_grant_when_the_issuer_allows`. On the pair's usual fresh record it checks
  the kept case, which proves live that smith honours OpenAI's earliest refresh time on a raised
  lead; the refresh and rotation at the issuer show in the first ChatGPT run that meets an
  expired record, which cannot run without them.

## Parts and tasks

- **refresh-story** (slice 1, needs nothing): `support/live.rs` gains `RefreshCase`,
  `refresh_case` and the margin and bound constants; `live_configuration.rs` pins every case
  offline; `live.rs`'s story checks its case and names it in its message.

## Design written by this cut

- `smith/docs/design/testing.md`, 2.3: the story "a run refreshes its grant when the issuer
  allows", and in "A failure says why" the case the refresh story checked; 8: the open question
  "The refresh story and an earliest refresh time" is settled and removed.
- `smith/docs/design/testing.status.md`, 2.3: the refresh story as it stands at df41529d3ee1
  (the three reds plan live-smith-reds-2 closed leave it).
- `smith/docs/design/STATUS.md`: its summary names the refresh story's expectation.

## Critical path

`refresh-story` alone (50 min), then the after-landing pair on a main holding it.

## After landing

The acceptance is the after-landing pair on a main holding `refresh-story`: the story is green
in its kept case (the usual one) or its refreshed case. A red names its case and the record's
facts: a kept case whose generation rose means a refresh before the issuer's earliest time (a
product red, reproduced in `credentials.rs` first); a refreshed case whose generation stayed
means the product path differs live from `an_exec_on_a_raised_lead_refreshes_its_record_at_once`.

## Overlaps with drafts beside this one

None: `sign-in-wait` owns `smith-local-shell`'s credentials and `shell.md`; `v0.1.0-ladder` and
`reference-m8` touch temper and jig. This plan changes only three files under
`smith/tests/end-to-end/` and `smith/docs/design/testing*.md`, `STATUS.md`.
