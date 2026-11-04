# live-login-guard: a panic after import can't strand a login

Director and agents/creds, 11-04. After-landing's `live-temper` run e7fcabf1fa6b (11-03 18:28Z): `a_person_asks_chatgpt_to_land...` panicked at `ctl.rs:38` right after `moved in`, before any export, so the account record died with the box and chatgpt-3 had to be retired. Its fab side (returns, sweep) is agents/loop's plan al-logins; this is the test side.

Code: `temper/crates/temper/tests/support/account_move.rs` (import, `moved in`) and `support/forge_land.rs` (~1163 the record, ~2173 `LOGIN NOT BACK`), `support/forge_land/rung.rs` (~567).

Goal: from the moment a lent record moves into a box until it's exported back, every exit path exports it: a panic anywhere in the story, a failed assertion, an early return. A guard value created right after the import does that in `Drop` (running while the box is still up, and not panicking itself while unwinding), once. An explicit export at the story's end disarms it. If the export fails, the guard writes the `LOGIN NOT BACK` notice with the store's path, as today, and leaves the store in place.

Rules:

- One direct task, test support only; no product change, no change to what a story asserts.
- A test that fails on the old code, through the fake backends and no live login: a story panicking after import still leaves the record back at the lent path, 0600.
- Check every live story that imports, Claude's and ChatGPT's. Claude's directory kind is used in place, so it probably needs nothing; say so in the report.
- No probes against real services. Never read a key, token or credential record: stat and sha256 at most.
- Keep every part in the plan. Wait for a designer before reporting `Done:`.
