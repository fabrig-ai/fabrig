# D037: One name stands for several things in skein's kits and crates

- **Kind:** naming
- **Where:** `skein/testing/skein-world/src/referee.rs`, `skein/testing/skein-world/src/domain/referee.rs`, `skein/testing/skein-conformance`, `skein/testing/skein-forge-conformance`, `skein/tests/forge/conformance`, `skein-forge-setup`, `skein-sim-world/src/program.rs`, `skein-fake-machine/src/fs.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T8) and `reviews/2026-10-31-skein-areas.md` (A0, A12)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 7)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low
- **Size:** S
- **Status:** open
- **Paid when:** a reviewer finds each of these names meaning one thing: skein-world's two referee frameworks no longer swap `Referee` and `Expectations`; "conformance" names one job (the kernel backends' suite) and the forge fact recorders another word; forge-setup spells `GitHub` one way; `Machine` is not both a trait and two structs a test imports together; the forge family's limit checks are named `valid`, as `skein/docs/foundation/programming-model.md`, 6.3, names them (amended first if a `Result` is to stay).

## What

- skein-world's root has `trait Referee` and `struct Expectations`
  (`referee.rs`); its `domain` module has `trait Expectations` and
  `struct Referee` (`domain/referee.rs`). Both are load-bearing (the
  review counted 85 impls of the root trait and 136 files on the domain
  framework), so the fix is a rename, not a merge.
- "conformance" names the kernel backends' suite
  (`skein/testing/skein-conformance`), the forge fact recorders
  (`skein/testing/skein-forge-conformance`,
  `skein/tests/forge/conformance`) and a world test.
- `skein_sim_world::Machine` is a trait; `skein_fake_machine::Machine`
  and `skein_channel::Machine` are structs, and tests alias them.
- The forge family checks limits with `check(limits) -> Result`
  (`skein-forge/src/forgejo/limits.rs`, `skein-forge/src/github/limits.rs`,
  `skein-forge-setup/src/forgejo/limits.rs`, `skein-forge-connection/src/limits.rs`),
  where `skein/docs/foundation/programming-model.md`, 6.3, names the
  check `valid(limits) -> bool`, as 23 files elsewhere in skein do.

## Why it matters

A reader in one module has the opposite meaning in mind from the next,
and a search finds the wrong thing.

## What pays it back

Rename the low-use pair of skein-world's frameworks, the forge fact
recorders, forge-setup's `Github*`, one of the `Machine`s and the
forge family's `check`s; each is a mechanical rename across the trees
that use it, the last after the designer settles 6.3's return type.
