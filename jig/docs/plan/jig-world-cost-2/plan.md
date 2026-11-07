# jig-world-cost-2: the plan

jig's focused suite reads about 80 CPU s against its 30. This plan repairs what a new profile of `main` found, in the order of `skein/docs/foundation/testing-strategy.md`, 8.1. No budget or build setting changes, and no story loses what it checks. It won't reach 30 alone and isn't asked to: what's left is read after landing.

## Parts and tasks

One task per item of `inputs/request.md`. Each may land short. At most two run at once, by `needs`: items 1, 3 and 5 in one chain, 2 then 4 in the other.

| Item | Part and task | Needs | What it does |
|---|---|---|---|
| 2 | `naming-row` | | A durable row keeps its typed key; the referee prints it on a failure |

Items 1, 3, 4 and 5 are being cut: this file gains them with the next push.

## Critical path

As it stands, `naming-row`, at about 25 minutes.

## Overlaps

`limits-worlds/conformance-causes` has merged into its trunk with `jig/testing/jig-conformance/**` and `jig/testing/jig-core-kit/src/conformance.rs` in its files. `naming-row` touches `referee.rs`, `referee/deletions.rs` and `core/naming.rs` there. It's a small change, and git or fab's merge settles it; the task doesn't touch `jig-core-kit/src/conformance.rs`.
