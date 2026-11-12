//! Component fixtures over the real store and the fake files (protocol/store/README.md, 9).
use jig_test_connector as d;
use skein_lib::{Token, Wall};
pub const FIRST: u16 = 1;
pub fn path() -> d::Path {
    jig_test_connector_kit::path(1, 1)
}
pub struct Component {
    pub store: jig_protocol_store::Store,
    pub files: crate::Files,
    pub limits: jig_protocol_store::Limits,
}
pub fn root_limits() -> jig_test_domain::Limits {
    jig_test_profile::limits()
}
pub fn component_limits() -> jig_protocol_store::Limits {
    let root = root_limits();
    let kv = skein_kv::Limits {
        key: 1024,
        value: 16_777_216,
        ops: 8192,
        commit: u32::MAX,
        queued: 8,
        queued_bytes: 34_359_738_360,
        budget: 4_194_304,
        segment: 8_589_934_590,
        snapshot_after: 4_294_967_295,
        chunk: 16_778_260,
        page: skein_kv::Page { rows: 2, bytes: 16_778_240 },
        deadline: skein_lib::Duration::from_secs(1),
    };
    jig_protocol_store::Limits {
        core: root.core,
        store: kv,
        secrets: kv,
        commits: jig_protocol_store::sizes::Commits { in_flight: root.journal.commits, writes: root.journal.writes },
        joins: 128,
        queue: 8,
        files: 8,
        reads: 4,
        orphan_erases: 1,
        held: 2,
    }
}
pub fn component() -> Component {
    build_component()
}
pub fn starting_component() -> Component {
    build_component()
}
fn build_component() -> Component {
    let root = root_limits();
    let limits = component_limits();
    let kv = limits.store;
    let stores =
        jig_protocol_store::Stores { store: skein_kv::Store::memory(&kv), secrets: skein_kv::Store::memory(&kv) };
    let store = jig_protocol_store::Store::over(&limits, jig_test_store::application(&root.connector), &[], stores);
    let mut component = Component { store, files: crate::Files::new(), limits };
    let root = component.files.root();
    let mut above = skein_lib::Queue::uncapped();
    let open = jig_protocol_store::Request::Open { owner: Token::new(900), root, installation: [0; 16] };
    send(&mut component, open, &mut above);
    assert!(matches_opened(&mut above));
    component
}
fn matches_opened(above: &mut skein_lib::Queue<jig_protocol_store::Event>) -> bool {
    match above.pop() {
        Some(jig_protocol_store::Event::Opened { last: 0, .. }) => above.is_empty(),
        Some(_) | None => false,
    }
}
pub fn routes() -> jig_test_store::route::Routes {
    let mut routes = jig_test_store::route::Routes::new(8);
    routes.configure(&root_limits());
    routes.configure_restore(&component_limits()).expect("restart page bounds fit");
    routes
}
pub fn send(
    component: &mut Component,
    request: jig_protocol_store::Request,
    above: &mut skein_lib::Queue<jig_protocol_store::Event>,
) {
    let mut below = jig_protocol_store::Below {
        store: skein_lib::Queue::uncapped(),
        secrets: skein_lib::Queue::uncapped(),
        transcripts: skein_lib::Queue::uncapped(),
    };
    let env = skein_lib::Env { now: skein_lib::Time::ZERO, wall: Wall::EPOCH, limits: component.limits };
    jig_protocol_store::down(&mut component.store, &env, request, above, &mut below);
    for _ in 0..128_u32 {
        let Some(request) = below.transcripts.pop() else {
            break;
        };
        let arrived = component.files.answer(request);
        jig_protocol_store::up(
            &mut component.store,
            &env,
            jig_protocol_store::Io::Transcripts(arrived),
            above,
            &mut below,
        );
    }
    assert!(below.store.is_empty() && below.secrets.is_empty() && below.transcripts.is_empty());
}
pub fn routed(
    component: &mut Component,
    routes: &mut jig_test_store::route::Routes,
    out: jig_test_store::StoreOut,
) -> Box<[jig_test_store::Up]> {
    let mut requests = skein_lib::Queue::with_capacity(64);
    requests.push(jig_test_store::route::down(routes, out).expect("routed request"));
    let mut found = skein_lib::List::with_capacity(128);
    for _ in 0..64_u32 {
        let Some(request) = requests.pop() else {
            break;
        };
        let mut above = skein_lib::Queue::uncapped();
        send(component, request, &mut above);
        for _ in 0..above.len() {
            let Some(event) = above.pop() else {
                break;
            };
            let mut outputs = skein_lib::Queue::with_capacity(8);
            jig_test_store::route::up(routes, event, &mut outputs);
            for _ in 0..8_u32 {
                let Some(output) = outputs.pop() else {
                    break;
                };
                match output {
                    jig_test_store::Up::Ask(request) => requests.push(request),
                    other @ jig_test_store::Up::Root(_) => {
                        found.push(other).expect("bounded fixture");
                    }
                }
            }
        }
    }
    assert!(requests.is_empty());
    found.into_boxed()
}
pub fn header(commits: u64) -> jig_test_domain::Write {
    jig_test_domain::Write::Save(jig_test_domain::Record::Core(jig_domain_core::Record::Core(
        jig_domain_core::CoreRecord::Deployment(jig_domain_core::Deployment {
            id: [18; 16],
            tasks: 21,
            people: 0,
            sign_ins: 0,
            messages: 0,
            runs: 1,
            calls: 0,
            connector_rows: 0,
            projects: 0,
            applications: 0,
            commits,
        }),
    )))
}
pub fn root_record(record: d::Record) -> jig_test_domain::Record {
    jig_test_domain::Record::Connector { number: FIRST, record }
}
pub fn turn(commits: u64) -> jig_test_domain::Write {
    jig_test_domain::Write::Save(jig_test_domain::Record::Core(jig_domain_core::Record::Core(
        jig_domain_core::CoreRecord::Turn(jig_domain_core::TurnRecord {
            task: 21,
            attempt: 1,
            turn: u32::try_from(commits).expect("fixture turn"),
            opens_window: commits == 1,
            spent: 0,
            read: None,
            at: Wall::EPOCH,
            transcript: Box::from(b"opaque".as_slice()),
        }),
    )))
}

pub fn core_commit() -> jig_test_store::StoreOut {
    jig_test_store::StoreOut::Commit {
        number: 1,
        writes: Box::new([
            header(1),
            jig_test_domain::Write::Save(jig_test_domain::Record::Core(jig_domain_core::Record::People(
                jig_domain_people::Stored::Roles { project: 1, holdings: Box::new([]) },
            ))),
            jig_test_domain::Write::Save(jig_test_domain::Record::Core(jig_domain_core::Record::People(
                jig_domain_people::Stored::ReadResults { party: 1, through: 0 },
            ))),
        ]),
    }
}
pub fn commit_writes(request: jig_protocol_store::Request) -> Box<[jig_protocol_store::Write]> {
    match request {
        jig_protocol_store::Request::Commit { writes, .. } => writes,
        jig_protocol_store::Request::Switch { .. }
        | jig_protocol_store::Request::Site { .. }
        | jig_protocol_store::Request::Open { .. }
        | jig_protocol_store::Request::Restore { .. }
        | jig_protocol_store::Request::Load { .. }
        | jig_protocol_store::Request::Get { .. }
        | jig_protocol_store::Request::Read { .. }
        | jig_protocol_store::Request::Transcript { .. }
        | jig_protocol_store::Request::Secrets { .. } => unreachable!("fixture requests a commit"),
    }
}
#[expect(clippy::too_many_lines, reason = "the fixture names every retained task field")]
pub fn ended_chat() -> Box<jig_domain_tasks::TaskRecord> {
    use jig_domain_tasks as tasks;
    Box::new(tasks::TaskRecord {
        session: None,
        talk: None,
        activation_open: false,
        closer: None,
        offered: jig_domain_tasks::Reasons::ALL,
        reasons: jig_domain_tasks::Reasons::FIRST,
        started_from: Box::new([]),
        asked_requester: false,
        placed: None,
        settings: tasks::SettingsRevision { task: 1, run: None },
        label: 1,
        flags: tasks::Flags { conversation: true, ..tasks::Flags::PLAIN },
        last_activity: Wall::EPOCH,
        ended_below: tasks::Endings::ZERO,
        ended_delegates: tasks::Endings::ZERO,
        question_lines: Box::new([]),
        hold: 0,
        proposal_rows: Box::new([]),
        instructions: None,
        llm: None,
        kept_account: None,
        refused: Box::new([]),
        grace: None,
        created_at: Wall::EPOCH,
        changed: Wall::EPOCH,
        phase_since: Wall::EPOCH,
        run_since: None,
        retry_notice: None,
        ended_at: Some(Wall::EPOCH),
        revision: 0,
        revisions: tasks::Revisions { hold: 0, spec: 0, person: 0 },
        milestone: 0,
        activity: 0,
        due_reason: tasks::Why::Made,
        narrowing: false,
        llm_changed: false,
        result_position: 0,
        last_reply: 0,
        replies: Box::new([]),
        escalation: tasks::Escalation::Unheld { revision: 0 },
        proposal: None,
        result_proposal: false,
        number: 1,
        project: 1,
        requester: tasks::Party::Person(1),
        run_spent: 0,
        run_reserved: 0,
        root: 1,
        depth: 0,
        executor: tasks::Executor::Agent { charter: 1 },
        taken_by: None,
        person_answer: None,
        person_wait: None,
        result_followups: Box::new([]),
        tracked: None,
        recurring: None,
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: b"chat".as_slice().into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: 16 },
        authority: tasks::Authority {
            tools: tasks::Tools(0),
            models: tasks::Models::ALL,
            grants: Box::new([]),
            delegation: tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 },
            budget: tasks::Budget { spend: 0, deadline: None },
            notes: tasks::Scopes(0),
            note_resources: Box::new([]),
        },
        numbers: tasks::Numbers { budget: 0, spent: 0, spent_below: 0, reserved: 0 },
        funder: tasks::Funder::Pool { project: 1, person: 1, period: 1 },
        waiting_on: Box::new([]),
        holdings: Box::new([]),
        holds_taken: false,
        hold_wait_since: None,
        hold_bound_since: None,
        dependencies: Box::new([]),
        added: Box::new([]),
        delegates: Box::new([]),
        references: Box::new([]),
        questions: Box::new([]),
        subscriptions: Box::new([]),
        wake: tasks::WakePolicy::DEFAULT,
        turn: 0,
        last_message: 0,
        inbox: Box::new([]),
        saved: Box::new([]),
        ever_turned: false,
        made: 1,
        attempt: 0,
        last_answer: None,
        tries: tasks::Tries::NONE,
        invalid_result: None,
        refusals: 0,
        room_wait: None,
        phase: tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::Report { words: b"done".as_slice().into() })),
    })
}
