//! Clock projection and ordered timer selection (jig/docs/design/domain/engine/README.md, 4.1).
use crate::tests::*;
use jig_domain_link::{Form, Scheme};

fn arm_brief(core: &mut Core, limits: &Limits, deadline: Time) {
    let mut out = Queue::uncapped();
    brief::gather_step(
        &mut core.brief,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.brief },
        brief::GatherEvent::Plan {
            brief: Token::new(7),
            budget: 32,
            deadline,
            sections: Box::new([brief::Planned::Connector {
                name: b"section",
                connector: 1,
                kind: 1,
                token: Token::new(8),
                size: 8,
                limit: 8,
                priority: 1,
                required: true,
            }]),
        },
        &mut out,
    );
    assert_eq!(core.brief.next_deadline(), Some(deadline));
}

#[test]
fn next_deadline_is_the_earliest_child_alarm() {
    let (mut core, env) = routing::engine_timers::timer_core(false);
    let people = core.people.next_deadline().expect("sign-in alarm");
    assert_eq!(core.next_deadline(env.now, env.wall), Some(people));
    let brief = Time::from_nanos(12);
    arm_brief(&mut core, &env.limits, brief);
    assert_eq!(core.next_deadline(env.now, env.wall), Some(brief));
    assert_eq!(core.due(Time::from_nanos(11), env.wall), None);
    assert_eq!(core.due(brief, env.wall), Some(Timer::Brief));
}

#[test]
fn saved_wall_expiry_counts_only_after_people_are_ready() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("fixture core configuration");
    let now = Time::from_nanos(7);
    let expiry = Wall::from_nanos(100);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    core.people.start_restore(env.wall);
    let mut out = Queue::uncapped();
    for record in [
        people::Stored::Person {
            number: 1,
            identity: people::Identity {
                key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
                login: Box::from(b"owner".as_slice()),
                name: Box::from(b"Owner".as_slice()),
            },
        },
        people::Stored::SignIn { number: 1, person: 1, expires: expiry },
    ] {
        people::step(&mut core.people, &env, people::Event::Restore { record }, &mut out);
    }
    assert_eq!(core.people.next_wall_expiry(), Some(expiry));
    assert!(!core.people.ready());
    assert_eq!(core.next_deadline(now, expiry), None);
    assert_eq!(core.due(now, expiry), None);
    finish_people(&mut core.people, &env);
    assert_eq!(core.next_deadline(now, expiry), Some(now));
    assert_eq!(core.due(now, expiry), Some(Timer::People));
    assert_eq!(core.next_deadline(now, Wall::from_nanos(90)), Some(Time::from_nanos(17)));
}

#[test]
fn due_selects_the_first_due_child_before_an_earlier_alarm() {
    let (mut core, env) = routing::engine_timers::timer_core(false);
    arm_brief(&mut core, &env.limits, Time::from_nanos(12));
    let people = core.people.next_deadline().expect("sign-in alarm");
    assert_eq!(core.due(people, env.wall), Some(Timer::People));
    core.stopping = true;
    assert_eq!(core.next_deadline(people, env.wall), None);
    assert_eq!(core.due(people, env.wall), None);
}

#[test]
fn decision_header_numbers_writing_or_dirty_decisions_once() {
    for (dirty, wrote, commit) in
        [(false, false, None), (false, true, Some(1)), (true, false, Some(1)), (true, true, Some(1))]
    {
        let limits = limits();
        let mut core = Core::new(config(41), &limits).expect("fixture core configuration");
        if !dirty {
            core.counters = Counters::new(core.counters.deployment());
        }
        let mut expected = core.counters.deployment();
        assert_eq!(expected.commits, 0);
        let header = match commit {
            Some(number) => {
                expected.commits = number;
                Some(Record::Core(CoreRecord::Deployment(expected)))
            }
            None => None,
        };
        assert_eq!(core.decision_header(wrote), header, "dirty={dirty}, wrote={wrote}");
        assert_eq!(core.counters.deployment(), expected);
        assert!(!core.counters.dirty());
        assert_eq!(core.decision_header(false), None);
        assert_eq!(core.counters.deployment(), expected);
    }
}

#[test]
fn a_dirty_header_is_taken_once_and_never_before_the_load() {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).expect("fixture core configuration");
    let Some(Record::Core(CoreRecord::Deployment(header))) = core.decision_header(false) else {
        unreachable!("a fresh deployment supplies its header");
    };
    assert_eq!(header.commits, 1);
    assert!(core.decision_header(false).is_none());
    core.counters.begin_start();
    assert!(core.decision_header(false).is_none());
    core.counters.finish_restore();
    assert!(core.decision_header(false).is_some());
    assert!(core.decision_header(false).is_none());
}

fn with_task_alarm() -> (Core, Limits) {
    let (mut core, limits) = core();
    make_task(&mut core, &limits, 1);
    let mut out = Queue::uncapped();
    tasks::step(
        &mut core.tasks,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks },
        tasks::Event::Subscribe {
            reply_to: ReplyTo::new(Token::new(3)),
            task: 1,
            subscription: tasks::Subscription {
                number: 1,
                kind: tasks::SubscriptionKind::Timer { at: Wall::from_nanos(10), period: None },
            },
        },
        &mut out,
    );
    assert_eq!(core.tasks.next_deadline(), Some(Time::from_nanos(10)));
    (core,limits)
}

#[test]
fn held_new_work_keeps_finishing_timers_visible() {
    let (mut core, limits) = with_task_alarm();
    arm_brief(&mut core, &limits, Time::from_nanos(20));
    let now = Time::from_nanos(20);
    assert_eq!(core.next_deadline(now, Wall::EPOCH), Some(Time::from_nanos(10)));
    assert_eq!(core.due(now, Wall::EPOCH), Some(Timer::Tasks));
    assert_eq!(core.next_finishing_deadline(now, Wall::EPOCH), Some(now));
    assert_eq!(core.finishing_due(now, Wall::EPOCH), Some(Timer::Brief));
}

#[test]
fn account_alarms_keep_running_while_the_core_stops() {
    let (mut core, limits) = core();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.accounts };
    let mut out = Queue::uncapped();
    accounts::step(&mut core.accounts, &env, accounts::Event::Want, &mut out);
    accounts::step(
        &mut core.accounts,
        &env,
        accounts::Event::Component(accounts::FromComponent::Granted {
            form: Form::Credential(Scheme::Bearer),
            account: 1,
            generation: 1,
            valid: Duration::from_secs(60),
        }),
        &mut out,
    );
    let deadline = core.accounts.next_deadline().expect("credential alarm");
    arm_brief(&mut core, &limits, Time::from_nanos(1));
    core.stopping = true;
    assert_eq!(core.next_deadline(deadline, env.wall), Some(deadline));
    assert_eq!(core.due(deadline, env.wall), Some(Timer::Account));
}

#[test]
fn task_timers_wait_for_the_start_to_open_without_hiding_finishing_timers() {
    let (mut core,limits)=with_task_alarm();
    arm_brief(&mut core,&limits,Time::from_nanos(20));
    let now=Time::from_nanos(20);
    core.opened=false;
    assert_eq!(core.next_deadline(now,Wall::EPOCH),Some(now),"only the finishing alarm is visible during restore");
    assert_eq!(core.due(now,Wall::EPOCH),Some(Timer::Brief));
    core.opened=true;
    assert_eq!(core.next_deadline(now,Wall::EPOCH),Some(Time::from_nanos(10)),"the task alarm remains armed for opening");
    assert_eq!(core.due(now,Wall::EPOCH),Some(Timer::Tasks));
}
