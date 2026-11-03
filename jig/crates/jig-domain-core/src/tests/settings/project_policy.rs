//! A project's policy item waits for its check (jig/docs/design/domain/engine.md, 17.2, 17.6).
use crate::settings as s;
use crate::tests::*;

const OWNER: u64 = 1;
const SIGN_IN: u64 = 10;
const PROJECT: u32 = 1;
const POLICY: u32 = 1;

fn project_core() -> (Core, Limits) {
    let (owner, limits) = core();
    let mut configured = config(41);
    configured.defaults.policies = Map::with_capacity(1);
    configured.defaults.policies.insert(POLICY, configured.seed_projects[0].policy.clone()).unwrap();
    let mut core = seeded_core(configured, &limits).unwrap();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    for record in [
        people::Stored::Person { number: OWNER, identity: owner.people.identity(OWNER).unwrap().clone() },
        people::Stored::SignIn {
            number: SIGN_IN,
            person: OWNER,
            expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()),
        },
        people::Stored::Roles {
            project: people::DEPLOYMENT,
            holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Owner }]),
        },
        people::Stored::Roles {
            project: PROJECT,
            holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Owner }]),
        },
    ] {
        people::step(&mut core.people, &env, people::Event::Restore { record }, &mut Queue::uncapped());
    }
    finish_people(&mut core.people, &env);
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    (core, limits)
}

fn defaults(core: &Core) -> Box<[(s::Setting, s::Default)]> {
    let mut values = List::with_capacity(core.current.defaults.len());
    for (&setting, default) in &core.current.defaults {
        values.push((setting, default.clone())).unwrap();
    }
    values.into_boxed()
}

#[test]
fn a_project_policy_item_is_refused_its_shape_until_its_check() {
    let (mut core, limits) = project_core();
    let setting = s::Setting::ProjectPolicy { project: PROJECT };
    let item = s::Item {
        setting,
        edit: s::Edit::Set(s::Value::Policy(Box::new(core.current.policy(POLICY).unwrap().clone()))),
    };
    let before = defaults(&core);
    let head = core.current.head.clone();
    let oldest = core.current.oldest;
    let keys = core.current.keys;
    for event in [
        Event::ChangeSettings {
            reply_to: ReplyTo::new(Token::new(91)),
            sign_in: SIGN_IN,
            key: [1; 16],
            digest: [1; 32],
            change: Box::new(s::Change {
                against: head.number,
                items: Box::new([item.clone()]),
                running: s::Running::Stays,
            }),
        },
        Event::ProposeSettings {
            reply_to: ReplyTo::new(Token::new(92)),
            sign_in: SIGN_IN,
            key: [2; 16],
            digest: [2; 32],
            change: Box::new(s::Proposed {
                against: head.number,
                reason: b"review".as_slice().into(),
                items: Box::new([item]),
            }),
        },
    ] {
        let mut out = step(&mut core, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits }, event);
        let mut outcomes = List::with_capacity(1);
        while let Some(request) = out.pop() {
            if let Request::Held(held) = &request
                && let Held::PeopleReply { reply: people::Reply::Outcome(outcome), .. } = held.as_ref()
            {
                outcomes.push(outcome.clone()).unwrap();
            }
            if let Request::Write(
                Write::Save(Record::Core(CoreRecord::Settings(_))) | Write::Erase(Key::Core(CoreKey::Settings(_))),
            ) = request
            {
                panic!("a refused policy item makes no settings or history writes");
            }
        }
        assert_eq!(
            outcomes.as_slice(),
            &[people::Outcome::Refused(people::Refusal::NotWellFormed {
                at: 0,
                why: people::Malformed::Settings { setting, fault: people::SettingFault::Shape },
            })]
        );
        assert_eq!(defaults(&core), before);
        assert!(core.current.changes.is_empty());
        assert!(core.current.applications.is_empty());
        assert_eq!(core.current.head, head);
        assert_eq!(core.current.oldest, oldest);
        assert_eq!(core.current.keys, keys);
        assert!(core.settings_proposals.is_empty());
    }
}
