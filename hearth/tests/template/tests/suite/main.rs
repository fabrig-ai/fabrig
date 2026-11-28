#[global_allocator]
static HEAP: skein_heap::Counting = skein_heap::Counting;

mod expressions;
mod memory;
mod templates;

mod fuzzy {
    //! Parser sweeps use the case seam (`skein/docs/design/testing/sweeps.md`, 4.2–4.5).

    use skein_lib::Rng;
    use skein_world::sweep::Case;
    use std::convert::Infallible;

    mod expressions;
    mod goldens;
    mod templates;

    type SweepResult = Result<(), Case<Infallible>>;

    const EXPRESSIONS: u64 = 0x0005_F000;
    const EXPRESSIONS_AT_LIMITS: u64 = 0x0037_F000;
    const TEMPLATES: u64 = 0x0006_F000;
    const STRINGS: u64 = 0x0036_F000;
    const TEMPLATES_AT_LIMITS: u64 = 0x0036_A000;

    fn offset_rng(seed: u64, offset: u64) -> Rng {
        Rng::new(seed.wrapping_add(offset))
    }

    #[test]
    fn offset_draws_accept_full_width_seeds() {
        for (offset, state) in [
            (EXPRESSIONS, 0x0005_EFFF),
            (EXPRESSIONS_AT_LIMITS, 0x0037_EFFF),
            (TEMPLATES, 0x0006_EFFF),
            (STRINGS, 0x0036_EFFF),
            (TEMPLATES_AT_LIMITS, 0x0036_9FFF),
            (hearth_test_fixtures::golden::SEED, 0x0030_5FFF),
        ] {
            assert_eq!(offset_rng(u64::MAX, offset).state(), state, "offset {offset:#x}");
        }
    }
}
