# fuzzy-context: a fuzzy case names itself, not its input

Draft, 2026-11-14. Panics' P4 (`panics.execution.md`, on main at 11:02:50Z) took the fuzzy sweeps' `catch_unwind` out, and each sweep now writes its context to stderr before it runs a case, as the design asks (`skein/docs/design/testing/README.md`, 5; `jig/docs/design/domain/testing.md`, 7). Three sweeps write each case's whole input in its debug form, a line per case over thousands of cases, and the gate's fuzzy step went from about 80 s on 2026-11-13 to about 300 s: it ends when its longest test does. In the CI builder's fuzzy run at 13:58Z, `temper-client-world`'s `public_paths_never_panic_and_print_back_what_they_read` took 299 s, `jig-client-wire-world`'s `public_records_decode_without_panic_and_re_encode_what_they_decode` 276 s, and `jig-client-protocol-world`'s `the_parser_accepts_exactly_the_canonical_paths` and `the_public_path_parser_never_panics` 169 s and 122 s. On 2026-11-13 the whole step took 78 s, so none of them took longer. The fourth sweep P4 changed this way, temper's codecs world, ran into the 600 s limit until its repair, `3e5f23b754`, wrote each input's length instead: 4 s, its seeds, cases and assertions unchanged. This request gives the other three the same repair. **Run, unless marked read.**

## What's wanted, ranked. Drop from the bottom if it passes two tasks, and say what you dropped

1. **The three sweeps write what names a case, not its bytes,** as `temper/tests/codecs/tests/suite/fuzzy/public_wire.rs` does since `3e5f23b754`: the seed, the case (its kind, number or mutation) and the input's length, before it runs.
   - `temper/tests/client/tests/suite/fuzzy/public_address.rs`: `check`'s line, which writes `input {path:?}`.
   - `jig/tests/client-wire/tests/suite/fuzzy/public_wire.rs`: `decoders`' line, which writes `input {bytes:?}` and `limits {limits:?}`. The bound's name replaces the limits.
   - `jig/tests/client-protocol/tests/suite/fuzzy/public_path.rs`: `inspect`'s and `printed`'s lines, which write `input {bytes:?}`.

   The seeds, draws, cases and assertions stay as they are. An assertion's message still names the input, and the runner prints it when the test fails. Nothing catches a panic.
2. **The design says it.** In `skein/docs/design/testing/README.md`, 5, the bullet on a harness that adds context, and in `jig/docs/design/domain/testing.md`, 7, the bullet on a harness that names a seed: what a harness writes before a case names the case, never its input, which the failing assertion's message names.

## Not in this plan

Any other fuzzy test, its seeds or its counts. The gate's suites and where the fuzzy suite runs. The panics count and its list (`panics.md`, 7).

**Tests stay cheap:** no new test; the sweeps' own runs are the check. **A plan runs the focused and fuzzy suites alone and is complete when it lands (`AGENTS.md`, 5).** A stored format or protocol change is a question first. `walk = false`.
