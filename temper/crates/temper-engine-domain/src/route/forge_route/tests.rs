use super::{commit_text, landing_words};
use alloc::boxed::Box;

mod ask_dropped;
mod call_room;
mod close_delegate;
mod due_ready;
mod flake;
mod landing_end;
mod limit_holds;
mod reds;
mod refused;
mod release;
mod room;
mod round_ends;
mod settling;
mod settling_due;
mod taking;

const SHA1: [u8; 32] = [
    0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
const SHA256: [u8; 32] = [0xab; 32];
const COMMITS: &[([u8; 32], &[u8])] = &[
    (SHA1, b"abcdef0123456789abcdef0123456789abcdef01"),
    (SHA256, b"abababababababababababababababababababababababababababababababab"),
];

#[test]
fn not_installed_ends_the_roots_adoption_part_as_missing_access() {
    use jig_domain_people as people;
    assert_eq!(
        super::adoption_failure(skein_forge::api::Error::NotInstalled),
        people::Outcome::Refused(people::Refusal::Authority {
            lacking: people::Lack::ResourceAccess,
            proposable: None
        })
    );
    assert_eq!(super::adoption_failure(skein_forge::api::Error::Busy), people::Outcome::Again(people::Again::Busy));
}

pub(super) use crate::tests::owned_repository as adopted;

#[test]
fn tree_branches_count_prefix_slashes_against_authority_segments() {
    // forge, host, owner, repository, branch, temper, root, marker/task.
    const AUTHORITY_SEGMENTS: u32 = 8;
    // The old live prefix puts live-x in a branch segment of its own.
    const DEEP_AUTHORITY_SEGMENTS: u32 = 9;
    const BRANCH_BYTES: u32 = 128;
    for (prefix, segments, fits) in [
        (b"temper/live-x/".as_slice(), AUTHORITY_SEGMENTS, false),
        (b"temper/live-x/", DEEP_AUTHORITY_SEGMENTS, true),
        (b"temper/", AUTHORITY_SEGMENTS, true),
        (b"temper/live-x-", AUTHORITY_SEGMENTS, true),
    ] {
        let mut repository = adopted(1);
        repository.prefix = prefix.into();
        for marker in *b"crs" {
            for attempt in [None, Some(3)] {
                let branch = super::tree_branch(&repository, 1, marker, 2, attempt, BRANCH_BYTES).expect("tree branch");
                let what = super::branch_what(&branch, BRANCH_BYTES).expect("branch parts");
                assert_eq!(super::resource_name(&repository, &what, segments).is_some(), fits, "branch {branch:?}");
            }
        }
    }
}

#[test]
fn delegations_with_one_repository_id_take_their_own_forges_holding() {
    use super::{forge, tasks};
    use alloc::boxed::Box;
    let (mut domain, env) = two_forges();
    let connector = domain.config.forge_connector;
    for number in [1, 3] {
        let spec = tasks::Spec {
            job: None,
            asks: false,
            words: Box::new([]),
            inputs: Box::new([]),
            parameters: Box::new([tasks::Parameter::Resource { name: 1, connector, resource: forge::tag(number, 1) }]),
        };
        let holdings = super::task_holdings(
            &mut domain,
            &env,
            5,
            7,
            8,
            tasks::Executor::Procedure { connector, code: 2 },
            &spec,
            None,
            false,
            &[],
        )
        .expect("delegation admitted");
        let [tasks::Holding::Write { resource, .. }] = holdings.as_ref() else { panic!("one write holding") };
        let name = temper_engine_domain_translate::forge_name(connector, resource, env.limits.forge.name_bytes)
            .expect("forge holding");
        assert_eq!((name.forge, name.repository), (number, 1));
    }
}

#[test]
fn a_change_on_its_requesters_branch_asks_no_write_holding() {
    use super::{forge, tasks};
    let (mut domain, env) = two_forges();
    let connector = domain.config.forge_connector;
    let spec = tasks::Spec {
        parameters: Box::new([
            tasks::Parameter::Resource { name: 1, connector, resource: forge::tag(1, 1) },
            tasks::Parameter::Bytes { name: 2, value: Box::from(b"main".as_slice()) },
            tasks::Parameter::Bytes { name: 3, value: Box::from(b"temper/7/g8".as_slice()) },
            tasks::Parameter::Number { name: 11, value: 1 },
        ]),
        ..goal_spec()
    };
    let holdings = super::task_holdings(
        &mut domain,
        &env,
        GOAL_PROJECT,
        GOAL_ROOT,
        GOAL_CHANGE,
        tasks::Executor::Procedure { connector, code: super::CHANGE_CODE },
        &spec,
        Some(GOAL_TASK),
        false,
        &[],
    )
    .expect("a change on its requester's branch admitted");
    assert!(holdings.is_empty(), "the goal keeps its holding instead of handing it down");
}

const GOAL_PROJECT: u32 = 5;
const GOAL_ROOT: u64 = 7;
const GOAL_TASK: u64 = 8;
const GOAL_DELEGATE: u64 = 9;
const GOAL_CHANGE: u64 = 10;

fn goal_spec() -> super::tasks::Spec {
    super::tasks::Spec { job: None, asks: false, words: Box::new([]), inputs: Box::new([]), parameters: Box::new([]) }
}

#[test]
fn a_tracked_task_naming_its_own_goal_branch_holds_g_in_the_home() {
    use super::{Work, tasks};
    use jig_domain_core::effects::connector;
    let (mut domain, env) = two_forges();
    let forge_connector = domain.config.forge_connector;
    let holdings = super::task_holdings(
        &mut domain,
        &env,
        GOAL_PROJECT,
        GOAL_ROOT,
        GOAL_TASK,
        tasks::Executor::Agent { charter: 1 },
        &goal_spec(),
        None,
        true,
        &[super::GOAL_BRANCH_SYMBOL],
    )
    .expect("tracked goal admitted");
    let expected_goal = tasks::Holding::Write {
        resource: tasks::Name {
            connector: forge_connector,
            path: Box::new([
                Box::from(1_u16.to_be_bytes()),
                Box::from(1_u32.to_be_bytes()),
                Box::from(b"temper/7/g8".as_slice()),
            ]),
        },
        kind: 1,
    };
    assert_eq!(
        holdings.as_ref(),
        &[
            expected_goal.clone(),
            tasks::Holding::Write {
                resource: tasks::Name {
                    connector: forge_connector,
                    path: Box::new([
                        Box::from(1_u16.to_be_bytes()),
                        Box::from(1_u32.to_be_bytes()),
                        Box::from(b"temper/7/r8".as_slice()),
                    ])
                },
                kind: 1,
            }
        ]
    );
    let tasks::Holding::Write { resource: goal_resource, .. } = expected_goal else { unreachable!() };
    let mut reported = false;
    while let Some(work) = domain.work.pop() {
        if let Work::Connector(
            _,
            jig_domain_contract::Up::Resources(jig_domain_contract::resources::Up::Resource {
                project,
                name,
                role,
                hold,
            }),
        ) = work
            && name == goal_resource
        {
            assert_eq!(project, GOAL_PROJECT);
            assert_eq!(role, connector::ResourceRole::Owned);
            assert_eq!(hold, connector::HoldKind::Exclusive { wait: false });
            reported = true;
        }
    }
    assert!(reported, "the goal's resource report makes its kind-1 hold refuse");
}

#[test]
fn an_untracked_task_naming_its_own_goal_branch_is_refused_naming_the_symbol() {
    let (mut domain, env) = two_forges();
    assert_eq!(
        super::task_holdings(
            &mut domain,
            &env,
            GOAL_PROJECT,
            GOAL_ROOT,
            GOAL_TASK,
            super::tasks::Executor::Agent { charter: 1 },
            &goal_spec(),
            None,
            false,
            &[super::GOAL_BRANCH_SYMBOL],
        ),
        Err(Some(jig_domain_core::HoldingsRefusal::Symbol { symbol: super::GOAL_BRANCH_SYMBOL }))
    );
}

#[test]
fn an_unknown_forge_symbol_is_refused_naming_it() {
    const UNKNOWN_SYMBOL: u16 = 99;
    let (mut domain, env) = two_forges();
    assert_eq!(
        super::task_holdings(
            &mut domain,
            &env,
            GOAL_PROJECT,
            GOAL_ROOT,
            GOAL_TASK,
            super::tasks::Executor::Person(super::tasks::PersonAddress::Person(1)),
            &goal_spec(),
            None,
            true,
            &[UNKNOWN_SYMBOL],
        ),
        Err(Some(jig_domain_core::HoldingsRefusal::Symbol { symbol: UNKNOWN_SYMBOL }))
    );
}

#[test]
fn a_symbol_on_a_project_without_a_home_is_refused() {
    const PROJECT_WITHOUT_HOME: u32 = 6;
    let (mut domain, env) = two_forges();
    assert_eq!(
        super::task_holdings(
            &mut domain,
            &env,
            PROJECT_WITHOUT_HOME,
            GOAL_ROOT,
            GOAL_TASK,
            super::tasks::Executor::Agent { charter: 1 },
            &goal_spec(),
            None,
            true,
            &[super::GOAL_BRANCH_SYMBOL],
        ),
        Err(Some(jig_domain_core::HoldingsRefusal::Symbol { symbol: super::GOAL_BRANCH_SYMBOL }))
    );
}

#[test]
fn holdings_asks_keep_the_refused_symbol() {
    use super::tasks;
    use jig_domain_core as core;
    const UNKNOWN_SYMBOL: u16 = 99;
    let (mut domain, env) = two_forges();
    let connector = domain.config.forge_connector;
    let request = skein_lib::Token::new(GOAL_TASK);
    let ask = jig_domain_contract::Ask::Resources(jig_domain_contract::resources::Ask::Holdings {
        request,
        project: GOAL_PROJECT,
        root: GOAL_ROOT,
        number: GOAL_TASK,
        executor: tasks::Executor::Person(tasks::PersonAddress::Person(1)),
        spec: goal_spec(),
        tracked: true,
        symbols: Box::new([UNKNOWN_SYMBOL]),
    });
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::super::asks::ask(&mut domain, &env, &mut decision, connector, ask);
    let Some(super::Work::Connector(
        _,
        jig_domain_contract::Up::Resources(jig_domain_contract::resources::Up::Holdings {
            refused,
            holdings,
            request: answered,
        }),
    )) = domain.work.pop()
    else {
        panic!("holdings answer")
    };
    assert_eq!(answered, request);
    assert_eq!(refused, Some(core::HoldingsRefusal::Symbol { symbol: UNKNOWN_SYMBOL }));
    assert!(holdings.is_none());
    assert!(domain.work.is_empty());
}

fn goal_holds_branch() -> (super::Domain, skein_lib::Env<super::Limits>) {
    goal_holds_branch_beneath(false)
}

#[test]
fn a_closer_waits_for_its_goals_branch_making() {
    use super::{Work, forge, tasks};
    const TAKING: u64 = 21;
    let (mut domain, env) = goal_holds_branch();
    fresh_procedure_forge(&mut domain, &env);
    make_goal_task(&mut domain, &env, GOAL_DELEGATE, tasks::Party::Task(GOAL_TASK), Box::new([]));
    let resource = forge::Name {
        forge: adopted(1).provider.forge,
        repository: adopted(1).provider.repository,
        what: super::branch_what(b"temper/8/g8", env.limits.forge.name_bytes).unwrap(),
    };
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Hold { entry: TAKING, task: GOAL_TASK, resource, from: None },
        &mut skein_lib::Queue::uncapped(),
    );
    let context = domain.engine.core().tasks.run_context(GOAL_DELEGATE).expect("due closer");
    domain.work.clear();

    let mut decision = crate::restart::load_decision(&mut domain, &env.limits);
    super::closer_route::start_closer_step(&mut domain, &env, &mut decision, &context);
    let mut step = None;
    while let Some(work) = domain.work.pop() {
        if let Work::Core(jig_domain_core::Event::ProcedureStep { task: GOAL_DELEGATE, decision, .. }) = work {
            step = Some((decision.end, decision.batch.is_empty()));
        }
    }
    assert_eq!(step, Some((super::StepEnd::Wait, true)));
}

fn goal_holds_branch_beneath(beneath: bool) -> (super::Domain, skein_lib::Env<super::Limits>) {
    use super::tasks;
    let (mut domain, mut env) = two_forges();
    // These tasks report one byte; their delegate result reserves that byte in the inbox.
    env.limits.tasks.result_bytes = 1;
    let holdings = super::task_holdings(
        &mut domain,
        &env,
        GOAL_PROJECT,
        GOAL_TASK,
        GOAL_TASK,
        tasks::Executor::Person(tasks::PersonAddress::Person(1)),
        &goal_spec(),
        None,
        true,
        &[super::GOAL_BRANCH_SYMBOL],
    )
    .expect("goal holding");
    let task_env = env.with(env.limits.tasks);
    let mut out = skein_lib::Queue::uncapped();
    tasks::step(
        &mut domain.engine.core_mut().tasks,
        &task_env,
        tasks::Event::OpenPeriod {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(1)),
            project: GOAL_PROJECT,
            period: 1,
            budget: 100,
        },
        &mut out,
    );
    let creator = if beneath {
        make_goal_task(&mut domain, &env, GOAL_ROOT, tasks::Party::Deployment { project: GOAL_PROJECT }, Box::new([]));
        tasks::Party::Task(GOAL_ROOT)
    } else {
        tasks::Party::Deployment { project: GOAL_PROJECT }
    };
    make_goal_task(&mut domain, &env, GOAL_TASK, creator, holdings);
    (domain, env)
}

#[test]
fn a_nested_goal_holding_its_branch_under_its_own_number_keeps_the_default_destination() {
    use super::forge;
    let (mut domain, env) = goal_holds_branch_beneath(true);
    assert_eq!(domain.engine.core().tasks.root(GOAL_TASK), Some(GOAL_ROOT));
    let name = forge::Name {
        forge: 1,
        repository: 1,
        what: super::branch_what(b"temper/8/g8", env.limits.forge.name_bytes).unwrap(),
    };
    let mut out = skein_lib::Queue::uncapped();
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Hold { entry: 0, task: GOAL_TASK, resource: name.clone(), from: None },
        &mut out,
    );
    assert_eq!(domain.forge.hold(&name).expect("old branch held").task, GOAL_TASK);
    let change = crate::ChangeCall::words(b"Small fix".as_slice().into(), None);
    assert_eq!(change.landing, None);
    let Ok(destination) = super::change_destination(&domain, &env, GOAL_TASK, &adopted(1), &change) else {
        panic!("legacy goal's change admitted")
    };
    assert_eq!(destination.as_ref(), b"main");
}

fn make_goal_task(
    domain: &mut super::Domain,
    env: &skein_lib::Env<super::Limits>,
    number: u64,
    creator: super::tasks::Party,
    holdings: Box<[super::tasks::Holding]>,
) {
    use super::tasks;
    let executor = tasks::Executor::Procedure { connector: 0, code: 1 };
    let (source, _, source_task) = crate::tests::live_task(None, executor);
    let authority = source.engine.core().tasks.task(source_task).unwrap().authority.clone();
    let mut out = skein_lib::Queue::uncapped();
    tasks::step(
        &mut domain.engine.core_mut().tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Make {
            reads_project: false,
            maker: None,
            places: Box::new([]),
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(number)),
            creator,
            batch: Box::new([tasks::New {
                number,
                project: GOAL_PROJECT,
                label: 0,
                flags: tasks::Flags::PLAIN,
                executor,
                llm: None,
                spec: goal_spec(),
                contract: tasks::Contract::Report { words: 1 },
                deadline: tasks::Deadline::Named,
                authority,
                numbers: tasks::Numbers { budget: 1, spent: 0, spent_below: 0, reserved: 0 },
                funder: tasks::Funder::Period { project: GOAL_PROJECT, period: 1 },
                dependencies: Box::new([]),
                holdings,
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: Some(1),
            }]),
        },
        &mut out,
    );
    assert!(domain.engine.core().tasks.task(number).is_some(), "fixture task made");
}

fn change_goal_holdings(
    domain: &mut super::Domain,
    env: &skein_lib::Env<super::Limits>,
    parent: Option<u64>,
    forge: u16,
) -> Result<Box<[super::tasks::Holding]>, Option<jig_domain_core::HoldingsRefusal>> {
    use super::tasks;
    let connector = domain.config.forge_connector;
    let spec = tasks::Spec {
        parameters: Box::new([
            tasks::Parameter::Resource { name: 1, connector, resource: super::forge::tag(forge, 1) },
            tasks::Parameter::Bytes { name: 2, value: Box::from(b"temper/8/g8".as_slice()) },
        ]),
        ..goal_spec()
    };
    super::task_holdings(
        domain,
        env,
        GOAL_PROJECT,
        GOAL_TASK,
        GOAL_CHANGE,
        tasks::Executor::Procedure { connector, code: super::CHANGE_CODE },
        &spec,
        parent,
        false,
        &[],
    )
}

#[test]
fn tracked_ancestry_skips_gatherings_and_plain_goals_have_no_milestone() {
    use super::tasks;
    let executor = tasks::Executor::Procedure { connector: 0, code: 1 };
    let (domain, env, number) = crate::tests::live_task(Some(1), executor);
    assert_eq!(super::tracked_ancestor(&domain, tasks::Party::Task(number)), Some(number));
    assert_eq!(super::milestones::of(&domain, Some(number)), None);
    let mut gathering = domain.engine.core().tasks.task(number).unwrap().clone();
    gathering.executor =
        tasks::Executor::Procedure { connector: jig_domain_core::RECURRING, code: jig_domain_core::GATHERING };
    gathering.flags.ended_by = tasks::EndedBy::Requester;
    let mut domain = crate::Domain::new(crate::tests::resume_fixture::config(7), &env.limits).unwrap();
    let mut out = skein_lib::Queue::uncapped();
    tasks::step(
        &mut domain.engine.core_mut().tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Restore { record: tasks::Stored::Live(Box::new(gathering.clone())) },
        &mut out,
    );
    assert_eq!(super::tracked_ancestor(&domain, tasks::Party::Task(number)), None);
    let mut goal = gathering;
    goal.number = number + 1;
    goal.requester = tasks::Party::Task(number);
    goal.executor = executor;
    tasks::step(
        &mut domain.engine.core_mut().tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Restore { record: tasks::Stored::Live(Box::new(goal)) },
        &mut out,
    );
    assert_eq!(super::tracked_ancestor(&domain, tasks::Party::Task(number + 1)), Some(number + 1));
    assert_eq!(super::milestones::of(&domain, Some(number + 1)), Some(number));
}

#[test]
fn a_change_into_a_goal_branch_from_outside_its_goal_is_refused_naming_the_goal() {
    let (mut domain, env) = goal_holds_branch();
    for parent in [None, Some(GOAL_ROOT)] {
        assert_eq!(
            change_goal_holdings(&mut domain, &env, parent, 1),
            Err(Some(jig_domain_core::HoldingsRefusal::Held { holder: GOAL_TASK }))
        );
    }
    assert!(change_goal_holdings(&mut domain, &env, None, 3).is_ok(), "same branch in another forge is unheld");
}

#[test]
fn a_change_from_inside_the_goals_subtree_into_its_branch_is_admitted() {
    let (mut domain, env) = goal_holds_branch();
    make_goal_task(&mut domain, &env, GOAL_DELEGATE, super::tasks::Party::Task(GOAL_TASK), Box::new([]));
    for parent in [GOAL_TASK, GOAL_DELEGATE] {
        let holdings = change_goal_holdings(&mut domain, &env, Some(parent), 1).expect("goal's change admitted");
        let [super::tasks::Holding::Write { resource, kind: 1 }] = holdings.as_ref() else { panic!("change branch") };
        assert_eq!(resource.path[2].as_ref(), b"temper/8/c10");
    }
}

#[test]
fn a_change_merge_into_a_held_branch_reserves_its_writer_before_authority_and_releases_on_veto() {
    use super::{Work, forge, forge_change, tasks};
    use jig_domain_core::effects::connector;
    const ENTRY: u64 = 11;
    let (mut domain, mut env) = goal_holds_branch();
    // forge, host, owner, repository, branch, temper, root, marker/task.
    env.limits.authority.segments = 8;
    make_goal_task(&mut domain, &env, GOAL_CHANGE, tasks::Party::Task(GOAL_TASK), Box::new([]));
    let mut row = change_row(Some(1));
    row.task = GOAL_CHANGE;
    row.change.task = GOAL_CHANGE;
    row.repository = adopted(1).provider;
    row.base = Box::from(b"temper/8/g8".as_slice());
    let prior = row.change.clone();
    let resource = super::hub_name(
        domain.config.forge_connector,
        &forge::Name {
            forge: row.repository.forge,
            repository: row.repository.repository,
            what: super::branch_what(&row.base, env.limits.forge.name_bytes).unwrap(),
        },
        &env.limits.tasks,
    )
    .unwrap();
    let authority_name = super::resource_name(
        &adopted(1),
        &super::branch_what(&row.base, env.limits.forge.name_bytes).unwrap(),
        env.limits.authority.segments,
    )
    .unwrap();
    restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    while domain.work.pop().is_some() {}
    forge_request(
        &mut domain,
        &env,
        forge::Request::ChangeDecision {
            task: GOAL_CHANGE,
            decision: forge_change::Decision::Effect(forge_change::Effect::Merge { head: SHA1, base: SHA1 }),
            entry: Some(ENTRY),
            evidence: Some(passed_evidence(prior, SHA1, SHA1)),
            close_at: None,
            close_refusal: None,
        },
    );
    assert_eq!(domain.engine.core().tasks.writer(&resource).unwrap().writer, tasks::Writer::Effect { entry: ENTRY });
    let Some(Work::Core(jig_domain_core::Event::EffectStart { owner, .. })) = domain.work.pop() else {
        panic!("the reserved merge goes to authority")
    };
    let Some(super::EffectFlight::Procedure { description, writer, .. }) = domain.forge_effects.get(&owner) else {
        panic!("merge description")
    };
    assert_eq!(writer.as_ref(), Some(&resource));
    assert_eq!(description.effect.name, authority_name, "authority still checks the forge branch's grant");
    assert!(description.effect.additional.is_empty(), "the stored hold name needs no authority grant");
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::effect_ask(
        &mut domain,
        &env,
        &mut decision,
        0,
        jig_domain_contract::effects::Ask::Drop {
            owner,
            answer: super::authority::Answer::Wait,
            findings: Box::new([]),
            why: connector::EffectRefusal::Authority,
        },
    );
    let Some(Work::Tasks(event @ tasks::Event::EffectSettled { .. })) = domain.work.pop() else {
        panic!("authority veto releases its reservation")
    };
    drop(jig_domain_core::step(
        domain.engine.core_mut(),
        &crate::limits::environment_core(&env),
        jig_domain_core::Event::Tasks(event),
    ));
    assert!(domain.engine.core().tasks.writer(&resource).is_none());
}

#[test]
fn saved_resources_keep_the_forge_in_their_path() {
    use super::forge;
    let tags = [forge::tag(1, 1), forge::tag(3, 1)];
    let resources = super::saved_resources(4, &tags).expect("valid tags");
    assert_ne!(resources[0].path, resources[1].path);
    assert_eq!(resources[0].path.as_ref(), &[Box::from(1_u16.to_be_bytes()), Box::from(1_u32.to_be_bytes())]);
    let decoded = super::saved_tags(&resources, 4).expect("valid saved paths");
    assert_eq!(decoded.as_ref(), tags.as_slice());
    let (domain, _) = two_forges();
    for (tag, number) in decoded.iter().zip([1, 3]) {
        assert_eq!(domain.forge.repository_tag(5, *tag).expect("own repository").provider.forge, number);
    }
    assert!(super::saved_resources(4, &[0x0001_0001_0000_0001]).is_none());
    let mut malformed = resources.clone();
    malformed[0].path[0] = Box::from(1_u32.to_be_bytes());
    assert!(super::saved_tags(&malformed, 4).is_none(), "a forge wider than u16 is invalid");
}

fn two_forges() -> (super::Domain, skein_lib::Env<super::Limits>) {
    two_forges_with_link(None)
}

fn two_forges_with_link(link: Option<super::forge::Link>) -> (super::Domain, skein_lib::Env<super::Limits>) {
    use super::forge;
    let mut config = crate::tests::resume_fixture::config(7);
    let first = config.forges.first().expect("configured fixture forge").clone();
    let third = forge::Configured { number: 3, ..first.clone() };
    config.forges = Box::new([first, third]);
    let mut limits = crate::tests::resume_fixture::limits();
    limits.forge.repositories = 3; // Two forges and a second repository on the first.
    limits.forge.client.repositories = 3;
    let (mut domain, env) = crate::tests::resume_fixture::root_with_config_and_limits(config, &limits);
    domain.forge.restore_begin(env.wall);
    for number in [1, 3] {
        restore_forge(
            &mut domain,
            &env,
            forge::Stored::Forge(forge::ForgeRecord {
                person: if number == 1 { link } else { None },
                git_program: None,
                number,
                kind: forge::Kind::Forgejo,
                mode: forge::Mode::Connected,
                state: forge::ForgeState::Kept,
                place: forge::Place {
                    api: forge::Address { host: b"forge.example".as_slice().into(), port: 443, tls: None },
                    git: Box::new([]),
                    web: None,
                },
                read_only: false,
                bot: forge::Bot {
                    login: b"bot".as_slice().into(),
                    name: b"Bot".as_slice().into(),
                    email: Box::new([]),
                },
                asked: false,
                first: None,
            }),
        );
        restore_forge(&mut domain, &env, forge::Stored::Repository(adopted(number)));
    }
    (domain, env)
}

fn forge_party_with_link(
    key: jig_domain_people::IdentityKey,
    link: Option<super::forge::Link>,
) -> (super::Domain, skein_lib::Env<super::Limits>) {
    use jig_domain_people as people;
    let (mut domain, env) = two_forges_with_link(link);
    domain.engine.core_mut().people = people::Domain::new(&env.limits.people, Box::new([]), 1, Box::new([]));
    domain.engine.core_mut().people.start_restore(env.wall);
    let people_env = env.with(env.limits.people);
    for record in [
        people::Stored::Person {
            number: 1,
            identity: people::Identity { key, login: Box::new([]), name: Box::new([]) },
        },
        people::Stored::SignIn { number: 10, person: 1, expires: skein_lib::Wall::from_nanos(100) },
        people::Stored::Roles {
            project: 5,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Maintainer }]),
        },
    ] {
        people::step(
            &mut domain.engine.core_mut().people,
            &people_env,
            people::Event::Restore { record },
            &mut skein_lib::Queue::uncapped(),
        );
    }
    assert!(domain.engine.core_mut().people.start_link());
    domain.engine.core_mut().people.start_sweep();
    assert!(domain.engine.core_mut().people.start_complete(&people_env));
    (domain, env)
}

#[test]
fn a_review_counts_only_from_the_forge_it_was_made_on() {
    use super::forge;
    use skein_forge::api;
    let linked = Some(forge::Link { user: 7, person: 1 });
    for (key, link, forge, author, counted) in [
        (forge::user_key(1, 7), None, 1, 7, true),
        (forge::user_key(1, 7), None, 3, 7, false),
        (forge::user_key(1, 7), None, 1, 8, false),
        (forge::user_key(2, 9), linked, 1, 7, true),
        (forge::user_key(2, 9), linked, 3, 7, false),
    ] {
        let (domain, env) = forge_party_with_link(key, link);
        let mut row = change_row(None);
        row.repository = api::Repository { forge, repository: 1 };
        let evidence = forge::ChangeEvidence {
            reviews: Box::new([api::Review {
                id: 1,
                provenance: api::Provenance::Original,
                revision: 1,
                author,
                verdict: api::Verdict::Approve,
                commit: SHA1,
                key: None,
                body: Box::new([]),
                at: skein_lib::Time::ZERO,
                official: true,
            }]),
            ..passed_evidence(row.change.clone(), SHA1, SHA1)
        };
        let facts = super::landing_reviewers(&domain, &env, &row, &evidence).expect("review facts");
        if counted {
            assert_eq!(
                facts.reviewers.as_ref(),
                &[forge::Reviewer {
                    at: skein_lib::Wall::from_nanos(0),
                    person: 1,
                    role: 1,
                    head: SHA1,
                    verdict: api::Verdict::Approve
                }]
            );
            assert!(facts.unmapped.is_empty());
        } else {
            assert!(
                facts.reviewers.is_empty(),
                "forge {forge}, author {author}: a stranger's review counted: {:?}",
                facts.reviewers
            );
            assert_eq!(
                facts.unmapped.as_ref(),
                &[forge::UnmappedReview { account: author, head: SHA1, verdict: api::Verdict::Approve }]
            );
        }
    }
}

#[test]
fn forge_human_is_known_on_its_forge_and_is_not_its_writer() {
    use super::forge;
    let linked = Some(forge::Link { user: 7, person: 1 });
    let shadowed = Some(forge::Link { user: 7, person: 2 });
    for (key, link, project, forge, writer_forge, counted) in [
        (forge::user_key(1, 7), None, 5, 1, 3, true),
        (forge::user_key(1, 7), None, 5, 1, 1, false),
        (forge::user_key(1, 7), None, 5, 3, 1, false),
        (forge::user_key(2, 7), None, 5, 1, 3, false),
        (jig_domain_people::IdentityKey { provider: 1, subject: forge::user_key(1, 7).subject }, None, 5, 1, 3, false),
        (forge::user_key(2, 9), linked, 5, 1, 3, true),
        (forge::user_key(2, 9), linked, 5, 3, 3, false),
        (forge::user_key(1, 7), shadowed, 5, 1, 3, false),
        (forge::user_key(2, 9), linked, 6, 1, 3, false),
        (forge::user_key(1, 7), shadowed, 6, 1, 3, false),
    ] {
        let (mut domain, env) = forge_party_with_link(key.clone(), link);
        for number in [1, 3] {
            restore_forge(
                &mut domain,
                &env,
                forge::Stored::Writer(skein_forge_client::Writer {
                    forge: number,
                    author: if number == writer_forge { 7 } else { 9 },
                }),
            );
        }
        assert_eq!(super::forge_human(&domain, project, forge, 1), counted, "key {key:?}, writer forge {writer_forge}");
    }
}

#[test]
fn person_forges_names_each_forge_once_with_its_own_user_and_writer() {
    use super::forge;
    let linked = Some(forge::Link { user: 7, person: 1 });
    let shadowed = Some(forge::Link { user: 7, person: 2 });
    for (key, link, writer, second_repository, users) in [
        (forge::user_key(1, 7), None, false, false, [Some(7), None]),
        (forge::user_key(3, 7), None, false, false, [None, Some(7)]),
        (forge::user_key(1, 7), None, true, false, [Some(7), None]),
        (
            jig_domain_people::IdentityKey { provider: 1, subject: forge::user_key(1, 7).subject },
            None,
            false,
            false,
            [None, None],
        ),
        (forge::user_key(1, 7), None, false, true, [Some(7), None]),
        (forge::user_key(2, 9), linked, false, false, [Some(7), None]),
        (forge::user_key(1, 7), shadowed, false, false, [None, None]),
    ] {
        let (mut domain, env) = forge_party_with_link(key.clone(), link);
        let mut repository = adopted(3);
        repository.host = b"other.example".as_slice().into();
        let forge_env = env.with(env.limits.forge);
        restore_in_forge_env(&mut domain, &forge_env, forge::Stored::Repository(repository));
        if writer {
            restore_in_forge_env(
                &mut domain,
                &forge_env,
                forge::Stored::Writer(skein_forge_client::Writer { forge: 1, author: 7 }),
            );
        }
        if second_repository {
            let mut repository = adopted(1);
            repository.provider.repository = 2;
            repository.home = false;
            repository.host = b"another.example".as_slice().into();
            repository.name = b"second".as_slice().into();
            restore_in_forge_env(&mut domain, &forge_env, forge::Stored::Repository(repository));
        }
        assert_eq!(domain.forge.readable_repositories(5).len(), if second_repository { 3 } else { 2 });
        let [first, other] = users;
        assert_eq!(
            super::person_forges(&domain, 5, 1).as_ref(),
            &[
                crate::PersonForge { host: b"forge.example".as_slice().into(), user: first, writer },
                crate::PersonForge { host: b"other.example".as_slice().into(), user: other, writer: false },
            ],
            "key {key:?}, writer {writer}, second repository {second_repository}"
        );
    }
}

#[test]
fn pushed_heads_route_only_to_their_own_forges_hold() {
    use super::{Work, forge};
    let (mut domain, env) = two_forges();
    let forge_env = env.with(env.limits.forge);
    for number in [1, 3] {
        let name = forge::Name {
            forge: number,
            repository: 1,
            what: forge::What::Branch(Box::new([Box::from(b"main".as_slice())])),
        };
        restore_in_forge_env(
            &mut domain,
            &forge_env,
            forge::Stored::Hold(forge::Hold {
                beside: None,
                design_writers: Box::new([]),
                name,
                task: 7,
                writer: Some(forge::Writer::Run { task: 8, attempt: 1 }),
                drift: None,
            }),
        );
    }
    for (number, expected_checked) in [(1, false), (3, true)] {
        super::left(
            &mut domain,
            &env,
            8,
            1,
            Box::new([forge::Pushed { tag: forge::tag(number, 1), commit: [2; 32], checked: expected_checked }]),
        );
        let Some(Work::Forge(forge::Event::Answered { pushed, .. })) = domain.work.pop() else {
            panic!("reported heads")
        };
        let [(name, commit, checked)] = pushed.as_ref() else { panic!("one matching hold") };
        assert_eq!(*checked, expected_checked);
        assert_eq!((name.forge, name.repository, *commit), (number, 1, [2; 32]));
        let Some(Work::Forge(forge::Event::Lost { task: 8, attempt: 1 })) = domain.work.pop() else {
            panic!("writer release follows reported heads")
        };
    }
}

#[test]
fn a_workspace_repository_copies_the_tasks_delivery() {
    let (domain, _) = two_forges();
    for delivered in [Some(SHA1), None] {
        let mut item = domain
            .forge
            .prepare_item(
                skein_forge::api::Repository { forge: 3, repository: 1 },
                b"temper/7".as_slice().into(),
                SHA1,
                7,
            )
            .unwrap();
        item.delivered = delivered;
        let repository =
            super::workspace_repository(item, Some(b"temper/7".as_slice().into()), b"temper/s41".as_slice().into())
                .unwrap();
        assert_eq!(repository.delivered, delivered);
    }
}

#[test]
fn a_parked_answers_checked_push_reaches_the_change_before_lost_writer_reads() {
    use super::{Work, forge, forge_change as change};
    use jig_domain_core as core;
    use jig_domain_host as host;
    use jig_domain_hub_link as hub;
    use skein_lib::{Queue, Token};
    let (mut domain, env) = two_forges();
    let mut row = change_row(None);
    row.delegate = Some((41, change::Delegate::Repair(change::Repair::Ci)));
    let name = forge::Name {
        forge: 3,
        repository: 1,
        what: forge::What::Branch(Box::new([b"temper".as_slice().into(), b"7".as_slice().into()])),
    };
    let forge_env = env.with(env.limits.forge);
    let mut out = Queue::uncapped();
    forge::step(
        &mut domain.forge,
        &forge_env,
        forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) },
        &mut out,
    );
    forge::step(
        &mut domain.forge,
        &forge_env,
        forge::Event::Restore {
            record: forge::Stored::Hold(forge::Hold {
                name: name.clone(),
                task: 7,
                writer: Some(forge::Writer::Run { task: 41, attempt: 1 }),
                beside: None,
                design_writers: Box::new([]),
                drift: None,
            }),
        },
        &mut out,
    );
    let payload = domain.hub_link.retain(crate::Payload::Answer {
        task: 41,
        attempt: 1,
        cumulative: 0,
        read: None,
        refused: Box::new([]),
        end: hub::Ending::Link(host::Ending::Parked { work: host::Work { left: None, saved: None } }),
    });
    domain
        .answer_parts
        .insert(
            payload,
            crate::Parts {
                saved: None,
                pushed: Box::new([forge::Pushed { tag: forge::tag(3, 1), commit: SHA1, checked: true }]),
                preparation: None,
            },
        )
        .unwrap();
    crate::link::answer(&mut domain, Token::new(41), Token::new(1), payload, None, skein_lib::Duration::ZERO);
    let Some(Work::Core(core::Event::AnswerPayload { end, .. })) = domain.work.pop() else { panic!("parked answer") };
    assert_eq!(end, jig_domain_tasks::End::Parked);
    // The core's terminal acknowledgement asks Lost for parked and ended answers alike.
    let connector = domain.config.forge_connector;
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::super::asks::ask(
        &mut domain,
        &env,
        &mut decision,
        connector,
        jig_domain_contract::Ask::Resources(jig_domain_contract::resources::Ask::Lost {
            task: 41,
            attempt: 1,
            resources: Box::new([]),
        }),
    );
    let Some(Work::Forge(finished @ forge::Event::Finished { task: 41, attempt: 1, lost: false })) = domain.work.pop()
    else {
        panic!("the parked attempt finishes before its push report")
    };
    forge::step(&mut domain.forge, &forge_env, finished, &mut out);
    let Some(Work::Forge(answer @ forge::Event::Answered { .. })) = domain.work.pop() else {
        panic!("reported head before Lost")
    };
    forge::step(&mut domain.forge, &forge_env, answer, &mut out);
    let row = domain.forge.change(7).unwrap();
    assert_eq!((row.change.last_checked, row.change.last_checked_by), (Some(SHA1), Some(41)));
    let Some(Work::Forge(lost @ forge::Event::Lost { .. })) = domain.work.pop() else {
        panic!("Lost follows Answered")
    };
    let mut lost_out = Queue::uncapped();
    forge::step(&mut domain.forge, &forge_env, lost, &mut lost_out);
    assert!(lost_out.is_empty(), "the reported writer needs no lost read: {lost_out:?}");
    assert_eq!(domain.forge.hold(&name).unwrap().writer, None);
}

#[test]
fn forge_resource_holds_name_the_branch_holder_and_failed_run() {
    use super::{forge, holds};
    let (domain, _) = two_forges();
    let resource = forge::Name {
        forge: 3,
        repository: 1,
        what: forge::What::Branch(Box::new([Box::from(b"temper".as_slice()), Box::from(b"7".as_slice())])),
    };
    assert!(domain.forge.change(7).is_none(), "an agent task has no change row");
    assert_eq!(holds::taken_words(&domain, &resource, 9).as_ref(), b"branch temper/7 in org/repo is taken by holder 9");
    assert_eq!(
        holds::lost_words(&domain, 7, 4, &resource, b"forge policy refused the operation").as_ref(),
        b"task 7's branch temper/7 in org/repo could not be read again after run 4 ended: forge policy refused the operation"
    );
}

#[test]
fn a_goals_failed_reread_routes_a_resource_hold_with_its_cause() {
    use super::{Work, forge, tasks};
    let (mut domain, env, task) = crate::tests::live_task(None, tasks::Executor::Agent { charter: 1 });
    let resource = forge::Name {
        forge: 1,
        repository: 1,
        what: forge::What::Branch(Box::new([Box::from(b"temper".as_slice()), Box::from(b"goal".as_slice())])),
    };
    let mut out = skein_lib::Queue::uncapped();
    domain.forge.restore_begin(env.wall);
    forge::step(
        &mut domain.forge,
        &env.with(env.limits.forge),
        forge::Event::Restore { record: forge::Stored::Repository(adopted(1)) },
        &mut out,
    );
    while out.pop().is_some() {}
    assert!(domain.forge.change(task).is_none());
    out.push(forge::Request::LostReadFailed {
        task,
        attempt: 4,
        resource,
        reason: Box::from(b"forge policy refused the operation".as_slice()),
    });
    let mut decision = super::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::outputs(&mut domain, &env, &mut decision, &mut out);
    let Some(Work::Tasks(event @ tasks::Event::HoldFrom { .. })) = domain.work.pop() else {
        panic!("reread failure holds the goal");
    };
    task_event(&mut domain, &env, event);
    let tasks::Phase::Held { why, source: Some(source), .. } = &domain.engine.core().tasks.task(task).unwrap().phase
    else {
        panic!("resource holds apply to agent tasks");
    };
    assert_eq!(*why, tasks::Hold::ResourceUnavailable);
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("retained cause") };
    assert_eq!(
        words.as_ref(),
        b"task 1's branch temper/goal in org/repo could not be read again after run 4 ended: forge policy refused the operation; resolve this reason, then task release 1 --hold 1; or task cancel 1 REASON"
    );
}

#[test]
fn writable_repositories_include_only_the_requested_project() {
    let (domain, _) = two_forges();
    assert_eq!(domain.forge.writable_repositories(5).len(), 2);
    assert!(domain.forge.writable_repositories(1).is_empty());
    assert!(domain.forge.readable_repositories(1).is_empty());
    assert_eq!(domain.forge.readable_repositories(5).len(), 2);
}

#[test]
fn writable_repositories_exclude_context_and_missing_push() {
    use super::forge;
    let (mut domain, env) = two_forges();
    let mut context = adopted(1);
    context.role = forge::Role::Context;
    let mut read_only = adopted(3);
    read_only.kinds.push = false;
    for repository in [context, read_only] {
        restore_forge(&mut domain, &env, forge::Stored::Repository(repository));
    }
    assert!(domain.forge.writable_repositories(5).is_empty());
    assert_eq!(domain.forge.readable_repositories(5).len(), 2);
}

#[test]
fn writable_repositories_exclude_an_ended_forge() {
    use super::forge;
    let (mut domain, env) = two_forges();
    let mut record = domain.forge.record(1).expect("kept forge").clone();
    record.state = forge::ForgeState::Ended { secrets_erased: false, box_removed: false };
    restore_forge(&mut domain, &env, forge::Stored::Forge(record));
    let repositories = domain.forge.writable_repositories(5);
    assert_eq!(repositories.len(), 1);
    assert_eq!(repositories[0].provider.forge, 3);
    assert_eq!(domain.forge.readable_repositories(5).len(), 1);
}

use crate::tests::change_row::change_row;

fn task_change_row(task: u64, pull: Option<u64>) -> super::forge::ChangeRow {
    let mut row = change_row(pull);
    row.task = task;
    row.change.task = task;
    row
}

fn restore_forge(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>, record: super::forge::Stored) {
    restore_in_forge_env(domain, &env.with(env.limits.forge), record);
}

fn restore_forge_with_engine_waits(
    domain: &mut super::Domain,
    env: &skein_lib::Env<super::Limits>,
    record: super::forge::Stored,
) {
    restore_in_forge_env(domain, &crate::translate::environment_forge(env), record);
}

// Setup restores the row without routing its requests; tests that inspect requests step directly.
fn restore_in_forge_env(
    domain: &mut super::Domain,
    env: &skein_lib::Env<super::forge::Limits>,
    record: super::forge::Stored,
) {
    super::forge::step(
        &mut domain.forge,
        env,
        super::forge::Event::Restore { record },
        &mut skein_lib::Queue::uncapped(),
    );
}

fn reset_tasks_for_restore(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>, seed: u64) {
    use super::tasks;
    domain.engine.core_mut().tasks = tasks::Domain::new(
        &env.limits.tasks,
        tasks::Kept { revisions: env.limits.settings.revisions, versions: env.limits.settings.versions },
        seed,
        0,
        Box::new([]),
        Box::new([]),
    );
}

fn task_event(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>, event: super::tasks::Event) {
    super::tasks::step(
        &mut domain.engine.core_mut().tasks,
        &env.with(env.limits.tasks),
        event,
        &mut skein_lib::Queue::uncapped(),
    );
}

fn passed_evidence(
    prior: super::forge_change::Change,
    head: [u8; 32],
    base_tip: [u8; 32],
) -> super::forge::ChangeEvidence {
    use super::{forge, forge_change::Status};
    forge::ChangeEvidence {
        behind: None,
        readback_owed: false,
        nearest_write: None,
        touched: forge::Touched::UNKNOWN,
        prior,
        head: Some(head),
        base_tip,
        contains_base: Status::Passed,
        ci: Status::Passed,
        gates: Box::new([]),
        reviews: Box::new([]),
        reviews_complete: true,
        size: None,
    }
}

fn unavailable_evidence(prior: super::forge_change::Change, head: [u8; 32]) -> super::forge::ChangeEvidence {
    use super::{forge, forge_change::Status};
    forge::ChangeEvidence {
        contains_base: Status::Unknown,
        ci: Status::Unknown,
        reviews_complete: false,
        ..passed_evidence(prior, head, [0; 32])
    }
}

#[test]
fn a_change_drift_account_names_its_held_step_and_the_branch_not_cached_ci() {
    use super::{forge, forge_change as change};
    use skein_lib::{Queue, Wall};
    let (mut domain, env) = two_forges();
    let at = Wall::from_nanos(0);
    let expected = [0xab; 32];
    let observed = [0xcd; 32];
    let stale = [0xef; 32];
    let target = change::Judgement {
        landing: 9,
        round: 1,
        pull: 10,
        branch: b"ready/9".as_slice().into(),
        base: b"main".as_slice().into(),
        prefix: stale,
    };
    let name = change_names(None)[0].clone();
    restore_forge(&mut domain, &env, forge::Stored::BranchHead(forge::BranchHead { name, commit: stale, owned: true }));
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    for was in [
        change::State::Checking { head: expected },
        change::State::Gating { head: expected, asked: Box::new([]) },
        change::State::Queued { head: expected, ready_since: at },
        change::State::First { head: expected, ready_since: at },
        change::State::Ready { head: expected, ready_since: at },
        change::State::Judging { head: expected, ready_since: at, target, asked: Box::new([]) },
        change::State::CancellingGates { head: expected },
        change::State::Recreating { head: expected },
        change::State::Landing { head: expected },
        change::State::Updating { from: expected, base: stale, ready_since: at },
        change::State::Preparing { from: expected, main: stale, ready_since: at },
        change::State::Opening { requested: true },
    ] {
        let mut row = change_row(Some(7));
        row.change.last_head = Some(if let change::State::Opening { .. } = was { expected } else { stale });
        row.ci = Some(change::Ci { head: stale, status: change::Status::Passed });
        row.change.state = change::State::Held {
            was: Box::new(was),
            hold: change::Hold::new(change::Cause::BranchMoved, Wall::EPOCH),
        };
        let evidence = passed_evidence(row.change.clone(), observed, stale);
        restore_forge(&mut domain, &env, forge::Stored::Change(Box::new(row)));
        let mut routed = Queue::uncapped();
        routed.push(forge::Request::ChangeDecision {
            task: 7,
            decision: change::Decision::Hold(change::Cause::BranchMoved),
            entry: None,
            evidence: Some(evidence),
            close_at: None,
            close_refusal: None,
        });
        super::outputs(&mut domain, &env, &mut decision, &mut routed);
        let Some(super::Work::Tasks(super::tasks::Event::HoldFrom { source, .. })) = domain.work.pop() else {
            panic!("a hold carries its read to the task");
        };
        let super::tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("drift account") };
        assert_eq!(words.as_ref(), b"its branch moved: expected abababababab, found cdcdcdcdcdcd; resolve this reason, then task release 7 --hold 1; or task cancel 7 REASON");
    }
}

#[test]
fn a_change_drift_account_prefers_the_forges_pair_including_unknown() {
    use super::{forge, forge_change as change};
    let (mut domain, env) = two_forges();
    let name = change_names(None)[0].clone();
    let mut row = change_row(Some(7));
    row.change.last_head = Some([0xef; 32]);
    row.change.state = change::State::Held {
        was: Box::new(change::State::Checking { head: [0xef; 32] }),
        hold: change::Hold::new(change::Cause::BranchMoved, skein_lib::Wall::EPOCH),
    };
    restore_forge(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    for observed in [Some([0xcd; 32]), None] {
        restore_forge(
            &mut domain,
            &env,
            forge::Stored::Hold(forge::Hold {
                beside: None,
                design_writers: Box::new([]),
                name: name.clone(),
                task: 7,
                writer: None,
                drift: Some(forge::Drift {
                    at: env.wall,
                    change: forge::DriftChange::Branch { expected: [0xab; 32], observed },
                }),
            }),
        );
        let expected: &[u8] = if observed.is_some() {
            b"its branch moved: expected abababababab, found cdcdcdcdcdcd"
        } else {
            b"its branch moved: expected abababababab, found unknown"
        };
        assert_eq!(
            super::holds::change_account(&domain, &env, 7, change::Cause::BranchMoved, None).unwrap().as_ref(),
            expected
        );
    }
}

fn change_names(pull: Option<u64>) -> Box<[super::forge::Name]> {
    use super::forge::{Name, What};
    Box::new([
        Name {
            forge: 3,
            repository: 1,
            what: What::Branch(Box::new([Box::from(b"temper".as_slice()), Box::from(b"7".as_slice())])),
        },
        Name {
            forge: 3,
            repository: 1,
            what: match pull {
                Some(number) => What::Pull(number),
                None => What::Branch(Box::new([Box::from(b"main".as_slice())])),
            },
        },
    ])
}

#[test]
fn repair_section_order_puts_every_repairs_reason_before_its_pull() {
    use super::forge_change::{Delegate, Repair};
    use crate::boundary::Work;
    use jig_domain_brief::{GatherEvent, Planned};
    use skein_lib::List;

    for (why, kinds) in [
        (Repair::Ci, [1, 3]),
        (Repair::Semantic, [1, 3]),
        (Repair::Landing { gate: 1 }, [1, 3]),
        (Repair::LandingAt { gate: 1, candidate: SHA1, below: None }, [1, 3]),
        (
            Repair::LandingAt {
                gate: 1,
                candidate: SHA1,
                below: Some(super::forge_change::Below {
                    green: SHA256,
                    changes: [None; 4],
                    by_test: false,
                    top: false,
                }),
            },
            [1, 3],
        ),
        (Repair::Gate(1), [2, 3]),
        (Repair::Review { head: [7; 32] }, [2, 3]),
    ] {
        let (mut domain, env) = two_forges();
        let mut row = change_row(Some(7));
        row.change.last_head = Some(SHA1);
        row.delegate = Some((8, Delegate::Repair(why)));
        restore_forge_with_engine_waits(&mut domain, &env, super::forge::Stored::Change(Box::new(row)));
        crate::assemble::finish_brief_plan(&mut domain, &env, 8, Some(7), List::with_capacity(8), Box::new([]));
        let mut gathered = None;
        let mut partners = List::with_capacity(2);
        let mut request_source = None;
        while let Some(work) = domain.work.pop() {
            if let Work::Brief(GatherEvent::Plan { sections, .. }) = work {
                gathered = Some(sections);
            } else if let Work::Forge(super::forge::Event::PlanBrief { section, source, partner }) = work {
                if let super::forge::BriefSource::Request { .. } = source {
                    request_source = Some(source);
                }
                partners.push((section, partner)).expect("two forge sections");
            }
        }
        if let Repair::Review { head: reviewed } = why {
            assert_eq!(
                request_source,
                Some(super::forge::BriefSource::Request {
                    item: super::forge::BriefItem { repository: 1, number: 7 },
                    head: super::forge::BriefCommit(SHA1),
                    reviewed: super::forge::BriefCommit(reviewed),
                })
            );
        }
        let sections = gathered.expect("repair plan");
        assert_eq!(sections.len(), 2);
        let mut tokens = List::with_capacity(2);
        for (section, expected_kind) in sections.iter().zip(kinds) {
            let Planned::Connector { kind, token, priority, size, limit, .. } = section else {
                panic!("required forge section")
            };
            assert_eq!(*kind, expected_kind, "{why:?} section order");
            assert_eq!(*priority, if *kind == 1 || *kind == 2 { 5 } else { 6 });
            assert_eq!(*size, 0);
            assert_eq!(
                *limit,
                match *kind {
                    1 => domain.limits.brief.budgets.ci,
                    2 => domain.limits.brief.budgets.reviews,
                    _ => domain.limits.brief.budgets.pull,
                }
            );
            tokens.push(*token).expect("two forge sections");
        }
        let tokens = tokens.as_slice();
        assert_eq!(partners.as_slice(), [(tokens[0], None), (tokens[1], Some(tokens[0]))]);
    }
}

#[test]
fn a_ci_repair_keeps_its_failed_step_and_log_end_before_pull_contents_and_optional_context() {
    let brief = planned_ci_repair(super::forge_change::Repair::Ci);
    assert_eq!(brief.sections[6].title.as_ref(), b"CI");
    assert_eq!(brief.sections[7].title.as_ref(), b"Pull request");
}

fn planned_ci_repair(why: super::forge_change::Repair) -> smith_domain_run::Brief {
    use crate::boundary::Work;
    use jig_domain_brief::{Core, GatherEvent, Planned};
    use skein_lib::{List, Writer};
    let (mut domain, mut env) = two_forges();
    domain.limits.brief.sections = 8;
    domain.limits.brief.brief_bytes = 14_592;
    domain.limits.brief.budgets.pull = 4608;
    domain.limits.brief.budgets.ci = 4096;
    env.limits.brief = domain.limits.brief;
    let mut row = change_row(Some(7));
    row.change.last_head = Some(SHA1);
    row.delegate = Some((8, super::forge_change::Delegate::Repair(why)));
    restore_forge_with_engine_waits(&mut domain, &env, super::forge::Stored::Change(Box::new(row)));
    let mut wanted = List::with_capacity(8);
    for (kind, size, priority, required) in [
        (Core::Task, 4608, 0, true),
        (Core::Results, 128, 1, true),
        (Core::TranscriptTail, crate::TRANSCRIPT_TAIL_BYTES, 2, true),
        (Core::Plan, 128, 3, false),
        (Core::Attempts, 128, 4, false),
        (Core::NotesIndex, 64, 4, false),
    ] {
        let mut text = Writer::new(usize::try_from(size).unwrap());
        for _ in 0..size {
            text.put(b"x").unwrap();
        }
        wanted.push(Planned::Core { kind, text: text.finish(), limit: size, priority, required }).unwrap();
    }
    crate::assemble::finish_brief_plan(&mut domain, &env, 8, Some(7), wanted, Box::new([]));
    let mut gathered = None;
    while let Some(work) = domain.work.pop() {
        if let Work::Brief(GatherEvent::Plan { budget, sections, .. }) = work {
            gathered = Some((budget, sections));
        }
    }
    let (budget, mut sections) = gathered.expect("repair plan");
    let mut pull = Writer::new(4608);
    let identity = b"Pull request: #7\nBranch: temper/7\nBase: main\nHead: abcdef0123456789abcdef0123456789abcdef01\nAddress: https://forge.example/org/repo/pulls/7\nTitle: Change\nBody: ";
    pull.put(identity).unwrap();
    for _ in identity.len()..4608 {
        pull.put(b"p").unwrap();
    }
    let pull = pull.finish();
    let mut ci = Writer::new(4096);
    let failure = b"Failed check: / changes (pull_request)\nDescription: CI failed\nLink: /actions/runs/2/jobs/1\nFailed step: 1 changes\nEnd of its log:\n";
    let end = b"\nMissing changes/2.md\nRepair the failed check.\n";
    ci.put(failure).unwrap();
    for _ in failure.len()..4096_usize.checked_sub(end.len()).unwrap() {
        ci.put(b"l").unwrap();
    }
    ci.put(end).unwrap();
    let ci = ci.finish();
    for section in &mut sections {
        if let Planned::Connector { kind, size, .. } = section {
            *size = u32::try_from(if *kind == 3 { pull.len() } else { ci.len() }).unwrap();
        }
    }
    let plan = jig_domain_brief::plan(&sections, budget).expect("bounded repair");
    let mut rendered = List::with_capacity(8);
    for placed in &plan.order {
        let section = &sections[usize::try_from(placed.index).unwrap()];
        match section {
            Planned::Connector { kind: 1, .. } => {
                assert_eq!(placed.size, 4096, "the repair's CI budget survives the maximal pull");
                let kept = &ci[..usize::try_from(placed.size).unwrap()];
                assert!(kept.starts_with(failure));
                assert!(kept.ends_with(end));
            }
            Planned::Connector { kind: 3, .. } => assert_eq!(placed.size, 3392, "pull contents yield first"),
            Planned::Core { limit, .. } => assert_eq!(placed.size, *limit, "other context still fits"),
            Planned::Connector { .. } => panic!("two repair sections"),
        }
        let (title, text): (&[u8], &[u8]) = match section {
            Planned::Connector { kind: 1, .. } => (b"CI", &ci),
            Planned::Connector { kind: 3, .. } => (b"Pull request", &pull),
            Planned::Core { text, .. } => (b"Task", text),
            Planned::Connector { .. } => panic!("two repair sections"),
        };
        rendered
            .push(smith_domain_run::Section {
                title: title.into(),
                text: text[..usize::try_from(placed.size).unwrap()].into(),
            })
            .unwrap();
    }
    assert_eq!(plan.size, budget);
    smith_domain_run::Brief { sections: rendered.into_boxed() }
}

#[test]
fn a_landing_repair_charter_fit_cuts_pull_contents_and_keeps_its_whole_ci() {
    use skein_lib::Duration;
    use smith_domain_run as run;

    let brief = planned_ci_repair(super::forge_change::Repair::LandingAt { gate: 1, candidate: SHA1, below: None });
    let mut ci = None;
    let mut pull = None;
    for section in &brief.sections {
        match section.title.as_ref() {
            b"CI" => ci = Some(section.text.clone()),
            b"Pull request" => pull = Some(section.text.clone()),
            _ => {}
        }
    }
    let ci = ci.expect("CI section");
    let pull = pull.expect("pull section");
    let why = super::forge_change::Repair::LandingAt {
        gate: 1,
        candidate: SHA1,
        below: Some(super::forge_change::Below { green: SHA256, changes: [None; 4], by_test: false, top: false }),
    };
    let mut row = change_row(Some(42));
    row.change.last_head = Some(SHA1);
    let mut failure = repair_failure(SHA1, Some(SHA256));
    failure.report = report_tests();
    let instructions = super::repair_words(&row, why, Some(&failure), &[], 4096).unwrap();
    let mut charter = run::Charter {
        mode: run::Mode::Finite,
        resume: false,
        waiting: Duration::ZERO,
        instructions: instructions.clone(),
        brief,
        conventions: None,
        grants: run::charter::Grants {
            contract: 1,
            wait: false,
            deliver: None,
            save: false,
            tools: run::charter::Tools { inspect: true, modify: true, shell: true },
            agents: false,
            host_tools: Box::new([]),
        },
        outcome: run::outcome::OutcomeSpec { change: None, verdicts: Box::new([]), report: None, failure: None },
        budget: run::Budget { turns: 1, spend: 100, time: Duration::from_secs(60) },
        reserve: run::charter::Reserve { turns: 0, time: Duration::ZERO },
        llm: run::charter::Llm {
            prices: run::Prices { input: 1, cached: 1, cache_write: None, output: 1, unit: 1 },
            dialect: 1,
            account: None,
            endpoint: run::charter::Endpoint(0),
            model: b"model".as_slice().into(),
            effort: None,
            speed: None,
            window: 65_536,
            output: 256,
        },
        models: Box::new([]),
        check_timeout: None,
    };
    let bound = charter.owned_bytes().unwrap().checked_sub(1024).unwrap();
    temper_protocol_brief::fit_brief_bytes(&mut charter, bound);
    assert!(charter.owned_bytes().unwrap() <= bound);
    assert_eq!(charter.brief.sections.len(), 8, "no section is omitted");
    assert_eq!(charter.instructions, instructions);
    assert!(
        skein_lib::bytes::find(&charter.instructions, b"Failed test: test skein-lib::suite fuzzy::a (fuzzy)").is_some()
    );
    for section in &charter.brief.sections {
        match section.title.as_ref() {
            b"CI" => assert!(section.text == ci, "the failed step and log end survive charter fitting whole"),
            b"Pull request" => {
                assert!(section.text.len() < pull.len(), "pull contents yield to the charter bound");
                assert!(section.text.starts_with(b"Pull request: #7\n"));
                assert!(section.text.ends_with(b" bytes; whole: read_issues]\n"));
            }
            _ => {}
        }
    }
}

#[test]
fn a_github_repair_prepares_when_the_ci_ceiling_fills_the_brief() {
    use crate::boundary::{ForgeBriefKind, Work};
    use jig_domain_brief::{Core, GatherDomain, GatherEvent, GatherRequest, Planned, gather_step};
    use skein_lib::{Id, List, Queue};
    let (mut domain, mut env) = two_forges();
    domain.limits.brief.sections = 3;
    domain.limits.brief.read_bytes = 512;
    domain.limits.brief.brief_bytes = 512;
    domain.limits.brief.budgets.pull = 512;
    domain.limits.brief.budgets.ci = 512;
    env.limits.brief = domain.limits.brief;
    let mut row = change_row(Some(7));
    row.change.last_head = Some(SHA1);
    row.delegate = Some((8, super::forge_change::Delegate::Repair(super::forge_change::Repair::Ci)));
    restore_forge_with_engine_waits(&mut domain, &env, super::forge::Stored::Change(Box::new(row)));
    let mut wanted = List::with_capacity(3);
    wanted
        .push(Planned::Core {
            kind: Core::Task,
            text: b"Repair failed GitHub check check.\n".as_slice().into(),
            limit: 512,
            priority: 0,
            required: true,
        })
        .unwrap();
    crate::assemble::finish_brief_plan(&mut domain, &env, 8, Some(7), wanted, Box::new([]));
    let limits = crate::limits::brief_limits(&env.limits.brief);
    let gather_env = env.with(limits);
    let mut gather = GatherDomain::new(&limits);
    let mut out = Queue::uncapped();
    while let Some(work) = domain.work.pop() {
        if let Work::Brief(plan @ GatherEvent::Plan { .. }) = work {
            gather_step(&mut gather, &gather_env, plan, &mut out);
        }
    }
    let pull = b"Pull request: #7\nBranch: temper/7\nBase: main\nHead: abcdef0123456789abcdef0123456789abcdef01\nAddress: https://github.com/org/repo/pull/7\n";
    let ci = b"Failed check: check\nDescription: CI failed\nLink: /actions/runs/2/jobs/1\n[job log could not be read]\nRepair the failed check.\n";
    assert_eq!(out.len(), 2, "gather both required forge sections");
    for _ in 0..out.len() {
        let Some(GatherRequest::Gather { section, budget: 512, .. }) = out.pop() else { panic!("bounded forge read") };
        let kind = domain.brief_connectors.get(Id::from_token(section)).unwrap().kind.clone();
        let size = match kind {
            ForgeBriefKind::Pull => pull.len(),
            ForgeBriefKind::Ci => ci.len(),
            ForgeBriefKind::Home | ForgeBriefKind::Reviews | ForgeBriefKind::Document(_) => panic!("pull and CI only"),
        };
        gather_step(
            &mut gather,
            &gather_env,
            GatherEvent::Ready { brief: skein_lib::Token::new(8), section, size: u32::try_from(size).unwrap() },
            &mut out,
        );
    }
    let Some(GatherRequest::Complete { order, .. }) = out.pop() else {
        panic!("the actual task, pull identity and CI sizes fit the unchanged 512-byte brief")
    };
    assert_eq!(order.len(), 3);
    assert!(out.is_empty(), "neither required section needs a cut or a refusal");
}

#[test]
fn a_changes_saved_pull_replaces_its_base_watch_within_two_names() {
    use super::{Decision, Work, forge};
    let (mut domain, env, mut out) = restored_change([forge::Stored::Names { task: 7, resources: change_names(None) }]);
    let forge_env = env.with(env.limits.forge);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    for pull in [None, Some(23)] {
        let mut row = change_row(pull);
        row.change.last_head = Some([1; 32]);
        let mut held = row.clone();
        held.pull = Some(24);
        held.change.last_head = None;
        forge::step(
            &mut domain.forge,
            &forge_env,
            forge::Event::Restore { record: forge::Stored::Change(Box::new(held)) },
            &mut out,
        );
        out.push(forge::Request::Save {
            record: forge::Stored::ChangeHot(Box::new(row.split().1)),
            release: None,
            read_afresh: None,
        });
        super::outputs(&mut domain, &env, &mut decision, &mut out);
        if pull.is_none() {
            assert!(domain.work.is_empty(), "before the pull, keep branch and base");
        }
    }
    let Some(Work::Forge(event @ forge::Event::Rename { .. })) = domain.work.pop() else {
        panic!("name the known pull")
    };
    let forge::Event::Rename { task, resources } = &event else { panic!("pull watch") };
    assert_eq!(*task, 7);
    assert_eq!(*resources, change_names(Some(23)));
    forge::step(&mut domain.forge, &forge_env, event, &mut out);
    assert_eq!(domain.forge.names(7), Some(change_names(Some(23)).as_ref()));
    assert!(domain.work.is_empty(), "the branch's hold needs no change");
}

#[test]
fn a_restored_changes_wake_names_its_known_pull_again() {
    use super::{Decision, Work, forge};
    // An old store, or a restart between saving the pull and saving its names.
    let mut row = change_row(Some(23));
    row.change.last_head = Some([1; 32]);
    let (mut domain, env, mut out) = restored_change([
        forge::Stored::Change(Box::new(row)),
        forge::Stored::Names { task: 7, resources: change_names(None) },
    ]);
    let forge_env = env.with(env.limits.forge);
    out.push(forge::Request::WakeProcedure { task: 7 });
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    super::outputs(&mut domain, &env, &mut decision, &mut out);
    let Some(Work::Forge(event @ forge::Event::Rename { .. })) = domain.work.pop() else {
        panic!("restore pull watch")
    };
    let forge::Event::Rename { task, resources } = &event else { panic!("pull watch") };
    assert_eq!(*task, 7);
    assert_eq!(*resources, change_names(Some(23)));
    forge::step(&mut domain.forge, &forge_env, event, &mut out);
    let Some(Work::Tasks(super::tasks::Event::WakeProcedure { task: 7 })) = domain.work.pop() else {
        panic!("wake the change's procedure")
    };
    assert!(domain.work.is_empty());
    // Later wakes preserve the established watch's polling cadence.
    for _ in 0..out.len() {
        drop(out.pop());
    }
    out.push(forge::Request::WakeProcedure { task: 7 });
    super::outputs(&mut domain, &env, &mut decision, &mut out);
    let Some(Work::Tasks(super::tasks::Event::WakeProcedure { task: 7 })) = domain.work.pop() else {
        panic!("wake the change's procedure")
    };
    assert!(domain.work.is_empty());
}

/// A change's restored records with its forge state settled and the outputs drained.
fn restored_change(
    records: impl IntoIterator<Item = super::forge::Stored>,
) -> (super::Domain, skein_lib::Env<super::Limits>, skein_lib::Queue<super::forge::Request>) {
    use super::forge;
    let (mut domain, mut env) = two_forges();
    env.limits.forge.resources_per_task = 2;
    let forge_env = env.with(env.limits.forge);
    let mut out = skein_lib::Queue::uncapped();
    for record in records {
        forge::step(&mut domain.forge, &forge_env, forge::Event::Restore { record }, &mut out);
    }
    forge::step(
        &mut domain.forge,
        &forge_env,
        forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
        &mut out,
    );
    while let Some(request) = out.pop() {
        drop(request);
    }
    (domain, env, out)
}

#[test]
fn a_finding_blame_mark_reaches_the_core_as_a_milestone_parts_mark() {
    use super::{Decision, Work, forge};
    let (mut domain, env) = two_forges();
    let mut out = skein_lib::Queue::uncapped();
    out.push(forge::Request::MilestoneMarked { task: 812 });
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    super::outputs(&mut domain, &env, &mut decision, &mut out);
    let Some(Work::Connector(
        jig_domain_contract::Connector(connector),
        jig_domain_contract::Up::Parts(jig_domain_contract::parts::Up::Marked { tasks }),
    )) = domain.work.pop()
    else {
        panic!("finding marks the milestone's parts")
    };
    assert_eq!(connector, domain.config.forge_connector);
    assert_eq!(tasks.as_slice(), &[812]);
    assert!(domain.work.is_empty());
}

#[test]
fn an_ended_changes_pull_wake_names_nothing() {
    use super::{Decision, Work, forge};
    // Its release unnamed it: the change row stays, its names row does not.
    let (mut domain, env, mut out) = restored_change([forge::Stored::Change(Box::new(change_row(Some(23))))]);
    out.push(forge::Request::WakeProcedure { task: 7 });
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    super::outputs(&mut domain, &env, &mut decision, &mut out);
    let Some(Work::Tasks(super::tasks::Event::WakeProcedure { task: 7 })) = domain.work.pop() else {
        panic!("wake the change's procedure")
    };
    assert!(domain.work.is_empty(), "an ended change watches nothing");
    assert_eq!(domain.forge.names(7), None);
}

#[test]
fn a_change_unnamed_after_its_pull_swap_was_queued_stays_unnamed() {
    use super::{Decision, Work, forge};
    let (mut domain, env, mut out) = restored_change([
        forge::Stored::Change(Box::new(change_row(Some(23)))),
        forge::Stored::Names { task: 7, resources: change_names(None) },
    ]);
    let forge_env = env.with(env.limits.forge);
    out.push(forge::Request::WakeProcedure { task: 7 });
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    super::outputs(&mut domain, &env, &mut decision, &mut out);
    let Some(Work::Forge(event @ forge::Event::Rename { .. })) = domain.work.pop() else { panic!("queue pull watch") };
    // The release completes before the queued swap applies.
    forge::step(&mut domain.forge, &forge_env, forge::Event::Unnamed { task: 7 }, &mut out);
    assert_eq!(domain.forge.names(7), None);
    while let Some(request) = out.pop() {
        drop(request);
    }
    forge::step(&mut domain.forge, &forge_env, event, &mut out);
    assert_eq!(domain.forge.names(7), None, "the swap revives no watch");
    assert!(out.is_empty());
}

#[test]
fn an_erased_tasks_unname_reaches_the_forge_as_unnamed() {
    use super::{Work, forge};
    let (mut domain, env, _) = restored_change([]);
    let connector = domain.config.forge_connector;
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::asks::ask(
        &mut domain,
        &env,
        &mut decision,
        connector,
        jig_domain_contract::Ask::Resources(jig_domain_contract::resources::Ask::Unname { task: 7 }),
    );
    let Some(Work::Forge(forge::Event::Ask(jig_domain_contract::Ask::Resources(
        jig_domain_contract::resources::Ask::Unname { task: 7 },
    )))) = domain.work.pop()
    else {
        panic!("the erased task reaches the forge");
    };
    assert!(domain.work.is_empty());
    crate::route::asks::ask(
        &mut domain,
        &env,
        &mut decision,
        connector + 1,
        jig_domain_contract::Ask::Resources(jig_domain_contract::resources::Ask::Unname { task: 7 }),
    );
    assert!(domain.work.is_empty());
}

#[test]
fn a_landed_result_names_the_landing_whole_after_its_pull_identity() {
    for (merge, landed) in [
        (SHA1, b"Landed: pull request #7 merged into release/v1 as abcdef012345.".as_slice()),
        (SHA256, b"Landed: pull request #7 merged into release/v1 as abababababab.".as_slice()),
    ] {
        let (mut domain, env) = two_forges();
        let mut row = change_row(Some(7));
        row.base = Box::from(b"release/v1".as_slice());
        row.change.last_head = Some(merge);
        let identity = landing_words(&merge, 512, domain.forge.repository(row.repository), 7, None);
        let evidence = row.result.words(&merge, 512);
        let edge = u32::try_from(identity.len() + 1 + landed.len()).unwrap();
        restore_forge_with_engine_waits(&mut domain, &env, super::forge::Stored::Change(Box::new(row)));
        let mut expected = skein_lib::Writer::new(identity.len() + 1 + landed.len());
        expected.put(&identity).unwrap();
        expected.put(b"\n").unwrap();
        expected.put(landed).unwrap();
        let prefix = expected.finish();
        let mut expected = skein_lib::Writer::new(prefix.len() + 1 + evidence.bytes.len());
        expected.put(&prefix).unwrap();
        expected.put(b"\n").unwrap();
        expected.put(&evidence.bytes).unwrap();
        assert_eq!(super::change_landing_words(&domain, 7, &merge, 7, 512), expected.finish());
        assert_eq!(super::change_landing_words(&domain, 7, &merge, 7, edge), prefix);
        // One byte short omits the whole Landed line and fits evidence as before.
        let bound = edge - 1;
        let fitted = evidence.fitted(bound - u32::try_from(identity.len()).unwrap() - 1).unwrap();
        let mut expected = skein_lib::Writer::new(identity.len() + 1 + fitted.len());
        expected.put(&identity).unwrap();
        expected.put(b"\n").unwrap();
        expected.put(&fitted).unwrap();
        assert_eq!(super::change_landing_words(&domain, 7, &merge, 7, bound), expected.finish());
        // Without room for the pull line, the merge id and fitted evidence stay unchanged.
        let bound = u32::try_from(identity.len() - 1).unwrap();
        let digits = commit_text(&merge);
        let fitted = evidence.fitted(bound - u32::try_from(digits.len()).unwrap() - 1).unwrap();
        let mut expected = skein_lib::Writer::new(digits.len() + 1 + fitted.len());
        expected.put(&digits).unwrap();
        expected.put(b"\n").unwrap();
        expected.put(&fitted).unwrap();
        assert_eq!(super::change_landing_words(&domain, 7, &merge, 7, bound), expected.finish());
        assert_eq!(super::change_landing_words(&domain, 7, &merge, 7, 7).as_ref(), &digits[..7]);
    }
}

#[test]
fn a_small_landed_result_cuts_only_evidence_and_keeps_both_identity_lines() {
    let (mut domain, env) = two_forges();
    let mut row = change_row(Some(7));
    row.change.last_head = Some(SHA1);
    row.result.head = Some(SHA1);
    row.result.checks.bytes = Box::from(b"build passed; release note passed".as_slice());
    row.result.repair_count = 1;
    row.result.repairs.bytes = Box::from(b"CI failed: release note".as_slice());
    row.result.reviews.bytes = Box::from(b"approved at abcdef0123456789abcdef0123456789abcdef01".as_slice());
    let identity = landing_words(
        &SHA1,
        512,
        domain.forge.repository(row.repository),
        7,
        domain.forge.configured(row.repository.forge),
    );
    restore_forge_with_engine_waits(&mut domain, &env, super::forge::Stored::Change(Box::new(row)));
    let whole = super::change_landing_words(&domain, 7, &SHA1, 7, 512);
    let bound = u32::try_from(identity.len() + 90).unwrap();
    let cut = super::change_landing_words(&domain, 7, &SHA1, 7, bound);
    assert!(cut.starts_with(&identity));
    let mut kept = None;
    for (at, pair) in cut.windows(2).enumerate() {
        if pair == b"\n[" {
            kept = Some(at);
        }
    }
    let kept = kept.expect("cut marker");
    assert!(whole.starts_with(&cut[..kept]));
    let lost = skein_lib::Decimal::of(u64::try_from(whole.len() - kept).unwrap());
    assert_eq!(&cut[kept + 2..cut.len() - b" bytes cut]\n".len()], lost.as_bytes());
    assert!(cut.ends_with(b" bytes cut]\n"));
    assert!(cut.len() <= usize::try_from(bound).unwrap());
}

fn repair_failure(prefix: [u8; 32], base: Option<[u8; 32]>) -> super::forge::LandingFailure {
    super::forge::LandingFailure {
        report: super::forge::ReceiptReport::Unset,
        gate: b"gate".as_slice().into(),
        run: super::forge::ReceiptRun {
            id: 4211,
            number: 998,
            url: b"https://forge.example/o/r/actions/runs/998".as_slice().into(),
        },
        prefix,
        base,
        jobs: Box::new([super::forge::ReceiptJob { job: 9001, attempt: 1, name: b"gate".as_slice().into() }]),
    }
}

#[test]
fn cancelled_resolutions_and_merge_jobs_keep_bounded_procedure_accounts() {
    use super::{forge, forge_change, tasks};
    let long = "é".repeat(256);
    for (kind, name) in [
        (forge_change::Delegate::Resolve { base: SHA1 }, "resolution"),
        (forge_change::Delegate::Merge { base: SHA1, from: Some(SHA256) }, "merge job"),
    ] {
        for reason in [b"another approach".as_slice(), b"".as_slice(), long.as_bytes()] {
            let mut row = change_row(Some(42));
            row.change.state = forge_change::State::Held {
                was: Box::new(forge_change::State::Resolving { requested: true, base: SHA1 }),
                hold: forge_change::Hold::new(forge_change::Cause::Failed, skein_lib::Wall::EPOCH),
            };
            row.delegate = Some((9, kind));
            let (mut domain, env, mut out) = restored_change([forge::Stored::Change(Box::new(row))]);
            forge::step(
                &mut domain.forge,
                &env.with(env.limits.forge),
                forge::Event::DelegateResult {
                    ended_at: None,
                    task: 7,
                    child: 9,
                    status: forge_change::Status::Failed,
                    no_change: false,
                    flake: false,
                    no_test: false,
                    failed: false,
                    lost: false,
                    cancelled: true,
                    words: reason.into(),
                },
                &mut out,
            );
            let decision = super::hold_change_decision(&domain, &env, 7, forge_change::Cause::Failed, None);
            let super::StepEnd::HoldFrom { why: tasks::Hold::Procedure, source } = decision.end else {
                panic!("cancelled {name} needs a procedure hold with its account")
            };
            let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("delegate account") };
            let opening = skein_lib::bytes::concat(&[b"its ", name.as_bytes(), b", task 9, was cancelled"]);
            if reason.is_empty() {
                assert_eq!(words.as_ref(), opening.as_ref());
            } else if reason.len() < 512 {
                assert_eq!(words.as_ref(), skein_lib::bytes::concat(&[&opening, b": ", reason]).as_ref());
            } else {
                assert!(words.starts_with(&skein_lib::bytes::concat(&[&opening, ": é".as_bytes()])));
                assert!(words.ends_with(b"..."));
                assert!(words.len() <= usize::try_from(tasks::HOLD_ACCOUNT_BYTES).unwrap());
                assert!(skein_lib::bytes::is_utf8(words));
            }
        }
    }
}

fn procedure_root() -> (super::Domain, skein_lib::Env<super::Limits>) {
    procedure_root_with_limits(crate::tests::resume_fixture::limits())
}

fn procedure_root_with_limits(mut limits: super::Limits) -> (super::Domain, skein_lib::Env<super::Limits>) {
    procedure_root_with_grants(&mut limits, Box::new([]), &[])
}

fn procedure_root_with_grants(
    limits: &mut super::Limits,
    grants: Box<[super::authority::Grant]>,
    records: &[crate::Record],
) -> (super::Domain, skein_lib::Env<super::Limits>) {
    use super::{authority, forge};
    let mut config = crate::tests::resume_fixture::config(7);
    let kinds = Box::new([
        authority::Executor::Charter(1),
        authority::Executor::Procedure(super::LANDING_CODE),
        authority::Executor::Procedure(super::AFTER_LANDING_CODE),
        authority::Executor::Procedure(super::GOAL_CLOSER_CODE),
        authority::Executor::Job(4),
    ]);
    config.core.defaults.rules.maximum_run_spend = 1000;
    config.core.defaults.rules.ceiling.delegation.kinds = kinds.clone();
    config.core.seed_projects[0].policy.policy.ceiling.delegation.kinds = kinds;
    config.core.defaults.rules.ceiling.grants = grants.clone();
    config.core.seed_projects[0].policy.policy.ceiling.grants = grants;
    // The closer procedure and close job add two executor places.
    limits.authority.executors = 5;
    // The closer procedure and close job add two executor places.
    limits.tasks.executor_kinds = 5;
    limits.tasks.parameters = 4;
    limits.tasks.spec_bytes = 256;
    let (mut domain, env) = crate::tests::resume_fixture::root_with_records(config, limits, records);
    for event in [
        jig_domain_core::Event::Tasks(super::tasks::Event::Restored),
        jig_domain_core::Event::Period { project: 1, period: 1, budget: 1000, next: None },
    ] {
        drop(jig_domain_core::step(domain.engine.core_mut(), &crate::limits::environment_core(&env), event));
    }
    domain.forge.restore_begin(env.wall);
    let mut repository = adopted(1);
    repository.project = 1;
    restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Repository(repository));
    (domain, env)
}

fn fresh_procedure_forge(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>) {
    // Finish the fixture's empty fresh pass before supplying its procedure facts.
    for event in [
        super::forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
        super::forge::Event::CatchUp,
    ] {
        super::forge::step(
            &mut domain.forge,
            &super::super::environment_forge(env),
            event,
            &mut skein_lib::Queue::uncapped(),
        );
    }
    assert!(domain.forge.fresh_reads_in());
}

fn forge_request(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>, request: super::forge::Request) {
    let mut out = skein_lib::Queue::uncapped();
    out.push(request);
    let mut decision = super::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::outputs(domain, env, &mut decision, &mut out);
    crate::decision::accept_routed(
        &mut domain.journal,
        &mut domain.engine.core_mut().counters,
        &env.limits.journal,
        decision,
        true,
    )
    .unwrap();
}

fn make_tier(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>) -> u64 {
    use super::{Work, forge, tasks};
    let repository = adopted(1).provider;
    forge_request(
        domain,
        env,
        forge::Request::TierRun { repository, tier: Box::from(b"fuzzy".as_slice()), number: 7, tip: SHA1 },
    );
    let Some(Work::Core(event)) = domain.work.pop() else { panic!("make tier task") };
    let jig_domain_core::Event::ConnectorRepair { executor, spec, contract, holdings, .. } = &event else {
        panic!("deployment seed")
    };
    assert_eq!(*executor, tasks::Executor::Procedure { connector: 0, code: super::AFTER_LANDING_CODE });
    assert!(spec.words.starts_with(b"Tier fuzzy run 7\n"));
    assert_eq!(
        spec.parameters.as_ref(),
        &[
            tasks::Parameter::Resource { name: 1, connector: 0, resource: forge::tag(1, 1) },
            tasks::Parameter::Bytes { name: 8, value: commit_text(&SHA1) },
            tasks::Parameter::Bytes { name: 10, value: Box::from(b"fuzzy".as_slice()) },
            tasks::Parameter::Number { name: 11, value: 7 },
        ]
    );
    assert_eq!(*contract, tasks::Contract::Report { words: env.limits.tasks.result_bytes });
    assert!(holdings.is_empty());
    let Some(Work::Forge(forge::Event::TierTask { task, repository: got, tier, number: 7 })) = domain.work.pop() else {
        panic!("answer tier task")
    };
    assert_eq!(got, repository);
    assert_eq!(tier.as_ref(), b"fuzzy");
    assert!(domain.work.is_empty());
    drop(jig_domain_core::step(domain.engine.core_mut(), &crate::limits::environment_core(env), event));
    let row = domain.engine.core().tasks.task(task).expect("tier task admitted");
    assert_eq!(row.numbers.budget, 0);
    assert_eq!(row.requester, tasks::Party::Deployment { project: 1 });
    task
}

#[test]
fn a_tier_task_starts_while_the_period_is_reserved_whole() {
    use super::tasks;
    let (mut domain, env) = procedure_root();
    let funder = tasks::Funder::Period { project: 1, period: domain.engine.core().period(1) };
    reserve_period(&mut domain, &env);
    let ledger = *domain.engine.core().tasks.funding(funder).expect("reserved period");
    assert!(domain.engine.core_mut().repair_seed(1).is_none(), "agent repair still needs unreserved spend");
    make_tier(&mut domain, &env);
    assert_eq!(domain.engine.core().tasks.funding(funder).unwrap().numbers.reserved, ledger.numbers.budget);
}

#[test]
fn a_tier_without_a_project_seed_is_refused() {
    use super::{Work, forge};
    let (mut domain, env) = two_forges(); // The repository names project 5; only project 1 has a policy.
    forge_request(
        &mut domain,
        &env,
        forge::Request::TierRun {
            repository: adopted(1).provider,
            tier: Box::from(b"fuzzy".as_slice()),
            number: 7,
            tip: SHA1,
        },
    );
    let Some(Work::Forge(forge::Event::TierRefused { repository, tier, number: 7 })) = domain.work.pop() else {
        panic!("refuse missing seed")
    };
    assert_eq!(repository, adopted(1).provider);
    assert_eq!(tier.as_ref(), b"fuzzy");
    assert!(domain.work.is_empty());
}

#[test]
fn a_held_tier_keeps_its_words_and_a_release_steps_it_again() {
    use super::{Work, forge, tasks};
    let (mut domain, env) = procedure_root();
    let task = make_tier(&mut domain, &env);
    offer_tier(&mut domain, &env, task);
    let Some(Work::Forge(forge::Event::StepTier { task: got, heard, .. })) = domain.work.pop() else {
        panic!("first tier step")
    };
    assert_eq!(got, task);
    assert!(!heard.released);
    let words = b"Tier fuzzy run 7 held after 3 runner faults, last: dispatch";
    forge_request(
        &mut domain,
        &env,
        forge::Request::TierDecision {
            projection: None,
            task,
            decision: forge::TierDecision::Held { words: Box::from(words.as_slice()) },
            entry: None,
        },
    );
    let Some(Work::Core(event)) = domain.work.pop() else { panic!("hold tier") };
    let jig_domain_core::Event::ProcedureStep { decision, .. } = &event else { panic!("held step") };
    assert_eq!(
        decision.end,
        super::StepEnd::HoldFrom {
            why: tasks::Hold::Procedure,
            source: Box::new(tasks::HoldSource::Account {
                connector: 0,
                entry: None,
                words: Box::from(words.as_slice())
            }),
        }
    );
    assert!(decision.messages.is_empty());
    drop(jig_domain_core::step(domain.engine.core_mut(), &crate::limits::environment_core(&env), event));
    let tasks::Phase::Held { why, source, .. } = &domain.engine.core().tasks.task(task).unwrap().phase else {
        panic!("held tier")
    };
    assert_eq!(*why, tasks::Hold::Procedure);
    assert_eq!(
        source.as_deref(),
        Some(&tasks::HoldSource::Account { connector: 0, entry: None, words: Box::from(words.as_slice()) })
    );
    release_held_tier(&mut domain, &env, task);
}

#[test]
fn a_done_tier_ends_with_the_verdicts_words() {
    use super::{Work, forge, tasks};
    let (mut domain, env) = procedure_root();
    let task = make_tier(&mut domain, &env);
    let words = b"Tier fuzzy run 7: green, cleared 2 landings";
    forge_request(
        &mut domain,
        &env,
        forge::Request::TierDecision {
            projection: None,
            task,
            decision: forge::TierDecision::Done { words: Box::from(words.as_slice()) },
            entry: None,
        },
    );
    let Some(Work::Core(event)) = domain.work.pop() else { panic!("finish tier") };
    let jig_domain_core::Event::ProcedureStep { decision, .. } = &event else { panic!("done step") };
    assert_eq!(decision.end, super::StepEnd::Result(tasks::TaskResult::Report { words: Box::from(words.as_slice()) }));
    drop(jig_domain_core::step(domain.engine.core_mut(), &crate::limits::environment_core(&env), event));
    let tasks::Phase::Closing(closing) = &domain.engine.core().tasks.task(task).unwrap().phase else {
        panic!("closing tier")
    };
    assert_eq!(closing.ending, tasks::Ending::Done(tasks::TaskResult::Report { words: Box::from(words.as_slice()) }));
}

#[test]
fn a_landing_merge_waiting_on_authority_is_deferred_not_vetoed() {
    use super::{Work, authority, forge};
    let (mut domain, env) = procedure_root();
    while domain.work.pop().is_some() {}
    super::drop_landing(&mut domain, &env, 91, 61, None, authority::Answer::Wait, true);
    let Some(Work::Forge(event)) = domain.work.pop() else { panic!("deferred merge") };
    assert_eq!(event, forge::Event::DeferLandingMerge { task: 61, entry: 91 });
    assert!(domain.work.is_empty(), "waiting does not hold the landing");
}

fn reserve_period(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>) -> u64 {
    use super::tasks;
    let seed = domain.engine.core_mut().repair_seed(1).expect("unreserved period can fund an agent");
    let task = seed.number();
    drop(jig_domain_core::step(
        domain.engine.core_mut(),
        &crate::limits::environment_core(env),
        jig_domain_core::Event::ConnectorRepair {
            seed: Box::new(seed),
            executor: tasks::Executor::Agent { charter: 1 },
            spec: tasks::Spec {
                job: None,
                asks: false,
                words: Box::from(b"reserve the period".as_slice()),
                parameters: Box::new([]),
                inputs: Box::new([]),
            },
            contract: tasks::Contract::Report { words: env.limits.tasks.result_bytes },
            holdings: Box::new([]),
        },
    ));
    assert_eq!(domain.engine.core().tasks.task(task).unwrap().numbers.budget, 1000);
    let funder = tasks::Funder::Period { project: 1, period: domain.engine.core().period(1) };
    let ledger = domain.engine.core().tasks.funding(funder).unwrap();
    assert_eq!(ledger.numbers.reserved, ledger.numbers.budget);
    task
}

fn reserved_turn_landing() -> (super::Domain, skein_lib::Env<super::Limits>, super::forge::LandingRow) {
    reserved_turn_landing_with_limits(&crate::tests::resume_fixture::limits())
}
fn reserved_turn_landing_with_limits(
    limits: &super::Limits,
) -> (super::Domain, skein_lib::Env<super::Limits>, super::forge::LandingRow) {
    let (mut domain, env) = procedure_root_with_limits(*limits);
    reserve_period(&mut domain, &env);
    let row = form_turn_landing(&mut domain, &env);
    (domain, env, row)
}

fn take_landing_admission(
    domain: &mut super::Domain,
    env: &skein_lib::Env<super::Limits>,
) -> (super::forge::LandingRow, Box<[super::forge::Name]>) {
    use super::Work;
    use jig_domain_contract::{Up, resources};
    use jig_domain_core::{Event, engine::In};

    let Some(Work::Connector(connector, up @ Up::Resources(resources::Up::Resource { .. }))) = domain.work.pop() else {
        panic!("branch resource")
    };
    assert_eq!(connector.0, domain.config.forge_connector);
    drop(crate::engine::step(&mut domain.engine, env, In::Connector(connector, up)));
    let Some(Work::Core(event @ Event::ConnectorRepair { .. })) = domain.work.pop() else { panic!("landing task") };
    drop(crate::engine::step(&mut domain.engine, env, In::Core(event)));
    let Some(Work::LandingAdmission { row, names }) = domain.work.pop() else { panic!("landing admission") };
    (row, names)
}

fn form_turn_landing(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>) -> super::forge::LandingRow {
    use super::{Work, forge, forge_change};
    let mut row = change_row(Some(7));
    row.repository = adopted(1).provider;
    row.change.way = forge_change::Way::Turn;
    row.change.state = forge_change::State::Ready { head: SHA1, ready_since: env.wall };
    restore_forge_with_engine_waits(domain, env, forge::Stored::Change(Box::new(row)));
    assert!(domain.forge.project_landing(
        1,
        forge_change::Way::Turn,
        Box::new([]),
        temper_engine_domain_forge_landing::Project::DEFAULT,
        forge::AfterLanding {
            wait: temper_engine_domain_forge_change::Wait::Cleared,
            verdict: temper_engine_domain_forge_tiers::Settings {
                retries: 2,
                suspects: temper_engine_domain_forge_tiers::Suspects::Bisect
            },
            bound: skein_lib::Duration::from_secs(7200),
        }
    ));
    super::start_landing(domain, env, 7);
    let (row, names) = take_landing_admission(domain, env);
    assert_eq!(row.landing.members[0].change, 7);
    for name in names {
        domain.work.push(Work::Forge(forge::Event::Hold {
            entry: 0,
            task: row.landing.task,
            resource: name,
            from: None,
        }));
    }
    domain.forge_forming = None;
    row
}

#[test]
fn a_landing_by_turn_starts_while_the_period_is_reserved_whole() {
    use super::tasks;
    let (domain, _, landing) = reserved_turn_landing();
    let row = domain.engine.core().tasks.task(landing.landing.task).expect("zero-budget landing admitted");
    assert_eq!(row.executor, tasks::Executor::Procedure { connector: 0, code: super::LANDING_CODE });
    assert_eq!(row.numbers.budget, 0);
    assert_eq!(row.spec.words.as_ref(), b"Land Change");
}

fn result_verdicts(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>) {
    use super::forge;
    let tiers = Box::new([
        forge::TierVerification {
            tier: b"fuzzy".as_slice().into(),
            verdict: Some(forge::TierVerdict { run: 2, blame: temper_engine_domain_forge_tiers::Blame::Cleared }),
            repaired: None,
        },
        forge::TierVerification {
            tier: b"browser".as_slice().into(),
            verdict: Some(forge::TierVerdict { run: 4, blame: temper_engine_domain_forge_tiers::Blame::Cleared }),
            repaired: None,
        },
    ]);
    restore_forge_with_engine_waits(
        domain,
        env,
        forge::Stored::Verification(forge::Verification {
            repository: change_row(Some(7)).repository,
            commit: SHA1,
            changes: Box::new([7]),
            tiers,
        }),
    );
}

#[test]
fn cleared_and_uncleared_results_qualify_the_landing_before_its_evidence() {
    use super::{forge, forge_change};
    for (state, line) in [
        (
            forge_change::State::Cleared { merge: SHA1 },
            b"Cleared by fuzzy run 2.\nCleared by browser run 4.".as_slice(),
        ),
        (
            forge_change::State::Uncleared { merge: SHA1 },
            b"Not cleared: no verdict within the verification bound.".as_slice(),
        ),
    ] {
        let (mut domain, env) = two_forges();
        let mut row = change_row(Some(7));
        row.change.state = state;
        row.change.last_head = Some(SHA1);
        result_verdicts(&mut domain, &env);
        restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Change(Box::new(row)));
        let words = super::change_landing_words(&domain, 7, &SHA1, 7, 512);
        let digits = commit_text(&SHA1);
        assert_eq!(&words[digits.len() + 1..digits.len() + 1 + line.len()], line);
        assert!(words[digits.len() + 1 + line.len()..].starts_with(b"\npull request #"));
        let bound = u32::try_from(digits.len() + 1 + line.len()).unwrap();
        let exact = super::change_landing_words(&domain, 7, &SHA1, 7, bound);
        assert!(exact.ends_with(line));
        let short = super::change_landing_words(&domain, 7, &SHA1, 7, bound - 1);
        for part in short.windows(line.len() - 1) {
            assert_ne!(part, &line[..line.len() - 1], "qualification is kept whole");
        }
    }
}

fn cleared_repair_result(by: Option<skein_forge::api::Commit>) -> (super::Domain, skein_lib::Env<super::Limits>) {
    use super::{forge, forge_change};
    let (mut domain, env) = two_forges();
    let mut row = change_row(Some(7));
    row.change.state = forge_change::State::Cleared { merge: SHA1 };
    let verification = forge::Verification {
        repository: row.repository,
        commit: SHA1,
        changes: Box::new([row.task]),
        tiers: Box::new([forge::TierVerification {
            tier: Box::from(b"browser".as_slice()),
            verdict: Some(forge::TierVerdict {
                run: 1,
                blame: temper_engine_domain_forge_tiers::Blame::Owns(temper_engine_domain_forge_tiers::Failures::Many),
            }),
            repaired: Some(forge::TierRepair { run: 2, tip: SHA256, by }),
        }]),
    };
    for record in [forge::Stored::Change(Box::new(row)), forge::Stored::Verification(verification)] {
        restore_forge_with_engine_waits(&mut domain, &env, record);
    }
    (domain, env)
}

#[test]
fn a_cleared_changes_result_names_its_repairing_landing() {
    let (domain, _) = cleared_repair_result(Some(SHA256));
    assert!(
        skein_lib::bytes::find(
            &super::change_landing_words(&domain, 7, &SHA1, 7, 512),
            b"\nCleared by browser run 2, which passed the failures blamed on it; repaired by abababababab.\n"
        )
        .is_some()
    );
}

#[test]
fn a_change_cleared_by_the_run_alone_keeps_its_words() {
    let (domain, _) = cleared_repair_result(None);
    assert_eq!(
        super::change_verification_words(&domain, 7, &SHA1, 512).as_ref(),
        b"Cleared by browser run 2, which passed the failures blamed on it."
    );
}

#[test]
fn repairing_tiers_keep_their_own_lines_among_other_tiers() {
    use super::forge;
    let (mut domain, env) = cleared_repair_result(Some(SHA256));
    let repository = change_row(Some(7)).repository;
    let repaired = domain.forge.landing_verification(repository, &SHA1).unwrap().tiers[0].clone();
    for (names, expected) in [
        (
            [b"browser".as_slice(), b"fuzzy", b"slow"],
            b"Cleared by browser run 2, which passed the failures blamed on it; repaired by abababababab.\nCleared by fuzzy run 4.\nCleared by slow run 4.".as_slice(),
        ),
        (
            [b"fuzzy".as_slice(), b"browser", b"slow"],
            b"Cleared by fuzzy run 4.\nCleared by browser run 2, which passed the failures blamed on it; repaired by abababababab.\nCleared by slow run 4.".as_slice(),
        ),
        (
            [b"fuzzy".as_slice(), b"slow", b"browser"],
            b"Cleared by fuzzy run 4.\nCleared by slow run 4.\nCleared by browser run 2, which passed the failures blamed on it; repaired by abababababab.".as_slice(),
        ),
    ] {
        let mut verification = domain.forge.landing_verification(repository, &SHA1).unwrap().clone();
        let mut tiers = skein_lib::List::uncapped();
        for name in names {
            let tier = if name == b"browser" {
                repaired.clone()
            } else {
                forge::TierVerification {
                    tier: name.into(),
                    verdict: Some(forge::TierVerdict { run: 4, blame: temper_engine_domain_forge_tiers::Blame::Cleared }),
                    repaired: None,
                }
            };
            tiers.push(tier).unwrap();
        }
        verification.tiers = tiers.into_boxed();
        restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Verification(verification));
        assert_eq!(super::change_verification_words(&domain, 7, &SHA1, 512).as_ref(), expected);
    }
}

#[test]
fn an_uncleared_cancelled_change_names_cancellation() {
    use super::{forge, forge_change, tasks};
    let (mut domain, env, task) =
        crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: super::CHANGE_CODE });
    let mut row = task_change_row(task, Some(7));
    row.change.state = forge_change::State::Uncleared { merge: SHA1 };
    domain.forge.restore_begin(env.wall);
    restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    drop(jig_domain_core::step(
        domain.engine.core_mut(),
        &crate::limits::environment_core(&env),
        jig_domain_core::Event::Tasks(tasks::Event::Cancel {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(123)),
            by: tasks::Party::Deployment { project: 1 },
            operation: tasks::Cancel { task, reason: Box::new([]), grace: None, message: None },
        }),
    ));
    assert_eq!(
        super::change_verification_words(&domain, task, &SHA1, 128).as_ref(),
        b"Not cleared: cancelled while verifying."
    );
}

fn offer_tier(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>, task: u64) {
    let context = domain.engine.core().tasks.run_context(task).unwrap();
    let (_, _, step) = domain.engine.core().tasks.procedure_due(task).unwrap();
    let mut decision = super::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::super::asks::ask(
        domain,
        env,
        &mut decision,
        domain.config.forge_connector,
        jig_domain_contract::Ask::Procedures(jig_domain_contract::procedures::Ask::Start { context, step, chat: None }),
    );
    crate::decision::accept_routed(
        &mut domain.journal,
        &mut domain.engine.core_mut().counters,
        &env.limits.journal,
        decision,
        true,
    )
    .unwrap();
}

#[test]
fn a_tier_without_a_task_number_is_refused() {
    use super::{Work, forge};
    let (mut domain, env) = procedure_root();
    let mut deployment = domain.engine.core().counters.deployment();
    deployment.tasks = u64::MAX;
    domain.engine.core_mut().counters = crate::Counters::new(deployment);
    forge_request(
        &mut domain,
        &env,
        forge::Request::TierRun {
            repository: adopted(1).provider,
            tier: Box::from(b"fuzzy".as_slice()),
            number: 7,
            tip: SHA1,
        },
    );
    let Some(Work::Forge(forge::Event::TierRefused { number: 7, .. })) = domain.work.pop() else {
        panic!("exhausted task number refuses tier")
    };
    assert!(domain.work.is_empty());
}

#[test]
fn a_waiting_tier_ends_its_step_and_is_stepped_on_a_forge_wake() {
    use super::{Work, forge, tasks};
    let (mut domain, env) = procedure_root();
    let task = make_tier(&mut domain, &env);
    forge_request(
        &mut domain,
        &env,
        forge::Request::TierDecision {
            projection: None,
            task,
            decision: forge::TierDecision::Wait { until: Some(skein_lib::Wall::from_nanos(100)) },
            entry: None,
        },
    );
    let Some(Work::Core(event)) = domain.work.pop() else { panic!("wait tier step") };
    let jig_domain_core::Event::ProcedureStep { decision, .. } = &event else { panic!("wait step") };
    assert_eq!(decision.end, super::StepEnd::Wait);
    drop(jig_domain_core::step(domain.engine.core_mut(), &crate::limits::environment_core(&env), event));
    assert!(domain.engine.core().tasks.procedure_due(task).is_none());
    forge_request(&mut domain, &env, forge::Request::WakeProcedure { task });
    let Some(Work::Tasks(event @ tasks::Event::WakeProcedure { .. })) = domain.work.pop() else { panic!("wake tier") };
    drop(jig_domain_core::step(
        domain.engine.core_mut(),
        &crate::limits::environment_core(&env),
        jig_domain_core::Event::Tasks(event),
    ));
    offer_tier(&mut domain, &env, task);
    let Some(Work::Forge(forge::Event::StepTier {
        task: got, heard: forge::TierHeard { released: false, .. }, ..
    })) = domain.work.pop()
    else {
        panic!("step woken tier")
    };
    assert_eq!(got, task);
    assert!(domain.work.is_empty());
}

#[test]
fn a_tier_effect_without_a_description_is_vetoed_before_it_can_run() {
    use super::{Work, forge, tasks};
    let (mut domain, env) = procedure_root();
    let task = make_tier(&mut domain, &env);
    forge_request(
        &mut domain,
        &env,
        forge::Request::TierDecision {
            projection: None,
            task,
            decision: forge::TierDecision::Wait { until: None },
            entry: Some(8),
        },
    );
    let Some(Work::Forge(forge::Event::VetoTier { task: got, entry: 8 })) = domain.work.pop() else {
        panic!("veto tier entry")
    };
    assert_eq!(got, task);
    let Some(Work::Tasks(tasks::Event::Hold { task: got, why: tasks::Hold::Effects })) = domain.work.pop() else {
        panic!("hold refused effect")
    };
    assert_eq!(got, task);
    assert!(domain.forge_effects.is_empty());
    assert!(domain.work.is_empty());
}

#[test]
fn a_change_made_with_parameter_14_keeps_its_route() {
    use super::{forge, forge_change, tasks};
    // forge, host, owner, repository, branch, main.
    const BASE_AUTHORITY_SEGMENTS: u32 = 6;
    for (parameter, requester, expected) in [
        (None, tasks::Party::Task(91), Some(forge_change::Route::None)),
        (Some(tasks::Parameter::Number { name: 14, value: 1 }), tasks::Party::Task(91), None),
        (Some(tasks::Parameter::Number { name: 14, value: 2 }), tasks::Party::Task(91), Some(forge_change::Route::Up)),
        (Some(tasks::Parameter::Number { name: 14, value: 3 }), tasks::Party::Task(91), None),
        (Some(tasks::Parameter::Number { name: 14, value: 0 }), tasks::Party::Task(91), None),
        (Some(tasks::Parameter::Bytes { name: 14, value: b"foreman".as_slice().into() }), tasks::Party::Task(91), None),
        (Some(tasks::Parameter::Resource { name: 14, connector: 0, resource: 1 }), tasks::Party::Task(91), None),
        (
            Some(tasks::Parameter::Number { name: 14, value: 2 }),
            tasks::Party::Person(1),
            Some(forge_change::Route::None),
        ),
        (Some(tasks::Parameter::Number { name: 14, value: 1 }), tasks::Party::Deployment { project: 1 }, None),
    ] {
        let (mut domain, mut env) = procedure_root();
        env.limits.authority.segments = BASE_AUTHORITY_SEGMENTS;
        let task = make_tier(&mut domain, &env);
        let mut context = domain.engine.core().tasks.run_context(task).unwrap();
        let mut parameters = skein_lib::List::uncapped();
        for parameter in [
            tasks::Parameter::Resource { name: 1, connector: 0, resource: forge::tag(1, 1) },
            tasks::Parameter::Bytes { name: 3, value: b"temper/change".as_slice().into() },
        ] {
            parameters.push(parameter).unwrap();
        }
        if let Some(parameter) = parameter {
            parameters.push(parameter).unwrap();
        }
        context.requester = requester;
        context.spec.parameters = parameters.into_boxed();
        assert_eq!(super::start_change(&mut domain, &env, &context), expected.is_some());
        let made = match domain.forge.change(task) {
            Some(row) => Some(row.questions.clone()),
            None => None,
        };
        let expected = match expected {
            Some(route) => Some(forge_change::Questions { route, ..forge_change::Questions::default() }),
            None => None,
        };
        assert_eq!(made, expected);
    }
}

#[test]
fn branch_moved_and_stalled_holds_keep_the_heads_and_wait_in_their_accounts() {
    use super::{forge_change as change, tasks};
    let expected = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0,
    ];
    let found = [
        0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0,
    ];
    for (was, why, words) in [
        (
            change::State::Opening { requested: true },
            change::Cause::Stalled,
            b"no progress at opening for 30 minutes: waiting for its pull request to open".as_slice(),
        ),
        (
            change::State::Updating { from: expected, base: found, ready_since: skein_lib::Wall::EPOCH },
            change::Cause::BranchMoved,
            b"its branch moved: expected 0123456789ab, found 89abcdef0123".as_slice(),
        ),
        (
            change::State::Checking { head: expected },
            change::Cause::BranchMoved,
            b"its branch moved: expected 0123456789ab, found 89abcdef0123".as_slice(),
        ),
        (
            change::State::Updating { from: expected, base: found, ready_since: skein_lib::Wall::EPOCH },
            change::Cause::Stalled,
            b"no progress at updating for 30 minutes: waiting for its update of 0123456789ab with main at 89abcdef0123"
                .as_slice(),
        ),
        (
            change::State::Checking { head: expected },
            change::Cause::Stalled,
            b"no progress at checking for 30 minutes: waiting for CI at 0123456789ab".as_slice(),
        ),
        (
            change::State::First { head: expected, ready_since: skein_lib::Wall::EPOCH },
            change::Cause::Stalled,
            b"no progress at first for 30 minutes: waiting for the forge to settle whether 0123456789ab merges"
                .as_slice(),
        ),
        (
            change::State::Landing { head: expected },
            change::Cause::Stalled,
            b"no progress at landing for 30 minutes: waiting for its merge of 0123456789ab".as_slice(),
        ),
    ] {
        let mut row = change_row(Some(1));
        row.change.last_head = Some(expected);
        row.ci = Some(change::Ci { head: found, status: change::Status::Pending });
        row.change.state =
            change::State::Held { was: Box::new(was), hold: change::Hold::new(why, skein_lib::Wall::EPOCH) };
        let (domain, mut env, _) = restored_change([
            super::forge::Stored::Change(Box::new(row)),
            super::forge::Stored::BranchHead(super::forge::BranchHead {
                name: change_names(None)[0].clone(),
                commit: found,
                owned: true,
            }),
        ]);
        env.limits.forge.change_policy.stall = skein_lib::Duration::from_secs(1800);
        let decision = super::change_hold_step(&domain, &env, 7, why, None);
        let category = if why == change::Cause::BranchMoved { tasks::Hold::Drift } else { tasks::Hold::Stalled };
        assert_eq!(
            decision.end,
            super::StepEnd::HoldFrom {
                why: category,
                source: Box::new(tasks::HoldSource::Account { connector: 0, entry: None, words: words.into() }),
            }
        );
    }
}

#[test]
fn a_record_judged_hold_names_the_deciding_read_and_its_nearest_write() {
    use super::{forge, forge_change as change, tasks};
    let expected = [0x12; 32];
    let found = [0x89; 32];
    let saved = [0x34; 32];
    for (behind, why, category, words) in [
        (
            None,
            change::Cause::BranchMoved,
            tasks::Hold::Drift,
            b"its branch moved: expected 121212121212, found 898989898989; temper's last write on it: push of 121212121212 by task 412 attempt 2, confirmed".as_slice(),
        ),
        (
            Some(forge::ReadBehind { read: forge::BehindRead::Branch, expected: Some(expected), found: Some(found) }),
            change::Cause::Stalled,
            tasks::Hold::Stalled,
            b"its branch reads 898989898989, an older head of temper's (expected 121212121212): the read may be stale or another's write".as_slice(),
        ),
    ] {
        let mut row = change_row(Some(1));
        row.change.last_head = Some(saved);
        row.change.state = change::State::Checking { head: expected };
        let prior = row.change.clone();
        row.change.state = change::State::Held { was: Box::new(row.change.state), hold: change::Hold::new(why, skein_lib::Wall::EPOCH) };
        let name = change_names(None)[0].clone();
        let (domain, env, _) = restored_change([
            forge::Stored::Change(Box::new(row)),
            forge::Stored::BranchHead(forge::BranchHead { name: name.clone(), commit: saved, owned: true }),
            forge::Stored::Writes(forge::Writes {
                name,
                head: Some(expected),
                next: 4,
                entries: Box::new([forge::Written {
                    order: 3,
                    operation: forge::writes::Operation::Push { before: Some(found), after: expected },
                    sender: forge::Sender::Attempt { task: 412, attempt: 2 },
                    state: forge::WriteState::Confirmed,
                }]),
            }),
        ]);
        let evidence = forge::ChangeEvidence {
            behind,
            nearest_write: Some(3),
            ..unavailable_evidence(prior, found)
        };
        let decision = super::change_hold_step(&domain, &env, 7, why, Some(&evidence));
        assert_eq!(
            decision.end,
            super::StepEnd::HoldFrom {
                why: category,
                source: Box::new(tasks::HoldSource::Account { connector: 0, entry: None, words: words.into() }),
            }
        );
    }
}

#[test]
fn a_restored_hold_keeps_its_deciding_read_account_and_refreshes_its_release_revision() {
    use super::{forge, forge_change as change, tasks};
    const EXPECTED: [u8; 32] = [0x12; 32];
    const FOUND: [u8; 32] = [0x89; 32];
    const ACCOUNT: &[u8] = b"its branch moved: expected 121212121212, found 898989898989";
    let executor = tasks::Executor::Procedure { connector: 0, code: super::CHANGE_CODE };
    let (mut domain, mut env, task) = crate::tests::live_task(None, executor);
    env.wall = skein_lib::Wall::from_nanos(1_000);
    let mut row = task_change_row(task, None);
    row.change.state = change::State::Checking { head: EXPECTED };
    let prior = row.change.clone();
    row.change.state = change::State::Held {
        was: Box::new(row.change.state),
        hold: change::Hold::new(change::Cause::BranchMoved, env.wall),
    };
    let (_, hot) = row.clone().split();
    domain.forge.restore_begin(env.wall);
    restore_forge(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    let mut out = skein_lib::Queue::uncapped();
    out.push(forge::Request::Save {
        record: forge::Stored::ChangeHot(Box::new(hot)),
        release: None,
        read_afresh: None,
    });
    out.push(forge::Request::ChangeDecision {
        task,
        decision: change::Decision::Hold(change::Cause::BranchMoved),
        entry: None,
        evidence: Some(unavailable_evidence(prior, FOUND)),
        close_at: None,
        close_refusal: None,
    });
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::outputs(&mut domain, &env, &mut decision, &mut out);
    apply_change_task_work(&mut domain, &env);
    let (first, first_revision) = held_change_words(&domain, task);
    let first: Box<[u8]> = Box::from(first);
    assert!(first.starts_with(ACCOUNT));
    let saved = domain.forge.change(task).expect("saved hold").clone();
    let change::State::Held { hold, .. } = &saved.change.state else { panic!("one held record") };
    assert_eq!(hold.since, env.wall);
    assert_eq!(hold.words.as_ref(), ACCOUNT);

    // Restore the committed hot part without the read. The task is live at a later revision.
    crate::decision::accept_routed(
        &mut domain.journal,
        &mut domain.engine.core_mut().counters,
        &env.limits.journal,
        decision,
        false,
    )
    .unwrap();
    let mut committed_hot = None;
    while let Some(commit) = domain.journal.commit() {
        for write in &commit.writes {
            if let crate::Write::Save(crate::Record::Forge { row }) = write
                && let forge::Stored::ChangeHot(hot) = row.as_ref()
                && hot.task == task
            {
                committed_hot = Some(hot.clone());
            }
        }
    }
    let (cold, _) = saved.split();
    let mut live = domain.engine.core().tasks.task(task).unwrap().clone();
    let ledger = *domain.engine.core().tasks.funding(live.funder).unwrap();
    live.phase = tasks::Phase::Active(tasks::Active::Due);
    live.held_since = None;
    live.due_reason = tasks::Why::Ready;
    let fixture = crate::tests::resume_fixture::config(1);
    let (mut restarted, env) = crate::tests::resume_fixture::root_with_records(
        fixture,
        &env.limits,
        &[
            crate::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ledger(ledger))),
            crate::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(Box::new(live)))),
        ],
    );
    restarted.forge.restore_begin(env.wall);
    restore_forge(
        &mut restarted,
        &env,
        forge::Stored::Change(Box::new(forge::ChangeRow::join(cold, *committed_hot.expect("committed hold")))),
    );
    super::change_decision(&mut restarted, &env, task, change::Decision::None);
    apply_change_task_work(&mut restarted, &env);
    let (again, revision) = held_change_words(&restarted, task);
    assert!(revision > first_revision);
    for (words, revision) in [(first.as_ref(), first_revision), (again, revision)] {
        let task = skein_lib::Decimal::of(task);
        let revision = skein_lib::Decimal::of(revision);
        let tail = skein_lib::bytes::concat(&[
            b"; resolve this reason, then task release ",
            task.as_bytes(),
            b" --hold ",
            revision.as_bytes(),
            b"; or task cancel ",
            task.as_bytes(),
            b" REASON",
        ]);
        assert_eq!(words.strip_suffix(tail.as_ref()), Some(ACCOUNT));
    }
}

fn held_change_words(domain: &super::Domain, task: u64) -> (&[u8], u64) {
    let row = domain.engine.core().tasks.task(task).expect("change task");
    let super::tasks::Phase::Held { source: Some(source), .. } = &row.phase else {
        panic!("change task held: {:?}", row.phase)
    };
    let super::tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("held with an account") };
    (words, row.revisions.hold)
}

#[test]
fn checking_holds_name_the_bound_the_execution_clock_reached() {
    use super::{forge_change as change, tasks};
    use skein_lib::{Duration, Wall};
    let head = [
        0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0,
    ];
    for (seen, waited, words) in [
        (
            change::Execution::Waiting,
            14_400,
            b"no progress at checking for 240 minutes: waiting for a runner for CI at 0123456789ab".as_slice(),
        ),
        (
            change::Execution::Waiting,
            14_399,
            b"no progress at checking for 30 minutes: waiting for CI at 0123456789ab".as_slice(),
        ),
        (
            change::Execution::Running,
            0,
            b"no progress at checking for 30 minutes: waiting for CI at 0123456789ab".as_slice(),
        ),
        (
            change::Execution::Unknown,
            0,
            b"no progress at checking for 30 minutes: waiting for CI at 0123456789ab".as_slice(),
        ),
    ] {
        let mut row = change_row(Some(1));
        row.change.state = change::State::Held {
            was: Box::new(change::State::Checking { head }),
            hold: change::Hold::new(change::Cause::Stalled, Wall::EPOCH),
        };
        row.change.clock = Some(change::Clock {
            since: Wall::EPOCH,
            head,
            run: 7,
            attempt: 2,
            ran: Duration::from_secs(1800),
            waited: Duration::from_secs(waited),
            unknown: Duration::ZERO,
            at: Wall::from_nanos(1_000_000_000),
            seen,
        });
        let (domain, mut env, _) = restored_change([super::forge::Stored::Change(Box::new(row))]);
        env.wall = Wall::from_nanos(1_000_000_000);
        env.limits.forge.change_policy.stall = Duration::from_secs(1800);
        env.limits.forge.change_policy.runner_wait = Duration::from_secs(14_400);
        let decision = super::hold_change_decision(&domain, &env, 7, change::Cause::Stalled, None);
        assert_eq!(
            decision.end,
            super::StepEnd::HoldFrom {
                why: tasks::Hold::Stalled,
                source: Box::new(tasks::HoldSource::Account { connector: 0, entry: None, words: words.into() }),
            }
        );
    }
}

#[test]
fn a_hold_past_questions_names_its_bound() {
    use super::{forge_change, tasks};
    for (open, words) in [
        (&[][..], b"its questions are spent: 4 of 4 taken".as_slice()),
        (
            &[forge_change::Open {
                asker: 9,
                number: 12,
                line: b"Which behavior?".as_slice().into(),
                words: Box::new([]),
                stands: forge_change::Stands::Taken,
            }][..],
            b"its questions are spent: 4 of 4 taken; question 12 of task 9 waits; release it to pass the question up"
                .as_slice(),
        ),
    ] {
        let mut row = change_row(None);
        row.questions.taken = 4;
        row.questions.open = open.into();
        let (domain, env, _) = restored_change([super::forge::Stored::Change(Box::new(row))]);
        let decision = super::hold_change_decision(&domain, &env, 7, forge_change::Cause::Questions, None);
        assert_eq!(
            decision.end,
            super::StepEnd::HoldFrom {
                why: tasks::Hold::Procedure,
                source: Box::new(tasks::HoldSource::Account { connector: 0, entry: None, words: words.into() }),
            }
        );
    }
}

fn release_held_tier(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>, task: u64) {
    use super::{Work, forge, tasks};
    let release = jig_domain_core::step(
        domain.engine.core_mut(),
        &crate::limits::environment_core(env),
        jig_domain_core::Event::Tasks(tasks::Event::Release {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(123)),
            by: tasks::Party::Person(1),
            operation: tasks::Release { task, again: None, message: None },
        }),
    );
    assert_eq!(domain.engine.core().tasks.task(task).unwrap().due_reason, tasks::Why::Released, "{release:?}");
    offer_tier(domain, env, task);
    let Some(Work::Forge(forge::Event::StepTier { task: got, heard, .. })) = domain.work.pop() else {
        panic!("released tier step")
    };
    assert_eq!(got, task);
    assert!(heard.released);
}

fn repairs_call(control: bool) -> (super::Domain, skein_lib::Env<super::Limits>, super::tasks::DelegationContext) {
    let (mut domain, env) = two_forges();
    let mut other = adopted(3);
    other.project = 6;
    restore_forge_with_engine_waits(&mut domain, &env, super::forge::Stored::Repository(other));
    let (source, _, task) = crate::tests::live_task(None, super::tasks::Executor::Procedure { connector: 0, code: 1 });
    let mut context = source.engine.core().tasks.delegation(task).expect("fixture delegation context");
    context.project = 5;
    context.numbers.budget = 100;
    context.tasks_left = 8;
    let family = jig_domain_core::ToolFamilies::JIG.control.0;
    if control {
        context.authority.tools.0 |= family;
    } else {
        context.authority.tools.0 &= !family;
    }
    (domain, env, context)
}

#[test]
fn repairs_without_control_is_refused_as_a_plan_member() {
    let (domain, env, context) = repairs_call(false);
    let change = crate::ChangeCall { repairs: Some(SHA1), ..crate::ChangeCall::words(b"Fix".as_slice().into(), None) };
    let Err(super::ChangeRefused::Words(words)) = super::change_batch(&domain, &env, 7, &context, &[change]) else {
        panic!("repairs refused")
    };
    assert_eq!(words.as_ref(), b"change's landing, branch, name, after, questions, repairs, changes, producer and llm are a plan's: this task's authority lacks the control family (amend, cancel, release).");
}

#[test]
fn repairs_naming_no_live_landing_is_refused() {
    let (domain, env, context) = repairs_call(true);
    let change = crate::ChangeCall { repairs: Some(SHA1), ..crate::ChangeCall::words(b"Fix".as_slice().into(), None) };
    let Err(super::ChangeRefused::Words(words)) = super::change_batch(&domain, &env, 7, &context, &[change]) else {
        panic!("unknown landing refused")
    };
    assert_eq!(
        words.as_ref(),
        b"repairs: abcdef0123456789abcdef0123456789abcdef01 is no live landing of temper's in org/repo."
    );
}

fn start_repairs(digits: Option<&[u8]>) -> (bool, skein_lib::Queue<super::Work>, Option<super::forge::ChangeRow>) {
    use super::tasks;
    let (mut domain, mut env) = two_forges();
    // forge, host, owner, repository, branch marker, main.
    env.limits.authority.segments = 6;
    let (source, _, task) = crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: 2 });
    let mut context = source.engine.core().tasks.run_context(task).expect("fixture run context");
    context.project = 5;
    let mut parameters = skein_lib::List::with_capacity(3);
    for parameter in [
        tasks::Parameter::Resource { name: 1, connector: 0, resource: super::forge::tag(1, 1) },
        tasks::Parameter::Bytes { name: 3, value: b"temper/repair".as_slice().into() },
    ] {
        parameters.push(parameter).expect("repository and branch");
    }
    if let Some(digits) = digits {
        parameters.push(tasks::Parameter::Bytes { name: 12, value: digits.into() }).expect("repairs");
    }
    context.spec.parameters = parameters.into_boxed();
    let accepted = super::start_change(&mut domain, &env, &context);
    let row = domain.forge.change(task).cloned();
    (accepted, domain.work, row)
}

#[test]
fn a_started_change_keeps_repairs_at_each_commit_width() {
    for (commit, digits) in COMMITS {
        let (accepted, _, row) = start_repairs(Some(digits));
        assert!(accepted, "commit {digits:?}");
        let row = row.unwrap();
        assert_eq!(row.repairs, Some(*commit));
    }
}

#[test]
fn a_started_change_without_repairs_keeps_none() {
    let (accepted, _, row) = start_repairs(None);
    assert!(accepted);
    let row = row.unwrap();
    assert_eq!(row.repairs, None);
}

#[test]
fn malformed_repairs_refuses_a_change_start() {
    for digits in [[b'A'; 40].as_slice(), [b'a'; 39].as_slice(), [b'g'; 64].as_slice()] {
        let (accepted, work, row) = start_repairs(Some(digits));
        assert!(row.is_none());
        assert!(!accepted, "commit {digits:?}");
        assert!(work.is_empty(), "no change or step queued for malformed repairs");
    }
}

/// temper/docs/design/domain/worker/jobs.md: the root writes R1's repository-relative command.
#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "unit fixtures format expected protocol bytes and resolution text"
)]
fn a_merge_jobs_spec_is_its_command_and_its_two_commits() {
    use temper_protocol_worker_wire::v1::{self, Record};
    for (head, head_text) in COMMITS {
        for (base, base_text) in COMMITS {
            let bytes =
                super::forge::jobs::merge(b"fabrig", Some(b".temper/merge"), head, base, 4096).expect("merge spec");
            let decoded = v1::JobSpec::decode(&v1::CEILINGS, &mut skein_lib::Reader::new(&bytes)).unwrap();
            assert_eq!(decoded.command.as_ref(), b"fabrig/.temper/hooks/merge");
            assert!(decoded.find && !decoded.refusal);
            assert_eq!(decoded.fallback.as_ref(), b"fabrig/.temper/merge");
            let expected = alloc::format!(
                "{{\"version\":1,\"head\":\"{}\",\"base\":\"{}\"}}",
                core::str::from_utf8(head_text).unwrap(),
                core::str::from_utf8(base_text).unwrap(),
            );
            assert_eq!(decoded.request.as_ref(), expected.as_bytes());
            let bound = u32::try_from(bytes.len()).unwrap();
            assert_eq!(super::forge::jobs::merge(b"fabrig", Some(b".temper/merge"), head, base, bound), Some(bytes));
            assert_eq!(super::forge::jobs::merge(b"fabrig", Some(b".temper/merge"), head, base, bound - 1), None);
        }
    }
}

fn word_parts(parts: &[&[u8]]) -> Box<[u8]> {
    let mut words = super::List::uncapped();
    for part in parts {
        super::append(&mut words, part);
    }
    words.into_boxed()
}

const LIVE_DELEGATE: u64 = 71;

fn live_delegate_amendment(
    kind: Option<super::forge_change::Delegate>,
    phase: super::tasks::Phase,
    words_changed: bool,
    reasons: &[&[u8]],
) -> (super::Domain, skein_lib::Env<super::Limits>, u64) {
    use super::{forge, forge_change as change, tasks};
    let (mut domain, mut env, task) =
        crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: super::CHANGE_CODE });
    env.limits.tasks.parameters = 5;
    env.limits.tasks.spec_bytes = 4096;
    env.limits.tasks.contract_choices = 2;
    let mut row = task_change_row(task, Some(7));
    row.change.last_head = Some(SHA1);
    row.merge_result = Some(b"made merge result: left file".as_slice().into());
    row.change.gates = Box::new([
        change::Gate {
            number: 1,
            kind: change::GateKind::Agent,
            blocking: true,
            freshness: change::Freshness::Clean,
            eager: true,
        },
        change::Gate {
            number: 2,
            kind: change::GateKind::Person,
            blocking: true,
            freshness: change::Freshness::Clean,
            eager: true,
        },
        change::Gate {
            number: 3,
            kind: change::GateKind::Check,
            blocking: true,
            freshness: change::Freshness::Clean,
            eager: true,
        },
    ]);
    row.delegate = match kind {
        Some(kind) => Some((LIVE_DELEGATE, kind)),
        None => None,
    };
    domain.forge.restore_begin(env.wall);
    restore_forge(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    let delegate =
        super::change_delegate(&domain, &env, task, kind.unwrap_or(change::Delegate::Repair(change::Repair::Ci)), None)
            .unwrap();
    let mut change = domain.engine.core().tasks.task(task).unwrap().clone();
    let mut child = change.clone();
    child.number = LIVE_DELEGATE;
    child.requester = tasks::Party::Task(task);
    child.executor = delegate.executor;
    child.spec = delegate.spec;
    child.contract = delegate.contract;
    child.phase = phase;
    child.revisions.spec = 4;
    change.phase = tasks::Phase::Active(tasks::Active::Idle);
    if words_changed {
        change.spec.words = b"Drop the originally requested file".as_slice().into();
    } else {
        change.dependencies = Box::new([72]);
        change.waiting_on = Box::new([72]);
    }
    change.revisions.spec = skein_lib::len_u64(reasons.len());
    let mut inbox = super::List::uncapped();
    for (index, reason) in reasons.iter().enumerate() {
        let revision = skein_lib::len_u64(index).checked_add(1).unwrap();
        inbox
            .push(tasks::Word {
                number: revision,
                from: change.requester,
                kind: tasks::MessageKind::Amendment { revision },
                words: (*reason).into(),
                at: env.wall,
                hits: 1,
                eligible: true,
            })
            .unwrap();
    }
    change.inbox = inbox.into_boxed();
    reset_tasks_for_restore(&mut domain, &env, 7);
    for record in [change, child] {
        task_event(&mut domain, &env, tasks::Event::Restore { record: tasks::Stored::Live(Box::new(record)) });
    }
    task_event(&mut domain, &env, tasks::Event::WakeProcedure { task });
    assert_eq!(domain.engine.core().tasks.restore_failure(), None);
    assert!(domain.engine.core().tasks.task(LIVE_DELEGATE).is_some());
    assert!(domain.engine.core().tasks.procedure_offered(task).unwrap() > 0);
    (domain, env, task)
}

fn live_amendment_step(
    domain: &mut super::Domain,
    env: &skein_lib::Env<super::Limits>,
    task: u64,
) -> super::ProcedureStep {
    super::relay::apply(domain, env, task, super::ProcedureStep::new(super::StepEnd::Wait))
}

#[test]
fn a_change_with_no_message_room_still_sends_the_questions_held_notice() {
    use super::{forge, forge_change as change, tasks};
    const REQUESTER: u64 = 91;
    const QUESTION: u64 = 25;
    let (mut domain, mut env, task) =
        crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: super::CHANGE_CODE });
    env.limits.tasks.batch = 1;
    env.wall = skein_lib::Wall::from_nanos(2_000_000_000);
    let mut row = change_row(Some(7));
    row.task = task;
    row.change.task = task;
    row.questions.route = change::Route::Up;
    row.questions.open = Box::new([change::Open {
        asker: LIVE_DELEGATE,
        number: 17,
        line: b"Which path?".as_slice().into(),
        words: b"Which path?".as_slice().into(),
        stands: change::Stands::Asked,
    }]);
    domain.forge.restore_begin(env.wall);
    restore_forge(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    let mut record = domain.engine.core().tasks.task(task).unwrap().clone();
    let mut asker = record.clone();
    asker.number = LIVE_DELEGATE;
    asker.requester = tasks::Party::Task(task);
    record.requester = tasks::Party::Task(REQUESTER);
    record.waiting_on = Box::new([]);
    record.phase = tasks::Phase::Active(tasks::Active::Idle);
    record.inbox = Box::new([tasks::Word {
        number: 26,
        from: record.requester,
        kind: tasks::MessageKind::Answer { question: QUESTION },
        words: b"Use the path.".as_slice().into(),
        at: env.wall,
        hits: 1,
        eligible: true,
    }]);
    reset_tasks_for_restore(&mut domain, &env, 7);
    for record in [record, asker] {
        task_event(&mut domain, &env, tasks::Event::Restore { record: tasks::Stored::Live(Box::new(record)) });
    }
    task_event(&mut domain, &env, tasks::Event::WakeProcedure { task });
    assert_eq!(domain.engine.core().tasks.restore_failure(), None);
    let mut decision = super::ProcedureStep::new(super::StepEnd::Wait);
    decision.messages = Box::new([jig_domain_core::StepMessage {
        notice: None,
        target: LIVE_DELEGATE,
        kind: tasks::MessageKind::Words,
        words: b"Already in the step".as_slice().into(),
    }]);
    let decision = super::relay::apply(&mut domain, &env, task, decision);
    let [ordinary, notice] = decision.messages.as_ref() else { panic!("one ordinary message and one notice") };
    assert_eq!(ordinary.notice, None);
    assert_eq!(ordinary.words.as_ref(), b"Already in the step");
    assert_eq!(notice.target, REQUESTER);
    assert_eq!(notice.notice, Some(QUESTION));
    assert_eq!(notice.kind, tasks::MessageKind::Words);
    assert_eq!(
        notice.words.as_ref(),
        b"Your answer to question 25 is held until this change has message room; held since 1970-01-01T00:00:02Z."
    );
    let step = domain.engine.core().tasks.procedure_due(task).unwrap().2;
    let saved = domain.forge_delegating.get(&(task, step)).unwrap().relayed.as_ref().unwrap();
    assert!(saved.more);
    assert_eq!(
        saved.questions.open[0].stands,
        change::Stands::Held {
            answer: b"Use the path.".as_slice().into(),
            question: QUESTION,
            blocked: Some(change::Blocked { dependency: None, since: env.wall }),
        }
    );
}

#[test]
fn a_live_resolution_keeps_its_words_and_takes_one_amendment_with_the_last_reason() {
    use super::{forge_change as change, tasks};
    for phase in [
        tasks::Phase::Waiting,
        tasks::Phase::Active(tasks::Active::Running { attempt: 1 }),
        tasks::Phase::Held { was: tasks::Was::Active(tasks::Active::Idle), why: tasks::Hold::Stopped, source: None },
    ] {
        let (mut domain, env, task) = live_delegate_amendment(
            Some(change::Delegate::Resolve { base: SHA1 }),
            phase,
            true,
            &[b"Keep both lines", b"Drop the file"],
        );
        let original = domain.engine.core().tasks.task(LIVE_DELEGATE).unwrap().spec.clone();
        let decision = live_amendment_step(&mut domain, &env, task);
        let [amendment] = decision.amendments.as_ref() else { panic!("one amendment: {:?}", decision.amendments) };
        assert_eq!(amendment.task, LIVE_DELEGATE);
        assert_eq!(amendment.revision, 4);
        let mut expected = original.clone();
        let change = domain.engine.core().tasks.task(task).unwrap();
        let questions = domain
            .forge_delegating
            .get(&(task, domain.engine.core().tasks.procedure_due(task).unwrap().2))
            .unwrap()
            .relayed
            .as_ref()
            .unwrap()
            .questions
            .clone();
        let block = super::change_context(task, 2, &change.spec.words, &questions, false, 4096).unwrap();
        let kept = super::before_context(&original.words);
        assert!(kept.starts_with(b"Resolve the merge conflict."));
        assert!(kept.ends_with(b"made merge result: left file"));
        expected.words = word_parts(&[kept, &block]);
        assert_eq!(amendment.amendment.spec.as_ref(), Some(&expected));
        assert_eq!(
            amendment.amendment.reason,
            super::delegate_amendment_words(task, 2, b"Drop the file", env.limits.tasks.message_bytes)
        );
        assert_eq!(
            domain.engine.core().tasks.task(LIVE_DELEGATE).unwrap().spec,
            original,
            "admission has not happened yet"
        );
    }
}

#[test]
fn dependencies_alone_still_amend_a_live_repairs_context() {
    use super::{forge_change as change, tasks};
    let (mut domain, env, task) = live_delegate_amendment(
        Some(change::Delegate::Repair(change::Repair::Ci)),
        tasks::Phase::Waiting,
        false,
        &[b"Wait for dependency"],
    );
    let original = domain.engine.core().tasks.task(LIVE_DELEGATE).unwrap().spec.clone();
    let decision = live_amendment_step(&mut domain, &env, task);
    let [amendment] = decision.amendments.as_ref() else { panic!("one amendment: {:?}", decision.amendments) };
    let spec = amendment.amendment.spec.as_ref().unwrap();
    assert_eq!(super::before_context(&spec.words), super::before_context(&original.words));
    assert_eq!(domain.engine.core().tasks.task(task).unwrap().spec.words.as_ref(), b"live goal");
    assert!(skein_lib::bytes::find(&spec.words, b"at spec revision 1.").is_some());
    assert!(skein_lib::bytes::find(&spec.words, b"Amended to spec revision 1: Wait for dependency").is_some());
    assert!(spec.words.ends_with(b"Its current spec, at revision 1:\nlive goal"));
}

#[test]
fn an_amendment_leaves_gates_and_absent_delegates_alone() {
    use super::{forge_change as change, tasks};
    for kind in [
        Some(change::Delegate::Gate { number: 1, head: SHA1 }),
        Some(change::Delegate::Gate { number: 2, head: SHA1 }),
        Some(change::Delegate::Gate { number: 3, head: SHA1 }),
        None,
    ] {
        let (mut domain, env, task) =
            live_delegate_amendment(kind, tasks::Phase::Active(tasks::Active::Idle), true, &[b"Drop the file"]);
        assert!(live_amendment_step(&mut domain, &env, task).amendments.is_empty(), "{kind:?}");
    }
}

#[test]
fn a_live_delegates_context_without_fixed_line_room_is_left_unchanged() {
    use super::{forge_change as change, tasks};
    let (mut domain, mut env, task) = live_delegate_amendment(
        Some(change::Delegate::Repair(change::Repair::Ci)),
        tasks::Phase::Waiting,
        true,
        &[b"Drop the file"],
    );
    let original = domain.engine.core().tasks.task(LIVE_DELEGATE).unwrap().spec.clone();
    let record = domain.engine.core().tasks.task(task).unwrap();
    let questions = change::Questions {
        amended: Box::new([change::Amended { revision: 1, reason: b"Drop the file".as_slice().into() }]),
        ..change::Questions::default()
    };
    let fixed = super::context_fixed(task, 1, &record.spec.words, &questions, false);
    let mut pinned = 0_usize;
    for parameter in &original.parameters {
        if let tasks::Parameter::Bytes { value, .. } = parameter {
            pinned = pinned.checked_add(value.len()).unwrap();
        }
    }
    assert!(pinned > 0);
    env.limits.tasks.spec_bytes =
        u32::try_from(pinned + super::before_context(&original.words).len() + fixed - 1).unwrap();
    assert!(live_amendment_step(&mut domain, &env, task).amendments.is_empty());
    assert_eq!(domain.engine.core().tasks.task(LIVE_DELEGATE).unwrap().spec, original);
    env.limits.tasks.spec_bytes += 1;
    assert_eq!(live_amendment_step(&mut domain, &env, task).amendments.len(), 1, "the fixed lines fit exactly");
}

#[test]
fn change_delegates_take_their_roles_saved_llm_choice() {
    use super::{forge, forge_change, tasks};
    use forge_change::{Delegate, Freshness, Gate, GateKind, Repair};
    const AGENT_GATE: u64 = 1;
    const CHECK_GATE: u64 = 2;
    const PERSON_GATE: u64 = 3;
    // Repository, base, branch, gate number and head are the gate's five parameters.
    const GATE_PARAMETERS: u32 = 5;
    // Room for review identity, repair opening and required change context.
    const SPEC_BYTES: u32 = 4096;
    let (mut domain, mut env, task) =
        crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: super::CHANGE_CODE });
    env.limits.tasks.parameters = GATE_PARAMETERS;
    env.limits.tasks.spec_bytes = SPEC_BYTES;
    let producing = choice_at_speed(3, Some(1), b"ultrafast");
    let review = choice_at_speed(2, Some(0), b"standard");
    let mut row = task_change_row(task, Some(7));
    row.change.last_head = Some(SHA1);
    row.producing_llm = Some(producing.clone());
    row.review_llm = Some(review.clone());
    row.change.gates = Box::new([
        Gate { number: AGENT_GATE, kind: GateKind::Agent, blocking: true, freshness: Freshness::Clean, eager: true },
        Gate { number: CHECK_GATE, kind: GateKind::Check, blocking: true, freshness: Freshness::Clean, eager: true },
        Gate { number: PERSON_GATE, kind: GateKind::Person, blocking: true, freshness: Freshness::Clean, eager: true },
    ]);
    domain.forge.restore_begin(env.wall);
    restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    for (kind, expected) in [
        (Delegate::Produce, Some(producing.clone())),
        (Delegate::Repair(Repair::Ci), Some(producing.clone())),
        (Delegate::Resolve { base: SHA1 }, Some(producing.clone())),
        (Delegate::Gate { number: AGENT_GATE, head: SHA1 }, Some(review)),
        (Delegate::Gate { number: CHECK_GATE, head: SHA1 }, Some(producing)),
        (Delegate::Gate { number: PERSON_GATE, head: SHA1 }, None),
    ] {
        let Some(delegate) = super::change_delegate(&domain, &env, task, kind, None) else {
            panic!("delegate {kind:?}")
        };
        assert_eq!(delegate.llm, expected, "{kind:?}");
    }
}
fn choice(model: u8, effort: Option<u8>) -> super::tasks::LlmChoice {
    super::tasks::LlmChoice {
        later: None,
        account: None,
        main: super::tasks::ModelEffort { model, effort, speed: None },
        subagents: Box::new([]),
    }
}

fn choice_at_speed(model: u8, effort: Option<u8>, speed: &[u8]) -> super::tasks::LlmChoice {
    let mut llm = choice(model, effort);
    llm.main.speed = Some(skein_lib::Name::from_checked(speed).expect("speed fixture fits"));
    llm
}

#[test]
fn change_llm_text_round_trips_each_role_and_optional_effort() {
    use super::{change_llm, tasks};
    for (text, producer, llm) in [
        (b";;".as_slice(), None, crate::ChangeLlm::default()),
        (b"6;;", Some(6), crate::ChangeLlm::default()),
        (b";3;", None, crate::ChangeLlm { producer: Some(choice(3, None)), review: None }),
        (b";;3.0", None, crate::ChangeLlm { producer: None, review: Some(choice(3, Some(0))) }),
        (
            b"6;3.1 0.0 2;3.0",
            Some(6),
            crate::ChangeLlm {
                producer: Some(tasks::LlmChoice {
                    subagents: Box::new([
                        tasks::ModelEffort { model: 0, effort: Some(0), speed: None },
                        tasks::ModelEffort { model: 2, effort: None, speed: None },
                    ]),
                    ..choice(3, Some(1))
                }),
                review: Some(choice(3, Some(0))),
            },
        ),
        (
            b"6;3.1@ultrafast 0.0 2;3.0@standard",
            Some(6),
            crate::ChangeLlm {
                producer: Some(tasks::LlmChoice {
                    subagents: Box::new([
                        tasks::ModelEffort { model: 0, effort: Some(0), speed: None },
                        tasks::ModelEffort { model: 2, effort: None, speed: None },
                    ]),
                    ..choice_at_speed(3, Some(1), b"ultrafast")
                }),
                review: Some(choice_at_speed(3, Some(0), b"standard")),
            },
        ),
        (
            b"4294967295;255.255;0",
            Some(u32::MAX),
            crate::ChangeLlm { producer: Some(choice(u8::MAX, Some(u8::MAX))), review: Some(choice(0, None)) },
        ),
    ] {
        assert_eq!(change_llm::spell(producer, &llm).as_ref(), text);
        assert_eq!(change_llm::read(text), Some((producer, llm)));
    }
}

#[test]
fn change_llm_text_refuses_malformed_parts_and_too_many_entries() {
    for text in [
        b"".as_slice(),
        b";",
        b";;;",
        b"6;3",
        b"0;;",
        b"4294967296;;",
        b"-1;;",
        b"+1;;",
        b"x;;",
        b";256;",
        b";1.256;",
        b";1.;",
        b";.1;",
        b";1.2.3;",
        b";1@;",
        b";1@abcdefghijklmnopqrstuvwxyz0123456;",
        b";1@Fast;",
        b";1@fast@standard;",
        b";;1@fast.speed",
        b";1@\xff;",
        b"; 1;",
        b";1 ;",
        b";1  2;",
        b";1\t2;",
        b";x;",
        b";;1  ",
        b";;\xff",
        b" 6;;",
    ] {
        assert_eq!(super::change_llm::read(text), None, "{text:?}");
    }
    let mut text = skein_lib::List::with_capacity(140);
    super::append(&mut text, b";0");
    for _ in 0..super::tasks::MODELS {
        super::append(&mut text, b" 1");
    }
    text.push(b';').unwrap();
    let accepted = super::change_llm::read(text.as_slice()).expect("main plus MODELS sub-agents");
    assert_eq!(super::change_llm::spell(accepted.0, &accepted.1).as_ref(), text.as_slice());
    text.truncate(text.len() - 1);
    super::append(&mut text, b" 2;");
    assert_eq!(super::change_llm::read(text.as_slice()), None);
}

pub(super) fn change_choice_world() -> (super::Domain, skein_lib::Env<super::Limits>, super::tasks::DelegationContext) {
    change_choice_world_with_producer_speeds(&[])
}

fn change_choice_world_with_producer_speeds(
    speeds: &[&[u8]],
) -> (super::Domain, skein_lib::Env<super::Limits>, super::tasks::DelegationContext) {
    use super::{authority, forge, tasks};
    let mut config = crate::tests::resume_fixture::config(7);
    let chat = crate::tests::resume_fixture::charter();
    let mut producer = chat.clone();
    producer.number = 6;
    producer.policy.model.name = b"producer".as_slice().into();
    let model = &mut producer.policy.model;
    let mut declared = skein_lib::List::with_capacity(u32::try_from(speeds.len()).expect("fixture speeds"));
    for name in speeds {
        declared
            .push(jig_domain_core::Speed {
                name: (*name).into(),
                also: Box::new([]),
                input_price: model.input_price.max(1),
                cached_price: model.cached_price.max(1),
                cache_write_price: model.cache_write_price,
                output_price: model.output_price.max(1),
            })
            .unwrap();
    }
    model.speeds = declared.into_boxed();
    let mut review = chat.clone();
    review.number = 4;
    review.policy.model.name = b"review".as_slice().into();
    config.core.settings.models =
        Box::new([chat.policy.model.clone(), producer.policy.model.clone(), review.policy.model.clone()]);
    config.core.defaults.charters = Box::new([chat, producer, review]);
    config.core.defaults.rules.ceiling.delegation = authority::Delegation {
        kinds: Box::new([
            authority::Executor::Charter(1),
            authority::Executor::Charter(6),
            authority::Executor::Charter(4),
            authority::Executor::Procedure(super::CHANGE_CODE),
        ]),
        tasks: 20,
        depth: 4,
    };
    config.core.seed_projects[0].policy.policy.ceiling = config.core.defaults.rules.ceiling.clone();
    let mut limits = crate::tests::resume_fixture::limits();
    limits.tasks.charters = 6;
    limits.tasks.parameters = 3;
    limits.tasks.executor_kinds = 4;
    limits.authority.executors = 4;
    limits.authority.segments = 8;
    let (mut domain, env) = crate::tests::resume_fixture::root_with_config_and_limits(config, &limits);
    domain.forge.restore_begin(env.wall);
    let mut repository = adopted(1);
    repository.project = 1;
    restore_forge(&mut domain, &env, forge::Stored::Repository(repository));
    let mut authority = jig_domain_core::task_authority(&domain.engine.core().authority.rules().ceiling);
    authority.tools.0 |= jig_domain_core::ToolFamilies::JIG.control.0;
    let context = tasks::DelegationContext {
        project: 1,
        requester: tasks::Party::Deployment { project: 1 },
        deciding: true,
        authority,
        numbers: tasks::Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
        tasks_left: 20,
    };
    (domain, env, context)
}

fn change_choice_batch(
    domain: &super::Domain,
    env: &skein_lib::Env<super::Limits>,
    context: &super::tasks::DelegationContext,
    change: crate::ChangeCall,
) -> Result<Box<[super::super::Delegate]>, super::tasks::Refusal> {
    match super::change_batch(domain, env, 1, context, &[change]) {
        Ok(batch) => Ok(batch),
        Err(super::ChangeRefused::Refusal(why)) => Err(why),
        Err(super::ChangeRefused::Words(words)) => panic!("unexpected refusal {words:?}"),
    }
}

#[test]
fn a_control_changes_procedure_keeps_its_producer_review_and_llm_parameter() {
    use super::tasks;
    let (domain, env, context) = change_choice_world();
    let llm = crate::ChangeLlm { producer: Some(choice(1, None)), review: Some(choice(2, None)) };
    let change = crate::ChangeCall {
        producer: Some(6),
        llm: llm.clone(),
        ..crate::ChangeCall::words(b"Make the fix".as_slice().into(), None)
    };
    let batch = change_choice_batch(&domain, &env, &context, change).unwrap();
    assert_eq!(
        batch[0].authority.delegation.kinds.as_ref(),
        &[tasks::AuthorityExecutor::Charter(6), tasks::AuthorityExecutor::Charter(4)]
    );
    assert_eq!(batch[0].spec.parameters[2], tasks::Parameter::Bytes { name: 11, value: b"6;1;2".as_slice().into() });
    let plain =
        change_choice_batch(&domain, &env, &context, crate::ChangeCall::words(b"Make the fix".as_slice().into(), None))
            .unwrap();
    assert_eq!(plain[0].spec.parameters.len(), 2);
    let llm_only = change_choice_batch(
        &domain,
        &env,
        &context,
        crate::ChangeCall {
            llm: crate::ChangeLlm { producer: Some(choice(0, None)), review: Some(choice(2, None)) },
            ..crate::ChangeCall::words(b"Make the fix".as_slice().into(), None)
        },
    )
    .unwrap();
    assert_eq!(llm_only[0].spec.parameters[2], tasks::Parameter::Bytes { name: 11, value: b";0;2".as_slice().into() });
    assert_eq!(
        llm_only[0].authority.delegation.kinds.as_ref(),
        &[tasks::AuthorityExecutor::Charter(1), tasks::AuthorityExecutor::Charter(4)]
    );
    let same = change_choice_batch(
        &domain,
        &env,
        &context,
        crate::ChangeCall { producer: Some(4), ..crate::ChangeCall::words(b"Make the fix".as_slice().into(), None) },
    )
    .unwrap();
    assert_eq!(same[0].authority.delegation.kinds.as_ref(), &[tasks::AuthorityExecutor::Charter(4)]);
}

fn change_family(questions: &[Option<crate::ChangeQuestions>]) -> Box<[crate::ChangeCall]> {
    let count = u32::try_from(questions.len()).unwrap();
    let mut changes = skein_lib::List::with_capacity(count);
    for question in questions {
        changes
            .push(crate::ChangeCall {
                questions: question.clone(),
                ..crate::ChangeCall::words(b"Family member".as_slice().into(), None)
            })
            .unwrap();
    }
    changes.into_boxed()
}

#[test]
fn mixed_change_routes_share_room_within_each_policy_cap() {
    use crate::ChangeQuestions::{None as NoQuestions, Up};
    use temper_engine_domain_forge_change::Route;
    for (project_route, routes, tasks_left, expected) in
        [(Route::None, [Some(Up), Some(NoQuestions)], 200, [80, 80]), (Route::Up, [None, Some(Up)], 4, [1, 1])]
    {
        let (mut domain, mut env, mut context) = change_choice_world();
        env.limits.forge.change_policy.tasks = 80;
        env.limits.forge.change_policy.questions = 80;
        context.tasks_left = tasks_left;
        context.authority.delegation.tasks = tasks_left;
        set_project_questions(&mut domain, &env, project_route);
        let changes = change_family(&routes);
        let batch = match super::change_batch(&domain, &env, 1, &context, &changes) {
            Ok(batch) => batch,
            Err(super::ChangeRefused::Refusal(why)) => panic!("{tasks_left} tasks left: {why:?}"),
            Err(super::ChangeRefused::Words(words)) => panic!("{tasks_left} tasks left: {words:?}"),
        };
        assert_eq!(batch.len(), expected.len());
        for (member, allowance) in batch.iter().zip(expected) {
            assert_eq!(member.authority.delegation.tasks, allowance);
        }
    }
}

#[test]
fn insufficient_change_room_names_capacity_and_whether_smaller_calls_help() {
    for (available, figure) in [(0, "available 0 tasks"), (1, "available 1 tasks"), (7, "available 7 tasks")] {
        let (domain, env, mut context) = change_choice_world();
        context.tasks_left = available;
        let changes = change_family(&[None, None, None, None]);
        let Err(super::ChangeRefused::Words(words)) = super::change_batch(&domain, &env, 1, &context, &changes) else {
            panic!("insufficient batch must be refused before constructing delegates")
        };
        let advice = if available >= 2 {
            "Fewer changes per call can fit; ended tasks still count against lifetime authority."
        } else {
            "Fewer changes per call cannot help: even one change needs two tasks of lifetime authority."
        };
        assert!(words.starts_with(b"change batch: task tree limit;"));
        assert!(skein_lib::bytes::find(&words, figure.as_bytes()).is_some(), "{words:?}");
        assert!(
            skein_lib::bytes::find(&words, b"required 8 (one change task and at least one descendant per member)")
                .is_some(),
            "{words:?}"
        );
        assert!(words.ends_with(advice.as_bytes()), "{words:?}");
    }
}

#[test]
fn a_changes_delegation_uses_its_task_limit_and_leaves_room_for_foremen() {
    use crate::ChangeQuestions;
    for (tasks, questions, tasks_left, expected) in [
        (6, Some(ChangeQuestions::None), 20, 6),
        (6, None, 20, 6),
        (8, Some(ChangeQuestions::Up), 20, 8),
        (u32::MAX, Some(ChangeQuestions::Up), 20, 19),
    ] {
        let (domain, mut env, mut context) = change_choice_world();
        env.limits.forge.change_policy.tasks = tasks;
        env.limits.forge.change_policy.questions = 4;
        context.tasks_left = tasks_left;
        let change = crate::ChangeCall {
            questions: questions.clone(),
            ..crate::ChangeCall::words(b"Make the fix".as_slice().into(), None)
        };
        let batch = change_choice_batch(&domain, &env, &context, change).unwrap();
        assert_eq!(batch[0].authority.delegation.tasks, expected, "tasks: {tasks}, questions: {questions:?}");
    }
}

#[test]
fn a_changes_unoffered_producer_is_refused_as_an_executor() {
    let (domain, env, context) = change_choice_world();
    let change =
        crate::ChangeCall { producer: Some(5), ..crate::ChangeCall::words(b"Make the fix".as_slice().into(), None) };
    assert_eq!(change_choice_batch(&domain, &env, &context, change).unwrap_err(), super::tasks::Refusal::Executor);
}

#[test]
fn a_changes_unset_or_unoffered_forge_charter_is_refused_as_an_executor() {
    use super::forge::settings::{SizedValue, Stock, Value};
    for (change, review) in [(0, 4), (5, 4), (1, 0), (1, 5)] {
        let (mut domain, env, context) = change_choice_world();
        domain.forge.give_settings_defaults(Box::new([
            Stock { kind: 3, scope: 0, number: 1, value: SizedValue { size: 4, value: Value::Charter(change) } },
            Stock { kind: 3, scope: 0, number: 2, value: SizedValue { size: 4, value: Value::Charter(review) } },
        ]));
        let change = crate::ChangeCall::words(b"Make the fix".as_slice().into(), None);
        assert_eq!(change_choice_batch(&domain, &env, &context, change).unwrap_err(), super::tasks::Refusal::Executor);
    }
}

#[test]
fn a_changes_llm_choices_are_checked_against_their_own_charters_and_run_models() {
    use super::tasks;
    let (domain, env, mut context) = change_choice_world();
    for (llm, model) in [
        (crate::ChangeLlm { producer: Some(choice(2, None)), review: None }, 2),
        (crate::ChangeLlm { producer: None, review: Some(choice(1, None)) }, 1),
    ] {
        let change = crate::ChangeCall {
            producer: Some(6),
            llm,
            ..crate::ChangeCall::words(b"Make the fix".as_slice().into(), None)
        };
        assert_eq!(
            change_choice_batch(&domain, &env, &context, change).unwrap_err(),
            tasks::Refusal::Llm { model, why: tasks::LlmFault::Charter }
        );
    }
    let llm = crate::ChangeLlm { producer: Some(choice(1, None)), review: Some(choice(2, None)) };
    context.authority.models = tasks::Models::from_parts(1, false);
    let change = crate::ChangeCall {
        producer: Some(6),
        llm,
        ..crate::ChangeCall::words(b"Make the fix".as_slice().into(), None)
    };
    assert_eq!(
        change_choice_batch(&domain, &env, &context, change).unwrap_err(),
        tasks::Refusal::Llm { model: 1, why: tasks::LlmFault::Authority }
    );
}

fn a_change_with_producer_choice(choice: super::tasks::LlmChoice) -> crate::ChangeCall {
    crate::ChangeCall {
        producer: Some(6),
        llm: crate::ChangeLlm { producer: Some(choice), review: None },
        ..crate::ChangeCall::words(b"Make the fix".as_slice().into(), None)
    }
}

#[test]
fn an_undeclared_change_speed_names_the_models_speeds_in_order() {
    for (declared, expected) in [
        (
            &[b"turbo".as_slice(), b"fast"][..],
            b"speed ultrafast is not one producer declares; it declares turbo, fast".as_slice(),
        ),
        (&[][..], b"speed ultrafast is not one producer declares; it declares none"),
    ] {
        let (domain, env, context) = change_choice_world_with_producer_speeds(declared);
        // The same model may run as main and sub-agent, at different speeds.
        let change = a_change_with_producer_choice(super::tasks::LlmChoice {
            subagents: Box::new([choice_at_speed(1, None, b"ultrafast").main]),
            ..choice_at_speed(1, None, b"standard")
        });
        let Err(super::ChangeRefused::Words(words)) = super::change_batch(&domain, &env, 1, &context, &[change]) else {
            panic!("the producer's speed was not refused in words")
        };
        assert_eq!(words.as_ref(), expected);
    }
}

#[test]
fn change_speed_choices_accept_declared_standard_and_omitted() {
    let (domain, env, context) = change_choice_world_with_producer_speeds(&[b"ultrafast"]);
    for (choice, expected) in [
        (choice_at_speed(1, None, b"ultrafast"), b"6;1@ultrafast;".as_slice()),
        (choice_at_speed(1, None, b"standard"), b"6;1@standard;"),
        (choice(1, None), b"6;1;"),
    ] {
        let batch = change_choice_batch(&domain, &env, &context, a_change_with_producer_choice(choice)).unwrap();
        assert!(
            batch[0].spec.parameters.contains(&super::tasks::Parameter::Bytes { name: 11, value: expected.into() })
        );
    }
}

#[test]
fn jig_admission_refuses_a_procedures_producer_beyond_its_creators_kinds() {
    use super::authority;
    let (domain, env, context) = change_choice_world();
    let change =
        crate::ChangeCall { producer: Some(6), ..crate::ChangeCall::words(b"Make the fix".as_slice().into(), None) };
    let batch = change_choice_batch(&domain, &env, &context, change).unwrap();
    let member = &batch[0];
    let creator = domain.engine.core().authority.rules().ceiling.clone();
    let mut child = creator.clone();
    child.delegation.kinds = Box::new([authority::Executor::Charter(6), authority::Executor::Charter(4)]);
    assert_eq!(jig_domain_core::task_authority(&child).delegation.kinds, member.authority.delegation.kinds);
    child.delegation.tasks = member.authority.delegation.tasks;
    child.delegation.depth = member.authority.delegation.depth;
    child.budget.spend = member.authority.budget.spend;
    let mut ask = authority::BatchAsk {
        project: 1,
        creator,
        numbers: authority::Numbers { budget: 100, spent: 0, spent_below: 0, reserved: 0 },
        tasks_left: context.tasks_left,
        tasks: Box::new([authority::Delegate {
            executor: authority::Executor::Procedure(super::CHANGE_CODE),
            authority: child,
            symbolic: Box::new([]),
        }]),
    };
    let mut why = skein_lib::Queue::uncapped();
    assert_eq!(
        authority::check_batch(&domain.engine.core().authority, &ask, &mut why).answer,
        authority::Answer::Allow
    );
    assert!(why.is_empty());
    ask.creator.delegation.kinds = Box::new([
        authority::Executor::Charter(1),
        authority::Executor::Charter(4),
        authority::Executor::Procedure(super::CHANGE_CODE),
    ]);
    assert_eq!(
        authority::check_batch(&domain.engine.core().authority, &ask, &mut why).answer,
        authority::Answer::Propose
    );
    let Some(authority::Finding::Authority { source: authority::Source::Task, lacks }) = why.pop() else {
        panic!("the procedure's producer exceeds its creator's delegation kinds");
    };
    assert!(lacks.executors);
}

pub(super) fn change_start_fixture() -> (super::Domain, skein_lib::Env<super::Limits>, Box<super::tasks::RunContext>) {
    use super::{forge, tasks};
    let (mut domain, mut env, task) =
        crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: super::CHANGE_CODE });
    env.limits.authority.segments = 8;
    let mut context = domain.engine.core().tasks.run_context(task).unwrap();
    context.spec.words = b"Make the fix".as_slice().into();
    context.spec.parameters = Box::new([
        tasks::Parameter::Resource { name: 1, connector: domain.config.forge_connector, resource: forge::tag(1, 1) },
        tasks::Parameter::Bytes { name: 3, value: b"temper/change".as_slice().into() },
    ]);
    let mut repository = adopted(1);
    repository.project = context.project;
    domain.forge.restore_begin(env.wall);
    restore_forge(&mut domain, &env, forge::Stored::Repository(repository));
    (domain, env, context)
}

fn follows_start(
    text: &[u8],
    route: Option<u64>,
) -> (bool, skein_lib::Queue<super::Work>, Option<super::forge::ChangeRow>) {
    use super::tasks;
    let (mut domain, mut env, mut context) = change_start_fixture();
    env.limits.forge.follow_merge = 2;
    context.requester = tasks::Party::Task(33);
    let mut parameters = skein_lib::List::with_capacity(4);
    for parameter in &context.spec.parameters {
        parameters.push(parameter.clone()).unwrap();
    }
    parameters.push(tasks::Parameter::Bytes { name: 18, value: text.into() }).unwrap();
    if let Some(value) = route {
        parameters.push(tasks::Parameter::Number { name: 14, value }).unwrap();
    }
    context.spec.parameters = parameters.into_boxed();
    if let Some(follows) = super::change_follows(text, env.limits.forge.follow_merge) {
        follow_fixture(&mut domain, &env, context.project, &follows.commits);
    }
    let accepted = super::start_change(&mut domain, &env, &context);
    let row = domain.forge.change(context.task).cloned();
    (accepted, domain.work, row)
}

#[test]
#[expect(clippy::disallowed_methods, reason = "parameter fixtures join text fragments at both commit widths")]
fn a_changes_start_keeps_follows_in_order_and_defaults_its_questions_up() {
    use super::{forge, forge_change};
    let mut first = [0_u8; 32];
    first[..20].fill(0xaa);
    let mut second = [0_u8; 32];
    second[..20].fill(0xbb);
    for (text, expected, route, questions) in [
        (
            [b"3:".as_slice(), &[b'a'; 40], b",", &[b'b'; 40]].concat(),
            Box::<[_]>::from([first, second]),
            None,
            forge_change::Route::Up,
        ),
        ([b"3:".as_slice(), &[b'c'; 64]].concat(), Box::<[_]>::from([[0xcc; 32]]), Some(2), forge_change::Route::Up),
    ] {
        let (accepted, _, row) = follows_start(&text, route);
        assert!(accepted, "{text:?}");
        let row = row.unwrap();
        assert_eq!(row.follows, Some(forge::ChangeFollows { follow: 3, commits: expected }));
        assert_eq!(row.questions.route, questions);
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "malformed parameter fixtures join bounded text fragments")]
fn malformed_follows_refuses_the_change_start_without_queuing_work() {
    let commit = [b'a'; 40];
    for text in [
        commit.to_vec(), // no colon
        b"3:".to_vec(),
        [b"x:".as_slice(), &commit].concat(),
        [b"3:".as_slice(), &[b'A'; 40]].concat(),
        [b"3:".as_slice(), &[b'a'; 39]].concat(),
        [b"3:".as_slice(), &commit, b",", &commit, b",", &commit].concat(),
        [b"4294967296:".as_slice(), &commit].concat(),
        [b"3:".as_slice(), &commit, b","].concat(),
    ] {
        let (accepted, work, row) = follows_start(&text, None);
        assert!(row.is_none());
        assert!(!accepted, "malformed follows: {text:?}");
        assert!(work.is_empty(), "no row or step for malformed parameter 18");
    }
}

#[test]
fn a_changes_start_keeps_parameter_elevens_charter_and_role_choices() {
    use super::tasks;
    for (text, producer, llm) in [
        (None, 1, crate::ChangeLlm::default()),
        (
            Some(b"6;3.1@ultrafast 0.0 2;3.0@standard".as_slice()),
            6,
            crate::ChangeLlm {
                producer: Some(tasks::LlmChoice {
                    subagents: Box::new([
                        tasks::ModelEffort { model: 0, effort: Some(0), speed: None },
                        tasks::ModelEffort { model: 2, effort: None, speed: None },
                    ]),
                    ..choice_at_speed(3, Some(1), b"ultrafast")
                }),
                review: Some(choice_at_speed(3, Some(0), b"standard")),
            },
        ),
        (Some(b";0;"), 1, crate::ChangeLlm { producer: Some(choice(0, None)), review: None }),
        (Some(b";;"), 1, crate::ChangeLlm::default()),
    ] {
        let (mut domain, env, mut context) = change_start_fixture();
        if let Some(text) = text {
            let mut parameters = skein_lib::List::with_capacity(3);
            for parameter in &context.spec.parameters {
                parameters.push(parameter.clone()).unwrap();
            }
            parameters.push(tasks::Parameter::Bytes { name: 11, value: text.into() }).unwrap();
            context.spec.parameters = parameters.into_boxed();
        }
        assert!(super::start_change(&mut domain, &env, &context));
        let row = domain.forge.change(context.task).unwrap();
        assert_eq!(row.producing_charter, producer);
        assert_eq!(row.review_charter, 4);
        assert_eq!(row.producing_llm, llm.producer);
        assert_eq!(row.review_llm, llm.review);
    }
}

#[test]
fn a_requesters_branch_and_role_choices_reach_start_together() {
    use super::{Work, forge, tasks};
    let (domain, mut env, delegation) = change_choice_world();
    env.limits.tasks.parameters = 7; // Branch, title, role choices and question route beside repository/base.
    let llm = crate::ChangeLlm { producer: Some(choice(1, None)), review: Some(choice(2, None)) };
    let change = crate::ChangeCall {
        branch: Some(crate::ChangeBranch::Mine),
        producer: Some(6),
        questions: Some(crate::ChangeQuestions::Up),
        llm: llm.clone(),
        ..crate::ChangeCall::words(b"Land the goal\r\nWhole branch".as_slice().into(), None)
    };
    let branch = b"temper/7/g8".as_slice();
    let delegate = super::chat_change(
        &domain,
        &env,
        &super::ChangeOrigin {
            authority: delegation.authority.clone(),
            tasks_left: delegation.tasks_left,
            source: None,
        },
        &change,
        &adopted(1),
        b"main".as_slice().into(),
        Some(branch.into()),
        50,
    )
    .unwrap();
    assert_eq!(
        delegate.spec.parameters.as_ref(),
        &[
            tasks::Parameter::Resource {
                name: 1,
                connector: domain.config.forge_connector,
                resource: forge::tag(1, 1)
            },
            tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
            tasks::Parameter::Bytes { name: 3, value: branch.into() },
            tasks::Parameter::Number { name: 11, value: 1 },
            tasks::Parameter::Bytes { name: 10, value: b"Land the goal".as_slice().into() },
            tasks::Parameter::Bytes { name: 11, value: b"6;1;2".as_slice().into() },
            tasks::Parameter::Number { name: 14, value: 2 },
        ]
    );
    let (mut domain, env, mut context) = change_start_fixture();
    context.requester = tasks::Party::Task(1);
    context.spec = delegate.spec;
    assert!(super::start_change(&mut domain, &env, &context));
    let row = domain.forge.change(context.task).unwrap();
    assert_eq!(row.branch.as_ref(), branch);
    assert_eq!(row.title.as_ref(), b"Land the goal");
    assert_eq!(row.questions.route, temper_engine_domain_forge_change::Route::Up);
    assert_eq!(row.producing_charter, 6);
    assert_eq!(row.review_charter, 4);
    assert_eq!(row.producing_llm, llm.producer);
    assert_eq!(row.review_llm, llm.review);
    while let Some(work) = domain.work.pop() {
        if let Work::Forge(forge::Event::Hold { .. }) = work {
            unreachable!("the requester keeps its branch hold");
        }
    }
}

#[test]
fn a_malformed_parameter_eleven_starts_no_change_and_the_route_holds_the_task() {
    use super::{Work, tasks};
    for parameter in [
        tasks::Parameter::Bytes { name: 11, value: b"6;3..1;".as_slice().into() },
        tasks::Parameter::Bytes { name: 11, value: b"0;;".as_slice().into() },
        tasks::Parameter::Number { name: 11, value: 6 },
    ] {
        let (mut domain, env, mut context) = change_start_fixture();
        let mut parameters = skein_lib::List::with_capacity(3);
        for parameter in &context.spec.parameters {
            parameters.push(parameter.clone()).unwrap();
        }
        parameters.push(parameter).unwrap();
        context.spec.parameters = parameters.into_boxed();
        assert!(!super::start_change(&mut domain, &env, &context));
        assert!(domain.forge.change(context.task).is_none());
        assert!(domain.work.is_empty());
        super::start_refused(&mut domain, context.task);
        let Some(Work::Tasks(tasks::Event::HoldFrom { task, why: tasks::Hold::Procedure, .. })) = domain.work.pop()
        else {
            panic!("malformed start holds the procedure")
        };
        assert_eq!(task, context.task);
        assert!(domain.work.is_empty());
    }
}

#[test]
fn a_changes_spending_floors_use_its_saved_charters() {
    use super::forge;
    for (producer, review, found) in [(1, 1, true), (2, 1, false), (1, 2, false)] {
        let (mut domain, env, context) = change_start_fixture();
        let mut row = change_row(None);
        row.task = context.task;
        row.change.task = context.task;
        row.repository = skein_forge::api::Repository { forge: 1, repository: 1 };
        row.producing_charter = producer;
        row.review_charter = review;
        domain.forge.give_settings_defaults(Box::new([forge::settings::Stock {
            kind: 3,
            scope: 0,
            number: 1,
            value: forge::settings::SizedValue { size: 4, value: forge::settings::Value::Charter(2) },
        }]));
        let expected = jig_domain_core::reserve_floor(&domain.engine.core().charter(1).unwrap().policy).unwrap();
        forge::step(
            &mut domain.forge,
            &env.with(env.limits.forge),
            forge::Event::Change { row },
            &mut skein_lib::Queue::uncapped(),
        );
        assert!(domain.forge.change(context.task).is_some());
        let spend = super::change_spend(&domain, context.task);
        assert_eq!(spend.is_some(), found);
        if let Some(spend) = spend {
            assert_eq!(spend.coding, expected);
            assert_eq!(spend.review, expected);
        }
    }
}

#[test]
fn a_changes_refused_model_holds_the_role_that_needed_its_delegate() {
    use super::{Work, forge, forge_change, tasks};
    use forge_change::{Delegate, Freshness, Gate, GateKind, Repair};
    for (kind, fault, role) in [
        (Delegate::Produce, tasks::LlmFault::Charter, Some(b"producer".as_slice())),
        (Delegate::Produce, tasks::LlmFault::Speed, Some(b"producer")),
        (Delegate::Gate { number: 1, head: SHA1 }, tasks::LlmFault::Speed, Some(b"review")),
        (Delegate::Repair(Repair::Ci), tasks::LlmFault::Charter, Some(b"producer")),
        (Delegate::Resolve { base: SHA1 }, tasks::LlmFault::Charter, Some(b"producer")),
        (Delegate::Gate { number: 1, head: SHA1 }, tasks::LlmFault::Charter, Some(b"review")),
        (Delegate::Gate { number: 2, head: SHA1 }, tasks::LlmFault::Charter, Some(b"producer")),
        (Delegate::Gate { number: 3, head: SHA1 }, tasks::LlmFault::Charter, None),
        (Delegate::Produce, tasks::LlmFault::Authority, None),
        (Delegate::Produce, tasks::LlmFault::Unconfigured, None),
    ] {
        let (mut domain, env, task) =
            crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: super::CHANGE_CODE });
        let mut row = task_change_row(task, Some(7));
        row.producing_charter = 2;
        row.review_charter = 4;
        row.producing_llm = Some(choice_at_speed(0, None, b"ultrafast"));
        row.review_llm = Some(choice_at_speed(0, None, b"fast"));
        let mut gates = skein_lib::List::with_capacity(3);
        for (number, kind) in [(1, GateKind::Agent), (2, GateKind::Check), (3, GateKind::Person)] {
            gates.push(Gate { number, kind, blocking: true, freshness: Freshness::Exact, eager: false }).unwrap();
        }
        row.change.gates = gates.into_boxed();
        domain.forge.restore_begin(env.wall);
        restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Change(Box::new(row)));
        let (connector, _, step) = domain.engine.core().tasks.procedure_due(task).expect("due change");
        domain
            .forge_delegating
            .insert(
                (task, step),
                super::ChangeStep {
                    delegate: Some((kind, None)),
                    relayed: None,
                    refusal: None,
                    released: false,
                    passing: None,
                },
            )
            .unwrap();
        let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
        super::super::asks::ask(
            &mut domain,
            &env,
            &mut decision,
            connector,
            jig_domain_contract::Ask::Procedures(jig_domain_contract::procedures::Ask::StepRefused {
                task,
                step,
                why: jig_domain_core::StepRefusal::Refused(tasks::Problem {
                    task: None,
                    why: tasks::Refusal::Llm { model: 0, why: fault },
                    blocked_by: None,
                    capacity: None,
                }),
            }),
        );
        match role {
            Some(role) => {
                let Some(Work::Tasks(tasks::Event::HoldFrom { task: held, source, .. })) = domain.work.pop() else {
                    panic!("charter model refusal holds the change")
                };
                assert_eq!(held, task);
                let tasks::HoldSource::Account { words, .. } = *source else { panic!("forge's account") };
                let mut expected = skein_lib::List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
                super::append(&mut expected, b"the change's ");
                super::append(&mut expected, role);
                super::append(&mut expected, b" choice names model ");
                super::append(&mut expected, &domain.engine.core().settings.models[0].name);
                if fault == tasks::LlmFault::Speed {
                    super::append(
                        &mut expected,
                        if role == b"review" { b" at speed fast" } else { b" at speed ultrafast" },
                    );
                }
                super::append(
                    &mut expected,
                    if role == b"review" { b", which charter 4" } else { b", which charter 2" },
                );
                super::append(&mut expected, b" no longer allows; cancel the change or make it again");
                assert_eq!(words.as_ref(), expected.as_slice());
            }
            None => assert!(domain.work.is_empty(), "other refusals and person gates keep the ordinary retry"),
        }
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table compares each pending dependency kind against the same restored task graph"
)]
fn pending_dependencies_distinguish_changes_elsewhere_and_other_tasks() {
    use super::{forge, forge_change as change, tasks};
    use skein_lib::Queue;
    let (fixture, _, task) = crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: 2 });
    let template = fixture.engine.core().tasks.task(task).unwrap().clone();
    for (registered, repository, base, executor, checked, kept, expected) in [
        (
            true,
            3,
            b"main".as_slice(),
            tasks::Executor::Procedure { connector: 0, code: 2 },
            Some(SHA1),
            false,
            change::PendingKind::Change { checked: Some(SHA1), kept: false },
        ),
        (
            true,
            3,
            b"main".as_slice(),
            tasks::Executor::Procedure { connector: 0, code: 2 },
            None,
            true,
            change::PendingKind::Change { checked: None, kept: true },
        ),
        (
            true,
            1,
            b"main".as_slice(),
            tasks::Executor::Procedure { connector: 0, code: 2 },
            None,
            false,
            change::PendingKind::Elsewhere,
        ),
        (
            true,
            3,
            b"other".as_slice(),
            tasks::Executor::Procedure { connector: 0, code: 2 },
            None,
            false,
            change::PendingKind::Other,
        ),
        (
            false,
            3,
            b"main".as_slice(),
            tasks::Executor::Procedure { connector: 0, code: 1 },
            None,
            false,
            change::PendingKind::Other,
        ),
        (
            false,
            3,
            b"main".as_slice(),
            tasks::Executor::Procedure { connector: 0, code: 2 },
            None,
            false,
            change::PendingKind::Change { checked: None, kept: false },
        ),
    ] {
        let (mut domain, env) = two_forges();
        reset_tasks_for_restore(&mut domain, &env, 7);
        let mut row = template.clone();
        row.number = 7;
        row.dependencies = Box::new([8]);
        row.waiting_on = Box::new([8]);
        let mut upstream = template.clone();
        upstream.number = 8;
        upstream.executor = executor;
        upstream.waiting_on = if kept { Box::new([9]) } else { Box::new([]) };
        let task_env = env.with(env.limits.tasks);
        for row in [row, upstream] {
            tasks::step(
                &mut domain.engine.core_mut().tasks,
                &task_env,
                tasks::Event::Restore { record: tasks::Stored::Live(Box::new(row)) },
                &mut Queue::uncapped(),
            );
        }
        assert!(domain.engine.core().tasks.task(7).is_some());
        assert!(domain.engine.core().tasks.task(8).is_some());
        let forge_env = super::super::environment_forge(&env);
        let mut upstream = change_row(None);
        upstream.task = 8;
        upstream.change.task = 8;
        upstream.repository.forge = repository;
        upstream.base = base.into();
        upstream.branch = b"temper/8".as_slice().into();
        upstream.change.last_checked = checked;
        if registered {
            restore_in_forge_env(&mut domain, &forge_env, forge::Stored::Change(Box::new(upstream.clone())));
        }
        {
            let commit = [9; 32];
            restore_in_forge_env(
                &mut domain,
                &forge_env,
                forge::Stored::BranchHead(forge::BranchHead {
                    name: forge::Name {
                        forge: repository,
                        repository: 1,
                        what: forge::What::Branch(Box::new([b"temper".as_slice().into(), b"8".as_slice().into()])),
                    },
                    commit,
                    owned: true,
                }),
            );
        }
        let pending = super::landing_route::pending(&domain, &env, 7, change_row(None).repository, b"main");
        assert_eq!(pending.as_ref(), &[change::Pending { task: 8, kind: expected }]);
    }
}

#[test]
fn a_consumed_task_release_is_not_replayed_after_restore() {
    use super::{forge, tasks};
    let (fixture, _, task) = crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: 2 });
    for (measure, revision) in [(false, 1), (false, 2), (true, 1), (true, 2)] {
        let (mut domain, env) = two_forges();
        reset_tasks_for_restore(&mut domain, &env, 7);
        let mut task = fixture.engine.core().tasks.task(task).unwrap().clone();
        task.number = 7;
        task.due_reason = if measure { tasks::Why::Measured } else { tasks::Why::Released };
        task.revisions.hold = revision;
        task_event(&mut domain, &env, tasks::Event::Restore { record: tasks::Stored::Live(Box::new(task)) });
        let mut row = change_row(None);
        row.task_release = Some(forge::TaskRelease { revision: 1, at: env.wall, measure });
        restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Change(Box::new(row)));
        assert_eq!(
            super::delegate_record::is_release(&domain, 7),
            revision != 1,
            "a consumed revision survives restart; a later release remains fresh"
        );
    }
}

#[test]
fn a_dependency_head_refreshes_a_change_already_reading_its_facts() {
    use super::{Work, forge, forge_change as change, tasks};
    let (fixture, _, task) = crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: 2 });
    let template = fixture.engine.core().tasks.task(task).unwrap().clone();
    let (mut domain, env) = two_forges();
    reset_tasks_for_restore(&mut domain, &env, 7);
    let mut waiting = template.clone();
    waiting.number = 7;
    waiting.phase = tasks::Phase::Active(tasks::Active::Due);
    waiting.dependencies = Box::new([8]);
    waiting.waiting_on = Box::new([8]);
    let mut upstream = template;
    upstream.number = 8;
    upstream.phase = tasks::Phase::Active(tasks::Active::Idle);
    for row in [waiting, upstream] {
        task_event(&mut domain, &env, tasks::Event::Restore { record: tasks::Stored::Live(Box::new(row)) });
    }
    let forge_env = super::super::environment_forge(&env);
    let mut upstream = change_row(None);
    upstream.task = 8;
    upstream.change.task = 8;
    upstream.branch = b"temper/8".as_slice().into();
    for row in [change_row(None), upstream.clone()] {
        restore_in_forge_env(&mut domain, &forge_env, forge::Stored::Change(Box::new(row)));
    }
    // The current asynchronous step began before the dependency's first push.
    let stale = super::change_step(&domain, &env, 7, 1, change_row(None).repository, b"main", false);
    let forge::Event::StepChange { pending, .. } = stale else { panic!("change step") };
    assert_eq!(
        pending.as_ref(),
        &[change::Pending { task: 8, kind: change::PendingKind::Change { checked: None, kept: false } }]
    );
    restore_in_forge_env(
        &mut domain,
        &forge_env,
        forge::Stored::BranchHead(forge::BranchHead {
            name: forge::Name {
                forge: 3,
                repository: 1,
                what: forge::What::Branch(Box::new([b"temper".as_slice().into(), b"8".as_slice().into()])),
            },
            commit: [9; 32],
            owned: true,
        }),
    );
    upstream.change.last_checked = Some(SHA1);
    restore_in_forge_env(&mut domain, &forge_env, forge::Stored::Change(Box::new(upstream)));
    // A checked-head notification must update the outstanding step, not send an idle-only wake.
    forge_request(&mut domain, &env, forge::Request::StepDependents { task: 8 });
    let Some(Work::Forge(forge::Event::StepChange { task: 7, pending, .. })) = domain.work.pop() else {
        panic!("a dependency wake refreshes the already-due step");
    };
    assert_eq!(
        pending.as_ref(),
        &[change::Pending { task: 8, kind: change::PendingKind::Change { checked: Some(SHA1), kept: false } }]
    );
    assert!(domain.work.is_empty());
}

#[test]
fn a_held_branch_change_refuses_each_parameter_past_capacity() {
    use super::tasks;
    const PARAMETERS: u32 = 7; // Repository, base, branch, requester marker, title, role choices and question route.
    const SPEND: u64 = 1000; // Within the fixture's run budget and above its minimum.
    let (domain, mut env, mut context) = change_choice_world();
    context.tasks_left = 8;
    context.authority.delegation.depth = 4;
    env.limits.authority.segments = 6; // forge, host, owner, repository, branch and base.
    let mut change = crate::ChangeCall::words(b"Land the goal\r\nIts report".as_slice().into(), None);
    change.questions = Some(crate::ChangeQuestions::Up);
    change.producer = Some(6);
    change.llm = crate::ChangeLlm { producer: Some(choice(1, None)), review: Some(choice(2, None)) };
    for capacity in 0..=PARAMETERS {
        env.limits.tasks.parameters = capacity;
        let result = super::chat_change(
            &domain,
            &env,
            &super::ChangeOrigin { authority: context.authority.clone(), tasks_left: context.tasks_left, source: None },
            &change,
            &adopted(1),
            b"main".as_slice().into(),
            Some(b"temper/goal".as_slice().into()),
            SPEND,
        );
        if capacity < PARAMETERS {
            assert_eq!(result.unwrap_err().task_refusal(), tasks::Refusal::Spec, "capacity {capacity}");
        } else {
            assert_eq!(result.unwrap().spec.parameters.len(), usize::try_from(PARAMETERS).unwrap());
        }
    }
}

#[test]
fn clearing_precedes_a_long_pull_address_inside_the_task_window() {
    let (mut domain, env) = cleared_repair_result(None);
    result_pull_address(&mut domain, &env);
    let verification = super::forge::Verification {
        repository: change_row(Some(7)).repository,
        commit: SHA1,
        changes: Box::new([7]),
        tiers: Box::new([super::forge::TierVerification {
            tier: Box::from(b"slow".as_slice()),
            verdict: Some(super::forge::TierVerdict {
                run: 2,
                blame: temper_engine_domain_forge_tiers::Blame::Cleared,
            }),
            repaired: None,
        }]),
    };
    restore_forge_with_engine_waits(&mut domain, &env, super::forge::Stored::Verification(verification));
    let words = super::change_landing_words(&domain, 7, &SHA1, 7, 512);
    assert!(words.starts_with(b"abcdef0123456789abcdef0123456789abcdef01\nCleared by slow run 2.\n"));
    assert!(skein_lib::bytes::find(&words, &[b'x'; 180]).is_some(), "long address fits the stored result");
    assert!(skein_lib::bytes::find(&words[..128], b"Cleared by slow run 2.").is_some());
}

fn result_pull_address(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>) {
    use super::forge;
    let base = b"https://forge.example/";
    let mut web = skein_lib::Writer::new(base.len().checked_add(180).unwrap());
    web.put(base).unwrap();
    web.put(&[b'x'; 180]).unwrap();
    let web = web.finish();
    let row = forge::ForgeRecord {
        person: None,
        number: change_row(Some(7)).repository.forge,
        kind: forge::Kind::Forgejo,
        mode: forge::Mode::Connected,
        state: forge::ForgeState::Kept,
        place: forge::Place {
            api: forge::Address { host: b"forge.example".as_slice().into(), port: 443, tls: None },
            git: web.clone(),
            web: Some(web),
        },
        git_program: None,
        read_only: false,
        bot: forge::Bot { login: b"bot".as_slice().into(), name: Box::new([]), email: Box::new([]) },
        asked: false,
        first: None,
    };
    restore_forge_with_engine_waits(domain, env, forge::Stored::Forge(row));
}

#[test]
fn a_live_room_refusal_rearms_the_change_delegate() {
    use super::{Work, forge, forge_change, tasks};
    let (mut domain, env, context) = change_start_fixture();
    let task = context.task;
    let (connector, _, step) = domain.engine.core().tasks.procedure_due(task).expect("due change");
    domain
        .forge_delegating
        .insert(
            (task, step),
            super::ChangeStep {
                delegate: Some((forge_change::Delegate::Produce, None)),
                relayed: None,
                refusal: None,
                released: false,
                passing: None,
            },
        )
        .unwrap();
    let problem = tasks::Problem::capacity(
        Some(task),
        tasks::Refusal::Live,
        tasks::Capacity {
            project: Some(context.project),
            by: tasks::Party::Task(task),
            action: tasks::CapacityAction::Batch { first: None, members: 1 },
            cause: tasks::CapacityCause::Tasks { limit: env.limits.tasks.tasks },
        },
    );
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::super::asks::ask(
        &mut domain,
        &env,
        &mut decision,
        connector,
        jig_domain_contract::Ask::Procedures(jig_domain_contract::procedures::Ask::StepRefused {
            task,
            step,
            why: jig_domain_core::StepRefusal::Refused(problem),
        }),
    );
    let Some(Work::Forge(forge::Event::DelegateAdmissionRefused {
        task: refused,
        kind: forge_change::Delegate::Produce,
    })) = domain.work.pop()
    else {
        panic!("the refused producer can be asked again")
    };
    assert_eq!(refused, task);
    assert!(domain.work.is_empty());
    assert!(domain.forge_delegating.is_empty());
}

fn set_project_questions(
    domain: &mut super::Domain,
    env: &skein_lib::Env<super::Limits>,
    route: temper_engine_domain_forge_change::Route,
) {
    use super::forge::settings::{Asked, Edit, SizedValue, Value};
    domain.forge.keep_settings(
        &crate::translate::environment_forge(env),
        2,
        None,
        &[Asked {
            kind: 6,
            scope: 1,
            number: 1,
            edit: Edit::Set(SizedValue { value: Value::Questions(route), size: 1 }),
        }],
        &mut skein_lib::Queue::uncapped(),
    );
}

fn questions_delegate(
    domain: &super::Domain,
    env: &skein_lib::Env<super::Limits>,
    context: &super::tasks::DelegationContext,
    questions: Option<crate::ChangeQuestions>,
) -> super::super::Delegate {
    let change = crate::ChangeCall { questions, ..crate::ChangeCall::words(b"Make the fix".as_slice().into(), None) };
    let batch = change_choice_batch(domain, env, context, change).unwrap();
    batch[0].clone()
}

fn kept_route(delegate: super::super::Delegate) -> temper_engine_domain_forge_change::Route {
    let (mut domain, env, mut context) = change_start_fixture();
    context.requester = super::tasks::Party::Task(1);
    context.spec = delegate.spec;
    assert!(super::start_change(&mut domain, &env, &context));
    domain.forge.change(context.task).unwrap().questions.route
}

#[test]
fn a_change_naming_no_route_takes_its_projects_questions() {
    use temper_engine_domain_forge_change::Route;
    for route in [Route::None, Route::Up] {
        let (mut domain, env, context) = change_choice_world();
        set_project_questions(&mut domain, &env, route);
        let delegate = questions_delegate(&domain, &env, &context, None);
        assert_eq!(kept_route(delegate), route);
    }
}

#[test]
fn a_named_route_wins_over_the_projects() {
    use temper_engine_domain_forge_change::Route;
    for (named, expected) in [(crate::ChangeQuestions::None, Route::None), (crate::ChangeQuestions::Up, Route::Up)] {
        let (mut domain, env, context) = change_choice_world();
        set_project_questions(&mut domain, &env, Route::Up);
        let delegate = questions_delegate(&domain, &env, &context, Some(named));
        assert_eq!(kept_route(delegate), expected);
    }
}

#[test]
fn a_later_change_of_the_questions_reaches_only_later_changes() {
    use temper_engine_domain_forge_change::Route;
    let (mut domain, env, mut context) = change_start_fixture();
    let (mut chooser, chooser_env, delegation) = change_choice_world();
    set_project_questions(&mut chooser, &chooser_env, Route::Up);
    context.requester = super::tasks::Party::Task(1);
    context.spec = questions_delegate(&chooser, &chooser_env, &delegation, None).spec;
    assert!(super::start_change(&mut domain, &env, &context));
    set_project_questions(&mut domain, &env, Route::Up);
    set_project_questions(&mut chooser, &chooser_env, Route::Up);
    assert_eq!(domain.forge.change(context.task).unwrap().questions.route, Route::Up);
    assert_eq!(kept_route(questions_delegate(&chooser, &chooser_env, &delegation, None)), Route::Up);
}

fn idle_change_procedure() -> (super::Domain, skein_lib::Env<super::Limits>, u64) {
    use super::tasks;
    let (mut domain, env, task) =
        crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: super::CHANGE_CODE });
    let (_, _, step) = domain.engine.core().tasks.procedure_due(task).expect("due procedure");
    task_event(
        &mut domain,
        &env,
        tasks::Event::Procedure {
            reads_project: false,
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(9)),
            read: None,
            task,
            step,
            decision: tasks::ProcedureDecision::new(tasks::ProcedureEnd::Wait),
        },
    );
    assert!(domain.engine.core().tasks.procedure_due(task).is_none());
    (domain, env, task)
}

const FAILED_CHANGE_WORDS: &[u8] = b"producer ran out of turns";

fn failed_change_procedure() -> (super::Domain, skein_lib::Env<super::Limits>, u64) {
    use super::{forge, forge_change, tasks};
    let (mut domain, env, task) =
        crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: super::CHANGE_CODE });
    let mut row = task_change_row(task, None);
    row.change.state = forge_change::State::Held {
        was: Box::new(forge_change::State::Producing { requested: true }),
        hold: forge_change::Hold::new(forge_change::Cause::Failed, skein_lib::Wall::EPOCH),
    };
    row.delegate_status = forge_change::Status::Failed;
    row.producer_failure = Some(FAILED_CHANGE_WORDS.into());
    domain.forge.restore_begin(env.wall);
    restore_forge(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    (domain, env, task)
}

fn apply_change_task_work(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>) {
    use super::Work;
    while let Some(work) = domain.work.pop() {
        let event = if let Work::Core(event) = work {
            event
        } else {
            let Work::Tasks(event) = work else { panic!("only the change's task decision is queued") };
            jig_domain_core::Event::Tasks(event)
        };
        drop(jig_domain_core::step(domain.engine.core_mut(), &crate::limits::environment_core(env), event));
    }
}

fn assert_failed_change_task(domain: &super::Domain, task: u64) {
    use super::tasks;
    let tasks::Phase::Held { why: tasks::Hold::EffectFailed, source: Some(source), .. } =
        &domain.engine.core().tasks.task(task).expect("live change task").phase
    else {
        panic!("the held row must hold its task: {:?}", domain.engine.core().tasks.task(task).unwrap().phase)
    };
    assert_eq!(
        source.as_ref(),
        &tasks::HoldSource::Account { connector: 0, entry: None, words: FAILED_CHANGE_WORDS.into() }
    );
}

#[test]
fn a_refused_change_step_still_holds_its_task_with_the_failure_words() {
    use super::{Work, forge_change, tasks};
    let (mut domain, env, task) = failed_change_procedure();
    super::change_decision(&mut domain, &env, task, forge_change::Decision::Hold(forge_change::Cause::Failed));
    let Some(Work::Core(jig_domain_core::Event::ProcedureStep { step, connector, .. })) = domain.work.pop() else {
        panic!("a fenced change step precedes its unfenced hold")
    };
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::super::asks::ask(
        &mut domain,
        &env,
        &mut decision,
        connector,
        jig_domain_contract::Ask::Procedures(jig_domain_contract::procedures::Ask::StepRefused {
            task,
            step,
            why: jig_domain_core::StepRefusal::Refused(tasks::Problem::new(Some(task), tasks::Refusal::Busy)),
        }),
    );
    apply_change_task_work(&mut domain, &env);
    assert_failed_change_task(&domain, task);
}

#[test]
fn a_stale_change_step_still_holds_its_task_with_the_failure_words() {
    use super::forge_change;
    let (mut domain, env, task) = failed_change_procedure();
    super::change_decision(&mut domain, &env, task, forge_change::Decision::Wait { until: None });
    super::change_decision(&mut domain, &env, task, forge_change::Decision::Hold(forge_change::Cause::Failed));
    apply_change_task_work(&mut domain, &env);
    assert_failed_change_task(&domain, task);
}

#[test]
fn a_held_change_deciding_nothing_holds_its_live_task_again() {
    let (mut domain, env, task) = failed_change_procedure();
    super::change_decision(&mut domain, &env, task, super::forge_change::Decision::None);
    apply_change_task_work(&mut domain, &env);
    assert_failed_change_task(&domain, task);
}

#[test]
fn a_persons_release_reaches_the_row_without_holding_its_task_again() {
    use super::{forge_change, tasks};
    let (mut domain, env, task) = failed_change_procedure();
    let task_env = env.with(env.limits.tasks);
    let mut out = skein_lib::Queue::uncapped();
    for event in [
        tasks::Event::Hold { task, why: tasks::Hold::EffectFailed },
        tasks::Event::Release {
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(10)),
            by: tasks::Party::Deployment { project: 1 },
            operation: tasks::Release { task, again: None, message: None },
        },
    ] {
        tasks::step(&mut domain.engine.core_mut().tasks, &task_env, event, &mut out);
    }
    assert_eq!(domain.engine.core().tasks.task(task).unwrap().due_reason, tasks::Why::Released);
    let row = domain.forge.change(task).unwrap();
    let event = super::change_step(&domain, &env, task, 1, row.repository, &row.base, false);
    let super::forge::Event::StepChange { heard, .. } = event else { panic!("change step") };
    assert!(heard.released);
    super::change_decision(&mut domain, &env, task, forge_change::Decision::None);
    apply_change_task_work(&mut domain, &env);
    assert_eq!(domain.engine.core().tasks.task(task).unwrap().phase, tasks::Phase::Active(tasks::Active::Idle));
}

#[test]
fn a_hold_decided_while_its_procedure_is_not_due_still_holds_its_task() {
    use super::{Work, forge_change, tasks};
    let (mut domain, env, task) = idle_change_procedure();
    super::change_decision(&mut domain, &env, task, forge_change::Decision::Hold(forge_change::Cause::Stalled));
    let Some(Work::Tasks(event @ tasks::Event::HoldFrom { task: held, why: tasks::Hold::Stalled, .. })) =
        domain.work.pop()
    else {
        panic!("a hold must reach an idle procedure without another fenced step")
    };
    assert_eq!(held, task);
    task_event(&mut domain, &env, event);
    let tasks::Phase::Held { why, source: Some(source), .. } =
        &domain.engine.core().tasks.task(task).expect("held task").phase
    else {
        panic!("idle procedure held")
    };
    assert_eq!(*why, tasks::Hold::Stalled);
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("stall account") };
    assert!(skein_lib::bytes::find(words, b"stall bound").is_some());
}

#[test]
fn a_hold_decided_while_its_procedure_is_not_due_keeps_its_account() {
    use super::{Work, forge_change, tasks};
    let (mut domain, env, task) = idle_change_procedure();
    super::change_decision(&mut domain, &env, task, forge_change::Decision::Hold(forge_change::Cause::Questions));
    let Some(Work::Tasks(event @ tasks::Event::HoldFrom { .. })) = domain.work.pop() else {
        panic!("an idle procedure's hold keeps its account")
    };
    task_event(&mut domain, &env, event);
    let tasks::Phase::Held { why: tasks::Hold::Procedure, source: Some(source), .. } =
        &domain.engine.core().tasks.task(task).expect("held task").phase
    else {
        panic!("accounted procedure hold")
    };
    assert_eq!(
        source.as_ref(),
        &tasks::HoldSource::Account {
            connector: 0,
            entry: None,
            words: b"its questions are spent: 4 of 4 taken".as_slice().into(),
        }
    );
}

/// temper/docs/design/protocol/wires.md, 6: the channel root names the job encoder's declaration.
#[test]
fn the_job_root_names_the_declaration_the_engine_encodes() {
    use jig_protocol_worker_wire::Carried;
    use temper_protocol_worker_wire::{
        application,
        v1::{self, Record},
        wire,
    };
    let application = application(&v1::CEILINGS).expect("temper's application");
    let mut found = None;
    for root in &application.roots {
        if root.carried == Carried::JobSpec {
            found = Some(root);
        }
    }
    let root = found.expect("job spec root");
    assert_eq!(root.wire, wire::FAMILY);
    assert_eq!(root.declaration, "JobSpec");
    for (bytes, refusal) in [
        (super::forge::jobs::close(b"repo", None, b"{}", 4096).expect("close spec"), true),
        (super::forge::jobs::merge(b"repo", None, &[3; 32], &[4; 32], 4096).expect("merge spec"), false),
    ] {
        let spec = v1::JobSpec::decode(&v1::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("JobSpec bytes");
        assert_ne!(spec.command.as_ref(), []);
        assert_ne!(spec.request.as_ref(), []);
        assert_eq!(spec.refusal, refusal);
        assert_eq!(spec.outdated, refusal);
        assert!(spec.find);
        assert!(spec.fallback.is_empty());
    }
}

/// temper/docs/design/domain/worker/jobs.md: preserve the close request whole.
#[test]
fn a_close_jobs_spec_is_its_command_and_its_request() {
    use temper_protocol_worker_wire::v1::{self, Record};
    let request = br#"{"version":1,"goal":7,"changes":[{"task":9,"landed":true}]}"#;
    let bytes = super::forge::jobs::close(b"fabrig", Some(b".temper/close"), request, 16384).expect("close spec");
    let decoded = v1::JobSpec::decode(&v1::CEILINGS, &mut skein_lib::Reader::new(&bytes)).expect("job spec");
    assert_eq!(decoded.command.as_ref(), b"fabrig/.temper/hooks/close");
    assert!(decoded.find && decoded.refusal);
    assert_eq!(decoded.fallback.as_ref(), b"fabrig/.temper/close");
    assert_eq!(decoded.request.as_ref(), request);
    let bound = u32::try_from(bytes.len()).expect("bounded spec");
    assert_eq!(super::forge::jobs::close(b"fabrig", Some(b".temper/close"), request, bound), Some(bytes));
    assert_eq!(
        super::forge::jobs::close(
            b"fabrig",
            Some(b".temper/close"),
            request,
            bound.checked_sub(1).expect("nonempty spec")
        ),
        None
    );
}

fn follow_fixture(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>, project: u32, commits: &[[u8; 32]]) {
    use super::forge;
    let mut queue = skein_lib::List::uncapped();
    for commit in commits {
        queue
            .push(forge::FollowEntry {
                before: SHA1,
                after: *commit,
                pull: None,
                commits: Box::new([]),
                more_commits: 0,
                files: None,
                more_files: 0,
                change: None,
                state: forge::FollowState::NotMade,
                word: true,
                remake: false,
            })
            .unwrap();
    }
    restore_forge(
        domain,
        env,
        forge::Stored::Follow(forge::FollowRow {
            project,
            follow: 3,
            source: skein_forge::api::Repository { forge: 1, repository: 1 },
            branch: b"main".as_slice().into(),
            heard: Some(SHA1),
            task: Some(33),
            stopped: None,
            queue: queue.into_boxed(),
            skips: Box::new([]),
            parity: None,
            questions: Box::new([]),
            pending: None,
        }),
    );
}
#[test]
fn follow_parameter_admission_names_missing_wrong_requester_and_live_commits() {
    use super::{follow_route, forge, tasks};
    let (mut domain, env, context) = change_start_fixture();
    follow_fixture(&mut domain, &env, context.project, &[SHA1, SHA256]);
    let follows = forge::ChangeFollows { follow: 3, commits: Box::new([SHA1]) };
    let missing = forge::ChangeFollows { follow: 3, commits: Box::new([[0x11; 32]]) };
    let words = follow_route::check(&domain, context.project, tasks::Party::Task(33), &missing, None, &[]).unwrap_err();
    assert!(words.starts_with(b"follows: 111111111111") && words.ends_with(b"the follow's queue lacks this commit"));
    let words = follow_route::check(&domain, context.project, tasks::Party::Task(34), &follows, None, &[]).unwrap_err();
    assert!(words.starts_with(b"follows: abcdef012345") && words.ends_with(b"the requester is not that follow's task"));
    let mut change = change_row(None);
    change.task = context.task;
    change.repository = skein_forge::api::Repository { forge: 1, repository: 1 };
    change.follows = Some(follows.clone());
    restore_forge(&mut domain, &env, forge::Stored::Change(Box::new(change)));
    let words = follow_route::check(&domain, context.project, tasks::Party::Task(33), &follows, None, &[]).unwrap_err();
    assert!(
        words.starts_with(b"follows: abcdef012345") && words.ends_with(b"another live follow change names this commit")
    );
    assert_eq!(
        follow_route::check(&domain, context.project, tasks::Party::Task(33), &follows, None, &[context.task]),
        Ok(())
    );
    assert_eq!(
        follow_route::check(&domain, context.project, tasks::Party::Task(33), &follows, Some(context.task), &[]),
        Ok(())
    );
}

#[test]
fn head_refusal_logs_full_account_and_holds_only_a_concerned_task() {
    use super::{Work, forge, tasks};
    const WORDS: &[u8] = b"record branch head on branch temper/2/c2 in org/repo: forge.client.resources is 2, and this needs 3: a limit in temper-limits. Ask temper to raise it, or raise it from Needs you. This goes on by itself once room frees or the limit rises.";
    for concerned in [false, true] {
        let (mut domain, env, task) = crate::tests::live_task(None, tasks::Executor::Agent { charter: 1 });
        let repository = adopted(1);
        domain.forge.restore_begin(env.wall);
        restore_forge(&mut domain, &env, forge::Stored::Repository(repository.clone()));
        let task = if concerned { Some(task) } else { None };
        let mut out = skein_lib::Queue::uncapped();
        out.push(forge::Request::HeadRefused {
            task,
            resource: forge::Name {
                forge: repository.provider.forge,
                repository: repository.provider.repository,
                what: forge::What::Branch(Box::new([
                    b"temper".as_slice().into(),
                    b"2".as_slice().into(),
                    b"c2".as_slice().into(),
                ])),
            },
            configured: 2,
            needed: 3,
        });
        let mut decision = super::Decision::decide(&mut domain.journal, &env.limits.journal);
        super::outputs(&mut domain, &env, &mut decision, &mut out);
        let Some(crate::Fact::HeadRefused { task: fact_task, words }) = domain.pop_fact() else {
            panic!("one head refusal fact");
        };
        assert_eq!(fact_task, task);
        assert_eq!(words.as_ref(), WORDS);
        assert!(domain.pop_fact().is_none());
        if let Some(task) = task {
            let Some(Work::ForgeLimitHold { row, words }) = domain.work.pop() else {
                panic!("the concerned task is held");
            };
            super::limit_holds::apply_hold(&mut domain, &env, &mut decision, row, words);
            let tasks::Phase::Held { why, source: Some(source), .. } =
                &domain.engine.core().tasks.task(task).unwrap().phase
            else {
                panic!("retained resource hold");
            };
            assert_eq!(*why, tasks::Hold::ResourceUnavailable);
            let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("hold account") };
            assert!(words.starts_with(WORDS));
        }
        assert!(domain.work.is_empty(), "no unrelated work is held");
    }
}

#[test]
fn a_configured_resource_limit_reaches_the_held_tasks_words() {
    use super::{Work, forge, tasks};
    let (mut domain, env, task) = crate::tests::live_task(None, tasks::Executor::Agent { charter: 1 });
    let mut out = skein_lib::Queue::uncapped();
    out.push(forge::Request::ResourceRefused {
        task,
        operation: b"subscribe",
        resource: None,
        cause: forge::RefusalCause::Limit { member: "forge.paths_per_subscription", configured: 4, needed: 5 },
    });
    let mut decision = super::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::outputs(&mut domain, &env, &mut decision, &mut out);
    let Some(Work::ForgeLimitHold { row, words }) = domain.work.pop() else {
        panic!("resource refusal routes a hold");
    };
    super::limit_holds::apply_hold(&mut domain, &env, &mut decision, row, words);
    let tasks::Phase::Held { why, source: Some(source), .. } = &domain.engine.core().tasks.task(task).unwrap().phase
    else {
        panic!("refused task is held");
    };
    assert_eq!(*why, tasks::Hold::ResourceUnavailable);
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("retained limit cause") };
    for text in [
        b"subscribe".as_slice(),
        b"forge.paths_per_subscription is 4, and this needs 5",
        b"This goes on by itself once room frees or the limit rises.",
    ] {
        let mut found = false;
        for part in words.windows(text.len()) {
            found |= part == text;
        }
        assert!(found, "missing {text:?} in {words:?}");
    }
}

#[test]
fn a_change_rerun_is_kind_17_on_its_own_branch() {
    use super::{forge, forge_change, tasks};
    let (mut domain, mut env) = goal_holds_branch();
    env.limits.authority.segments = 8;
    make_goal_task(&mut domain, &env, GOAL_CHANGE, tasks::Party::Task(GOAL_TASK), Box::new([]));
    let mut row = change_row(Some(1));
    row.task = GOAL_CHANGE;
    row.change.task = GOAL_CHANGE;
    row.repository = adopted(1).provider;
    let expected = super::resource_name(
        &adopted(1),
        &super::branch_what(&row.branch, env.limits.forge.name_bytes).unwrap(),
        env.limits.authority.segments,
    )
    .unwrap();
    let evidence =
        forge::ChangeEvidence { ci: forge_change::Status::Failed, ..passed_evidence(row.change.clone(), SHA1, SHA1) };
    restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    let description = super::describe_change_effect(
        &domain,
        &env,
        GOAL_CHANGE,
        forge_change::Effect::Rerun { head: SHA1 },
        &evidence,
    )
    .expect("re-run description");
    assert_eq!(description.effect.kind, 17);
    assert_eq!(description.effect.name, expected);
    assert_eq!(description.effect.state, SHA1);
    let declaration = crate::translate::forge_declaration(description.connector);
    let mut recovery = None;
    for kind in &declaration.effects {
        if kind.kind == description.effect.kind {
            recovery = Some(kind.recovery);
        }
    }
    assert_eq!(recovery, Some(jig_domain_contract::effects::Recovery::Unrecoverable));
}

#[expect(clippy::too_many_lines, reason = "one restored train fixture couples task readiness and joined repositories")]
fn train_root(joined: bool) -> (super::Domain, skein_lib::Env<super::Limits>) {
    use super::{forge, forge_change as c, tasks};
    use skein_lib::Queue;
    let mut limits = crate::tests::resume_fixture::limits();
    // Two ready changes and their retained landing tasks need admission room.
    limits.tasks.tasks = 16;
    limits.tasks.project_tasks = 16;
    limits.people.inbox_tasks = 32;
    let (mut domain, mut env) = procedure_root_with_limits(limits);
    fresh_procedure_forge(&mut domain, &env);
    env.limits.tasks.batch = 2;
    env.limits.tasks.batch_bytes = 16_384;
    let (fixture, _, task) = crate::tests::live_task(None, tasks::Executor::Procedure { connector: 0, code: 2 });
    let template = fixture.engine.core().tasks.task(task).unwrap().clone();
    let mut batch = skein_lib::List::with_capacity(2);
    for number in [8, 7] {
        batch
            .push(tasks::New {
                number,
                project: 1,
                label: 0,
                flags: tasks::Flags::PLAIN,
                executor: tasks::Executor::Procedure { connector: 0, code: 2 },
                llm: None,
                spec: tasks::Spec {
                    parameters: Box::new([
                        tasks::Parameter::Resource {
                            name: 1,
                            connector: 0,
                            resource: forge::tag(1, if joined && number == 8 { 2 } else { 1 }),
                        },
                        tasks::Parameter::Bytes { name: 2, value: b"main".as_slice().into() },
                    ]),
                    ..template.spec.clone()
                },
                contract: template.contract.clone(),
                deadline: tasks::Deadline::Named,
                authority: template.authority.clone(),
                numbers: template.numbers,
                funder: tasks::Funder::Period { project: 1, period: 1 },
                dependencies: if joined && number == 7 { Box::new([8]) } else { Box::new([]) },
                holdings: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
                recurring: None,
                tracked: None,
            })
            .expect("two changes");
        let mut row = change_row(Some(number));
        row.task = number;
        row.change.task = number;
        row.repository = adopted(1).provider;
        if joined && number == 8 {
            row.repository.repository = 2;
        }
        row.change.way = c::Way::Turn;
        row.change.state = c::State::Ready { head: SHA1, ready_since: env.wall };
        row.change.ready_since = Some(env.wall);
        restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    }
    task_event(
        &mut domain,
        &env,
        tasks::Event::OpenPeriod { reply_to: super::super::internal(u64::MAX), project: 1, period: 1, budget: 1000 },
    );
    let mut made = Queue::uncapped();
    tasks::step(
        &mut domain.engine.core_mut().tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Make {
            reads_project: false,
            maker: None,
            places: Box::new([]),
            reply_to: super::super::internal(u64::MAX),
            creator: tasks::Party::Deployment { project: 1 },
            batch: batch.into_boxed(),
        },
        &mut made,
    );
    assert!(domain.engine.core().tasks.task(7).is_some(), "{made:?}");
    if joined {
        let mut repo = adopted(1);
        repo.project = 1;
        repo.provider.repository = 2;
        repo.name = Box::from(b"other".as_slice());
        repo.home = false;
        restore_forge_with_engine_waits(&mut domain, &env, forge::Stored::Repository(repo));
    }
    // These restored ready rows have already completed their procedure steps.
    for task in [7, 8] {
        let (_, _, step) = domain.engine.core().tasks.procedure_due(task).unwrap();
        task_event(
            &mut domain,
            &env,
            tasks::Event::Procedure {
                reads_project: false,
                reply_to: super::super::internal(u64::MAX),
                task,
                step,
                read: None,
                decision: tasks::ProcedureDecision::new(tasks::ProcedureEnd::Wait),
            },
        );
    }
    turn_project(&mut domain);
    (domain, env)
}
fn turn_project(domain: &mut super::Domain) {
    assert!(domain.forge.project_landing(
        1,
        super::forge_change::Way::Turn,
        Box::new([]),
        temper_engine_domain_forge_landing::Project::DEFAULT,
        super::forge::AfterLanding {
            wait: temper_engine_domain_forge_change::Wait::Cleared,
            verdict: temper_engine_domain_forge_tiers::Settings {
                retries: 2,
                suspects: temper_engine_domain_forge_tiers::Suspects::Bisect,
            },
            bound: skein_lib::Duration::from_secs(7200),
        }
    ));
    domain.opened = true;
}
fn judge_trains(domain: &mut super::Domain, env: &skein_lib::Env<super::Limits>) {
    let mut decision = super::Decision::decide(&mut domain.journal, &env.limits.journal);
    crate::route::route_into(domain, env, &mut decision);
    crate::decision::accept_routed(
        &mut domain.journal,
        &mut domain.engine.core_mut().counters,
        &env.limits.journal,
        decision,
        true,
    )
    .unwrap();
}
#[test]
fn the_root_forms_a_fresh_repositorys_train_while_another_repository_is_unread() {
    use super::{Work, forge, landing_route};
    use skein_forge::api::{Answer, Error, Op, Read};
    use skein_lib::{Duration, Queue};
    let (mut domain, env) = train_root(false);
    let changes = [domain.forge.change(7).unwrap().clone(), domain.forge.change(8).unwrap().clone()];
    let config = crate::tests::resume_fixture::config(7);
    domain.forge =
        forge::Domain::new(&env.limits.forge, 4, config.forge, &config.identities, &config.forges, false).unwrap();
    domain.forge.restore_begin(env.wall);
    let forge_env = super::super::environment_forge(&env);
    let mut first_repository = adopted(1);
    first_repository.project = 1;
    let first = first_repository.provider;
    restore_in_forge_env(&mut domain, &forge_env, forge::Stored::Repository(first_repository));
    for change in changes {
        restore_in_forge_env(&mut domain, &forge_env, forge::Stored::Change(Box::new(change)));
    }
    let other = skein_forge::api::Repository { repository: 2, ..first };
    let mut repository = adopted(1);
    repository.project = 1;
    repository.provider = other;
    repository.home = false;
    repository.name = b"other".as_slice().into();
    restore_in_forge_env(&mut domain, &forge_env, forge::Stored::Repository(repository));
    let mut change = domain.forge.change(8).unwrap().clone();
    change.repository = other;
    restore_in_forge_env(&mut domain, &forge_env, forge::Stored::Change(Box::new(change)));
    for (repository, task) in [(first, 7), (other, 8)] {
        let name = forge::Name {
            forge: repository.forge,
            repository: repository.repository,
            what: forge::What::Branch(Box::new([b"marker".as_slice().into()])),
        };
        restore_in_forge_env(&mut domain, &forge_env, forge::Stored::Names { task, resources: Box::new([name]) });
    }
    for event in [forge::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic }, forge::Event::CatchUp]
    {
        forge::step(&mut domain.forge, &forge_env, event, &mut Queue::uncapped());
    }
    turn_project(&mut domain);
    assert!(!landing_route::form_landings(&mut domain, &env));
    let mut calls = Queue::uncapped();
    for _ in 0..forge_env.limits.client.calls {
        forge::resume(&mut domain.forge, &forge_env, &mut calls);
    }
    let mut responses = Queue::uncapped();
    while let Some(request) = calls.pop() {
        if let forge::Request::Call { call, repository, op, .. } = request {
            let Op::Read(Read::Branch { .. }) = op else { panic!("fresh branch read: {op:?}") };
            let result = if repository == other {
                Ok(Answer::Commit(SHA1))
            } else {
                Err(Error::RateLimited { after: Duration::from_secs(1) })
            };
            forge::step(
                &mut domain.forge,
                &forge_env,
                forge::Event::Client(skein_forge_client::Event::Answered {
                    free: 0,
                    rate: None,
                    call,
                    cost: 1,
                    result,
                }),
                &mut responses,
            );
        }
    }
    assert!(!domain.forge.fresh_reads_in());
    assert!(domain.forge.repository_fresh(other));
    assert!(landing_route::form_landings(&mut domain, &env));
    let mut members = None;
    while let Some(work) = domain.work.pop() {
        if let Work::LandingAdmission { row, .. } = work {
            members = Some(row.landing.members);
        }
    }
    let members = members.expect("the fresh train is admitted");
    assert_eq!(members.len(), 1);
    assert_eq!(members.first().unwrap().change, 8);
    assert_eq!(domain.forge.landing_of(7), None);
}

#[test]
fn the_root_forms_a_train_from_a_restored_change_with_a_stale_landing_link() {
    use super::{forge, landing};
    use skein_lib::Queue;
    for phase in [None, Some(landing::Phase::Landed), Some(landing::Phase::Cancelled)] {
        let (mut domain, env) = train_root(false);
        let mut row = domain.forge.change(7).unwrap().clone();
        row.previous = Some(forge::PreviousLink { landing: Some(99), ask: None });
        let forge_env = super::super::environment_forge(&env);
        let event = forge::Event::Restore { record: forge::Stored::Change(Box::new(row)) };
        forge::step(&mut domain.forge, &forge_env, event, &mut Queue::uncapped());
        if let Some(phase) = phase {
            let (_, _, mut row) = reserved_turn_landing();
            row.landing.task = 99;
            row.landing.phase = phase;
            let event = forge::Event::Restore { record: forge::Stored::Landing(row) };
            forge::step(&mut domain.forge, &forge_env, event, &mut Queue::uncapped());
        }
        domain.forge.fit_restored(&env.limits.forge).unwrap();
        judge_trains(&mut domain, &env);
        let formed = domain.forge.landing_of(7).unwrap();
        assert_ne!(formed, 99, "the stale member is taken again, before the later task");
        assert_eq!(domain.forge.landing(formed).unwrap().landing.members[0].change, 7);
    }
}

#[test]
fn the_root_forms_from_restored_readiness_and_again_when_the_train_ends() {
    use super::{Work, forge, tasks};
    let (mut domain, env) = train_root(false);
    judge_trains(&mut domain, &env);
    let first = domain.forge.landing_of(7).unwrap();
    assert_eq!(domain.forge.landing_of(8), None);
    assert_eq!(domain.forge.landing(first).unwrap().landing.branch.as_ref(), b"main");
    // Cancelling the landing releases its ready member and frees the train.
    domain.work.push(Work::Forge(forge::Event::SettleEffects {
        task: first,
        root: first,
        ending: forge::ReleaseEnding::Cancelled,
    }));
    // Keep the member out of forming while the terminal is settled by jig.
    domain.work.push(Work::Tasks(tasks::Event::HoldFrom {
        task: 7,
        why: tasks::Hold::Stopped,
        source: Box::new(tasks::HoldSource::Account {
            connector: 0,
            entry: None,
            words: Box::from(b"cancelled".as_slice()),
        }),
    }));
    judge_trains(&mut domain, &env);
    let second = domain.forge.landing_of(8).unwrap();
    assert_ne!(first, second);
    assert_eq!(domain.forge.landing(first).unwrap().landing.phase, super::landing::Phase::Cancelled);
    domain.work.push(Work::Forge(forge::Event::SettleEffects {
        task: second,
        root: second,
        ending: forge::ReleaseEnding::Cancelled,
    }));
    judge_trains(&mut domain, &env);
    let replacement = domain.forge.landing_of(8).unwrap();
    assert_ne!(replacement, second, "cancellation frees the train with its member still ready");
    assert_eq!(domain.forge.landing(second).unwrap().landing.phase, super::landing::Phase::Cancelled);

    let (mut domain, env) = train_root(false);
    domain.work.push(Work::Forge(forge::Event::Priority { task: 8, priority: -1_i32 }));
    judge_trains(&mut domain, &env);
    assert!(domain.forge.landing_of(8).is_some(), "priority is judged before forming");
    assert_eq!(domain.forge.landing_of(7), None);
}
#[test]
fn either_train_holds_the_units_first_change_in_dependency_order() {
    use super::tasks;
    for first in [7, 8] {
        let (mut domain, env) = train_root(true);
        super::start_landing(&mut domain, &env, first);
        judge_trains(&mut domain, &env);
        let tasks::Phase::Held { .. } = domain.engine.core().tasks.task(8).unwrap().phase else {
            panic!("train C{first} must hold upstream C8");
        };
        assert_eq!(
            domain.engine.core().tasks.task(7).unwrap().phase,
            tasks::Phase::Active(tasks::Active::Idle),
            "the other member is not held too"
        );
        let revision = domain.engine.core().tasks.task(8).unwrap().revisions.hold;
        for other in [7, 8] {
            super::start_landing(&mut domain, &env, other);
            judge_trains(&mut domain, &env);
        }
        assert_eq!(domain.engine.core().tasks.task(8).unwrap().revisions.hold, revision);
        assert_eq!(domain.engine.core().tasks.task(7).unwrap().phase, tasks::Phase::Active(tasks::Active::Idle));
        domain.work.push(super::Work::Tasks(tasks::Event::Release {
            reply_to: super::super::internal(u64::MAX),
            by: tasks::Party::Deployment { project: 1 },
            operation: tasks::Release { task: 8, again: None, message: None },
        }));
        judge_trains(&mut domain, &env);
        let tasks::Phase::Held { .. } = domain.engine.core().tasks.task(8).unwrap().phase else {
            panic!("released unit must be held again");
        };
        assert!(domain.engine.core().tasks.task(8).unwrap().revisions.hold > revision);
        assert_eq!(domain.engine.core().tasks.task(7).unwrap().phase, tasks::Phase::Active(tasks::Active::Idle));
    }
}

#[test]
fn a_unit_holds_its_first_dependency_and_a_release_refuses_it_again() {
    use super::{Work, tasks};
    let (mut domain, env) = train_root(true);
    judge_trains(&mut domain, &env);
    for task in [7, 8] {
        assert_eq!(domain.forge.landing_of(task), None);
    }
    let tasks::Phase::Held { why, source: Some(source), .. } = &domain.engine.core().tasks.task(8).unwrap().phase
    else {
        panic!("unit hold")
    };
    assert_eq!(*why, tasks::Hold::Procedure);
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("hold account") };
    assert!(words.starts_with(b"unit of 2 changes: C8, C7: a unit across repositories"));
    assert!(skein_lib::bytes::find(words, b"merge-queue.md, 13)").is_some());
    let revision = domain.engine.core().tasks.task(8).unwrap().revisions.hold;
    domain.work.push(Work::Tasks(tasks::Event::Release {
        reply_to: super::super::internal(u64::MAX),
        by: tasks::Party::Deployment { project: 1 },
        operation: tasks::Release { task: 8, again: None, message: None },
    }));
    judge_trains(&mut domain, &env);
    let tasks::Phase::Held { .. } = domain.engine.core().tasks.task(8).unwrap().phase else { panic!("held again") };
    assert!(domain.engine.core().tasks.task(8).unwrap().revisions.hold > revision, "released, then held anew");
    assert_eq!(domain.forge.landing_of(7), None);
}

#[test]
fn every_change_hold_has_words_even_without_an_account() {
    use super::{StepEnd, forge, forge_change as change, tasks};
    let (mut domain, env) = two_forges();
    let authority = change::AuthorityHold {
        recovery: change::AuthorityRecovery { task: 7, spec: 1, hold: 1, tasks: 3 },
        lack: change::AuthorityLack::Tasks,
        had: 3,
        released: false,
    };
    for why in [
        change::Cause::Stalled,
        change::Cause::Runner,
        change::Cause::Repairs,
        change::Cause::Resolutions,
        change::Cause::Updates,
        change::Cause::BranchMoved,
        change::Cause::BranchMissing,
        change::Cause::PullClosed,
        change::Cause::Retargeted,
        change::Cause::Failed,
        change::Cause::Rejected,
        change::Cause::Authority(authority),
        change::Cause::Questions,
    ] {
        let mut row = change_row(Some(7));
        row.change.state = change::State::Held {
            was: Box::new(change::State::Repairing { requested: true, why: change::Repair::Gate(1) }),
            hold: change::Hold::new(why, skein_lib::Wall::EPOCH),
        };
        row.producer_failure = Some(Box::new([]));
        row.effect = change::EffectResult::Failed;
        row.effect_account = Some(forge::EffectAccount { entry: 1, words: Box::new([]) });
        restore_forge(&mut domain, &env, forge::Stored::Change(Box::new(row)));
        let step = super::hold_change_decision(&domain, &env, 7, why, None);
        let StepEnd::HoldFrom { source, .. } = step.end else { panic!("wordless hold: {why:?}") };
        let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("hold account: {why:?}") };
        assert!(!words.is_empty(), "empty hold: {why:?}");
    }
}

#[test]
fn a_runner_hold_uses_its_own_account() {
    use super::{StepEnd, forge, forge_change as change, tasks};
    const TASK: u64 = 7;
    const HEAD: [u8; 32] = [3; 32];
    const WORDS: &[u8] =
        b"CI's runner failed at 030303030303: job 1 of check build: set-up failed; the change wasn't judged.";
    let (mut domain, env) = two_forges();
    let mut row = change_row(Some(TASK));
    row.change.state = change::State::Held {
        was: Box::new(change::State::Checking { head: HEAD }),
        hold: change::Hold::new(change::Cause::Runner, skein_lib::Wall::EPOCH),
    };
    row.runner_ci = Some(forge::RunnerCi::Held { words: WORDS.into() });
    restore_forge(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    let step = super::hold_change_decision(&domain, &env, TASK, change::Cause::Runner, None);
    let StepEnd::HoldFrom { why: tasks::Hold::Procedure, source } = step.end else { panic!("runner hold account") };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("hold account") };
    assert_eq!(words.as_ref(), WORDS);
}

#[test]
fn a_refused_change_delegate_holds_with_words_and_recovery() {
    use super::{Work, forge, forge_change as change, tasks};
    let (mut domain, env, task) = idle_change_procedure();
    let mut row = task_change_row(task, None);
    row.delegate = Some((9, change::Delegate::Produce));
    domain.forge.restore_begin(env.wall);
    restore_forge(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    let mut decision = crate::Decision::decide(&mut domain.journal, &env.limits.journal);
    super::super::asks::ask(
        &mut domain,
        &env,
        &mut decision,
        0,
        jig_domain_contract::Ask::Resources(jig_domain_contract::resources::Ask::DelegateRefused { task: Some(task) }),
    );
    let Some(Work::Forge(forge::Event::DelegateRefused { .. })) = domain.work.pop() else { panic!("forge refusal") };
    let Some(Work::Tasks(event @ tasks::Event::HoldFrom { .. })) = domain.work.pop() else {
        panic!("delegate refusal must have words")
    };
    task_event(&mut domain, &env, event);
    let tasks::Phase::Held { why: tasks::Hold::Procedure, source: Some(source), .. } =
        &domain.engine.core().tasks.task(task).unwrap().phase
    else {
        panic!("refusal account")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("refusal words") };
    assert!(skein_lib::bytes::find(words, b"delegate admission was refused").is_some());
    assert!(skein_lib::bytes::find(words, b"task release").is_some());
}

#[test]
fn a_prepare_intent_and_an_earlier_suffix_are_not_an_admitted_job() {
    use super::{forge, landing_route};
    use temper_engine_domain_forge_landing::Preparation;
    let (mut domain, env, mut row) = reserved_turn_landing();
    let task = row.landing.task;
    let context = domain.engine.core().tasks.run_context(task).expect("landing context");
    row.landing.preparation = 1;
    row.landing.members[0].asked = Some(1);
    let mut second = row.landing.members[0].clone();
    second.change = 8;
    row.landing.members = Box::new([row.landing.members[0].clone(), second]);
    let check = |domain: &mut super::Domain, row: &forge::LandingRow, expected| {
        restore_forge_with_engine_waits(domain, &env, forge::Stored::Landing(row.clone()));
        assert_eq!(landing_route::preparation(domain, &context), expected);
    };
    check(&mut domain, &row, Preparation::Absent);
    row.job = Some(forge::history::JobRecord {
        step: forge::history::JobStep {
            task: 100,
            attempt: 1,
            job: forge::history::Job::Prepare,
            outcome: Some(forge::history::JobOutcome::Done),
            head: None,
        },
        round: 1,
        members: Box::new([0, 1]),
        head: None,
        written: true,
    });
    assert!(domain.engine.core().tasks.task(100).is_none(), "ended jobs need no live delegate");
    check(&mut domain, &row, Preparation::Absent);
    row.landing.round.failure = br#"{"combined":[{"member":2,"by":"merge"}]}"#.as_slice().into();
    check(&mut domain, &row, Preparation::Idle);
    row.landing.members[0].candidate = Some(SHA1);
    check(&mut domain, &row, Preparation::Absent);
    row.job.as_mut().expect("admitted job").members = Box::new([1]);
    check(&mut domain, &row, Preparation::Idle);
    row.landing.preparation = 2;
    check(&mut domain, &row, Preparation::Absent);
}

fn unprepared_round() -> (super::Domain, skein_lib::Env<super::Limits>, super::forge::LandingRow) {
    use super::forge::history::{Job, JobOutcome, JobRecord, JobStep};
    let (domain, env, mut row) = reserved_turn_landing();
    row.landing.preparation = 1;
    row.landing.members[0].asked = Some(1);
    let mut second = row.landing.members[0].clone();
    second.change = 8;
    row.landing.members = Box::new([row.landing.members[0].clone(), second]);
    row.job = Some(JobRecord {
        step: JobStep { task: 100, attempt: 1, job: Job::Prepare, outcome: Some(JobOutcome::Done), head: None },
        round: 1,
        members: Box::new([0, 1]),
        head: None,
        written: true,
    });
    (domain, env, row)
}

fn read_preparation(
    domain: &mut super::Domain,
    env: &skein_lib::Env<super::Limits>,
    row: &super::forge::LandingRow,
    context: &super::tasks::RunContext,
) -> temper_engine_domain_forge_landing::Preparation {
    use super::forge;
    restore_forge_with_engine_waits(domain, env, forge::Stored::Landing(row.clone()));
    super::landing_route::preparation(domain, context)
}

#[test]
fn a_prepare_that_ended_without_an_answer_is_absent_even_with_a_kept_report() {
    use super::forge::history::{JobFailure, JobOutcome};
    use temper_engine_domain_forge_landing::Preparation;
    let (mut domain, env, mut row) = unprepared_round();
    let context = domain.engine.core().tasks.run_context(row.landing.task).expect("landing context");
    assert!(domain.engine.core().tasks.task(100).is_none());
    for outcome in
        [JobOutcome::Lost, JobOutcome::Cancelled, JobOutcome::Timeout, JobOutcome::Failed { class: JobFailure::Run }]
    {
        row.job.as_mut().unwrap().step.outcome = Some(outcome);
        for report in [b"".as_slice(), br#"{"combined":[{"member":1,"by":"merge"}]}"#] {
            row.landing.round.failure = report.into();
            assert_eq!(read_preparation(&mut domain, &env, &row, &context), Preparation::Absent, "{outcome:?}");
        }
    }
}

#[test]
fn an_ended_prepares_record_needs_an_answer_kept_for_its_suffix() {
    use super::forge::history::JobOutcome;
    use temper_engine_domain_forge_landing::Preparation;
    let (mut domain, env, mut row) = unprepared_round();
    let context = domain.engine.core().tasks.run_context(row.landing.task).expect("landing context");
    for outcome in [Some(JobOutcome::Done), Some(JobOutcome::Conflict), None] {
        row.job.as_mut().unwrap().step.outcome = outcome;
        for (report, expected) in [
            (b"".as_slice(), Preparation::Absent),
            (b"job failed".as_slice(), Preparation::Absent),
            (br#"{"combined":[{"member":1,"by":"merge"}]}"#.as_slice(), Preparation::Idle),
            (br#"{"conflicts":[{"member":2,"with":"member","files":["code"]}]}"#.as_slice(), Preparation::Idle),
        ] {
            row.landing.round.failure = report.into();
            assert_eq!(read_preparation(&mut domain, &env, &row, &context), expected, "{outcome:?}");
        }
    }
    row.landing.members[0].candidate = Some(SHA1);
    row.job.as_mut().unwrap().members = Box::new([1]);
    row.landing.round.failure = br#"{"combined":[{"member":1,"by":"merge"}]}"#.as_slice().into();
    assert_eq!(read_preparation(&mut domain, &env, &row, &context), Preparation::Absent);
}

#[test]
fn each_refusal_of_a_kept_prepare_reads_absent() {
    use super::forge::{LandingRow, history::Job};
    use temper_engine_domain_forge_landing::{Leave, Preparation};
    type Case = (&'static str, fn(&mut LandingRow), Preparation);
    let (mut domain, env, mut row) = unprepared_round();
    let context = domain.engine.core().tasks.run_context(row.landing.task).expect("landing context");
    row.landing.round.failure = br#"{"combined":[{"member":1,"by":"merge"}]}"#.as_slice().into();
    let cases: [Case; 6] = [
        ("kept", |_| {}, Preparation::Idle),
        ("an earlier round", |row| row.job.as_mut().unwrap().round = 0, Preparation::Absent),
        ("a merge job", |row| row.job.as_mut().unwrap().step.job = Job::Merge, Preparation::Absent),
        ("a member since left", |row| row.landing.members[1].left = Some(Leave::Cancelled), Preparation::Absent),
        ("a member since landed", |row| row.landing.members[0].landed = Some(SHA1), Preparation::Absent),
        (
            "a report for no job member",
            |row| {
                row.landing.members[1].candidate = Some(SHA1);
                row.job.as_mut().unwrap().members = Box::new([0]);
                row.landing.round.failure = br#"{"combined":[{"member":2,"by":"merge"}]}"#.as_slice().into();
            },
            Preparation::Absent,
        ),
    ];
    for (case, change, expected) in cases {
        let mut row = row.clone();
        change(&mut row);
        assert_eq!(read_preparation(&mut domain, &env, &row, &context), expected, "{case}");
    }
}

fn prepare_child(
    domain: &mut super::Domain,
    env: &skein_lib::Env<super::Limits>,
    task: u64,
    phase: super::tasks::Phase,
) {
    use super::tasks;
    let mut parent = domain.engine.core().tasks.task(task).unwrap().clone();
    parent.delegates = Box::new([100]);
    let mut child = parent.clone();
    child.number = 100;
    child.requester = tasks::Party::Task(task);
    child.delegates = Box::new([]);
    child.executor = tasks::Executor::Job { kind: super::forge::PREPARE_KIND };
    child.spec.words = Box::new([]);
    child.spec.parameters = Box::new([]);
    child.spec.inputs = Box::new([]);
    child.spec.asks = false;
    child.spec.job =
        Some(Box::new(tasks::JobSpec { bytes: Box::new([]), deadline: skein_lib::Duration::from_secs(10) }));
    child.contract = tasks::Contract::Job { bytes: env.limits.tasks.result_bytes };
    child.phase = phase.clone();
    reset_tasks_for_restore(domain, env, 101);
    let mut out = skein_lib::Queue::uncapped();
    for task in [parent, child] {
        tasks::step(
            &mut domain.engine.core_mut().tasks,
            &env.with(env.limits.tasks),
            tasks::Event::Restore { record: tasks::Stored::Live(Box::new(task)) },
            &mut out,
        );
    }
    assert!(domain.engine.core().tasks.task(100).is_some(), "{phase:?}: {out:?}");
}

#[test]
fn a_prepare_inbox_answer_precedes_a_live_child_without_a_record() {
    use super::tasks;
    use temper_engine_domain_forge_landing::Preparation;
    let (mut domain, env, mut row) = unprepared_round();
    row.job = None;
    let mut context = domain.engine.core().tasks.run_context(row.landing.task).expect("landing context");
    context.inbox = Box::new([tasks::Word {
        number: 1,
        from: tasks::Party::Task(100),
        kind: tasks::MessageKind::Result(tasks::ResultKind::Job),
        words: br#"{"combined":[{"member":1,"by":"merge"}]}"#.as_slice().into(),
        at: env.wall,
        hits: 1,
        eligible: true,
    }]);
    assert_eq!(read_preparation(&mut domain, &env, &row, &context), Preparation::Idle);
    prepare_child(&mut domain, &env, row.landing.task, tasks::Phase::Active(tasks::Active::Due));
    assert_eq!(read_preparation(&mut domain, &env, &row, &context), Preparation::Idle);
    context.inbox[0].words = b"unreadable report".as_slice().into();
    assert_eq!(read_preparation(&mut domain, &env, &row, &context), Preparation::Failed);
    context.inbox[0].words = br#"{"conflicts":[{"member":1,"with":"branch","files":["code"]}]}"#.as_slice().into();
    row.landing.round.branch = Box::new([]);
    assert_eq!(read_preparation(&mut domain, &env, &row, &context), Preparation::Conflict);
    let revision = domain.engine.core().tasks.task(100).unwrap().revisions.hold;
    prepare_child(
        &mut domain,
        &env,
        row.landing.task,
        tasks::Phase::Held {
            was: tasks::Was::Active(tasks::Active::Due),
            why: tasks::Hold::Failures(tasks::Class::Run),
            source: None,
        },
    );
    context.inbox[0].kind = tasks::MessageKind::Escalation { task: 100, revision };
    assert_eq!(read_preparation(&mut domain, &env, &row, &context), Preparation::Failed);
}

#[test]
fn a_live_prepare_counts_whatever_round_asked_it() {
    use super::tasks;
    use temper_engine_domain_forge_landing::Preparation;
    for (phase, expected) in [
        (tasks::Phase::Active(tasks::Active::Due), Preparation::Running),
        (
            tasks::Phase::Active(tasks::Active::BackingOff { until: skein_lib::Wall::from_nanos(100) }),
            Preparation::Running,
        ),
        (tasks::Phase::Active(tasks::Active::Running { attempt: 1 }), Preparation::Running),
        (tasks::Phase::Waiting, Preparation::Running),
        (
            tasks::Phase::Closing(tasks::Closing {
                stage: tasks::Stage::Effects,
                ending: tasks::Ending::Failed { reason: Box::new([]) },
            }),
            Preparation::Running,
        ),
        (
            tasks::Phase::Held {
                was: tasks::Was::Active(tasks::Active::Due),
                why: tasks::Hold::Failures(tasks::Class::Run),
                source: None,
            },
            Preparation::Failed,
        ),
    ] {
        let (mut domain, env, mut row) = unprepared_round();
        row.job.as_mut().unwrap().round = 0;
        prepare_child(&mut domain, &env, row.landing.task, phase.clone());
        let context = domain.engine.core().tasks.run_context(row.landing.task).expect("landing context");
        assert_eq!(read_preparation(&mut domain, &env, &row, &context), expected, "{phase:?}");
    }
}

fn report_tests() -> super::forge::ReceiptReport {
    super::forge::ReceiptReport::Tests(Box::new([
        super::forge::ReceiptTest { key: b"test skein-lib::suite fuzzy::a".as_slice().into(), fuzzy: true },
        super::forge::ReceiptTest { key: b"test skein-lib::suite b".as_slice().into(), fuzzy: false },
    ]))
}

/// questions.md, 1, What every delegate is told: the step's retained amendment is visible before admission.
#[test]
fn a_delegate_made_at_the_amendment_step_carries_its_reason() {
    use super::{forge, forge_change as change, tasks};
    let (mut domain, mut env, task) = idle_change_procedure();
    env.limits.tasks.parameters = 5;
    env.limits.tasks.spec_bytes = 4096;
    let record = domain.engine.core().tasks.task(task).unwrap();
    let mut spec = record.spec.clone();
    spec.words = b"Drop it".as_slice().into();
    let requester = tasks::Party::Person(1);
    let mut out = skein_lib::Queue::uncapped();
    tasks::step(
        &mut domain.engine.core_mut().tasks,
        &env.with(env.limits.tasks),
        tasks::Event::Amend {
            reads_project: false,
            places: Box::new([]),
            reply_to: skein_lib::ReplyTo::new(skein_lib::Token::new(1)),
            by: requester,
            operation: tasks::Amend {
                task,
                funder: None,
                message: 1,
                stop_run: false,
                amendment: Box::new(tasks::Amendment {
                    tracked: None,
                    addressed: None,
                    instructions: None,
                    release: None,
                    spec: Some(spec),
                    wake: None,
                    dependencies: None,
                    authority: None,
                    llm: None,
                    reason: b"Drop the file".as_slice().into(),
                }),
            },
        },
        &mut out,
    );
    assert_eq!(domain.engine.core().tasks.task(task).unwrap().revisions.spec, 1, "{out:?}");
    let (connector, code, step) = domain.engine.core().tasks.procedure_due(task).unwrap();
    let mut row = task_change_row(task, Some(7));
    row.change.last_head = Some(SHA1);
    domain.forge.restore_begin(env.wall);
    restore_forge(&mut domain, &env, forge::Stored::Change(Box::new(row)));
    super::delegate_change_step(
        &mut domain,
        &env,
        task,
        step,
        connector,
        code,
        &(change::Delegate::Repair(change::Repair::Ci), None, None),
    );
    let Some(super::Work::Core(jig_domain_core::Event::ProcedureStep { decision, .. })) = domain.work.pop() else {
        panic!("delegate step")
    };
    let delegate = decision.batch.first().expect("repair described");
    let expected = b"\nRulings, newest first:\nAmended to spec revision 1: Drop the file\nIts current spec, at revision 1:\nDrop it";
    assert!(delegate.spec.words.ends_with(expected));
    assert!(
        domain.forge.change(task).unwrap().questions.amended.is_empty(),
        "nothing commits before the step is admitted"
    );
    let pending = domain.forge_delegating.get(&(task, step)).unwrap();
    assert_eq!(
        pending.relayed.as_ref().unwrap().questions.amended.as_ref(),
        &[change::Amended { revision: 1, reason: b"Drop the file".as_slice().into() }]
    );
}
