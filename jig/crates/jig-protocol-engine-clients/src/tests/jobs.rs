//! Jobs are not party-created executors (jig/docs/design/client/domain/contract/documents.md, 7).
use jig_client_protocol_wire::Record as _;
use super::fixtures as f;
use crate::translate::{asks, operations};
use jig_client_protocol_wire as w;
use skein_lib::{List, Reader, Writer};

/// Create refuses a job as an executor not offered (jig/docs/design/client/domain/contract/documents.md, 7).
#[test]
fn create_naming_a_job_is_refused_as_not_offered() {
    let limits = f::wire();
    let (ordinary, _) = f::create_ask();
    assert_eq!(operations::refused(&ordinary, &limits), Ok(None));
    let w::Ask::Create(create) = ordinary else {
        panic!("create fixture");
    };
    let mut reader = Reader::new(create.tasks.as_ref());
    let mut tasks = List::with_capacity(2);
    for job in [false, true] {
        let bytes = skein_codec::piece::split(&mut reader).unwrap().unwrap();
        let mut task = w::NewTask::decode(&limits, &mut Reader::new(&bytes)).unwrap();
        if job {
            task.executor = w::Executor::Job(w::ExecutorJob { kind: 7 });
        }
        let task = task;
        let mut writer = Writer::new(usize::try_from(task.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")).unwrap());
        task.write(&jig_client_protocol_wire::CEILINGS, &mut writer).unwrap();
        tasks.push(writer.finish()).unwrap();
    }
    let bytes = f::pieces(tasks.as_slice());
    let ask =
        w::Ask::Create(w::Create { project: create.project, tasks: bytes });
    let answer = operations::refused(&ask, &limits).unwrap().unwrap();
    let w::Final::Refused(value) = f::final_record(answer, &limits) else {
        panic!("refusal");
    };
    assert_eq!((&value.refusal), &w::Refusal::NotOffered);
    assert_eq!(value.at, None);
    assert_eq!(
        asks::ask(ask, &f::models(), &limits),
        Err(crate::translate::Untranslatable { path: w::paths::NEW_TASK_EXECUTOR })
    );
}
