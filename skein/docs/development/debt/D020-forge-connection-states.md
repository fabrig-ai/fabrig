# D020: forge-connection's connections keep their state in flags beside a thin tag

- **Kind:** complexity
- **Where:** `skein-forge-connection/src/connection.rs`, `skein-forge-connection/src/hooks/connection.rs`, `skein-forge-connection/src/installations.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T4, the forge-connection row) and `reviews/2026-10-31-skein-areas.md` (A3, F6)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 2)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** a forge connection's and a webhook connection's phases hold their data, and each sets its deadline in one place after a transition (`skein/docs/design/forge-connection.md`, 8.1 and 8.4).
- **Measure:** `rg -c 'deadline =' skein/crates/skein-forge-connection/src -g '!**/tests/**' | awk -F: '{s+=$2} END{print s}'`: 18 at base

## What

- `Connection` (`connection.rs`) has a `Copy` `Phase` tag with
  `draining`, `closing`, `written` and other fields beside it; the phase
  is assigned at a dozen sites.
- `Hooked` (`hooks/connection.rs`) keeps its state in flags and
  `Option`s where 8.4 has four states.
- Non-test code sets a deadline at 18 sites; 8.1 says one.

## Why it matters

State that the design draws as a table is spread over fields, so a
reader reconstructs it from flags, and a transition that forgets one
field leaves a state the design does not have.

## What pays it back

Data-carrying phases and one `follow` that sets demand and deadline.
Do it with D013, and with D016 if the shared transport comes first.
