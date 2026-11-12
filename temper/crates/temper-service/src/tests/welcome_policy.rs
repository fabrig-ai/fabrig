//! Real init bytes through startup parsing and the root's project/adoption decisions
//! (`temper/docs/design/protocol/configuration/stock.md`).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    reason = "ordinary regression code retains whole snapshots and prints its offline measurement"
)]
use super::{application, config, parse};
use crate::{settings, tests::fixtures};
use ::core::{num::NonZeroU16, str};
use jig_domain_core as core;
use jig_domain_people as people;
use skein_lib::{List, ReplyTo, Secret, Token};
use temper_engine_domain::{self as engine, Delivery, Event, Key, PartyInput, Record};
use temper_engine_domain_forge as forge;
use temper_engine_domain_world::{direct::Driver, forge_stand::ForgeStand, store::Store};

fn generated_files() -> temper_init::Answer {
    let port = |value| NonZeroU16::new(value).expect("nonzero synthetic port");
    temper_init::files(
        &temper_init::Facts {
            sign_in: false,
            binary: b"/bin/temper",
            git: Some(b"/bin/git"),
            claude: None,
            login: Some(b"ada"),
            ports: temper_init::Ports {
                clients: port(8367),
                owned: port(8368),
                forge: port(8369),
                llm: port(8370),
                webhooks: port(8371),
            },
        },
        &temper_init::Directories { config: b"/config", data: b"/data" },
        &[0; 32],
        &[1; 32],
    )
    .expect("actual init producer")
}

fn generated() -> (engine::Config, engine::Limits) {
    let mut files = settings::Files { entries: List::with_capacity(8) };
    let mut document = None;
    for file in generated_files().files {
        if let temper_init::Content::Bytes { bytes, .. } = file.content {
            if file.name == b"temper.json" {
                document = Some(bytes.clone());
            }
            let mut path = match file.directory {
                temper_init::Directory::Config => b"/config/".to_vec(),
                temper_init::Directory::Data => b"/data/".to_vec(),
            };
            path.extend_from_slice(file.name);
            files.entries.push((path.into_boxed_slice(), Secret::new(bytes))).expect("generated files");
        }
    }
    let directories = config::Directories::given(b"/config", b"/data", b"/cache").expect("absolute directories");
    let parsed = parse(&document.expect("generated temper.json"), &directories).expect("real welcome preparation");
    let mut problems = super::Problems::empty();
    let resolved = application::resolve(&parsed.application, &files, &mut problems).expect("generated file values");
    assert!(problems.is_empty());
    let mut settings = settings::Engine {
        host_reply_bytes: None,
        agent_charter_bytes: None,
        machine: parsed.application.machine,
        directories: directories.clone(),
        jig: parsed.engine,
        clients: parsed.application.clients,
        sign_in: resolved.sign_in,
        webhooks: resolved.webhooks,
        files,
        demo: parsed.application.demo,
        welcome: false,
        program_environment: Box::new([]),
        host_part: None,
        ways_stock: crate::stock::ways(),
        kinds_stock: Box::default(),
        landings: parsed.application.landings,
        limits: parsed.application.limits,
    };
    let limits = super::engine_limits(&mut settings.jig.limits, &settings.limits, settings.jig.memory)
        .expect("unchanged effective limits");
    let seeds = fixtures::seeds();
    let core = config::core(&settings.jig, seeds.deployment, seeds.seed);
    let (forge, identities) = crate::forge::config(&settings, seeds.deployment);
    let forges = Box::new([]);
    (
        engine::Config {
            host_part: None,
            kinds_stock: crate::stock::forge(&limits),
            ways_stock: Box::default(),
            machine: Some(crate::system::machine_config(&settings.machine)),
            period_span: crate::stock::PERIOD_SPAN,
            core,
            forge,
            identities,
            forges,
            playground: crate::system::playground_config(
                settings.demo.as_ref(),
                settings.welcome,
                settings.limits.setup.deadline,
            ),
            landings: settings.landings,
            under_box: false,
        },
        limits,
    )
}

fn sign_in_owner(driver: &mut Driver) {
    driver.send(Event::Party(PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(900)),
        identity: people::Identity {
            key: people::IdentityKey { provider: 1, subject: b"ada@localhost".as_slice().into() },
            login: b"ada".as_slice().into(),
            name: b"Ada".as_slice().into(),
        },
    }));
}

#[test]
fn a_fresh_init_names_the_standard_profiles() {
    let init = generated_files()
        .files
        .into_iter()
        .find_map(|file| match file.content {
            temper_init::Content::Bytes { bytes, .. } if file.name == b"temper.json" => Some(bytes),
            temper_init::Content::Bytes { .. } | temper_init::Content::Link { .. } => None,
        })
        .expect("generated temper.json");
    for path in [b"engine.limits.profile".as_slice(), b"workers[0].limits.profile", b"application.limits.profile"] {
        assert_eq!(fixtures::value(&init, path).as_ref(), br#""standard""#, "{}", str::from_utf8(path).unwrap());
    }
    let overrides = fixtures::value(&init, b"engine.limits.overrides");
    let document = config::document(&overrides, &config::DocumentLimits { bytes: 16_384, tokens: 4096, depth: 4 })
        .expect("init engine overrides");
    let mut task_overrides = Vec::new();
    for at in (1..document.len().checked_sub(1).unwrap()).step_by(2) {
        let member = document.text(document.token(at).unwrap()).expect("override name");
        assert!(
            !member.starts_with(b"people.") && !member.starts_with(b"authority."),
            "init overrides {} instead of taking standard",
            str::from_utf8(member).unwrap()
        );
        if member.starts_with(b"tasks.") {
            task_overrides.push(str::from_utf8(member).expect("ASCII override name"));
        }
    }
    task_overrides.sort_unstable();
    assert_eq!(task_overrides.as_slice(), ["tasks.inbox_bytes", "tasks.message_bytes", "tasks.result_bytes"]);
    let (_, limits) = generated();
    assert_eq!(limits.tasks.inbox_messages, 82);
    assert_eq!(limits.tasks.message_bytes, 10_660);
    assert_eq!(limits.tasks.result_bytes, 10_660);
    assert_eq!(limits.tasks.inbox_bytes, limits.tasks.inbox_messages * limits.tasks.message_bytes);
}

#[test]
fn a_fresh_box_me_lists_the_stock_charters_at_fabrig_scale() {
    const EMPTY: Token = Token::new(901);
    const READ: Token = Token::new(902);
    let (config, limits) = generated();
    let mut driver = Driver::cold(Store::with_limits(&limits), config, &limits);
    driver.settle();
    sign_in_owner(&mut driver);
    driver.settle();
    driver.send(Event::Party(PartyInput::Read {
        reply_to: ReplyTo::new(EMPTY),
        sign_in: driver.session(),
        read: core::reads::Read::Frame,
    }));
    driver.settle();
    let document = driver
        .delivered
        .iter()
        .find_map(|delivery| {
            let Delivery::Read { reply, answer: core::reads::Answer::Document(document) } = delivery else {
                return None;
            };
            (*reply == ReplyTo::new(EMPTY)).then_some(document)
        })
        .expect("owner's initial frame");
    let core::documents::Document::Frame(frame) = document.as_ref() else { panic!("frame") };
    assert!(frame.projects.is_empty());
    ask(&mut driver, 1, people::Ask::MakeProject { name: "Hello".into(), policy: 1 });
    assert_eq!(reply(&driver, 1), &people::Reply::Outcome(people::Outcome::ProjectMade { project: 1 }));
    driver.send(Event::Party(PartyInput::Read {
        reply_to: ReplyTo::new(READ),
        sign_in: driver.session(),
        read: core::reads::Read::Frame,
    }));
    driver.settle();
    let document = driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            let Delivery::Read { reply, answer: core::reads::Answer::Document(document) } = delivery else {
                return None;
            };
            (*reply == ReplyTo::new(READ)).then_some(document)
        })
        .expect("owner's frame");
    let core::documents::Document::Frame(frame) = document.as_ref() else { panic!("frame") };
    let project = frame.projects.iter().find(|project| project.project == 1).expect("first made project");
    let offered = project.creating.as_ref().expect("owner may start a chat");
    let core::acts::Bounds::Create { procedures, charters, .. } = &offered.within else { panic!("create bounds") };
    assert_eq!(procedures.iter().map(|procedure| procedure.code).collect::<Vec<_>>(), [2, 3, 4, 6]);
    assert_eq!(charters.as_slice(), &[1, 2, 4, 5, 6, 9, 11, 12, 13]);
    let charters = charters.as_slice().to_vec();
    for (number, (turns, seconds)) in charters.into_iter().zip([
        (64, 3600),
        (800, 28_800),
        (800, 28_800),
        (800, 28_800),
        (800, 28_800),
        (4, 300),
        (64, 3600),
        (64, 3600),
        (800, 28_800),
    ]) {
        let token = Token::new(1000 + u64::from(number));
        driver.send(Event::Party(PartyInput::Read {
            reply_to: ReplyTo::new(token),
            sign_in: driver.session(),
            read: core::reads::Read::Setting {
                setting: core::settings::Setting::Charter(number),
                revision: 1,
                proposal: None,
            },
        }));
        driver.settle();
        let charter = driver
            .delivered
            .iter()
            .find_map(|delivery| {
                let Delivery::Read { reply, answer: core::reads::Answer::Setting(document) } = delivery else {
                    return None;
                };
                if *reply != ReplyTo::new(token) {
                    return None;
                }
                let Some(core::settings::Value::Charter(charter)) = &document.value else { panic!("stock charter") };
                Some(charter)
            })
            .expect("offered charter's current run policy");
        assert_eq!(charter.number, number);
        assert_eq!(charter.policy.turns, turns, "charter {number}");
        assert_eq!(charter.policy.time, skein_lib::Duration::from_secs(seconds), "charter {number}");
        assert_eq!(charter.policy.reserve_turns, 1, "charter {number}");
        assert_eq!(charter.policy.reserve_time, skein_lib::Duration::from_secs(60), "charter {number}");
    }
}

fn policies(driver: &Driver) -> Vec<(u32, people::PolicyValue)> {
    driver
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Core(core::Record::People(people::Stored::Policy { project, value, .. })) => {
                Some((*project, value.clone()))
            }
            Record::Core(_) | Record::Forge { .. } => None,
        })
        .collect()
}

fn ask(driver: &mut Driver, key: u8, ask: people::Ask) {
    driver.send(Event::Party(PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: driver.session(),
        key: [key; 16],
        ask,
    }));
    driver.settle();
}

fn reply(driver: &Driver, request: u8) -> &people::Reply {
    driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let Delivery::WebReply { to, reply } = delivery
                && *to == ReplyTo::new(Token::new(u64::from(request)))
            {
                Some(reply)
            } else {
                None
            }
        })
        .expect("one completed people request")
}

#[test]
#[expect(
    clippy::too_many_lines,
    clippy::disallowed_macros,
    clippy::print_stderr,
    reason = "one offline proof checks selected outcomes and reports its exact snapshot measurement"
)]
fn real_init_policy_fits_with_two_long_github_repositories() {
    let (config, limits) = generated();
    // Init runs the Standard profile; also prove the stricter product question.
    assert_eq!(limits.policy_bytes, 16_384);
    assert!(config.core.seed_projects.is_empty());
    assert_eq!(config.kinds_stock.len(), 14);
    let stock = config.core.defaults.policies.get(&1).expect("stock policy 1");
    let snapshot =
        core::policy_snapshot(&stock.policy, &stock.permissions, &stock.templates).expect("whole stock policy");
    assert_eq!(limits.authority.projects, 8);
    assert!(engine::refused(generated().0, &limits).is_empty());
    let mut strict = limits;
    strict.policy_bytes = 8192;
    assert!(engine::refused(generated().0, &strict).is_empty(), "the landing join also admits at 8192");
    // Chosen GitHub-shaped owner: 39 ASCII bytes; repositories reach the
    // unchanged adoption field bound (64), not GitHub's external name limit.
    // Admission below proves acceptance by the current code, without live API claims.
    let owner = vec![b'o'; 39];
    assert_eq!(limits.forge.name_bytes, 64);
    let first = vec![b'a'; 64];
    let second = vec![b'b'; 64];
    let mut kept = fixtures::forge_record(2);
    kept.kind = forge::Kind::GitHub;
    kept.place.api.host = b"api.github.com".as_slice().into();
    kept.place.api.port = 443;
    kept.place.api.tls = Some(b"api.github.com".as_slice().into());
    kept.place.git = b"https://github.com".as_slice().into();
    let mut driver = Driver::cold(Store::with_limits(&limits), config, &limits);
    driver.settle();
    driver.store.seed(Box::new([Record::Forge { row: Box::new(forge::Stored::Forge(kept)) }]), Box::new([]));
    driver.reopen(generated().0);
    driver.forge = Some(ForgeStand::with(&[(&owner, &first, 7), (&owner, &second, 8)]).at_forge(2));
    driver.settle();
    let before = policies(&driver);
    assert!(before.is_empty());
    sign_in_owner(&mut driver);
    driver.send(Event::Forge(Box::new(forge::Event::Connection {
        forge: 2,
        state: forge::Connection::Connected { id: 1, login: b"app[bot]".as_slice().into() },
    })));
    driver.settle();
    assert_eq!(policies(&driver), before, "connection and sign-in make no project policy");
    for (key, name, repository) in [(1, &first, 7), (2, &second, 8)] {
        ask(
            &mut driver,
            key,
            people::Ask::MakeProject {
                // Welcome permits editing the project name independently; jig caps it at 64.
                name: str::from_utf8(&name[..people::NAME_BYTES]).expect("ASCII project name").into(),
                policy: 1,
            },
        );
        let project = u32::from(key);
        assert_eq!(reply(&driver, key), &people::Reply::Outcome(people::Outcome::ProjectMade { project }));
        let adoption = forge::Adoption {
            project,
            forge: 2,
            host: b"github.com".as_slice().into(),
            owner: owner.clone().into_boxed_slice(),
            name: name.clone().into_boxed_slice(),
            prefix: b"temper/".as_slice().into(),
            role: forge::Role::Owned,
            landing: b"main".as_slice().into(),
            ci: true,
            checks: Box::new([]),
            home: true,
        };
        // Welcome sends a typed application; its real route locates the
        // repository before producing the generic, core-admitted Adopt.
        driver.send(Event::Party(PartyInput::Application {
            scope: people::ApplicationScope::Project(project),
            reply_to: ReplyTo::new(Token::new(u64::from(key + 10))),
            sign_in: driver.session(),
            key: [key + 10; 16],
            digest: [key; 32],
            carried: None,
            request: engine::Application::Forge(forge::Ask::Adopt(adoption)),
        }));
        driver.settle();
        let adopted = driver
            .delivered
            .iter()
            .find_map(|delivery| {
                if let Delivery::Application { to, outcome } = delivery
                    && *to == ReplyTo::new(Token::new(u64::from(key + 10)))
                {
                    Some(outcome)
                } else {
                    None
                }
            })
            .expect("completed adoption");
        let Some(engine::ApplicationAnswer::Adopt(forge::ApplicationOutcome::Adopted(adopted))) = adopted else {
            panic!("long-name repository adopted: {adopted:?}")
        };
        assert_eq!(adopted.repository, skein_forge::api::Repository { forge: 2, repository });
        let Some(Record::Forge { row }) =
            driver.store.rows.get(&Key::Forge(forge::Key::Repository(adopted.repository)))
        else {
            panic!("committed adopted repository")
        };
        let forge::Stored::Repository(row) = row.as_ref() else { panic!("repository key names its row") };
        assert_eq!((row.project, row.home), (project, true));
        let after = policies(&driver);
        assert_eq!(after.len(), usize::from(key));
        for (_, value) in &after {
            assert_eq!(value, &snapshot, "adoption adds no policy grants, requirements or metadata");
        }
    }
    let capacity = u8::try_from(limits.authority.projects).expect("profile project capacity fits request keys");
    for key in 3..=capacity {
        ask(&mut driver, key, people::Ask::MakeProject { name: format!("Project {key}").into(), policy: 1 });
        assert_eq!(
            reply(&driver, key),
            &people::Reply::Outcome(people::Outcome::ProjectMade { project: u32::from(key) })
        );
    }
    ask(&mut driver, capacity + 1, people::Ask::MakeProject { name: "Past capacity".into(), policy: 1 });
    assert_eq!(
        reply(&driver, capacity + 1),
        &people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Limit {
            which: people::LimitKind::Projects
        })),
        "the next welcome selection exceeds the profile's project capacity"
    );
    let bytes = people::policy_bytes(&snapshot).expect("exact joined snapshot bytes");
    // The standing charter adds one executor place in each of four policy authorities.
    assert_eq!(bytes, 6359);
    std::eprintln!(
        "real init joined policy: {bytes}/8192 requested bytes, configured bound {}; profile project capacity {capacity}; adopted=2; owner=39, repositories=64 bytes each; next welcome MakeProject refuses Projects",
        limits.policy_bytes
    );
    assert!(bytes <= 8192);
}

#[test]
fn the_template_machine_gives_the_domain_its_places() {
    let (config, _) = generated();
    assert!(config.forges.is_empty());
    assert!(config.identities.is_empty());
    let machine = config.machine.expect("machine is passed to the domain");
    let owned = machine.owned.unwrap();
    assert_eq!(owned.api.host.as_ref(), b"127.0.0.1");
    assert_eq!(owned.api.port, 8368);
    assert!(owned.api.tls.is_none());
    assert_eq!(machine.fake.unwrap().api.port, 8369);
    assert_eq!(machine.github.api.host.as_ref(), b"api.github.com");
    assert_eq!(machine.github.api.port, 443);
    assert_eq!(machine.github.api.tls.as_deref(), Some(b"api.github.com".as_slice()));
    assert_eq!(machine.github.git.as_ref(), b"https://github.com");
    assert_eq!(machine.github.web.as_deref(), Some(b"https://github.com".as_slice()));
}
