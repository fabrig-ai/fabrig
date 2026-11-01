//! Policy resize admission checks the open period's exact room before mutation.
pub(in crate::tests) mod fixtures;
pub(in crate::tests) mod periods;
mod settings;
use crate::{funding::PoolResize, tests::*};

#[test]
fn a_pool_raise_exactly_fills_the_period_and_one_unit_more_is_refused() {
    for budget in [1000, u64::MAX] {
        let (mut core, limits) = make_tree();
        send_task(
            &mut core,
            &limits,
            tasks::Event::ResizePeriod { reply_to: ReplyTo::new(Token::new(79)), project: 1, period: 1, budget },
        );
        send_task(
            &mut core,
            &limits,
            tasks::Event::CarvePool {
                reply_to: ReplyTo::new(Token::new(80)),
                project: 1,
                person: 1,
                period: 1,
                budget: 400,
            },
        );
        let period = tasks::Funder::Period { project: 1, period: 1 };
        let pool = tasks::Funder::Pool { project: 1, person: 1, period: 1 };
        let period_before = *core.tasks.funding(period).unwrap();
        let pool_before = *core.tasks.funding(pool).unwrap();
        assert_eq!(period_before.numbers.reserved, 500);
        let policy_before = core.authority.policy(1).unwrap().clone();
        let mut before = policy_before.clone();
        before.period_spend = budget;
        let mut after = before.clone();
        after.roles[0].period_spend = budget - 100;
        let plan = core.resize(1, &before, &after, &limits).unwrap();
        assert_eq!(plan.period, None);
        assert_eq!(plan.pools.as_slice(), &[PoolResize { person: 1, from: 400, to: budget - 100 }]);
        after.roles[0].period_spend = budget - 99;
        assert_eq!(
            core.resize(1, &before, &after, &limits),
            Err(people::Refusal::Authority {
                proposable: None,
                lacking: people::Lack::Budget { needs: budget - 499, left: budget - 500 }
            })
        );
        assert_eq!(core.tasks.funding(period), Some(&period_before));
        assert_eq!(core.tasks.funding(pool), Some(&pool_before));
        assert_eq!(core.authority.policy(1), Some(&policy_before));
    }
}

#[test]
fn a_role_raise_past_ceiling_and_room_names_ceiling_first() {
    let (mut core, limits) = make_tree();
    let period = tasks::Funder::Period { project: 1, period: 1 };
    let before = *core.tasks.funding(period).unwrap();
    assert_eq!(
        core.change_policy(&limits, 1, 1, people::PolicyChange::RoleSpend { role: 0, period_spend: 1001 }),
        Err(people::Refusal::Authority {
            proposable: None,
            lacking: people::Lack::Ceiling {
                holder: people::CeilingHolder::Policy,
                dimension: people::CeilingDimension::PeriodSpend,
                value: Some(1000),
                asked: Some(1001),
            }
        })
    );
    assert_eq!(core.tasks.funding(period), Some(&before));
}
