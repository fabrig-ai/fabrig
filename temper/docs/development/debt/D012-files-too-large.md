# D012: The forge's domain and route files, and nine more, are too large to read

- **Kind:** complexity
- **Where:** `temper-engine-domain-forge/src/domain.rs`, `temper-engine-domain/src/route/forge_route.rs`, `temper-client-view/src/words.rs`, `temper/crates/temper-client-gallery-tool/src/worlds.rs`, `temper-engine-domain-forge/src/landing.rs`, `temper-engine-domain-forge/src/boundary.rs`, `temper-engine-domain-forge/src/tiers.rs`, `temper-settings-form/src/wire.rs`, `temper-worker-domain-checkout/src/hold.rs`, `temper-box/src/host.rs`, `temper-service/src/setup/lifecycle.rs`
- **Seen:** 2026-11-03 at `41d63da8c3`, while checking D002 and D011 (new here)
- **Checked:** 2026-11-03 at `a6a8155d35`
- **Value:** medium (the forge's core logic readable a module at a time; smaller diffs, fewer merge meetings)
- **Size:** L
- **Status:** open
- **Paid when:** no hand-written production file in temper is over 1,500 lines (fabrig's one threshold for "too large to read"; the measure is 0), except one a comment says is kept whole on purpose; the forge domain and its root route split along the design's parts, not cut at a line count.
- **Measure:** `find temper -name '*.rs' -not -path '*/generated/*' -not -path '*/tests/*' -not -path 'temper/testing/*' -not -name 'tests.rs' -not -name 'tests_*.rs' -not -name '*_tests.rs' -exec wc -l {} + | awk '$2!="total" && $1>1500' | wc -l`: 11

## What

Eleven hand-written production files are over 1,500 lines. Two stand out:

- `temper-engine-domain-forge/src/domain.rs`, 6,303 lines and 191
  functions, "top-level forge routing and commit fence";
- `temper-engine-domain/src/route/forge_route.rs`, 5,154 lines and 128
  functions, the root's translation for the forge connector, beside a
  `temper-engine-domain/src/route/forge_route/` directory that already
  holds eight of its parts and its tests.

Then `temper-client-view/src/words.rs` (3,137, checked English data),
the gallery's `temper/crates/temper-client-gallery-tool/src/worlds.rs` (2,592), the forge
domain's `landing.rs` (2,417), `boundary.rs` (2,106) and `tiers.rs`
(1,894), `temper-settings-form/src/wire.rs` (1,692),
`temper-worker-domain-checkout/src/hold.rs` (1,659),
`temper-box/src/host.rs` (1,618) and
`temper-service/src/setup/lifecycle.rs` (1,599).

Test files are not counted, but five are over 3,000 lines:
`temper/tests/real-loop/src/case/oracle.rs` (11,865 lines, the reference
case's checkpoints), `temper/tests/engine/domain/tests/suite/root_routes.rs`,
`temper-engine-domain/src/route/forge_route/tests.rs`,
`temper/tests/engine/domain/tests/suite/forge_routes.rs` and
`temper-client-view/src/tests/task.rs`.

The two forge files carry 17 of temper's 162
`#[expect(clippy::too_many_lines, ...)]`.

## Why it matters

A reader, or an agent with a bounded window, cannot hold either forge
file whole; every change to the forge's routing meets every other in the
same file, which is where merges collide.

## What pays it back

Split by the design's parts, a plan per file: the forge domain by
`temper/docs/design/domain/forge/README.md`'s procedures (change, landing,
tiers, issues already have crates of their own), the root route into
the `forge_route/` modules it has begun, the other files by their own
concerns, and the oracle by the reference case's steps. `words.rs` may
be kept whole as data, with a comment saying so (the backfill recommends
it; the plan settles it). Each split moves code without changing it, so its
tests are the existing ones.
