use hearth_accounts_world::{
    fixtures::{self, FILES, LIMITS},
    world,
};
use hearth_protocol_accounts::read;
#[test]
fn a_read_at_its_limits_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 2_000;
    const FILLED_LINES: usize = 64;
    for file in FILES {
        for bytes in fixtures::at_the_limits(file, &LIMITS) {
            let lines = read(file, &bytes, &LIMITS).unwrap();
            assert_eq!(lines.len(), FILLED_LINES, "{file:?}: the fixture fills the limits");
            drop(lines);
            let most = world::read_metered(file, &bytes, &LIMITS, CEILING);
            assert!(most <= CEILING, "{file:?}");
        }
    }
}
