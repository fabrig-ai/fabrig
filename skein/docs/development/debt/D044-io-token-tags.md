# D044: The terminal tags its Io's operation tokens by hand

- **Kind:** other
- **Where:** `skein-ui-text/src/terminal.rs`, `skein-lib/src/token.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T9 item 1) and `reviews/2026-10-31-skein-areas.md` (A6, F1)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** S
- **Status:** open
- **Paid when:** the terminal uses `Token::tagged` or a shared service routing mechanism to distinguish its `Io`'s completions from the application's machines, without a hand bit tag. Routing two `Io`s on one kernel belongs to the service (`skein/docs/design/io.md`, 5).
- **Measure:** `rg -c 'const TAG: u64 = 1 << 6' skein -g '*.rs' | awk -F: '{s+=$2} END{print s}'`: 1 at this commit

## What

`skein-ui-text/src/terminal.rs` sets bit 62 (`TAG`) to tell its `Io`'s
operations from the application's machines. This is a second `Io` on
one kernel, which `skein/docs/design/io.md`, 5, leaves to the service.
`FileIo`'s bit 59 distinguishes file operations from `Io`'s; it does
not distinguish two `Io`s. The terminal's hand tag does not use lib's
`Token::tagged`.

## Why it matters

The terminal and its owner must preserve the same routing convention,
or a completion can reach the wrong machine.

## What pays it back

Use `Token::tagged` for the terminal's service namespace, or share the
service's routing mechanism. smith's route-table debt cites this item
(`smith/docs/development/debt/D003-host-io-route-table.md`).
