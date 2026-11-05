//! Away and a goal's host call share the roots (temper/docs/design/domain/testing.md, 6.3).
use super::{Joint, client, engine_limits, fixtures, walking};
use jig_domain_people as people;
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain as engine;
use temper_engine_domain_world::store::Store;

impl Joint {
    /// Sessions span the next 08:00 on the world's clock; goals remain offered.
    #[must_use]
    pub fn with_away_people(seed: u64, limits: client::Limits) -> Self {
        let mut bounds = engine_limits();
        bounds.people.sign_in_lifetime = Duration::from_secs(24 * 60 * 60);
        // The goal, membership, two Home entries and switch acts retain distinct replies.
        bounds.people.requests = 16;
        bounds.authority.roles = 2;
        bounds.documents.executors = 2;
        let configs = std::array::from_fn(|_| {
            let mut config = walking::config(seed);
            config.core.connectors = Box::new([fixtures::FORGE]);
            let policy = &mut config.core.seed_projects[0].policy.policy;
            let owner = policy.roles[0].clone();
            let mut member = owner.clone();
            member.number = people::Role::Member.number();
            member.decides = jig_domain_authority::Proposals(0);
            policy.roles = Box::new([owner, member]);
            config
        });
        Self::with_engine(seed, &limits, Store::new(), configs, &bounds, 1)
    }

    /// A tracked goal requested by the signed-in person, with the control tools.
    pub fn begin_away_goal(&mut self) -> u64 {
        self.answer_runs = false;
        self.engine.sign_in();
        self.engine.settle();
        let people::Ask::Create { project, mut batch } = self.engine.chat_ask(1, b"Make a small fix") else {
            panic!("goal template")
        };
        batch[0].tracked = Some(0);
        batch[0].flags.conversation = false;
        batch[0].authority.tools = 1 | 2 | 4 | 512;
        self.engine.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(2301)),
            sign_in: self.engine.session(),
            key: [83; 16],
            ask: people::Ask::Create { project, batch },
        }));
        self.drain();
        let task = self.assignments.last().expect("goal assigned").task;
        assert!(self.engine.task(task).expect("live goal").tracked.is_some());
        task
    }
}
