# D019: The channel's two components copy their kind text

- **Kind:** duplication
- **Where:** `smith-protocol-channel/src/component.rs`, `smith-host-protocol/src/component.rs`, `smith-channel`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 13, Q6, the two `component.rs`)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (one spelling of a frame kind's name)
- **Size:** S
- **Status:** open
- **Paid when:** the agent's and the host's channel components name frame kinds through one function in `smith-channel`, with no copy left in either `component.rs`.

## What

`kind_text`, which names a frame kind in refusals, is the same 16 lines
in `smith-protocol-channel/src/component.rs:682` and
`smith-host-protocol/src/component.rs:415`. The review saw more shared
lines between the two files; the quality run moved framing and kinds into
`smith-channel`, and this copy is what remains.

## Why it matters

The two halves speak one wire; a new kind must be named in both by hand.

## What pays it back

Move `kind_text` into `smith-channel`, beside the kinds both halves
already take from it. One task, no wire change.
