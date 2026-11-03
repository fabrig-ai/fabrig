# D004: `Env` is rebuilt by hand where `env.with(limits)` says it

- **Kind:** primitives
- **Where:** `temper-engine-domain`, `temper-service`, `temper-client-tools`, `temper-forge-setup-shell`, `temper-worker-domain`, and four more crates and worlds
- **Seen:** 2026-10-31 at `4a6683c72`, in the skein review of that day (`skein/docs/development/debt/reviews/2026-10-31-skein.md`), theme T1's table ("`Env { now, wall, limits: … }` rewrap: 72 temper")
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (shorter steps; a child step's clocks always the parent's)
- **Size:** S
- **Status:** open
- **Paid when:** temper hands a child step its limits with `env.with(limits)`; the measure is 0.
- **Measure:** `rg -U -c --type rust 'Env \{\s*now: (\w+\.)+now,\s*wall: (\w+\.)+wall' temper | awk -F: '{s+=$2} END {print s}'`: 78

## What

`env.with(limits)` hands a child step the same clocks with its own
limits (`skein/docs/design/lib.md`, 9; added by `c6ef164146`). jig took
it up and has 3 rewraps left. temper calls it nowhere and rebuilds the
`Env` field by field 78 times in 39 files, as
`Env { now: env.now, wall: env.wall, limits: ... }`: 35 in
`temper-engine-domain`, 12 in `temper-service`, 8 in
`temper-client-tools`, 7 in `temper-forge-setup-shell`, 8 in temper's
worlds.

## Why it matters

The rewrap is three fields where one call says the intent; a rewrap that
takes one clock from elsewhere would pass review unseen.

## What pays it back

A mechanical sweep with D003, crate by crate. skein's own rewraps are
skein's `skein/docs/development/debt/D003-env-rewraps.md`; `Env::with`
exists, so this item does not wait on it.
