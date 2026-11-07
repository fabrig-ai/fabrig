+++
id = "status-email"
part = "program-email"
kind = "feature"
needs = []
files = ["smith/crates/smith-protocol-program/src/status.rs", "smith/crates/smith-protocol-program/src/tests.rs", "smith/crates/smith-protocol-program/src/lib.rs", "smith/crates/smith-host-domain/src/program.rs", "smith/crates/smith-host-protocol/src/program.rs", "smith/crates/smith-host-protocol/src/program/tests.rs", "smith/tests/fake-claude/src/commands/status.rs", "smith/tests/hosts/tests/suite/program_source.rs"]
effort = "high"
minutes = 60
+++

# A program's check names the email its status names

## Goal

Claude Code's `claude auth status` names an `email` (read on 2026-11-07, version 2.1.312; `jig/docs/design/domain/providers.md`, 3.3). smith reads it from a usable answer and hands it up beside *usable*. It is words to show: no email, or one that doesn't read, leaves the answer usable.

Acceptance:

- `section 2` of the contracts note, exactly.
- A table test in `smith-protocol-program`, beside its status table, over `email(code, stdout)`:
  - `{"loggedIn":true,"authMethod":"claude.ai","email":"you@example.com"}`, exit 0: `Some`.
  - No `email`; a number; empty; 256 bytes; holding a control byte: `None`, and `status(..)` is still `Usable` for each of them.
  - 255 bytes: `Some`.
  - Signed in otherwise, signed out, exit 1, an unreadable answer: `None`, whatever it names.
- `status()`'s table is otherwise unchanged, its 4096-byte bound among it.
- `smith/tests/hosts`, `program_source.rs`: the fake's usable check answers `ProgramCheck::Usable { email: Some(b"you@example.com") }`.
- The module's header says the walk retains the three fields.
- The focused and fuzzy tests of `smith-protocol-program`, `smith-host-domain`, `smith-host-protocol`, `smith-local-service`, `smith-local-shell` and `smith/tests/hosts` pass, and `heavy cargo check -p jig-protocol-worker-agents --tests`.

## Inputs

- `smith/docs/design/protocol/programs.md`, 3 and 4.1.
- `smith/docs/design/protocol/hosts.md`, section 2.
- `temper/docs/plan/account-email/notes/contracts.md`, 2.
- `temper/docs/plan/account-email/notes/common.md`.

## Owns and may change

Owns the header's files. Readers of `ProgramCheck::Usable`, each edited only for the new field:

- `smith/crates/smith-local-service/src/service/program_check.rs`, `smith/crates/smith-local-shell/src/credentials/program_check.rs`, its `tests.rs`, and `credentials/login/program.rs`: `Usable { .. }`.
- `jig/crates/jig-protocol-worker-agents/src/sources.rs`: `smith::ProgramCheck::Usable { email: _ } => LoginState::Usable`. `login-wire` fills it.

The agent's own check, `smith-protocol-program/src/backend/start.rs`, reads `Status` and stays as it is: `Status` doesn't change.

Options, each pre-approved:

- Reading the answer twice, once for `status` and once for `email`, is fine: a check is rare. A shared collector with the filter's new field is the simplest shape. `limits().tokens` grows to hold it.
- A text past the collector's cap arrives as `Kind::Long`, which `email` treats as absent; `read` must not return `None` on it.
- The fake's status names the email only while signed in, as a `const` beside `status_reply`. A fake-claude test that pins the reply's exact fields gets the new one.

## Steps

```rust
// status.rs
const FILTER: json::Filter = .. // adds Key::Field(b"email"), Keep::Text(json::Cap::new(1))
json::Collector::new(FILTER, &limits, &[32, EMAIL_BYTES + 1])
pub fn email(code: u8, stdout: &[u8]) -> Option<Box<[u8]>>;

// smith-host-protocol's program.rs
Status::Usable => SourceAnswer::Check(ProgramCheck::Usable { email: program::email(code, stdout) }),
```

A new test breaks its rule once by hand. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents, one command at a time. Commit and push.

## Report

As the implementer's text says. Never print a real program's status.
