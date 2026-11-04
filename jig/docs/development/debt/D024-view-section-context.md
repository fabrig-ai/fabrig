# D024: The view passes a section's context through a field set around calls

- **Kind:** coupling
- **Where:** `jig-client-view/src/state/ui.rs`, `jig-client-view/src/state/page/shown.rs`, `jig-client-view/src/state/page/routing.rs`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Duplicated code": the section context, a hidden parameter)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (each call says what it reads)
- **Size:** S
- **Status:** open
- **Paid when:** `section_context` is an argument built by one function, not a field of `Ui` reset to `u32::MAX` after use.
- **Measure:** `rg -o 'section_context' jig/crates/jig-client-view/src | wc -l` (16 at base)

## What

`Ui` has `section_context: u32` (`jig-client-view/src/state/ui.rs`, line 74), set before a call and reset to
`u32::MAX` after it (line 554); `shown.rs` uses it 12 times, and `routing.rs` reads it
(line 37). The bits are set by three functions, `context_subject`, `context_section` and
`context_read` (`shown.rs`, lines 327, 336 and 365).

## Why it matters

A hidden parameter makes a call's result depend on what ran before it; a missed reset
leaks one section's context into another.

## What pays it back

One function that builds the context, passed as an argument to the calls that read it.
