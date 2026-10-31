+++
id = "lib-llm"
part = "lib-sites"
kind = "migration"
needs = ["kit-lib", "counts"]
files = ["skein/crates/skein-llm-connection/src/**", "skein/crates/skein-llm/src/affinity.rs", "skein/crates/skein-llm/src/openai/**", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 30
+++
# llm and llm-connection on lib's primitives

## Goal

Draft, 3.4: llm-connection's private `earlier` and `Lifecycle`, and
llm's private `hex` and `decimal`, move onto lib's, by `notes/lib.md`.

Acceptance:

- `skein-llm-connection`: `src/deadlines.rs:159` (`fn earlier`) goes for
  `skein_lib::earliest`; `src/component.rs:24` (`enum Lifecycle`) goes
  for `skein_lib::Lifecycle`.
- `skein-llm`: `src/affinity.rs:30` (`fn hex`, a digit) goes for
  `hex::digit`; `src/openai/common.rs:373` (`pub(crate) fn decimal`)
  goes for `Decimal::parse`, by `lib.md`'s parse default.
- `earliest` and `lifecycle` each fall by 1, `hex_decimal` by 2 (`--write`).
- `heavy cargo nextest run -p skein-llm -p skein-llm-world -p
  skein-llm-connection -p skein-llm-connection-world` passes (counts
  named); clippy clean for both crates.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/lib.md`.
- `skein_lib::{earliest, Lifecycle, hex, Decimal}` on your base.

## Owns and may change

- The files in the header, `skein/scripts/old-forms.txt`. In skein-llm,
  callers of `decimal` under `src/openai/` change only at the call.
- `skein-fake-llm-protocol`'s test `hex` (`provider/inflate.rs:359`)
  decodes a whole text: not a copy, not yours.
- The llm-connection world (`skein/tests/llm-connection`) is
  `stage-llm`'s: run its tests, do not edit it.

## Steps

1. Each site by `lib.md`.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`.
