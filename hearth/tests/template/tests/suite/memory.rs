use hearth_protocol_template::expression;
use hearth_template_world::fixtures;
use skein_heap::Meter;

#[test]
fn compiling_at_the_limits_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 140_000;
    for (text, what, _) in fixtures::at_the_limits() {
        let meter = Meter::new();
        meter.start();
        let compiled = expression::compile(&text, &fixtures::LIMITS);
        meter.check(meter.end(), CEILING, &what);
        drop(compiled);
    }
}

#[test]
fn compiling_templates_and_strings_at_the_limits_holds_no_more_than_their_ceilings() {
    const TEMPLATE_CEILING: u64 = 800_000;
    const STRING_CEILING: u64 = 800_000;
    let limits = &fixtures::MEMORY_LIMITS;
    for (text, what, _) in fixtures::templates_at_the_limits(limits) {
        let meter = Meter::new();
        meter.start();
        let compiled = hearth_protocol_template::template::compile(&text, limits);
        meter.check(meter.end(), TEMPLATE_CEILING, &what);
        drop(compiled);
        let meter = Meter::new();
        meter.start();
        let compiled = hearth_protocol_template::interpolation::compile(&text, limits);
        meter.check(meter.end(), STRING_CEILING, &what);
        drop(compiled);
    }
}
