+++
id = "public-limits"
part = "jig-public"
kind = "interface"
needs = []
files = ["jig/crates/jig-protocol-config/src/**", "jig/crates/jig-protocol-engine-clients/src/limits.rs", "jig/crates/jig-protocol-engine-clients-http/src/lib.rs", "jig/crates/jig-domain-core/src/vocabulary.rs", "jig/testing/jig-test-domain/src/lib.rs", "jig/testing/jig-test-service/src/limits.rs", "jig/testing/jig-test-profile/src/lib.rs", "temper/crates/temper-engine-domain/src/limits.rs", "temper/crates/temper-service/src/limits.rs"]
effort = "high"
minutes = 90
+++

# Public reading's limits: the figures, every profile's zero, and their relations at start

## Goal

Every limit public reading needs exists, is zero in every profile, is configurable by the
deployment's `limits.overrides`, and is checked against the others at start, so the tasks
that build public reading read their figures from `Limits` and never add one
(`jig/docs/design/protocol/configuration.md`, section 6; `jig/docs/design/domain/people.md`,
13.5 and 13.9; `jig/docs/design/domain/root.md`, section 10). Nothing reads them yet but the
start's checks.

Acceptance:

- `jig_protocol_engine_clients::Limits` has `public: PublicLimits` exactly as
  `notes/contracts.md`, section 1 (with `PublicLimits::OFF` and `on`);
  `jig_protocol_engine_clients_http::Limits` has `public_calls: u32`; `jig_domain_core::Limits`
  has `public: PublicIndexLimits` (`goals`, `ended`, `lines`, `depth`, `u16`, and `OFF`); jig's
  root `Limits` (`jig-test-domain`) and temper's (`temper-engine-domain`) have
  `public_watermark: u32` and `public_reads: u32`, set as the contracts say where their
  watermark is set.
- Both profiles (`profile/small.rs`, `profile/standard.rs`) give every public figure zero.
- `limits.overrides` takes `"clients.public.reads"`, `"clients.public.rate"`,
  `"clients.public.rate_burst"`, `"clients.public.bytes"`, `"clients.public.bytes_burst"`,
  `"clients.public.answer_bytes"`, `"clients.public.max_age"`, `"clients_http.public_calls"`,
  `"core.public.goals"`, `"core.public.ended"`, `"core.public.lines"` and
  `"core.public.depth"`, parsed as their neighbours are.
- The start refuses, at the override's path with `What::Bound` (as `overrides.rs`'s
  `codec_problem` spells it), each broken relation of configuration.md, section 6, checked
  only when `clients.public.rate > 0`, but for `max_age <= 60`, which is always checked:
  `rate_burst >= rate`; `bytes_burst >= bytes`; `answer_bytes <= bytes_burst`;
  `public_calls <= clients.public.reads`; `reads > 0`; each `core.public.*` above zero; and
  the public index's memory within the fit check, as the estimate the fit check already makes
  for the core gains the index's bound (`goals + ended` goals times `lines` lines per public
  project, at most `people.projects` projects, each line priced at the `PublicLine` bound
  `public-wire` will give; until then a constant of 128 bytes a line, named and commented).
  The kinds' bounds against `answer_bytes` are checked by `public-component`, not here.
- Step tests in `jig-protocol-config`'s `src/tests/` (as `tests/profile.rs` and the
  overrides' tests are):
  - `every_profile_turns_public_reading_off`: both profiles' public figures are all zero and
    the start passes;
  - `public_overrides_parse_at_their_paths`: each key above set by an override reaches its
    field;
  - `public_relations_refuse_the_start_at_their_paths`: with the rate set above zero, each
    relation broken alone refuses the start at its path, and the figures of
    `temper/docs/design/draft/first-release.status.md`'s proposal for dev.fabrig.ai pass;
  - `a_zero_rate_checks_only_max_age`: a rate of zero with every other figure inconsistent
    passes, and `max_age` 61 refuses.
- The root's tests (`jig-test-domain`'s and temper's root's, where `Limits::valid` or its
  equivalent is tested) still pass, and `public_watermark` is half the watermark in
  `jig-test-service`'s and temper's effective limits (one assertion each where their limits
  are tested).

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, section 1.
- `jig/docs/design/protocol/configuration.md`, section 6 (public reading's limits and their
  relations); `jig/docs/design/domain/root.md`, section 10 (the public watermark);
  `jig/docs/design/domain/people.md`, 13.5 and 13.9.
- As they stand: `jig-protocol-config`'s `profile.rs`, `profile/small.rs`,
  `profile/standard.rs`, `overrides.rs` (`apply` and `parse`), `clients.rs`
  (`check_settings`), `domain.rs` (`whole`, `check_effective_limits`);
  `jig-protocol-engine-clients/src/limits.rs` (`valid`); `jig-test-domain/src/lib.rs`
  (`Limits`, `default_watermark`); `temper-engine-domain/src/limits.rs`;
  `temper-service/src/limits.rs` (`engine_candidate`).

## Owns and may change

- Owns the files in the header, two of them shared, by hunk, with `public-vocabulary`, which
  runs at the same time: in `jig/crates/jig-domain-core/src/vocabulary.rs` edit only the
  core's `Limits` struct (and `PublicIndexLimits` beside it); in
  `jig/testing/jig-test-domain/src/lib.rs` only the root's `Limits` and its watermark's
  function. `public-vocabulary` edits the `Event`, `Ask` and `Held` enums and the root's
  routing, never these.
- **Reader literals,** each edited only to add the new field, its value zero (or `OFF`), or
  for the roots' `public_watermark` half the watermark beside it (checked division) and
  `public_reads` the clients' `public.reads`:
  - clients `Limits`: `jig/tests/client/src/joint/component.rs`,
    `temper/tests/client/src/component.rs`, `jig/crates/jig-protocol-engine-clients/src/tests/fixtures.rs`;
  - `clients_http` `Limits`: `jig/tests/engine-clients/src/fixtures.rs`,
    `jig/tests/client-protocol/src/fixtures.rs`;
  - root `Limits`: `jig/testing/jig-test-domain/src/tests/fixtures.rs`, and every literal of
    temper's root `Limits` the compiler names;
  - core `Limits`: every literal the compiler names (`jig-protocol-store`'s tests and
    fixtures, `jig-test-profile`, among them).
  Any other literal the compiler names in jig or temper: the same one-line edit, listed in the
  report.
- `temper/crates/temper-service/src/tests/limits.rs` and `tests/fixtures.rs` only where the
  compiler or an assertion of the effective limits needs the new fields.
- Pre-approved: if the core's `Limits` is not configured by `core.*` overrides the way
  `clients.*` is, add `core.public.*` the way the core's nearest configured field is, and say
  which in the report. If the fit check has no per-process estimate the index's memory can
  join, check `goals + ended` times `lines` times the line constant times `people.projects`
  against `u32::MAX` bytes instead, and report it under `## Left`.

## Steps

1. The structs and their `OFF`s; the profiles; the reader literals until
   `heavy cargo check -p jig-protocol-config -p jig-test-service -p temper-service --tests`
   passes.
2. The overrides' keys and the relations; the step tests above; run them alone
   (`heavy cargo nextest run -p jig-protocol-config --lib -E 'test(/public/)'`).
3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; clippy on every package you changed; the
   focused tests of `jig-protocol-config`, `jig-protocol-engine-clients`,
   `jig-protocol-engine-clients-http`, `jig-test-domain`, `jig-test-service`,
   `temper-engine-domain` and `temper-service`; the citations check.
4. Commit (why: public reading's figures, off in every profile and checked at start, which
   every later task reads); push your branch.

## Report

As the implementer's text says, and also: the overrides' keys as they parse, the relation
checks with the path each refuses at, and every literal you edited.
