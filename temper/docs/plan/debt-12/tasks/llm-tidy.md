+++
id = "llm-tidy"
part = "owners"
kind = "feature"
needs = ["link-surface"]
files = ["skein/crates/skein-llm-connection/src/call.rs", "skein/crates/skein-llm-connection/src/component.rs", "skein/tests/llm-connection/src/world.rs", "skein/tests/llm-connection/tests/suite/failures.rs"]
effort = "high"
minutes = 30
+++

# The LLM connection reads the link's face, and its close order is pinned

## Goal

`link_env` is seven lines and a fifth copy inline, "is the close due" is a `match`, two calls of `close_socket`
do nothing, and `an_idle_tls_connection_sends_close_notify_before_socket_close` can't fail on the order it's named
for, because the world hands a `Stream` after `Close` to the wire. About 12 production lines go.

Acceptance:

- `rg -n 'skein_link::Limits' skein/crates/skein-llm-connection/src` finds one line, in `link_env`.
- `component.rs` calls neither `next_deadline` for `close_due`, nor `close_socket`.
- `an_idle_tls_connection_sends_close_notify_before_socket_close` keeps its name and fails with `Link::pump`'s two
  halves swapped, events before cipher.
- `heavy cargo nextest run -p skein-llm-connection -p skein-llm-connection-world` passes with no assertion changed.
- `heavy cargo clippy -p skein-llm-connection -p skein-llm-connection-world --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/debt-12/notes/common.md`
- `temper/docs/plan/debt-12/notes/contracts.md`, 2 and 3
- `temper/docs/plan/debt-12/inputs/link-review.md`, findings 5, 6 and 10's first sentence
- `skein/docs/design/link.md`, 3.2 and 3.3

## Owns and may change

- **Owns:** the files in this spec's `files`.
- **Options:**
  - If `close_socket` has no reader left outside `call.rs`, it loses its `pub(crate)`.
  - If the world's new drop turns another test red, stop and ask: that test saw a `Stream` after `Close`, which
    io drops (`skein/docs/design/link.md`, 3.2).

## Steps

1. `link_env`, as the contracts' section 2, and `pub(crate)`. The test at `component.rs`'s
   `IoRequest::Close { entity: Token::new(100) }` builds the same `Env` inline: it calls `link_env`.
2. `let close_due = connection.link.due(env.now);`, and the `match` goes.
3. In `IoEvent::Connecting`, the `if connection.phase == Phase::Closing { connection.close_socket(env, io); }`
   goes, and in `IoEvent::Connected` the `else`. By reading: before `Connected` a connection reaches
   `Phase::Closing` only by `close_socket`, an abort (`cancel`, `route`'s `abort_pending`, `llm::Event::Close` in
   `Phase::Connecting`), or by `closed`. `close_transport` starts in later phases. So the link is already Aborted
   or Closed, `connecting` already sent what it held, and a second `close(true)` changes nothing. Check that
   against the code once more before you delete: a graceful close there would make these lines an abort.
4. The world's `IoRequest::Stream` drops what comes after the peer's `closing`, as io does and as
   `skein/tests/link/src/lib.rs` says. Swap `Link::pump`'s halves by hand, see the test go red because
   `wire.server.closed` is false, and restore.
5. Fmt, clippy, tests. `python3 -I -B scripts/old_forms.py` in `skein/`. Commit and push.

## Report

The report of your implementer text, and: what you checked for step 3; what you broke and what failed; net
lines, production and tests apart.
