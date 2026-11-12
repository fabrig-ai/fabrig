//! Away and a goal's host call share the roots (temper/docs/design/testing/checks.md, 3).
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

impl Joint {
    /// Away's Home sequence uses the stock ways setting and a goal's report.
    #[must_use]
    pub fn with_away_sequence(seed: u64, limits: client::Limits) -> Self {
        use temper_engine_domain_world::{direct::Driver, kinds};
        let mut bounds = engine_limits();
        bounds.people.requests = 16;
        bounds.documents.executors = 2;
        bounds.tasks.charters = kinds::limits().tasks.charters;
        bounds.people.amendment_bytes = kinds::limits().people.amendment_bytes;
        bounds.tasks.depth = 3;
        bounds.tasks.tree_tasks = 3;
        bounds.fleet.slots = 2;
        // The chat's goal call and the goal's decision, note and question calls coexist.
        bounds.call_records = 8;
        // Retained goal news, the overturn and the next question's answer credit coexist until its turn commits.
        bounds.tasks.inbox_bytes = bounds.tasks.message_bytes.checked_mul(3).expect("news, message and answer fit");
        // The chat retains both questions and their answer notices beside its goal's result credit.
        bounds.tasks.inbox_messages = 6;
        bounds.people.inbox_recipients = 2 * bounds.tasks.inbox_messages + bounds.authority.roles + 4;
        let [bootstrap, config, restart] = std::array::from_fn(|_| {
            let mut config = walking::config(seed);
            config.core.connectors = Box::new([fixtures::FORGE]);
            let kinds = kinds::configuration(&bounds);
            config.core.kinds = temper_engine_service::stock::kinds(&bounds, fixtures::FORGE);
            config.core.defaults.charters = kinds.core.defaults.charters;
            config.kinds_stock = kinds.kinds_stock;
            config.core.defaults.rules.ceiling.notes = jig_domain_authority::Scopes::GOAL;
            config.core.defaults.rules.ceiling.delegation.depth = 3;
            config.core.defaults.rules.ceiling.delegation.tasks = 3;
            let policy = &mut config.core.seed_projects[0].policy.policy;
            policy.ceiling.notes = jig_domain_authority::Scopes::GOAL;
            policy.roles[0].authority.notes = jig_domain_authority::Scopes::GOAL;
            policy.ceiling.delegation.depth = 3;
            policy.roles[0].authority.delegation.depth = 3;
            policy.ceiling.delegation.tasks = 3;
            policy.roles[0].authority.delegation.tasks = 3;
            config.core.defaults.templates[0].contract = jig_domain_tasks::Contract::Report { words: 128 };
            config
        });
        let forges = super::restored_forges(Store::new(), walking::config(seed), &config, &bounds);
        let mut records =
            temper_engine_domain_world::fixtures::account_rows(&config.core.settings.llm_providers).into_vec();
        records.extend(forges.rows.into_values().filter(|row| matches!(row, engine::Record::Forge { .. })));
        let mut driver = Driver::cold(Store::with_limits(&bounds), bootstrap, &bounds);
        driver.settle();
        let mut store = driver.take_store();
        store.seed(records.into_boxed_slice(), Box::new([]));
        let mut front = Self::with_engine(seed, &limits, store, [config, restart], &bounds, 2);
        front.engine.sign_in();
        front.engine.settle();
        let ways =
            temper_engine_domain_forge::settings::Value::Ways(temper_engine_domain_forge::settings::ProjectWays {
                drafts: false,
                goals: true,
                changes: false,
                way: 0,
            });
        front.engine.send(kinds::change(
            front.engine.session(),
            kinds::revision(&front.engine.store.rows),
            82,
            Box::new([kinds::item(7, 1, 1, &ways)]),
        ));
        front.engine.settle();
        assert_eq!(kinds::revision(&front.engine.store.rows), 2, "Away's project ways accepted");
        front
    }
}
