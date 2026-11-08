//! The root makes each reader's ways through the core's watch and commit barrier
//! (temper/docs/design/domain/forge.md, 8.13; temper/docs/design/client/README.md, 4.6).
use jig_domain_core::{
    documents::{Watched, Window},
    settings as core_settings,
};
use jig_domain_people as people;
use skein_lib::{ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Event, PartyInput, ways};
use temper_engine_domain_forge::{self as forge, settings};
use temper_engine_domain_world::{direct::Driver, fixtures, forge_stand::ForgeStand, kinds, store::Store};

const PROJECT: u32 = 1;
const FORGE: u16 = 1;
const WATCHER: Token = Token::new(700);
// The stock has three ways (configuration.md, 13.10).
const STOCK_WAYS: usize = 3;
// The direct driver's eight passes reach an unacknowledged journal commit.
const HELD_PASSES: usize = 8;

fn driver() -> Driver {
    let mut limits = kinds::limits();
    limits.documents.roles = limits.documents.roles.max(2);
    limits.people.requests = 5; // Initial roles, two settings changes, adoption and changed roles.
    limits.authority.roles = 2; // Owner and member.
    limits.people.people = 3; // Owner, member and deployment service.
    limits.people.holdings = 3; // Owner of deployment and project, then member of project.
    let mut config = kinds::configuration(&limits);
    let policy = &mut config.core.seed_projects[0].policy.policy;
    let mut member = policy.roles[0].clone();
    member.number = people::Role::Member.number();
    policy.roles = Box::new([policy.roles[0].clone(), member]);
    let mut driver = Driver::cold(Store::with_limits(&limits), config, &limits);
    driver.forge = Some(ForgeStand::new());
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver
}

fn watch(driver: &mut Driver, sign_in: u64) {
    driver.send(Event::Party(PartyInput::Watch {
        watcher: WATCHER,
        sign_in,
        subject: Watched::Application {
            scope: people::ApplicationScope::Project(PROJECT),
            connector: 0,
            subject: Token::new(ways::subject(PROJECT)),
        },
        window: Window { rows: 1, after: None },
    }));
}

fn documents(driver: &Driver) -> Vec<&ways::WaysDocument> {
    driver
        .delivered
        .iter()
        .filter_map(|delivery| match delivery {
            Delivery::Ways { watcher, document } => {
                assert_eq!(*watcher, WATCHER);
                Some(document.as_ref())
            }
            _ => None,
        })
        .collect()
}

fn ack(driver: &mut Driver) {
    driver.send(Event::Party(PartyInput::Delivered { watcher: WATCHER }));
    driver.settle();
}

fn rule() -> settings::LandingRule {
    settings::LandingRule {
        pattern: forge::GatePattern {
            segments: Box::new([b"branch".as_slice().into()]),
            last: forge::GateLast::Open(Box::new([])),
        },
        enforce: true,
        ci: false,
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    }
}

fn change(driver: &mut Driver, key: u8, items: Box<[core_settings::Item]>) {
    driver.send(kinds::change(driver.session(), kinds::revision(&driver.store.rows), key, items));
}

fn own_ways() -> settings::ProjectWays {
    settings::ProjectWays { drafts: false, goals: false, changes: true, away: true, way: 0 }
}

fn items() -> [Box<[ways::Item]>; STOCK_WAYS] {
    let mut approve = rule();
    approve.ci = true;
    approve.up_to_date = true;
    approve.approvals =
        Box::new([settings::RuleApproval { role: 0, people: 1, freshness: forge::JudgeFreshness::Clean }]);
    let mut review = approve.clone();
    review.approvals = Box::new([]);
    review.gates =
        Box::new([settings::RuleGate { number: 1, blocking: true, freshness: forge::JudgeFreshness::Clean }]);
    let mut main = approve.clone();
    main.approvals = Box::new([]);
    main.pattern.last = forge::GateLast::Exact(b"main".as_slice().into());
    let mut goal = rule();
    goal.pattern.last = forge::GateLast::Open(b"temper".as_slice().into());
    let rules = [vec![(1, approve)], vec![(1, review)], vec![(1, main), (2, goal)]];
    rules.map(|rules| {
        let mut items: Vec<_> = (1..=4)
            .map(|number| ways::Item {
                kind: 2,
                scope: PROJECT,
                number,
                edit: rules
                    .iter()
                    .find(|(n, _)| *n == number)
                    .map_or(ways::Edit::Remove, |(_, rule)| ways::Edit::Set(settings::Value::Rule(rule.clone()))),
            })
            .collect();
        if rules.len() == 2 {
            let mut value = own_ways();
            value.drafts = true;
            value.goals = true;
            value.way = 4;
            items.push(ways::Item {
                kind: 7,
                scope: PROJECT,
                number: 1,
                edit: ways::Edit::Set(settings::Value::Ways(value)),
            });
        }
        items.into_boxed_slice()
    })
}

fn set_roles(driver: &mut Driver, key: u8, holdings: Box<[people::Holding]>) {
    driver.send(Event::Party(PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: driver.session(),
        key: [key; 16],
        models: Box::new([]),
        ask: people::Ask::SetRoles { project: PROJECT, holdings },
    }));
    driver.settle();
}

fn member(driver: &mut Driver) -> u64 {
    driver.send(Event::Party(PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(11)),
        identity: people::Identity {
            key: forge::user_key(1, 8),
            login: b"member".as_slice().into(),
            name: Box::new([]),
        },
    }));
    driver.settle();
    let owner_session = driver.session();
    let owner = driver
        .delivered
        .iter()
        .find_map(|delivery| match delivery {
            Delivery::WebReply { reply: people::Reply::SignedIn { person, sign_in, .. }, .. }
                if *sign_in == owner_session =>
            {
                Some(*person)
            }
            _ => None,
        })
        .expect("owner sign-in reply");
    let (person, session) = driver
        .delivered
        .iter()
        .find_map(|delivery| match delivery {
            Delivery::WebReply { reply: people::Reply::SignedIn { person, sign_in, .. }, .. }
                if *sign_in != owner_session =>
            {
                Some((*person, *sign_in))
            }
            _ => None,
        })
        .expect("member sign-in reply");
    set_roles(
        driver,
        12,
        Box::new([
            people::Holding { person: owner, role: people::Role::Owner },
            people::Holding { person, role: people::Role::Member },
        ]),
    );
    session
}

#[test]
fn a_ways_watch_delivers_the_stock_items_and_readers_permission_after_commit() {
    for (is_member, may_change) in [(false, true), (true, false)] {
        let mut driver = driver();
        let session = if is_member { member(&mut driver) } else { driver.session() };
        change(&mut driver, 20, Box::new([kinds::item(7, PROJECT, 1, &settings::Value::Ways(own_ways()))]));
        watch(&mut driver, session);
        for _ in 0..HELD_PASSES {
            driver.advance(false);
        }
        assert!(documents(&driver).is_empty(), "the document stays behind the commit barrier");
        driver.settle();
        let got = documents(&driver);
        assert_eq!(got.len(), 1, "{is_member}: {:?}", driver.delivered);
        let document = got[0];
        assert_eq!(document.project, PROJECT);
        assert_eq!(document.may_change, may_change);
        assert_eq!(document.ways.len(), STOCK_WAYS);
        for ((entry, name), items) in
            document.ways.iter().zip(["You approve", "Agents review", "Draft first"]).zip(items())
        {
            assert_eq!(entry.name.as_ref(), name.as_bytes());
            assert_eq!(entry.items, items, "{name}");
            assert_eq!(entry.delegation, name == "Draft first");
        }
    }
}

#[test]
fn committed_settings_adoptions_and_roles_mark_the_ways_watch() {
    let mut driver = driver();
    member(&mut driver);
    let session = driver.session();
    watch(&mut driver, session);
    driver.settle();
    ack(&mut driver);
    change(&mut driver, 20, Box::new([kinds::item(2, PROJECT, 1, &settings::Value::Rule(rule()))]));
    for _ in 0..HELD_PASSES {
        driver.advance(false);
    }
    assert_eq!(documents(&driver).len(), 1, "the refreshed document waits for the rule's commit");
    driver.settle();
    assert_eq!(documents(&driver).len(), 2, "the rule's commit marks ways");
    ack(&mut driver);
    let mut charter = temper_engine_domain_world::walking::charter();
    charter.policy.instructions = b"Read the changed settings.".as_slice().into();
    change(
        &mut driver,
        21,
        Box::new([core_settings::Item {
            setting: core_settings::Setting::Charter(1),
            edit: core_settings::Edit::Set(core_settings::Value::Charter(Box::new(charter))),
        }]),
    );
    driver.settle();
    assert_eq!(documents(&driver).len(), 3, "jig's committed settings mark ways");
    ack(&mut driver);
    driver.send(Event::Party(PartyInput::Application {
        scope: people::ApplicationScope::Project(PROJECT),
        reply_to: ReplyTo::new(Token::new(30)),
        sign_in: driver.session(),
        key: [30; 16],
        digest: [30; 32],
        carried: None,
        request: engine::Application::Forge(forge::Ask::Adopt(fixtures::adoption(FORGE))),
    }));
    driver.settle();
    let got = documents(&driver);
    assert_eq!(got.len(), 4, "adoption marks ways");
    assert_eq!(got[3].own.repository.as_deref(), Some(b"org/repo".as_slice()));
    ack(&mut driver);
    let mut holdings = driver
        .store
        .rows
        .values()
        .find_map(|record| match record {
            engine::Record::Core(jig_domain_core::Record::People(people::Stored::Roles {
                project: PROJECT,
                holdings,
            })) => Some(holdings.clone()),
            _ => None,
        })
        .expect("project role record");
    for holding in &mut holdings {
        holding.role = if holding.role == people::Role::Owner { people::Role::Member } else { people::Role::Owner };
    }
    set_roles(&mut driver, 40, holdings);
    let got = documents(&driver);
    assert_eq!(
        got.len(),
        5,
        "the reader's new role marks ways: {:?}",
        driver
            .delivered
            .iter()
            .filter(|delivery| matches!(delivery, Delivery::WebReply { .. } | Delivery::WatchEnded { .. }))
            .collect::<Vec<_>>()
    );
    assert!(!got[4].may_change, "the reader is now a member");
}

// The stock paths cross the root's settings, adoption and watch boundaries.
#[expect(clippy::too_many_lines, reason = "the fixture names the stock, adoption and restored tiers together")]
fn path_driver(ci_and_tiers: bool, equal_paths: bool) -> Driver {
    let mut limits = kinds::limits();
    limits.tasks.charters = 5;
    limits.tasks.spec_bytes = 512;
    limits.tasks.parameters = 4;
    limits.people.requests = 8; // Adoption, chat and the owner's settings change.
    let mut config = kinds::configuration(&limits);
    let models = config
        .core
        .settings
        .models
        .iter()
        .enumerate()
        .map(|(at, model)| core_settings::CatalogueModel {
            first_read: None,
            number: u32::try_from(at).expect("fixture models") + 1,
            model: model.clone(),
            shown: None,
            description: None,
            picker: true,
            withdrawn: false,
            source: core_settings::ModelSource::Stock,
            estimated: false,
        })
        .collect();
    config.core.defaults.rules.ceiling.tools.0 |= jig_domain_core::ToolFamilies::JIG.ask.0;
    let policy = &mut config.core.seed_projects[0].policy.policy;
    policy.ceiling.tools.0 |= jig_domain_core::ToolFamilies::JIG.ask.0;
    policy.roles[0].authority.tools.0 |= jig_domain_core::ToolFamilies::JIG.ask.0;
    let stock = temper_service::stock::with_models(models, limits.tasks.result_bytes);
    let mut charters = config.core.defaults.charters.into_vec();
    let mut coordinator = charters[0].clone();
    coordinator.number = 5;
    charters.push(coordinator);
    config.core.defaults.charters = charters.into_boxed_slice();
    let mut template = stock.templates.iter().find(|template| template.number == 4).expect("stock draft goal").clone();
    template.resources[0].connector = config.core.connectors[0];
    let mut templates = config.core.defaults.templates.into_vec();
    templates.push(template);
    config.core.defaults.templates = templates.into_boxed_slice();
    if equal_paths {
        config.ways_stock[0] = config.ways_stock[1].clone();
    }
    if ci_and_tiers {
        let tiers = [(b"slow".as_slice(), forge::Follows::Landing), (b"goal".as_slice(), forge::Follows::Goals)].map(
            |(name, follows)| engine::TierCheck {
                name: name.into(),
                workflow: name.into(),
                ignored: Box::new([]),
                artifact: b"tests".as_slice().into(),
                format: temper_engine_domain_forge_tiers::Format::Lines,
                follows,
                span_ms: 0,
            },
        );
        assert!(
            config
                .landings
                .insert(
                    PROJECT,
                    engine::ProjectLanding {
                        by: engine::Way::Queue,
                        merge_job: None,
                        repositories: Box::new([engine::RepositoryChecks {
                            repository: skein_forge::api::Repository { forge: FORGE, repository: 2 },
                            gates: Box::new([]),
                            tiers: Box::new(tiers),
                        }]),
                        after: engine::AfterLandingCheck { tier_retries: 0, verification_bound_ms: 1000 },
                    }
                )
                .is_ok()
        );
    }
    let restart = temper_engine_domain_world::oracle::config::copy(&config);
    config.landings = skein_lib::Map::with_capacity(1);
    let accounts = fixtures::account_rows(&config.core.settings.llm_providers);
    let bootstrap = temper_engine_domain_world::oracle::config::copy(&config);
    let mut cold = Driver::cold(Store::with_limits(&limits), bootstrap, &limits);
    cold.settle();
    let mut store = cold.take_store();
    store.seed(accounts, Box::new([]));
    let mut driver = Driver::cold(store, config, &limits);
    driver.forge = Some(ForgeStand::new());
    driver.settle();
    driver.sign_in();
    driver.settle();
    let adoption = fixtures::adoption(FORGE);

    driver.send(Event::Party(PartyInput::Application {
        scope: people::ApplicationScope::Project(PROJECT),
        reply_to: ReplyTo::new(Token::new(30)),
        sign_in: driver.session(),
        key: [30; 16],
        digest: [30; 32],
        carried: None,
        request: engine::Application::Forge(forge::Ask::Adopt(adoption)),
    }));
    driver.settle();
    // Restore the adopted repository before the configured tiers start following it.
    let mut store = driver.take_store();
    if !ci_and_tiers {
        // Adoption asks replacement checks; this saved snapshot exercises neither kind of check.
        let saved = store
            .rows
            .values()
            .find_map(|record| match record {
                engine::Record::Forge { row } => match row.as_ref() {
                    forge::Stored::Repository(repository) => {
                        let mut repository = repository.clone();
                        repository.ci = false;
                        Some(engine::Record::Forge { row: Box::new(forge::Stored::Repository(repository)) })
                    }
                    _ => None,
                },
                engine::Record::Core(_) => None,
            })
            .expect("adopted repository");
        store.seed(Box::new([saved]), Box::new([]));
    }
    let mut driver = Driver::cold(store, restart, &limits);
    driver.forge = Some(ForgeStand::new());
    driver.settle();
    driver.sign_in();
    driver.settle();
    driver.send(Event::Credential { number: 1, generation: 1, valid: skein_lib::Duration::from_secs(3600) });
    driver.settle();
    let session = driver.session();
    watch(&mut driver, session);
    driver.settle();
    driver
}

fn words(path: &ways::Path) -> String {
    path.stops.iter().map(|stop| std::str::from_utf8(&stop.words).expect("path words")).collect::<Vec<_>>().join(", ")
}

#[test]
fn stock_ways_paths_follow_the_projects_ci_and_tiers() {
    for (ci_and_tiers, expected) in [
        (
            true,
            [
                "you ask, produce, check, repair, check, you approve, update, resolve, merge, verify, main",
                "you ask, produce, check, review, repair, check, update, resolve, merge, verify, main",
                "draft, plan, produce ×n, merge, check, repair, goal branch, check, update, resolve, merge, verify, main",
            ],
        ),
        (
            false,
            [
                "you ask, produce, you approve, update, resolve, merge, main",
                "you ask, produce, review, repair, update, resolve, merge, main",
                "draft, plan, produce ×n, merge, goal branch, update, resolve, merge, main",
            ],
        ),
    ] {
        let driver = path_driver(ci_and_tiers, false);
        let document = documents(&driver).last().copied().expect("ways document");
        assert_eq!(document.current, Some(1), "Agents review: {ci_and_tiers}");
        assert_eq!(document.ways.len(), STOCK_WAYS);
        assert_eq!(document.own.repository.as_deref(), Some(b"org/repo".as_slice()));
        for (way, expected) in document.ways.iter().zip(expected) {
            assert_eq!(words(&way.path), expected, "{}: {ci_and_tiers}", String::from_utf8_lossy(&way.name));
        }
    }
}

fn see_ways(driver: &mut Driver, completion: u32) -> String {
    let run = driver.last_assignment();
    driver.call(
        Token::new(7),
        run.task,
        run.attempt,
        u64::from(completion),
        engine::Call { completion, position: 0, tool: engine::Tool::See { line: b"ways".as_slice().into(), from: 0 } },
    );
    driver.pump();
    let at = driver
        .delivered
        .iter()
        .rposition(|delivery| {
            matches!(delivery,
                Delivery::Host(delivery) if matches!(delivery.as_ref(), engine::HostDelivery::See { .. })
            )
        })
        .expect("chat person's see ways");
    let Delivery::Host(delivery) = driver.delivered.remove(at) else { unreachable!() };
    let engine::HostDelivery::See { to, answer, .. } = *delivery else { unreachable!() };
    let rendered = temper_engine_smith::see_answer(b"ways", 0, &answer, &[], 8192);
    let text = std::str::from_utf8(rendered.text()).expect("see text").to_owned();
    assert!(!rendered.error(), "{text}");
    driver.send(Event::Worker(engine::WorkerInput::RenderedHostCall {
        to,
        answer: jig_domain_core::SettledAnswer::Host { error: false, body: text.as_bytes().into() },
    }));
    driver.settle();
    text
}

#[test]
fn ways_current_is_the_first_equal_path_and_see_ways_marks_yours() {
    for equal_paths in [false, true] {
        let mut driver = path_driver(true, equal_paths);
        let document = documents(&driver).last().copied().expect("ways document");
        assert_eq!(document.current, Some(u32::from(!equal_paths)), "the first equal path is current");
        assert_eq!(document.ways[0].path.same(&document.ways[1].path), equal_paths);
        driver.hello(Token::new(7));
        driver.settle();
        driver.send(Event::Party(PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(1001)),
            sign_in: driver.session(),
            key: [31; 16],
            ask: driver.chat_ask(PROJECT, b"How do we work?"),
        }));
        driver.settle();
        let text = see_ways(&mut driver, 1);
        assert!(text.contains("Agents review · now\n"), "{text}");
        assert_eq!(text.starts_with("Agents review · now\n"), equal_paths, "{text}");
        assert_eq!(text.matches(" · now").count(), 1, "only the first equal way: {text}");
        ack(&mut driver);
        let mut own = rule();
        own.pattern.last = forge::GateLast::Open(Box::new([]));
        own.up_to_date = true;
        own.gates =
            Box::new([settings::RuleGate { number: 1, blocking: true, freshness: forge::JudgeFreshness::Clean }]);
        change(&mut driver, 40, Box::new([kinds::item(2, PROJECT, 1, &settings::Value::Rule(own))]));
        driver.settle();
        let document = documents(&driver).last().copied().expect("changed ways");
        assert_eq!(document.current, None, "CI removed from the project's rule");
        assert!(!document.own.stops.iter().any(|stop| stop.words.as_ref() == b"check"));
        let text = see_ways(&mut driver, 2);
        assert!(text.contains("\n\nYours · now\n"), "{text}");
        assert_eq!(text.matches(" · now").count(), 1, "only Yours: {text}");
    }
}
