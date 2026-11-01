//! First-start project funding after deployment settings change
//! (jig/docs/design/domain/engine.md, 17.2; jig/docs/design/domain/people.md, 5.2).
use crate::{settings as s, tests::*};

fn first_start() -> (Core, Limits) {
    let limits = limits();
    let mut configured = config(41);
    configured.seed_projects[0].policy.permissions = Box::new([]);
    configured.defaults.policies = Map::with_capacity(1);
    configured.defaults.policies.insert(1, configured.seed_projects[0].policy.clone()).unwrap();
    let mut core = Core::new(configured, &limits).unwrap();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    s::check(&core.current, &core.settings, &core.connectors, &limits).unwrap();
    core.people.start_restore(env.wall);
    core.authority.start_restore();
    crate::start::first::start_seed(&mut core, &env, &mut Queue::uncapped());
    assert_eq!(core.start_failure(), None);
    assert_eq!(core.counters.deployment().people, 1);
    assert!(core.people.identity(1).is_some(), "seeded owner");
    core.authority.start_link().unwrap();
    people::step(
        &mut core.people,
        &env.with(limits.people),
        people::Event::Restore {
            record: people::Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(60_000_000_000) },
        },
        &mut Queue::uncapped(),
    );
    finish_people(&mut core.people, &env.with(limits.people));
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    send_task(
        &mut core,
        &limits,
        tasks::Event::OpenPeriod { reply_to: ReplyTo::new(Token::new(0)), project: 1, period: 1, budget: 1000 },
    );
    let (period, pool) = core.open_person_pool(1, 1);
    for event in [period, pool].into_iter().flatten() {
        send_task(&mut core, &limits, event);
    }
    (core, limits)
}

fn raised_settings(core: &Core) -> s::Change {
    let mut rules = core.current.rules().clone();
    rules.ceiling.budget.spend = 207_552;
    rules.period_spend = 449_696;
    let mut policy = core.current.policy(1).unwrap().clone();
    policy.policy.ceiling.budget.spend = 207_552;
    policy.policy.period_spend = 449_696;
    policy.policy.roles[0].authority.budget.spend = 207_552;
    policy.policy.roles[0].period_spend = 415_104;
    s::Change {
        against: core.current.newest(),
        running: s::Running::Stays,
        items: Box::new([
            s::Item { setting: s::Setting::Rules, edit: s::Edit::Set(s::Value::Rules(Box::new(rules))) },
            s::Item { setting: s::Setting::Policy(1), edit: s::Edit::Set(s::Value::Policy(Box::new(policy))) },
        ]),
    }
}

fn fund(core: &mut Core, limits: &Limits, key: u8, change: people::PolicyChange) {
    let out = watch_outputs(
        core,
        limits,
        Event::People(people::Event::Ask {
            reply_to: ReplyTo::new(Token::new(u64::from(key))),
            sign_in: 10,
            key: [key; 16],
            ask: people::Ask::ChangePolicy { project: 1, change },
            models: Box::new([]),
        }),
    );
    let mut outcome = None;
    for request in &out {
        if let Request::Held(held) = request
            && let Held::PeopleReply { reply: people::Reply::Outcome(reply), .. } = held.as_ref()
        {
            assert!(outcome.is_none(), "one funding answer");
            outcome = Some(reply);
        }
    }
    assert_eq!(outcome, Some(&people::Outcome::PolicyChanged { project: 1 }));
    core.reclaim();
}

#[test]
fn first_start_project_needs_funding_after_settings_are_raised() {
    let (mut core, limits) = first_start();
    let period = tasks::Funder::Period { project: 1, period: 1 };
    let pool = tasks::Funder::Pool { project: 1, person: 1, period: 1 };
    assert_eq!(core.tasks.funding(period).unwrap().numbers.budget, 1000);
    assert_eq!(core.tasks.funding(pool).unwrap().numbers.budget, 500);
    let change = raised_settings(&core);
    watch_outputs(
        &mut core,
        &limits,
        Event::ChangeSettings {
            reply_to: ReplyTo::new(Token::new(1)),
            sign_in: 10,
            key: [1; 16],
            digest: [1; 32],
            change: Box::new(change),
        },
    );
    assert_eq!(core.current.newest(), 2);
    core.reclaim();
    assert_eq!(core.current.policy(1).unwrap().policy.period_spend, 449_696);
    assert_eq!(core.authority.rules().period_spend, 449_696);
    assert_eq!(core.tasks.funding(period).unwrap().numbers.budget, 1000);
    assert_eq!(core.tasks.funding(pool).unwrap().numbers.budget, 500);
    fund(&mut core, &limits, 2, people::PolicyChange::ProjectSpend { period_spend: 449_696 });
    assert_eq!(core.tasks.funding(period).unwrap().numbers.budget, 449_696);
    assert_eq!(core.tasks.funding(pool).unwrap().numbers.budget, 500);
    fund(&mut core, &limits, 3, people::PolicyChange::RoleSpend { role: 0, period_spend: 415_104 });
    assert_eq!(core.tasks.funding(pool).unwrap().numbers.budget, 415_104);
    assert_eq!(core.tasks.funding(period).unwrap().numbers.reserved, 415_104);
}
