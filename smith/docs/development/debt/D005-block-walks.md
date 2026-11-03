# D005: Every walk of a message destructures `Block` again

- **Kind:** duplication
- **Where:** `smith-domain-session/src/llm.rs` (`Block`), `smith-domain-session/src/session.rs`, `smith-domain-session/src/session/`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality-areas.md`: report 03, finding 6)
- **Seen:** 2026-10-31 at `679eacb3c`, in the quality run's closing report (outside the repository): left, as it changes a public block shape across siblings
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (shorter walks; a new block kind breaks one match, not every walk)
- **Size:** S
- **Status:** open
- **Paid when:** a tool call is its own type inside `Block`, reached through one exhaustive accessor beside the enum, and the session's walks use it instead of destructuring `Block::ToolCall` each time.
- **Measure:** `grep -rn 'Block::ToolCall {' smith/crates/smith-domain-session/src --include=*.rs | grep -vE 'tests?\.rs|_tests' | wc -l` is 81

## What

The session's production code destructures `Block::ToolCall { .. }` 81
times, and lists the other variants by hand in each walk that wants only
calls or only text. Text extraction ends in `unreachable!` where a walk
already knows the variant.

## Why it matters

Each walk restates the enum. A new block kind, or a new field on a call,
is an edit at every site rather than at one accessor.

## What pays it back

`Block::ToolCall(ToolCall)`, with one `tool_call()` and one `text()`
accessor matched exhaustively beside the enum, and `let Some(..) else`
at the walks (`skein/docs/foundation/programming-model.md`, 10.2 allows it
on `Option`). The variant's shape changes for the session's translators in
the protocol crates, so their tests move too.
