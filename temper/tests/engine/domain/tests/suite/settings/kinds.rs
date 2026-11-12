//! Kinds judged, kept and used (temper/docs/design/domain/forge/change/gates.md, 2 and 3, and temper/docs/design/domain/forge/README.md, 10).
use jig_domain_core::settings as s;
use jig_domain_people as people;
use skein_fake_forge_domain::api as raw;
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Record};
use temper_engine_domain_forge::{self as forge, settings};
use temper_engine_domain_world::kinds;

fn rule() -> settings::LandingRule {
    settings::LandingRule {
        sets: Box::new([]),
        pattern: forge::GatePattern {
            segments: Box::new([b"forge".as_slice().into()]),
            last: forge::GateLast::Open(Box::new([])),
        },
        enforce: true,
        ci: false,
        up_to_date: false,
        gates: Box::new([]),
        approvals: Box::new([]),
    }
}

#[test]
fn a_rule_out_of_bounds_is_refused_naming_what_fails() {
    let mut driver = kinds::driver();
    let gate = settings::RuleGate { number: 1, blocking: true, freshness: forge::JudgeFreshness::Clean };
    let mut invalid = rule();
    invalid.gates = Box::new([gate, gate]);
    let items = Box::new([
        kinds::item(3, 0, 1, &settings::Value::Charter(1)),
        kinds::item(1, 0, 2, &settings::Value::Rule(invalid)),
    ]);
    driver.send(kinds::change(driver.session(), 1, 20, items));
    driver.settle();
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Judged { setting, why, at: 1 })), .. }
        if *setting == people::Setting::Application { kind: 1, scope: 0, number: 2 }
            && why.as_ref() == settings::Refusal::Gates.words()
    )), "forge refusal: {:?}", driver.delivered);
    assert_eq!(kinds::revision(&driver.store.rows), 1);
    assert!(!driver.store.rows.values().any(|record| matches!(record,
        Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Setting(settings::SettingRow {
            key: settings::SettingKey { slot: settings::SettingSlot::Change, .. }, ..
        }))
    )));
}

fn apply(driver: &mut temper_engine_domain_world::direct::Driver, key: u8, items: Box<[s::Item]>) {
    let against = kinds::revision(&driver.store.rows);
    driver.send(kinds::change(driver.session(), against, key, items));
    driver.settle();
    assert_eq!(kinds::revision(&driver.store.rows), against + 1, "{:?}", driver.delivered);
}

fn kept(
    driver: &temper_engine_domain_world::direct::Driver,
    kind: u32,
    scope: u32,
    number: u32,
    slot: settings::SettingSlot,
) -> Option<&settings::Edit> {
    driver.store.rows.values().find_map(|record| {
        if let Record::Forge { row } = record
            && let forge::Stored::Setting(row) = row.as_ref()
            && row.key == (settings::SettingKey { kind, scope, number, slot })
        {
            Some(&row.edit)
        } else {
            None
        }
    })
}

fn ask(driver: &mut temper_engine_domain_world::direct::Driver, key: u8, ask: people::Ask) {
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: driver.session(),
        key: [key; 16],
        ask,
    }));
    driver.pump();
}

fn propose(driver: &mut temper_engine_domain_world::direct::Driver, key: u8) -> (u64, u64) {
    propose_items(driver, key, Box::new([kinds::item(1, 0, 2, &settings::Value::Rule(rule()))]))
}

fn propose_items(
    driver: &mut temper_engine_domain_world::direct::Driver,
    key: u8,
    items: Box<[s::Item]>,
) -> (u64, u64) {
    let against = kinds::revision(&driver.store.rows);
    driver.send(kinds::proposal(driver.session(), against, key, items));
    driver.settle();
    let proposal = driver
        .delivered
        .iter()
        .rev()
        .find_map(|delivery| {
            if let Delivery::WebReply {
                reply: people::Reply::Outcome(people::Outcome::Proposed { proposal, .. }),
                ..
            } = delivery
            {
                Some(*proposal)
            } else {
                None
            }
        })
        .expect("party's proposal kept");
    let stored_key = driver
        .store
        .rows
        .values()
        .find_map(|record| {
            if let Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Settings(
                s::Record::Proposal { proposer: people::Proposer::Party(1), proposal: number, value },
            ))) = record
                && u64::from(*number) == proposal
            {
                value.key
            } else {
                None
            }
        })
        .expect("deployment-unique proposal key");
    (proposal, stored_key)
}

#[test]
fn a_proposal_of_rules_is_kept_then_accepted() {
    let mut driver = kinds::driver();
    let (proposal, key) = propose(&mut driver, 21);
    let proposed =
        kept(&driver, 1, 0, 2, settings::SettingSlot::Proposal(key)).expect("forge keeps the proposal").clone();
    assert!(matches!(&proposed, settings::Edit::Set(value) if value.value == settings::Value::Rule(rule())));
    assert_eq!(kept(&driver, 1, 0, 2, settings::SettingSlot::Change), None);
    assert_eq!(kinds::revision(&driver.store.rows), 1, "a proposal changes no setting");
    ask(
        &mut driver,
        22,
        people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Party(1),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Stays),
        },
    );
    assert_eq!(kinds::revision(&driver.store.rows), 2, "{:?}", driver.delivered);
    assert_eq!(kept(&driver, 1, 0, 2, settings::SettingSlot::Change), Some(&proposed));
    assert_eq!(kept(&driver, 1, 0, 2, settings::SettingSlot::Proposal(key)), None);

    let (withdrawn, key) = propose(&mut driver, 23);
    assert!(kept(&driver, 1, 0, 2, settings::SettingSlot::Proposal(key)).is_some());
    ask(&mut driver, 24, people::Ask::Withdraw { project: people::DEPLOYMENT, proposal: withdrawn });
    assert_eq!(kept(&driver, 1, 0, 2, settings::SettingSlot::Proposal(key)), None);
    assert_eq!(kept(&driver, 1, 0, 2, settings::SettingSlot::Change), Some(&proposed));
    assert_eq!(kinds::revision(&driver.store.rows), 2);
}

#[test]
fn a_deleted_project_loses_its_rules() {
    let mut driver = kinds::driver();
    apply(
        &mut driver,
        25,
        Box::new([
            kinds::item(1, 0, 2, &settings::Value::Rule(rule())),
            kinds::item(2, 1, 1, &settings::Value::Rule(rule())),
        ]),
    );
    let deployment: Vec<_> = driver
        .store
        .rows
        .values()
        .filter_map(|record| {
            if let Record::Forge { row } = record
                && let forge::Stored::Setting(row) = row.as_ref()
                && row.key.scope == 0
            {
                Some(row.clone())
            } else {
                None
            }
        })
        .collect();
    assert!(kept(&driver, 2, 1, 1, settings::SettingSlot::Change).is_some());
    let chat = driver.chat_ask(1, b"work under the project rules");
    ask(&mut driver, 27, chat);
    ask(&mut driver, 28, people::Ask::Cancel { project: 1, task: 1, reason: Box::new([]), grace: None });
    driver.pump();
    ask(&mut driver, 26, people::Ask::RemoveProject { project: 1, grace: None });
    driver.hello(Token::new(7));
    driver.fire_at(Duration::from_secs(5));
    driver.settle();
    assert!(
        driver.delivered.iter().any(|delivery| matches!(
            delivery,
            Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::ProjectRemoved { project: 1 }), .. }
        )),
        "{:?}",
        driver.delivered
    );
    assert!(
        !driver.store.rows.values().any(|record| matches!(record,
            Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Setting(row) if row.key.scope == 1)
        )),
        "all of the project's setting rows are erased: {:?}",
        driver.store.rows
    );
    for row in deployment {
        assert_eq!(kept(&driver, row.key.kind, 0, row.key.number, row.key.slot), Some(&row.edit));
    }
}

#[test]
fn a_restart_holds_the_rules_and_finds_the_stock_unchanged() {
    use temper_engine_domain_forge_change::State;
    let mut world = kinds::forge_world();
    let goal = kinds::goal(&mut world);
    let mut initial = goal_rule();
    initial.ci = true;
    initial.up_to_date = true;
    world_change(&mut world, 28, Box::new([kinds::item(2, 1, 1, &settings::Value::Rule(initial))]));
    let (change, producer) = kinds::goal_change(&mut world, &goal);
    kinds::produce(&mut world, change, &producer);
    for _ in 0..200 {
        world.tick();
        if matches!(kinds::change_row(&world, change).change.state, State::Checking { .. }) {
            break;
        }
    }
    assert!(matches!(kinds::change_row(&world, change).change.state, State::Checking { .. }));
    world.until(temper_engine_domain_world::forge_routes::Until::Ready);
    let defaults: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| {
            if let Record::Forge { row } = record
                && let forge::Stored::Setting(row) = row.as_ref()
                && row.key.slot == settings::SettingSlot::Default
            {
                Some(row.clone())
            } else {
                None
            }
        })
        .collect();
    let revision = kinds::revision(&world.store.rows) + 1;
    kinds::restart_after_rule_commit(&mut world, 29, kinds::item(2, 1, 1, &settings::Value::Rule(goal_rule())));
    for _ in 0..200 {
        world.tick();
        if matches!(kinds::change_row(&world, change).change.state, State::Landed { .. }) {
            break;
        }
    }
    assert!(
        matches!(kinds::change_row(&world, change).change.state, State::Landed { .. }),
        "the first restored change steps use the kept rule, before any new settings input"
    );
    assert_eq!(world.merges, 1);
    assert_eq!(
        kinds::revision(&world.store.rows),
        revision,
        "the core is told no default differs, so no release revision is made"
    );
    for row in defaults {
        assert_eq!(
            world.store.rows.get(&engine::Key::Forge(forge::Key::Setting(row.key))),
            Some(&Record::Forge { row: Box::new(forge::Stored::Setting(row)) })
        );
    }
    world.close();
}

fn goal_rule() -> settings::LandingRule {
    settings::LandingRule {
        pattern: forge::GatePattern {
            segments: [b"forge".as_slice(), b"forge.example", b"org", b"repo", b"branch"].map(Box::from).into(),
            last: forge::GateLast::Open(b"temper".as_slice().into()),
        },
        ..rule()
    }
}

fn world_change(world: &mut temper_engine_domain_world::forge_routes::World, key: u8, items: Box<[s::Item]>) {
    let against = kinds::revision(&world.store.rows);
    world.send(kinds::change(world.signed_in.expect("owner"), against, key, items));
    world.until(temper_engine_domain_world::forge_routes::Until::Ready);
    assert_eq!(kinds::revision(&world.store.rows), against + 1, "{:?}", world.web_replies);
}

#[test]
fn a_landing_rule_changed_reaches_the_next_merge() {
    use temper_engine_domain_forge_change::State;
    let mut world = kinds::forge_world();
    let goal = kinds::goal(&mut world);
    let mut initial = goal_rule();
    initial.ci = true;
    initial.up_to_date = true;
    world_change(&mut world, 29, Box::new([kinds::item(2, 1, 1, &settings::Value::Rule(initial))]));
    let (change, producer) = kinds::goal_change(&mut world, &goal);
    assert_eq!(
        kinds::change_row(&world, change).base.as_ref(),
        format!("temper/{}/g{}", goal.task, goal.task).as_bytes()
    );
    kinds::produce(&mut world, change, &producer);
    for _ in 0..200 {
        world.tick();
        if matches!(kinds::change_row(&world, change).change.state, State::Checking { .. }) {
            break;
        }
    }
    assert!(
        matches!(kinds::change_row(&world, change).change.state, State::Checking { .. }),
        "no CI verdict keeps the change checking"
    );
    assert_eq!(world.merges, 0);
    let rules = world
        .store
        .rows
        .values()
        .find_map(|record| {
            if let Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Settings(
                s::Record::Default {
                    setting: s::Setting::Rules,
                    default: s::Default { value: s::Value::Rules(rules), .. },
                },
            ))) = record
            {
                Some(rules.clone())
            } else {
                None
            }
        })
        .expect("rules default");
    for requirement in [5, 6] {
        assert!(rules.requirements.iter().any(|rule| rule.judge.requirement == requirement));
    }
    world_change(&mut world, 30, Box::new([kinds::item(2, 1, 1, &settings::Value::Rule(goal_rule()))]));
    for _ in 0..200 {
        world.tick();
        if matches!(kinds::change_row(&world, change).change.state, State::Landed { .. }) {
            break;
        }
    }
    let row = kinds::change_row(&world, change);
    assert!(matches!(row.change.state, State::Landed { .. }), "{:?}", row.change.state);
    assert_eq!(world.merges, 1, "the two standing requirements met at the next merge");
    assert_eq!(row.change.updates, 0);
    assert_eq!(row.change.repairs, 0);
    assert!(!world.repository_writes.iter().any(|(_, write)| matches!(write, skein_forge::api::Write::Update { .. })));
    let pull = row.pull.expect("change pull");
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number: pull })) else {
        panic!("pull")
    };
    assert!(pull.merged.is_some());
    assert!(!pull.statuses.iter().any(|status| status.state == raw::Check::Passed), "CI never passed");
    world.close();
}

#[test]
fn the_forges_charter_change_reaches_the_next_change() {
    let mut world = kinds::forge_world();
    let goal = kinds::goal(&mut world);
    assert!(
        world.store.rows.values().any(|record| matches!(record,
            Record::Forge { row } if matches!(row.as_ref(), forge::Stored::Setting(settings::SettingRow {
                key: settings::SettingKey { kind: 3, scope: 0, number: 1, slot: settings::SettingSlot::Default },
                edit: settings::Edit::Set(settings::SizedValue { value: settings::Value::Charter(2), .. }),
            }))
        )),
        "the forge starts with producer charter 2"
    );
    world_change(&mut world, 31, Box::new([kinds::item(3, 0, 1, &settings::Value::Charter(1))]));
    let (change, producer) = kinds::goal_change(&mut world, &goal);
    assert_eq!(producer.charter, 1);
    assert_eq!(kinds::change_row(&world, change).producing_charter, 1);
    assert_eq!(kinds::change_row(&world, change).review_charter, 4);

    let against = kinds::revision(&world.store.rows);
    world.send(kinds::change(
        world.signed_in.expect("owner"),
        against,
        32,
        Box::new([kinds::item(3, 0, 1, &settings::Value::Charter(3))]),
    ));
    world.until(temper_engine_domain_world::forge_routes::Until::Ready);
    assert!(
        world.web_replies.iter().any(|reply| matches!(
            reply,
            people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
                at: 0,
                why: people::Malformed::Settings {
                    setting: people::Setting::Application { kind: 3, scope: 0, number: 1 },
                    fault: people::SettingFault::Reference,
                    detail: None
                },
            }))
        )),
        "the core refuses the unoffered charter, naming kind 3: {:?}",
        world.web_replies
    );
    assert_eq!(kinds::revision(&world.store.rows), against);
    assert_eq!(kinds::change_row(&world, change).producing_charter, 1);
    world_change(&mut world, 33, Box::new([kinds::item(2, 1, 1, &settings::Value::Rule(goal_rule()))]));
    kinds::produce(&mut world, change, &producer);
    for _ in 0..200 {
        world.tick();
        if matches!(
            kinds::change_row(&world, change).change.state,
            temper_engine_domain_forge_change::State::Landed { .. }
        ) {
            break;
        }
    }
    assert!(matches!(
        kinds::change_row(&world, change).change.state,
        temper_engine_domain_forge_change::State::Landed { .. }
    ));
    world.close();
}

fn proposal_key(rows: &std::collections::BTreeMap<engine::Key, Record>, proposal: u64) -> u64 {
    rows.values()
        .find_map(|record| {
            if let Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Settings(
                s::Record::Proposal { proposer: people::Proposer::Party(1), proposal: number, value },
            ))) = record
                && u64::from(*number) == proposal
            {
                value.key
            } else {
                None
            }
        })
        .expect("connector proposal key")
}

fn forge_kept(
    world: &temper_engine_domain_world::forge_routes::World,
    kind: u32,
    number: u32,
    slot: settings::SettingSlot,
) -> &settings::Edit {
    let key = settings::SettingKey { kind, scope: 0, number, slot };
    let Some(Record::Forge { row }) = world.store.rows.get(&engine::Key::Forge(forge::Key::Setting(key))) else {
        panic!("missing {key:?}; rows: {:?}", world.store.rows.keys().collect::<Vec<_>>())
    };
    let forge::Stored::Setting(row) = row.as_ref() else { panic!("setting row") };
    &row.edit
}

fn decide_restore(proposal: u64) -> people::Ask {
    people::Ask::DecideProposal {
        project: people::DEPLOYMENT,
        proposer: people::Proposer::Party(1),
        proposal,
        decision: people::ProposalDecision::Accept,
        running: Some(people::Running::Stays),
    }
}

#[test]
fn a_landing_rule_and_the_forges_charter_changed_then_restored() {
    use temper_engine_domain_forge_change::State;
    use temper_engine_domain_world::forge_routes::{Gate, StoryLimits, Until, World};
    // The changed charter 2 differs from this story's initial producer charter 1.
    let mut world = World::configured_limits(
        true,
        true,
        true,
        1000,
        Gate::default(),
        None,
        StoryLimits { kinds: true, forge_charter: Some(1), ..StoryLimits::default() },
    )
    .with_settings_history();
    // The fixture imports its kinds in a release before the party edits them.
    let goal = kinds::goal(&mut world);
    let earlier_revision = kinds::revision(&world.store.rows) + 1;
    let replaced_by = u64::from(earlier_revision + 1);
    let earlier = goal_rule();
    world_change(
        &mut world,
        40,
        Box::new([
            kinds::item(1, 0, 2, &settings::Value::Rule(earlier.clone())),
            kinds::item(3, 0, 1, &settings::Value::Charter(2)),
        ]),
    );
    let mut later = earlier.clone();
    later.ci = true;
    world_change(
        &mut world,
        41,
        Box::new([
            kinds::item(1, 0, 2, &settings::Value::Rule(later)),
            kinds::item(3, 0, 1, &settings::Value::Charter(1)),
        ]),
    );
    let rule = forge_kept(&world, 1, 2, settings::SettingSlot::Replaced(replaced_by)).clone();
    let charter = forge_kept(&world, 3, 1, settings::SettingSlot::Replaced(replaced_by)).clone();
    assert!(matches!(&rule, settings::Edit::Set(value) if value.value == settings::Value::Rule(earlier)));
    assert!(matches!(&charter, settings::Edit::Set(value) if value.value == settings::Value::Charter(2)));
    let (change, producer) = kinds::goal_change(&mut world, &goal);
    kinds::produce(&mut world, change, &producer);
    for _ in 0..200 {
        world.tick();
        if matches!(kinds::change_row(&world, change).change.state, State::Checking { .. }) {
            break;
        }
    }
    assert!(matches!(kinds::change_row(&world, change).change.state, State::Checking { .. }));
    assert_eq!(world.merges, 0);
    world.send(kinds::restore(world.signed_in.expect("owner"), earlier_revision, 42));
    world.until(Until::Ready);
    let proposal = world
        .web_replies
        .iter()
        .rev()
        .find_map(|reply| {
            if let people::Reply::Outcome(people::Outcome::Restored { proposal, to: 0, left_out }) = reply {
                assert!(left_out.is_empty());
                Some(*proposal)
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("restore: {:?}", world.web_replies));
    let key = proposal_key(&world.store.rows, proposal);
    assert_eq!(forge_kept(&world, 1, 2, settings::SettingSlot::Proposal(key)), &rule);
    assert_eq!(forge_kept(&world, 3, 1, settings::SettingSlot::Proposal(key)), &charter);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(43)),
        sign_in: world.signed_in.expect("owner"),
        key: [43; 16],
        ask: decide_restore(proposal),
    }));
    world.until(Until::Ready);
    assert_eq!(kinds::revision(&world.store.rows), earlier_revision + 2, "{:?}", world.web_replies);
    assert_eq!(forge_kept(&world, 1, 2, settings::SettingSlot::Change), &rule);
    assert_eq!(forge_kept(&world, 3, 1, settings::SettingSlot::Change), &charter);
    for _ in 0..200 {
        world.tick();
        if matches!(kinds::change_row(&world, change).change.state, State::Landed { .. }) {
            break;
        }
    }
    let row = kinds::change_row(&world, change);
    assert!(matches!(row.change.state, State::Landed { .. }), "{:?}", row.change.state);
    assert_eq!(world.merges, 1);
    let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number: row.pull.expect("pull") }))
    else {
        panic!("pull")
    };
    assert!(pull.merged.is_some());
    assert!(!pull.statuses.iter().any(|status| status.state == raw::Check::Passed), "CI never passed");
    world.close();
}

#[test]
fn a_restore_leaves_out_a_deleted_projects_rule() {
    let mut driver = kinds::driver();
    apply(
        &mut driver,
        44,
        Box::new([
            kinds::item(1, 0, 2, &settings::Value::Rule(rule())),
            kinds::item(2, 1, 1, &settings::Value::Rule(rule())),
        ]),
    );
    ask(&mut driver, 45, people::Ask::RemoveProject { project: 1, grace: None });
    driver.fire_at(Duration::from_secs(5));
    driver.pump();
    driver.send(kinds::restore(driver.session(), 1, 46));
    driver.settle();
    let (proposal, left_out) = kinds::restored(&driver);
    assert_eq!(left_out.as_ref(), &[s::Setting::Application { kind: 2, scope: 1, number: 1 }]);
    let key = proposal_key(&driver.store.rows, proposal);
    assert_eq!(kept(&driver, 2, 1, 1, settings::SettingSlot::Proposal(key)), None);
    assert!(kept(&driver, 1, 0, 2, settings::SettingSlot::Proposal(key)).is_some());
    ask(&mut driver, 47, decide_restore(proposal));
    assert_eq!(kinds::revision(&driver.store.rows), 3, "{:?}", driver.delivered);
}

#[test]
fn forgetting_erases_the_forges_replaced_rows() {
    use temper_engine_domain_world::{direct::Driver, store::Store};
    let mut limits = kinds::limits();
    // Keep two earlier revisions beside the current one; making 5 forgets 2.
    limits.settings.history = 2;
    // Five edits reach an existing replaced row on both sides of pruning.
    limits.people.requests = 5;
    let mut driver = Driver::cold(Store::with_limits(&limits), kinds::configuration(&limits), &limits);
    driver.settle();
    driver.sign_in();
    driver.settle();
    for (key, enforce) in [(48, true), (49, false), (50, true), (51, false)] {
        let mut value = rule();
        value.enforce = enforce;
        apply(&mut driver, key, Box::new([kinds::item(1, 0, 2, &settings::Value::Rule(value))]));
    }
    assert!(!driver.store.rows.contains_key(&engine::Key::Core(jig_domain_core::Key::Core(
        jig_domain_core::CoreKey::Settings(s::Key::History(2)),
    ))));
    assert_eq!(kept(&driver, 1, 0, 2, settings::SettingSlot::Replaced(2)), None);
    assert!(
        matches!(kept(&driver, 1, 0, 2, settings::SettingSlot::Replaced(3)), Some(settings::Edit::Set(value)) if value.value == settings::Value::Rule(rule()))
    );
    // This row exists before it falls out of history, unlike revision 2's
    // first set, which replaced no changed value.
    apply(&mut driver, 52, Box::new([kinds::item(1, 0, 2, &settings::Value::Rule(rule()))]));
    assert!(!driver.store.rows.contains_key(&engine::Key::Core(jig_domain_core::Key::Core(
        jig_domain_core::CoreKey::Settings(s::Key::History(3)),
    ))));
    assert_eq!(kept(&driver, 1, 0, 2, settings::SettingSlot::Replaced(3)), None);
    let mut previous = rule();
    previous.enforce = false;
    assert!(
        matches!(kept(&driver, 1, 0, 2, settings::SettingSlot::Replaced(4)), Some(settings::Edit::Set(value)) if value.value == settings::Value::Rule(previous))
    );
}

fn project_ways() -> settings::Value {
    settings::Value::Ways(settings::ProjectWays { drafts: true, goals: true, changes: false, way: 0 })
}

fn project_member(driver: &mut temper_engine_domain_world::direct::Driver) -> u64 {
    driver.send(engine::Event::Party(engine::PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(60)),
        identity: people::Identity {
            key: temper_engine_domain_forge::user_key(1, 8_u64),
            login: b"member".as_slice().into(),
            name: b"Member".as_slice().into(),
        },
    }));
    driver.settle();
    let (member, session) = driver
        .delivered
        .iter()
        .find_map(|delivery| {
            if let Delivery::WebReply { to, reply: people::Reply::SignedIn { person, sign_in, .. } } = delivery
                && *to == ReplyTo::new(Token::new(60))
            {
                Some((*person, *sign_in))
            } else {
                None
            }
        })
        .expect("member signed in");
    ask(
        driver,
        61,
        people::Ask::SetRoles {
            project: 1,
            holdings: Box::new([
                people::Holding { person: 1, role: people::Role::Owner },
                people::Holding { person: member, role: people::Role::Member },
            ]),
        },
    );
    assert!(driver.delivered.iter().any(|delivery| matches!(
        delivery,
        Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::RolesSet { project: 1 }), .. }
    )));
    session
}

#[test]
fn a_projects_ways_are_changed_by_its_owner_and_read_by_its_member() {
    let mut driver = kinds::driver();
    let value = project_ways();
    apply(&mut driver, 62, Box::new([kinds::item(7, 1, 1, &value)]));
    let stored = kept(&driver, 7, 1, 1, settings::SettingSlot::Change).unwrap().clone();
    assert!(matches!(&stored, settings::Edit::Set(kept) if kept.value == value));
    let member = project_member(&mut driver);
    driver.send(engine::Event::Party(engine::PartyInput::ForgeRead {
        reply: ReplyTo::new(Token::new(63)),
        sign_in: member,
        scope: people::ApplicationScope::Project(1),
        query: engine::ForgeRead::Setting { kind: 7, scope: 1, number: 1 },
    }));
    driver.settle();
    assert!(
        driver.delivered.iter().any(|delivery| matches!(delivery,
            Delivery::ForgeRead { reply, answer: engine::ForgeReadAnswer::Setting(Some(kept)) }
                if *reply == ReplyTo::new(Token::new(63)) && kept.value == value
        )),
        "{:?}",
        driver.delivered
    );
    let against = kinds::revision(&driver.store.rows);
    driver.send(kinds::change(
        member,
        against,
        64,
        Box::new([kinds::item(
            7,
            1,
            1,
            &settings::Value::Ways(settings::ProjectWays { drafts: false, goals: false, changes: false, way: 0 }),
        )]),
    ));
    driver.settle();
    assert!(driver.delivered.iter().any(|delivery| matches!(delivery,
        Delivery::WebReply { to, reply: people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Role { held: Some(people::Role::Member) })) }
            if *to == ReplyTo::new(Token::new(64))
    )), "{:?}", driver.delivered);
    assert_eq!(kinds::revision(&driver.store.rows), against);
    assert_eq!(kept(&driver, 7, 1, 1, settings::SettingSlot::Change), Some(&stored));
}

#[test]
fn a_proposal_of_a_projects_ways_is_kept_then_accepted() {
    let mut driver = kinds::driver();
    let value = project_ways();
    let (proposal, key) = propose_items(&mut driver, 65, Box::new([kinds::item(7, 1, 1, &value)]));
    let proposed = kept(&driver, 7, 1, 1, settings::SettingSlot::Proposal(key)).expect("project ways proposed").clone();
    assert!(matches!(&proposed, settings::Edit::Set(kept) if kept.value == value));
    assert_eq!(kept(&driver, 7, 1, 1, settings::SettingSlot::Change), None);
    assert_eq!(kinds::revision(&driver.store.rows), 1);
    ask(
        &mut driver,
        66,
        people::Ask::DecideProposal {
            project: people::DEPLOYMENT,
            proposer: people::Proposer::Party(1),
            proposal,
            decision: people::ProposalDecision::Accept,
            running: Some(people::Running::Stays),
        },
    );
    assert_eq!(kinds::revision(&driver.store.rows), 2, "{:?}", driver.delivered);
    assert_eq!(kept(&driver, 7, 1, 1, settings::SettingSlot::Change), Some(&proposed));
    assert_eq!(kept(&driver, 7, 1, 1, settings::SettingSlot::Proposal(key)), None);
}

#[test]
fn a_project_member_reads_its_kinds_at_their_revisions() {
    use temper_engine_domain_forge_change::Route;
    let mut driver = kinds::driver();
    let earlier = [settings::Value::Rule(rule()), settings::Value::Questions(Route::Up), project_ways()];
    let later = [
        settings::Value::Rule(settings::LandingRule { ci: true, ..rule() }),
        settings::Value::Questions(Route::None),
        settings::Value::Ways(settings::ProjectWays { drafts: false, goals: false, changes: true, way: 0 }),
    ];
    for (key, values) in [(67, &earlier), (68, &later)] {
        apply(
            &mut driver,
            key,
            [2, 6, 7].into_iter().zip(values).map(|(kind, value)| kinds::item(kind, 1, 1, value)).collect(),
        );
    }
    let member = project_member(&mut driver);
    for (revision, values) in [(2, earlier), (3, later)] {
        for (kind, value) in [2, 6, 7].into_iter().zip(values) {
            const REPLY: ReplyTo = ReplyTo::new(Token::new(70));
            driver.delivered.clear();
            driver.send(engine::Event::Party(engine::PartyInput::ForgeRead {
                reply: REPLY,
                sign_in: member,
                scope: people::ApplicationScope::Project(1),
                query: engine::ForgeRead::RevisionSetting { kind, scope: 1, number: 1, revision },
            }));
            driver.settle();
            assert!(
                driver.delivered.iter().any(|delivery| matches!(delivery,
                    Delivery::ForgeRead { reply: to, answer: engine::ForgeReadAnswer::Setting(Some(kept)) }
                        if *to == REPLY && kept.value == value
                )),
                "kind {kind}, revision {revision}: {:?}",
                driver.delivered
            );
        }
    }
}

#[test]
fn the_retired_forge_charter_number_is_refused_for_changes_and_proposals() {
    for proposal in [false, true] {
        for number in [3, 4] {
            let mut driver = kinds::driver();
            let before = kinds::revision(&driver.store.rows);
            let items = Box::new([kinds::item(3, 0, number, &settings::Value::Charter(2))]);
            let event = if proposal {
                kinds::proposal(driver.session(), before, 90, items)
            } else {
                kinds::change(driver.session(), before, 90, items)
            };
            driver.send(event);
            driver.settle();
            assert!(
                driver.delivered.iter().any(|delivery| matches!(
                    delivery,
                    Delivery::WebReply { reply: people::Reply::Outcome(people::Outcome::Refused(_)), .. }
                )),
                "unknown forge-charter {number}: {:?}",
                driver.delivered
            );
            assert_eq!(kinds::revision(&driver.store.rows), before);
            assert_eq!(kept(&driver, 3, 0, number, settings::SettingSlot::Change), None);
        }
    }
}
