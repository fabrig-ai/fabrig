use hearth_git_world::{
    fixtures::{self, SMALL},
    world,
};
use skein_heap::Meter;
#[test]
fn each_decoder_at_its_limits_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 1_900;
    for case in fixtures::at_the_limits(&SMALL) {
        let mut run = world::Stepped::new(&case, &SMALL);
        let meter = Meter::new();
        meter.start();
        run.open();
        let step = meter.end();
        run.drop_events();
        meter.check(step, CEILING, &case.what);
        while run.pending() {
            meter.start();
            run.call();
            let step = meter.end();
            run.drop_events();
            meter.check(step, CEILING, &case.what);
        }
    }
}
