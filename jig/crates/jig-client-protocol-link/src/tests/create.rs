use super::*;
use skein_lib::Reader;
#[test]
fn a_create_s_tasks_cross_as_pieces_within_the_request_s_bytes() {
    let limits = fixtures::limits();
    let ask = fixtures::create_ask();
    let record = translate::asks::ask(&ask, limits.wire()).unwrap();
    let w::Ask::Create(value) = &record else { panic!("create") };
    assert_eq!(value.project(), fixtures::PROJECT);
    let mut reader = Reader::new(value.tasks());
    let first = skein_codec::piece::split(&mut reader).unwrap().unwrap();
    let second = skein_codec::piece::split(&mut reader).unwrap().unwrap();
    assert_eq!(skein_codec::piece::split(&mut reader), Ok(None));
    let first = w::NewTask::decode(limits.wire(), &mut Reader::new(&first)).unwrap();
    let second = w::NewTask::decode(limits.wire(), &mut Reader::new(&second)).unwrap();
    assert_eq!(first.spec().words(), b"kept");
    assert_eq!(second.label(), 14);
    assert_eq!(translate::store::restore_ask(record, &limits), Ok(ask));
}
#[test]
fn a_new_task_keeps_each_contract_and_executor_through_the_store() {
    let limits = fixtures::limits();
    for (name, executor, contract) in [
        ("report", c::Executor::Person(c::PersonAddress::Party(u64::MAX)), c::NewContract::Report { words: 17 }),
        (
            "verdict",
            c::Executor::Procedure { connector: 5, code: 19 },
            c::NewContract::Verdict {
                choices: fixtures::list([c::VerdictChoice {
                    code: 3,
                    words: 4,
                    followups: 5,
                    fields: fixtures::list([c::FieldRule { name: b"opaque".as_slice().into(), max: 6 }]),
                }]),
            },
        ),
        (
            "change",
            c::Executor::Agent { charter: fixtures::CHARTER },
            c::NewContract::Change { connector: 5, kind: 6, words: 18 },
        ),
    ] {
        let mut task = fixtures::new_task(executor, contract);
        task.budget = c::NewBudget::Named;
        task.flags = c::TaskFlags {
            ended_by: c::EndedBy::Result,
            conversation: true,
            listed: false,
            retention: c::Retention::Kept,
        };
        let ask = c::Ask::Create { project: 7, tasks: fixtures::list([task]) };
        let wire = translate::asks::ask(&ask, limits.wire()).unwrap();
        assert_eq!(translate::store::restore_ask(wire, &limits), Ok(ask), "{name}");
    }
}

#[test]
fn a_named_deadline_keeps_its_optional_wall_through_create_propose_and_restore() {
    let limits = fixtures::limits();
    for (name, wall) in [("none", None), ("finite", Some(fixtures::DEADLINE))] {
        let task = fixtures::task_with_deadline(wall);
        for (operation, ask) in fixtures::deadline_asks(task) {
            let wire = translate::asks::ask(&ask, limits.wire()).unwrap();
            assert_eq!(translate::store::restore_ask(wire, &limits), Ok(ask), "{name}: {operation}");
        }
    }
}

#[test]
fn a_span_keeps_its_duration_through_create_propose_and_restore() {
    let limits = fixtures::limits();
    let mut task = fixtures::task_with_deadline(Some(fixtures::DEADLINE));
    task.deadline = c::NewDeadline::Span(skein_lib::Duration::from_secs(5));
    for (operation, ask) in fixtures::deadline_asks(task) {
        let wire = translate::asks::ask(&ask, limits.wire()).unwrap();
        assert_eq!(translate::store::restore_ask(wire, &limits), Ok(ask), "{operation}");
    }
}

#[test]
fn a_new_tasks_closer_crosses_the_wire_both_ways() {
    for ended_by in [c::EndedBy::Closer(c::Closer { connector: 1, code: 6 }), c::EndedBy::Standing] {
        let limits = fixtures::limits();
        let mut task = fixtures::new_task(c::Executor::Agent { charter: 1 }, c::NewContract::Report { words: 32 });
        task.flags.ended_by = ended_by;
        let ask = c::Ask::Create { project: 7, tasks: fixtures::list([task]) };
        let wire = translate::asks::ask(&ask, limits.wire()).unwrap();
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(wire.measure()));
        wire.encode(&mut out).unwrap();
        let wire = w::Ask::decode(limits.wire(), &mut Reader::new(&out.finish())).unwrap();
        assert_eq!(translate::store::restore_ask(wire, &limits), Ok(ask));
    }
}
#[test]
fn a_templates_closer_reaches_me() {
    for ended_by in [c::EndedBy::Closer(c::Closer { connector: 1, code: 6 }), c::EndedBy::Standing] {
        let limits = fixtures::limits();
        let values = jig_client_test_records::settings::settings_values();
        let mut template = None;
        for value in &values {
            if let c::SettingValue::Template(value) = value {
                template = Some(value.clone());
            }
        }
        let mut template = template.expect("template fixture");
        template.flags.conversation = false;
        template.tracked = None;
        template.flags.ended_by = ended_by;
        let value = c::SettingValue::Template(template.clone());
        let w::SettingValue::Template(wire) = translate::settings::wire_setting_value(&value, limits.wire()).unwrap()
        else {
            panic!("template");
        };
        let restored = translate::documents::me::template(&wire, &limits).unwrap();
        assert_eq!(restored, *template);
    }
}
