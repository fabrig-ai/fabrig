# goal-call-faults: the plan

Three faults a goal's coordinator met in its host calls on a real
deployment (`inputs/request.md`), each fixed with its failing row first
in an existing table, and the design saying so (item 4) where each is
written. Design written by this cut: `temper/docs/design/domain/agent.md`,
4.3.1 (a batch's members) and its status.

## Parts and tasks

| Part | Task | Request item | Crates (files) | Needs | Minutes |
|---|---|---|---|---|---|
| `change-batch` | `batch-options` | 1, 4 | temper-engine-smith (`calls.rs`, `Problem`), temper-service (`tools.rs`'s rendering) | | 40 |

Items 2 (`subscribe` busy to itself) and 3 (`read_task`'s 128-byte
pages) are being surveyed; their parts follow in a later push.

## Decisions

- **Item 1, a batch's options: refused, not inherited.** The design
  already makes each change of `changes` an object of its own members
  and refuses `words` beside `changes`. Inheriting would need a rule per
  member (an outer `name`, `after`, `merge`, `repairs` or `branch` means
  nothing shared across a batch) and which value wins; a refusal at
  decoding makes no change, names the member and says where it goes
  (`host call member belongs in each change, not beside changes:
  landing`), so nothing given is dropped unheard. The first such member
  in the declaration's order is named. A new `Problem::Beside` carries
  it, since `Range` renders "out of range", which reads as a bad value
  and could lead a run to drop the member. The declaration's schema
  text is unchanged, so no host-part figure moves. An unknown member
  stays accepted in both forms, as today: outside the request.

## Overlaps

- None so far: `temper-engine-smith` and `temper-service/src/tools.rs`
  are on no other draft's branch; `tui-box-front/serve-sign-in`
  (merged) changed other files of `temper-service`.

## Critical path

`batch-options` (40) alone so far.
