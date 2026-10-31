+++
id = "lib-forge-setup"
part = "lib-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-lib", "abort-setup"]
files = ["skein/crates/skein-forge-setup/src/**", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 30
+++
# lib's primitives in skein-forge-setup

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/lib-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.4. After abort-setup, which shares the host's files.

## Sites

- `src/host.rs:376` `earlier(&mut Option<Time>, Time)` →
  `skein_lib::earliest`.
- `src/github/manifest.rs:109` and `src/host/states.rs:104` `hex` →
  `hex::digit`.
- `src/github/app.rs:38` `decimal(id) -> Box<[u8]>` →
  `Decimal::of(id).as_bytes()`.
- `src/reconcile.rs:379` `hex(&Secret) -> Secret` encodes a whole
  secret: it stays, built on `hex::digit` (decisions.md).

## Acceptance

- `earliest` falls by one, `hex_decimal` by three.
- `heavy cargo nextest run -p skein-forge-setup -p <tests/forge/setup's
  package>`, with counts.
