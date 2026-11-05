use hearth_mmdebstrap_world::{fixtures, reference, world};
use hearth_protocol_mmdebstrap::{limits, tail::Cause};
#[test]
fn an_output_reader_at_its_limits_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 5_600;
    let limits = &limits::SHIPPED;
    let bytes = fixtures::at_the_limits(limits);
    let decoded = world::read_output(&bytes, 0, limits, world::End::Whole).decoded;
    assert_eq!(decoded.error, None);
    assert_eq!(decoded.insts.len(), 1);
    let inst = decoded.insts.first().expect("one install triple");
    let name = usize::try_from(limits.name).expect("name fits");
    assert_eq!(inst.name.as_bytes().len(), name);
    assert_eq!(inst.version.len(), name);
    assert_eq!(inst.architecture.as_bytes().len(), name);
    assert_eq!(decoded.tail.len(), usize::try_from(limits.tail_lines).expect("tail count fits"));
    for line in &decoded.tail {
        assert_eq!(line.len(), usize::try_from(limits.tail_line).expect("tail line fits"));
    }
    assert_eq!(decoded.cause, Cause::Unknown(inst.name.clone()));
    let most = world::read_output_metered(&bytes, limits, CEILING);
    assert!(most <= CEILING);
}
#[test]
fn a_stanza_reader_at_its_limits_holds_no_more_than_its_ceiling() {
    const CEILING: u64 = 32_000;
    let limits = &limits::SHIPPED;
    let bytes = fixtures::index_at_the_limits(limits);
    assert_eq!(bytes.len(), usize::try_from(limits.stanza).expect("stanza fits"));
    let decoded = reference::index(&bytes, world::End::Whole, limits);
    assert_eq!(decoded.error, None);
    assert_eq!(decoded.stanzas.len(), 1);
    let stanza = decoded.stanzas.first().expect("one stanza");
    let name = usize::try_from(limits.name).expect("name fits");
    assert_eq!(stanza.package.as_bytes().len(), name);
    assert_eq!(stanza.version.len(), name);
    assert_eq!(stanza.architecture.as_bytes().len(), name);
    assert_eq!(stanza.filename.len(), usize::try_from(limits.filename).expect("filename fits"));
    let most = world::read_index_metered(&bytes, limits, CEILING);
    assert!(most <= CEILING);
}
