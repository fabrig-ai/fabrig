# D009: Limits are copied between layers and mapped by hand

- **Kind:** duplication
- **Where:** `jig-client-protocol-link/src/limits.rs`, `jig-domain-core/src/limits.rs`, `jig-protocol-store/src/lib.rs`, `jig-protocol-store/src/codec.rs`, `jig-client`, `jig-client-domain`, `jig-protocol-config`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T5 "limits overlap", T3, A1 F7, A2 F3, A6 F3, A7 F14, A8 F3, F4, A9 F13, A12 F18; order of work, item 13)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 13; bug 3 "settled short of the review's fix")
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (one place per bound; no copies that must agree by hand)
- **Size:** M
- **Status:** open
- **Paid when:** each bound is declared once by its owner and derived elsewhere; the core's `Limits` holds no field whose only check is equality with another; the link's wire limits come from one derivation, not a hand-listed mapping.
- **Measure:** `rg -o '\.min\(w::CEILINGS' jig/crates/jig-client-protocol-link/src/limits.rs | wc -l` (15 at base)

## What

- The core's composite `Limits` keeps copies of one value: `valid` (`jig-domain-core/src/limits.rs`) refuses unless `tasks.revisions == settings.revisions`, `tasks.versions == settings.versions` and `fleet.versions == settings.versions`, so the duplicated fields stay and every profile must set them alike.
- The client side has six `Limits` types (`jig-client/src/limits.rs`, `jig-client/src/thread.rs`, `jig-client-protocol-http/src/lib.rs`, `jig-client-shell-native/src/connections.rs`, `jig-client-domain/src/limits.rs`, `jig-client-protocol-link/src/limits.rs`).
- The link maps the client's knobs onto `w::Limits` by hand, field by field, 15 of them through `.min(w::CEILINGS.…)` (`limits.rs`, from line 109).
- The store keeps `Limits`, a derived `codec::View` and `Levels`.

## Why it matters

A bound written in several places changes in one; fixture limits that must agree by hand
turned 616 to 640 temper tests red in the quality run.

## What pays it back

One `Limits` per concept, derived values computed once by their owner; the link's mapping
generated with D007 (bound classes in the schema). Whether a configuration above a
ceiling is refused or capped is a design question for jig's designer when this item is
planned; the backfill recommends refusing, like every other bound.
