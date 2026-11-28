#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod compose;
mod debian;
mod decoded;
mod decoders;
mod diff_reuse;
mod entries;
mod errors;
mod evaluate;
mod exports;
mod golden_harness;
mod goldens;
mod laws;
mod memory;
mod oracle;
mod properties;
mod schema;
mod templates;
mod thin;

mod fuzzy {
    //! Render's campaign cases (skein/docs/design/testing/sweeps.md, 4).

    use skein_lib::Rng;

    mod entries;
    mod evaluate;
    mod oracle;
    mod properties;
    mod schema;
    mod templates;

    fn reference_rng(offset: u64, seed: u64) -> Rng {
        Rng::new(offset.wrapping_add(seed))
    }

    #[test]
    fn reference_draws_wrap_the_full_width_case_seed() {
        // Each block wraps to the seed just before it.
        for (offset, expected) in [
            (entries::SEED, 0x0016_efff),
            (evaluate::SEED, 0x0014_efff),
            (schema::SEED, 0x0013_efff),
            (templates::SEED, 0x0015_efff),
        ] {
            assert_eq!(reference_rng(offset, u64::MAX).state(), expected, "block {offset:016x}");
        }
    }
}
