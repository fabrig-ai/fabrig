//! Specification source facts through the public task document constructor.
use crate::tests::*;

const FAILURE_WORDS: &[u8] = concat!("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx", "é CI failed: missing artifact").as_bytes();

const UTF8_WORDS: &[u8] = b"~\0^\x01";

#[test]
fn task_specification_keeps_source_inputs_and_separate_page_loss() {
    let (core, mut limits) = make_tree();
    limits.documents.lines = 2;
    let mut row = core.tasks.task(2).unwrap().clone();
    row.spec.inputs = Box::new([1, u64::MAX]);
    row.dependencies = Box::new([91, 92, 93]);
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert!(task.cut, "the dependency list omits its third row");
    assert!(!task.spec.cut, "the entire specification still fits");
    assert_eq!(task.spec.words, task.words);
    assert_eq!(task.spec.parameters, task.parameters);
    assert_eq!(task.spec.inputs.len(), 2);
    assert_eq!(task.spec.inputs.get(0), Some(&1));
    assert_eq!(task.spec.inputs.get(1), Some(&u64::MAX));
    let TaskLink::Live(input) = task.inputs.get(0).unwrap() else { panic!("live input") };
    assert_eq!(input.number, 1);
    assert_eq!(task.inputs.get(1), Some(&TaskLink::Stored { number: u64::MAX, status: None }));
    assert_eq!(task.dependencies.len(), 2);
    assert_eq!(core.tasks.task(2).unwrap().spec.inputs.len(), 0, "projection does not mutate the owner");
}

#[test]
fn task_specification_marks_parameter_and_input_list_loss_independently() {
    let (core, mut limits) = make_tree();
    limits.documents.lines = 2;
    let mut row = core.tasks.task(2).unwrap().clone();
    row.spec.parameters = Box::new([
        tasks::Parameter::Number { name: 7, value: 8 },
        tasks::Parameter::Number { name: 9, value: 10 },
        tasks::Parameter::Number { name: 11, value: 12 },
    ]);
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert!(task.spec.cut, "only parameters exceed the spec list bound");
    assert_eq!(task.spec.parameters.len(), 2);
    assert_eq!(task.spec.parameters.get(0), Some(&Parameter::Number { name: 7, value: 8 }));
    assert_eq!(task.spec.parameters.get(1), Some(&Parameter::Number { name: 9, value: 10 }));
    assert!(task.spec.inputs.is_empty());
    assert!(!task.spec.words.cut);

    row.spec.parameters = Box::new([]);
    row.spec.inputs = Box::new([u64::MAX, 1, 3]);
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert!(task.spec.cut, "only inputs exceed the spec list bound");
    assert!(task.spec.parameters.is_empty());
    assert_eq!(task.spec.inputs.len(), 2);
    assert_eq!(task.spec.inputs.get(0), Some(&u64::MAX));
    assert_eq!(task.spec.inputs.get(1), Some(&1));
    assert!(!task.spec.words.cut);
}

#[test]
fn task_specification_preserves_parameter_identity_and_local_text_loss() {
    let (core, mut limits) = make_tree();
    limits.documents.lines = 3;
    limits.documents.text_bytes = 3;
    let mut row = core.tasks.task(2).unwrap().clone();
    row.spec.words = Box::from(&b"abcdef"[..]);
    row.spec.parameters = Box::new([
        tasks::Parameter::Bytes { name: u32::MAX, value: Box::from(UTF8_WORDS) },
        tasks::Parameter::Number { name: 2, value: u64::MAX },
        tasks::Parameter::Resource { name: 3, connector: u16::MAX, resource: u64::MAX },
    ]);
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert!(!task.spec.cut, "text loss does not omit specification list rows");
    assert!(!task.cut, "text loss is local to each text");
    assert_eq!(task.spec.words, Text { bytes: Box::from(&b"abc"[..]), cut: true });
    assert_eq!(
        task.spec.parameters.get(0),
        Some(&Parameter::Bytes { name: u32::MAX, value: Text { bytes: Box::from(&UTF8_WORDS[..3]), cut: true } })
    );
    assert_eq!(task.spec.parameters.get(1), Some(&Parameter::Number { name: 2, value: u64::MAX }));
    assert_eq!(
        task.spec.parameters.get(2),
        Some(&Parameter::Resource {
            name: 3,
            part: Part { connector: u16::MAX, kind: PartKind::Resource, key: PartKey::Number(u64::MAX) }
        })
    );
    assert_eq!(task.spec.parameters, task.parameters);
}

#[test]
fn task_dependencies_show_added_provenance_pending_and_stub_status() {
    const DEPENDENCY: u64 = 7;
    let (mut core, limits) = make_tree();
    let mut row = core.tasks.task(2).unwrap().clone();
    let parent = core.tasks.task(1).unwrap().clone();
    fixtures::restore(&mut core, &limits, Box::new([parent, row.clone()]));
    row.phase = tasks::Phase::Active(tasks::Active::Idle);
    row.dependencies = Box::new([1, DEPENDENCY]);
    row.waiting_on = Box::new([1]);
    let added = tasks::Added { task: DEPENDENCY, at: Wall::from_nanos(11), by: tasks::Party::Person(1) };
    row.added = Box::new([added.clone()]);
    send_task(
        &mut core,
        &limits,
        tasks::Event::Restore {
            record: tasks::Stored::Stub(tasks::Stub {
                task: DEPENDENCY,
                phase: tasks::Status::Done,
                result: Token::new(DEPENDENCY),
            }),
        },
    );
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert_eq!(task.dependency_facts.get(0).unwrap().number, 1);
    assert!(task.dependency_facts.get(0).unwrap().pending);
    assert!(task.dependency_facts.get(0).unwrap().keeps);
    assert_eq!(task.dependency_facts.get(1).unwrap().added, Some(added));
    assert_eq!(task.dependency_facts.get(1).unwrap().status, Some(tasks::Status::Done));
    assert!(!task.dependency_facts.get(1).unwrap().pending);
    assert!(!task.dependency_facts.get(1).unwrap().keeps);
    row.waiting_on = Box::new([]);
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert!(!task.dependency_facts.get(0).unwrap().keeps);
}

#[test]
fn a_kept_person_card_offers_no_answer_and_keeps_its_take() {
    let (core, limits) = make_tree();
    let mut row = core.tasks.task(2).unwrap().clone();
    row.executor = tasks::Executor::Person(tasks::PersonAddress::Role(people::Role::Owner.number()));
    row.phase = tasks::Phase::Active(tasks::Active::Idle);
    row.dependencies = Box::new([1]);
    row.waiting_on = Box::new([1]);
    let card = person_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    let mut take = false;
    for offer in card.acts.as_slice() {
        assert_ne!(offer.act, crate::acts::Act::Answer);
        if offer.act == crate::acts::Act::Take {
            take = true;
        }
    }
    assert!(take);
}

#[test]
fn dependency_refusals_keep_the_problem_names_at_the_people_boundary() {
    let (core, _) = make_tree();
    for (why, expected) in [
        (tasks::Refusal::Circle { cut: true }, people::Refusal::Circle { tasks: Box::new([7, 5, 9]), cut: true }),
        (tasks::Refusal::Kept, people::Refusal::Kept { pending: Box::new([7, 5, 9]) }),
        (
            tasks::Refusal::DependencyEnded { how: tasks::Status::Failed },
            people::Refusal::Dependency { dependency: 7, how: people::DependencyStatus::Failed },
        ),
    ] {
        let problem = tasks::Problem::naming(Some(2), why, &[7, 5, 9]);
        assert_eq!(
            crate::routing::refusals::person_problem_refusal(&core, problem),
            people::Outcome::Refused(expected)
        );
    }
}

#[test]
fn a_circle_hold_projection_cuts_the_chips_without_losing_the_source_cut() {
    let (core, mut limits) = make_tree();
    limits.documents.lines = 2;
    let mut row = core.tasks.task(2).unwrap().clone();
    row.revisions.hold = 1;
    row.phase = tasks::Phase::Held {
        was: tasks::Was::Waiting,
        why: tasks::Hold::Circle,
        source: Some(Box::new(tasks::HoldSource::Circle(tasks::Circle { tasks: Box::new([9, 7, 5]), cut: false }))),
    };
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    let circle = task.hold.unwrap().circle.unwrap();
    assert_eq!(circle.tasks.as_ref(), &[9, 7]);
    assert!(circle.cut);
}

#[test]
fn task_text_retrieval_preserves_failure_words_past_the_summary() {
    let (core, mut limits) = make_tree();
    limits.documents.text_bytes = 128;
    let mut row = core.tasks.task(2).unwrap().clone();
    let words = FAILURE_WORDS;
    row.spec.words = Box::from(words);
    row.instructions = Some(Box::from(words));
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    assert!(task.spec.words.cut);
    assert!(task.instructions.as_ref().unwrap().cut);
    for field in [crate::reads::TaskTextField::Specification, crate::reads::TaskTextField::Instructions] {
        let mut from = 0;
        let mut whole = skein_lib::Writer::new(words.len());
        loop {
            let chunk = crate::reads::task_text_from_row(&core, &limits, 1, &row, field, from).unwrap();
            assert!(chunk.bytes.len() <= 128);
            assert!(skein_lib::bytes::is_utf8(&chunk.bytes));
            whole.put(&chunk.bytes).unwrap();
            match chunk.next {
                Some(next) => {
                    assert!(next > from);
                    from = next;
                }
                None => break,
            }
        }
        let whole = whole.finish();
        assert_eq!(whole.as_ref(), words);
        assert!(whole.ends_with(b"CI failed: missing artifact"));
    }
}

#[test]
fn task_text_offsets_reject_mid_character_and_past_end_without_stalling() {
    use crate::reads::{TaskTextField, Why, task_text_from_row};
    let (core, mut limits) = make_tree();
    let mut row = core.tasks.task(2).unwrap().clone();
    row.spec.words = Box::from("éZ".as_bytes());
    limits.documents.text_bytes = 1;
    assert_eq!(task_text_from_row(&core, &limits, 1, &row, TaskTextField::Specification, 0), Err(Why::Busy));
    for from in [1, 4, u64::MAX] {
        assert_eq!(task_text_from_row(&core, &limits, 1, &row, TaskTextField::Specification, from), Err(Why::Unknown));
    }
    let end = task_text_from_row(&core, &limits, 1, &row, TaskTextField::Specification, 3).unwrap();
    assert!(end.bytes.is_empty());
    assert_eq!(end.next, None);
    assert_eq!(task_text_from_row(&core, &limits, 99, &row, TaskTextField::Specification, 2), Err(Why::Unknown));
}

#[test]
fn task_text_reads_live_sources_and_retained_suffixes_under_the_same_visibility() {
    use crate::reads::{Answer, Read, TaskTextField};
    let (mut core, mut limits) = watch_core();
    limits.documents.text_bytes = 128;
    let read = Read::TaskText { task: 1, field: TaskTextField::Specification, from: 0 };
    let Held::Read { answer: Answer::TaskText(page), .. } = read_start(&mut core, &limits, 10, read) else {
        panic!("live source answered without store")
    };
    assert_eq!(page.bytes, core.tasks.task(1).unwrap().spec.words);
    let mut row = fixtures::ended_report(99, tasks::Party::Person(1));
    row.spec.words = Box::from(FAILURE_WORDS);
    let read = Read::TaskText { task: 99, field: TaskTextField::Specification, from: 127 };
    let Held::ReadLoad { load, .. } = read_start(&mut core, &limits, 10, read) else { panic!("retained load") };
    let loaded = crate::reads::Loaded::Page {
        rows: Box::new([stored_row(Record::Tasks(tasks::Stored::Ended(Box::new(row))), None)]),
        next: None,
    };
    let Answer::TaskText(page) = loaded_answer(&mut core, &limits, load, loaded).unwrap() else {
        panic!("retained source chunk")
    };
    assert_eq!(page.bytes.as_ref(), "é CI failed: missing artifact".as_bytes());
    assert_eq!(page.next, None);
}

#[test]
fn task_instruction_text_uses_the_tasks_charter_revision_without_an_override() {
    use crate::reads::{TaskTextField, task_text_from_row};
    let (core, limits) = make_tree();
    let row = core.tasks.task(2).unwrap();
    let tasks::Executor::Agent { charter } = row.executor else { panic!("agent") };
    assert_eq!(row.instructions, None);
    let page = task_text_from_row(&core, &limits, 1, row, TaskTextField::Instructions, 0).unwrap();
    assert_eq!(page.bytes.as_ref(), core.charter_at(charter, row.settings.task).unwrap().policy.instructions.as_ref());
}

#[test]
fn connector_hold_accounts_keep_their_own_bound_in_task_and_chip() {
    let (core, mut limits) = make_tree();
    limits.documents.text_bytes = 128;
    let mut row = core.tasks.task(2).unwrap().clone();
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    for _ in 0..tasks::HOLD_ACCOUNT_BYTES {
        words.push(b'x').unwrap();
    }
    let words = words.into_boxed();
    row.revisions.hold = 1;
    row.phase = tasks::Phase::Held {
        was: tasks::Was::Waiting,
        why: tasks::Hold::ResourceUnavailable,
        source: Some(Box::new(tasks::HoldSource::Account { connector: 0, entry: None, words: words.clone() })),
    };
    let task = task_from_row(&core, &limits, 1, &row, Wall::EPOCH).unwrap();
    for hold in [task.hold.unwrap(), task.chip.details.hold.unwrap()] {
        let text = hold.words.unwrap();
        assert_eq!(text.bytes, words);
        assert!(!text.cut);
    }
}
