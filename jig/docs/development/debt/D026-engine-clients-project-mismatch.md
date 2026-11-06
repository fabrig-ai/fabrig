# D026: The clients' protocol layer answers a people rule itself

- **Kind:** coupling
- **Where:** `jig-protocol-engine-clients/src/translate/operations.rs`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T4 "protocol layers", A7 F2)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (T4: engine-clients answers people's `ProjectMismatch` and decodes compound asks three times)
- **Checked:** 2026-11-06
- **Value:** low (one place admits a compound ask)
- **Size:** M
- **Status:** kept (the check is the wire's own shape, `jig/docs/design/client/domain/contract.md`, 3.4; moving it takes a stored enum's new variant and changes which refusal a malformed ask gets)
- **Paid when:** kept; if people's `Operation` comes to hold a project, the clients' component passes a compound ask to people whole, people refuses an operation of another project, and the component decodes a compound ask once.

## What

`operations.rs` (around line 225) decodes each operation of a `w::Ask::Compound`, compares
its project with the ask's, and answers `w::Malformed::ProjectMismatch` itself, encoding
the refusal there. People's own admission holds the same rule (`jig/docs/design/domain/people.md`: a compound request's operations are all in its project).

## Why it matters

Admission split across layers can disagree, and the people world never sees the case.

Measured for the plan debt-28, which left it open as bigger than its old Size S:

- People can't see the case today. `jig-domain-people`'s `Operation` holds no project ("whose project is named once by its compound", `compound.rs`), and only the wire's operations name one. People's rule in `jig/docs/design/domain/people.md` is that each operation's task is in the request's project; this check compares two numbers on the wire, which is why its refusal is called malformed.
- For people to refuse it, `Operation` takes a project in each of its 6 variants, in about 22 files with temper's worlds among them. The store's codec drops it on encode and fills it on decode, since the stored operation holds none. `people::Malformed` gains a variant, whose stored enum in `people-v14.schema` has none, so the codec's arm is `unreachable!`, or the schema takes a version.
- Moving the check moves its order. Today it runs before anything else, even before a later piece fails to decode. Decoding once by moving it into `translate/operations.rs`'s `compound` changes that case too, since `asks::count` scans all the pieces first.

## What pays it back

Translate the compound ask whole and let people refuse it; decode its operations once. The stored schema's new version and the moved refusal are what that costs, and the plan debt-29 kept the check rather than pay them: its request stops at a stored format, and at a test's changed outcome.
