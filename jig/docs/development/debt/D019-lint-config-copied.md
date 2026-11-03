# D019: Test crates copy their lint configuration, and eight switch the replay bans off

- **Kind:** lints
- **Where:** `jig/tests/*/Cargo.toml`, `jig/testing/*/Cargo.toml`, `jig/crates/jig-client-shell-native/Cargo.toml`, `jig/crates/jig-client-shell-web/Cargo.toml`, `clippy.toml` copies
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T10 "lint exception volume", A0 F9, A11 F12)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (T10: the `[lints]` block in 40 manifests, `disallowed_*` allowed in 8, 23 `clippy.toml` copies)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (one lint regime per kind of crate; the replay bans hold in every world)
- **Size:** S
- **Status:** open
- **Paid when:** jig's test crates and shells take their lints from a shared table (`[lints] workspace = true` or one per kind), no manifest allows a `disallowed_*` lint, and each `clippy.toml` beyond the tree's and `tests/`'s states why it differs.
- **Measure:** `rg -l '^\[lints\.clippy\]' -g Cargo.toml jig | wc -l` (40 at base)

## What

- 40 manifests write their own `[lints.clippy]` block (26 in `tests/`, 12 in `testing/`, the two shells), in 16 different variants;
- 8 of them set `disallowed_types`, `disallowed_methods` and `disallowed_macros` to `"allow"` (`rg -l 'disallowed_[a-z]+ = "allow"' -g Cargo.toml jig | wc -l`), for example `tests/core/Cargo.toml` and `tests/conformance/Cargo.toml`, which also switches off the bans that keep a world replayable (`AGENTS.md`, 2: one thread, one loop, a seed replays);
- 23 `clippy.toml` files in jig, against the tree's and `tests/`'s two.

## Why it matters

A copied block drifts (16 variants already), and a world that allows the disallowed
lists can use wall-clock time or threads without the gate saying so.

## What pays it back

One shared lint table per kind of crate, each world's real need for an exception as a
scoped `#[expect]` with a reason. skein and temper have the same debt
(`skein/docs/development/debt/D036-lint-blocks-by-hand.md`,
`temper/docs/development/debt/D011-manual-map-expectations.md`): the profiles are proposed
as one plan across the trees when the first of the three is paid.
