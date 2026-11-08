# ignored-engine: the plan

Twelve ignored temper tests come back into the focused suite that
selects them (`inputs/request.md`; `skein/docs/foundation/testing-strategy.md`,
8: an ignored test is a finding its owner runs until it's fixed). One
task per group of the request, effort medium. Each task un-ignores its
tests first, then fixes each failure by its cause: the product against
its design, or the world, joint or fake that lacks a capability
(`notes/common.md`).

## Parts and tasks

| Part | Task | Tests | Where | Needs | Minutes |
|---|---|---|---|---|---|
| `restore` | `restore-stories` | `settings/kinds.rs`'s three restore stories; `settings_tools.rs`'s each-kind story | engine-domain and client worlds; product only where a story shows it wrong | | 60 |

The other three groups (the engine-domain world's changes to existing
records, the forge routes, the GitHub joint) follow in later batches.

## Critical path

`restore-stories` alone so far; every task of this plan is independent
of the others.

## Overlaps with other drafts

- `tui-box-front/world-front` changes `temper/tests/client/src/`'s
  `fixtures.rs`, `component.rs`, `tui.rs` and `front.rs`;
  `restore-stories` owns none of them and adds lines only if it must.
