# D007: The contract is written out by hand in four or five places

- **Kind:** duplication
- **Where:** `jig-protocol-engine-clients/src/translate`, `jig-client-protocol-link/src/translate`, `jig-client-domain/src/contract`, `jig-protocol-store/src/codec`, `testing/jig-client-scripted-engine/src/codec`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Duplicated code": the piece-decoding loop)
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T3, A0 F5, A6 F1, A7 F3, F14, A8 F2, A12 F9; order of work, item 20)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 20, out of the run's scope)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** high (the review estimates 30,000 to 45,000 hand-written lines)
- **Size:** L
- **Status:** open
- **Paid when:** the domain-side mirror types of a record family and both translations are generated from its schema, and the hand-written translation directories hold only what the schema cannot say.
- **Measure:** `rg -o 'Parts \{' jig/crates/jig-protocol-engine-clients/src/translate | wc -l` (675 at base)

## What

One record family is described in its schema, then again by hand:

- the engine's translation, `jig-protocol-engine-clients/src/translate`: 10,475 lines, 675 `…Parts {` literals;
- the link's translation, `jig-client-protocol-link/src/translate`: 11,010 lines;
- the client domain's mirror, `jig-client-domain/src/contract`: 6,477 lines;
- the store codec, `jig-protocol-store/src/codec`: 15,316 lines, mostly structural;
- the scripted engine's codec values, and `ops`'s `infrastructure_values.rs` and `observability_values.rs`;
- the link's `jig-client-protocol-link/src/limits.rs` lists each bound class's members by hand, once per class; bound classes in the schema would generate these lists;
- `temper-settings-form/src/{lib,fields,kinds}.rs` holds its own limits map with 36 `.min(…CEILINGS` calls, and `jig-protocol-engine-clients/src/translate` narrows with 22 more at encoding.

## Why it matters

A field added to a schema is added by hand four or five times, and the copies can
disagree; most of jig's growth since 10-18 is here.

## What pays it back

A skein job first (`AGENTS.md`: a missing generic mechanism goes into skein before its
jig consumer): skein-codegen "mirror" declarations that emit the owned domain-side types,
both translations, typed piece helpers and the limits mapping from bound classes in the
schema.
