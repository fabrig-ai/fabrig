//! Task settings revisions survive the complete task record's bytes.
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{Reader, Writer};

#[test]
fn task_settings_round_trip_with_and_without_a_run_revision() {
    for run in [None, Some(6)] {
        let mut parts = jig_client_test_records::task_record();
        parts.settings = Some(w::TaskSettings { task: 7, run, newest: 9 });
        let task = parts;
        let mut out = Writer::new(skein_lib::length::usize_of(
            task.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
        ));
        task.write(&jig_client_protocol_wire::CEILINGS, &mut out).expect("encode task");
        let bytes = out.finish();
        let mut input = Reader::new(&bytes);
        assert_eq!(w::Task::decode(&w::CEILINGS, &mut input), Ok(task));
        assert!(input.is_empty());
    }
}
