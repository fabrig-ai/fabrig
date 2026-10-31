+++
id = "lib-wire"
part = "lib-sites"
kind = "migration"
needs = ["kit-lib", "counts"]
files = ["skein/crates/skein-http/src/client/head.rs", "skein/crates/skein-http/src/server/head.rs", "skein/crates/skein-json/src/writer.rs", "skein/crates/skein-json/src/string.rs", "skein/crates/skein-codec/src/patch/plan.rs", "skein/crates/skein-codec/src/patch/text.rs", "skein/crates/skein-ui/src/phrases.rs", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 35
+++
# http, json, codec and ui on lib's hex and decimal

## Goal

Draft, 3.4: the protocol crates' private `hex` and `decimal` move onto
`skein_lib::hex` and `Decimal`, by `notes/lib.md`.

Acceptance:

- `skein-http`: `client/head.rs:281` and `server/head.rs:380` (`fn
  decimal`, identical) go; their callers use `Decimal::parse`.
- `skein-json`: `writer.rs:593` (`fn hex`, a digit) goes for
  `hex::digit`; `string.rs:260` (`fn hex(unit, byte)`) keeps its name and
  work, and takes the digit's value from `hex::value` (a `None` is its
  `Error::Escape`).
- `skein-codec`: `patch/plan.rs:1058` (`fn hex`, a value) goes for
  `hex::value`; `patch/text.rs:327` (`pub(crate) fn decimal`) goes for
  `Decimal::parse`, by `lib.md`'s "a parse that differs" default.
- `skein-ui`: `phrases.rs:533` (`fn decimal(value, writer)`) goes; its
  callers write `Decimal::of(value).as_bytes()`.
- `hex_decimal` falls by 6 (`--write`).
- `heavy cargo nextest run -p skein-http -p skein-http-world -p
  skein-json -p skein-json-world -p skein-codec -p skein-codec-tests -p
  skein-ui -p skein-ui-tests` passes (counts named); clippy clean for the
  four crates. `skein-tls` is also checked alone: `heavy cargo check -p
  skein-tls` (skein's AGENTS.md).

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/lib.md`.
- `skein_lib::hex`, `Decimal::parse` (`skein/crates/skein-lib/src/hex.rs`,
  `decimal.rs`) on your base.

## Owns and may change

- The files in the header, `skein/scripts/old-forms.txt`.
- `skein/tests/json/src/reference.rs`'s `hex` is the independent
  reference parser: leave it.
- If a codec or ui call site sits in a file not listed (the function's
  callers), change only the call: name the file in the report.

## Steps

1. Each site by `lib.md`; differences kept by its defaults are named.
2. `--write` from `skein/`; checks of `common.md`; commit; push.

## Report

Per `common.md`.
