# D019: jig's test crates and shells copy one lint block, which no shared table holds

- **Kind:** lints
- **Where:** `jig/tests/*/Cargo.toml`, `jig/testing/*/Cargo.toml`, `jig/crates/jig-client-shell-native/Cargo.toml`, `jig/crates/jig-client-shell-web/Cargo.toml`, `clippy.toml` copies
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T10 "lint exception volume", A0 F9, A11 F12)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (T10: the `[lints]` block in 40 manifests, `disallowed_*` allowed in 8, 23 `clippy.toml` copies)
- **Checked:** 2026-11-03 at `c895688b8864347b17b961ec5ef8425cdd496839`
- **Value:** medium (one lint regime per kind of crate; the replay bans hold in every world)
- **Size:** S
- **Status:** open
- **Paid when:** jig's test crates and shells take their lints from a shared table (`[lints] workspace = true` or one per kind), no manifest allows a `disallowed_*` lint, and each `clippy.toml` beyond the tree's and `tests/`'s states why it differs.
- **Measure:** `rg -l '^\[lints\.clippy\]' -g Cargo.toml jig | wc -l` (40 at base)

## What

The 40 manifests write one `[lints]` block, the same in each, by hand (26 in
`tests/`, 12 in `testing/`, the two shells). Cargo shares among members only the
root's `[workspace.lints]`, all or nothing, and that table is the step code's.
Nothing holds the 40 copies to one form.

## Why it matters

A copied block drifts.

## What pays it back

One shared lint table per kind of crate, each world's real need for an exception as a
scoped `#[expect]` with a reason. skein and temper have the same debt
(`skein/docs/development/debt/D036-lint-blocks-by-hand.md`,
`temper/docs/development/debt/D011-manual-map-expectations.md`): the profiles are proposed
as one plan across the trees when the first of the three is paid.

The way to hold the copies to one form is the user's, asked as plan debt-1's
question `d019-shared-table`.
