# D001: Every root echoes the core's own steps back to it

- **Kind:** coupling
- **Where:** `examples/ops/domain/src/route.rs`, `testing/jig-test-domain/src/routes.rs`, `jig-domain-core/src/vocabulary.rs`; temper's `temper-engine-domain/src/route/now.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T4, A1 F3, A12 F3; order of work, item 9)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 9: "roots still send nine public hand-offs")
- **Checked:** 2026-11-04 at `e2fd13de21`
- **Value:** high (each root shrinks; a new root cannot get the order wrong; unblocks D004)
- **Size:** M
- **Status:** open
- **Paid when:** a `Now` value stays when any root does real work with it; none is answered by every root with a fixed core event.
- **Measure:** `rg -c 'core::Now::' jig/examples/ops/domain/src/route.rs` (39 at `e2fd13de21`)

## What

The core asks its root for work as `core::Now` values. `Now::PrepareAgent` is gone;
the core continues agent preparation internally. The four values below stay because
temper's engine root does real work with them:

- `Now::Activate` checks its byte budget and read room;
- `Now::BriefCorePlanned` plans the application's brief;
- `Now::CompleteBrief` assembles the brief;
- `Now::WorkspaceRequest` prepares the workspace.

`ops`'s `route.rs` has 39 `core::Now::` lines. Its and the test root's fixed answers
belong to D004; they do not make these values unnecessary to every root.

## Why it matters

`jig/docs/design/README.md`, 6.4 gives ordering to the core, and 6.3 gives every root one
shape. Each copy of these arms is a place where a root can reorder or drop a step the core
relies on, and each new step ripples into every root.

## What pays it back

The core continues its own steps internally (crate-private `Work`, as it already does
for `StartBrief`), and keeps in `Now` only what needs the root. Then D004 can move what
remains of the wiring into jig.
