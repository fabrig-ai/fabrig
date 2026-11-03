# D008: HTTP client and server parse message heads separately

- **Kind:** duplication
- **Where:** `skein-http/src/client/head.rs`, `skein-http/src/server/head.rs`, `skein-llm`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T1, the HTTP head row) and `reviews/2026-10-31-skein-areas.md` (A7)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** S
- **Status:** open
- **Paid when:** one module of skein-http parses header fields and reads `Content-Length`, `Transfer-Encoding` and persistence for both sides; its predicates and head measure are public, and skein-llm uses them instead of its own.

## What

`client/head.rs` and `server/head.rs` each define `field`,
`content_length` and `persistent`, and each walks the transfer coding.
The predicates are private, so skein-llm re-implements token and value
checks and the head length for itself (the review's A9 notes).

## Why it matters

Both sides of one RFC 9112 reading, kept in step by hand. A fix to a
field rule (obsolete folding, a duplicate length) lands twice or not at
all.

## What pays it back

A shared `header` module in skein-http with the field parser and the
three readings, used by both machines, and the predicates exported.
