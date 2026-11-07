+++
id = "public-wire"
part = "codecs"
kind = "feature"
needs = []
files = [
  "temper/tests/codecs/**",
  "temper/docs/plan/limits-fuzzy/figures/public-wire.md",
]
effort = "high"
minutes = 50
+++

# The public records' decoder sweep

## Goal

`fuzzy::public_wire::public_records_decoders_never_panic_and_reencode_the_same` costs 3.9 CPU s of `temper-codecs-world`'s 5. Nobody has profiled it. Profile it, name each cause, fix the largest in `temper/tests/codecs/**`, and stop when the next is under about 2% of the sweep. All 2,000 seeds still draw all their inputs.

Acceptance:

- `temper/docs/plan/limits-fuzzy/figures/public-wire.md` holds, in `notes/measuring.md`'s form, instructions and native CPU before and after for this test, with each cause fixed, what it saved, what's left and what you stopped at.
- For each of the three kinds, each seed in `1..=2000` still checks its arbitrary, flipped, inserted and removed inputs, and their bytes are the same as before for that seed: your report says how you know.
- The fixed checks, "at every bound" and "length past ceiling", still hold for each kind, with the same assertions: a bound record decodes and re-encodes to the same bytes, and a length past its ceiling is refused at that record's path.
- A failing input's message still names its seed, kind, case and bytes.
- `heavy cargo nextest run -p temper-codecs-world` and the same with `--profile fuzzy` pass, with the same tests by name as your base.

## Inputs

- `temper/docs/plan/limits-fuzzy/notes/common.md`, `temper/docs/plan/limits-fuzzy/notes/measuring.md`.
- `temper/docs/design/client/forge.md`, 11.4, for what these records promise; `skein/docs/design/codec.md`, 6; `skein/docs/foundation/testing-strategy.md`, 8.
- `temper/tests/codecs/tests/suite/fuzzy/public_wire.rs`; `temper/tests/codecs/src/lib.rs`.

## Owns and may change

Owns `temper/tests/codecs/**` and the figures file. The decoders, generated under `temper/crates/temper-client-protocol-wire/src/generated/`, and `skein_lib`'s `Reader` and `Writer` aren't yours: a cause there goes under `## Left`, with its share, for `under-causes`.

Where to look first. Reading shows these; the profile decides whether any is the cause:

- **Two of each kind's checks don't vary with the seed.** In `sweep`, "at every bound" decodes `golden` and "length past ceiling" decodes `golden` with one fixed length, and neither reads `rng`. So 2,000 seeds decode the same two inputs 2,000 times.
- **Each mutated input starts from a whole bound record,** so it decodes a record at every ceiling, with an allocation for each of its lists. Those decodes are the draws, and they stay.
- **`temper_codecs_world::drawn`** makes its bytes one `rng.below(256)` at a time, through a `try_from`, and the world's `records`, `store` and `worker_wire` sweeps and a focused test in `tests/suite/records.rs` read it too.
- **`catch_unwind` per check,** beside the one per seed.

Options, each decided:

- **The fixed checks are over 2% of the sweep:** check each kind's two fixed inputs once, in the same test, before the seeds' loop, through the same `check`. Their messages name the kind and case, with no seed. The sketch:

  ```rust
  #[test]
  fn public_records_decoders_never_panic_and_reencode_the_same() {
      let fixtures = catch_unwind(bound_records).expect("seed 1: build bound records");
      for (kind, golden, length_at, ceiling) in &fixtures {
          fixed(*kind, golden, *length_at, *ceiling);
      }
      for seed in 1..=SEEDS {
          assert!(catch_unwind(|| sweep(seed, &fixtures)).is_ok(), "seed {seed}: public record sweep panicked");
      }
  }
  ```

  `sweep` loses those two checks and draws nothing less, so each seed's other inputs keep their bytes. No new test, no renamed test.
- **`drawn` is over 2% of the sweep:** make it cheaper in `temper/tests/codecs/src/lib.rs`, drawing the same bytes for the same `Rng` state if you can. If it can only be cheaper by drawing other bytes, the seeds still draw the same number of inputs, the world's sweeps must still pass, and your report says so.
- **A check's `catch_unwind` is over 2%:** one per seed already stands, and it names the seed. Keep the message's kind and case some other way, or leave it and say why.
- **The world's other four sweeps** (`client_wire`, `records`, `store`, `worker_wire`) cost about 1.1 CPU s together: leave them unless a fix you've made reaches them, and then give their figures too.
- **The next cause is a decoder's own work on a bound record:** that's what the sweep exists to run. It stays, and your report says so with its share.

## Steps

1. Profile the test on your base, as `notes/measuring.md` says, and take its native CPU.
2. Fix the largest, re-profile, repeat until the stop rule. Before you change the draws, keep one seed's inputs from your base to compare in your report.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-codecs-world --all-targets -- -D warnings`; the runs in the acceptance; commit and push.

## Report

Instructions and CPU seconds before and after, each cause and what it saved, how you know each seed's draws and both fixed checks stand, what you left and whose it is, and your implementation and check minutes apart.
