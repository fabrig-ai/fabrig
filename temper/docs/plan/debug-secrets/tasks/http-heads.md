+++
id = "http-heads"
part = "http-heads"
kind = "feature"
needs = []
files = ["skein/crates/skein-http/src/header.rs", "skein/crates/skein-http/src/server.rs", "skein/crates/skein-http/src/server/head.rs", "skein/crates/skein-http/src/client.rs", "skein/crates/skein-http/src/tests.rs", "skein/crates/skein-http/src/tests/debug.rs", "skein/docs/design/http.status.md"]
effort = "medium"
minutes = 30
+++

# skein-http: a head prints its lengths

## Goal

An `Authorization`, `Cookie` or `Set-Cookie` value reaches records that derive `Debug` over it, and a request's target can hold a code or a state. `skein-http`'s head records print lengths instead, so everything that holds them prints without those bytes.

Acceptance:

- `Header` prints `Header { name: [..], value_len: N }`, the form `jig-client-protocol-http`'s wrappers already print.
- A server's `Call` prints `Call { method: .., target_len: N, version: .., headers: [..], body: .. }`.
- A server's private `RequestLine` (`server/head.rs`) prints `RequestLine { method: .., target_len: N, version: .. }`.
- A client's private `Sending` (`client.rs`) prints `Sending { head_len: Some(N), upload: .. }`, or `None` once the head is sent.
- The tests below pass, and each fails with the derive put back.
- `heavy cargo nextest run -p skein-http -p skein-http-world` passes.

## Inputs

- `temper/docs/plan/debug-secrets/notes/common.md`
- `skein/docs/design/lib.md`, 6 (A carrier prints lengths)
- `skein/docs/design/http.md`, 2

## Owns and may change

- **Owns:** the files in `files`. `src/tests/debug.rs` is new, with its `mod debug;` line in `src/tests.rs`.
- **Status entry:** in `skein/docs/design/http.status.md`, remove the `2: a head's lengths ...` entry and keep the `6:` entry.
- **Stays as it is:** a client's `Call` and `Response` and a server's `Response` keep their derives: their fields print through `Header`, and a client's target is the side above's own. `Intake` and lib's stream records also stay (`skein/docs/design/lib.status.md`, 6).
- **If the `Intake` still holds the canary when you format a machine,** deliver whole lines so that it holds none. If it can't be emptied at that point, take the machine's state one line later, and say so in your report. Don't change `Intake`.

## Steps

1. In `header.rs`, `server.rs` and `server/head.rs`, write the three `Debug` impls. In `client.rs`, write `Sending`'s: its `head_len` is the head's length as an `Option<usize>`.
2. In `src/tests/debug.rs`, with one made-up canary per test:
   - `debug_of_a_header_shows_its_name_and_its_values_length`: an `Authorization` field; the exact form.
   - `debug_of_a_servers_call_shows_lengths_never_its_target_or_values`: a `Call` literal whose target holds one canary and whose `Cookie` field holds another; the exact form.
   - `debug_of_a_server_reading_a_head_holds_no_target`: a `Server` given a request line with a canary in its target, and no blank line yet, formatted whole.
   - `debug_of_a_client_holding_its_head_holds_no_value`: a `Client` given a `Call` with a canary in an `Authorization` field and driven up to its demand for room, as `written` in `src/tests/request.rs` begins, formatted whole; it shows `head_len: Some(`.
3. Narrow tests, then `cargo fmt`, `old_forms.py` and clippy on `skein-http`, as the implementer's text has them. Commit and push.

## Report

As the implementer's text has it, and name each rule you broke by hand with the failure you saw.
