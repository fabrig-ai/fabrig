# D013: Names the 10-31 review flagged remain

- **Kind:** naming
- **Where:** `smith-domain-run/src/boundary.rs`, `smith-local-service/src/host_io.rs`, `smith-agent-service/src/profile.rs`, `smith-domain-run/src/charter.rs`, `smith`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality.md`: theme "Naming"; `reviews/2026-10-31-quality-areas.md`: report 13, Q8)
- **Seen:** 2026-10-31 at `679eacb3c`, in the quality run's task reports (outside the repository): the run's `Failure` rename left, as temper names it
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (names say what the design says)
- **Size:** S
- **Status:** open
- **Paid when:** a reviewer finds the run's failure named `RunFailure` in the run as in the host, no `adapter` naming the local host's io hub or the LLM component's limits, and "checkout" only where git's checkout is meant.

## What

The run renamed most of what the review listed (`Phase` for LLM timing to
`CompletionPhase`, `OpenEvent` to `Heard` and `Received`, `Repo` to
`Directory`). Left:

- **`Failure`** names six enums: the run's (`smith-domain-run/src/boundary.rs:1029`),
  the session's, the agent service's, the local domain's and two in the
  local shell. The host already calls the run's `RunFailure`.
- **`adapter`**: the local host's io hub is `HostIo`, but its tests still
  bind it as `adapter` (151 lines of `host_io.rs`), and the LLM limits'
  field is `limits.llm.adapter` (`profile.rs:621`, `projection.rs:47`).
  The design uses "adapter" for the shell's `Agent` and the benchmarks'
  agents only.
- **"checkout"** appears in 55 production lines, mostly prose, such as the
  run's charter docs (`charter.rs:140`, `:149`, `:175`), where the design
  speaks of a workspace's directories.

## Why it matters

Two meanings for one word make reviewers look twice; the design's words are
the code's.

## What pays it back

Rename when a file is opened anyway. The run's `Failure` to `RunFailure`
changes a public name that temper uses, so it goes in one task with
temper's use sites.
