//! A delegate's provider requests after its owner's live controls change.
use super::{Script, VecDeque, World, engine, parties};
use temper_engine_domain_world::fixtures::{SECOND_LLM_ACCOUNT, add_second_account};

const RANK: super::Token = super::Token::new(973);

pub(super) fn ranked(to: super::Token, reply: &super::people::Reply) -> bool {
    if to != RANK {
        return false;
    }
    assert_eq!(*reply, super::people::Reply::Outcome(super::people::Outcome::AccountChanged));
    true
}

fn open_config(seed: u64) -> engine::Config {
    let mut config = config(seed);
    config.core.defaults.templates =
        temper_engine_domain_world::fixtures::templates_ending(super::tasks::EndedBy::Requester);
    config
}

/// One survey delegate, followed by its owner's wait.
#[must_use]
pub fn survey_delegation(model: u8) -> Script {
    let arguments = serde_json::to_vec(&serde_json::json!({"batch":[{
        "executor":{"kind":"agent","charter":1},
        "spec":{"words":"@system Survey the work","parameters":[]},
        "contract":{"kind":"report","words":128},"llm":{"main":{"model":model}},
        "authority":{"tools":2,"grants":[],"delegation":{"kinds":[],"tasks":0,"depth":0},"budget":{"spend":100},"notes":0}
    }]})).expect("survey fixture");
    Script {
        cue: b"@system".as_slice().into(),
        turns: Box::new([super::call(b"delegate", &arguments), super::call(b"wait", b"{}")]),
    }
}

fn config(seed: u64) -> engine::Config {
    let mut config = super::config(seed);
    let charter = &mut config.core.defaults.charters[0];
    let mut owner = charter.policy.model.clone();
    owner.name = b"owner".as_slice().into();
    charter.policy.alternatives = Box::new([owner.clone()]);
    config.core.settings.models = Box::new([charter.policy.model.clone(), owner]);
    config.core.defaults.rules.ceiling.delegation.depth = 2;
    config.core.seed_projects[0].policy.policy.ceiling.delegation.depth = 2;
    config.core.seed_projects[0].policy.policy.roles[0].authority.delegation.depth = 2;
    config
}

impl World {
    /// A parked owner with two usable deployment accounts before the person's rank change.
    #[must_use]
    pub fn ranking_chat(seed: u64, requester: Script, survey: Script) -> Self {
        let mut world = Self::configured_limits(seed, requester.clone(), false, open_config, |limits| {
            limits.tasks.depth = 2;
            temper_engine_domain_world::fixtures::two_account_limits(limits);
        });
        add_second_account(&mut world.store);
        world.scripts = VecDeque::from([requester, survey]);
        world
    }

    /// Move the second deployment account first through the person's authenticated act.
    pub fn rank_second_account(&mut self) {
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: super::ReplyTo::new(RANK),
            sign_in: self.party.sign_in.expect("authenticated holder"),
            key: [73; 16],
            ask: super::people::Ask::AccountChange {
                holder: super::people::AccountHolder::Deployment,
                change: super::people::AccountChange::MoveAccount { account: SECOND_LLM_ACCOUNT, place: 0 },
            },
        }));
        for _ in 0..10000 {
            self.tick();
            if !self.committing()
                && self.store.rows.values().any(|row| {
                    matches!(
                        row,
                        engine::Record::Core(jig_domain_core::Record::Accounts(jig_domain_accounts::Stored::Account {
                            number: SECOND_LLM_ACCOUNT,
                            place: 0,
                            ..
                        }))
                    )
                })
            {
                return;
            }
        }
        panic!("account rank did not commit");
    }

    /// Let the same owner delegate after the rank, without recreating its task.
    pub fn request_survey(&mut self) {
        let mut script = survey_delegation(0);
        let mut turns = script.turns.into_vec();
        turns.extend(super::text_script(b"Survey kept.").turns);
        script.turns = turns.into_boxed_slice();
        self.process.resume(script);
        self.act(
            parties::Ask::Say {
                task: parties::Task::Number(1),
                words: b"Delegate the survey.".as_slice().into(),
                release: None,
            },
            [74; 16],
        );
    }

    /// A quiet requester and one report delegate, placed in succession on one slot.
    #[must_use]
    pub fn delegating_chat(seed: u64, requester: Script, delegate: Script) -> Self {
        let mut world =
            Self::configured_limits(seed, requester.clone(), false, config, |limits| limits.tasks.depth = 2);
        world.scripts = VecDeque::from([requester, delegate]);
        world
    }

    /// Inspect the provider boundary of a current or parked task.
    #[must_use]
    pub fn task_agent(&self, task: u64) -> &super::Agent {
        if self.assignment().task == task {
            self.agent()
        } else {
            self.parked.get(&task).expect("task process").agent()
        }
    }

    /// Answer the delegate's question through the requester's person.
    pub fn answer_delegate_question(&mut self, words: &[u8]) {
        self.root_events.push_back(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: super::ReplyTo::new(super::Token::new(902)),
            sign_in: self.party.sign_in.expect("authenticated requester"),
            key: [72; 16],
            ask: super::people::Ask::AnswerQuestion { project: 1, task: 1, question: 1, words: words.into() },
        }));
    }

    /// Resume the requester with a control call, and supply the delegate's fresh model script.
    pub fn resume_requester(&mut self, script: Script, delegate: Script) {
        // A model change recaps the two committed assistant turns before opening
        // its new window. The replacement script supplies that summary and result.
        self.process.script = delegate.clone();
        self.process.fresh = Some(delegate);
        self.parked.get_mut(&1).expect("parked requester").resume(script);
        self.act(
            parties::Ask::Say {
                task: parties::Task::Number(1),
                words: b"Apply the new direction.".as_slice().into(),
                release: None,
            },
            [71; 16],
        );
    }
}
