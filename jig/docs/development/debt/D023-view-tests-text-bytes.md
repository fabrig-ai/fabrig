# D023: The view's tests re-implement skein-ui's node accounting

- **Kind:** tests
- **Where:** `jig-client-view/src/tests.rs`, `jig-client-view/src/tests`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Duplicated code": skein-ui's node accounting, again)
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (its tech-debt cross-check: `text_bytes` users 5 to 19)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (the view's size tests cannot drift from skein-ui)
- **Size:** S
- **Status:** open
- **Paid when:** jig's tests read a node's text bytes from skein-ui, and `text_bytes` is gone.
- **Measure:** `rg -l 'text_bytes' jig/crates/jig-client-view/src | wc -l` (41 at base)

## What

`text_bytes` (`jig-client-view/src/tests.rs`, line 13) adds up a node's name, value,
external and text runs, copying skein-ui's private accounting. 41 files of the view use it
(5 on 10-18, 19 at the review).

## Why it matters

When skein-ui changes what it counts, these tests keep passing on the old rule, or fail
for no fault of the view.

## What pays it back

skein-ui's `Node::text_bytes` gives the bytes the builder charges a node
(`skein/docs/design/ui.md`, 15). The tests read it and the helper goes.
