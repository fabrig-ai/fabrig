//! Shared projects' status funding, interleaved sittings and late reports
//! (temper/docs/design/domain/forge/attention.md, 3; jig/docs/design/domain/accounts.md, 4.2).
use super::*;
use jig_domain_accounts as accounts;
use jig_domain_core::{documents, reads};

const PROJECT_KEY: u32 = 30;
const PROJECT_SOURCE: u32 = 31;

struct Shared {
    world: World,
    requester: engine::Assignment,
    members: [(u64, u64); 3],
    start: Time,
}

fn sign_in(world: &mut World, identity: u64) -> (u64, u64) {
    let key = forge_top::user_key(1, identity);
    world.send(engine::Event::Party(engine::PartyInput::SignedIn {
        until: None,
        held: None,
        reply_to: ReplyTo::new(Token::new(identity)),
        identity: people::Identity {
            key: key.clone(),
            login: b"member".as_slice().into(),
            name: b"Member".as_slice().into(),
        },
    }));
    settle(world);
    let person = world
        .store
        .rows
        .values()
        .find_map(|row| match row {
            Record::Core(jig_domain_core::Record::People(people::Stored::Person { number, identity }))
                if identity.key == key =>
            {
                Some(*number)
            }
            _ => None,
        })
        .expect("member identity saved");
    (person, world.signed_in.expect("member signed in"))
}

fn account(number: u32, provider: u32, person: u64, uses: Option<Box<[accounts::Use]>>) -> Record {
    Record::Core(jig_domain_core::Record::Accounts(accounts::Stored::Account {
        number,
        provider,
        holder: accounts::Holder::Party(person),
        place: number,
        uses,
        subject: Box::new([]),
        display: Box::new([]),
        list: None,
    }))
}
fn source(account: u32, number: u32, kind: accounts::Kind) -> Record {
    Record::Core(jig_domain_core::Record::Accounts(accounts::Stored::Source {
        account,
        number,
        place: 0,
        kind,
        name: Box::new([]),
        held: None,
    }))
}

fn shared(project_key: bool) -> Shared {
    // All three sessions cover this world's interleaved, multi-hour sittings.
    let world = status_world_with_sessions(true, true, None, Duration::from_secs(8 * 3600));
    let (mut world, requester) = opened_chat_in(world);
    let owner = (1, world.signed_in.expect("owner signed in"));
    let second = sign_in(&mut world, 8);
    let third = sign_in(&mut world, 9);
    let members = [owner, second, third];
    world.signed_in = Some(owner.1);
    let first_role = standing_role(&mut world, 110);
    let second_role = standing_role(&mut world, 111);
    drain_api(&mut world);
    seed_note(&mut world, 1, first_role, b"now", b"Repairing the tests that fail together.");
    seed_note(&mut world, 2, second_role, b"now", b"Watching the landing queue.");
    let mut rows = vec![Record::Core(jig_domain_core::Record::People(people::Stored::Roles {
        project: PROJECT,
        holdings: members
            .iter()
            .map(|(person, _)| people::Holding { person: *person, role: people::Role::Owner })
            .collect(),
    }))];
    // Replace the world's deployment-wide key: it would conceal the missing project uses.
    let mut erases = Vec::new();
    let mut provider = None;
    for row in world.store.rows.values() {
        if let Record::Core(jig_domain_core::Record::Accounts(row)) = row {
            if let accounts::Stored::Account { provider: got, .. } = row {
                provider = Some(*got);
            }
            erases.push(Key::Core(jig_domain_core::Key::Accounts(row.key())));
        }
    }
    let provider = provider.expect("fixture provider");
    for (index, (person, _)) in members.iter().enumerate() {
        let number = 10 + u32::try_from(index).expect("three members") * 4;
        rows.push(account(number, provider, *person, None));
        rows.push(source(number, number + 1, accounts::Kind::SignIn));
        rows.push(account(number + 2, provider, *person, None));
        rows.push(source(number + 2, number + 3, accounts::Kind::Key));
    }
    if project_key {
        rows.push(account(PROJECT_KEY, provider, owner.0, Some(Box::new([accounts::Use::Project(PROJECT)]))));
        rows.push(source(PROJECT_KEY, PROJECT_SOURCE, accounts::Kind::Key));
    }
    rows.push(Record::Core(jig_domain_core::Record::Accounts(accounts::Stored::Next { number: 32 })));
    for (number, kind, severity, title) in [
        (1, forge_top::FindingKind::Qa, forge_top::Severity::High, b"Split refunds lose a cent".as_slice()),
        (2, forge_top::FindingKind::Flake, forge_top::Severity::Low, b"Tests fail only together".as_slice()),
        (3, forge_top::FindingKind::Red, forge_top::Severity::Medium, b"CI failed without a change fault".as_slice()),
    ] {
        rows.push(Record::Forge {
            row: Box::new(forge_top::Stored::Finding(forge_top::Finding {
                project: PROJECT,
                number,
                kind,
                severity,
                title: title.into(),
                state: forge_top::FindingState::New,
                area: None,
                body: Box::new([]),
                filer: forge_top::Filer::Party(owner.0),
                at: world.env.wall,
                why: None,
                hander: None,
                goal: None,
                reason: None,
                signature: None,
                seen: None,
            })),
        });
    }
    world.store.seed(rows.into_boxed_slice(), erases.into_boxed_slice());
    restart_status_world(&mut world);
    let start = world.env.now;
    Shared { world, requester, members, start }
}

impl Shared {
    fn at(&mut self, minutes: u64) {
        self.world.env.now = self.start.saturating_add(Duration::from_secs(minutes * 60));
        self.world.env.wall = Wall::from_nanos(self.world.env.now.as_nanos());
        // The fake credential provider renews the forge grant across clock jumps.
        self.world.send(engine::Event::Credential { number: 1, generation: 1, valid: Duration::from_secs(3600) });
    }
    fn look(&mut self, member: usize, minutes: u64, key: u8) {
        self.at(minutes);
        let (party, session) = self.members[member];
        self.world.signed_in = Some(session);
        let at = self.world.env.wall;
        looked(&mut self.world, key);
        let Some(Record::Forge { row }) =
            self.world.store.rows.get(&Key::Forge(forge_top::Key::Looked { party, project: PROJECT }))
        else {
            panic!("member's look saved")
        };
        assert!(matches!(row.as_ref(), forge_top::Stored::Looked(row) if row.party == party && row.latest == at));
    }
    fn land(&mut self, minutes: u64, number: u64, landed: u64) {
        self.at(minutes);
        self.world.signed_in = Some(self.members[0].1);
        land_numbered(&mut self.world, &self.requester, number, landed);
    }
    fn first_run(&mut self) {
        self.look(1, 0, 180);
        assert!(!self.world.store.rows.contains_key(&Key::Forge(forge_top::Key::ProjectFacts { project: PROJECT })));
        self.land(10, 93, 1);
        self.look(2, 40, 181);
        assert!(
            !self
                .world
                .store
                .rows
                .contains_key(&Key::Forge(forge_top::Key::PersonBrief { party: self.members[2].0, project: PROJECT })),
            "a member's first look starts none"
        );
        self.land(60, 94, 2);
        self.look(1, 80, 182);
    }
    fn document(&mut self, member: usize) -> forge_top::StatusView {
        let Some(forge_top::Document::Status(status)) =
            self.world.oracle_document(self.members[member].1, PROJECT, forge_top::status_subject(PROJECT))
        else {
            panic!("member's status document")
        };
        status
    }
    fn since(&mut self, member: usize, previous: u64, seen: u64) -> forge_top::StatusView {
        let status = self.document(member);
        let previous = self.start.saturating_add(Duration::from_secs(previous * 60));
        assert_eq!(status.previous, Some(Wall::from_nanos(previous.as_nanos())));
        assert_eq!(status.seen, seen);
        status
    }
}

fn procedure_and_agent(world: &World, party: u64) -> (forge_top::StatusRunning, u64) {
    let running = person_brief(world, party).running.clone().expect("look started StatusRun");
    let procedure = running.task.expect("StatusTask saved");
    let row = live_task(world, procedure);
    assert_eq!(row.executor, tasks::Executor::Procedure { connector: 1, code: 5 });
    assert_eq!(row.funder, tasks::Funder::Period { project: PROJECT, period: 1 });
    let delegate = *row.delegates.first().expect("charter 9 delegate made");
    let agent = live_task(world, delegate);
    assert_eq!(agent.executor, tasks::Executor::Agent { charter: 9 });
    let words = core::str::from_utf8(&agent.spec.words).expect("UTF-8 words");
    assert!(words.contains("Open findings: 3\n- qa: 1 high\n- flake: 1 low\n- red: 1 medium\n"), "{words}");
    assert!(words.contains("Repairing the tests that fail together."), "{words}");
    assert!(words.contains("Watching the landing queue."), "{words}");
    (running, delegate)
}
fn assigned(world: &World, party: u64) -> (forge_top::StatusRunning, engine::Assignment) {
    let (running, delegate) = procedure_and_agent(world, party);
    let agent = run(world);
    assert_eq!(agent.task, delegate);
    assert_eq!(live_task(world, delegate).kept_account, Some(PROJECT_KEY));
    assert_eq!(agent.run.sources.first(), Some(&Some(PROJECT_SOURCE)));
    (running, agent)
}
fn heard(
    world: &mut World,
    party: u64,
    running: &forge_top::StatusRunning,
    agent: &engine::Assignment,
) -> forge_top::Brief {
    let before = world.env.wall;
    finish(world, agent);
    let brief = person_brief(world, party).brief.clone().expect("report saved as brief");
    assert!(brief.at >= before && brief.at <= world.env.wall);
    assert_eq!(brief.landed, running.landed);
    assert_eq!(brief.findings, running.findings);
    assert!(brief.findings.is_some());
    assert_eq!(brief.sentence.as_ref(), SENTENCE);
    assert_eq!(brief.lede.as_ref(), b"The review passed and the branch is ready.");
    brief
}
fn new_to(status: &forge_top::StatusView, brief: &forge_top::Brief) {
    assert_eq!(status.brief.as_ref(), Some(brief));
    let previous = status.previous.expect("member's previous sitting");
    assert!(brief.landed > status.seen || brief.at > previous, "Home draws the brief: {status:?}");
}

#[test]
fn a_shared_projects_key_carries_interleaved_looks_through_to_each_members_home() {
    let mut shared = shared(true);
    shared.first_run();
    let a = shared.members[1].0;
    let b = shared.members[2].0;
    let (first, agent_a) = assigned(&shared.world, a);
    assert_eq!(first.landed, 2);
    shared.land(100, 95, 3);
    shared.look(2, 120, 183);
    let (second, agent_b) = assigned(&shared.world, b);
    assert_ne!(first.task, second.task, "A's run does not stop B's");
    let brief_a = heard(&mut shared.world, a, &first, &agent_a);
    new_to(&shared.since(1, 0, 0), &brief_a);
    assert_eq!(shared.document(0).brief, None, "owner has no brief yet");
    assert_eq!(shared.document(2).brief, None, "B has no brief until B's result");
    let brief_b = heard(&mut shared.world, b, &second, &agent_b);
    new_to(&shared.since(2, 40, 1), &brief_b);
    assert_eq!(shared.document(1).brief.as_ref(), Some(&brief_a));
    assert_eq!(brief_b.landed, 3);
    shared.world.close();
}

fn waiting(world: &mut World, party: u64) -> forge_top::StatusRunning {
    let (running, agent) = procedure_and_agent(world, party);
    let row = live_task(world, agent);
    assert_eq!(row.phase, tasks::Phase::Active(tasks::Active::Due), "account wait, not refusal or hold");
    assert_eq!(row.kept_account, None);
    assert!(!world.assigned.iter().any(|run| run.task == agent));
    let reads::Answer::Document(document) = world.party_read(reads::Read::Task { task: agent }) else {
        panic!("waiting task document")
    };
    let documents::Document::Task(document) = *document else { panic!("task document") };
    assert_eq!(
        document.run_wait,
        Some(documents::TaskRunWait::Account { account: None, source: None, why: documents::TaskWaitWhy::Empty })
    );
    assert_eq!(person_brief(world, party).running.as_ref(), Some(&running));
    assert!(person_brief(world, party).brief.is_none());
    running
}

#[test]
fn a_shared_projects_members_plans_and_default_keys_leave_its_brief_waiting() {
    let mut shared = shared(false);
    shared.first_run();
    let first = waiting(&mut shared.world, shared.members[1].0);
    shared.land(100, 95, 3);
    shared.look(2, 120, 183);
    let other = waiting(&mut shared.world, shared.members[2].0);
    assert_ne!(other.task, first.task, "each member has their own waiting run");
    shared.look(2, 200, 184);
    let second = waiting(&mut shared.world, shared.members[2].0);
    assert_ne!(second.task, other.task, "an expired account wait never blocks the next sitting");
    assert_eq!(second.landed, 3);
    for member in 0..3 {
        assert!(shared.document(member).brief.is_none(), "Home keeps its computed words");
    }
    shared.world.close();
}

#[test]
fn a_shared_projects_late_status_report_cannot_replace_the_next_sittings_brief() {
    let mut shared = shared(true);
    shared.first_run();
    let a = shared.members[1].0;
    let (first, late) = assigned(&shared.world, a);
    shared.land(100, 95, 3);
    shared.look(1, 160, 183);
    let (second, agent) = assigned(&shared.world, a);
    assert_ne!(second.task, first.task);
    assert_eq!(second.landed, 3);
    let brief = heard(&mut shared.world, a, &second, &agent);
    new_to(&shared.since(1, 80, 2), &brief);
    assert_eq!(
        live_task(&shared.world, late.task).phase,
        tasks::Phase::Active(tasks::Active::Running { attempt: late.attempt }),
        "the old completion is still held by the worker"
    );
    answer(
        &mut shared.world,
        &late,
        tasks::TaskResult::Report {
            words: b"An obsolete brief.\n\nThese words must not replace the current brief.".as_slice().into(),
        },
    );
    settle(&mut shared.world);
    assert!(
        shared.world.root.tasks().task(first.task.expect("old procedure")).is_none(),
        "late report consumed and procedure ended"
    );
    assert_eq!(person_brief(&shared.world, a).brief.as_ref(), Some(&brief));
    assert!(person_brief(&shared.world, a).running.is_none());
    shared.look(1, 200, 184);
    new_to(&shared.since(1, 160, 3), &brief);
    shared.world.close();
}
