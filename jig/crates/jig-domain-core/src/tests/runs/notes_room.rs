//! A store-room refusal defers only the next activation; a real notes
//! operation still wakes its waiting briefs when that operation completes.
use crate::tests::*;
use jig_domain_link::{Form, Scheme};

const FIRST: u64 = 1;
const SECOND: u64 = 2;
const CHANNEL: Token = Token::new(7);
const TASKS: u32 = 2; // FIRST and SECOND are the fixture's only live tasks.
const DEFERRED: u32 = 1; // Only FIRST has its notes load refused.
// Each conversation reads its own goal, project and deployment indexes.
const SCOPES: u32 = 3;
// Two distinct conversation scopes and their shared project and deployment.
const ALL_SCOPES: u32 = TASKS + 2;

fn ready() -> (Core, Limits) {
    let (mut core, limits) = core();
    core.opened = true;
    make_task(&mut core, &limits, FIRST);
    make_task(&mut core, &limits, SECOND);
    runs::run_choice::host(&mut core, &limits, CHANNEL, &[b"scripted"]);
    watch_outputs(&mut core, &limits, Event::Account(accounts::Event::Want));
    watch_outputs(
        &mut core,
        &limits,
        Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
            form: Form::Credential(Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        })),
    );
    (core, limits)
}

fn prepare(core: &mut Core, limits: &Limits, task: u64) -> Box<[Request]> {
    let context = core.tasks.run_context(task).expect("live task");
    watch_outputs(core, limits, Event::Activate { context, ready: true, busy: false })
}

fn load(out: &[Request]) -> Option<Token> {
    for request in out {
        if let Request::Held(held) = request
            && let Held::NotesLoad { owner, .. } = held.as_ref()
        {
            return Some(*owner);
        }
    }
    None
}

fn activates(out: &[Request]) -> bool {
    for request in out {
        if let Request::Now(now) = request
            && let Now::Activate { .. } = now.as_ref()
        {
            return true;
        }
    }
    false
}

fn planned(out: &[Request]) -> List<u64> {
    // At most the fixture's two live tasks can finish their brief here.
    let mut tasks = List::with_capacity(TASKS);
    for request in out {
        if let Request::Now(now) = request
            && let Now::BriefCorePlanned { task, .. } = now.as_ref()
        {
            tasks.push(*task).expect("two fixture tasks");
        }
    }
    tasks
}

fn preparing() -> (Core, Limits, Token) {
    let (mut core, limits) = ready();
    let out = prepare(&mut core, &limits, FIRST);
    let owner = load(&out).expect("cold notes index load");
    (core, limits, owner)
}

fn failed(core: &mut Core, limits: &Limits, owner: Token) -> Box<[Request]> {
    watch_outputs(core, limits, Event::Notes(notes::Event::LoadFailed { owner }))
}

fn empty_page(core: &mut Core, limits: &Limits, owner: Token) -> Box<[Request]> {
    watch_outputs(
        core,
        limits,
        Event::Notes(notes::Event::Loaded {
            owner,
            rows: notes::Rows { records: List::with_capacity(limits.notes.load_rows) },
            more: false,
        }),
    )
}

fn same_accounting(before: &tasks::TaskRecord, after: &tasks::TaskRecord) {
    assert_eq!(after.refusals, before.refusals);
    assert_eq!(after.retry_notice, before.retry_notice);
    assert_eq!(after.tries, before.tries);
    assert_eq!(after.attempt, before.attempt);
    assert_eq!(after.made, before.made);
    assert_eq!(after.numbers, before.numbers);
}

// Preparations can outnumber the two briefs gathering at once.
fn four_ready() -> (Core, Limits) {
    let mut limits = limits();
    limits.tasks.tasks = 4;
    limits.tasks.project_tasks = 4;
    limits.tasks.tree_tasks = 4;
    let mut core = core_with_limits(&limits);
    core.opened = true;
    for task in 1..=4 {
        make_task(&mut core, &limits, task);
    }
    runs::run_choice::host(&mut core, &limits, CHANNEL, &[b"scripted"]);
    watch_outputs(&mut core, &limits, Event::Account(accounts::Event::Want));
    watch_outputs(
        &mut core,
        &limits,
        Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
            form: Form::Credential(Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        })),
    );
    (core, limits)
}

fn no_brief_failure(out: &[Request]) {
    for request in out {
        if let Request::Now(now) = request
            && let Now::BriefFailed { task, failure } = now.as_ref()
        {
            panic!("task {task} failed: {failure:?}");
        }
    }
}

fn finish_index(core: &mut Core, limits: &Limits, task: u64, mut owner: Token, pages: u32) -> Box<[Request]> {
    let before = core.tasks.task(task).expect("preparing task").clone();
    let mut out: Box<[Request]> = Box::new([]);
    for _ in 0..pages {
        out = empty_page(core, limits, owner);
        no_brief_failure(&out);
        if let Some(next) = load(&out) {
            owner = next;
        }
    }
    assert_eq!(planned(&out).as_slice(), [task]);
    let after = core.tasks.task(task).expect("prepared task");
    assert_eq!(after.phase, tasks::Phase::Active(tasks::Active::Preparing));
    same_accounting(&before, after);
    out
}

#[test]
fn more_preparations_than_briefs_wait_and_resume_without_counting() {
    let (mut core, limits) = four_ready();
    let out = prepare(&mut core, &limits, FIRST);
    let mut owner = load(&out).expect("first cold index holds notes busy");
    for task in 2..=4 {
        let before = core.tasks.task(task).expect("live task").clone();
        let out = prepare(&mut core, &limits, task);
        assert!(load(&out).is_none());
        no_brief_failure(&out);
        assert_eq!(core.pending_note_briefs.get(&task), Some(&crate::routing::NoteBrief::Operation));
        let after = core.tasks.task(task).expect("waiting task");
        assert_eq!(after.phase, tasks::Phase::Active(tasks::Active::Preparing));
        same_accounting(&before, after);
    }
    assert_eq!(limits.brief.briefs, 2);
    assert_eq!(core.pending_note_briefs.capacity(), limits.tasks.tasks);
    for task in 1..=4 {
        // The first index loads shared project and deployment scopes too.
        let pages = if task == FIRST { SCOPES } else { 1 };
        let out = finish_index(&mut core, &limits, task, owner, pages);
        if task < 4 {
            owner = load(&out).expect("the next waiting index starts");
        } else {
            assert!(load(&out).is_none());
        }
    }
    assert!(core.pending_note_briefs.is_empty());
    assert!(core.due.is_empty());
}

#[test]
fn a_refused_load_defers_its_task_with_two_other_waiters() {
    let (mut core, limits) = four_ready();
    let out = prepare(&mut core, &limits, FIRST);
    let owner = load(&out).expect("first index in flight");
    let before = core.tasks.task(FIRST).expect("preparing task").clone();
    for task in 2..=3 {
        assert!(load(&prepare(&mut core, &limits, task)).is_none());
    }
    let out = failed(&mut core, &limits, owner);
    no_brief_failure(&out);
    assert!(!activates(&out));
    let after = core.tasks.task(FIRST).expect("deferred task");
    assert_eq!(after.phase, tasks::Phase::Active(tasks::Active::Due));
    same_accounting(&before, after);
    assert_eq!(core.due.len(), DEFERRED);
    let owner = load(&out).expect("second task resumes after the refused load");
    let out = finish_index(&mut core, &limits, SECOND, owner, SCOPES);
    let owner = load(&out).expect("third task resumes next");
    let out = finish_index(&mut core, &limits, 3, owner, 1);
    assert!(load(&out).is_none());
    assert!(core.pending_note_briefs.is_empty());
}

fn fill_wait_table(core: &mut Core) {
    for task in 100..u64::from(core.pending_note_briefs.capacity()).checked_add(100).expect("fixture markers") {
        core.pending_note_briefs.insert(task, crate::routing::NoteBrief::Activation).expect("bounded marker");
    }
    assert_eq!(core.pending_note_briefs.len(), core.pending_note_briefs.capacity());
}

fn capacity_pause(core: &mut Core, limits: &Limits, task: u64, before: &tasks::TaskRecord, out: &[Request]) {
    let mut failures = 0_u32;
    for request in out {
        if let Request::Now(now) = request
            && let Now::BriefFailed { task: failed, failure } = now.as_ref()
        {
            assert_eq!(*failed, task);
            assert_eq!(failure.fate, jig_domain_brief::GatherFate::Capacity);
            assert_eq!(failure.section, b"notes");
            assert_eq!(failure.words.as_ref(), b"no room for the index");
            failures = failures.checked_add(1).expect("bounded outputs");
        }
    }
    assert_eq!(failures, 1, "one capacity failure reaches the root");
    assert!(load(out).is_none());
    assert!(!activates(out));
    let row = core.tasks.task(task).expect("paused task");
    let tasks::Phase::Active(tasks::Active::BackingOff { until }) = row.phase else {
        panic!("capacity pauses the task: {:?}", row.phase);
    };
    assert!(until > Wall::EPOCH);
    assert_eq!(row.tries, before.tries);
    assert_eq!(row.attempt, before.attempt);
    assert_eq!(row.numbers, before.numbers);
    assert_eq!(row.made, before.made);
    assert_eq!(row.reasons, before.reasons);
    assert_eq!(row.inbox, before.inbox);
    assert_eq!(row.refusals, before.refusals.checked_add(1).expect("one capacity refusal"));
    assert_eq!(
        row.retry_notice,
        Some(tasks::RetryNotice::Refused {
            count: row.refusals,
            reason: Some(b"notes: no room for the index".as_slice().into()),
        }),
    );
    assert!(!core.contexts.contains_key(&task));
    assert!(!core.transcripts.contains_key(&task));
    assert!(!core.pending_note_briefs.contains_key(&task));
    let now = core.tasks.next_deadline().expect("backoff alarm");
    assert!(now > Time::ZERO);
    let mut out = fire(core, &Env { now, wall: until, limits: *limits }, Timer::Tasks);
    let mut activated = false;
    while let Some(request) = out.pop() {
        if let Request::Now(now) = request
            && let Now::Activate { context } = now.as_ref()
            && context.task == task
        {
            activated = true;
        }
    }
    assert!(activated, "backoff makes the paused task due again");
    let row = core.tasks.task(task).expect("due again");
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Due));
    assert_eq!(row.tries, before.tries);
    assert_eq!(row.attempt, before.attempt);
}

#[test]
fn a_full_notes_wait_table_pauses_a_starting_brief() {
    let (mut core, limits, _) = preparing();
    fill_wait_table(&mut core);
    let before = core.tasks.task(SECOND).expect("live task").clone();
    let out = prepare(&mut core, &limits, SECOND);
    capacity_pause(&mut core, &limits, SECOND, &before, &out);
}

#[test]
fn a_full_notes_wait_table_pauses_a_deferred_brief() {
    let (mut core, limits, owner) = preparing();
    fill_wait_table(&mut core);
    let before = core.tasks.task(FIRST).expect("preparing task").clone();
    let out = failed(&mut core, &limits, owner);
    capacity_pause(&mut core, &limits, FIRST, &before, &out);
}

#[test]
fn a_stale_notes_waiter_does_not_strand_the_next_preparation() {
    let (mut core, limits) = four_ready();
    let out = prepare(&mut core, &limits, FIRST);
    let mut owner = load(&out).expect("first index in flight");
    for task in 2..=3 {
        assert!(load(&prepare(&mut core, &limits, task)).is_none());
    }
    watch_outputs(&mut core, &limits, Event::Tasks(tasks::Event::Hold { task: SECOND, why: tasks::Hold::Stopped }));
    let held = core.tasks.task(SECOND).expect("held waiter").clone();
    assert!(!core.contexts.contains_key(&SECOND));
    for _ in 0..SCOPES {
        let out = empty_page(&mut core, &limits, owner);
        no_brief_failure(&out);
        owner = load(&out).expect("third index starts even though the second preparation is gone");
    }
    assert_eq!(core.tasks.task(SECOND).expect("still held").phase, held.phase);
    same_accounting(&held, core.tasks.task(SECOND).expect("still held"));
    let out = finish_index(&mut core, &limits, 3, owner, 1);
    assert!(load(&out).is_none());
    assert!(core.pending_note_briefs.is_empty());
}

#[test]
fn refused_notes_room_returns_due_without_a_replacement_load() {
    let (mut core, limits, owner) = preparing();
    let before = core.tasks.task(FIRST).expect("preparing task").clone();
    let runs = core.counters.deployment().runs;
    let out = failed(&mut core, &limits, owner);
    assert!(load(&out).is_none());
    assert!(!activates(&out));
    let row = core.tasks.task(FIRST).expect("deferred task");
    assert_eq!(before.phase, tasks::Phase::Active(tasks::Active::Preparing));
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Due));
    same_accounting(&before, row);
    assert_eq!(core.counters.deployment().runs, runs);
    assert!(core.contexts.is_empty());
    assert!((core.result_read_flights() == 0));
    assert!(core.dependency_results.is_empty());
    assert!(core.transcripts.is_empty());
    assert!((core.note_flights() == 0));
    assert!(core.pending_note_briefs.is_empty(), "the one-shot activation marker was consumed");
    assert_eq!(core.due.len(), DEFERRED);
}

#[test]
fn a_later_due_activation_prepares_the_refused_brief_again() {
    let (mut core, limits, owner) = preparing();
    let before = core.tasks.task(FIRST).expect("preparing task").clone();
    failed(&mut core, &limits, owner);
    let context = core.take_due().expect("the ready host makes the deferred task eligible");
    let out = watch_outputs(&mut core, &limits, Event::Activate { context, ready: true, busy: false });
    let mut owner = load(&out).expect("later preparation issues a fresh load");
    let row = core.tasks.task(FIRST).expect("preparing again");
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Preparing));
    same_accounting(&before, row);
    let mut out = out;
    for _ in 0..SCOPES {
        out = empty_page(&mut core, &limits, owner);
        if let Some(next) = load(&out) {
            owner = next;
        }
    }
    assert!(planned(&out).as_slice().contains(&FIRST));
}

#[test]
fn a_busy_notes_operation_still_releases_its_waiting_brief() {
    let (mut core, limits, mut owner) = preparing();
    let out = prepare(&mut core, &limits, SECOND);
    assert!(load(&out).is_none());
    assert_eq!(core.pending_note_briefs.get(&SECOND), Some(&crate::routing::NoteBrief::Operation));
    let mut finished = List::with_capacity(TASKS);
    for _ in 0..ALL_SCOPES {
        let out = empty_page(&mut core, &limits, owner);
        for task in &planned(&out) {
            finished.push(*task).expect("fixture task count");
        }
        if let Some(next) = load(&out) {
            owner = next;
        }
    }
    assert_eq!(finished.as_slice(), [FIRST, SECOND]);
    assert!(core.pending_note_briefs.is_empty());
    assert!(core.due.is_empty());
}

#[test]
fn ordinary_preparation_waiting_keeps_its_existing_activation_route() {
    let (mut core, limits, _) = preparing();
    let out = watch_outputs(&mut core, &limits, Event::Tasks(tasks::Event::PreparationWaiting { task: FIRST }));
    assert!(activates(&out));
    assert!(core.due.is_empty());
}

#[test]
fn a_tasks_end_clears_its_unused_activation_marker() {
    let (mut core, limits) = ready();
    core.pending_note_briefs.insert(FIRST, crate::routing::NoteBrief::Activation).expect("bounded marker");
    watch_outputs(
        &mut core,
        &limits,
        Event::Tasks(tasks::Event::Cancel {
            reply_to: ReplyTo::new(Token::new(FIRST)),
            by: tasks::Party::Person(1),
            operation: tasks::Cancel { task: FIRST, reason: b"done".as_slice().into(), grace: None, message: None },
        }),
    );
    watch_outputs(&mut core, &limits, Event::Tasks(tasks::Event::EffectsSettled { task: FIRST }));
    watch_outputs(&mut core, &limits, Event::Tasks(tasks::Event::Settled { task: FIRST }));
    assert!(core.tasks.task(FIRST).is_none());
    assert!(core.pending_note_briefs.is_empty());
}

#[test]
fn preparation_without_a_transcript_reaches_the_brief_inside_the_core() {
    let (mut core, limits) = ready();
    let out = prepare(&mut core, &limits, FIRST);
    assert_eq!(core.tasks.task(FIRST).unwrap().phase, tasks::Phase::Active(tasks::Active::Preparing));
    assert!(load(&out).is_some(), "preparation reached the notes stage of the brief");
    for request in out {
        if let Request::Now(now) = request
            && let Now::StartPreparation { .. } = *now
        {
            panic!("a preparation without a transcript needs no load handoff");
        }
    }
}

#[test]
fn preparation_with_a_transcript_requests_one_load_then_reaches_the_brief() {
    let (mut core, limits) = ready();
    previous_window(&mut core, &limits, 0, false);
    let context = core.tasks.run_context(FIRST).unwrap();
    let out = watch_outputs(&mut core, &limits, Event::Activate { context, ready: true, busy: false });
    let mut loads = 0_u32;
    for request in out {
        match request {
            Request::Write(_) => {}
            Request::Now(now) => {
                let Now::StartPreparation { task: FIRST, from: (1, 1), .. } = *now else {
                    panic!("only the transcript load leaves preparation");
                };
                loads += 1;
            }
            other @ (Request::ProjectMade { .. } | Request::Ask { .. } | Request::Held(_)) => {
                panic!("unexpected preparation output: {other:?}");
            }
        }
    }
    assert_eq!(loads, 1, "only one transcript load leaves the core");
    assert!((core.note_flights() == 0));
    let out = watch_outputs(
        &mut core,
        &limits,
        Event::TranscriptLoaded {
            task: FIRST,
            load: TranscriptLoad::Page {
                rows: Box::new([(
                    TurnRecord {
                        task: FIRST,
                        attempt: 1,
                        turn: 1,
                        spent: 0,
                        read: None,
                        opens_window: true,
                        at: Wall::EPOCH,
                        transcript: b"turn".as_slice().into(),
                    },
                    Some(b"text".as_slice().into()),
                )]),
                done: true,
            },
        },
    );
    assert!(load(&out).is_some(), "the load answer reached the brief");
    assert!(core.transcripts.get(&FIRST).unwrap().loaded);
}

fn previous_window(core: &mut Core, limits: &Limits, transient: u32, closed: bool) {
    let mut proof = RunProof::claimed(FIRST, 1, 1, None, fleet::HostKind::Worker);
    proof.window = Some(CurrentWindow { attempt: 1, turn: 1, closed });
    core.proofs.insert(FIRST, proof).unwrap();
    let mut row = core.tasks.task(FIRST).unwrap().clone();
    row.attempt = 1;
    row.tries.transient = transient;
    let ledger = *core.tasks.funding(row.funder).unwrap();
    fixtures::restore(core, limits, Box::new([row]));
    send_task(core, limits, tasks::Event::Restore { record: tasks::Stored::Ledger(ledger) });
    send_task(core, limits, tasks::Event::Check);
    send_task(core, limits, tasks::Event::Restored);
}

fn preparation_loads(out: &[Request]) -> List<(u64, (u64, u32))> {
    let mut loads = List::with_capacity(TASKS);
    for request in out {
        if let Request::Now(now) = request
            && let Now::StartPreparation { task, from } = now.as_ref()
        {
            loads.push((*task, *from)).expect("fixture task count bounds loads");
        }
    }
    loads
}

#[test]
fn a_busy_root_keeps_an_agent_with_a_window_to_load_due() {
    for (transient, closed) in [(0, false), (1, true), (1, false)] {
        let (mut core, limits) = ready();
        previous_window(&mut core, &limits, transient, closed);
        let context = core.tasks.run_context(FIRST).unwrap();
        let out = watch_outputs(&mut core, &limits, Event::Activate { context, ready: true, busy: true });
        assert_eq!(core.tasks.task(FIRST).unwrap().phase, tasks::Phase::Active(tasks::Active::Due));
        assert_eq!(core.due.len(), 1);
        assert_eq!(core.due.iter().next().expect("deferred activation").task, FIRST);
        assert!(!core.contexts.contains_key(&FIRST));
        assert!(!core.transcripts.contains_key(&FIRST));
        assert!(preparation_loads(&out).is_empty());
        let context = core.take_due().expect("busy activation stays eligible");
        assert_eq!(context.task, FIRST);
        let out = watch_outputs(&mut core, &limits, Event::Activate { context, ready: true, busy: false });
        assert_eq!(preparation_loads(&out).as_slice(), [(FIRST, (1, 1))]);
        assert_eq!(core.tasks.task(FIRST).unwrap().phase, tasks::Phase::Active(tasks::Active::Preparing));
    }
}

#[test]
fn a_busy_root_still_prepares_an_agent_with_nothing_to_load() {
    fn retry() -> (Core, Limits) {
        let (mut core, limits) = ready();
        previous_window(&mut core, &limits, 1, false);
        core.proofs.get_mut(&FIRST).unwrap().window = None;
        (core, limits)
    }
    for setup in [ready, retry] {
        let (mut core, limits) = setup();
        let context = core.tasks.run_context(FIRST).unwrap();
        let out = watch_outputs(&mut core, &limits, Event::Activate { context, ready: true, busy: true });
        assert!(preparation_loads(&out).is_empty());
        assert_eq!(core.tasks.task(FIRST).unwrap().phase, tasks::Phase::Active(tasks::Active::Preparing));
        assert!(core.due.is_empty());
        assert!(core.contexts.contains_key(&FIRST));
        assert_eq!(core.transcripts.get(&FIRST).unwrap().window, None);
        assert!(load(&out).is_some(), "no transcript load is needed before the brief");
    }
}
