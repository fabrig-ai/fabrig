# D016: The socket, TLS and HTTP transport is written four times

- **Kind:** primitives
- **Where:** `skein-forge-connection/src/connection.rs`, `skein-forge-setup/src/host/connection.rs`, `skein-oauth-accounts/src/exchange.rs`, `skein-llm-connection/src/call.rs`
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 1a and decision D9)
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (areas A3 and A4, the duplicated transport stack)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** high
- **Size:** M
- **Status:** open
- **Paid when:** one concrete transport (a socket token, an optional TLS client, its queues, the handshake, upload, close and abort) is used by the four components; each sends TLS `close_notify` on a graceful close.

## What

- Each of the four files holds its own socket token, optional
  `skein_tls::client::Client`, cipher and plain queues, handshake,
  routing, upload and close/abort (1,188, 766, 625 and 518 lines).
- They have drifted: only oauth-accounts and llm-connection send TLS
  `Close` (`skein-llm-connection/src/call.rs` calls
  `tls::Request::Close`); neither forge component does.
- The `Abort` both forge components lacked has landed with the
  component kit; that part is paid.

## Why it matters

A fix to the transport (a close rule, a fault) is made four times, and
the copies already disagree on how a TLS connection ends.

## What pays it back

A short design note first (the component kit's later run,
`skein/docs/design/draft/component-kit.md`, 5), then one `Link` over io
and tls with `connected`, `up`, `down`, `close(abort)` and `has_work`,
and the four components on it. It overlaps D013 and D020 in
forge-connection.
