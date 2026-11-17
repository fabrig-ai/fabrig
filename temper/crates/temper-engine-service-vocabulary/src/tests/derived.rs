//! Store and forge carriers, checked by their owners (configuration/derived.md, 5–6).
use crate::{forge, limits, store};
use jig_protocol_config as config;
use temper_service_core::derived::{self, STOCK};

fn forge_check(own: &limits::Limits) -> Result<(), skein_forge_connection::Unusable> {
    let mut problems = config::Problems::empty();
    let settings = forge::settings(&[], skein_lib::List::with_capacity(0), None, &mut problems).unwrap();
    let mut io = config::stock();
    derived::engine(&mut io, STOCK).unwrap();
    skein_forge_connection::limits::check(&own.forge_component, &forge::shared_limits(own, &io.io), &settings)
}
#[test]
fn forge_body_carrier() {
    for payload in [STOCK.payload, STOCK.payload * 2] {
        let mut own = limits::profile(limits::Profile::Standard);
        limits::derive(&mut own, derived::Sources { payload, ..STOCK }).unwrap();
        assert_eq!(own.forge_component.body_bytes, 2 * (payload - 2) + 21);
        forge_check(&own).unwrap();
        own.forge_component.body_bytes -= 1;
        let why = forge_check(&own).unwrap_err();
        assert_eq!(
            why,
            skein_forge_connection::Unusable::BodyBytes {
                body_bytes: own.forge_component.body_bytes,
                needed: u64::from(2 * (payload - 2) + 21)
            }
        );
        let mut problems = config::Problems::empty();
        forge::problems(why, &mut problems);
        assert_eq!(
            problems.problems.get(0).unwrap().path,
            config::Path::root()
                .member(b"application")
                .member(b"limits")
                .member(b"overrides")
                .member(b"forge_component.body_bytes")
        );
        assert_eq!(
            problems.problems.get(1).unwrap().path,
            config::Path::root()
                .member(b"application")
                .member(b"limits")
                .member(b"overrides")
                .member(b"forge.client.op_bytes")
        );
    }
}
#[test]
fn forge_answer_carrier() {
    for payload in [STOCK.payload, STOCK.payload * 2] {
        let mut own = limits::profile(limits::Profile::Standard);
        let stock = own.forge.client.answer_bytes;
        limits::derive(&mut own, derived::Sources { payload, ..STOCK }).unwrap();
        let least = 2 * own.forge_component.body_bytes
            + own.forge_component.string_bytes
            + u32::try_from(skein_forge::measure::FILE_BYTES).unwrap();
        assert_eq!(own.forge.client.answer_bytes, stock.max(least));
        forge_check(&own).unwrap();
        limits::apply(&mut own, b"forge.client.answer_bytes", u64::from(least - 1)).unwrap();
        let why = forge_check(&own).unwrap_err();
        assert_eq!(
            why,
            skein_forge_connection::Unusable::PageSize { listing: skein_forge_connection::limits::Listing::PullFiles }
        );
        let mut problems = config::Problems::empty();
        forge::problems(why, &mut problems);
        assert_eq!(
            problems.problems.get(3).unwrap().path,
            config::Path::root()
                .member(b"application")
                .member(b"limits")
                .member(b"overrides")
                .member(b"forge.client.answer_bytes")
        );
        assert_eq!(
            problems.problems.get(0).unwrap().path,
            config::Path::root()
                .member(b"application")
                .member(b"limits")
                .member(b"overrides")
                .member(b"forge_component.body_bytes")
        );
    }
}
#[test]
fn decoded_read_carrier() {
    let twice = STOCK.turn * 2;
    let row = u64::try_from(size_of::<jig_domain_core::reads::Row>()).unwrap();
    let record = u64::try_from(size_of::<jig_domain_core::Record>()).unwrap();
    assert_eq!(derived::read(twice, row, record), Some(u64::from(twice) * 2));
    for turn in [STOCK.turn, jig_protocol_store::core_v6::CEILINGS.turn_record_transcript] {
        let (mut root, _, _) = crate::tests::fixtures::store_limits();
        let profile = config::stock().store;
        root.loads.transcript_bytes = turn;
        root.loads.bytes = turn + u32::try_from(size_of::<temper_engine_domain::Record>()).unwrap();
        root.loads.reply_bytes = root.loads.bytes;
        root.loads.read_bytes = derived::read(turn, row, record).unwrap();
        assert_eq!(root.loads.read_bytes, u64::from(turn) * 2);
        store::admit(&root, &root.forge, &profile, Some(turn)).unwrap();
        let least = u64::from(turn) + row + record;
        root.loads.read_bytes = least - 1;
        assert_eq!(
            store::admit(&root, &root.forge, &profile, Some(turn)).unwrap_err(),
            store::Refusal::ReadRoom { read: least - 1, needs: least }
        );
    }
    let (mut root, _, _) = crate::tests::fixtures::store_limits();
    root.loads.transcript_bytes = twice;
    root.loads.read_bytes = derived::read(twice, row, record).unwrap();
    assert_eq!(
        store::admit(&root, &root.forge, &config::stock().store, Some(twice)).unwrap_err(),
        store::Refusal::Transcript {
            bytes: twice,
            ceiling: jig_protocol_store::core_v6::CEILINGS.turn_record_transcript
        }
    );
}
