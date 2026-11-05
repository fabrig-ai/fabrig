# D031: Worlds are used as kits by other packages

- **Kind:** tests
- **Where:** `tests/boundary/src/tests/kits.rs`, `tests/core`, `tests/client`, `tests/engine-clients`, `tests/client-protocol`, `tests/conformance`, `jig/testing`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T8 "worlds are kits", A0 F4, A11 F11; order of work, item 19)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 19: 57 pending package to `-world` pairs, 26 of them `-world` to `-world`)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (worlds change without breaking other worlds; kits say what they offer)
- **Size:** M
- **Status:** open
- **Paid when:** the boundary's `PENDING` list of world edges is empty: no package depends on a `-world` package except through skein's and smith's own world kits.
- **Measure:** `awk '/^const PENDING/,/^\];/' jig/tests/boundary/src/tests/kits.rs | grep -c '^    ("'` (57 at base)

## What

The quality run added the rule and an allowlist of the edges it found
(`kits.rs`, `PENDING`, "removing an edge also removes its entry"). The list is unchanged:
57 edges, 26 of them from one jig world to another, 8 onto `jig-core-world` (whose
driver and peers the client, protocol, conformance and engine-clients worlds reuse), the
rest onto skein's and smith's worlds. `jig/testing` and `jig/tests` no longer predict
which is a kit.

## Why it matters

`jig/docs/design/domain/testing.md`, 7 puts fakes and kits apart from worlds. A world
used as a kit cannot change its fixtures without breaking its dependents, and the
dependency cycles slow builds.

## What pays it back

Move each reused driver, peer and fixture into a `jig/testing` kit, one edge at a time,
deleting its `PENDING` line. The edges onto `skein-world` may be the kit's by design; if
so, the rule should exempt them, so the list counts only jig's own. skein's world
boundaries follow `skein/docs/design/testing.md`, 6.
