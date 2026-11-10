//! The fixed large render at shipped limits (hearth/docs/design/domain/render/README.md, 5.4).
use hearth_builder_world::{fixtures, simulated};

#[test]
fn a_large_render_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 2_700_000;
    let outcome =
        simulated::checked_with(&fixtures::large::scenario(), fixtures::SIMULATION_SEED, hearth::limits(), CEILING);
    let output = outcome.machine.contents(fixtures::OUTPUT).expect("printed specs");
    assert_eq!(
        fixtures::client_exits(&outcome),
        [skein_io::kernel::Exit::Code(0)],
        "{}",
        String::from_utf8_lossy(&output)
    );
    assert_eq!(String::from_utf8_lossy(&output).matches("target       ").count(), fixtures::large::TARGETS);
    let (entries, documents) = fixtures::large::served(&outcome.trace);
    assert_eq!(entries, fixtures::large::ENTRIES, "git served the whole revision listing");
    assert_eq!(documents, fixtures::large::DOCUMENTS, "git served every document, declaration and template");
}
