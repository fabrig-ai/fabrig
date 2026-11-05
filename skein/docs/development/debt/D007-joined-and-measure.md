# D007: Byte joining and measuring helpers are copied per file

- **Kind:** duplication
- **Where:** `skein-http/src/client/request.rs`, `skein-http/src/server/response.rs`, `skein-http/src/sse/writer.rs`, `skein-fake-forge-forgejo`, `skein-fake-forge-github`, `skein-forge-connection`, `skein-oauth-issuer`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T1, the concat row) and `reviews/2026-10-31-skein-areas.md` (A0, F11)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** S
- **Status:** open
- **Paid when:** skein-lib offers one measured concatenation and one length measure, and no skein file defines its own `joined` or `Measure`.
- **Measure:** `rg -c 'fn joined\b|struct Measure\b' skein -g '*.rs' | awk -F: '{s+=$2} END{print s}'`: 13 at base

## What

- Ten files define a private `fn joined` that measures parts and copies
  them into one box (for example `skein-fake-forge-forgejo/src/setup.rs`,
  `skein-fake-forge-github/src/service.rs`,
  `skein-forge/src/forgejo/server/files.rs`); five of the ten are in
  tests.
- skein-http defines `struct Measure(Option<u32>)` three times, in the
  request writer, the response writer and the SSE writer.

## Why it matters

Small, but each copy chooses its own overflow answer (a panic, `None`,
an error), and the next file writes a fifth.

## What pays it back

`skein_lib::bytes::concat` is the one concatenation: delete the copies onto it, each overflow answer
with them. Then a `Measure` in lib (or one in skein-http for the three writers), and delete those.
