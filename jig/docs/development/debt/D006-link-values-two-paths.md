# D006: The link's values have two import paths

- **Kind:** duplication
- **Where:** `jig-domain-host/src/lib.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T5, A5 F1, A0 F10, A4 F15; order of work, item 10)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 10)
- **Checked:** 2026-11-06
- **Value:** low (one name per value)
- **Size:** M
- **Status:** open
- **Paid when:** `SourceName`, `Worker`, `Form`, `Grant` and the link's other values are reached only through `jig-domain-link`.
- **Measure:** `rg -n 'pub use jig_domain_link' jig/crates | wc -l` (2 remaining)

## What

The vocabulary's copies and the program-name grammar's copies are gone; callers ask
`skein_lib::Name` or `SourceName`. What is left is two `pub use jig_domain_link` lines in the hub,
and their readers. Accounts and fleet no longer re-export the link's values, and their
readers name `jig_domain_link`. The cut counted 536 uses through `jig_domain_host`,
most through an alias such as `use jig_domain_host as host`. Temper's readers name
`jig_domain_link`.

## Why it matters

Two paths to one type hide where it lives.

## What pays it back

Drop the re-exports crate by crate, importing from `jig-domain-link` with a dependency
line in about 25 manifests. A private `use` at each crate's root keeps `crate::SourceName`
inside it. This adds a line per file more than it removes.
