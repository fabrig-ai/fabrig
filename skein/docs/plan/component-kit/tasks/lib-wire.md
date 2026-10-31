+++
id = "lib-wire"
part = "lib-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-lib"]
files = ["skein/crates/skein-http/src/**", "skein/crates/skein-json/src/**", "skein/crates/skein-codec/src/**", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 35
+++
# lib's primitives in http, json and codec

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/lib-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.4, for the wire crates.

## Sites

- skein-http: `src/client/head.rs:281` and `src/server/head.rs:380`
  `decimal` (identical) → `Decimal::parse`.
- skein-json: `src/string.rs:260` `hex(unit, byte)` (a `\u` escape's
  accumulator: `hex::value` inside, the function goes where it is then a
  line); `src/writer.rs:593` `hex` → `hex::digit`.
- skein-codec: `src/patch/plan.rs:1058` `hex` → `hex::value`;
  `src/patch/text.rs:327` `decimal` → `Decimal::parse`.

`skein/tests/json/src/reference.rs`'s `hex` is the independent reference
parser: not yours, and it stays (decisions.md).

## Acceptance

- `hex_decimal` falls by six (`--write`).
- `heavy cargo nextest run -p skein-http -p skein-json -p skein-codec`
  and their worlds (tests/http, tests/json, tests/codec), with counts.
