//! A person's projectless task (jig/docs/design/domain/authority.md, 6.1).
use super::runs::run_choice::{account, granted, host};
use crate::tests::*;

const PERSON: u64 = 1;
const OTHER: u64 = 2;
const PROJECT: u32 = people::DEPLOYMENT;
const POOL: tasks::Funder = tasks::Funder::Pool { project: PROJECT, person: PERSON, period: 0 };
const PERIOD: tasks::Funder = tasks::Funder::Period { project: PROJECT, period: 0 };
const RUN_SPEND: u64 = 100; // The fixture's rules.maximum_run_spend.
const CEILING_SPEND: u64 = 1000; // The fixture's rules.ceiling.budget.spend.
const PERIOD_SPEND: u64 = 2000;

fn fixture() -> (Core, Env<Limits>) {
    fixture_with_project(false)
}

fn fixture_with_project(project: bool) -> (Core, Env<Limits>) {
    let mut limits = limits();
    limits.accounts.accounts = 2;
    limits.accounts.holder_accounts = 2;
    let mut configuration = config(41);
    configuration.defaults.rules.period_spend = PERIOD_SPEND;
    let mut alternative = scripted_model();
    alternative.name = b"second".as_slice().into();
    alternative.provider = 2;
    alternative.effort = Some(b"high".as_slice().into());
    configuration.settings.models = Box::new([scripted_model(), alternative.clone()]);
    configuration.defaults.charters[0].policy.alternatives = Box::new([alternative]);
    let mut provider = configuration.settings.llm_providers.providers[0].clone();
    provider.number = 2;
    configuration.settings.llm_providers.providers =
        Box::new([configuration.settings.llm_providers.providers[0].clone(), provider]);
    if !project {
        configuration.seed_projects = Box::new([]);
    }
    configuration.owners = Box::new([]);
    configuration.connectors = Box::new([]);
    let mut core = seeded_core(configuration, &limits).expect("projectless deployment");
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = Queue::uncapped();
    for person in [PERSON, OTHER] {
        people::step(
            &mut core.people,
            &env.with(limits.people),
            people::Event::Restore {
                record: people::Stored::Person {
                    number: person,
                    identity: people::Identity {
                        key: people::IdentityKey { provider: 0, subject: person.to_be_bytes().into() },
                        login: b"person".as_slice().into(),
                        name: b"Person".as_slice().into(),
                    },
                },
            },
            &mut out,
        );
    }
    finish_people(&mut core.people, &env.with(limits.people));
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    (core, env)
}

fn event(seed: PersonSeed, llm: Option<tasks::LlmChoice>) -> Event {
    Event::PersonTask {
        seed: Box::new(seed),
        executor: tasks::Executor::Agent { charter: 1 },
        llm,
        spec: tasks::Spec {
            job: None,
            asks: false,
            words: b"answer once".as_slice().into(),
            parameters: Box::new([]),
            inputs: Box::new([]),
        },
        contract: tasks::Contract::Report { words: 64 },
    }
}

fn make_person(core: &mut Core, env: &Env<Limits>) -> u64 {
    let seed = core.person_seed(PERSON).expect("known person's pool has room");
    let number = seed.number();
    drop(step(core, env, event(seed, None)));
    number
}

#[test]
fn the_first_seed_opens_rules_funding_and_the_second_reuses_it() {
    let (mut core, env) = fixture();
    let first = make_person(&mut core, &env);
    let row = core.tasks.task(first).expect("person task admitted without a project");
    assert_eq!(row.funder, POOL);
    assert_eq!(row.requester, tasks::Party::Deployment { project: PROJECT });
    assert_eq!(row.numbers.budget, RUN_SPEND);
    assert_eq!(core.tasks.funding(PERIOD).unwrap().numbers.budget, PERIOD_SPEND);
    assert_eq!(core.tasks.funding(POOL).unwrap().numbers.reserved, RUN_SPEND);
    assert_eq!(core.tasks.funding(POOL).unwrap().numbers.budget, CEILING_SPEND);
    let second = core.person_seed(PERSON).unwrap();
    assert!(second.opens.0.is_none());
    assert!(second.opens.1.is_none());
    drop(step(&mut core, &env, event(second, None)));
    assert_eq!(core.tasks.funding(POOL).unwrap().numbers.reserved, RUN_SPEND * 2);
    assert_eq!(core.tasks.funding(PERIOD).unwrap().numbers.reserved, CEILING_SPEND);
}

#[test]
fn the_seed_removes_every_permission_beyond_a_model_and_budget() {
    let (mut core, _) = fixture();
    let seed = core.person_seed(PERSON).expect("known person's pool has room");
    let given = seed.authority;
    assert_eq!(given.tools, tasks::Tools(0));
    assert!(given.grants.is_empty());
    assert_eq!(given.notes, tasks::Scopes(0));
    assert!(given.note_resources.is_empty());
    assert_eq!(given.delegation, tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 });
    assert_eq!(given.budget.spend, RUN_SPEND);
    assert_eq!(given.models, tasks::Models::ALL);
}

fn task_document(core: &Core, env: &Env<Limits>, task: u64) -> Task {
    let Document::Task(document) =
        make(core, &env.limits, PERSON, &Subject::Task { task }, &window(), env.wall).expect("payer may read Task")
    else {
        panic!("Task document")
    };
    *document
}

#[test]
fn only_the_payer_can_read_its_account_wait_in_the_task_document() {
    let (mut core, env) = fixture();
    let task = make_person(&mut core, &env);
    let document = task_document(&core, &env, task);
    assert_eq!(document.run_wait, Some(TaskRunWait::Account { account: None, source: None, why: TaskWaitWhy::Empty }));
    assert_eq!(document.kept_account, None);
    assert_eq!(document.chip.details.observation, None);
    assert_eq!(make(&core, &env.limits, OTHER, &Subject::Task { task }, &window(), env.wall), Err(Missing::NotAllowed));
    assert_eq!(
        task_from_row(&core, &env.limits, OTHER, core.tasks.task(task).unwrap(), env.wall),
        Err(Missing::NotAllowed)
    );
}

fn owner_fixture() -> (Core, Env<Limits>) {
    let (mut core, env) = fixture();
    people::step(
        &mut core.people,
        &env.with(env.limits.people),
        people::Event::Roles {
            project: PROJECT,
            holdings: Box::new([people::Holding { person: PERSON, role: people::Role::Owner }]),
        },
        &mut Queue::uncapped(),
    );
    (core, env)
}

#[test]
fn a_deployment_owner_can_read_a_persons_task_without_being_offered_an_act() {
    let (mut core, env) = owner_fixture();
    let task = make_person(&mut core, &env);
    let document = task_document(&core, &env, task);
    assert!(!document.acts.is_empty(), "the document retains refused acts");
    for offered in &document.acts {
        match offered.admission {
            crate::acts::Admission::Refused(_) => {}
            crate::acts::Admission::Allowed
            | crate::acts::Admission::Bounded { .. }
            | crate::acts::Admission::Proposed { .. } => {
                panic!("a projectless task is the application's: {offered:?}");
            }
        }
    }
}

#[test]
fn a_deployment_owner_cannot_send_words_to_a_persons_task() {
    let (mut core, env) = owner_fixture();
    let task = make_person(&mut core, &env);
    let answer = core.person_message(
        &env.limits,
        PersonMessage {
            request: Token::new(1),
            person: PERSON,
            project: PROJECT,
            task,
            question: None,
            words: b"another word".as_slice().into(),
            at: env.wall,
        },
    );
    assert_eq!(answer, Err(people::Refusal::Role { held: Some(people::Role::Owner) }));
}

#[test]
fn a_live_person_task_is_absent_from_chats_inbox_results_and_me_projects() {
    let (mut core, env) = fixture();
    let task = make_person(&mut core, &env);
    assert!(core.tasks.task(task).is_some(), "the query runs with the task live");
    assert_eq!(
        core.tasks.task(task).unwrap().requester,
        tasks::Party::Deployment { project: PROJECT },
        "a person's query must not select its requester"
    );
    let subject = Subject::Query {
        query: TaskQuery {
            project: None,
            requester: Some(tasks::Party::Person(PERSON)),
            root: None,
            parent: None,
            label: None,
            phase: None,
            executor: None,
            tracked: None,
            order: QueryOrder::Number,
        },
    };
    let Document::Lines(chats) = make(&core, &env.limits, PERSON, &subject, &window(), env.wall).unwrap() else {
        panic!("Chats-style query")
    };
    assert!(chats.lines.is_empty());
    let Document::Inbox(inbox) =
        make(&core, &env.limits, PERSON, &Subject::Inbox { party: PERSON }, &window(), env.wall).unwrap()
    else {
        panic!("inbox")
    };
    assert!(inbox.needs.is_empty());
    assert!(inbox.updates.is_empty());
    assert_eq!(inbox.cached_needs, 0);
    assert_eq!(inbox.cached_updates, Count::Known(0));
    let Document::Frame(me) =
        make(&core, &env.limits, PERSON, &Subject::Frame { party: PERSON }, &window(), env.wall).unwrap()
    else {
        panic!("me")
    };
    assert!(me.projects.is_empty());
    assert_eq!(me.updates, Count::Known(0));
    assert!(core.people.cached_results(PERSON).unwrap_or(&[]).is_empty());
    assert!(core.people.cached_updates(&env.limits.people, PERSON).unwrap().is_empty());
}

#[test]
fn the_next_seed_is_bounded_by_room_and_refused_after_it() {
    let (mut core, env) = fixture();
    make_person(&mut core, &env);
    // The live task reserves RUN_SPEND; leave exactly one further unit in its pool.
    send_task(
        &mut core,
        &env.limits,
        tasks::Event::ResizePool {
            reply_to: ReplyTo::new(Token::new(1)),
            project: PROJECT,
            person: PERSON,
            period: 0,
            budget: RUN_SPEND + 1,
        },
    );
    let last = core.person_seed(PERSON).expect("one unit remains");
    assert_eq!(last.authority.budget.spend, 1);
    drop(step(&mut core, &env, event(last, None)));
    assert!(core.person_seed(PERSON).is_none(), "the live task reserved the last unit");
    assert!(core.person_seed(u64::MAX).is_none(), "unknown person");
}

fn person_account(
    core: &mut Core,
    env: &Env<Limits>,
    number: u32,
    provider: u32,
    kind: accounts::Kind,
    uses: Option<Box<[Use]>>,
) {
    for record in [
        accounts::Stored::Account {
            list: None,
            number,
            provider,
            holder: accounts::Holder::Party(PERSON),
            place: 0,
            uses,
            subject: Box::new([]),
            display: Box::new([]),
        },
        accounts::Stored::Source { account: number, number, place: 0, kind, name: Box::new([]), held: None },
    ] {
        account(core, &env.limits, accounts::Event::Restore { record });
    }
}

fn host_accounts(core: &mut Core, env: &Env<Limits>, sources: &[u32]) {
    account(core, &env.limits, accounts::Event::Want);
    for source in sources {
        granted(core, &env.limits, *source);
    }
    host(core, &env.limits, Token::new(7), &[b"scripted", b"second"]);
}

fn claim(core: &mut Core, env: &Env<Limits>, task: u64, account: Option<u32>) {
    send_task(core, &env.limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(1)), task });
    send_task(
        core,
        &env.limits,
        tasks::Event::Claim {
            session_end: None,
            offered: jig_domain_tasks::Reasons::ALL,
            reasons: jig_domain_tasks::Reasons::FIRST,
            reply_to: ReplyTo::new(Token::new(1)),
            task,
            attempt: 1,
            resumed: false,
            budget: RUN_SPEND,
            account,
            writes: Box::new([]),
        },
    );
}

#[test]
fn a_persons_plan_and_key_choose_the_first_usable_model_and_keep_its_account() {
    for kind in [accounts::Kind::SignIn, accounts::Kind::Key] {
        let (mut core, env) = fixture();
        person_account(&mut core, &env, 1, 2, kind, None);
        host_accounts(&mut core, &env, &[1]);
        let task = make_person(&mut core, &env);
        assert_eq!(core.tasks.payer(task), Some(tasks::Payer::Party(PERSON)));
        let choice = core.run_choice(task).expect("person's account serves projectless task");
        assert_eq!(choice.policy.model.name.as_ref(), b"second");
        assert_eq!(choice.accounts.first().unwrap().account, 1);
        assert_eq!(core.tasks.task(task).unwrap().llm, None);
        claim(&mut core, &env, task, Some(1));
        assert_eq!(task_document(&core, &env, task).kept_account, Some(1));
    }
}

#[test]
fn a_task_held_to_an_account_runs_on_it_alone() {
    // Accounts as (number, provider), the account held, and its model and default effort.
    for (held_accounts, held, model, main, subagent, unheld_account) in [
        (
            &[(1, 1), (2, 2)][..],
            2,
            b"second".as_slice(),
            tasks::ModelEffort { model: 1, effort: Some(1) },
            tasks::ModelEffort { model: 0, effort: Some(0) },
            1,
        ), // Claude, then ChatGPT
        (
            &[(1, 2), (2, 1)][..],
            2,
            b"scripted".as_slice(),
            tasks::ModelEffort { model: 0, effort: Some(0) },
            tasks::ModelEffort { model: 1, effort: Some(1) },
            2,
        ), // ChatGPT, then Claude
        (
            &[(1, 2), (2, 2)][..],
            2,
            b"second".as_slice(),
            tasks::ModelEffort { model: 1, effort: Some(1) },
            tasks::ModelEffort { model: 0, effort: Some(0) },
            1,
        ), // A second account of one provider
    ] {
        let (mut core, env) = fixture();
        for &(number, provider) in held_accounts {
            person_account(&mut core, &env, number, provider, accounts::Kind::Key, None);
        }
        host_accounts(&mut core, &env, &[1, 2]);
        let unheld = make_person(&mut core, &env);
        assert_eq!(
            core.run_choice(unheld).unwrap().accounts.first().unwrap().account,
            unheld_account,
            "{held_accounts:?}"
        );
        let seed = core.person_seed(PERSON).expect("room");
        let task = seed.number();
        let llm =
            core.account_llm(1, seed.models(), tasks::Payer::Party(PERSON), PROJECT, held).expect("it serves a model");
        assert_eq!(llm.account, Some(held), "{held_accounts:?}");
        assert_eq!(llm.main, main, "{held_accounts:?}");
        assert_eq!(llm.subagents.as_ref(), &[subagent], "{held_accounts:?}");
        drop(step(&mut core, &env, event(seed, Some(llm))));
        let choice = core.run_choice(task).expect("the named account is usable");
        assert_eq!(choice.policy.model.name.as_ref(), model, "{held_accounts:?}");
        assert_eq!(choice.accounts.first().unwrap().account, held, "{held_accounts:?}");
        claim(&mut core, &env, task, Some(held));
        assert_eq!(task_document(&core, &env, task).kept_account, Some(held), "{held_accounts:?}");
    }
}

#[test]
fn an_account_choice_requires_coverage_and_waits_for_its_source() {
    for (uses, sources, expected) in [
        (Some(Box::<[Use]>::from([Use::Project(1)])), &[1][..], None),
        (
            None,
            &[][..],
            Some(tasks::LlmChoice {
                later: None,
                account: Some(1),
                main: tasks::ModelEffort { model: 1, effort: Some(1) },
                subagents: Box::new([tasks::ModelEffort { model: 0, effort: Some(0) }]),
            }),
        ),
    ] {
        let (mut core, env) = fixture();
        person_account(&mut core, &env, 1, 2, accounts::Kind::Key, uses);
        host_accounts(&mut core, &env, sources);
        let seed = core.person_seed(PERSON).expect("room");
        let task = seed.number();
        let llm = core.account_llm(1, seed.models(), tasks::Payer::Party(PERSON), PROJECT, 1);
        assert_eq!(llm, expected, "granted sources: {sources:?}");
        if let Some(llm) = llm {
            drop(step(&mut core, &env, event(seed, Some(llm))));
            assert_eq!(
                core.run_choice(task),
                Err(RunWait::Account { account: Some(1), source: Some(1), why: accounts::WaitWhy::Refreshing })
            );
        }
    }
}

#[test]
fn a_project_only_key_leaves_the_charters_first_model_waiting_without_an_account() {
    let (mut core, env) = fixture();
    person_account(&mut core, &env, 1, 2, accounts::Kind::Key, Some(Box::new([Use::Project(1)])));
    host_accounts(&mut core, &env, &[1]);
    let task = make_person(&mut core, &env);
    assert_eq!(
        core.run_choice(task),
        Err(RunWait::Account { account: None, source: None, why: accounts::WaitWhy::Empty })
    );
    assert_eq!(core.next_llm(core.tasks.task(task).unwrap()).unwrap().main.model, 0);
    let document = task_document(&core, &env, task);
    assert_eq!(document.llm, None);
    assert_eq!(document.run_wait, Some(TaskRunWait::Account { account: None, source: None, why: TaskWaitWhy::Empty }));
}

#[test]
fn a_person_task_with_a_usable_account_is_admitted_to_run() {
    let (mut core, env) = fixture();
    person_account(&mut core, &env, 1, 2, accounts::Kind::Key, None);
    host_accounts(&mut core, &env, &[1]);
    let task = make_person(&mut core, &env);
    let context = core.tasks.run_context(task).expect("live agent task");
    let admission = core.run_admission(&context, env.wall, Box::new([]));
    match admission {
        RunAdmission::Allow(_) => {}
        other @ (RunAdmission::Wait(_) | RunAdmission::Hold(_)) => panic!("projectless run admission: {other:?}"),
    }
}

#[test]
fn a_real_projects_new_period_does_not_renew_projectless_money() {
    let (mut core, env) = fixture_with_project(true);
    let task = make_person(&mut core, &env);
    let period = *core.tasks.funding(PERIOD).unwrap();
    let pool = *core.tasks.funding(POOL).unwrap();
    drop(step(&mut core, &env, Event::Period { project: 1, period: 1, budget: CEILING_SPEND, next: None }));
    assert_eq!(core.period(1), 1, "real project's period opened");
    assert_eq!(core.period(PROJECT), 0);
    assert_eq!(core.tasks.funding(PERIOD), Some(&period));
    assert_eq!(core.tasks.funding(POOL), Some(&pool));
    assert_eq!(core.tasks.task(task).unwrap().funder, POOL);
}

#[test]
fn failures_past_the_tries_hold_without_an_inbox_until_cancelled() {
    let (mut core, mut env) = fixture();
    let task = make_person(&mut core, &env);
    for count in 1..=2 {
        send_task(&mut core, &env.limits, tasks::Event::Prepare { reply_to: ReplyTo::new(Token::new(1)), task });
        let out = step(
            &mut core,
            &env,
            Event::Tasks(tasks::Event::PreparationCounted {
                task,
                class: tasks::Class::Permanent,
                reason: b"cannot prepare".as_slice().into(),
            }),
        );
        for request in &out {
            if let Request::Now(now) = request
                && let Now::RestoreRefused = now.as_ref()
            {
                panic!("no recipient is valid for projectless work");
            }
        }
        assert_eq!(core.tasks.task(task).unwrap().tries.permanent, count);
        if count == 1 {
            let tasks::Phase::Active(tasks::Active::BackingOff { until }) = core.tasks.task(task).unwrap().phase else {
                panic!("first failure backs off")
            };
            env.now = core.tasks.next_deadline().unwrap();
            env.wall = until;
            drop(fire(&mut core, &env, Timer::Tasks));
        }
    }
    let held = core.tasks.task(task).unwrap().phase.clone();
    match &held {
        tasks::Phase::Held { why, .. } => assert_eq!(*why, tasks::Hold::Failures(tasks::Class::Permanent)),
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => {
            panic!("exhausted tries must hold: {held:?}");
        }
    }
    for person in [PERSON, OTHER] {
        let Document::Inbox(inbox) =
            make(&core, &env.limits, person, &Subject::Inbox { party: person }, &window(), env.wall).unwrap()
        else {
            panic!("inbox")
        };
        assert!(inbox.needs.is_empty());
        assert!(inbox.updates.is_empty());
    }
    env.now = env.now.checked_add(Duration::from_secs(7200)).unwrap();
    env.wall = Wall::from_nanos(env.wall.as_nanos() + 7_200_000_000_000);
    drop(fire(&mut core, &env, Timer::Tasks));
    assert_eq!(core.tasks.task(task).unwrap().phase, held);
    drop(step(
        &mut core,
        &env,
        Event::Tasks(tasks::Event::Cancel {
            reply_to: ReplyTo::new(Token::new(2)),
            by: tasks::Party::Deployment { project: PROJECT },
            operation: tasks::Cancel { task, reason: Box::new([]), grace: None, message: None },
        }),
    ));
    // The unit fixture has no connector roots; supply their settled cleanup.
    drop(step(&mut core, &env, Event::Tasks(tasks::Event::EffectsSettled { task })));
    drop(step(&mut core, &env, Event::Tasks(tasks::Event::Settled { task })));
    if let Some(row) = core.tasks.task(task) {
        match row.phase {
            tasks::Phase::Ended(_) => {}
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                panic!("cancelled task: {:?}", row.phase);
            }
        }
    }
}

#[test]
fn run_progress_uses_the_current_chip_observation_and_ends_with_the_run() {
    let (mut core, mut env) = fixture();
    let task = make_person(&mut core, &env);
    assert!(!core.run_past_starting(task));
    assert!(!core.run_past_starting(u64::MAX));
    claim(&mut core, &env, task, None);
    env.limits.fleet.engine_slots = 1;
    core.fleet = fixtures::restored_fleet(&env.limits, fixtures::needs());
    let mut out = Queue::uncapped();
    fleet::step(
        &mut core.fleet,
        &env.with(env.limits.fleet),
        fleet::Event::Start {
            placement: None,
            version: fleet::Version { charter: 1, revision: 1 },
            grants: Box::new([]),
            reply_to: ReplyTo::new(Token::new(1)),
            run: Token::new(task),
            attempt: Token::new(1),
            workstream: 1,
            kinds: fleet::Kinds::Engine,
            assignment: fleet::Assignment { turns: Token::new(0), answered: Token::new(0) },
        },
        &mut out,
    );
    fleet::resume(&mut core.fleet, &env.with(env.limits.fleet), &mut out);
    for (activity, work, past) in [
        (fleet::Activity::Preparing, ChipWork::Preparing, false),
        (fleet::Activity::Starting, ChipWork::Starting, false),
        (fleet::Activity::Active, ChipWork::Active, true),
        (fleet::Activity::Writing, ChipWork::Writing, true),
        (fleet::Activity::Waiting, ChipWork::Waiting, true),
        (fleet::Activity::Ending, ChipWork::Ending, true),
    ] {
        fleet::step(
            &mut core.fleet,
            &env.with(env.limits.fleet),
            fleet::Event::Observe {
                channel: Token::new(0),
                run: Token::new(task),
                attempt: Token::new(1),
                report: fleet::Report::Activity(activity),
            },
            &mut out,
        );
        assert_eq!(task_document(&core, &env, task).chip.details.observation.unwrap().work, work);
        assert_eq!(core.run_past_starting(task), past, "{work:?}");
    }
    send_task(
        &mut core,
        &env.limits,
        tasks::Event::Activation {
            reply_to: ReplyTo::new(Token::new(1)),
            task,
            attempt: 1,
            end: tasks::End::Parked,
            saved: None,
            cause: tasks::Cause::Unpriced,
        },
    );
    assert!(!core.run_past_starting(task), "old host observation cannot outlive the run");
    assert!(task_document(&core, &env, task).chip.details.observation.is_none());
}
