# Known reds on the goal branch

Failing on g and owned by no live change of this goal; delivery checks pass them (tools/pre-pr/design.md, 5).
Since 2026-11-12: the delivery toolchain nightly-2026-10-21 raises these lints on lines no G6 change touches.

- lint jig-domain-fleet clippy::unneeded_wildcard_pattern jig/crates/jig-domain-fleet/src/attempt.rs
- lint jig-domain-fleet clippy::needless_bool jig/crates/jig-domain-fleet/src/sources.rs
- lint jig-domain-accounts clippy::needless_bool jig/crates/jig-domain-accounts/src/limits.rs
