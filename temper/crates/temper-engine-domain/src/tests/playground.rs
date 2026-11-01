//! Root start phases on the domain fixture, with the connector's row supplied
//! at the start seam (`temper/docs/design/draft/playground-reset.md`, 6.1).
use super::resume_fixture::root;
use crate::route::{held, kept, playground};
use crate::{Decision, Delivery, Domain, Limits, Work};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{Duration, Env, Queue, ReplyTo, Time, Token, Wall};
use temper_engine_domain_forge as forge;

const FORGE: u16 = 3;
const PROJECT: u32 = 5;
const ACCOUNT: u32 = 6;
const SOURCE: u32 = 7;
const SIGN_IN: u64 = 10;
const PERSON: u64 = 1;
const REQUEST: Token = Token::new(21);
const CARRIED: u64 = (1_u64 << 63) + 1;
const DEADLINE: Duration = Duration::from_secs(10);

fn row(project: Option<u32>, account: Option<u32>) -> forge::Playground {
    forge::Playground { state: forge::PlaygroundState::Running, forge: FORGE, project, account }
}

fn fixture(carried: bool) -> (Domain, Env<Limits>) {
    let (mut domain, env) = root();
    forge::step(
        &mut domain.forge,
        &crate::translate::environment_forge(&env),
        forge::Event::Restore { record: forge::Stored::Forge(super::resume_fixture::forge_record(FORGE)) },
        &mut Queue::uncapped(),
    );
    domain.forge.configure_demo(Some(forge::DemoConfig {
        project_name: "Playground".into(),
        project_policy: 1,
        provider: 1,
        adoption: forge::Adoption {
            project: 1,
            forge: 1,
            home: true,
            host: b"forge.example".as_slice().into(),
            owner: b"playground".as_slice().into(),
            name: b"widget".as_slice().into(),
            prefix: b"temper/".as_slice().into(),
            role: forge::Role::Owned,
            landing: b"main".as_slice().into(),
            ci: true,
            checks: Box::new([]),
        },
        deadline: DEADLINE,
    }));
    let mut out = Queue::uncapped();
    people::step(
        &mut domain.core.people,
        &Env { now: env.now, wall: env.wall, limits: env.limits.people },
        people::Event::Restore {
            record: people::Stored::SignIn {
                number: SIGN_IN,
                person: PERSON,
                expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()),
            },
        },
        &mut out,
    );
    domain
        .playground_contexts
        .insert(
            REQUEST,
            playground::Context {
                sign_in: SIGN_IN,
                carried: if carried { Some(core::Carried { number: CARRIED, digest: [37; 32] }) } else { None },
            },
        )
        .unwrap();
    (domain, env)
}

fn deliveries(domain: &mut Domain, env: &Env<Limits>, decision: Decision) -> Queue<Delivery> {
    crate::decision::accept_routed(&mut domain.journal, &mut domain.core.counters, &env.limits.journal, decision, true)
        .unwrap();
    while let Some(output) = crate::decision::commit(&mut domain.journal, &env.limits.journal) {
        let crate::JournalOutput::Commit { number, .. } = output else { panic!("store commit") };
        crate::decision::committed(&mut domain.journal, number);
    }
    let mut released = Queue::uncapped();
    let mut out = Queue::uncapped();
    loop {
        crate::decision::resume(&mut domain.journal, &mut released);
        let Some(output) = released.pop() else { break };
        let crate::JournalOutput::Deliver(delivery) = output else { panic!("delivery") };
        out.push(delivery);
    }
    out
}

fn begin(domain: &mut Domain, env: &Env<Limits>, row: forge::Playground) -> Queue<Delivery> {
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    // fe-playground-forge owns saving/restoring this row. Supply exactly the
    // snapshot PlaygroundStart reads, without building a second connector.
    playground::begin(domain, env, &mut decision, REQUEST, row);
    deliveries(domain, env, decision)
}

fn act(domain: &mut Domain) -> (ReplyTo, [u8; 16], people::Act, Option<core::Carried>) {
    let Some(Work::Core(core::Event::Act { reply_to, connector, key, act, carried })) = domain.work.pop() else {
        panic!("deployment act")
    };
    assert_eq!(connector, domain.config.forge_connector);
    (reply_to, key, *act, carried)
}

fn acted(domain: &mut Domain, env: &Env<Limits>, to: ReplyTo, outcome: people::Outcome) -> Queue<Delivery> {
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    held::held(domain, env, &mut decision, core::Held::Acted { to, outcome });
    let mut out = deliveries(domain, env, decision);
    let Some(Delivery::DemoActed { to, outcome }) = out.pop() else { panic!("durable act reply") };
    assert!(out.is_empty());
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    kept::acted(domain, env, &mut decision, to, outcome);
    deliveries(domain, env, decision)
}

fn connected(domain: &mut Domain, env: &Env<Limits>) {
    let mut out = Queue::uncapped();
    forge::step(
        &mut domain.forge,
        &Env { now: env.now, wall: env.wall, limits: env.limits.forge },
        forge::Event::Connection {
            forge: FORGE,
            state: forge::Connection::Connected { id: 1, login: b"bot".as_slice().into() },
        },
        &mut out,
    );
}

fn account(domain: &mut Domain, env: &Env<Limits>, number: u32) {
    domain.core.accounts =
        accounts::Domain::new(&env.limits.accounts, super::resume_fixture::config(7).core.settings.llm_providers);
    let mut out = Queue::uncapped();
    for record in [
        accounts::Stored::Account {
            number,
            provider: 1,
            holder: accounts::Holder::Deployment,
            place: number,
            uses: Some(Box::new([accounts::Use::Party(PERSON), accounts::Use::Project(PROJECT)])),
            subject: Box::new([]),
            display: Box::new([]),
        },
        accounts::Stored::Source {
            account: number,
            number: number.checked_add(1).unwrap(),
            place: 1,
            kind: accounts::Kind::Key,
            name: Box::new([]),
            held: None,
        },
    ] {
        accounts::step(
            &mut domain.core.accounts,
            &Env { now: env.now, wall: env.wall, limits: env.limits.accounts },
            accounts::Event::Restore { record },
            &mut out,
        );
    }
    assert!(domain.core.accounts.account(number).is_some());
}

fn answer(domain: &mut Domain) -> forge::StartAnswer {
    let Some(Work::Forge(forge::Event::PlaygroundAnswered { answer })) = domain.work.pop() else {
        panic!("start answer")
    };
    assert!(domain.work.is_empty());
    assert!(domain.playground_start.is_none());
    answer
}

fn adoption(domain: &mut Domain) -> ReplyTo {
    let Some(Work::Core(core::Event::Application { reply_to, sign_in, scope, carried, .. })) = domain.work.pop() else {
        panic!("adoption admission")
    };
    assert_eq!(sign_in, SIGN_IN);
    assert_eq!(scope, people::ApplicationScope::Project(PROJECT));
    assert!(carried.is_none());
    let Some((_, crate::Application::Forge(forge::Ask::Adopt(adoption)))) = &domain.application else {
        panic!("adoption")
    };
    assert_eq!((adoption.forge, adoption.project), (FORGE, PROJECT));
    assert_eq!(adoption.owner.as_ref(), b"playground");
    assert_eq!(adoption.name.as_ref(), b"widget");
    assert!(domain.work.is_empty());
    reply_to
}

fn repository() -> forge::Repository {
    forge::Repository {
        project: PROJECT,
        home: true,
        provider: skein_forge::api::Repository { forge: FORGE, repository: 1 },
        host: b"forge.example".as_slice().into(),
        owner: b"playground".as_slice().into(),
        name: b"widget".as_slice().into(),
        prefix: b"temper/".as_slice().into(),
        role: forge::Role::Owned,
        ci: true,
        checks: Box::new([]),
        kinds: forge::Kinds {
            read: true,
            push: true,
            open: true,
            land: true,
            review: true,
            status: true,
            comment: true,
            issue: true,
            branch: true,
        },
        protection: forge::Protection::Absent,
        settings: skein_forge::api::Settings {
            default_branch: b"main".as_slice().into(),
            merge: true,
            squash: true,
            rebase: false,
        },
    }
}

#[test]
fn a_start_makes_the_project_then_the_account_serving_it_alone() {
    let (mut domain, env) = fixture(true);
    assert!(begin(&mut domain, &env, row(None, None)).is_empty());
    let (to, _, project, carried) = act(&mut domain);
    assert_eq!(project, people::Act::MakeProject { maker: PERSON, name: "Playground".into(), policy: 1 });
    let saved = crate::Record::Core(core::Record::People(people::Stored::Acted {
        key: people::ActKey { connector: domain.config.forge_connector, key: playground::start_key(FORGE, b"project") },
        act: project,
        outcome: people::Outcome::ProjectMade { project: PROJECT },
        at: env.wall,
    }));
    assert_eq!(crate::record_bytes(&saved), Some(10));
    assert!(carried.is_none());
    assert!(acted(&mut domain, &env, to, people::Outcome::ProjectMade { project: PROJECT }).is_empty());
    let Some(Work::Forge(forge::Event::PlaygroundProject { project: PROJECT })) = domain.work.pop() else {
        panic!("project saved")
    };
    let (to, _, account_act, carried) = act(&mut domain);
    assert_eq!(
        account_act,
        people::Act::AddKey {
            provider: 1,
            join: None,
            digest: [37; 32],
            uses: Some(Box::new([people::Use::Party(PERSON), people::Use::Project(PROJECT)])),
        }
    );
    assert_eq!(carried.unwrap().number, CARRIED);
    account(&mut domain, &env, ACCOUNT);
    assert!(acted(&mut domain, &env, to, people::Outcome::AccountKeyKept { source: SOURCE }).is_empty());
    let Some(Work::Forge(forge::Event::PlaygroundAccount { account: ACCOUNT })) = domain.work.pop() else {
        panic!("account saved")
    };
    assert!(domain.work.is_empty());
    connected(&mut domain, &env);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    playground::progress(&mut domain, &env, &mut decision);
    assert!(deliveries(&mut domain, &env, decision).is_empty());
    let _to = adoption(&mut domain);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    playground::adopted(
        &mut domain,
        &env,
        &mut decision,
        Some(forge::Answered::Adopt(forge::ApplicationOutcome::Adopted(forge::AdoptedRepository {
            repository: repository().provider,
            owner: b"playground".as_slice().into(),
            name: b"widget".as_slice().into(),
            role: forge::Role::Owned,
            home: true,
            kinds: repository().kinds,
            protection: forge::Protection::Absent,
        }))),
    );
    assert!(deliveries(&mut domain, &env, decision).is_empty());
    assert_eq!(answer(&mut domain), forge::StartAnswer::Running);
}

#[test]
fn a_start_with_a_project_kept_scopes_the_account_to_the_starting_person_and_project() {
    let (mut domain, env) = fixture(true);
    assert!(begin(&mut domain, &env, row(Some(PROJECT), None)).is_empty());
    let (_, _, account, carried) = act(&mut domain);
    assert_eq!(
        account,
        people::Act::AddKey {
            provider: 1,
            join: None,
            digest: [37; 32],
            uses: Some(Box::new([people::Use::Party(PERSON), people::Use::Project(PROJECT)])),
        }
    );
    assert_eq!(carried.unwrap().number, CARRIED);
}

#[test]
fn a_start_with_project_and_account_kept_only_adopts() {
    for carried in [false, true] {
        let (mut domain, env) = fixture(carried);
        connected(&mut domain, &env);
        let mut out = begin(&mut domain, &env, row(Some(PROJECT), Some(ACCOUNT)));
        if carried {
            let Some(Delivery::CarriedUnused { number: CARRIED }) = out.pop() else { panic!("unused key") };
        }
        assert!(out.is_empty());
        let to = adoption(&mut domain);
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        held::held(
            &mut domain,
            &env,
            &mut decision,
            core::Held::PeopleReply { to, reply: people::Reply::Refused(people::Refusal::Unknown) },
        );
        assert!(deliveries(&mut domain, &env, decision).is_empty());
        assert_eq!(answer(&mut domain), forge::StartAnswer::Failed(forge::StartFailure::Adoption));
    }
}

#[test]
fn the_start_waits_for_the_fake_forge_until_its_deadline() {
    let (mut domain, mut env) = fixture(false);
    assert!(begin(&mut domain, &env, row(Some(PROJECT), Some(ACCOUNT))).is_empty());
    assert!(domain.work.is_empty());
    assert_eq!(domain.next_deadline(&env), Some(Time::ZERO.saturating_add(DEADLINE)));
    env.now = Time::from_nanos(DEADLINE.as_nanos() - 1);
    crate::fire(&mut domain, &env);
    assert!(domain.playground_start.is_some());
    env.now = Time::ZERO.saturating_add(DEADLINE);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    playground::progress(&mut domain, &env, &mut decision);
    assert!(deliveries(&mut domain, &env, decision).is_empty());
    assert_eq!(answer(&mut domain), forge::StartAnswer::Failed(forge::StartFailure::NotConnected));
}

#[test]
#[expect(clippy::wildcard_enum_match_arm, reason = "this assertion rejects every unexpected delivery")]
fn a_missing_key_or_grant_fails_the_start_and_names_why() {
    let (mut domain, env) = fixture(false);
    assert!(begin(&mut domain, &env, row(Some(PROJECT), None)).is_empty());
    assert_eq!(answer(&mut domain), forge::StartAnswer::Failed(forge::StartFailure::Account));
    for (project, grant, failure) in [
        (None, b"make_project".as_slice(), forge::StartFailure::Group { why: b"project".as_slice().into() }),
        (Some(PROJECT), b"add_key".as_slice(), forge::StartFailure::Account),
    ] {
        let (mut domain, env) = fixture(true);
        assert!(begin(&mut domain, &env, row(project, None)).is_empty());
        let (to, _, _, carried) = act(&mut domain);
        if let Some(carried) = carried {
            assert!(playground::carried_unused(&mut domain, carried.number));
        }
        let mut out = acted(&mut domain, &env, to, people::Outcome::Refused(people::Refusal::NotOffered));
        let mut notes = 0_u32;
        let mut unused = 0_u32;
        while let Some(delivery) = out.pop() {
            match delivery {
                Delivery::DemoNote { line } => {
                    let mut found = false;
                    for word in line.windows(grant.len()) {
                        found |= word == grant;
                    }
                    assert!(found);
                    assert!(line.len() <= 256);
                    notes += 1;
                }
                Delivery::CarriedUnused { number: CARRIED } => unused += 1,
                other => panic!("diagnostic or unused: {other:?}"),
            }
        }
        assert_eq!((notes, unused), (1, 1));
        assert_eq!(answer(&mut domain), forge::StartAnswer::Failed(failure));
    }
}

#[test]
fn the_start_acts_carry_keys_derived_from_the_playground_forge() {
    const PROJECT_KEY: [u8; 16] = [174, 235, 169, 192, 222, 183, 69, 53, 145, 76, 32, 23, 174, 57, 14, 11];
    const ACCOUNT_KEY: [u8; 16] = [216, 162, 142, 116, 252, 104, 241, 3, 252, 29, 121, 41, 200, 202, 110, 96];
    for (project, expected, purpose) in
        [(None, PROJECT_KEY, b"project".as_slice()), (Some(PROJECT), ACCOUNT_KEY, b"account".as_slice())]
    {
        let (mut domain, env) = fixture(true);
        assert!(begin(&mut domain, &env, row(project, None)).is_empty());
        let (_, key, _, _) = act(&mut domain);
        assert_eq!(key, expected);
        assert_ne!(key, playground::start_key(FORGE + 1, purpose));
    }
}

#[test]
fn again_waits_for_resume_and_reuses_the_key_and_carried_value() {
    for project in [None, Some(PROJECT)] {
        let (mut domain, mut env) = fixture(true);
        assert!(begin(&mut domain, &env, row(project, None)).is_empty());
        let (to, key, sent, carried) = act(&mut domain);
        if let Some(carried) = carried {
            assert!(playground::carried_unused(&mut domain, carried.number));
        }
        assert!(acted(&mut domain, &env, to, people::Outcome::Again(people::Again::Busy)).is_empty());
        assert!(domain.work.is_empty());
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        playground::progress(&mut domain, &env, &mut decision);
        assert!(deliveries(&mut domain, &env, decision).is_empty());
        assert!(domain.work.is_empty());
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        playground::resume(&mut domain, &env, &mut decision);
        assert!(deliveries(&mut domain, &env, decision).is_empty());
        let (to, repeated, act, value) = act(&mut domain);
        assert_eq!((key, sent, carried), (repeated, act, value));
        assert!(acted(&mut domain, &env, to, people::Outcome::Again(people::Again::Busy)).is_empty());
        env.now = Time::ZERO.saturating_add(DEADLINE);
        let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
        playground::resume(&mut domain, &env, &mut decision);
        let mut out = deliveries(&mut domain, &env, decision);
        let Some(Delivery::CarriedUnused { number: CARRIED }) = out.pop() else { panic!("unused key after busy") };
        assert!(out.is_empty());
        assert_eq!(
            answer(&mut domain),
            forge::StartAnswer::Failed(forge::StartFailure::Group { why: b"busy".as_slice().into() })
        );
    }
}

#[test]
fn a_group_failure_names_only_the_playground_forge_and_bounds_the_reason() {
    let (mut domain, env) = fixture(false);
    assert!(begin(&mut domain, &env, row(Some(PROJECT), Some(ACCOUNT))).is_empty());
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    playground::connection(&mut domain, &env, &mut decision, FORGE + 1, b"other");
    assert!(domain.work.is_empty());
    playground::connection(&mut domain, &env, &mut decision, FORGE, &[b'x'; 257]);
    assert!(deliveries(&mut domain, &env, decision).is_empty());
    assert_eq!(
        answer(&mut domain),
        forge::StartAnswer::Failed(forge::StartFailure::Group { why: Box::new([b'x'; 256]) })
    );
}

#[test]
fn an_existing_adoption_must_match_the_project_forge_owner_and_name() {
    for mismatch in 0_u8..5 {
        let (mut domain, env) = fixture(false);
        connected(&mut domain, &env);
        let mut repository = repository();
        match mismatch {
            0 => {}
            1 => repository.project += 1,
            2 => repository.provider.forge += 1,
            3 => repository.owner = b"other".as_slice().into(),
            4 => repository.name = b"other".as_slice().into(),
            _ => unreachable!(),
        }
        let mut out = Queue::uncapped();
        forge::step(
            &mut domain.forge,
            &Env { now: env.now, wall: env.wall, limits: env.limits.forge },
            forge::Event::Restore { record: forge::Stored::Repository(repository) },
            &mut out,
        );
        assert!(begin(&mut domain, &env, row(Some(PROJECT), Some(ACCOUNT))).is_empty());
        if mismatch == 0 {
            assert_eq!(answer(&mut domain), forge::StartAnswer::Running);
        } else {
            let _to = adoption(&mut domain);
        }
    }
}

#[test]
fn another_start_joins_without_replacing_the_active_context_or_deadline() {
    let (mut domain, mut env) = fixture(true);
    assert!(begin(&mut domain, &env, row(None, None)).is_empty());
    let (to, _, _, _) = act(&mut domain);
    env.now = Time::ZERO.saturating_add(Duration::from_secs(1));
    let mut out = Queue::uncapped();
    out.push(forge::Request::PlaygroundStart { request: Token::new(22) });
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::forge_route::outputs(&mut domain, &env, &mut decision, &mut out);
    assert!(deliveries(&mut domain, &env, decision).is_empty());
    assert!(domain.work.is_empty());
    assert_eq!(domain.playground_start.as_ref().unwrap().until, Time::ZERO.saturating_add(DEADLINE));
    assert!(acted(&mut domain, &env, to, people::Outcome::ProjectMade { project: PROJECT }).is_empty());
    let Some(Work::Forge(forge::Event::PlaygroundProject { project: PROJECT })) = domain.work.pop() else {
        panic!("project")
    };
    let (_, _, _, carried) = act(&mut domain);
    assert_eq!(carried.unwrap().number, CARRIED);
}

#[test]
fn a_group_failure_keeps_the_project_answer_before_failing_the_start() {
    let (mut domain, env) = fixture(true);
    assert!(begin(&mut domain, &env, row(None, None)).is_empty());
    let (to, _, _, _) = act(&mut domain);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    playground::connection(&mut domain, &env, &mut decision, FORGE, b"failed");
    assert!(deliveries(&mut domain, &env, decision).is_empty());
    assert!(domain.work.is_empty(), "an in-flight project must still reach the row");
    let mut out = acted(&mut domain, &env, to, people::Outcome::ProjectMade { project: PROJECT });
    let Some(Work::Forge(forge::Event::PlaygroundProject { project: PROJECT })) = domain.work.pop() else {
        panic!("project saved")
    };
    let Some(Delivery::CarriedUnused { number: CARRIED }) = out.pop() else { panic!("unused key") };
    assert!(out.is_empty());
    assert_eq!(
        answer(&mut domain),
        forge::StartAnswer::Failed(forge::StartFailure::Group { why: b"failed".as_slice().into() })
    );
}

#[test]
fn a_retry_keeps_the_maker_after_the_sign_in_expires() {
    let (mut domain, env) = fixture(true);
    assert!(begin(&mut domain, &env, row(None, None)).is_empty());
    let (to, key, original, _) = act(&mut domain);
    assert!(acted(&mut domain, &env, to, people::Outcome::Again(people::Again::Busy)).is_empty());
    domain.core.people = people::Domain::new(&env.limits.people, Box::new([]), 1, Box::new([]));
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    playground::resume(&mut domain, &env, &mut decision);
    assert!(deliveries(&mut domain, &env, decision).is_empty());
    let (_, retried_key, retried, _) = act(&mut domain);
    assert_eq!((key, original), (retried_key, retried));
}
