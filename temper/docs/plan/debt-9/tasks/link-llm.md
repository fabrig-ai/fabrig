+++
id = "link-llm"
part = "transport"
kind = "feature"
needs = ["link-crate"]
files = ["skein/crates/skein-llm-connection/**", "skein/tests/llm-connection/**", "skein/docs/design/llm-connection.status.md"]
effort = "high"
minutes = 80
+++

# skein: llm-connection on `Link`, and its `close_notify` delivered

## Goal

`skein-llm-connection/src/call.rs`' `Connection` stands on `Link`. By reading, `close_notify` is lost at base: `route` pops
TLS's events before its ciphertext, so TLS's `Closed` sends the socket's `Close` ahead of the `Send` made in the same step, and
io drops a stream request once closing. D016 says this crate sends it. It asks TLS to, and the bytes don't reach the wire.

Acceptance:

- **Reproduce first:** the pin fails on your base, with `closed` false. If it passes there, say so in your report: the reading
  was wrong, and the task goes on.
- `rg -n 'cipher_down|tls_events|tls_closed|socket_close_sent' skein/crates/skein-llm-connection/src -g '!*test*'` finds
  nothing.
- `Abort` on a connecting socket stays, as `skein-llm-connection/src/tests.rs` pins it.
- `call.rs`'s non-test lines fall.
- `llm-connection.status.md`'s 5 entry under **Not yet** is gone.

## Inputs

- `temper/docs/plan/debt-9/notes/common.md`, `temper/docs/plan/debt-9/notes/link.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/link.md`, 3; `skein/docs/design/llm-connection.md`, 2 and 5.

## Owns and may change

- Owns: `skein-llm-connection/src/**`, `skein/tests/llm-connection/**`.
- `skein-llm-connection/Cargo.toml`: one line, `skein-link.workspace = true`, and `Cargo.lock`.
- `skein/docs/design/llm-connection.status.md`: that sentence goes, and the date line.
- `Control` stays: it's the owner's close policy, and `aborting` reads it. Its physical half, `socket_close_sent` and
  `tls_closed`, is the link's.

## Steps

1. The pin, first: a story in `skein/tests/llm-connection`: a call over TLS ends, the idle connection closes gracefully, and the
   server's `closed` is true. See it fail on the base.
2. `Connection` per `notes/link.md`.
3. `close_transport` is `link.close(.., false, io)`; `close_socket` where it means abort, and in `Phase::Connecting`, is
   `link.close(.., true, io)`. `Phase::Closing` still follows.
4. `route`'s sources: llm events, plain, then `link.pump`, then `llm::resume`. `Moved::Above(Event::Ready)` is `Head` and
   `Start`; `Stream` is `plain_up`, not when aborting; `Failed` is `llm::abort`. The `Send` deadline in Head stays on `plain_down`.
5. `LCc`'s `next_deadline` and `fire`, per the note: the `Table` has no closing deadline, and stays so.
6. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-llm-connection --all-targets -- -D
   warnings`; `heavy cargo nextest run -p skein-llm-connection -p skein-llm-connection-world`; `heavy cargo check --tests` on
   `smith-protocol-llm` and `temper-service`, its readers across trees.
7. The citations check. Commit and push.

## Report

`notes/link.md`'s break, the common note's items. The reproduction's failure on your base, and the net lines of `call.rs`.
