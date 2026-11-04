# D017: `unreachable!` re-narrows values the types could carry

- **Kind:** complexity
- **Where:** `jig-domain-host/src/hosted.rs`, `jig-domain-core/src/effects.rs`, `jig-protocol-accounts/src/component.rs`, `jig-domain-core/src/settings/application.rs`, `jig-domain-notes/src/read.rs`, `jig-domain-core/src/settings.rs`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Signals in the step code": `unreachable!` arms)
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T7 "the same smell elsewhere", A1 F12, A5 F10, A7 F6)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 9: about 372 outside tests; T7: notes `ReadPending`, engine-clients' `Waiting`)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (a pairing the types encode cannot be broken by a later change)
- **Size:** M
- **Status:** open
- **Paid when:** each `unreachable!` left in production guards a state machine's invariant; none re-matches a value whose variant an earlier match already fixed.
- **Measure:** `rg -o 'unreachable!' jig/crates jig/examples -g '!**/tests/**' -g '!**/tests.rs' -g '!**/tests_*.rs' -g '!**/generated/**' | wc -l` (374 at base)

## What

374 `unreachable!` in production code (228 on 10-18; about 372 at the quality run). The
largest clusters: `jig-domain-host/src/hosted.rs` 20, `jig-domain-fleet/src/attempt.rs` 20,
`jig-domain-core/src/effects.rs` 15, `jig-protocol-accounts/src/component.rs` 11,
`jig-domain-core/src/settings/application.rs` 11, `jig-domain-notes/src/read.rs` 10,
`jig-domain-core/src/settings.rs` 9. Many are honest invariants (fleet's `attempt.rs` is
a state table). Others come from a value stored in a map of several families and matched
again when it comes back (notes' pending reads; payload maps in the roots), or from an
`if let` on an enum ending in `unreachable!`.

## Why it matters

The invariant lives in the programmer's head, not in a type; a refactor that breaks the
pairing compiles and aborts at run time.

## What pays it back

Maps keyed by family, so a lookup returns the narrow type; types that carry the pairing
(a read and the document answering it). Follows from D002 and D015.
