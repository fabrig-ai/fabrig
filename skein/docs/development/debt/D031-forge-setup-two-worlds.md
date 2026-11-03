# D031: The forge setup world is written twice, once without the host

- **Kind:** tests
- **Where:** `skein/tests/forge/setup/src/world.rs`, `skein/tests/forge/setup/src/host.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T5, worlds that bypass their design) and `reviews/2026-10-31-skein-areas.md` (A4)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 4)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** the setup world's stories all run on `HostWorld`, through the host component and HTTP bytes, and the plans-only `World` is gone.

## What

`skein/tests/forge/setup` has two worlds: `World` (`world.rs`, 362
lines) plays the reconcile plans directly against the fake's service,
and `HostWorld` (`host.rs`, 705 lines) plays the host component.
`skein/docs/design/forge-setup.md`, 7, says the setup world plays the
host component through HTTP bytes; the Forgejo stories on `World` never
exercise the host's connection.

## Why it matters

Two harnesses at one tier, and the stories that matter most (Forgejo's
convergence) skip the code that ships between the plans and the wire.

## What pays it back

Move the `World` stories onto `HostWorld`, then delete `World`. The
review estimated 766 lines out.
