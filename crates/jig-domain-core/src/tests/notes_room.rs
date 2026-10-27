//! A store-room refusal defers only the next activation; a real notes
//! operation still wakes its waiting briefs when that operation completes.
use super::*;

const FIRST: u64 = 1;
const SECOND: u64 = 2;
const CHANNEL: Token = Token::new(7);
const TASKS: u32 = 2; // FIRST and SECOND are the fixture's only live tasks.
const DEFERRED: u32 = 1; // Only FIRST has its notes load refused.
// The fixture's briefs read project and deployment indexes, one empty page each.
const SCOPES: u32 = 2;

fn ready() -> (Core, Limits) {
    let (mut core, limits) = core();
    make_task(&mut core, &limits, FIRST);
    make_task(&mut core, &limits, SECOND);
    run_choice::host(&mut core, &limits, CHANNEL, &[b"scripted"]);
    watch_outputs(&mut core, &limits, Event::Account(accounts::Event::Want));
    watch_outputs(
        &mut core,
        &limits,
        Event::Account(accounts::Event::Component(accounts::FromComponent::Granted {
            form: accounts::Form::Credential(accounts::Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        })),
    );
    (core, limits)
}

fn prepare(core: &mut Core, limits: &Limits, task: u64) -> Box<[Request]> {
    let context = core.tasks.run_context(task).expect("live task");
    watch_outputs(core, limits, Event::PreparedAgent { context, transcript_waiter: None, busy: false });
    watch_outputs(core, limits, Event::Tasks(tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(task)), task }));
    watch_outputs(core, limits, Event::StartBrief { task })
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

fn prepares(out: &[Request]) -> bool {
    for request in out {
        if let Request::Now(now) = request
            && let Now::PrepareAgent { .. } = now.as_ref()
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
    assert!(core.result_reads.is_empty());
    assert!(core.dependency_results.is_empty());
    assert!(core.transcripts.is_empty());
    assert!(core.note_routes.is_empty());
    assert!(core.pending_note_briefs.is_empty(), "the one-shot activation marker was consumed");
    assert_eq!(core.due.len(), DEFERRED);
}

#[test]
fn a_later_due_activation_prepares_the_refused_brief_again() {
    let (mut core, limits, owner) = preparing();
    let before = core.tasks.task(FIRST).expect("preparing task").clone();
    failed(&mut core, &limits, owner);
    let context = core.take_due().expect("the ready host makes the deferred task eligible");
    let out = watch_outputs(&mut core, &limits, Event::Activate { context, ready: true });
    assert!(prepares(&out));
    let out = prepare(&mut core, &limits, FIRST);
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
    assert_eq!(core.pending_note_briefs.get(&SECOND), Some(&routing::NoteBrief::Operation));
    let mut out = out;
    for _ in 0..SCOPES {
        out = empty_page(&mut core, &limits, owner);
        if let Some(next) = load(&out) {
            owner = next;
        }
    }
    assert_eq!(planned(&out).as_slice(), [FIRST, SECOND]);
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
    core.pending_note_briefs.insert(FIRST, routing::NoteBrief::Activation).expect("bounded marker");
    watch_outputs(
        &mut core,
        &limits,
        Event::Tasks(tasks::Event::Control {
            reply_to: ReplyTo::new(Token::new(FIRST)),
            by: tasks::Party::Person(1),
            task: FIRST,
            control: tasks::Control::Cancel { reason: b"done".as_slice().into(), grace: None, message: None },
        }),
    );
    watch_outputs(&mut core, &limits, Event::Tasks(tasks::Event::EffectsSettled { task: FIRST }));
    watch_outputs(&mut core, &limits, Event::Tasks(tasks::Event::Settled { task: FIRST }));
    assert!(core.tasks.task(FIRST).is_none());
    assert!(core.pending_note_briefs.is_empty());
}
