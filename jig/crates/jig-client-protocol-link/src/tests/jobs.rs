//! Job task values at the client boundary (jig/docs/design/client/domain/contract.md, 5.7).
use super::*;
use skein_lib::Writer;

/// A job's kind survives restoring and encoding a client Create (jig/docs/design/client/domain/contract.md, 5.7).
#[test]
fn a_job_executor_round_trips() {
    let limits = fixtures::limits();
    let wire = w::Executor::Job(w::ExecutorJob::new(limits.wire(), w::ExecutorJobParts { kind: 7 }).unwrap());
    let executor = c::Executor::Job { kind: 7 };
    let task = fixtures::new_task(executor, c::NewContract::Report { words: 32 });
    let ask = c::Ask::Create { project: 7, tasks: fixtures::list([task]) };
    let record = translate::asks::ask(&ask, limits.wire()).unwrap();
    let w::Ask::Create(create) = &record else {
        panic!("create");
    };
    let bytes = skein_codec::piece::split(&mut Reader::new(create.tasks())).unwrap().unwrap();
    let task = w::NewTask::decode(limits.wire(), &mut Reader::new(&bytes)).unwrap();
    assert_eq!(task.executor(), &wire);
    let restored = translate::store::restore_ask(record.clone(), &limits).unwrap();
    assert_eq!(restored, ask);
    assert_eq!(translate::asks::ask(&restored, limits.wire()), Ok(record));
}

/// Delegation names the same job kind both ways (jig/docs/design/client/domain/contract.md, 5.7).
#[test]
fn a_job_delegation_kind_round_trips() {
    let limits = fixtures::limits();
    let wire = w::AuthorityExecutor::Job(
        w::AuthorityExecutorJob::new(limits.wire(), w::AuthorityExecutorJobParts { value: 7 }).unwrap(),
    );
    let kind = translate::documents::authority_executor_fields(&wire);
    assert_eq!(kind, c::AuthorityExecutor::Job(7));
    assert_eq!(translate::asks::wire_owner_authority_executor(kind, limits.wire()), Ok(wire));
}

/// Both chip projections preserve a job kind (jig/docs/design/client/domain/contract.md, 5.7).
#[test]
fn a_job_chip_keeps_its_kind_in_documents_and_refusals() {
    let limits = fixtures::limits();
    let wire =
        w::ChipExecutor::Job(w::ChipExecutorJob::new(limits.wire(), w::ChipExecutorJobParts { kind: 7 }).unwrap());
    assert_eq!(translate::answers::executor(&wire), c::ChipExecutor::Job { kind: 7 });
    let mut task = fixtures::task_record().into_parts();
    let mut reader = Reader::new(task.chip.rows());
    let mut rows = skein_lib::List::with_capacity(3);
    for index in 0_u32..3 {
        let bytes = skein_codec::piece::split(&mut reader).unwrap().unwrap();
        let mut fact = w::Fact::decode(limits.wire(), &mut Reader::new(&bytes)).unwrap();
        if index == 0 {
            let w::Fact::Chip(row) = fact else {
                panic!("chip root");
            };
            let mut row = row.into_parts();
            let mut fields = row.value.into_parts();
            fields.executor = wire.clone();
            row.value = w::ChipFields::new(limits.wire(), fields).unwrap();
            fact = w::Fact::Chip(w::ChipRow::new(limits.wire(), row).unwrap());
        }
        rows.push(fact).unwrap();
    }
    assert!(reader.is_empty());
    task.chip = fixtures::chip_from_rows(rows.as_slice());
    let task = translate::documents::task(w::Task::new(limits.wire(), task).unwrap(), &limits).unwrap();
    assert_eq!(task.chip.executor, c::ChipExecutor::Job { kind: 7 });
}

/// A job executor filter crosses the generated codec (jig/docs/design/client/domain/contract.md, 5.7).
#[test]
fn a_job_executor_filter_round_trips_the_wire() {
    let limits = fixtures::limits();
    let query = c::TaskQuery {
        project: Some(7),
        requester: None,
        root: None,
        parent: None,
        label: None,
        phase: None,
        executor: Some(c::ExecutorKind::Job),
        tracked: None,
        order: c::QueryOrder::Number,
    };
    let read = translate::reads::read(&c::Read::Tasks { query, live: None, ended: None }, limits.wire()).unwrap();
    let mut out = Writer::new(usize::try_from(read.measure()).unwrap());
    read.encode(&mut out).unwrap();
    let decoded = w::Read::decode(limits.wire(), &mut Reader::new(&out.finish())).unwrap();
    assert_eq!(decoded, read);
    let w::Read::Tasks(value) = decoded else {
        panic!("tasks query");
    };
    assert_eq!(value.query().executor(), &Some(w::ExecutorKind::Job));
}

/// A job's result bound crosses both translations (jig/docs/design/client/domain/contract.md, 5.7).
#[test]
fn a_job_contract_round_trips() {
    let limits = fixtures::limits();
    let contract = w::TaskContractDocument::Job(
        w::TaskContractDocumentJob::new(limits.wire(), w::TaskContractDocumentJobParts { bytes: 4096 }).unwrap(),
    );
    let values = jig_client_test_records::settings::settings_values();
    let mut template = None;
    for value in &values {
        match value {
            c::SettingValue::Template(value) => template = Some(value.clone()),
            c::SettingValue::Application(_)
            | c::SettingValue::Charter(_)
            | c::SettingValue::Policy(_)
            | c::SettingValue::Rules(_) => {}
        }
    }
    let template = template.expect("template fixture");
    let value = translate::settings::wire_setting_value(&c::SettingValue::Template(template), limits.wire()).unwrap();
    let w::SettingValue::Template(value) = value else {
        panic!("template");
    };
    let mut value = value.into_parts();
    value.contract = contract;
    let wire = w::SettingValue::Template(Box::new(w::Template::new(limits.wire(), value).unwrap()));
    let domain = translate::settings::restore_setting_value(&wire, &limits).unwrap();
    let c::SettingValue::Template(template) = &domain else {
        panic!("template");
    };
    assert_eq!(template.contract, c::TaskContractDocument::Job { bytes: 4096 });
    assert_eq!(translate::settings::wire_setting_value(&domain, limits.wire()), Ok(wire));
}
