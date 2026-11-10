# D039: forge-client's module headers cite the wrong design sections

- **Kind:** naming
- **Where:** `skein-forge-client/src/limits.rs`, `skein-forge-client/src/keep.rs`, `skein-forge-client/src/identity.rs`, `skein-forge-client/src/facts.rs`, `skein-forge-client/src/domain.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein-areas.md` (A4, F11)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** S
- **Status:** open
- **Paid when:** each forge-client module's header cites the section of `skein/docs/design/forge/client.md` that designs it.

## What

The citation gate checks that a cited section exists, not that it is
the right one. forge-client's headers cite sections that exist but
design something else:

- `limits.rs` cites 5 (fresh reads); its limits are 2.2;
- `keep.rs` cites 5; keeping up is 4, its working set 3;
- `identity.rs` cites 6 (the request budget); keys are 7.1;
- `facts.rs` cites 5 and `domain.rs` 5 to 6; the owner and the domain
  are 2.

## Why it matters

A reader sent to the wrong section reads the wrong contract.

## What pays it back

Correct the five headers. The review found about ten similar citations
in forge-connection; check them in the same task.
