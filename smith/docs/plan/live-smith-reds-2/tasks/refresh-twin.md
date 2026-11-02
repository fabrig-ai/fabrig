+++
id = "refresh-twin"
part = "refresh"
kind = "feature"
needs = []
files = ["smith/tests/end-to-end/credentials.rs"]
effort = "high"
minutes = 90
+++
# A fake-backed exec on a raised refresh lead refreshes its record at once

## Goal

`smith::live a_run_refreshes_its_grant_at_the_issuer` fails at every after-landing run since
11-02 10:49 (main 46e6e70d9: "the issuer refresh was kept; generation before=7 after=7"): the
exec, with `-c deployment.refresh_lead=34560000` (400 days, `support::live::with_refresh_lead`),
completes, delivers and exits 0, but the ChatGPT record's generation does not rise. Read offline,
the product should refresh before the first grant: the lead reaches the oauth driver's limits
(`smith/crates/smith-local-shell/src/credentials/product.rs`, `prepare` and `limits`), the
account is a `SignIn` source (`smith/crates/smith-local-service/src/credentials/adopt.rs`), and
`grant_loaded` refreshes when `now >= lead`, the record holds a refresh token and no earliest
refresh time is ahead (`skein/crates/skein-oauth-accounts/src/component.rs`;
`component/accounts.rs`, `refresh_ready`); the write raises the generation under the record's
lock (`component/lock.rs`). Only `live_configuration`'s
`the_refresh_story_lead_outlives_an_access_token` checks the lead, and only that it resolves.

Write the offline twin of the story, through the shipped product, and its earliest-refresh
variant, so the product path is pinned whatever the live record holds.

Acceptance, in `smith/tests/end-to-end/credentials.rs` (the `end_to_end` binary):

- `an_exec_on_a_raised_lead_refreshes_its_record_at_once`: a sign-in account on the fake issuer
  with a seeded current record (generation 1, a refresh token, its access token expiring in about
  an hour, no earliest refresh time); `smith exec --json` with
  `-c deployment.refresh_lead=34560000` against the fake provider. The exec exits 0 with an
  accepted answer, the fake issuer answered one refresh, and the record's generation is 2 after.
- `an_exec_before_the_earliest_refresh_time_keeps_its_record`: the same with the record's
  `earliest_refresh_at` an hour ahead: the run completes on the held token, the issuer answered no
  refresh, and the generation stays 1 (skein/docs/design/oauth.md, 2 and 6.3: "no refresh is
  asked before it").
- If the first is red, the red is fixed in the one crate that holds it, red before and green after.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/live-smith-reds-2/notes/common.md` (read it first).
- `smith/docs/design/testing.md`, 2.3 (the refresh story) and 8 ("The refresh story and an
  earliest refresh time").
- `smith/docs/design/shell.md`, 6.5 (`refresh_lead`) and 7.3 (records across processes).
- `skein/docs/design/oauth.md`, 2 (the earliest refresh time) and 6.3 (refreshed while held).

## Owns and may change

- `smith/tests/end-to-end/credentials.rs`: the two tests and private helpers (`seed_bytes` may take
  the expiry and earliest refresh time as arguments; its callers keep their values).
- Read, never change: `support/mod.rs`, `support/exec.rs`, `support/provider_login.rs`. Call their
  public helpers (`provider_login::keep_record`, `exec::fake_provider_settings`'s endpoint shape,
  `run_product`); copy what is private.
- Pre-approved:
  - The fake issuer: the `end_to_end` harness's `IssuerProcess`
    (`smith/tests/local-process/src/process.rs`, `with_token`) as `support/mod.rs` spawns it for an
    authenticated scenario; its issued access token is the one the fake provider accepts after the
    refresh. If counting the issuer's refreshes needs an observation it does not expose, assert
    the generation only and say so.
  - If no public helper runs `smith exec` with both the fake issuer and the fake provider, write a
    private runner in `credentials.rs` modelled on `support/mod.rs`'s `run_with_provider` (the
    world, the binary, the peer, the issuer) and say so; if that needs a private item of
    `support/`, stop and ask.
  - If the variant's expectations differ (the product refuses the start, or refreshes anyway),
    stop: commit the red test, push, and ask with what it did.
  - If the first test is red, the fix is in smith's code (`smith-local-shell` settings or
    credentials outside `credentials/product.rs` and `credentials/login/**`, which plan
    `sign-in-wait` holds, or `smith-local-service`); a fix needing `product.rs`, skein or a stored
    format is a question.

## Steps

1. Read the inputs, `credentials.rs` (`settings`, `seed_bytes`, `seed`, `record`), `support/mod.rs`
   (`settings`, `run_with_provider`, the issuer's spawn) and `support/exec.rs`.
2. Write the first test; run it:
   `heavy cargo nextest run -p smith --features end-to-end --test end_to_end -E 'test(/raised_lead/)'`.
   Note its first result (green, or the red's assertion) for the report; if red, fix per
   "Pre-approved" and rerun green.
3. Write the variant; run it: `... -E 'test(/earliest_refresh/)'`.
4. `cargo fmt`; `heavy cargo clippy -p smith --features end-to-end --all-targets -- -D warnings`
   (and every crate you changed).
5. `heavy cargo nextest run -p smith --features end-to-end --test end_to_end -E 'test(/credentials/)'`;
   if you changed a crate, its package's tests and its direct dependents'.
6. Commit, push.

## Report

Each test's first result, any fix (crate, test, red and green), the files changed, the test runs
(selected, passed), and implementation and check minutes apart.
