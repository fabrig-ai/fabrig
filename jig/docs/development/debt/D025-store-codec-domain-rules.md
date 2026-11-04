# D025: The store codec applies domain rules

- **Kind:** coupling
- **Where:** `jig-protocol-store/src/codec/settings.rs`, `jig-protocol-store/src/boundary.rs`, `jig-protocol-store/src/commits.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T4 "protocol layers", A6 F2, F5, F11)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (T4: the store codec remaps three models and copies a domain literal; `Write::CopySettingsRevision`)
- **Checked:** 2026-11-03 at `a6a8155d35`
- **Value:** medium (the store stores; the domains decide)
- **Size:** M
- **Status:** open
- **Paid when:** the store codec translates rows without choosing values (no "model gone" sentinel of its own), and the store's writes are generic, with no write named after one domain's operation.

## What

- `jig-protocol-store/src/codec/settings.rs` (2,254 lines) remaps model catalogues and invents `u8::MAX` as "model gone" (lines 2134 to 2170);
- `Write::CopySettingsRevision(u32)` (`boundary.rs`, line 49; `commits.rs`, line 146) is a settings operation inside the store's generic write set;
- the review also found a domain message literal copied to size a bound.

## Why it matters

`jig/AGENTS.md`: jig's protocol layers translate; the rule belongs to the domain that
owns it. A rule in the codec is invisible to the domain's tests and changes with the
stored format.

## What pays it back

Domain functions for the remaps, called by the codec; a generic `Write::Copy`. Ties to
D007: generated codecs leave only these as hand-written code.
