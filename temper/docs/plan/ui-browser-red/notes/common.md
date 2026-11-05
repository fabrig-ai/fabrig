# ui-browser-red: what every task holds to

Cited by each task of this plan.

- **The plan's standing rules** are the section `For every task of this plan` of `temper/docs/plan/ui-browser-red/inputs/request.md`: read it. Decide, don't ask.
- **The test stays whole.** `browser_sign_in::signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract` keeps every cookie, reload, reconnect and sign-out assertion. Never change a port, a budget or a timeout to hide a failure, and never mark the test ignored or retried.
- **No real service.** `dev.fabrig.ai` isn't probed (`AGENTS.md`, 3). "A real browser" in this plan is the tier's Chromium against a real `temper serve`.
- **Running the test**, as the browser tier's earlier repairs did, through heavy on a builder:
  - alone: `TIERS_WRAPPER=heavy tools/tiers/run temper browser -- --test browser_all -E 'test(=browser_sign_in::signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract)'`;
  - the whole tier: `TIERS_WRAPPER=heavy tools/tiers/run --cpu target/ui-browser-red-cpu temper browser`.
