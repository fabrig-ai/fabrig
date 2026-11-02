+++
id = "sign-in-wait"
part = "sign-in-wait"
kind = "feature"
needs = []
files = ["smith/crates/smith-local-shell/src/credentials/product.rs", "smith/crates/smith-local-shell/src/credentials/login/**", "smith/crates/smith-local-shell/src/credentials/tests*", "smith/crates/smith-local-shell/tests/**", "smith/docs/design/shell.md"]
effort = "medium"
minutes = 30
+++
# smith login's sign-in waits 10 minutes and says so

Read first: `smith/docs/plan/sign-in-wait/notes/common.md` and
`smith/docs/plan/sign-in-wait/inputs/request.md`.

## What to build

- **`smith/crates/smith-local-shell/src/credentials/product.rs`:** `sign_in_time` becomes
  `Duration::from_secs(600)`, 10 minutes. Keep the other limits.
- **`login/provider/driver.rs`:** on `CredentialRequest::Visit`, after the `Sign in:` line (and
  the `ssh -L` line when it prints one), print one more line from the same limit:
  `Waiting up to 10 minutes for the browser.` Derive the minutes from `sign_in_time`; do not
  write the number twice.
- **The failure when the deadline passes:** today it prints `account <name>: not signed in`.
  When the sign-in ended because its deadline passed, print
  `account <name>: not signed in: no answer from the browser within 10 minutes` instead. Other
  failures keep their wording.
- **`smith/docs/design/shell.md`:**
  - 4.5, the `sign_in` bullet: smith waits up to the sign-in deadline (10 minutes, 6.5) and says
    so, and says when that deadline ends a sign-in;
  - 6.5: add a `sign_in_time` row, `10 minutes (raised from 120 s, 2026-11-02)`, marked
    provisional like its neighbours.

## Acceptance

- A test shows a login's Visit output includes the waiting line, built from `sign_in_time`.
- A test shows the deadline failure's wording; existing login tests stay green.
- smith's own checks, as its AGENTS.md says (fmt, clippy, the affected crates' tests), are green.

## Pre-approved defaults

- If the driver cannot tell a deadline from other failures without a new variant, add the
  smallest one that can (a `Failed` reason or a flag), within the files above.
- Phrase the waiting line to match smith's other terminal lines, as long as it names the
  minutes.

## Report

Give the two new lines as smith prints them, the design row, and the tests added.
