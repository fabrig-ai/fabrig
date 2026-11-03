# D032: Machine-world harness pieces are copied between the protocol worlds

- **Kind:** tests
- **Where:** `skein/tests/http/src/client_world.rs`, `skein/tests/http/src/server_world.rs`, `skein/tests/tls/src/world.rs`, `skein/tests/json`, `skein/tests/protocol`, `skein/tests/io`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T5, copies between worlds) and `reviews/2026-10-31-skein-areas.md` (A7, A12)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 4)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** a protocol-world kit in `skein/testing/` holds the stream-below simulation (`meets`, `delivers`, `fault`, `draw`), the `Seen` record and the transcript loader, and the http, json, tls and protocol worlds use it; tests/io runs its worlds on `skein_world` and io's shared stage.
- **Measure:** `rg -c 'struct Seen\b' skein/tests skein/testing | awk -F: '{s+=$2} END{print s}'`: 16 at base

## What

- `fn meets` and `fn delivers` are defined in the HTTP client world,
  the HTTP server world and the TLS world; `fn draw` 15 times and
  `fn fault` 8 times across `skein/tests` and `skein/testing`.
- `struct Seen` is defined 16 times, in the protocol, json, http and tls
  worlds' suites among others.
- The review found tests/io running three world loops, none of them on
  the `skein_io::stage` services use.

## Why it matters

Each copy of the below-simulator is a place where a fault is drawn
differently, so the worlds do not hold their machines to one standard.

## What pays it back

One protocol-world kit in `skein/testing/`, then each world on it, one
per task; tests/io onto `skein_world` and the shared stage.
