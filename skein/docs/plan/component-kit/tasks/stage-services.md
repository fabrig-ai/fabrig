+++
id = "stage-services"
part = "stage-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-io", "shell-surfaces", "echo-stage"]
files = ["skein/tests/llm-connection/src/simulated.rs", "skein/tests/oauth-accounts/src/sign_in.rs", "skein/tests/oauth-accounts/src/sign_in/browser.rs", "skein/tests/oauth-accounts/src/world.rs", "skein/tests/tls/src/native.rs", "skein/tests/tls/tests/suite/fake_peer.rs", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 45
+++
# Stage sites in the llm-connection, oauth-accounts and tls worlds

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/stage-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.3.

## Sites

- `skein/tests/llm-connection/src/simulated.rs:160`.
- `skein/tests/oauth-accounts/src/sign_in.rs:432`,
  `src/sign_in/browser.rs:358`, `src/world.rs:679`.
- `skein/tests/tls/src/native.rs:643`,
  `tests/suite/fake_peer.rs:102`.

## Pre-approved defaults

- Leave the `enum Process` dispatchers in these files (`simulated.rs:278`,
  `sign_in.rs:539`, `world.rs:808`, `fake_peer.rs:221`) as they are:
  forwarding-sites moves them.

## Acceptance

- `io_resume` falls by six, less named sites.
- `heavy cargo nextest run -p <each world's package>`, default then
  fuzzy, with counts.

## Answers

2026-10-31. May the run skip `python3 -I tools/slots/trim <slot>`, which
this tree does not have? Yes: skip it and continue. This task gives no trim
command; fab prepares the slot before the run, and heavy waits for disk by
itself (exit 75 is `Infra: disk`, reported, never worked around). Free disk
only in your own `target/`.
