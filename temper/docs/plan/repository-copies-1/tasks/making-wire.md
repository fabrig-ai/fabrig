+++
id = "making-wire"
part = "making-records"
kind = "interface"
needs = ["copy-surface"]
files = ["temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema", "temper/crates/temper-client-protocol-wire/src/generated/v1.rs", "temper/crates/temper-client-protocol-wire/golden/**", "temper/crates/temper-client-protocol-wire/src/lib.rs", "temper/crates/temper-client-protocol-wire/src/source.rs", "temper/crates/temper-client-protocol-wire/src/tests.rs", "temper/crates/temper-client-protocol-wire/src/tests/source.rs"]
effort = "medium"
minutes = 50
+++

# `MakeRepository` on the wire, and the address's check

## Goal

The client wire carries the request, its outcome and the rows (contracts T3), and one check of an
address serves both the client and the engine (contracts T4). Nothing sends them yet. `needs` holds
this behind `copy-surface` only to stay off `to-begin-store`'s lines while they land.

Acceptance:

- The schema's lines of T3; generated code and goldens from the writer; `CLIENT_VERSION` stays 11.
- `ForgeRow.makings` round-trips at its bound, and `MAKINGS` is set in `limits()`.
- `source.rs`, new tests in `src/tests/source.rs`: `parse` takes each form of the draft's 5.4 and gives
  the same `Source`; refuses `http://`, `ssh://`, `git@`, a path of more segments, an empty segment,
  `.` and `..`, a name past 100 bytes, a host that is no DNS name; takes `.invalid` hosts and a port;
  `address` is `https://HOST/OWNER/NAME`, parsed back to the same `Source`.
- `heavy cargo nextest run -p temper-client-protocol-wire -p temper-codecs-world -p temper-client-domain
  -p temper-client-tools` pass; `heavy cargo check --tests -p temper-service -p temper-client-view -p
  temper-client-shell-web -p temper-client-shell-terminal -p temper-client-shell-cli -p temper-client-world
  -p temper-client-gallery` passes.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, T3 and T4.
- `temper/docs/design/client/README.md`, 4.6, "`MakeRepository`" and "the subject `forges`".
- `temper/docs/design/draft/repository-copies.md`, 5.4.

## Owns and may change

- The files of the header. `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test
  suite -E 'test(drift::)'` writes generated code and goldens; never by hand.
- **The compiler's fallout, each `makings: Box::new([])` or an arm alone, in any temper file:** the
  `ForgeRowParts` literals (`temper-service/src/family.rs`, `temper-client-domain/src/tests/**`,
  `temper-client-view/src/tests/connect_forge.rs`, `temper-client-tools/src/tests/**`,
  `temper-client-shell-*`'s fixtures, `temper/tests/client/src/front.rs`,
  `temper/tests/codecs/tests/suite/client_wire.rs`, `temper/tools/gallery/src/worlds.rs` and
  `worlds/github.rs`), and the `w::Request` and `w::Outcome` matches (`temper-client-domain/src/forge/
  {family,forges,playground,github}.rs`, `temper-client-tools/src/forges.rs`,
  `temper-client-tools/src/forges/{lifecycle,repositories,connection}.rs`,
  `temper-client-tools/src/jig/render/outcome.rs`, `temper-service/src/family.rs`). A request nothing
  serves yet is refused in `temper-service/src/family.rs`'s `to_root` as a request it cannot decode.
- `temper/tests/codecs/tests/suite/fuzzy/client_wire.rs`: the new variants in its generators.
- Pre-approved: if `lib.rs`'s validation (about 309 to 345) bounds requests by hand, `MakeRepository`
  takes `Adopt`'s; `address` is bounded by the schema alone.

## Steps

1. The schema, the writer. 2. `limits()` and the validation. 3. `source.rs` and its tests, each biting:
take `http://`, and let `..` through, each once. 4. The compiler's fallout. 5. The implementer's checks;
commit and push.

## Report

As the implementer's text asks; any name T3 had to change, and the worst case of a `Forges` document.
