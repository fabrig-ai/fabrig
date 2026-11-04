# D020: Nothing stops an inner `#![allow]` from coming back

- **Kind:** lints
- **Where:** `tests/boundary/src`, the workspace's `Cargo.toml` lints
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T10 "23 inner `#![allow]` attributes"; order of work, item 3)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (keeps a paid debt paid)
- **Size:** S
- **Status:** open
- **Paid when:** a check that the gate runs fails on an inner `#![allow` anywhere in jig.

## What

The 23 inner `#![allow]` attributes are gone (`rg -l '#!\[allow' jig -g '*.rs'` finds
none). The gap that let them in stays: the workspace's `allow_attributes = "deny"` checks
only outer attributes, and neither `tests/boundary` nor `jig/scripts/old-forms.txt` counts
inner ones. temper still has some, so the hole is real.

## Why it matters

`AGENTS.md`, 2 says a lint exception is a scoped `#[expect]`, never `#[allow]`; without a
check the rule holds only by review.

## What pays it back

A `tests/boundary` test (or an old form at 0 in `jig/scripts/old-forms.txt`) that refuses
`#![allow`.
