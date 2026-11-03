# D018: The local domain composes person-facing text

- **Kind:** coupling
- **Where:** `smith-local-domain/src/person.rs`, `smith-local-domain/src/domain.rs`, `smith-local-service/src/service.rs`, `smith-local-protocol/src/terminal.rs`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 10, finding 9)
- **Seen:** 2026-10-31 at `679eacb3c`, in the quality run's task reports (outside the repository): "remaining domain text rendering" left
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (one route for words; the domain decides, the terminal renders)
- **Size:** S
- **Status:** open
- **Paid when:** `smith-local-domain` emits typed notices only and holds no person-facing literal text; all words are rendered in the local protocol's `terminal.rs`.

## What

Failures reach the person as typed `Notice`s rendered in
`smith-local-protocol/src/terminal.rs`. But the domain also writes words
itself: `turn_text` (`person.rs:206`) builds lines such as
`b"Tool finished\n"` and `b"Tool failed\n"` (`person.rs:247` to `:275`),
and `Domain::tool_verdict` and `result_text` (`domain.rs:96`, `:102`)
expose that rendering. The service then patches a notice's words with
`effort_notice` (`service.rs:318`).

## Why it matters

Two routes for the person's words: a change of wording must find both, and
the domain's step code formats bytes, which the model keeps out of step
code. `smith/docs/design/domain/host.md`, 8 asks for each kind's words,
matched exhaustively, in one place.

## What pays it back

Typed notice variants for tool verdicts, turn lines and the effort notice;
their words move into `terminal.rs`, and the domain's text helpers go.
About a hundred lines move. Check what reads the shell's fixed texts
before changing any byte.
