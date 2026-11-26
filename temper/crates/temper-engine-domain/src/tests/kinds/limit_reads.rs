use super::*;
use jig_domain_people::{ApplicationScope, Role};

#[test]
fn limits_require_a_deployment_role_and_carry_the_roots_held_bytes() {
    for (role, scope, allowed) in [
        (Some(Role::Observer), ApplicationScope::Deployment, true),
        (None, ApplicationScope::Deployment, false),
        (Some(Role::Owner), ApplicationScope::Project(PROJECT), false),
    ] {
        let (mut domain, env) = reader(role);
        let query = crate::ForgeRead::Limits { after: Some(b"forge.entries".as_slice().into()) };
        let answer = read_forge(&mut domain, &env, READ_SIGN_IN, scope, query);
        if allowed {
            assert_eq!(
                answer,
                Delivery::ForgeRead {
                    reply: READ_REPLY,
                    answer: crate::ForgeReadAnswer::Limits {
                        after: Some(b"forge.entries".as_slice().into()),
                        held: domain.held_bytes(),
                    }
                }
            );
        } else {
            assert_eq!(
                answer,
                Delivery::Read {
                    reply: READ_REPLY,
                    answer: jig_domain_core::reads::Answer::None(jig_domain_core::reads::Why::Unknown)
                }
            );
        }
    }
}
#[test]
fn limit_holds_read_returns_the_kept_revision_and_only_visible_projects() {
    use crate::route::forge_route::limit_holds;
    let (mut domain, env, task) = live_task(None, jig_domain_tasks::Executor::Procedure { connector: 0, code: 1 });
    let (mut auth, _) = reader(Some(Role::Owner));
    ::core::mem::swap(&mut domain.engine.core_mut().people, &mut auth.engine.core_mut().people);
    let mut decision = Decision::decide(&mut domain.journal, &env.limits.journal);
    let row = forge::LimitHold {
        task,
        member: b"forge.entries".as_slice().into(),
        configured: 8,
        needed: 9,
        revision: 0,
        at: env.wall,
        order: 0,
        room: 1,
        set: forge::LimitSet::Operation,
    };
    limit_holds::apply_hold(&mut domain, &env, &mut decision, row, b"forge.entries is full".as_slice().into());
    crate::domain::close(&mut domain, &env, decision, &mut Queue::uncapped());
    let kept = domain.forge_limit_holds();
    assert_eq!(kept.len(), 1);
    assert!(kept[0].revision > 0);
    assert_eq!(
        read_forge(
            &mut domain,
            &env,
            READ_SIGN_IN,
            ApplicationScope::Deployment,
            crate::ForgeRead::LimitHolds { after: None }
        ),
        Delivery::ForgeRead {
            reply: READ_REPLY,
            answer: crate::ForgeReadAnswer::LimitHolds { rows: kept.clone(), next: None }
        }
    );
    assert_eq!(
        read_forge(
            &mut domain,
            &env,
            READ_SIGN_IN,
            ApplicationScope::Deployment,
            crate::ForgeRead::LimitHolds { after: Some(task) }
        ),
        Delivery::ForgeRead {
            reply: READ_REPLY,
            answer: crate::ForgeReadAnswer::LimitHolds { rows: Box::new([]), next: None }
        }
    );
    let (mut auth, _) = reader_in(Some(Role::Member), PROJECT + 1);
    ::core::mem::swap(&mut domain.engine.core_mut().people, &mut auth.engine.core_mut().people);
    assert_eq!(
        read_forge(
            &mut domain,
            &env,
            READ_SIGN_IN,
            ApplicationScope::Deployment,
            crate::ForgeRead::LimitHolds { after: None }
        ),
        Delivery::ForgeRead {
            reply: READ_REPLY,
            answer: crate::ForgeReadAnswer::LimitHolds { rows: Box::new([]), next: None }
        }
    );
}
