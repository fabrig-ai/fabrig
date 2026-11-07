+++
id = "login-wire"
part = "program-email"
kind = "feature"
needs = ["status-email"]
files = ["jig/crates/jig-domain-link/src/lib.rs", "jig/crates/jig-domain-host/src/sources.rs", "jig/crates/jig-protocol-worker-agents/src/sources.rs", "jig/crates/jig-protocol-worker-agents/src/tests/sources.rs", "jig/crates/jig-protocol-worker-wire/schema/worker-wire-v9.schema", "jig/crates/jig-protocol-worker-wire/src/generated/v9.rs", "jig/crates/jig-protocol-worker-wire/golden/**", "jig/crates/jig-protocol-worker-wire/src/tests/sources.rs", "jig/crates/jig-protocol-worker-link/src/translate.rs", "jig/crates/jig-protocol-engine-workers/src/translate.rs", "jig/crates/jig-domain-fleet/src/sources.rs", "jig/tests/host/tests/suite/sources.rs", "jig/tests/worker-link/tests/suite/sources.rs", "jig/tests/engine-workers/tests/suite/sources.rs", "jig/tests/fleet/tests/suite/program_sources.rs"]
effort = "high"
minutes = 90
+++

# A host's logins carry the email each status names

## Goal

A worker's `logins` names the email of a usable login, from smith's check, over the wire, into the fleet's copy of that host's logins. Nothing reads it there yet: `program-report` does.

Acceptance:

- Section 5 of the contracts note, but for `Request::ProgramEmail`, which is `program-report`'s.
- `jig-protocol-worker-agents`, `tests/sources.rs`: `ProgramCheck::Usable { email }` translates to `Answer::Checked { state: LoginState::Usable, email }`, and each other check to `email: None`.
- `jig-protocol-worker-wire`, `tests/sources.rs`, and `jig/tests/worker-link`: a row with an email and a row without round-trip through the link; past 255 bytes or not UTF-8 is refused, as a method past its bound is.
- `jig/tests/host`, `sources.rs`: a check whose email differs from the last one's publishes `logins`, as a state change does, and one that says the same publishes nothing. A row of the existing table.
- `jig/tests/fleet`, `program_sources.rs`: the fleet keeps the email it was told, and one past the bound is out of bound as an over-long detail is.
- `jig/tests/engine-workers`: a row with an email reaches the fleet.
- The focused and fuzzy tests of the packages named, and `jig-domain-core`, pass, and `heavy cargo check --workspace --tests`.

## Inputs

- `jig/docs/design/protocol/channel.md`, 5.1, *logins*.
- `smith/docs/design/protocol/hosts.md`, section 2.
- `temper/docs/plan/account-email/notes/contracts.md`, 2 and 5.
- `temper/docs/plan/account-email/notes/common.md`.

## Owns and may change

Owns the header's files. Readers, each edited only for the field: `Login` literals gain `email: None`, in `jig/testing/jig-channel-kit/src/program_sources.rs`, `jig/testing/jig-core-kit/src/world/{admission,deliveries}.rs`, `jig/testing/jig-test-domain/src/tests/link.rs`, `jig-domain-fleet/src/tests/needs.rs`, `jig-domain-core`'s tests, `jig-protocol-engine-clients/src/tests/programs.rs`, `jig/tests/channel/tests/suite/program_sources.rs`, `temper/crates/temper-service/src/tests/first_model.rs`, `temper/tests/client/src/front/account_acts.rs`, `temper/tests/worker/domain/tests/suite/orders.rs`, `temper/tests/engine/domain/tests/suite/configuring.rs`, `temper/crates/temper-engine-domain/src/admission.rs`, and any other the compiler names. Other types share the name `Login`, in `smith-local-shell` among them: they don't change.

`LoginState` doesn't change: its sixty-odd readers stay as they are.

Options, each pre-approved:

- The email goes on the row, not in `LoginState`, so `Row` in `jig-domain-host/src/sources.rs` keeps it beside `state`, and `changed` compares both. A failed check keeps the row's last email only if it keeps its state; otherwise `None`.
- The client's program host rows (`jig-domain-core/src/documents/programs.rs`) carry no email: it reaches a client only as an account's fact.
- Regenerate with `common.md`'s command; if the worker wire's family isn't under those packages' drift test, run the one that names it, by `jig/tests/codec-families.rs`.

## Steps

```rust
// jig-domain-host, sources.rs
pub enum Answer { Checked { state: LoginState, email: Option<Box<[u8]>> }, SignedOut(SignOutResult) }

// jig-protocol-worker-agents, sources.rs
smith::ProgramCheck::Usable { email } => sources::Answer::Checked { state: LoginState::Usable, email },
```

Each new test breaks its rule once by hand. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents, one command at a time. Commit and push.

## Report

As the implementer's text says.
