# D001: Every root echoes the core's own steps back to it

- **Kind:** coupling
- **Where:** `examples/ops/domain/src/route.rs`, `testing/jig-test-domain/src/routes.rs`, `jig-domain-core/src/vocabulary.rs`; temper's `temper-engine-domain/src/route/now.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T4, A1 F3, A12 F3; order of work, item 9)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 9: "roots still send nine public hand-offs")
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** high (each root shrinks; a new root cannot get the order wrong; unblocks D004)
- **Size:** M
- **Status:** open
- **Paid when:** a root handles only the core's `Now` values that need the root (a peer to call, a store to load, a payload to keep); none is answered by pushing a fixed core event straight back.
- **Measure:** `rg -c 'core::Now::' jig/examples/ops/domain/src/route.rs` (39 at base)

## What

The core asks its root for work as `core::Now` values, and several of them come back
unchanged as a core event, the root adding nothing it owns. In `ops`'s `route.rs`
(1,446 lines, 39 `core::Now::` arms, 50 pushes of `Work::Core(core::Event::…)`):

- `Now::Activate` returns as `Event::Activate` with `domain.ready()`;
- `Now::PrepareAgent` returns as `Event::PreparedAgent`;
- `Now::BriefCorePlanned` returns as a `GatherEvent::Plan` built from the core's own limits;
- `Now::CompleteBrief` returns as `Event::BriefAssembled { ready: true }`;
- `Now::WorkspaceRequest` returns as `Event::WorkspacePrepared` with empty holds.

The test application's `routes.rs` (39 arms, 26 pushes) and temper's engine root write
the same arms. The restore protocol moved into the core (`Core::restore`, used by
`examples/ops/domain/src/restart.rs`); the preparation and brief steps did not.

## Why it matters

`jig/docs/design/README.md`, 6.4 gives ordering to the core, and 6.3 gives every root one
shape. Each copy of these arms is a place where a root can reorder or drop a step the core
relies on, and each new step ripples into every root.

## What pays it back

The core continues its own steps internally (crate-private `Work`, as it already does
for `StartBrief`), and keeps in `Now` only what needs the root. Then D004 can move what
remains of the wiring into jig.
