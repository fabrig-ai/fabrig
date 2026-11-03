# D002: Three owners still hold lifecycles in flags and `Option`s

- **Kind:** complexity
- **Where:** `smith-local-service/src/host_io.rs` (`HostIo`), `smith-agent-service/src/service.rs` (`Stages`), `smith-local-domain/src/domain.rs` (`Domain`)
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality.md`: theme "Several lifecycles folded into one struct"; `reviews/2026-10-31-quality-areas.md`: report 13, Q4)
- **Seen:** 2026-10-31 at `679eacb3c`, in the quality run's task reports (outside the repository): local delivery's save coordination left on `Domain`; `HostIo` and `Stages` not converted
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (each lifecycle readable and testable alone; the review's first defect was a transition left half done in such an owner)
- **Size:** M
- **Status:** open
- **Paid when:** `HostIo`, `Stages` and `Domain` hold each lifecycle as its own state enum with its own transitions and `follow`, and no field beside them is meaningful in some states only (`skein/docs/foundation/programming-model.md`, 5.4).

## What

The quality run converted most of the owners the review listed (`Backend`
54 to 34 fields, the store, the host `Agent`, the channel components, the
session's `Conversation`). Three remain:

- **`HostIo`** (`host_io.rs:56`), the local host's io hub, formerly
  `ProcessAdapter`: 34 fields, 10 of them `bool` or `Option`, among them
  `reading`, `failed`, `closing`, `close_root_pending`, `process`,
  `terminal_input` and `signal_operation`. The process tree's life and its
  close are spread over them.
- **`Stages`** (`service.rs:169`), the agent service's composition: 55
  fields, 3 `bool` and 13 `Option`. The run's end is held across
  `failure`, `final_failure`, `final_refusal`, `outcome`, `delivery`,
  `cancel_pending` and `root_close_pending`.
- **`Domain`** (`domain.rs:37`), the local host's domain: 27 fields, 10
  `Option`. Delivery moved into `delivery/machine.rs`, but its save
  coordination is still `saving`, `delivery_owner` and `reconciling` on
  `Domain`.

## Why it matters

A lifecycle spread over flags lets states exist that the design has no
name for, and each handler must check the flags by hand. The model now
says so in so many words, and the converted owners show the form.

## What pays it back

One lifecycle at a time, each after its tests stop writing private state:
`Domain`'s delivery save first (beside `delivery/machine.rs`), then
`Stages`' run end, then `HostIo`'s close. `HostIo`'s io routing is D003;
the two can go together.
