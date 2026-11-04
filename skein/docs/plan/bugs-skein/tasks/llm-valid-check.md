+++
id = "llm-valid-check"
part = "llm-connection"
kind = "feature"
needs = []
files = ["skein/crates/skein-llm-connection/src/limits.rs", "skein/crates/skein-llm-connection/src/component.rs", "skein/crates/skein-llm-connection/src/tests.rs", "skein/docs/design/llm-connection.status.md"]
effort = "high"
minutes = 35
+++

# llm-connection's `valid` holds exactly when `Component::new` would

## Goal

`skein-llm-connection`'s `valid` (`src/limits.rs`) lacks seven refusals `Component::new` makes
(`src/component.rs`): `HttpReadTlsRead`, `HttpSendTlsSend`, `TlsReadIoIntake`, `TlsSendIoOutput`,
`HttpReadIoIntake`, `HttpSendIoOutput` and `PlaintextAddress`. One check serves both. (Bug 6,
re-checked: holds.)

Acceptance:

- `limits.rs` has `pub(crate) fn check(limits, endpoints) -> Result<(), EndpointError>` holding
  `new`'s checks, in their order, on a copy of each endpoint's limits, ending with `valid`'s
  `client::valid` and `tls::valid` loop.
- `valid` is `check(..).is_ok()`; `new` calls `check`, then writes each `http.request`.
- `valid`'s signature, `Component::new`'s and `EndpointError` do not change.
- The tests below fail on the old code.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/llm-connection.md`, 7 ("`valid(limits, endpoints)`").

## Owns and may change

- Owns: `src/limits.rs`, `src/component.rs` (`new`'s checks), `src/tests.rs`.
- `skein/docs/design/llm-connection.status.md`: remove this plan's **Not yet** entry
  (`notes/common.md`).
- No reader outside the crate calls this `valid` (surveyed). Mind the order:
  `valid_refuses_unrepresentable_tls_intake` expects `Limits`, not a stream error.
- `smith-protocol-llm/src/component.rs` has a copy of its own: not this bug, `## Left`.

## Steps

1. In `src/tests.rs`, add `assert!(!valid(&config, &endpoints))` before `Component::new` in
   `construction_names_each_stream_capacity_relationship`, and the same when `!admitted` in the
   plaintext-address test. Run on the old code and quote the failure.
2. Write `check` and move `new`'s checks into it.
3. `heavy cargo nextest run -p skein-llm-connection --lib`, then the focused tests of
   `skein-llm-connection` and `skein-llm-connection-world`. `cargo fmt`,
   `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-llm-connection
   --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the tests showed on the old code.
