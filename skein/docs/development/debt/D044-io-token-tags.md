# D044: Hand bit tags work around io's and FileIo's colliding operation tokens

- **Kind:** other
- **Where:** `skein/testing/skein-world/src/files.rs`, `skein-ui-text/src/terminal.rs`, `skein-io/src/file_layer.rs`, `skein-lib/src/token.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T9 item 1) and `reviews/2026-10-31-skein-areas.md` (A6, F1)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** S
- **Status:** open
- **Paid when:** a completion's machine is known from io's design (a lane chosen at construction, or another rule `skein/docs/design/io.md`, 5 states), and no skein code tags tokens by hand to tell `Io`'s completions from `FileIo`'s.
- **Measure:** `rg -c 'const TAG: u64 = 1 << 6' skein -g '*.rs' | awk -F: '{s+=$2} END{print s}'`: 2 at base

## What

`Io` names operations by slot and generation, and `FileIo` counts from
1 (`file_layer.rs`, `next_op`), so their tokens can be equal on one
kernel. `skein/docs/design/io.md`, 5, has a service hand each
completion back to the machine whose operation it was, but gives no
way to tell. Callers tag by hand: skein-world's files set bit 63
(`files.rs`, `TAG`), skein-ui-text's terminal bit 62 (`terminal.rs`,
`TAG`), and smith keeps a route table. lib's `Token::tagged`, added on
10-31, is not used by either.

## Why it matters

Each workaround is a convention two machines must not break, and a new
service that runs both without one is exposed to a misrouted
completion.

## What pays it back

The designer settles the rule in io.md first (the review proposed a
lane per machine); then io implements it and the hand tags go. Until
then, the two tags could at least use `Token::tagged`. smith's route
table waits on this item (`smith/docs/development/debt/D003-host-io-route-table.md`).
