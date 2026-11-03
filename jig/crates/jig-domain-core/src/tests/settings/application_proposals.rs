//! Proposal admission for application settings (jig/docs/design/domain/engine.md, 17.6).
use crate::settings::{self as s, Kind, KindScope};
use crate::tests::*;

#[test]
fn application_only_proposal_keeps_its_sized_set() {
    let (owner, mut limits) = core();
    limits.people.projects = 2;
    let mut configured = config(41);
    configured.kinds = Box::new([Kind {
        number: 1,
        scope: KindScope::Deployment,
        connector: 2,
        value_bytes: 8,
        per_scope: 2,
        charters: 0,
    }]);
    configured.connectors = Box::new([2]);
    let mut core = seeded_core(configured, &limits).unwrap();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.people };
    for record in [
        people::Stored::Person { number: 1, identity: owner.people.identity(1).unwrap().clone() },
        people::Stored::SignIn { number: 10, person: 1, expires: Wall::from_nanos(Duration::from_secs(60).as_nanos()) },
        people::Stored::Roles {
            project: 0,
            holdings: Box::new([people::Holding { person: 1, role: people::Role::Owner }]),
        },
    ] {
        people::step(&mut core.people, &env, people::Event::Restore { record }, &mut Queue::uncapped());
    }
    finish_people(&mut core.people, &env);
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    let output = step(
        &mut core,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits },
        Event::ProposeSettings {
            reply_to: ReplyTo::new(Token::new(91)),
            sign_in: 10,
            key: [1; 16],
            digest: [1; 32],
            change: Box::new(s::Proposed {
                against: 1,
                reason: b"review".as_slice().into(),
                items: Box::new([s::Item {
                    setting: s::Setting::Application { kind: 1, scope: 0, number: 1 },
                    edit: s::Edit::Set(s::Value::Application(b"value".as_slice().into())),
                }]),
            }),
        },
    );
    let set = match core.settings_proposals.first() {
        Some((_, set)) => Some(set),
        None => None,
    };
    assert!(set.is_some(), "application-only proposal must be admitted: {output:?}");
    assert_eq!(
        set.unwrap().applications.as_ref(),
        &[s::application::Recorded { kind: 1, scope: 0, number: 1, edit: s::application::SizedEdit::Set { size: 5 } }]
    );
}
