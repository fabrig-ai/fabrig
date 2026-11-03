+++
id = "kinds-codec"
part = "kinds-root"
kind = "interface"
needs = ["kinds-values", "change-call-llm"]
files = ["temper/crates/temper-client-protocol-wire/**", "temper/crates/temper-engine-smith/Cargo.toml", "temper/crates/temper-engine-smith/src/kinds.rs", "temper/crates/temper-engine-smith/src/lib.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/crates/temper-engine-smith/src/tests/kinds.rs", "temper/tests/codecs/tests/suite/fuzzy/client_wire.rs", "Cargo.lock"]
effort = "high"
minutes = 90
+++

# kinds-codec: a value of temper's kinds in temper's codec

## Goal

A setting of kinds 1 to 3 crosses jig as bytes (C1's `Value::Application`), the encoding of
`temper-client-protocol-wire`'s `LandingRule` or of a charter's number (`root.md`, 5.6), and
is decoded, bounded, where the request that carries it enters (the clients' component, a run's
`propose_settings`). This task gives the encoding and the two translations every entrance and
reader shares, and uses neither yet.

Acceptance:

- `client-wire-v1.schema`, in place (`CLIENT_VERSION` stays 11): records for a landing rule
  member for member as `temper_engine_domain_forge::settings::LandingRule` (its pattern's
  segments and last, exact or open; `enforce`, `ci`, `up_to_date`; its gates, each number,
  blocking and freshness; its approvals, each role, people and freshness) and a charter's
  number (`record ForgeCharter { charter: u32 }`), each bounded by the wire's `Bounds` from
  the figures the decoder is given (segments, segment bytes, gates, approvals). Generated code
  and goldens regenerated through the drift test.
- A new module `temper_engine_smith::kinds` (module doc citing
  `temper/docs/design/domain/root.md`, 5.6 and `protocol/configuration.md`, 13.2), with
  `temper-client-protocol-wire` a new dependency of `temper-engine-smith` (the one this spec
  allows):
  - `pub struct Bounds { pub segments: u32, pub segment_bytes: u32, pub gates: u32, pub
    approvals: u32 }`;
  - `pub fn decode(kind: u32, bytes: &[u8], bounds: &Bounds) -> Option<settings::SizedValue>`:
    kinds 1 and 2 a `LandingRule`, kind 3 a charter, `size` the bytes' length; none for any
    other kind, bytes that do not decode whole, or a list past its bound (out of shape: what
    a value means is the forge's `judge`, not this);
  - `pub fn encode(value: &settings::Value) -> Box<[u8]>`, so that `decode` of `encode` gives
    the value back with `size` the encoding's length; `pub fn size(value: &settings::Value) ->
    u32` (the stock's sizes, `kinds-start`).
- Tests in `src/tests/kinds.rs` pass, and the codecs world's client wire fuzzing round-trips
  the new records.

## Inputs

- `temper/docs/design/domain/root.md`, 5.6 (**Typed at the entrance**);
  `temper/docs/design/protocol/configuration.md`, 13.2 (**A landing rule's value**) and
  section 3's rule table (members and their meaning).
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C1 (bytes across
  jig) and C3.

## Owns and may change

- Owns the files in the header. `lib.rs` of temper-engine-smith: one `pub mod kinds;` line;
  `tests.rs`: one `mod kinds;` line. `Cargo.lock`: the new dependency edge alone.
- Options: if the wire's `Bounds` cannot take a list's bound at decode time, bound the records
  at the schema's ceilings (gates and approvals at 64, segments at 16 of 4,096 bytes) and check
  `bounds` after decoding, saying so. If the client wire's version is not kept in place on this
  trunk (its design says otherwise), ask.

## Steps

1. The schema's records; regenerate:
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E 'test(drift::)'`.
2. The module and its dependency.
3. Tests in `src/tests/kinds.rs`:
   - `each_kind_round_trips_its_value`: a deployment rule with gates and approvals, a project
     rule with an open last, a charter;
   - `a_value_past_its_bounds_or_cut_short_is_out_of_shape`: a gate past `gates`, a segment
     past `segment_bytes`, trailing bytes, a truncated rule, kind 4;
   - `size_is_the_encodings_length`.
   Narrow: `heavy cargo nextest run -p temper-engine-smith --lib -E 'test(/kinds::/)'` (check
   the count: three); `heavy cargo nextest run -p temper-codecs-world --test suite`.
4. fmt, old forms, clippy on both crates; their focused tests.
5. Commit ("Encode temper's kinds' values in temper's codec") and push.

## Report

The implementer's text's report, plus: the record names written; the goldens regenerated
(count).
