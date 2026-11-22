//! Account holds driven by the provider's answer, across both roots and Smith's wire.
use super::{
    Duration, Finish, Key, Line, ReplyTo, Script, Token, Turn, VecDeque, World, call, engine, people, tasks, worker,
};
use jig_domain_accounts as accounts;

fn configuration(seed: u64) -> engine::Config {
    let mut config = super::config(seed);
    config.core.defaults.rules.ceiling.delegation.depth = 3;
    let policy = &mut config.core.seed_projects[0].policy.policy;
    policy.ceiling.delegation.depth = 3;
    for role in &mut policy.roles {
        role.authority.delegation.depth = 3;
    }
    config
}

impl World {
    /// Delegate one report, whose first provider query meets the supplied account failure.
    #[must_use]
    pub fn provider_hold(seed: u64, error: skein_fake_llm_domain::api::Error) -> Self {
        let director = Script {
            cue: b"@system".as_slice().into(),
            turns: Box::new([
                call(b"delegate", br#"{"batch":[{"executor":{"kind":"agent","charter":1},"spec":{"words":"@held Retain this specification after the account resumes."},"contract":{"kind":"report","words":128},"authority":{"tools":0,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":20},"notes":0}}]}"#),
                call(b"wait", b"{}"),
                call(b"finish", br#"{"report":"The held work reported."}"#),
            ]),
        };
        let child = Script {
            cue: b"@held".as_slice().into(),
            turns: Box::new([Turn {
                lines: Box::new([
                    Line::Fail {
                        error,
                        report: skein_fake_llm_domain::api::Report::none(),
                        credentials: Box::new([0]),
                    },
                    Line::Call {
                        name: b"finish".as_slice().into(),
                        arguments: br#"{"outcome":{"form":"report","text":"The held work reported.","fields":{}}}"#
                            .as_slice()
                            .into(),
                    },
                ]),
                finish: Finish::ToolCalls,
                tokens: 20,
            }]),
        };
        let mut world =
            Self::configured_limits(seed, director.clone(), false, configuration, |limits| limits.tasks.depth = 3);
        world.scripts = VecDeque::from([director, child]);
        world.account_generation = Some(1);
        world.worker_env.limits.reply_bytes = 8192;
        world.worker = worker::Domain::new(&world.worker_env.limits, Box::new([]));
        world
    }

    /// Wait for the failed attempt and its durable account hold without acting as a person.
    pub fn until_account_held(&mut self) {
        for _ in 0..10000 {
            self.tick();
            if self.root.tasks().task(2).is_some_and(|task| task.phase == tasks::Phase::Active(tasks::Active::Due))
                && self.observed.gone >= 2
                && self.store.rows.values().any(|row| {
                    matches!(
                        row,
                        engine::Record::Core(jig_domain_core::Record::Accounts(accounts::Stored::Source {
                            held: Some(_),
                            ..
                        }))
                    )
                })
                && !self.committing()
            {
                return;
            }
        }
        panic!(
            "provider hold did not settle: assignments={}, queries={}, answer={:?}",
            self.observed.assignments,
            self.observed.queries.len(),
            self.agent().answer()
        );
    }

    /// The authenticated engine input sent by ctl's account replace-key command.
    pub fn replace_provider_key(&mut self) {
        self.account_generation = Some(2);
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Key {
            carried: jig_domain_core::Carried { number: 1_u64 << 63, digest: [7; 32] },
            reply_to: ReplyTo::new(Token::new(903)),
            sign_in: self.party.sign_in.expect("signed in"),
            key: [45; 16],
            ask: Box::new(people::Ask::AccountChange {
                holder: people::AccountHolder::Deployment,
                change: people::AccountChange::ReplaceKey { source: 1, digest: [7; 32] },
            }),
        }));
    }

    /// Read an account or task document through authenticated engine input.
    pub fn provider_document(&mut self, read: jig_domain_core::reads::Read) -> jig_domain_core::documents::Document {
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Read {
            reply_to: ReplyTo::new(Token::new(904)),
            sign_in: self.party.sign_in.expect("signed in"),
            read: Box::new(read),
        }));
        for _ in 0..10000 {
            self.tick();
            if let Some(answer) = self.account_read.take() {
                let jig_domain_core::reads::Answer::Document(document) = answer else { panic!("account document") };
                return *document;
            }
        }
        panic!("account document did not arrive");
    }

    /// Drive until both the original child and its director have durable results.
    pub fn finish_provider_work(&mut self) {
        for _ in 0..10000 {
            self.tick();
            if [1, 2].iter().all(|task| {
                self.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(*task))))
            }) && !self.committing()
            {
                return;
            }
        }
        panic!(
            "provider work did not finish: assignments={}, queries={}, answer={:?}",
            self.observed.assignments,
            self.observed.queries.len(),
            self.agent().answer()
        );
    }

    pub(super) fn provider_account(&mut self, request: &engine::Request) -> Option<Vec<engine::Event>> {
        let generation = self.account_generation?;
        let engine::Request::Account(request) = request else { return None };
        let event = match request {
            accounts::Request::Component(accounts::ToComponent::Rejected { account, .. }) => {
                accounts::FromComponent::Failed { account: *account, failure: accounts::Failure::SignInAgain }
            }
            accounts::Request::Component(accounts::ToComponent::Want { account }) => accounts::FromComponent::Granted {
                account: *account,
                generation,
                valid: Duration::from_secs(3600),
                form: jig_domain_link::Form::Credential(jig_domain_link::Scheme::Key),
            },
            accounts::Request::Signing(accounts::Signing::Keep { source, .. }) => accounts::FromComponent::Granted {
                account: *source,
                generation,
                valid: Duration::from_secs(3600),
                form: jig_domain_link::Form::Credential(jig_domain_link::Scheme::Key),
            },
            _ => return None,
        };
        self.protocol.accounts.push(request.clone());
        Some(vec![engine::Event::Account(accounts::Event::Component(event))])
    }
}
