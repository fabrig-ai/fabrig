+++
id = "lib-llm"
part = "lib-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-lib"]
files = ["skein/crates/skein-llm/src/**", "skein/crates/skein-llm-connection/src/**", "skein/scripts/old-forms.txt"]
effort = "medium"
minutes = 30
+++
# lib's primitives in skein-llm and skein-llm-connection

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/lib-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.4.

## Sites

- skein-llm: `src/affinity.rs:30` `hex` → `hex::digit`;
  `src/openai/common.rs:373` `decimal` → `Decimal::parse`.
- skein-llm-connection: `src/component.rs:24` `enum Lifecycle` →
  `skein_lib::Lifecycle`; `src/deadlines.rs:159` `earlier` →
  `skein_lib::earliest`.

## Acceptance

- `hex_decimal` falls by two, `lifecycle` and `earliest` by one each.
- `heavy cargo nextest run -p skein-llm -p skein-llm-connection` and
  their worlds (tests/llm, tests/llm-connection), with counts.
