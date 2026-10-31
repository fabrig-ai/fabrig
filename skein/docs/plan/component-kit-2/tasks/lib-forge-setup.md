+++
id = "lib-forge-setup"
part = "lib-sites"
kind = "migration"
needs = ["kit-lib", "counts", "abort-setup"]
files = ["skein/crates/skein-forge-setup/src/**", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 35
+++
# skein-forge-setup on lib's earliest, hex and decimal

## Goal

Draft, 3.4: forge-setup's private `earlier`, its two nibble `hex`s and
its `decimal` writer move onto lib's, by `notes/lib.md`.

Acceptance:

- `src/host.rs:376` (`fn earlier`, `&mut` shape) goes for
  `skein_lib::earliest`.
- `src/github/manifest.rs:109` and `src/host/states.rs:104` (`fn hex`,
  a digit) go for `hex::digit`.
- `src/github/app.rs:38` (`pub(crate) fn decimal(id) -> Box<[u8]>`) goes;
  its callers (`src/github/{accept,app,convert,installations,paste,reconcile}.rs`,
  `src/reachable.rs`) take `Decimal::of(id).as_bytes()` (boxed where
  they need a `Box<[u8]>`).
- `src/reconcile.rs:379` (`hex(&Secret) -> Secret`) encodes a whole
  secret: it keeps its name and work and maps each nibble with
  `hex::digit`.
- `earliest` falls by 1, `hex_decimal` by 3 (`--write`); the pop loops in
  `host/states.rs` use what they pop and stay.
- `heavy cargo nextest run -p skein-forge-setup -p
  skein-forge-setup-world` passes (counts named); clippy clean.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/lib.md`.
- `skein_lib::{earliest, hex, Decimal}` on your base.
- `abort-setup`'s `Request::Abort` is merged on your base: keep it.

## Owns and may change

- `skein/crates/skein-forge-setup/src/**`, `skein/scripts/old-forms.txt`.
- The dialect-mixing `Reply` and the plans' state (the review's themes 2
  and 3) are later runs: change nothing but these sites.

## Steps

1. Each site by `lib.md`.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`.
