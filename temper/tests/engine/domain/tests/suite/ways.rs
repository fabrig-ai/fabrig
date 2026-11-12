//! The root makes each reader's ways through the core's watch and commit barrier
//! (temper/docs/design/domain/forge/change/branch-path.md; temper/docs/design/client/forge/records.md).
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
// The stock has three ways (protocol/configuration/settings.md, 8).
const STOCK_WAYS: usize = 3;
// The direct driver's eight passes reach an unacknowledged journal commit.
const HELD_PASSES: usize = 8;

fn configuration() -> (engine::Config, engine::Limits) {
    let mut limits = kinds::limits();
    limits.documents.roles = limits.documents.roles.max(2);
    limits.people.requests = 6; // Initial roles, three settings changes, adoption and changed roles.
    limits.authority.roles = 2; // Owner and member.
    limits.people.people = 3; // Owner, member and deployment service.
    limits.people.holdings = 3; // Owner of deployment and project, then member of project.
    let mut config = kinds::configuration(&limits);
    let policy = &mut config.core.seed_projects[0].policy.policy;
    let mut member = policy.roles[0].clone();
    member.number = people::Role::Member.number();
    policy.roles = Box::new([policy.roles[0].clone(), member]);
    (config, limits)
}

fn driver() -> Driver {
    let (config, limits) = configuration();
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
    settings::ProjectWays { drafts: false, goals: false, changes: true, way: 0 }
}

fn items() -> [Box<[ways::Item]>; STOCK_WAYS] {
    use temper_protocol_reaches::{Classes, Way};
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
    let rules = [
        (vec![(1, approve)], Way::YouApprove),
        (vec![(1, review)], Way::AgentsReview),
        (vec![(1, main), (2, goal)], Way::DraftFirst),
    ];
    rules.map(|(rules, way)| {
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
        items.push(ways::Item {
            kind: 8,
            scope: PROJECT,
            number: 1,
            edit: ways::Edit::Set(settings::Value::Classes(Classes::stock(way))),
        });
        if way == Way::DraftFirst {
            items.push(ways::Item { kind: 13, scope: PROJECT, number: 1, edit: ways::Edit::Set(settings::Value::ProjectTriager(settings::ProjectTriager { owner: settings::AreaOwner::Charter(14) })) });
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
            if name == "Draft first" {
                assert_eq!(entry.roles.len(), 1);
                assert_eq!(entry.roles[0].role.charter, 14);
                assert!(!entry.roles[0].running);
            } else {
                assert!(entry.roles.is_empty());
            }
        }
    }
}

#[test]
fn committed_settings_adoptions_and_roles_mark_the_ways_watch() {
    let mut driver = driver();
    let next_owner = member(&mut driver);
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
    assert!(got.iter().all(|document| document.words.is_none()), "no project's words have been held");
    ack(&mut driver);
    let words = settings::ProjectWords {
        headline: b"Agents review every change".as_slice().into(),
        lede: b"Agents make and review changes before they land.".as_slice().into(),
    };
    let before_words = documents(&driver).len();
    driver.send(kinds::change(
        next_owner,
        kinds::revision(&driver.store.rows),
        41,
        Box::new([kinds::item(10, PROJECT, 1, &settings::Value::ProjectWords(words.clone()))]),
    ));
    for _ in 0..HELD_PASSES {
        driver.advance(false);
    }
    assert_eq!(documents(&driver).len(), before_words, "the words wait for their commit");
    driver.settle();
    let got = documents(&driver);
    assert_eq!(
        got.len(),
        before_words.checked_add(1).unwrap(),
        "the project's words' commit marks ways: {:?}",
        driver.delivered.iter().filter(|delivery| matches!(delivery, Delivery::WebReply { .. })).collect::<Vec<_>>()
    );
    assert_eq!(got.last().unwrap().words.as_ref(), Some(&words));
}

const VIEW_REPLY: Token = Token::new(800);
const CHANNEL: Token = Token::new(7);
const VIEW_MAIN: &[&str] =
    &["you ask", "produce", "check", "review", "repair", "check", "conflict", "resolve", "merge", "verify", "main"];
const VIEW_OTHER: &[&str] =
    &["you ask", "produce", "check", "review", "repair", "check", "conflict", "resolve", "merge", "other"];
const VIEW_AFTER: &[&str] = &["you ask", "produce", "you approve", "merge", "verify", "main"];

fn view_driver() -> (Driver, engine::Assignment) {
    let (mut config, mut limits) = configuration();
    // One owner of the deployment and project, plus a project member and a stranger.
    limits.people.initial_owners = 2;
    limits.people.holdings = 4;
    limits.people.people = 4;
    limits.people.sign_ins = 3;
    limits.people.requests = 12;
    limits.authority.segments = 4; // A forge rule names its host, owner and repository.
    config.core.kinds = temper_service::stock::kinds(&limits, config.core.connectors[0]);
    limits.tasks.parameters = 4; // A tier procedure carries repository, tip, tier and run.
    limits.tasks.spec_bytes = 256; // The tier specification includes its commit and branch.
    let settings_family = config.core.settings.tools.settings;
    config.core.defaults.rules.ceiling.tools.0 |= settings_family;
    config.core.defaults.charters[0].policy.tools.0 |= settings_family;
    for project in &mut config.core.seed_projects {
        project.policy.policy.ceiling.tools.0 |= settings_family;
        for role in &mut project.policy.policy.roles {
            role.authority.tools.0 |= settings_family;
        }
    }
    let mut owner = config.core.owners[0].clone();
    owner.project = people::DEPLOYMENT;
    config.core.owners = Box::new([config.core.owners[0].clone(), owner]);
    let mut template = config.core.defaults.templates[0].clone();
    template.number = 4;
    template.executor = people::TemplateExecutor::Charter(1);
    template.tracked = Some(1);
    template.resources = Box::new([people::Symbol { connector: 0, symbol: 1 }]);
    config.core.defaults.templates = config.core.defaults.templates.into_vec().into_iter().chain([template]).collect();
    config
        .landings
        .insert(
            PROJECT,
            engine::ProjectLanding {
                check_timeout: None,
                by: engine::Way::Queue,
                close_job: None,
                merge_job: None,
                repositories: Box::new([engine::RepositoryChecks {
                    repository: skein_forge::api::Repository { forge: FORGE, repository: 2 },
                    gates: Box::new([]),
                    tiers: Box::new([engine::TierCheck {
                        name: b"focused".as_slice().into(),
                        workflow: b"checks.yml".as_slice().into(),
                        ignored: Box::new([]),
                        artifact: b"result".as_slice().into(),
                        format: temper_engine_domain_forge_tiers::Format::Junit,
                        follows: forge::Follows::Branch(b"main".as_slice().into()),
                        span_ms: 0,
                        description: Box::default(),
                    }]),
                }]),
                after: engine::AfterLandingCheck { tier_retries: 1, verification_bound_ms: 60_000 },
            },
        )
        .expect("one project landing");
    let mut driver = Driver::cold(Store::with_limits(&limits), config, &limits);
    driver.forge = Some(ForgeStand::new());
    driver.settle();
    driver.sign_in();
    driver.settle();
    let run = view_agent(&mut driver);
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
    // A second revision lets @revision 1 exercise the newest-only contract.
    change(&mut driver, 31, Box::new([kinds::item(7, PROJECT, 1, &settings::Value::Ways(own_ways()))]));
    driver.settle();
    (driver, run)
}

fn read_view(driver: &mut Driver, sign_in: u64, address: &str) -> Result<ways::View, jig_domain_core::reads::Why> {
    driver.delivered.clear();
    driver.send(Event::Party(PartyInput::ForgeRead {
        reply: ReplyTo::new(VIEW_REPLY),
        sign_in,
        scope: people::ApplicationScope::Project(PROJECT),
        query: engine::ForgeRead::View { project: PROJECT, address: address.as_bytes().into() },
    }));
    driver.settle();
    driver
        .delivered
        .iter()
        .find_map(|delivery| match delivery {
            Delivery::ForgeRead { reply, answer: engine::ForgeReadAnswer::View(view) }
                if *reply == ReplyTo::new(VIEW_REPLY) =>
            {
                Some(Ok(view.as_ref().clone()))
            }
            Delivery::Read { reply, answer: jig_domain_core::reads::Answer::None(why) }
                if *reply == ReplyTo::new(VIEW_REPLY) =>
            {
                Some(Err(*why))
            }
            _ => None,
        })
        .expect("one terminal view answer")
}

fn view_agent(driver: &mut Driver) -> engine::Assignment {
    driver.send(Event::Party(PartyInput::Key {
        reply_to: ReplyTo::new(Token::new(39)),
        sign_in: driver.session(),
        key: [39; 16],
        ask: Box::new(people::Ask::AccountChange {
            holder: people::AccountHolder::Deployment,
            change: people::AccountChange::AddKey { provider: 1, join: None, digest: [39; 32] },
        }),
        carried: jig_domain_core::Carried { number: 1_u64 << 63, digest: [39; 32] },
    }));
    driver.settle();
    driver.hello(CHANNEL);
    driver.period();
    driver.settle();
    driver.send(Event::Party(PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(40)),
        sign_in: driver.session(),
        key: [40; 16],
        models: Box::new([]),
        ask: driver.chat_ask(PROJECT, b"Explain the ways"),
    }));
    driver.settle();
    driver.last_assignment()
}

fn view_proposal(driver: &mut Driver, run: &engine::Assignment) -> (u64, u64) {
    use temper_engine_domain_world::host_tool::host_tool;
    let value = serde_json::json!({
        "pattern":{"segments":["6272616e6368"],"last":{"open":""}},
        "enforce":true,"ci":false,"up_to_date":false,"gates":[],
        "approvals":[{"role":0,"people":1,"freshness":"clean"}]
    });
    let input = serde_json::json!({"against": kinds::revision(&driver.store.rows), "reason":"Let the owner approve", "items":[{"setting":{"kind":"landing-rule","project":PROJECT,"number":1},"set":value}]});
    let (body, error) = host_tool(driver, run, 1, b"propose_settings", input.to_string().as_bytes());
    assert!(!error, "{}", String::from_utf8_lossy(&body));
    let proposal = driver.task(run.task).expect("proposing task").proposal.as_ref().expect("waiting proposal").number;
    (run.task, proposal)
}

fn words(path: &ways::Path) -> Vec<&str> {
    path.stops.iter().map(|stop| std::str::from_utf8(&stop.words).expect("path words are text")).collect()
}

#[derive(Debug, PartialEq, Eq)]
enum Picture {
    Way { name: String, line: String, current: bool, words: Vec<String> },
    Path { repository: Option<String>, branch: String, words: Vec<String> },
    Proposal { before: Vec<String>, after: Vec<String> },
    Charter { number: u32, who: String, name: String, facts: Vec<(String, String)> },
    Rule { kind: u32, number: u32, who: String, name: String, facts: Vec<(String, String)> },
    Roles(Vec<(String, String)>),
    NotDrawn(String),
}
fn strings(words: &[&str]) -> Vec<String> {
    words.iter().map(|word| (*word).to_owned()).collect()
}
fn drawn(view: ways::View) -> Picture {
    let text = |bytes: Box<[u8]>| String::from_utf8(bytes.into()).expect("view text");
    match view {
        ways::View::Way { name, line, path, current } => {
            Picture::Way { name: text(name), line: text(line), current, words: strings(&words(&path)) }
        }
        ways::View::Path(path) => Picture::Path {
            words: strings(&words(&path)),
            repository: path.repository.map(text),
            branch: text(path.branch),
        },
        ways::View::Proposal { before, after } => {
            Picture::Proposal { before: strings(&words(&before)), after: strings(&words(&after)) }
        }
        ways::View::Charter { charter, card } => Picture::Charter {
            number: charter,
            who: text(card.who),
            name: text(card.name),
            facts: card.facts.into_iter().map(|fact| (text(fact.label), text(fact.value))).collect(),
        },
        ways::View::Rule { kind, number, card } => Picture::Rule {
            kind,
            number,
            who: text(card.who),
            name: text(card.name),
            facts: card.facts.into_iter().map(|fact| (text(fact.label), text(fact.value))).collect(),
        },
        ways::View::Roles(roles) => {
            Picture::Roles(roles.into_iter().map(|(_, card)| (text(card.name), text(card.who))).collect())
        }
        ways::View::NotDrawn(words) => Picture::NotDrawn(text(words)),
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "view address and rule card rows share one project and proposal")]
fn views_draw_stored_ways_paths_charters_and_waiting_proposals() {
    let (mut driver, run) = view_driver();
    let member = member(&mut driver);
    let owner = driver.session();
    let (task, proposal) = view_proposal(&mut driver, &run);
    let proposal_address = format!("proposal t{task}/{proposal}");
    let newest = kinds::revision(&driver.store.rows);
    let draft = &[
        "draft",
        "plan",
        "produce ×n",
        "merge",
        "goal branch",
        "check",
        "repair",
        "check",
        "conflict",
        "resolve",
        "merge",
        "verify",
        "main",
    ];
    for (address, session, accept, expected) in [
        ("roles".to_owned(), owner, false, Picture::Roles(vec![("Chat".into(), "For each chat: talks with you and asks for the changes".into()), ("Coding agent".into(), "For each change: makes each change, repairs it and resolves its conflicts".into()), ("Reviewer".into(), "For each change: reviews each change at its head, and holds it until it approves".into())])),
        ("way agents review".to_owned(), owner, false, Picture::Way { name: "Agents review".into(), line: "An agent makes each change, another reviews it, and code lands it.".into(), current: true, words: strings(VIEW_MAIN) }),
        ("way Draft first".to_owned(), owner, false, Picture::Way { name: "Draft first".into(), line: "You and an agent write a design draft; agents build it on its own branch, and it lands whole.".into(), current: false, words: strings(draft) }),
        ("path org/repo".to_owned(), owner, false, Picture::Path { repository: Some("org/repo".into()), branch: "main".into(), words: strings(VIEW_MAIN) }),
        ("path org/repo other".to_owned(), owner, false, Picture::Path { repository: Some("org/repo".into()), branch: "other".into(), words: strings(VIEW_OTHER) }),
        ("path org/repo @revision 1".to_owned(), owner, false, Picture::NotDrawn(format!("revision 1 isn't drawn yet; views draw the newest settings, revision {newest}."))),
        ("charter 4".to_owned(), owner, false, Picture::Charter { number: 4, who: "an agent".into(), name: "the reviewer".into(), facts: vec![("Models".into(), "scripted".into()), ("Can".into(), "read the code and the forge; change and push its branch; run commands; start sub-agents; never merge".into()), ("Wakes".into(), "at once on words, results, failures and questions".into()), ("Asked to".into(), "Complete the task within its authority.".into())] }),
        (format!("path org/repo @revision {newest}"), owner, false, Picture::Path { repository: Some("org/repo".into()), branch: "main".into(), words: strings(VIEW_MAIN) }),
        ("rule 2/1".to_owned(), owner, false, Picture::Rule { kind: 2, number: 1, who: "a landing rule · holds a merge until it is met".into(), name: "the project's rule 1".into(), facts: vec![("Covers".into(), "every branch".into()), ("Asks".into(), "CI, up to date".into()), ("Gates".into(), "the reviewer, blocking, at a clean head".into())] }),
        ("rule 1/1".to_owned(), owner, false, Picture::NotDrawn("no rule 1/1.".into())),
        ("rule 3/1".to_owned(), owner, false, Picture::NotDrawn("no view rule 3/1; views: way NAME · path OWNER/NAME [BRANCH] · proposal tN/K · charter N · rule K/N.".into())),
        ("rule 2/0".to_owned(), owner, false, Picture::NotDrawn("no view rule 2/0; views: way NAME · path OWNER/NAME [BRANCH] · proposal tN/K · charter N · rule K/N.".into())),
        ("rule 2/1 @revision 2".to_owned(), owner, false, Picture::NotDrawn("no view rule 2/1 @revision 2; views: way NAME · path OWNER/NAME [BRANCH] · proposal tN/K · charter N · rule K/N.".into())),
        ("path org/missing".to_owned(), owner, false, Picture::NotDrawn("no repository org/missing in this project.".into())),
        ("proposal p99/1".to_owned(), owner, false, Picture::NotDrawn("no proposal p99/1.".into())),
        ("proposal t99/1".to_owned(), owner, false, Picture::NotDrawn("no proposal t99/1.".into())),
        ("path malformed".to_owned(), owner, false, Picture::NotDrawn("no view path malformed; views: way NAME · path OWNER/NAME [BRANCH] · proposal tN/K · charter N · rule K/N.".into())),
        ("charter 99".to_owned(), owner, false, Picture::NotDrawn("no charter 99.".into())),
        (proposal_address.clone(), owner, false, Picture::Proposal { before: strings(VIEW_MAIN), after: strings(VIEW_AFTER) }),
        ("way unknown".to_owned(), owner, false, Picture::NotDrawn("no way unknown; the ways are You approve, Agents review and Draft first.".into())),
        ("way".to_owned(), owner, false, Picture::NotDrawn("no view way; views: way NAME · path OWNER/NAME [BRANCH] · proposal tN/K · charter N · rule K/N.".into())),
        ("charter 99".to_owned(), member, false, Picture::NotDrawn("no charter 99.".into())),
        (proposal_address.clone(), member, false, Picture::Proposal { before: strings(VIEW_MAIN), after: strings(&["you ask", "produce", "an owner approves", "merge", "verify", "main"]) }),
        (proposal_address.clone(), owner, true, Picture::NotDrawn(format!("proposal t{task}/{proposal} is decided; a decided proposal's paths aren't drawn yet."))),
        ("rule 2/1".to_owned(), owner, false, Picture::Rule { kind: 2, number: 1, who: "a landing rule · holds a merge until it is met".into(), name: "the project's rule 1".into(), facts: vec![("Covers".into(), "every branch".into()), ("Approvals".into(), "you approve".into())] }),
        ("rule 2/1".to_owned(), member, false, Picture::Rule { kind: 2, number: 1, who: "a landing rule · holds a merge until it is met".into(), name: "the project's rule 1".into(), facts: vec![("Covers".into(), "every branch".into()), ("Approvals".into(), "an owner approves".into())] }),
    ] {
        if accept {
            driver.send(Event::Party(PartyInput::Ask { reply_to: ReplyTo::new(Token::new(50)), sign_in: owner, key: [50; 16], models: Box::new([]), ask: people::Ask::DecideProposal { project: people::DEPLOYMENT, proposer: people::Proposer::Task(task), proposal, decision: people::ProposalDecision::Accept, running: Some(people::Running::Stays) } }));
            driver.settle();
        }
        assert_eq!(drawn(read_view(&mut driver, session, &address).expect("authorized view")), expected, "{address}");
    }
    for (key, forge_pattern, last, covers) in [
        (70, false, forge::GateLast::Exact(b"main".as_slice().into()), "the branch main"),
        (71, false, forge::GateLast::Open(b"temper".as_slice().into()), "branches beginning temper"),
        (72, true, forge::GateLast::Exact(b"main".as_slice().into()), "org/repo's branch main"),
        (73, true, forge::GateLast::Open(Box::new([])), "org/repo's every branch"),
        (74, true, forge::GateLast::Open(b"temper".as_slice().into()), "org/repo's branches beginning temper"),
    ] {
        let mut advice = rule();
        advice.enforce = false;
        if forge_pattern {
            advice.pattern.segments = [b"forge".as_slice(), b"host", b"org", b"repo"].map(Box::from).into();
        }
        advice.pattern.last = last;
        advice.gates = Box::new([
            settings::RuleGate { number: 1, blocking: true, freshness: forge::JudgeFreshness::Exact },
            settings::RuleGate { number: 2, blocking: false, freshness: forge::JudgeFreshness::Clean },
        ]);
        advice.approvals = Box::new([
            settings::RuleApproval { role: 0, people: 2, freshness: forge::JudgeFreshness::Clean },
            settings::RuleApproval { role: 1, people: 1, freshness: forge::JudgeFreshness::Exact },
        ]);
        let kind = if forge_pattern { 1 } else { 2 };
        let scope = if forge_pattern { 0 } else { PROJECT };
        let against = kinds::revision(&driver.store.rows);
        driver.send(kinds::change(
            owner,
            against,
            key,
            Box::new([kinds::item(kind, scope, 1, &settings::Value::Rule(advice))]),
        ));
        driver.settle();
        assert_eq!(kinds::revision(&driver.store.rows), against + 1, "{key}: {:?}", driver.delivered);
        assert_eq!(
            drawn(read_view(&mut driver, member, &format!("rule {kind}/1")).expect("rule advice")),
            Picture::Rule {
                kind,
                number: 1,
                who: "a landing rule · advice, enforcing nothing".into(),
                name: if forge_pattern { "the deployment's rule 1" } else { "the project's rule 1" }.into(),
                facts: vec![
                    ("Covers".into(), covers.into()),
                    ("Gates".into(), "the reviewer, advice; the reviewer, advice, at a clean head".into()),
                    ("Approvals".into(), "an owner approves ×2; a role 1 approves".into())
                ],
            }
        );
    }
    let owner_view = read_view(&mut driver, owner, "charter 4");
    assert_eq!(read_view(&mut driver, member, "charter 4"), owner_view);
    driver.send(Event::Party(PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(60)),
        identity: people::Identity {
            key: forge::user_key(1, 9),
            login: b"stranger".as_slice().into(),
            name: Box::new([]),
        },
    }));
    driver.settle();
    let stranger = driver
        .delivered
        .iter()
        .find_map(|delivery| match delivery {
            Delivery::WebReply { reply: people::Reply::SignedIn { sign_in, .. }, .. } => Some(*sign_in),
            _ => None,
        })
        .expect("stranger sign-in");
    assert_eq!(read_view(&mut driver, stranger, "way Agents review"), Err(jig_domain_core::reads::Why::Unknown));
    driver.answer(
        CHANNEL,
        run.task,
        run.attempt,
        0,
        jig_domain_tasks::End::Finished {
            result: jig_domain_tasks::TaskResult::Report { words: b"The ways are explained.".as_slice().into() },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    driver.settle();
    // Let the configured tier finish its pending poll before the restart oracle takes its snapshot.
    if let Some(due) = driver.root.next_deadline(&driver.env) {
        driver.fire_at(skein_lib::Duration::from_nanos(due.as_nanos()));
        driver.settle();
    }
}

// The stock paths cross the root's settings, adoption and watch boundaries.
#[expect(clippy::too_many_lines, reason = "the fixture names the stock, adoption and restored tiers together")]
fn path_driver(ci_and_tiers: bool, equal_paths: bool) -> Driver {
    let mut limits = kinds::limits();
    limits.tasks.charters = 12; // Kind 9's stock lines run up to charter 12.
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
    if let jig_domain_tasks::EndedBy::Closer(closer) = &mut template.flags.ended_by {
        closer.connector = config.core.connectors[0];
    }
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
                description: Box::default(),
            },
        );
        assert!(
            config
                .landings
                .insert(
                    PROJECT,
                    engine::ProjectLanding {
                        check_timeout: None,
                        by: engine::Way::Queue,
                        close_job: None,
                        merge_job: Some(b"merge".as_slice().into()),
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

fn path_words(path: &ways::Path) -> String {
    path.stops.iter().map(|stop| std::str::from_utf8(&stop.words).expect("path words")).collect::<Vec<_>>().join(", ")
}

#[test]
fn stock_ways_paths_follow_the_projects_ci_and_tiers() {
    for (ci_and_tiers, expected) in [
        (
            true,
            [
                "you ask, produce, check, repair, check, you approve, conflict, resolve, merge, verify, main",
                "you ask, produce, check, review, repair, check, conflict, resolve, merge, verify, main",
                "draft, plan, produce ×n, merge, check, repair, catch up, goal branch, check, conflict, resolve, merge, verify, main",
            ],
        ),
        (
            false,
            [
                "you ask, produce, you approve, conflict, resolve, merge, main",
                "you ask, produce, review, repair, conflict, resolve, merge, main",
                "draft, plan, produce ×n, merge, goal branch, conflict, resolve, merge, main",
            ],
        ),
    ] {
        let driver = path_driver(ci_and_tiers, false);
        let document = documents(&driver).last().copied().expect("ways document");
        assert_eq!(document.current, Some(1), "Agents review: {ci_and_tiers}");
        assert_eq!(document.ways.len(), STOCK_WAYS);
        assert_eq!(document.own.repository.as_deref(), Some(b"org/repo".as_slice()));
        for (way, expected) in document.ways.iter().zip(expected) {
            assert_eq!(path_words(&way.path), expected, "{}: {ci_and_tiers}", String::from_utf8_lossy(&way.name));
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
    let rendered =
        temper_engine_smith::see_answer(b"ways", 0, &answer, &[], 8192, temper_engine_smith::ForgeReadBounds::STOCK);
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
        assert!(
            text.contains("Agents review · now\nAgents review every change, and code lands it\nAn agent makes each change, another reviews it, and code lands it.\n"),
            "{text}"
        );
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

#[test]
fn a_ways_watch_marks_a_live_triager_only_after_its_commit() {
    use jig_domain_authority::Executor;
    let (mut config, mut limits) = configuration();
    limits.tasks.charters = 16;
    limits.authority.executors = 2;
    limits.tasks.executor_kinds = 2;
    let mut triager = config.core.defaults.charters[0].clone();
    triager.number = 14;
    config.core.defaults.charters = Box::new([config.core.defaults.charters[0].clone(), triager]);
    let kinds = Box::new([Executor::Charter(1), Executor::Charter(14)]);
    config.core.defaults.rules.ceiling.delegation.kinds = kinds.clone();
    let policy = &mut config.core.seed_projects[0].policy.policy;
    policy.ceiling.delegation.kinds = kinds.clone();
    for role in &mut policy.roles {
        role.authority.delegation.kinds = kinds.clone();
    }
    let mut driver = Driver::cold(Store::with_limits(&limits), config, &limits);
    driver.settle();
    driver.sign_in();
    driver.settle();
    let session = driver.session();
    watch(&mut driver, session);
    driver.settle();
    let roles = &documents(&driver).last().unwrap().ways[2].roles;
    assert_eq!(roles.len(), 1);
    assert!(!roles[0].running);
    ack(&mut driver);
    let people::Ask::Create { project, mut batch } = driver.chat_ask(PROJECT, b"Route findings") else { panic!("create fixture") };
    batch[0].executor = people::Executor::Agent { charter: 14 };
    batch[0].label = 3;
    batch[0].flags = people::Flags { ended_by: people::EndedBy::Standing, conversation: false, listed: true, retention: people::Retention::Kept };
    driver.send(Event::Party(PartyInput::Ask { reply_to: ReplyTo::new(Token::new(99)), sign_in: session, key: [99; 16], models: Box::new([]), ask: people::Ask::Create { project, batch } }));
    let before = documents(&driver).len();
    for _ in 0..HELD_PASSES { driver.advance(false); }
    assert_eq!(documents(&driver).len(), before, "role state waits for its commit");
    driver.settle();
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery, Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::Created { .. }), .. })), "created role: {:?}", driver.delivered);
    assert!(documents(&driver).last().unwrap().ways[2].roles[0].running, "the committed live triager marks ways");
}
