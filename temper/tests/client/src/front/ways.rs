//! Stock ways at the client/engine boundary (temper/docs/design/client/README.md, 5.13).
use super::{Joint, client, core_rows, engine, engine_limits, fixtures};
use jig_domain_authority as authority;
use jig_domain_core::settings as s;
use jig_domain_people as people;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain_world::{direct::Driver, kinds, oracle, store::Store};

#[derive(Clone, Copy)]
pub enum Reader {
    Owner,
    BoxOwner,
    Member,
}

/// The stock's kinds and draft goal, with the world's short agent charters.
#[must_use]
#[expect(clippy::too_many_lines, reason = "one fixture joins the stock, policy and reader identity")]
pub fn fixture(seed: u64, limits: &client::Limits, reader: Reader) -> Joint {
    let mut engine_limits = engine_limits();
    engine_limits.tasks.charters = 5;
    engine_limits.people.amendment_bytes = 8192;
    engine_limits.people.requests = 12;
    engine_limits.people.people = 6; // Box owner, forge collaborators and the deployment service.
    engine_limits.people.initial_owners = 2;
    engine_limits.people.holdings = 4;
    engine_limits.authority.executors = 4;
    engine_limits.tasks.executor_kinds = 4;
    engine_limits.documents.executors = 4;
    engine_limits.authority.roles = 2;
    let mut config = kinds::configuration(&engine_limits);
    config.core.seed = seed;
    config.core.connectors = Box::new([fixtures::FORGE]);
    config.core.kinds = temper_service::stock::kinds(&engine_limits, fixtures::FORGE);
    let models = config
        .core
        .settings
        .models
        .iter()
        .enumerate()
        .map(|(at, model)| s::CatalogueModel {
            first_read: None,
            number: u32::try_from(at).expect("fixture models") + 1,
            model: model.clone(),
            shown: None,
            description: None,
            picker: true,
            withdrawn: false,
            source: s::ModelSource::Stock,
            estimated: false,
        })
        .collect();
    let stock = temper_service::stock::with_models(models, engine_limits.tasks.result_bytes);
    let mut templates = config.core.defaults.templates.into_vec();
    templates.push(stock.templates.iter().find(|template| template.number == 4).expect("stock draft goal").clone());
    config.core.defaults.templates = templates.into_boxed_slice();
    let mut charters = config.core.defaults.charters.into_vec();
    let mut coordinator = charters[0].clone();
    coordinator.number = 5;
    charters.push(coordinator);
    config.core.defaults.charters = charters.into_boxed_slice();
    config.core.defaults.rules.ceiling.delegation.kinds = Box::new([
        authority::Executor::Charter(1),
        authority::Executor::Charter(5),
        authority::Executor::Role(0),
        authority::Executor::Job(3),
    ]);
    let project = &mut config.core.seed_projects[0].policy;
    project.permissions = Box::new([]);
    let mut member = project.policy.roles[0].clone();
    member.number = people::Role::Member.number();
    project.policy.roles = Box::new([project.policy.roles[0].clone(), member]);
    let identity = people::Identity {
        key: match reader {
            Reader::BoxOwner => people::IdentityKey { provider: 3, subject: b"box-owner".as_slice().into() },
            Reader::Owner | Reader::Member => temper_engine_domain_forge::user_key(1, 7),
        },
        login: b"person".as_slice().into(),
        name: b"Person".as_slice().into(),
    };
    let owner = match reader {
        Reader::Owner | Reader::BoxOwner => identity.key.clone(),
        Reader::Member => temper_engine_domain_forge::user_key(1, 8),
    };
    config.core.owners = Box::new([
        people::InitialOwner { project: 0, identity: owner.clone() },
        people::InitialOwner { project: 1, identity: owner },
    ]);
    let bootstrap = oracle::config::copy(&config);
    let accounts = temper_engine_domain_world::fixtures::account_rows(&config.core.settings.llm_providers);
    let mut cold = Driver::cold(Store::with_limits(&engine_limits), bootstrap, &engine_limits);
    cold.settle();
    let mut store = cold.take_store();
    store.seed(accounts, Box::new([]));
    let component = crate::component::Composition::with_provider(&config, limits.client.domain, identity.key.provider);
    let restart = oracle::config::copy(&config);
    let mut joint = Joint::with_engine(seed, limits, store, [config, restart], &engine_limits, 1);
    joint.component = component;
    joint.sign_in_identity = Some(identity.clone());
    if matches!(reader, Reader::Member) {
        joint.caller = fixtures::asking_member(&mut joint.engine);
    } else {
        joint.engine.send(engine::Event::Party(engine::PartyInput::SignedIn {
            held: None,
            reply_to: ReplyTo::new(Token::new(700)),
            identity,
        }));
        joint.engine.settle();
    }
    if matches!(reader, Reader::BoxOwner) {
        let mut adoption = temper_engine_domain_world::fixtures::adoption(1);
        adoption.owner = b"acme".as_slice().into();
        adoption.name = b"shop".as_slice().into();
        joint.engine.send(engine::Event::Party(engine::PartyInput::Application {
            scope: people::ApplicationScope::Project(1),
            reply_to: ReplyTo::new(Token::new(701)),
            sign_in: joint.engine.session(),
            key: [71; 16],
            digest: [71; 32],
            carried: None,
            request: engine::Application::Forge(temper_engine_domain_forge::Ask::Adopt(adoption)),
        }));
        joint.engine.settle();
    }
    joint.last_commit = joint.engine.store.header().commits;
    joint.truth = jig_client_kit::joint::truth::Truth::after(core_rows(&joint.engine));
    joint
}
