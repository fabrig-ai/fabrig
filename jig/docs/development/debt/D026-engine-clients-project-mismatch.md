# D026: The clients' protocol layer answers a people rule itself

- **Kind:** coupling
- **Where:** `jig-protocol-engine-clients/src/translate/operations.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T4 "protocol layers", A7 F2)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (T4: engine-clients answers people's `ProjectMismatch` and decodes compound asks three times)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (one place admits a compound ask)
- **Size:** S
- **Status:** open
- **Paid when:** the clients' component passes a compound ask to people whole, and people refuses an operation of another project; the component decodes a compound ask once.

## What

`operations.rs` (around line 225) decodes each operation of a `w::Ask::Compound`, compares
its project with the ask's, and answers `w::Malformed::ProjectMismatch` itself, encoding
the refusal there. People's own admission holds the same rule (`jig/docs/design/domain/people.md`: a compound request's operations are all in its project).

## Why it matters

Admission split across layers can disagree, and the people world never sees the case.

## What pays it back

Translate the compound ask whole and let people refuse it; decode its operations once.
