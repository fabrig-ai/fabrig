+++
id = "part-names"
part = "tidying"
kind = "feature"
needs = ["part-leftovers", "undecoded", "bundle-part"]
files = ["jig/crates/jig-protocol-store/src/host.rs", "jig/crates/jig-protocol-store/src/host/tests.rs", "jig/crates/jig-protocol-accounts/src/host.rs", "jig/crates/jig-protocol-accounts/src/host_tests.rs", "jig/testing/jig-test-store-files/src/disk.rs", "jig/testing/jig-test-service/src/**", "jig/docs/design/protocol/service.status.md", "temper/crates/temper-service/src/stop.rs"]
effort = "medium"
minutes = 35
+++

# B7, jig: one shape for the parts

## Goal

The request keeps B7 to the renames, the accessor and the helper; a shared trait across parts is out of scope, and so
are accounts' `output` panicking and the method docs. `temper-names` does temper's half. Behaviour stays unchanged.

Acceptance:

- `jig_protocol_store::host::Host` is `Drivers`, as `notes/contracts.md` says. `rg '\bHost\b'` over the readers below
  finds it nowhere; `Host` for anything else (skein's, the client shells', the HTTP field's, the generated codecs') is
  another thing and stays.
- `Hosted::mark_durable` exists, and no file of `jig/testing/jig-test-service/src` writes `.component.mark_durable()`.
- `Accounts::is_empty` is `idle`.
- `jig/testing/jig-test-service/src/worker.rs` has one helper for its four `SourceAnswered` pushes.
- `jig/docs/design/protocol/service.status.md` loses its **Not yet** line for 11, 11.5 and its date is today's.
- **Size cap:** net at most -5 lines.

## Inputs

- `temper/docs/plan/debt-10/notes/common.md`, `temper/docs/plan/debt-10/notes/contracts.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.5, on the trunk.

## Owns and may change

- Owns: `jig/crates/jig-protocol-store/src/host.rs` and its tests; `Accounts::is_empty` in
  `jig/crates/jig-protocol-accounts/src/host.rs`.
- May change, the name alone: `jig/testing/jig-test-store-files/src/disk.rs`, `jig/tests/store` and
  `jig/tests/store-codecs` where they name the type, `jig/testing/jig-test-service/src/{lib,stop}.rs`,
  `jig/testing/jig-test-service/src/tests/stop.rs`, and temper's `temper/crates/temper-service/src/stop.rs`, one line: this
  branch removes the name it reads.
- `Hosted::component` stays public: `service.md`, 11.5 says why, and other reads of it stay.
- `jig/crates/jig-protocol-store/src/{lib,secrets}.rs` and `jig/tests/store` call `Component::mark_durable`, which stays:
  they hold no `Hosted`.
- The helper, in `impl` of the worker: `fn answered(&mut self, settled: Option<(Token, Answer)>)`, with `Answer` whatever
  `self.sources.fire` returns. Its call sites may keep their `if let`, or pass the `Option` where that reads better. The
  `if let` at the third site guards a longer condition: leave what doesn't fit, and say so.

## Steps

1. `Host` to `Drivers`. `heavy cargo check -p jig-protocol-store -p jig-test-store-files --tests`.
2. `Hosted::mark_durable`, its sites in `jig-test-service`.
3. `Accounts::idle`, its readers.
4. The helper.
5. Update the status file. `heavy cargo nextest run -p jig-protocol-store -p jig-protocol-accounts -p jig-test-service`,
   `heavy cargo check -p temper-service --tests`. fmt, clippy, old forms, citations. Commit, push: one commit per step.

## Report

As the implementer's text says, with the net lines. Renames pin no new rule.
