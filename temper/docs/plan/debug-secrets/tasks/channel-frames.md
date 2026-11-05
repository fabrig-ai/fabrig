+++
id = "channel-frames"
part = "channel-frames"
kind = "feature"
needs = []
files = ["skein/crates/skein-channel/src/frame.rs", "skein/crates/skein-channel/src/boundary.rs", "skein/crates/skein-channel/src/machine.rs", "skein/crates/skein-channel/src/tests.rs", "skein/docs/design/channel.status.md"]
effort = "medium"
minutes = 30
+++

# skein-channel: a frame prints its lengths

## Goal

An application's credential can travel in a channel body, so a frame's bytes must not reach `Debug`. `Frame`, `Event::Body` and the body the machine is still reading print lengths instead.

Acceptance:

- `Frame` prints `Frame { kind: 256, body_len: 1 }`; `Request::Send` shows it through its `frame` field.
- `Event::Body` prints `Body { kind: 256, body_len: 1 }`.
- `ReadState::Body` (`machine.rs`) prints `Body { header: .., body_len: N, filled: M }`, with every other variant of `ReadState` printing as it derived. So a `Machine` part way through a body prints none of it.
- The test below passes and fails with the old forms put back. `debug_of_records_without_a_credential_is_the_derived_form` no longer holds the `send` and `body` cases, which move to the new test with their new forms.
- `heavy cargo nextest run -p skein-channel -p skein-channel-world` passes.

## Inputs

- `temper/docs/plan/debug-secrets/notes/common.md`
- `skein/docs/design/lib.md`, 6 (A carrier prints lengths)
- `skein/docs/design/channel.md`, 3, and 6 for the body as it is read

## Owns and may change

- **Owns:** the files in `files`.
- **Status entry:** in `skein/docs/design/channel.status.md`, remove the `3: a frame's lengths ...` entry and keep the `Departs` entry.
- **Stays as it is:** a refusal's `text` and `FrameWriter` (`skein/docs/design/lib.status.md`, 6). The `Refuse` case of the derived-form test stays.
- **If a test elsewhere in `skein/tests/channel` compares a printed frame or body,** it changes with the form. Say so in your report.

## Steps

1. In `frame.rs`, write `Frame`'s `Debug`: `kind` and `body_len`, no bytes. In `boundary.rs`, `Event`'s `Body` arm prints `body_len`. In `machine.rs`, write `ReadState`'s `Debug`, listing every variant.
2. In `src/tests.rs`, beside the existing Debug tests, add `debug_of_a_frame_and_a_body_shows_lengths_never_bytes`, with a made-up canary as the body:
   - a `Frame` from `frame_writer`, a `Request::Send` holding one, and an `Event::Body`: exact forms;
   - a `Machine` given a frame's header and part of its body through the `Peer` harness, formatted whole: no canary, in either rendering.
3. Narrow tests, then `cargo fmt`, `old_forms.py` and clippy on `skein-channel`, as the implementer's text has them. Commit and push.

## Report

As the implementer's text has it, and name each rule you broke by hand with the failure you saw.
