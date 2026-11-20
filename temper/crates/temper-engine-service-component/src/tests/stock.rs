use super::fixtures;
use jig_protocol_config as config;
#[test]
fn startup_carries_the_components_stock_kinds_and_period_span() {
    let document = fixtures::engine_part(&fixtures::document());
    let seeds = fixtures::seeds();
    let start = crate::startup::checked_start_with(
        &document,
        &config::Directories::working(),
        &fixtures::files(),
        None,
        &seeds,
        fixtures::forge_runtime(),
        None,
        &crate::stock::Given::Stock,
    )
    .unwrap();
    let settings = start.engine;
    let mut limits = crate::limits::engine(&settings.jig.limits, &settings.limits).unwrap();
    limits.host_reply_bytes = settings.host_reply_bytes;
    let core = crate::system::core_config(&settings.jig, &seeds);
    let connector = *core.connectors.first().unwrap();
    assert_eq!(settings.kinds, crate::stock::kinds(&limits, connector));
    assert_eq!(settings.period_span, crate::stock::PERIOD_SPAN);
}

pub(super) fn generated() -> Box<[u8]> {
    temper_config_shell::init_document(false)
}
pub(super) fn engine(document: &[u8]) -> config::Engine {
    let bytes = fixtures::value(document, b"engine");
    let document =
        config::document(&bytes, &config::DocumentLimits { bytes: 1_048_576, tokens: 16_384, depth: 32 }).unwrap();
    crate::startup::engine(&document, 0, &config::Directories::working()).unwrap()
}
