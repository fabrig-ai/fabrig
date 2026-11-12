//! Stock ways at the client/engine boundary (temper/docs/design/client/pages/settings.md, 2).
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

#[derive(Clone, Copy)]
pub enum Repository {
    CiAndTiers,
    Neither,
    Yours,
}

/// The stock's kinds and draft goal, with the world's short agent charters.
#[must_use]
pub fn fixture(seed: u64, limits: &client::Limits, reader: Reader) -> Joint {
    with_repository(seed, limits, reader, None)
}

/// Keep existing callers' repositories; path stories adopt CI and both tiers or neither.
#[must_use]
pub fn with_repository(seed: u64, limits: &client::Limits, reader: Reader, repository: Option<Repository>) -> Joint {
    configured(seed, limits, reader, repository, false)
}
/// A follow's target and a separately owned source project (client/pages/settings.md, 2).
#[must_use]
pub fn follows(seed: u64, limits: &client::Limits) -> Joint {
    configured(seed, limits, Reader::Owner, None, true)
}
#[expect(clippy::too_many_lines, reason = "the fixture joins stock, policy, projects and reader identity")]
fn configured(seed: u64, limits: &client::Limits, reader: Reader, repository: Option<Repository>, projects: bool) -> Joint {
    let mut engine_limits = engine_limits();
    engine_limits.tasks.charters = 13; // Kind 9's stock lines run up to charter 13.
    engine_limits.tasks.spec_bytes = engine_limits.tasks.spec_bytes.max(512);
    engine_limits.tasks.parameters = 4;
    engine_limits.people.amendment_bytes = 8192;
    engine_limits.people.requests = 12;
    engine_limits.people.people = 6; // Box owner, forge collaborators and the deployment service.
    engine_limits.people.initial_owners = 2;
    engine_limits.people.holdings = 4;
    // Draft first adds the closer procedure and close job to four executor places.
    engine_limits.authority.executors = 6;
    // Draft first adds the closer procedure and close job to four executor places.
    engine_limits.tasks.executor_kinds = 6;
    // Draft first adds the closer procedure and close job to four executor places.
    engine_limits.documents.executors = 6;
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
        authority::Executor::Procedure(6),
        authority::Executor::Job(4),
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
    if projects { super::first_run_projects(&mut config, &mut engine_limits); }
    let bootstrap = oracle::config::copy(&config);
    let accounts = temper_engine_domain_world::fixtures::account_rows(&config.core.settings.llm_providers);
    let mut cold = Driver::cold(Store::with_limits(&engine_limits), bootstrap, &engine_limits);
    cold.settle();
    let mut store = cold.take_store();
    store.seed(accounts, Box::new([]));
    let component = crate::component::Composition::with_provider(&config, limits.client.domain, identity.key.provider);
    let restart = oracle::config::copy(&config);
    let repository_config = oracle::config::copy(&config);
    let mut joint = Joint::with_engine(seed, limits, store, [config, restart], &engine_limits, 1);
    joint.component = component;
    joint.sign_in_identity = Some(identity.clone());
    if matches!(reader, Reader::Member) {
        joint.caller = fixtures::asking_member(&mut joint.engine);
    } else {
        joint.engine.send(engine::Event::Party(engine::PartyInput::SignedIn {
            held: None,
            reply_to: ReplyTo::new(Token::new(700)),
            identity: identity.clone(),
        }));
        joint.engine.settle();
    }
    if matches!(reader, Reader::BoxOwner) || repository.is_some() {
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
    if let Some(repository) = repository {
        let mut store = joint.engine.take_store();
        if matches!(repository, Repository::Neither) {
            let saved = store
                .rows
                .values()
                .find_map(|record| match record {
                    engine::Record::Forge { row } => match row.as_ref() {
                        temper_engine_domain_forge::Stored::Repository(row) => {
                            let mut row = row.clone();
                            row.ci = false;
                            Some(engine::Record::Forge {
                                row: Box::new(temper_engine_domain_forge::Stored::Repository(row)),
                            })
                        }
                        _ => None,
                    },
                    engine::Record::Core(_) => None,
                })
                .expect("adopted repository");
            store.seed(Box::new([saved]), Box::new([]));
        }
        let mut config = repository_config;
        if matches!(repository, Repository::CiAndTiers | Repository::Yours) {
            let tiers = [
                (b"slow".as_slice(), temper_engine_domain_forge::Follows::Landing),
                (b"goal".as_slice(), temper_engine_domain_forge::Follows::Goals),
            ]
            .map(|(name, follows)| engine::TierCheck {
                name: name.into(),
                workflow: name.into(),
                ignored: Box::new([]),
                artifact: b"tests".as_slice().into(),
                format: temper_engine_domain_forge_tiers::Format::Lines,
                follows,
                span_ms: 0,
                description: Box::default(),
            });
            assert!(
                config
                    .landings
                    .insert(
                        1,
                        engine::ProjectLanding {
                            check_timeout: None,
                            by: engine::Way::Queue,
                            close_job: None,
                            merge_job: None,
                            repositories: Box::new([engine::RepositoryChecks {
                                repository: skein_forge::api::Repository { forge: fixtures::FORGE, repository: 7 },
                                gates: Box::new([]),
                                tiers: Box::new(tiers),
                            }]),
                            after: engine::AfterLandingCheck { tier_retries: 0, verification_bound_ms: 7_200_000 },
                        }
                    )
                    .is_ok()
            );
        }
        let restart = oracle::config::copy(&config);
        let component =
            crate::component::Composition::with_provider(&config, limits.client.domain, identity.key.provider);
        joint = Joint::with_engine(seed, limits, store, [config, restart], &engine_limits, 1);
        joint.component = component;
        joint.sign_in_identity = Some(identity.clone());
        if matches!(reader, Reader::Member) {
            joint.caller = fixtures::asking_member(&mut joint.engine);
        } else {
            joint.engine.send(engine::Event::Party(engine::PartyInput::SignedIn {
                held: None,
                reply_to: ReplyTo::new(Token::new(702)),
                identity,
            }));
            joint.engine.settle();
        }
        if matches!(repository, Repository::Yours) {
            use temper_engine_domain_forge::{self as forge, settings};
            let own = settings::LandingRule {
                sets: Box::new([]),
                pattern: forge::GatePattern {
                    segments: Box::new([b"branch".as_slice().into()]),
                    last: forge::GateLast::Open(Box::new([])),
                },
                enforce: true,
                ci: false,
                up_to_date: true,
                gates: Box::new([settings::RuleGate {
                    number: 1,
                    blocking: true,
                    freshness: forge::JudgeFreshness::Clean,
                }]),
                approvals: Box::new([]),
            };
            joint.engine.send(kinds::change(
                joint.engine.session(),
                kinds::revision(&joint.engine.store.rows),
                72,
                Box::new([kinds::item(2, 1, 1, &settings::Value::Rule(own))]),
            ));
            joint.engine.settle();
        }
        joint.engine.send(engine::Event::Credential {
            number: 1,
            generation: 1,
            valid: skein_lib::Duration::from_secs(3600),
        });
        joint.engine.settle();
    }
    joint.last_commit = joint.engine.store.header().commits;
    joint.truth = jig_client_kit::joint::truth::Truth::after(core_rows(&joint.engine));
    joint
}
