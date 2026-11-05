//! Distinct people in the dismissal stories (temper/docs/design/domain/testing.md, 2.1).
use super::{Joint, client, engine_limits, fixtures, translate};
use jig_client_kit::harness::{ClientId, Observed};
use jig_domain_people as people;
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain as engine;
use temper_engine_domain_world::{direct::Driver, store::Store, walking};

const ROSTER_REQUEST: Token = Token::new(2101);
const ROSTER_KEY: [u8; 16] = [81; 16];

impl Joint {
    /// Two seeded projects owned by one person, with room for three devices.
    #[must_use]
    pub fn with_to_begin_people(seed: u64, limits: client::Limits) -> Self {
        let mut bounds = engine_limits();
        bounds.people.sign_ins = 4;
        bounds.people.initial_owners = 2;
        bounds.authority.projects = 2;
        bounds.people.projects = 3;
        let configs = configurations(seed).map(|mut config| {
            let policy = config.core.seed_projects.first().expect("seed project").policy.clone();
            let mut projects = config.core.seed_projects.into_vec();
            projects.push(jig_domain_core::SeedProject { number: 2, name: "Project 2".into(), policy });
            config.core.seed_projects = projects.into_boxed_slice();
            let mut owners = config.core.owners.into_vec();
            owners.push(people::InitialOwner {
                project: 2,
                identity: owners.first().expect("seed owner").identity.clone(),
            });
            config.core.owners = owners.into_boxed_slice();
            config
        });
        Self::with_engine(seed, &limits, Store::new(), configs, &bounds, 1)
    }

    /// Wire sends of project 1's Home skip, including any duplicate attempt.
    #[must_use]
    pub fn dismissal_sends(&self) -> usize {
        self.dismissal_sends
    }

    pub(super) fn count_dismissal(&mut self, request: &client::Request) {
        if let client::Request::Unlinked {
            request: jig_client::UnlinkedRequest::Down { down: jig_client::Down::Send { request, .. } },
        } = request
            && Some(request)
                == client::forge::dismissed::ask(
                    client::forge::dismissed::Dismissal::ToBegin { project: 1 },
                    &self.limits.forge,
                )
                .as_ref()
        {
            self.dismissal_sends += 1;
        }
    }

    /// The joined stories speak the configured FORGE connector's wire number.
    #[must_use]
    pub fn with_dismissal_people(seed: u64, limits: client::Limits) -> Self {
        Self::with_engine(seed, &limits, Store::new(), configurations(seed), &engine_limits(), 1)
    }

    /// Restart from the committed store with the fixture's connector configuration.
    /// The offline interval and watch loss follow `FrontFault::Restart`.
    pub fn restart_dismissal_people(&mut self, offline: Duration) {
        let limits = self.engine.env.limits;
        let store = self.engine.take_store();
        self.engine = Driver::with_repositories(store, configurations(self.seed), &limits, fixtures::REPOSITORIES);
        self.engine.pump();
        self.engine.send(temper_engine_domain_world::protocol::hello_slots(
            fixtures::QUESTION_WORKER,
            limits.fleet.slots,
            Box::new([]),
        ));
        self.engine.settle();
        self.reads = 0;
        self.read_loads.clear();
        self.offline_until = Some(self.now.saturating_add(offline));
        self.observed.push(Observed::Reachable { live: false });
        self.drop_watches();
    }

    /// Bind a fresh person's real identity without changing the default caller.
    pub fn bind_dismissal_person(&mut self, who: ClientId, party: u64, subject: u64) {
        self.client_limits.insert(who, self.limits);
        self.component.receiving(who, self.limits.client.domain);
        let identity = people::Identity {
            key: people::IdentityKey { provider: 0, subject: subject.to_be_bytes().into() },
            login: b"other".as_slice().into(),
            name: b"Other".as_slice().into(),
        };
        self.transport.bind(who, party, identity.clone());
        self.callbacks.push((
            who,
            translate::callback(jig_client_test_domain::Event::Client {
                event: jig_client::Event::Up { up: self.transport.hello() },
            })
            .into(),
        ));
        self.component.sign_in(who, identity);
        self.pump_component();
    }

    /// Give the second person a member's role through the owner's real request.
    pub fn add_dismissal_member(&mut self, owner: ClientId, party: u64, project: u32) {
        self.engine.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(ROSTER_REQUEST),
            sign_in: self.transport.sign_in(owner).expect("signed-in owner"),
            key: ROSTER_KEY,
            ask: people::Ask::SetRoles {
                project,
                holdings: Box::new([
                    people::Holding { person: super::fixtures::PARTY, role: people::Role::Owner },
                    people::Holding { person: party, role: people::Role::Member },
                ]),
            },
        }));
        self.engine.settle();
        assert!(
            self.engine.delivered.iter().any(|delivery| matches!(delivery,
                engine::Delivery::WebReply {
                    to, reply: people::Reply::Outcome(people::Outcome::RolesSet { project: held, .. }),
                } if *to == ReplyTo::new(ROSTER_REQUEST) && *held == project
            )),
            "the second person is a project member"
        );
    }
}

fn configurations(seed: u64) -> [engine::Config; 2] {
    std::array::from_fn(|_| {
        let mut config = walking::config(seed);
        config.core.connectors = Box::new([fixtures::FORGE]);
        // These three-device stories use no goals; reserve the fixed engine watch table for dismissal facts.
        let mut templates = config.core.defaults.templates.into_vec();
        templates.retain(|template| template.number != jig_client::shapes::GOAL);
        config.core.defaults.templates = templates.into_boxed_slice();
        config
    })
}
