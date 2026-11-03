//! Kinds judged, kept and used (temper/docs/design/domain/forge.md, 8.7, 8.9 and 18).
use jig_domain_core::settings as s;
use jig_domain_people as people;
use skein_fake_forge_domain::api as raw;
use skein_lib::{Duration, ReplyTo, Token};
use temper_engine_domain::{self as engine, Delivery, Record};
use temper_engine_domain_forge::{self as forge, settings};
use temper_engine_domain_world::kinds;

fn rule() -> settings::LandingRule {
    settings::LandingRule {
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
    let against = kinds::revision(&driver.store.rows);
    driver.send(kinds::proposal(
        driver.session(),
        against,
        key,
        Box::new([kinds::item(1, 0, 2, &settings::Value::Rule(rule()))]),
    ));
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
    ask(&mut driver, 26, people::Ask::RemoveProject { project: 1, grace: None });
    driver.fire_at(Duration::from_secs(5));
    driver.pump();
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
