# D006: The link's values have two import paths, and a grammar is copied

- **Kind:** duplication
- **Where:** `jig-domain-fleet/src/lib.rs`, `jig-domain-host/src/lib.rs`, `jig-domain-accounts/src/lib.rs`, `jig-domain-people/src/lib.rs`, `jig-client-domain/src/contract/deployment.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T5, A5 F1, A0 F10, A4 F15; order of work, item 10)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 10)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (one name per value; one grammar to change)
- **Size:** S
- **Status:** open
- **Paid when:** `SourceName`, `Worker`, `Form`, `Grant` and the program-name grammar are reached only through `jig-domain-link`, and no other crate defines a program-name check.
- **Measure:** `rg -n 'pub use jig_domain_link' jig/crates | wc -l` (6 at base)

## What

The leaf crate `jig-domain-link` exists, so the three copies of the vocabulary are gone.
What is left:

- fleet, host and accounts re-export it (6 `pub use jig_domain_link` lines), and callers use both paths: 30 uses through the re-exports, 29 direct;
- the program-name grammar is written twice more, byte for byte: `program_name` in `jig-domain-people/src/lib.rs` and `valid_program_name` in `jig-client-domain/src/contract/deployment.rs`.

## Why it matters

Two paths to one type hide where it lives; a grammar written three times changes in one
place and not the others.

## What pays it back

Drop the re-exports and import from `jig-domain-link`; people and the client domain call
its grammar (or the client domain keeps a checked copy, tested against it, if it must not
depend on the link).
