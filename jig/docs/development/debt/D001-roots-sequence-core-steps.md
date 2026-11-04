# D001: Every root echoes the core's own steps back to it

- **Kind:** coupling
- **Where:** `examples/ops/domain/src/route.rs`, `testing/jig-test-domain/src/routes.rs`, `jig-domain-core/src/vocabulary.rs`; temper's `temper-engine-domain/src/route/now.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T4, A1 F3, A12 F3; order of work, item 9)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 9: "roots still send nine public hand-offs")
- **Checked:** 2026-11-04
- **Value:** high (each root shrinks; a new root cannot get the order wrong; unblocks D004)
- **Size:** M
- **Status:** open
- **Paid when:** a root handles only the core's `Now` values that need the root (a peer to call, a store to load, a payload to keep); none is answered by pushing a fixed core event straight back.
- **Measure:** `rg -c 'core::Now::' jig/examples/ops/domain/src/route.rs` (38 at `8065b8b98cf37a4b2b9fb99fb71149c6dd8634f6`)

## What

The core asks its root for work as `core::Now` values. The roots do real work
for the three hand-offs below, but some roots answer with fixed values. In
`ops`'s `route.rs` there are 38 lines with `core::Now::`:

- `Now::BriefCorePlanned`: ops and the test root build a plan from the core's
  limits. temper binds documents, home access and forge sections, and may answer
  not ready.
- `Now::CompleteBrief`: ops and the test root keep the brief payload. temper
  takes forge sections and counts their bytes.
- `Now::WorkspaceRequest`: ops answers with empty holds. The test root checks
  workspaces and credentials; temper prepares a workspace.

The fixed answers left are ops's and the test root's plans for
`BriefCorePlanned`, and ops's empty workspace answer. temper does real work in
all three hand-offs. Keeping a brief payload needs its root even in ops and
the test application.

## Why it matters

`jig/docs/design/README.md`, 6.4 gives ordering to the core, and 6.3 gives every root one
shape. Each copy of these arms is a place where a root can reorder or drop a step the core
relies on, and each new step ripples into every root.

## What pays it back

The core continues its own steps internally (crate-private `Work`, as it already does
for `StartBrief`), and keeps in `Now` only what needs the root. Then D004 can move what
remains of the wiring into jig.
