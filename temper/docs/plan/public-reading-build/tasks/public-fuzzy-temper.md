+++
id = "public-fuzzy-temper"
part = "public-checks"
kind = "feature"
needs = ["forge-public-wire", "public-web"]
files = ["temper/tests/codecs/tests/suite/fuzzy/public_wire.rs", "temper/tests/codecs/tests/suite/main.rs", "temper/tests/client/tests/suite/fuzzy/public_address.rs", "temper/tests/client/tests/suite/main.rs"]
effort = "medium"
minutes = 60
+++

# Fuzzy tests: the forge's public records' decoders and the public addresses' parser

## Goal

Seeded sweeps over what a visitor's browser decodes and parses, in the worlds where temper
keeps its fuzzy modules (`notes/common.md`, "Tests"): the forge's public records' decoders
never panic and re-encode what they decode (`temper/docs/design/client/forge.md`, 11.4), and
the address codec's parser, over seeded paths under `/public`, never panics and prints back
what it read (`temper/docs/design/client/browser.md`, 8.4).

Acceptance:

- `temper/tests/codecs/tests/suite/fuzzy/public_wire.rs`, declared in its suite's
  `mod fuzzy { .. }`, as `fuzzy/client_wire.rs` is:
  `public_records_decoders_never_panic_and_reencode_the_same`: for seeds 1 to N (as
  `client_wire.rs` sweeps), arbitrary bytes and mutations of each kind's valid record (built
  at its bounds, with one byte flipped, inserted, removed, and a length set past its bound)
  through `temper_client_protocol_wire::public_v1`'s `PublicChange`, `PublicLandings` and
  `PublicLanding` decoders at `public_v1::CEILINGS` (or the limits `public_limits` gives at the
  panels' default bounds): a decode that succeeds and consumes every byte re-encodes to the
  same bytes; none panics; a length past its bound is refused. The seed is in every failure.
- `temper/tests/client/tests/suite/fuzzy/public_address.rs`, declared in its suite's
  `mod fuzzy { .. }`: `public_paths_never_panic_and_print_back_what_they_read`: for seeded
  paths under `/public` (valid ones built from random identifiers and pages; and mutations:
  digits changed to upper case or non-hex, identifiers shortened or lengthened, segments
  added, emptied, doubled or dotted, a percent-encoded byte, a query, a trailing slash, bytes
  past ASCII), `temper_client_domain::address::parse` never panics; a path the test's own
  oracle (browser.md 8.1's grammar, written in the test) finds valid reads as
  `Place::Public` of its page and `print_public` gives it back byte for byte; every other path
  under `/public` reads as `Place::Public(Public::Projects)`. The seed is in every failure.
- Both run in the `fuzzy` profile, within its budget, and are absent from the focused one
  (the profiles select `test(/^fuzzy::/)`).

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, section 5.
- `temper/docs/design/client/browser.md`, 8.1 and 8.4; `temper/docs/design/client/forge.md`,
  11.4.
- `tasks/forge-public-wire.md` (the records, `public_limits`), `tasks/public-web.md`
  (`address::parse`, `Place::Public`, `Public`, `print_public`).
- As they stand: `temper/tests/codecs/tests/suite/fuzzy/client_wire.rs` (the sweep's form),
  `temper/tests/codecs/tests/suite/main.rs`, `temper/tests/client/tests/suite/fuzzy/client.rs`
  and `main.rs`; `.config/nextest.toml`'s `fuzzy` profile.

## Owns and may change

- Owns the two new files, and in each suite's `main.rs` only the module line inside
  `mod fuzzy`.
- Nothing else: a decoder or parser that panics or prints back otherwise is reported with the
  seed and the input under `## Left`, the test left failing, never repaired here (the codec is
  `forge-public-wire`'s, the parser `public-web`'s).
- Pre-approved: if `public_v1` has no `CEILINGS`, use `public_limits` at the bounds the
  service declares and say which.

## Steps

1. The records' sweep; run it alone
   (`heavy cargo nextest run -p temper-codecs-world --test suite --profile fuzzy -E 'test(/fuzzy::public_wire/)'`).
2. The addresses' sweep
   (`heavy cargo nextest run -p temper-client-world --test suite --profile fuzzy -E 'test(/fuzzy::public_address/)'`).
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-codecs-world -p temper-client-world --all-targets -- -D warnings`; each world's
   fuzzy suite whole (`--profile fuzzy`), checking its budget; the citations check.
4. Commit (why: what a visitor's browser decodes and parses holds over seeded inputs); push
   your branch.

## Report

As the implementer's text says, and also: the seeds and cases swept per test, and each
sweep's time against the fuzzy budget.
