# D046: skein-ui offers no `Limits` constructor and hides its node accounting

- **Kind:** primitives
- **Where:** `skein-ui/src/limits.rs`, `skein-ui/src/tree.rs`
- **Seen:** 2026-11-03 at `41d63da8c3`, in jig's tech-debt list (its `skein_ui::Limits` and node accounting entries)
- **Seen:** 2026-11-03 at `41d63da8c3`, in jig's backfill (`jig/docs/development/debt/D022-ui-limits-literals.md` and `jig/docs/development/debt/D023-view-tests-text-bytes.md`, the callers' side, which wait on this item)
- **Checked:** 2026-11-03 at `a6a8155d35`
- **Value:** medium
- **Size:** S
- **Status:** open
- **Paid when:** skein-ui gives a constructor for a tree without scenes, so callers set only the fields they mean, and exposes the per-node byte accounting that callers now re-implement.
- **Measure:** `rg -U -c '(skein_ui::)?Limits \{[^{}]*?scene_marks: 0' -g '*.rs' skein jig temper | awk -F: '{s+=$2} END{print s}'`: 34 at base

## What

- `skein_ui::Limits` (`limits.rs`) has a `valid` check and no
  constructor; 34 literals across the trees write it out whole, each
  with `scene_marks: 0` and `scene_bytes: 0`: 25 in jig, 7 in temper,
  2 in skein.
- jig's client view re-implements skein-ui's private text accounting
  in its tests (`text_bytes`), and drifts when skein-ui's does.

## Why it matters

The fix is skein's even though the copies are jig's: skein-ui is where
the fields and the accounting live. Each new limit touches every
literal.

## What pays it back

A `Limits` constructor without scenes and a public accounting function
in skein-ui. The callers then move onto them, as their own trees' items:
jig's literals are `jig/docs/development/debt/D022-ui-limits-literals.md`
and its `text_bytes` is `jig/docs/development/debt/D023-view-tests-text-bytes.md`,
both waiting on this item; temper's 7 literals and skein's 2 follow in
the same sweep.
