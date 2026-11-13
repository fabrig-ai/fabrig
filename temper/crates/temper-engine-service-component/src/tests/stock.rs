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
    limits.memory_bytes = settings.jig.memory;
    limits.host_reply_bytes = settings.host_reply_bytes;
    let core = crate::system::core_config(&settings.jig, &seeds);
    let connector = *core.connectors.first().unwrap();
    assert_eq!(settings.kinds, crate::stock::kinds(&limits, connector));
    assert_eq!(settings.period_span, crate::stock::PERIOD_SPAN);
}

pub(super) fn generated() -> Box<[u8]> {
    use core::num::NonZeroU16;
    let answer = temper_init::files(
        &temper_init::Facts {
            sign_in: false,
            binary: b"/bin/temper",
            git: Some(b"/bin/git"),
            claude: None,
            login: Some(b"ada"),
            ports: temper_init::Ports {
                clients: NonZeroU16::new(8367).unwrap(),
                owned: NonZeroU16::new(8368).unwrap(),
                forge: NonZeroU16::new(8369).unwrap(),
                llm: NonZeroU16::new(8370).unwrap(),
                webhooks: NonZeroU16::new(8371).unwrap(),
            },
        },
        &temper_init::Directories { config: b"/config", data: b"/data" },
        &[0; 32],
        &[1; 32],
    )
    .unwrap();
    for file in answer.files {
        if file.name == b"temper.json" {
            let temper_init::Content::Bytes { bytes, .. } = file.content else { panic!("document bytes") };
            return bytes;
        }
    }
    panic!("init writes temper.json")
}
pub(super) fn engine(document: &[u8]) -> config::Engine {
    fixtures::parse_engine(&fixtures::value(document, b"engine"))
}
