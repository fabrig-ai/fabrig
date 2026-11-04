# D011: The fake GitHub stands on the fake Forgejo, which is named as the neutral fake

- **Kind:** coupling
- **Where:** `skein-fake-forge-github`, `skein-fake-forge-protocol`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (themes T2 and T8, the fake forge rows) and `reviews/2026-10-31-skein-areas.md` (A5)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 3)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** the git route the two fakes share is typed by neutral types, not by Forgejo's `Limits`; the GitHub fake names no `forgejo::` item beyond what the neutral root (D010) holds; the crate that is the Forgejo fake says so in its name; the two fake servers share one connection core.
- **Measure:** `rg -c 'forgejo::' skein/crates/skein-fake-forge-github/src | awk -F: '{s+=$2} END{print s}'`: 26 at base

## What

- `skein-fake-forge-github` names `forgejo::` 26 times.
- The git route is neutral in purpose but typed by Forgejo's protocol
  `Limits`, so GitHub's fake (and temper) fill in Forgejo limits they do
  not use.
- `skein-fake-forge-protocol` is the Forgejo fake, but its name reads as
  the fakes' shared protocol layer.
- The two fake servers (`skein-fake-forge-protocol/src/server.rs`, 926
  lines, and `skein-fake-forge-github/src/server.rs`, 792 lines) each
  keep a data-less `State` and their own connection machinery; the
  review put the shared part at about 200 lines.

## Why it matters

A change to the Forgejo fake can break the GitHub fake, and a reader
looking for the neutral layer opens the Forgejo one.

## What pays it back

After D010: a neutral git route, the GitHub fake on neutral types, one
connection core for both servers, and a rename of the Forgejo fake
(a public crate name temper uses, so the rename is the user's call).
