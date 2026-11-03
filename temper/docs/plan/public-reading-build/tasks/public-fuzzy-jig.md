+++
id = "public-fuzzy-jig"
part = "public-checks"
kind = "feature"
needs = ["public-http", "public-wire"]
files = ["jig/tests/client-protocol/tests/suite/fuzzy/public_path.rs", "jig/tests/client-wire/tests/suite/fuzzy/public_wire.rs"]
effort = "medium"
minutes = 60
+++

# Fuzzy tests of jig's public path and public records

## Goal

Seeded sweeps show that the public route's parser never panics, accepts exactly the
canonical paths and prints back what it read, and that jig's public records' decoders never
panic and re-encode what they decode (`jig/docs/design/protocol/client.md`, section 19's
Testing; `jig/docs/design/client/domain/contract.md`, section 12). They run in the `fuzzy`
profile only.

Acceptance:

- `jig/tests/client-protocol/tests/suite/fuzzy/public_path.rs` (a module line in the
  `fuzzy` block of `tests/suite/main.rs`):
  - `the_public_path_parser_never_panics`: seeded arbitrary bytes, and canonical paths
    mutated (a byte flipped, inserted or removed; a segment doubled, emptied, made `.` or
    `..`, percent-encoded, upper-cased; a leading zero; a number past `u16`; a query, a
    fragment, a trailing slash; a third id), each through
    `jig_protocol_engine_clients_http::public_path`;
  - `the_parser_accepts_exactly_the_canonical_paths`: for every input it accepts,
    `jig_client_protocol_http::public_target` of what it read gives the input back byte for
    byte; and every `PublicPath` drawn from the seed (owner, version and kind over their whole
    range with their edges, zero to two ids) prints a target the parser accepts and reads back
    equal;
  - every failure names its seed and the input's bytes.
- `jig/tests/client-wire/tests/suite/fuzzy/public_wire.rs` (a module line in the `fuzzy`
  block of `tests/suite/main.rs`), as `client_wire.rs` checks the client records:
  `public_records_decode_without_panic_and_re_encode_what_they_decode` over seeded arbitrary
  bytes and the `golden/public/` files mutated, for `PublicProjects`, `PublicBoard`,
  `PublicGoal`, `PublicCard`, `PublicLine`, `PublicCounts`, `PublicApplication` and the three
  enums, under the limits `public_limits` gives at the bounds `public-wire`'s tests use and at
  their largest.
- Both run under `heavy cargo nextest run -p <world> --profile fuzzy`, within the fuzzy
  tier's budget; neither runs in the default profile.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md` ("Tests": what a fuzzy test is);
  `notes/contracts.md`, sections 3 and 4.
- `jig/docs/design/protocol/client.md`, 17.5 (the path's form) and section 19 (Testing).
- As they stand: `jig/tests/client-wire/tests/suite/fuzzy/client_wire.rs` (the precedent),
  `jig/tests/client-protocol/tests/suite/fuzzy/protocol.rs`, both worlds' `tests/suite/main.rs`;
  `public-http`'s `public_path`, `public_target` and `PublicPath`; `public-wire`'s
  `public_v1`, `public_limits` and `golden/public/`; `.config/nextest.toml`'s `fuzzy` profile.

## Owns and may change

- Owns the files in the header, and one module line each in
  `jig/tests/client-protocol/tests/suite/main.rs` and `jig/tests/client-wire/tests/suite/main.rs`.
- A panic, an accepted non-canonical path or a record that re-encodes otherwise is a finding.
  Pre-approved: fix it where it lies when the fix is local to the public path's parser
  (`jig-protocol-engine-clients-http`) or the public records' limits
  (`jig-client-protocol-wire/src/public.rs`), with a step test there that replays the seed's
  input, and report it. A finding whose fix reaches further stops the task with a `Question:`,
  the failing test pushed on your branch and named with its seed; never an `#[ignore]`.
- Pre-approved: if `jig/tests/client-wire` cannot reach the generated limits' constructor,
  build the limits from `PublicIndexBounds` as `public-wire`'s tests do and say so.

## Steps

1. The two modules; run each alone in its profile
   (`heavy cargo nextest run -p jig-client-protocol-world --profile fuzzy -E 'test(/fuzzy::public_path::/)'`,
   `heavy cargo nextest run -p jig-client-wire-world --profile fuzzy -E 'test(/fuzzy::public_wire::/)'`),
   checking each selected its tests.
2. Their time against the fuzzy budget (`AGENTS.md`, 6): over it is red; shorten the sweep
   before anything else.
3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-client-protocol-world -p jig-client-wire-world --all-targets -- -D warnings`;
   both worlds' default profiles still pass (the new tests not among them); the citations
   check.
4. Commit (why: seeded sweeps of the public path and records); push your branch.

## Report

As the implementer's text says, and also: each sweep's size and time, and any finding with
its seed.

## Answers

2026-11-03. **The user's decision on public titles.** User, 2026-11-03 15:40Z: public reading shows titles. A goal's and a task's title (the first line of its spec, as jig's client contract names a task's title) crosses into its public line, and a public project's name crosses into its public record. There is no author flag and no human publication capability for these. Everything else in people.md 13.3's "No text at all" list still never crosses: spec beyond its first line, instructions, inputs, results, words, questions and answers, transcripts, notes, reasons, labels' meanings, proposals, escalations, people's and parties' names, charters', procedures' and roles' names. Review finding 1 is settled by the user's decision.

For this task: Titles and the project's name are expected in public bytes now; every other text source keeps its marker and must never appear.
