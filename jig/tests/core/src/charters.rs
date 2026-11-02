//! Charter fixtures and agent actions shared by the core world's stories.

use crate::world::World;
use jig_domain_accounts as accounts;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_test_domain as root;
use skein_lib::{Duration, ReplyTo, Token};

impl World {
    /// Make this charter's account usable without changing another charter's grant.
    pub fn grant_account(&mut self, account: u32) {
        self.send(root::Event::Core(core::Event::Account(accounts::Event::Component(
            accounts::FromComponent::Granted {
                form: accounts::Form::Credential(accounts::Scheme::Bearer),
                account,
                generation: 1,
                valid: Duration::from_secs(60),
            },
        ))));
    }

    /// Delegate one agent task from the currently assigned chat.
    pub fn delegate_agent(&mut self, charter: u32, words: &[u8]) {
        let mut authority = self.assigned.as_ref().expect("assigned caller").run.authority.clone();
        authority.delegation = tasks::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
        authority.budget.spend = 20;
        self.send(root::Event::Core(core::Event::DelegateValidated {
            to: ReplyTo::new(Token::new(201)),
            key: self.call_key(1),
            batch: Box::new([core::Delegate {
                deadline: jig_domain_tasks::Deadline::Named,
                label: 0,
                flags: jig_domain_tasks::Flags::PLAIN,
                tracked: None,
                budget: jig_domain_people::Budget::Named,
                executor: tasks::Executor::Agent { charter },
                spec: tasks::Spec {
                    job: None,
                    asks: false,
                    words: words.into(),
                    parameters: Box::new([]),
                    inputs: Box::new([]),
                },
                contract: tasks::Contract::Report { words: 128 },
                authority,
                llm: None,
                symbolic_grants: Box::new([]),
                dependencies: Box::new([]),
                wake: tasks::WakePolicy::DEFAULT,
            }]),
            stubs: Box::new([]),
        }));
    }
}
