//! Accounts watch records preserve every terminal outcome and optional wall.
use jig_client_protocol_wire as w;
use skein_lib::{Reader, Writer};

#[test]
fn whole_accounts_round_trip_with_every_terminal_outcome() {
    let limits = jig_client_test_records::limits();
    let ends = std::iter::once(None).chain(jig_client_test_records::account_end_records().into_iter().map(Some));
    for end in ends {
        let record = jig_client_test_records::accounts_record(end);
        let mut out = Writer::new(skein_lib::length::usize_of(record.measure()));
        record.encode(&mut out).expect("measured accounts record");
        let bytes = out.finish();
        assert_eq!(w::Accounts::decode(&limits, &mut Reader::new(&bytes)), Ok(record));
    }
}

#[test]
fn window_rates_round_trip_absent_zero_and_the_u32_ceiling() {
    let limits = jig_client_test_records::limits();
    let mut rates = Vec::new();
    for fact in &jig_client_test_records::account_fact_records() {
        if let w::AccountFactValue::Window(window) = fact.value() {
            let mut writer = Writer::new(usize::try_from(window.measure()).unwrap());
            window.encode(&mut writer).unwrap();
            let decoded = w::AccountWindow::decode(&limits, &mut Reader::new(&writer.finish())).unwrap();
            assert_eq!(&decoded, window);
            rates.push(*decoded.rate());
        }
    }
    assert_eq!(rates, [None, Some(0), Some(600), Some(u32::MAX)]);
}
