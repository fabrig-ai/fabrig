# D036: A client-world helper picks a page by its name as a string

- **Kind:** tests
- **Where:** `tests/client/tests/suite/text_pages.rs`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Readability of the worlds' tests": a helper dispatches on strings)
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (A10 F11)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (the compiler checks the page names)
- **Size:** S
- **Status:** open
- **Paid when:** `special` takes an enum of its pages and has no `unreachable!` arm.

## What

`fn special(page: &str)` (`text_pages.rs`, line 75) compares `page` with string literals
to choose a page world, and ends in `_ => unreachable!("named page")` (line 118).

## Why it matters

A misspelt page name compiles and fails at run time.

## What pays it back

An enum of the special pages, matched exhaustively.
