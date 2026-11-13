//! Agent task reads use task relationships, never a person's visibility
//! (jig/docs/design/domain/engine/executors.md, 3).

use super::{Destination, Loaded, Pending, Range, Scope, TaskTextField};
use crate::{CallKey, Core, Held, Limits, Now, Request};
use alloc::boxed::Box;
pub use jig_domain_core_vocabulary::reads::run_task::*;
use jig_domain_tasks as tasks;
use skein_lib::{Env, Id, Queue, ReplyTo};

fn related(core: &Core, caller: u64, row: &tasks::TaskRecord) -> bool {
    let Some(owner) = core.tasks.task(caller) else { return false };
    if owner.project != row.project {
        return false;
    }
    if row.requester == tasks::Party::Task(caller) {
        return true;
    }
    match row.phase {
        tasks::Phase::Ended(_) => return false,
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {}
    }
    caller == row.number
        || owner.requester == tasks::Party::Task(row.number)
        || owner.delegates.contains(&row.number)
        || owner.references.contains(&row.number)
        || match row.escalation {
            tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(task), .. } => task == caller,
            tasks::Escalation::Waiting { .. }
            | tasks::Escalation::Unheld { .. }
            | tasks::Escalation::Routing { .. }
            | tasks::Escalation::Rejected { .. } => false,
        }
}

fn dependency(core: &Core, caller: u64, row: &tasks::TaskRecord) -> bool {
    let Some(owner) = core.tasks.task(caller) else { return false };
    let tasks::Party::Task(requester) = owner.requester else { return false };
    let Some(requester) = core.tasks.task(requester) else { return false };
    owner.project == row.project && requester.project == row.project && requester.dependencies.contains(&row.number)
}

fn descendant(
    core: &Core,
    caller: u64,
    row: &tasks::TaskRecord,
    linked: &[Box<tasks::TaskRecord>],
    depth: u32,
) -> bool {
    let Some(owner) = core.tasks.task(caller) else { return false };
    let mut child = row;
    for _ in 0..depth {
        if child.project != owner.project {
            return false;
        }
        let number = match child.requester {
            tasks::Party::Task(number) => number,
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => return false,
        };
        let mut parent = core.tasks.task(number);
        if parent.is_none() {
            for record in linked {
                if record.number == number {
                    parent = Some(record);
                    break;
                }
            }
        }
        let Some(parent) = parent else { return false };
        if parent.project != owner.project || parent.depth.checked_add(1) != Some(child.depth) {
            return false;
        }
        if parent.number == caller {
            return true;
        }
        child = parent;
    }
    false
}

fn value(
    core: &Core,
    limits: &Limits,
    caller: u64,
    row: &tasks::TaskRecord,
    read: Read,
    linked: &[Box<tasks::TaskRecord>],
) -> Result<TaskRead, tasks::Refusal> {
    let Read { field, from, room, .. } = read;
    if !(related(core, caller, row)
        || dependency(core, caller, row)
        || descendant(core, caller, row, linked, limits.tasks.depth))
    {
        return Err(tasks::Refusal::Reference);
    }
    let result_pending = field == TaskTextField::Result
        && match row.phase {
            tasks::Phase::Ended(_) => false,
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                true
            }
        };
    if result_pending && from != 0 {
        return Err(tasks::Refusal::Unknown);
    }
    let field = if result_pending { TaskTextField::Specification } else { field };
    let source = match field {
        TaskTextField::Specification => row.spec.words.as_ref(),
        TaskTextField::Result => super::task_text::report(row).ok_or(tasks::Refusal::Unknown)?,
        TaskTextField::Instructions => return Err(tasks::Refusal::Unknown),
    };
    let words = super::task_text::chunk(row.number, field, source, from, room.min(limits.call_answer_bytes))
        .or(Err(tasks::Refusal::Unknown))?;
    let state = task_state(row);
    let hold = task_hold(limits, row)?;
    Ok(TaskRead {
        task: row.number,
        state,
        spec_revision: row.revisions.spec,
        dependencies: row.dependencies.clone(),
        questions: row.questions.clone(),
        words,
        verdict_code: verdict_code(row, field),
        result_pending,
        hold,
    })
}

fn verdict_code(row: &tasks::TaskRecord, field: TaskTextField) -> Option<u32> {
    if field != TaskTextField::Result {
        return None;
    }
    match &row.phase {
        tasks::Phase::Ended(
            tasks::Ending::Done(tasks::TaskResult::Verdict { code, .. })
            | tasks::Ending::Cancelled { result: Some(tasks::TaskResult::Verdict { code, .. }), .. },
        ) => Some(*code),
        tasks::Phase::Ended(_)
        | tasks::Phase::Waiting
        | tasks::Phase::Active(_)
        | tasks::Phase::Closing(_)
        | tasks::Phase::Held { .. } => None,
    }
}

fn task_state(row: &tasks::TaskRecord) -> TaskState {
    match &row.phase {
        tasks::Phase::Waiting => TaskState::Waiting,
        tasks::Phase::Active(active) => TaskState::Active(*active),
        tasks::Phase::Closing(stage) => TaskState::Closing(stage.stage),
        tasks::Phase::Held { .. } => TaskState::Held,
        tasks::Phase::Ended(ending) => TaskState::Ended(match ending {
            tasks::Ending::Done(_) => tasks::Status::Done,
            tasks::Ending::Failed { .. } => tasks::Status::Failed,
            tasks::Ending::Cancelled { .. } => tasks::Status::Cancelled,
        }),
    }
}

fn task_hold(limits: &Limits, row: &tasks::TaskRecord) -> Result<Option<TaskHold>, tasks::Refusal> {
    Ok(match crate::documents::task_hold(limits, row) {
        Ok(Some(hold)) => Some(TaskHold {
            revision: hold.revision,
            reason: hold.reason,
            words: match hold.words {
                Some(text) => Some(text.bytes),
                None => None,
            },
        }),
        Ok(None) => None,
        Err(_) => return Err(tasks::Refusal::Busy),
    })
}

fn answer(to: ReplyTo, key: CallKey, value: Result<TaskRead, tasks::Refusal>) -> Queue<Request> {
    let mut out = Queue::uncapped();
    out.push(Request::Now(Box::new(Now::TaskRead { to, key, answer: Box::new(value) })));
    out
}

pub(crate) fn run(core: &mut Core, env: &Env<Limits>, to: ReplyTo, key: CallKey, read: Read) -> Queue<Request> {
    if let Some(part) = core.authorize_tool(key, crate::ToolKind::Read) {
        let mut out = Queue::uncapped();
        out.push(Request::Now(Box::new(Now::EffectAnswer { to, key, part })));
        return out;
    }
    if let Some(row) = core.tasks.task(read.task) {
        return answer(to, key, value(core, &env.limits, key.task, row, read, &[]));
    }
    load(core, env, to, key, read)
}

fn load(core: &mut Core, env: &Env<Limits>, to: ReplyTo, key: CallKey, read: Read) -> Queue<Request> {
    if core.reads.len() >= env.limits.load_slots {
        return answer(to, key, Err(tasks::Refusal::Busy));
    }
    let Read { task, field, from, .. } = read;
    let range = Range {
        party: 0,
        backwards: false,
        scope: if read.joined || field == TaskTextField::Result {
            Scope::Task { task }
        } else {
            Scope::TaskText { task, field, from }
        },
        after: None,
        rows: 1,
        bytes: env.limits.load_bytes,
    };
    let pending = Pending {
        destination: Destination::RunTask { to, key, read },
        sign_in: 0,
        party: 0,
        range: range.clone(),
        tasks: None,
    };
    let load = core.reads.insert(pending).expect("read room checked").token();
    let mut out = Queue::uncapped();
    out.push(Request::Held(Box::new(Held::ReadLoad { load, range: Box::new(range) })));
    out
}

pub(super) fn loaded(core: &mut Core, env: &Env<Limits>, name: Id<Pending>, result: Loaded) -> Queue<Request> {
    let pending = core.reads.get_mut(name).expect("pending task read");
    let Destination::RunTask { to, key, read } = core::mem::replace(&mut pending.destination, Destination::Done) else {
        unreachable!("task read")
    };
    core.reads.retire(name);
    if !core.current_proof(key.task, key.attempt) {
        return Queue::uncapped();
    }
    if let Some(part) = core.authorize_tool(key, crate::ToolKind::Read) {
        let mut out = Queue::uncapped();
        out.push(Request::Now(Box::new(Now::EffectAnswer { to, key, part })));
        return out;
    }
    match result {
        Loaded::Page { rows, .. } => {
            for row in rows {
                if let crate::Record::Tasks(tasks::Stored::Ended(owner)) = &row.value
                    && owner.number == read.task
                {
                    let value = value(core, &env.limits, key.task, owner, read, &row.linked);
                    if value == Err(tasks::Refusal::Reference)
                        && read.field == TaskTextField::Specification
                        && !read.joined
                    {
                        return load(core, env, to, key, Read { joined: true, ..read });
                    }
                    return answer(to, key, value);
                }
            }
            answer(to, key, Err(tasks::Refusal::Reference))
        }
        Loaded::Busy | Loaded::TooLinked { .. } => answer(to, key, Err(tasks::Refusal::Busy)),
        Loaded::Gone | Loaded::TurnProblem { .. } => answer(to, key, Err(tasks::Refusal::Unknown)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use skein_lib::length::len_u64;

    fn in_document_room(
        core: &Core,
        limits: &Limits,
        caller: u64,
        row: &tasks::TaskRecord,
        field: TaskTextField,
        from: u64,
        linked: &[Box<tasks::TaskRecord>],
    ) -> Result<TaskRead, tasks::Refusal> {
        value(
            core,
            limits,
            caller,
            row,
            Read { joined: false, task: row.number, field, from, room: limits.documents.text_bytes },
            linked,
        )
    }

    #[test]
    fn task_read_room_is_independent_of_the_clients_window() {
        let (core, mut limits) = crate::tests::make_tree();
        limits.documents.text_bytes = 4;
        let mut row = core.tasks.task(2).expect("delegate").clone();
        row.spec.words = b"a specification past the client window".as_slice().into();
        let read = value(
            &core,
            &limits,
            1,
            &row,
            Read { joined: false, task: row.number, field: TaskTextField::Specification, from: 0, room: 64 },
            &[],
        )
        .expect("related task");
        assert_eq!(read.words.bytes, row.spec.words);
        assert_eq!(read.words.next, None);
    }

    #[test]
    fn task_read_words_are_capped_by_call_answers_and_the_named_room() {
        let (core, mut limits) = crate::tests::make_tree();
        limits.documents.text_bytes = 4;
        let mut row = core.tasks.task(2).expect("delegate").clone();
        row.spec.words = b"a specification past both limits".as_slice().into();
        for (room, answer, expected) in [(64, 12, 12), (8, 64, 8)] {
            limits.call_answer_bytes = answer;
            let read = value(
                &core,
                &limits,
                1,
                &row,
                Read { task: row.number, field: TaskTextField::Specification, from: 0, room, joined: false },
                &[],
            )
            .expect("related task");
            assert_eq!(read.words.bytes.len(), expected);
            assert_eq!(read.words.next, Some(u64::try_from(expected).unwrap()));
        }
    }

    #[test]
    fn task_read_answer_bound_covers_the_largest_read() {
        let (core, mut limits) = crate::tests::make_tree();
        limits.documents.text_bytes = 4;
        limits.call_answer_bytes = 64;
        let mut row = core.tasks.task(2).expect("delegate").clone();
        row.spec.words = skein_lib::bytes::zeroed(64);
        limits.tasks.dependencies = 2;
        row.dependencies = Box::new([1; 2]);
        let credit =
            tasks::QuestionCredit { number: 1, answerer: tasks::Party::Task(1), asked: tasks::Party::Task(1), line: 2 };
        limits.tasks.inbox_messages = 2;
        row.questions = Box::new([credit.clone(), credit]);
        row.phase = tasks::Phase::Held {
            was: tasks::Was::Waiting,
            why: tasks::Hold::Procedure,
            source: Some(Box::new(tasks::HoldSource::Account {
                connector: 1,
                entry: None,
                words: skein_lib::bytes::zeroed(usize::try_from(tasks::HOLD_ACCOUNT_BYTES).unwrap()),
            })),
        };
        let read = value(
            &core,
            &limits,
            1,
            &row,
            Read { joined: false, task: row.number, field: TaskTextField::Specification, from: 0, room: u32::MAX },
            &[],
        )
        .expect("largest task read");
        assert_eq!(read.words.bytes.len(), 64);
        let retained = u64::try_from(size_of::<Result<TaskRead, tasks::Refusal>>()).unwrap() + owned_bytes(&read);
        assert_eq!(answer_bound(&limits), Some(retained));
        assert_eq!(answer_carrier_bound(&limits), retained.checked_sub(64));
    }

    #[test]
    fn live_result_reads_return_only_the_first_specification_chunk() {
        let (core, mut limits) = crate::tests::make_tree();
        limits.documents.text_bytes = 4;
        let mut row = core.tasks.task(2).expect("delegate").clone();
        row.spec.words = b"a\xc3\xa9bcdef".as_slice().into();
        row.revisions.spec = 2;
        row.revisions.hold = 7;
        row.dependencies = Box::new([3]);
        row.questions = Box::new([tasks::QuestionCredit {
            number: 103,
            answerer: tasks::Party::Task(1),
            asked: tasks::Party::Task(1),
            line: 106,
        }]);
        for phase in [
            tasks::Phase::Waiting,
            tasks::Phase::Active(tasks::Active::Due),
            tasks::Phase::Held {
                was: tasks::Was::Waiting,
                why: tasks::Hold::Procedure,
                source: Some(Box::new(tasks::HoldSource::Account {
                    connector: 2,
                    entry: None,
                    words: b"amend authority and release".as_slice().into(),
                })),
            },
        ] {
            row.phase = phase;
            let mut spec =
                in_document_room(&core, &limits, 1, &row, TaskTextField::Specification, 0, &[]).expect("specification");
            assert!(!spec.result_pending);
            assert_eq!(spec.words.field, TaskTextField::Specification);
            assert_eq!(spec.words.from, 0);
            assert_eq!(spec.words.bytes.as_ref(), b"a\xc3\xa9b");
            assert_eq!(spec.words.next, Some(4));
            assert_eq!(spec.verdict_code, None);
            spec.result_pending = true;
            assert_eq!(
                in_document_room(&core, &limits, 1, &row, TaskTextField::Result, 0, &[]),
                Ok(spec),
                "live result keeps the specification header and chunk"
            );
            assert_eq!(
                in_document_room(&core, &limits, 1, &row, TaskTextField::Result, 1, &[]),
                Err(tasks::Refusal::Unknown)
            );
        }
    }

    #[test]
    fn ended_tasks_without_reports_do_not_fall_back_to_specifications() {
        let (core, limits) = crate::tests::make_tree();
        let mut row = core.tasks.task(2).expect("delegate").clone();
        for ending in [
            tasks::Ending::Failed { reason: b"failed".as_slice().into() },
            tasks::Ending::Cancelled {
                by: tasks::Party::Task(1),
                at: skein_lib::Wall::EPOCH,
                reason: b"cancelled".as_slice().into(),
                result: None,
            },
        ] {
            row.phase = tasks::Phase::Ended(ending);
            assert_eq!(
                in_document_room(&core, &limits, 1, &row, TaskTextField::Result, 0, &[]),
                Err(tasks::Refusal::Unknown)
            );
            let spec = in_document_room(&core, &limits, 1, &row, TaskTextField::Specification, 0, &[])
                .expect("ended specification");
            assert!(!spec.result_pending);
        }
    }

    #[test]
    fn result_reads_keep_verdict_codes_outside_reason_offsets() {
        let (core, mut limits) = crate::tests::make_tree();
        limits.documents.text_bytes = 5; // The UTF-8 reason is two five-byte windows.
        let mut row = core.tasks.task(2).expect("delegate").clone();
        row.phase = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Verdict {
            code: 2,
            fields: Box::new([]),
            words: b"caf\xc3\xa9 bold".as_slice().into(),
        }));
        for (from, bytes, next) in [(0, b"caf\xc3\xa9".as_slice(), Some(5)), (5, b" bold".as_slice(), None)] {
            let read = value(
                &core,
                &limits,
                1,
                &row,
                Read {
                    joined: false,
                    task: row.number,
                    field: TaskTextField::Result,
                    from,
                    room: limits.documents.text_bytes,
                },
                &[],
            )
            .expect("requester result");
            assert!(!read.result_pending);
            assert_eq!(read.verdict_code, Some(2));
            assert_eq!(read.words.bytes.as_ref(), bytes);
            assert_eq!(read.words.next, next);
            assert_eq!(read.words.total, 10);
        }
        row.phase = tasks::Phase::Ended(tasks::Ending::Cancelled {
            by: tasks::Party::Task(1),
            at: skein_lib::Wall::EPOCH,
            reason: b"stopped".as_slice().into(),
            result: Some(tasks::TaskResult::Verdict {
                code: 2,
                fields: Box::new([]),
                words: b"bold".as_slice().into(),
            }),
        });
        assert_eq!(
            in_document_room(&core, &limits, 1, &row, TaskTextField::Result, 0, &[])
                .expect("kept verdict")
                .verdict_code,
            Some(2)
        );
        let spec =
            in_document_room(&core, &limits, 1, &row, TaskTextField::Specification, 0, &[]).expect("requester spec");
        assert!(!spec.result_pending);
        assert_eq!(spec.verdict_code, None);
        assert_eq!(
            in_document_room(&core, &limits, 3, &row, TaskTextField::Result, 0, &[]),
            Err(tasks::Refusal::Reference)
        );
        row.phase =
            tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: b"bold".as_slice().into() }));
        let report = in_document_room(&core, &limits, 1, &row, TaskTextField::Result, 0, &[]).expect("report");
        assert_eq!(report.verdict_code, None);
        assert!(!report.result_pending);
    }

    #[test]
    fn task_reads_keep_question_credits_after_delivery_is_consumed() {
        let (core, mut limits) = crate::tests::make_tree();
        let mut row = core.tasks.task(2).expect("delegate").clone();
        row.inbox = Box::new([]);
        limits.tasks.inbox_messages = 2;
        let credit = tasks::QuestionCredit {
            number: 103,
            answerer: tasks::Party::Task(1),
            asked: tasks::Party::Task(1),
            line: 106,
        };
        row.questions = Box::new([credit.clone(), tasks::QuestionCredit { number: 104, ..credit.clone() }]);
        let read =
            in_document_room(&core, &limits, 1, &row, TaskTextField::Specification, 0, &[]).expect("requester read");
        assert_eq!(read.questions, row.questions);
        let inline = u64::try_from(size_of::<Result<TaskRead, tasks::Refusal>>()).expect("carrier");
        assert!(answer_bound(&limits).expect("bounded answer") >= inline + owned_bytes(&read));
        let mut without = read.clone();
        without.questions = Box::new([]);
        assert_eq!(
            owned_bytes(&read) - owned_bytes(&without),
            u64::try_from(size_of_val(row.questions.as_ref())).expect("credit bytes")
        );
        assert_eq!(
            in_document_room(&core, &limits, 3, &row, TaskTextField::Specification, 0, &[]),
            Err(tasks::Refusal::Reference)
        );
    }

    #[test]
    fn task_reads_preserve_connector_words_and_require_a_relationship() {
        let (core, mut limits) = crate::tests::make_tree();
        limits.documents.text_bytes = 4;
        let mut row = core.tasks.task(2).expect("delegate").clone();
        row.spec.words = b"a\xc3\xa9bcdef".as_slice().into();
        row.phase = tasks::Phase::Held {
            was: tasks::Was::Waiting,
            why: tasks::Hold::Procedure,
            source: Some(Box::new(tasks::HoldSource::Account {
                connector: 2,
                entry: None,
                words: b"missing grant; amend authority and release".as_slice().into(),
            })),
        };
        row.revisions.hold = 7;
        let first =
            in_document_room(&core, &limits, 1, &row, TaskTextField::Specification, 0, &[]).expect("direct delegate");
        assert_eq!(first.words.bytes.as_ref(), b"a\xc3\xa9b");
        assert_eq!(first.words.next, Some(4));
        let hold = first.hold.expect("connector hold");
        assert_eq!(hold.revision, 7);
        assert_eq!(hold.words.as_deref(), Some(b"missing grant; amend authority and release".as_slice()));
        assert!(
            in_document_room(&core, &limits, 1, &row, TaskTextField::Specification, 2, &[]).is_err(),
            "continuation cannot split UTF-8"
        );
        assert_eq!(
            in_document_room(&core, &limits, 3, &row, TaskTextField::Specification, 0, &[]),
            Err(tasks::Refusal::Reference)
        );
        row.escalation = tasks::Escalation::Waiting {
            revision: 7,
            holder: tasks::EscalationHolder::Task(3),
            entry: 1,
            since: skein_lib::Wall::EPOCH,
        };
        assert!(
            in_document_room(&core, &limits, 3, &row, TaskTextField::Specification, 4, &[]).is_ok(),
            "escalation holder can read its subject"
        );
        row.phase = tasks::Phase::Ended(tasks::Ending::Failed { reason: b"ended".as_slice().into() });
        assert!(
            in_document_room(&core, &limits, 1, &row, TaskTextField::Specification, 4, &[]).is_ok(),
            "retained requester remains authorized"
        );
        assert_eq!(
            in_document_room(&core, &limits, 3, &row, TaskTextField::Specification, 4, &[]),
            Err(tasks::Refusal::Reference)
        );
    }

    fn retained_fixture() -> (Core, Limits, super::super::Row) {
        let (mut core, limits) = crate::tests::make_tree();
        core.proofs
            .insert(
                1,
                crate::RunProof {
                    task: 1,
                    attempt: 9,
                    conversation: 1,
                    credentials: Box::new([]),
                    accounts: Box::new([]),
                    placement: None,
                    account_failure: None,
                    account_terminal: false,
                    window: None,
                    crossed: false,
                    host: crate::fleet::HostKind::Engine,
                    offered: None,
                    outstanding: Box::new([]),
                    turn: None,
                    terminal: None,
                },
            )
            .expect("goal's later attempt");
        let mut change = core.tasks.task(2).expect("change").clone();
        change.number = 20;
        change.phase = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report {
            words: b"Left: producer follow-up accepted".as_slice().into(),
        }));
        let mut producer = change.clone();
        producer.number = 21;
        producer.requester = tasks::Party::Task(20);
        producer.depth = 2;
        producer.spec.words = b"produce a patch".as_slice().into();
        producer.phase = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report {
            words: b"Left: document the retained report".as_slice().into(),
        }));
        let mut row = crate::tests::stored_row(crate::Record::Tasks(tasks::Stored::Ended(Box::new(producer))), None);
        row.ancestors = Box::new([20, 1]);
        row.linked = Box::new([Box::new(change), Box::new(core.tasks.task(1).expect("goal").clone())]);
        (core, limits, row)
    }

    fn retained_load(core: &mut Core, env: &Env<Limits>, task: u64, from: u64) -> Id<Pending> {
        let key = CallKey { task: 1, attempt: 9, completion: 1, position: 0 };
        let mut out = run(
            core,
            env,
            ReplyTo::new(skein_lib::Token::new(90)),
            key,
            Read { joined: false, task, field: TaskTextField::Result, from, room: env.limits.documents.text_bytes },
        );
        let Some(Request::Held(held)) = out.pop() else { panic!("retained load") };
        let Held::ReadLoad { load, range } = *held else { panic!("store read") };
        assert_eq!(range.scope, Scope::Task { task }, "load includes requester evidence");
        assert!(out.pop().is_none(), "loading decides nothing");
        Id::from_token(load)
    }

    fn retained_reply(
        core: &mut Core,
        limits: &Limits,
        row: super::super::Row,
        from: u64,
    ) -> Result<TaskRead, tasks::Refusal> {
        let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: *limits };
        let crate::Record::Tasks(tasks::Stored::Ended(owner)) = &row.value else { panic!("retained task") };
        let load = retained_load(core, &env, owner.number, from);
        let mut out = loaded(core, &env, load, Loaded::Page { rows: Box::new([row]), next: None });
        let Some(Request::Now(now)) = out.pop() else { panic!("unrecorded task read answer") };
        let Now::TaskRead { answer, .. } = *now else { panic!("task read") };
        assert!(out.pop().is_none(), "read changes no durable state");
        core.reclaim();
        *answer
    }

    fn specification_load(core: &mut Core, env: &Env<Limits>, task: u64) -> Id<Pending> {
        let key = CallKey { task: 1, attempt: 9, completion: 1, position: 0 };
        let mut out = run(
            core,
            env,
            ReplyTo::new(skein_lib::Token::new(90)),
            key,
            Read {
                joined: false,
                task,
                field: TaskTextField::Specification,
                from: 0,
                room: env.limits.documents.text_bytes,
            },
        );
        let Some(Request::Held(held)) = out.pop() else { panic!("text load") };
        let Held::ReadLoad { load, range } = *held else { panic!("store read") };
        assert_eq!(range.scope, Scope::TaskText { task, field: TaskTextField::Specification, from: 0 });
        assert!(out.pop().is_none());
        Id::from_token(load)
    }

    fn joined_specification_load(core: &mut Core, env: &Env<Limits>, row: &super::super::Row) -> Id<Pending> {
        let load = specification_load(core, env, PRODUCER);
        let text = crate::tests::stored_row(row.value.clone(), None);
        let mut out = loaded(core, env, load, Loaded::Page { rows: Box::new([text]), next: None });
        let Some(Request::Held(held)) = out.pop() else { panic!("unproved text needs a joined load") };
        let Held::ReadLoad { load, range } = *held else { panic!("joined read") };
        assert_eq!(range.scope, Scope::Task { task: PRODUCER });
        assert!(out.pop().is_none());
        Id::from_token(load)
    }

    fn read_answer(mut out: Queue<Request>) -> Result<TaskRead, tasks::Refusal> {
        let Some(Request::Now(now)) = out.pop() else { panic!("task answer, not another load") };
        let Now::TaskRead { answer, .. } = *now else { panic!("task read") };
        assert!(out.pop().is_none());
        *answer
    }

    #[test]
    fn a_goal_reads_a_live_grandchilds_specification_and_pending_result() {
        let (mut core, limits, _) = retained_fixture();
        report_delegate(&mut core, &limits, CHANGE, GOAL, change_contract(&limits));
        report_delegate(&mut core, &limits, PRODUCER, CHANGE, change_contract(&limits));
        let row = core.tasks.task(PRODUCER).expect("live producer");
        let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
        let expected = (row.spec.words.clone(), task_state(row));
        for field in [TaskTextField::Specification, TaskTextField::Result] {
            let out = run(
                &mut core,
                &env,
                ReplyTo::new(skein_lib::Token::new(90)),
                CallKey { task: GOAL, attempt: 9, completion: 1, position: 0 },
                Read { joined: false, task: PRODUCER, field, from: 0, room: limits.call_answer_bytes },
            );
            let read = read_answer(out).expect("live grandchild read");
            assert_eq!(read.result_pending, field == TaskTextField::Result);
            assert_eq!((read.words.bytes, read.state), expected);
        }
    }

    #[test]
    fn retained_specifications_load_requester_proof_only_when_needed() {
        for direct in [false, true] {
            let (mut core, limits, mut row) = retained_fixture();
            if direct {
                let crate::Record::Tasks(tasks::Stored::Ended(owner)) = &mut row.value else { panic!("owner") };
                owner.requester = tasks::Party::Task(GOAL);
                owner.depth = 1;
                row.linked = Box::new([]);
            }
            let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
            let load = if direct {
                specification_load(&mut core, &env, PRODUCER)
            } else {
                joined_specification_load(&mut core, &env, &row)
            };
            let read = read_answer(loaded(&mut core, &env, load, Loaded::Page { rows: Box::new([row]), next: None }))
                .expect("proven retained specification");
            assert_eq!(read.words.bytes.as_ref(), b"produce a patch");
            assert_eq!(read.words.field, TaskTextField::Specification);
            assert_eq!(read.state, TaskState::Ended(tasks::Status::Done));
        }
    }

    #[test]
    fn retained_specification_proof_keeps_load_bounds() {
        for result in [Loaded::Busy, Loaded::TooLinked { joins: 1 }] {
            let (mut core, limits, mut row) = retained_fixture();
            let mut env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
            let load = joined_specification_load(&mut core, &env, &row);
            assert_eq!(read_answer(loaded(&mut core, &env, load, result)), Err(tasks::Refusal::Busy));
            core.reclaim();
            let load = specification_load(&mut core, &env, PRODUCER);
            row.linked = Box::new([]);
            env.limits.load_slots = 0;
            assert_eq!(
                read_answer(loaded(&mut core, &env, load, Loaded::Page { rows: Box::new([row]), next: None })),
                Err(tasks::Refusal::Busy),
                "the second load must acquire a slot"
            );
        }
    }

    #[test]
    fn task_reads_admit_only_the_immediate_requesters_dependencies() {
        for fault in 0_u8..8 {
            let (mut core, limits, _) = retained_fixture();
            report_delegate(&mut core, &limits, CHANGE, GOAL, change_contract(&limits));
            report_delegate(&mut core, &limits, PRODUCER, CHANGE, change_contract(&limits));
            let mut rows = [GOAL, 2, 3, 4, CHANGE, PRODUCER, PRODUCER]
                .map(|number| core.tasks.task(number).expect("fixture task").clone());
            rows.get_mut(6).expect("sibling").number = 22;
            let requester = rows.get_mut(4).expect("immediate requester");
            requester.dependencies = Box::new([4]);
            match fault {
                0 => {}
                1 => requester.dependencies = Box::new([3]), // Own dependency alone grants no read.
                2 => requester.dependencies = Box::new([]),  // Grandparent's dependency alone grants no read.
                3 => requester.project = 2,
                4 => rows.get_mut(5).expect("caller").project = 2,
                5 => rows.get_mut(3).expect("target").project = 2,
                6 => rows.get_mut(5).expect("caller").requester = tasks::Party::Person(1),
                7 => rows.get_mut(5).expect("caller").requester = tasks::Party::Task(99),
                _ => panic!("fixture fault in 0..8"),
            }
            rows.get_mut(0).expect("grandparent").dependencies = if fault == 2 { Box::new([4]) } else { Box::new([]) };
            rows.get_mut(5).expect("caller").dependencies = if fault == 1 { Box::new([4]) } else { Box::new([]) };
            core.tasks = tasks::Domain::new(
                &limits.tasks,
                tasks::Kept { revisions: limits.settings.revisions, versions: limits.settings.versions },
                17,
                core.current.newest(),
                crate::charters::models(&core.current, &core.settings, &limits),
                crate::charters::efforts(&core.settings),
            );
            for row in rows {
                task_event(&mut core, &limits, tasks::Event::Restore { record: tasks::Stored::Live(Box::new(row)) });
            }
            let row = core.tasks.task(4).expect("live dependency");
            let read = in_document_room(&core, &limits, PRODUCER, row, TaskTextField::Specification, 0, &[]);
            if fault == 0 {
                let mut specification = read.expect("requester's dependency");
                assert_eq!(specification.words.bytes, row.spec.words);
                specification.result_pending = true;
                assert_eq!(
                    in_document_room(&core, &limits, PRODUCER, row, TaskTextField::Result, 0, &[]),
                    Ok(specification),
                    "a live dependency's result read answers its specification"
                );
                let sibling = core.tasks.task(22).expect("sibling without dependency");
                assert_eq!(
                    in_document_room(&core, &limits, PRODUCER, sibling, TaskTextField::Specification, 0, &[]),
                    Err(tasks::Refusal::Reference)
                );
            } else {
                assert_eq!(read, Err(tasks::Refusal::Reference), "fault {fault}");
            }
            let mut retained = row.clone();
            retained.phase = tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report {
                words: b"dependency report".as_slice().into(),
            }));
            for field in [TaskTextField::Specification, TaskTextField::Result] {
                let read = in_document_room(&core, &limits, PRODUCER, &retained, field, 0, &[]);
                if fault == 0 {
                    let expected = if field == TaskTextField::Specification {
                        retained.spec.words.as_ref()
                    } else {
                        b"dependency report"
                    };
                    assert_eq!(read.expect("retained requester's dependency").words.bytes.as_ref(), expected);
                } else {
                    assert_eq!(read, Err(tasks::Refusal::Reference), "retained fault {fault}, {field:?}");
                }
            }
        }
    }

    #[test]
    fn goal_rereads_retained_producer_reports() {
        const REPORT: &[u8] = b"Left: document the retained report";
        for direct in [false, true] {
            let mut report = skein_lib::Writer::new(REPORT.len());
            let mut from = 0;
            for _ in 0..REPORT.len() {
                let (mut core, mut limits, mut row) = retained_fixture();
                limits.documents.text_bytes = 8;
                if direct {
                    let crate::Record::Tasks(tasks::Stored::Ended(producer)) = &mut row.value else {
                        panic!("producer")
                    };
                    producer.requester = tasks::Party::Task(1);
                    producer.depth = 1;
                    row.linked = Box::new([]);
                }
                let read = retained_reply(&mut core, &limits, row, from)
                    .expect("goal may reread its ended producer through the ended change");
                assert_eq!(read.words.field, TaskTextField::Result);
                assert_eq!(read.words.total, len_u64(REPORT.len()));
                assert_eq!(read.words.from, from);
                report.put(&read.words.bytes).expect("report chunks fit the source length");
                match read.words.next {
                    Some(next) => {
                        assert!(next > from);
                        from = next;
                    }
                    None => break,
                }
            }
            assert_eq!(report.finish().as_ref(), REPORT, "complete report, never specification bytes");
        }
    }

    const GOAL: u64 = 1;
    const DELEGATE_TEMPLATE: u64 = 2;
    const FIRST_ATTEMPT: u64 = 1;
    const CHANGE: u64 = 20;
    const PRODUCER: u64 = 21;
    const CHANGE_REPORT: &[u8] = b"Change complete\n## Left\n- retain the caf\xc3\xa9 review\n";
    const PRODUCER_REPORT: &[u8] = b"Producer complete\n## Left\n- check the caf\xc3\xa9 follow-up\n";
    const REVIEW_REPORT: &[u8] = b"Review accepted\n## Left\n- explain the caf\xc3\xa9 check\n";
    const CHUNK_BYTES: u32 = 6; // The producer/review character at byte 41 crosses a window boundary.

    fn change_contract(limits: &Limits) -> tasks::Contract {
        tasks::Contract::Change { connector: 1, kind: 1, words: limits.tasks.result_bytes }
    }

    fn verdict_contract(limits: &Limits) -> tasks::Contract {
        tasks::Contract::Verdict {
            choices: Box::new([tasks::Verdict {
                code: 7,
                words: limits.tasks.result_bytes,
                followups: 0,
                fields: Box::new([]),
            }]),
        }
    }

    fn review_result() -> tasks::TaskResult {
        tasks::TaskResult::Verdict { code: 7, fields: Box::new([]), words: REVIEW_REPORT.into() }
    }

    fn change_result(words: &[u8]) -> tasks::TaskResult {
        tasks::TaskResult::Change {
            connector: 1,
            kind: 1,
            resource: 1,
            title: tasks::ChangeTitle::new(b"Finished work".as_slice().into()),
            words: words.into(),
        }
    }

    fn task_event(core: &mut Core, limits: &Limits, event: tasks::Event) -> Queue<tasks::Request> {
        let mut out = Queue::uncapped();
        tasks::step(
            &mut core.tasks,
            &Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits: limits.tasks },
            event,
            &mut out,
        );
        out
    }

    fn report_delegate(core: &mut Core, limits: &Limits, number: u64, requester: u64, contract: tasks::Contract) {
        let source = core.tasks.task(DELEGATE_TEMPLATE).expect("delegate template");
        let new = tasks::New {
            number,
            project: source.project,
            executor: source.executor,
            llm: source.llm.clone(),
            spec: source.spec.clone(),
            contract,
            authority: source.authority.clone(),
            numbers: source.numbers,
            funder: tasks::Funder::Task(requester),
            deadline: tasks::Deadline::Named,
            label: source.label,
            flags: tasks::Flags::PLAIN,
            dependencies: Box::new([]),
            holdings: Box::new([]),
            wake: source.wake,
            recurring: None,
            tracked: None,
        };
        let mut out = task_event(
            core,
            limits,
            tasks::Event::Make {
                reply_to: ReplyTo::new(skein_lib::Token::new(number)),
                creator: tasks::Party::Task(requester),
                batch: Box::new([new]),
            },
        );
        while let Some(request) = out.pop() {
            if let tasks::Request::Refused { problem, .. } = request {
                panic!("delegate creation refused: {problem:?}");
            }
        }
    }

    fn finish_report(core: &mut Core, limits: &Limits, task: u64, result: tasks::TaskResult) -> Box<tasks::TaskRecord> {
        let to = || ReplyTo::new(skein_lib::Token::new(task));
        for event in [
            tasks::Event::Prepare { reply_to: to(), task },
            tasks::Event::Claim {
                session_end: None,
                offered: jig_domain_tasks::Reasons::ALL,
                reasons: jig_domain_tasks::Reasons::FIRST,
                reply_to: to(),
                task,
                attempt: FIRST_ATTEMPT,
                resumed: false,
                budget: 1,
                account: None,
                writes: Box::new([]),
            },
            tasks::Event::Started { task, attempt: FIRST_ATTEMPT },
            tasks::Event::Activation {
                reply_to: to(),
                task,
                attempt: FIRST_ATTEMPT,
                end: tasks::End::Finished { result, cancel_delegates: false },
                saved: None,
                cause: tasks::Cause::Unpriced,
            },
            tasks::Event::EffectsSettled { task },
            tasks::Event::Settled { task },
        ] {
            let mut out = task_event(core, limits, event);
            while let Some(request) = out.pop() {
                if let tasks::Request::Refused { problem, .. } = &request {
                    panic!("delegate completion refused: {problem:?}");
                }
                if let tasks::Request::Save { record: tasks::Stored::Ended(row) } = request {
                    return row;
                }
            }
        }
        panic!("delegate must end with its admitted result");
    }

    fn retained_contract_reports(
        contract: tasks::Contract,
        result: tasks::TaskResult,
        requester: u64,
    ) -> (Core, Limits, super::super::Row, super::super::Row) {
        let (mut core, mut limits, _) = retained_fixture();
        report_delegate(&mut core, &limits, CHANGE, GOAL, change_contract(&limits));
        report_delegate(&mut core, &limits, PRODUCER, requester, contract);
        let producer = finish_report(&mut core, &limits, PRODUCER, result);
        let change = finish_report(&mut core, &limits, CHANGE, change_result(CHANGE_REPORT));
        let mut descendant = crate::tests::stored_row(crate::Record::Tasks(tasks::Stored::Ended(producer)), None);
        descendant.linked = Box::new([change.clone()]);
        let direct = crate::tests::stored_row(crate::Record::Tasks(tasks::Stored::Ended(change)), None);
        limits.documents.text_bytes = CHUNK_BYTES;
        (core, limits, direct, descendant)
    }

    fn assert_report_chunks(core: &mut Core, limits: &Limits, row: super::super::Row, expected: &[u8]) {
        let mut report = skein_lib::Writer::new(expected.len());
        let mut from = 0;
        for _ in 0..expected.len() {
            let mut retained = crate::tests::stored_row(row.value.clone(), None);
            retained.linked = row.linked.clone();
            let read = retained_reply(core, limits, retained, from).expect("goal may reread kept result words");
            assert_eq!(read.state, TaskState::Ended(tasks::Status::Done));
            assert_eq!(read.words.field, TaskTextField::Result);
            assert_eq!(read.words.total, len_u64(expected.len()));
            assert_eq!(read.words.from, from);
            assert!(read.words.bytes.len() <= skein_lib::length::usize_of(CHUNK_BYTES));
            assert!(skein_lib::bytes::is_utf8(&read.words.bytes));
            report.put(&read.words.bytes).expect("report chunks fit the source length");
            match read.words.next {
                Some(next) => {
                    assert!(next > from);
                    from = next;
                }
                None => break,
            }
        }
        assert_eq!(report.finish().as_ref(), expected, "complete report and Left list bytes");
    }

    #[test]
    fn goal_rereads_retained_change_and_verdict_reports() {
        let (_, limits) = crate::tests::make_tree();
        for (contract, result, words) in [
            (change_contract(&limits), change_result(PRODUCER_REPORT), PRODUCER_REPORT),
            (verdict_contract(&limits), review_result(), REVIEW_REPORT),
        ] {
            for requester in [GOAL, CHANGE] {
                let (mut core, limits, direct, descendant) =
                    retained_contract_reports(contract.clone(), result.clone(), requester);
                assert_report_chunks(&mut core, &limits, direct, CHANGE_REPORT);
                assert_report_chunks(&mut core, &limits, descendant, words);
            }
        }
    }

    #[test]
    fn retained_reports_require_proven_requesters_in_the_callers_project() {
        for field in [TaskTextField::Result, TaskTextField::Specification] {
            for fault in 0_u8..6 {
                let (_, limits) = crate::tests::make_tree();
                let (mut core, limits, _, mut row) =
                    retained_contract_reports(verdict_contract(&limits), review_result(), CHANGE);
                let crate::Record::Tasks(tasks::Stored::Ended(producer)) = &mut row.value else { panic!("producer") };
                assert_eq!(
                    in_document_room(&core, &limits, 3, producer, field, 0, &row.linked),
                    Err(tasks::Refusal::Reference),
                    "a sibling shares the root but has no requester chain"
                );
                match fault {
                    0 => row.linked = Box::new([]),
                    1 => producer.requester = tasks::Party::Person(1), // A root number alone proves no ancestry.
                    2 => producer.project = 2,
                    3 => row.linked.first_mut().expect("change").project = 2,
                    4 => row.linked.first_mut().expect("change").requester = tasks::Party::Task(21),
                    5 => producer.depth = 3,
                    _ => panic!("fixture fault is in 0..6"),
                }
                let answer = if field == TaskTextField::Result {
                    retained_reply(&mut core, &limits, row, 0)
                } else {
                    let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
                    let load = joined_specification_load(&mut core, &env, &row);
                    read_answer(loaded(&mut core, &env, load, Loaded::Page { rows: Box::new([row]), next: None }))
                };
                assert_eq!(answer, Err(tasks::Refusal::Reference), "fault {fault}, field {field:?}");
            }
        }
    }

    #[test]
    fn retained_report_replies_do_not_cross_attempts() {
        let (_, limits) = crate::tests::make_tree();
        let (mut core, limits, _, row) = retained_contract_reports(verdict_contract(&limits), review_result(), CHANGE);
        let env = Env { now: skein_lib::Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
        let load = retained_load(&mut core, &env, 21, 0);
        core.proofs.get_mut(&1).expect("goal").attempt = 10;
        let mut out = loaded(&mut core, &env, load, Loaded::Page { rows: Box::new([row]), next: None });
        assert!(out.pop().is_none(), "a stale attempt cannot receive report bytes");
    }
}
