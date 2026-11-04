# D010: hearth's test crates and shell copy one lint block by hand

- **Kind:** lints
- **Where:** `hearth/tests/*/Cargo.toml`, `hearth/testing/*/Cargo.toml`, `hearth/crates/hearth/Cargo.toml`, the `clippy.toml` copies
- **Seen:** 2026-11-04 at `66968a30ef`, in jig's D019 and skein's D036, the same debt; met in hearth's manifests
- **Checked:** 2026-11-04 at `66968a30ef`
- **Value:** medium (one lint regime per kind of crate)
- **Size:** S
- **Status:** open (joins the cross-tree lint plan)
- **Paid when:** hearth's crates take their lints from the shared profiles the other trees take, and each `clippy.toml` beyond the tree's and `tests/`'s states why it differs.
- **Measure:** `git grep -lE '^\[lints\.(rust|clippy)\]' -- 'hearth/*Cargo.toml' | wc -l`: 14

## What

Nine worlds, four kits and the binary each write their own `[lints.rust]`
and `[lints.clippy]`, 15 lines or more. Only `hearth-fake-machine` of the
test crates takes `[lints] workspace = true`.

Eight `clippy.toml` files sit in hearth. `hearth-recordings`'s is
`tests/clippy.toml` again, and `hearth-test-mirror`'s differs from it by
its thread rule. Fake git's and fake machine's are one file twice, and fake
mmdebstrap's differs from them by a line.

## Why it matters

A copied block drifts, and a change of policy is 14 edits.

## What pays it back

Take the shared profile when the cross-tree plan comes (jig's D019, skein's
D036). hearth has no step of its own to do first.
