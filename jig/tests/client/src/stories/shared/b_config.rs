//! Front construction for seeded person stories (jig/docs/design/client/domain/testing.md, 4–5).
use super::b_seed::Facts;
use jig_fake_workers::{Call, Script, Worker};

#[must_use]
pub fn setup(
    seed: u64,
    facts: &Facts,
    mut limits: jig_test_domain::Limits,
) -> (jig_test_domain::Config, jig_test_domain::Limits, jig_core_world::peers::Peers) {
    let mut configuration = jig_test_profile::config(seed);
    let mut scripts = vec![vec![Script::Wait].into_boxed_slice()];
    match facts {
        Facts::NoChatTemplate => {
            configuration.core.defaults.templates = configuration
                .core
                .defaults
                .templates
                .into_vec()
                .into_iter()
                .filter(|template| template.number != jig_client::shapes::CHAT)
                .collect();
        }
        Facts::GoalOffer { procedure } => {
            let policy = &mut configuration.core.seed_projects[0].policy.policy;
            for role in &mut policy.roles {
                role.authority.budget.spend = 1;
                role.period_spend = 1;
            }
            if *procedure {
                let kinds: Box<[jig_domain_authority::Executor]> =
                    Box::new([jig_domain_authority::Executor::Procedure(7)]);
                policy.ceiling.delegation.kinds.clone_from(&kinds);
                for role in &mut policy.roles {
                    role.authority.delegation.kinds.clone_from(&kinds);
                }
                let mut rules = configuration.core.defaults.rules.clone();
                rules.ceiling.delegation.kinds = kinds;
                configuration.core.defaults.rules = jig_domain_authority::Domain::new(rules, limits.core.authority)
                    .expect("procedure policy")
                    .rules()
                    .clone();
                let mut procedure = configuration.first.procedures[0].clone();
                procedure.number = 7;
                configuration.second.procedures = Box::new([procedure]);
            }
        }
        Facts::ProposedGoal | Facts::Delegates => {
            configuration.core.seed_projects[0].policy.policy.roles[0].authority.delegation.tasks = 3;
            configuration.core.defaults.templates[0].budget = jig_domain_people::TemplateBudget::Amount(10);
            let mut rules = configuration.core.defaults.rules.clone();
            rules.maximum_run_spend = 5;
            let mut authority =
                jig_domain_authority::Domain::new(rules, limits.core.authority).expect("conversation run allowance");
            let mut out = skein_lib::Queue::uncapped();
            jig_domain_authority::step(
                &mut authority,
                jig_domain_authority::Event::Policy {
                    project: 1,
                    policy: configuration.core.seed_projects[0].policy.policy.clone(),
                },
                &mut out,
            );
            assert_eq!(out.pop(), Some(jig_domain_authority::PolicyFact::Added { project: 1 }));
            configuration.core.seed_projects[0].policy.policy = authority.policy(1).expect("seed policy").clone();
            configuration.core.defaults.rules = authority.rules().clone();
            limits.core.people.sign_ins = 4;
            limits.core.people.operations = 3;
            limits.core.tasks.tasks = 3;
            limits.core.tasks.project_tasks = 3;
            limits.core.tasks.tree_tasks = 3;
            limits.core.tasks.delegates = 2;
            limits.core.fleet.calls = 2;
            limits.core.tasks.inbox_messages = 8;
            limits.core.tasks.inbox_bytes = 512;
            scripts = vec![match facts {
                Facts::ProposedGoal => vec![
                    Script::Call(Call::ProposeGoal {
                        words: b"Resolve incident".as_slice().into(),
                        budget: 2,
                        priority: 0,
                    }),
                    Script::Wait,
                ]
                .into_boxed_slice(),
                Facts::Delegates => vec![
                    Script::Call(Call::Delegate { words: b"Cancelled delegate".as_slice().into(), budget: 1 }),
                    Script::Call(Call::Delegate { words: b"Moved delegate".as_slice().into(), budget: 1 }),
                    Script::Wait,
                ]
                .into_boxed_slice(),
                Facts::Goals
                | Facts::HeldConversation
                | Facts::FamilyResource
                | Facts::NoChatTemplate
                | Facts::GoalOffer { .. } => {
                    unreachable!("conversation scenarios")
                }
            }];
        }
        Facts::Goals | Facts::HeldConversation | Facts::FamilyResource => {}
    }
    let peers = jig_core_world::peers::Peers::new(vec![Worker::new(1, 1, scripts)], Vec::new());
    (configuration, limits, peers)
}
