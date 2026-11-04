# D005: The civil calendar is written three times

- **Kind:** duplication
- **Where:** `skein-forge/src/forgejo/time.rs`, `skein-http/src/server/response.rs`, `skein-ui/src/date.rs`, `skein/tests/forge/machine/src/github.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T1, the calendar row) and `reviews/2026-10-31-skein-areas.md` (A7, H7)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** S
- **Status:** open
- **Paid when:** one civil-date conversion of a `Wall` serves skein-ui, skein-http's `Date` header and skein-forge's RFC 3339 and HTTP-date reads; no other crate converts days to a civil date.

## What

- skein-ui's phrases and views now share `skein-ui/src/date.rs`
  (`civil`, landed 10-31); that copy is paid.
- `skein-forge/src/forgejo/time.rs` (166 lines) keeps its own leap-year
  and month tables, a year loop, RFC 3339 `read`/`write` and
  `http_date`.
- `skein-http/src/server/response.rs` converts days to a date again
  (Hinnant's algorithm) for the server's `Date:` header, with its own
  weekday and month tables.
- The forge machine world writes an IMF date of its own
  (`skein/tests/forge/machine/src/github.rs`, `imf`).
- GitHub's dialect reads times through `forgejo::time` (see D010).

## Why it matters

Three copies of date arithmetic, each tested on its own, and each a
place for an off-by-one at a year or month edge. The Forgejo copy also
ties GitHub's dialect to Forgejo's modules.

## What pays it back

A civil date on `Wall` in skein-lib (lib may hold it; skein-forge
depends only on lib and json), IMF-fixdate read and write in skein-http,
and RFC 3339 at skein-forge's root, all on the one conversion. Then the
world's `imf` uses skein-http's writer. It helps D010.
