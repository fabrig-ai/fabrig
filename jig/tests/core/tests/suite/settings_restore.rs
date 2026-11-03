//! Restoring a version through the real store (jig/docs/design/domain/engine.md, 17.5, 17.7, 17.10;
//! jig/docs/design/domain/connectors.md, 17; jig/docs/design/domain/people.md, 5.1 and 5.2).
use crate::settings_support::*;
use jig_core_world::{
    client_peer::ClientPeer,
    world::{Boot, Setup, World},
};
use jig_domain_core::{self as core, documents, reads, settings};
use jig_domain_people as people;
use jig_test_connector as connector;
use jig_test_domain as root;
use skein_lib::{ReplyTo, Token};

const HISTORY: u32 = 64;
const PROJECT: u32 = 1;
const SECOND: u32 = 2;
const APP: settings::Setting = settings::Setting::Application { kind: 1, scope: PROJECT, number: 1 };
const REASON: &[u8] = b"Return to the reviewed settings";

fn setup() -> (World, u64) {
    let mut world = start(HISTORY);
    let owner = sign_in(&mut world);
    (world, owner)
}

fn configured(config: root::Config, limits: &root::Limits) -> (World, u64) {
    let mut world = World::new(Setup {
        seed: SEED,
        configuration: Some(config),
        limits: *limits,
        boot: Boot::Empty,
        ..Setup::default()
    });
    assert!(world.domain.ready(), "{:?}", world.domain.start_failure());
    world.client_peer = Some(ClientPeer::default());
    let owner = sign_in(&mut world);
    world.send(root::Event::Core(core::Event::People(people::Event::Roles {
        project: PROJECT,
        holdings: Box::new([people::Holding { person: OWNER, role: people::Role::Owner }]),
    })));
    (world, owner)
}

fn application(history: u32) -> (World, u64) {
    let (mut config, mut limits) = configuration();
    config.core.defaults.rules.period_spend = 200;
    config.core.kinds = Box::new([settings::Kind {
        number: 1,
        scope: settings::KindScope::Project,
        connector: 1,
        value_bytes: 64,
        per_scope: 2,
        charters: 1,
    }]);
    config.first.settings = Some(connector::SettingsKind {
        kind: 1,
        project: true,
        value_bytes: 64,
        per_scope: 2,
        charters: true,
        defaults: Box::new([(PROJECT, 1, b"d".as_slice().into())]),
    });
    config.core.grants = Box::new([people::ActGrant { connector: 1, may: Box::new([people::ActKind::RemoveProject]) }]);
    limits.core.people.act_grants = 1;
    limits.core.settings.history = history;
    configured(config, &limits)
}

fn app(value: &[u8]) -> settings::Item {
    settings::Item { setting: APP, edit: settings::Edit::Set(settings::Value::Application(value.into())) }
}

fn accept(world: &mut World, owner: u64, proposal: u64) {
    let reply =
        decide(world, owner, 90, OWNER, proposal, people::ProposalDecision::Accept, Some(people::Running::Stays));
    assert!(
        matches!(
            reply,
            people::Reply::Outcome(people::Outcome::ProposalDecided { choice: people::ProposalChoice::Accepted, .. })
        ),
        "{reply:?}"
    );
}

fn review(world: &mut World, owner: u64, proposal: u64) -> documents::SettingsReview {
    let document = proposal_document(world, owner, OWNER, proposal);
    assert!(document.reason.bytes.starts_with(b"restores revision "));
    let documents::ProposalAction::Settings { review: Some(review), .. } = document.action else {
        panic!("settings review")
    };
    *review
}

fn later(world: &mut World, owner: u64) -> Box<[settings::Item]> {
    let earlier = charter_and_template(world);
    changed(&change(world, owner, 1, 1, earlier.clone()), 2);
    let next = Box::new([
        run_charter(world, b"Instructions from revision three"),
        settings::Item { setting: settings::Setting::Template(2), edit: settings::Edit::Reset },
    ]);
    changed(&change(world, owner, 2, 2, next), 3);
    earlier
}

fn no_left_out(reply: &people::Reply) -> u64 {
    let (proposal, left_out) = restored(reply);
    assert!(left_out.is_empty());
    proposal
}

#[test]
#[ignore = "engine.md, 17.10: Restore is not admitted by people (restore-request)"]
fn restoring_a_revision_proposes_what_later_revisions_replaced() {
    let (mut world, owner) = setup();
    let earlier = later(&mut world, owner);
    let proposal = no_left_out(&restore(&mut world, owner, 3, 2, REASON));
    let document = proposal_document(&mut world, owner, OWNER, proposal);
    assert_eq!(document.reason.bytes.as_ref(), b"restores revision 2: Return to the reviewed settings");
    let reviewed = review(&mut world, owner, proposal);
    assert_eq!(reviewed.newest, 3);
    assert_eq!(reviewed.items.len(), 2);
    let charter = reviewed.items.iter().find(|item| item.setting == settings::Setting::Charter(1)).expect("charter");
    assert_eq!(charter.edit, reads::RevisionEdit::Set);
    let settings::Edit::Set(value) = &earlier[0].edit else { panic!("set charter") };
    assert_eq!(charter.would_be.as_ref(), Some(value));
    let template = reviewed.items.iter().find(|item| item.setting == settings::Setting::Template(2)).expect("template");
    assert_eq!(template.edit, reads::RevisionEdit::Removed);
    assert_eq!(template.would_be, None);
    accept(&mut world, owner, proposal);
    let settings::Value::Charter(expected) = value else { panic!("charter") };
    assert_eq!(world.domain.settings().charter(1), Some(expected.as_ref()));
    assert!(world.domain.settings().template(2).is_none());
    assert_eq!(world.domain.settings().newest(), 4);
    world.close();
}

#[test]
#[ignore = "engine.md, 17.10: Restore is not admitted by people (restore-request)"]
fn a_restore_reads_every_page_of_the_history_after_r() {
    // Three later records cross two pages of two rows each.
    const PAGE_ROWS: u32 = 2;
    let (config, mut limits) = configuration();
    limits.core.documents.rows = PAGE_ROWS;
    let (mut world, owner) = configured(config, &limits);
    let earlier = run_charter(&world, b"Revision two");
    changed(&change(&mut world, owner, 1, 1, Box::new([earlier.clone()])), 2);
    let edits = Box::new([run_charter(&world, b"Revision three"), template_edit(&world, 7)]);
    changed(&change(&mut world, owner, 2, 2, edits), 3);
    let edit = run_charter(&world, b"Revision four");
    changed(&change(&mut world, owner, 3, 3, Box::new([edit])), 4);
    let edit = template_edit(&world, 9);
    changed(&change(&mut world, owner, 4, 4, Box::new([edit])), 5);
    let proposal = no_left_out(&restore(&mut world, owner, 5, 2, REASON));
    let reviewed = review(&mut world, owner, proposal);
    assert_eq!(reviewed.items.len(), 2);
    let charter = reviewed.items.iter().find(|item| item.setting == earlier.setting).expect("charter from older page");
    let settings::Edit::Set(value) = earlier.edit else { panic!("charter") };
    assert_eq!(charter.would_be, Some(value));
    let template = reviewed
        .items
        .iter()
        .find(|item| item.setting == settings::Setting::Template(1))
        .expect("template from older page");
    assert_eq!(template.edit, reads::RevisionEdit::Reset);
    assert_eq!(
        template.would_be,
        Some(settings::Value::Template(Box::new(configuration().0.core.defaults.templates[0].clone())))
    );
    world.close();
}

#[test]
#[ignore = "engine.md, 17.10: Restore is not admitted by people (restore-request)"]
fn a_restore_of_a_forgotten_revision_names_the_oldest_kept() {
    let mut world = start(2);
    let owner = sign_in(&mut world);
    for number in 2..=5 {
        let item = template_edit(&world, u64::from(number));
        changed(
            &change(&mut world, owner, u8::try_from(number).expect("request key"), number - 1, Box::new([item])),
            number,
        );
    }
    let reads::Answer::Revisions(history) = read(&mut world, owner, reads::Read::Revisions { before: None }) else {
        panic!("history")
    };
    let oldest = history.rows.iter().last().expect("oldest history row").head.number;
    assert_eq!(oldest, 3);
    assert_eq!(
        restore(&mut world, owner, 5, 1, REASON),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Forgotten { oldest }))
    );
    assert!(inventory(&mut world, owner).pending.is_empty());
    world.close();
}

#[test]
#[ignore = "engine.md, 17.10: Restore is not admitted by people (restore-request)"]
fn a_restore_of_the_newest_is_empty() {
    let (mut world, owner) = setup();
    assert_eq!(
        restore(&mut world, owner, 1, 1, REASON),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::SettingsUnchanged
        }))
    );
    assert!(inventory(&mut world, owner).pending.is_empty());
    world.close();
}

#[test]
#[ignore = "engine.md, 17.10: Restore is not admitted by people (restore-request)"]
fn a_revision_made_while_a_restore_loads_refuses_it_stale() {
    let (mut world, owner) = setup();
    let edit = template_edit(&world, 7);
    changed(&change(&mut world, owner, 1, 1, Box::new([edit])), 2);
    // Both requests enter the world's queue before its store can answer the first load.
    let edit = template_edit(&world, 9);
    world.send_all(vec![
        root::Event::Core(core::Event::People(people::Event::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(41)),
            sign_in: owner,
            key: [2; 16],
            ask: people::Ask::Restore { revision: 1, reason: REASON.into() },
        })),
        root::Event::Core(core::Event::ChangeSettings {
            reply_to: ReplyTo::new(Token::new(42)),
            sign_in: owner,
            key: [3; 16],
            digest: [3; 32],
            change: Box::new(settings::Change {
                against: 2,
                items: Box::new([edit]),
                running: settings::Running::Stays,
            }),
        }),
    ]);
    let replies = &world.client_peer.as_ref().expect("client").delivered;
    assert!(replies.iter().any(|(_, delivery)| matches!(delivery, root::Delivery::Core(core::Held::PeopleReply { to, reply: people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Stale { newest: 3 })) }) if *to == ReplyTo::new(Token::new(41)))), "{replies:?}");
    assert_eq!(world.domain.settings().newest(), 3);
    assert!(inventory(&mut world, owner).pending.is_empty());
    world.close();
}

#[test]
#[ignore = "engine.md, 17.10: Restore is not admitted by people (restore-request)"]
fn a_restore_past_the_whole_set_is_refused_too_large() {
    // A restore may carry one project's policy; two later policies exceed it.
    let (mut config, limits) = configuration();
    config.core.defaults.rules.period_spend = 300;
    let (mut world, owner) = configured(config, &limits);
    assert_eq!(
        ask(&mut world, owner, 1, people::Ask::MakeProject { name: "Second project".into(), policy: 1 }),
        people::Reply::Outcome(people::Outcome::ProjectMade { project: SECOND })
    );
    for (project, against) in [(PROJECT, 1), (SECOND, 2)] {
        let item = project_policy_item(&world, project, 120);
        changed(
            &change(&mut world, owner, u8::try_from(against + 1).expect("key"), against, Box::new([item])),
            against + 1,
        );
    }
    assert_eq!(
        restore(&mut world, owner, 4, 1, REASON),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Settings {
                setting: settings::Setting::ProjectPolicy { project: SECOND },
                fault: settings::Fault::Count
            },
        }))
    );
    assert!(inventory(&mut world, owner).pending.is_empty());
    world.close();
}

#[test]
#[ignore = "engine.md, 17.10: Restore is not admitted by people (restore-request)"]
fn a_restore_proposes_a_project_policy_it_replaced() {
    let (mut config, limits) = configuration();
    config.core.defaults.rules.period_spend = 200;
    let (mut world, owner) = configured(config, &limits);
    let before = project_document(&mut world, owner, PROJECT).policy;
    let item = project_policy_item(&world, PROJECT, 120);
    changed(&change(&mut world, owner, 1, 1, Box::new([item])), 2);
    let proposal = no_left_out(&restore(&mut world, owner, 2, 1, REASON));
    let reviewed = review(&mut world, owner, proposal);
    assert_eq!(reviewed.items.len(), 1);
    let item = reviewed.items.get(0).expect("project policy");
    assert_eq!(item.setting, settings::Setting::ProjectPolicy { project: PROJECT });
    assert_eq!(item.edit, reads::RevisionEdit::Set);
    let Some(settings::Value::Policy(policy)) = &item.would_be else { panic!("whole policy") };
    assert_eq!(policy.as_ref(), &configuration().0.core.seed_projects[0].policy);
    accept(&mut world, owner, proposal);
    assert_eq!(project_document(&mut world, owner, PROJECT).policy, before);
    world.close();
}

fn connector_row(world: &World, key: connector::RecordKey) -> Option<&connector::Record> {
    world.store.rows.get(&root::Key::Connector { number: 1, key }).map(|row| {
        let root::Record::Connector { record, .. } = row else { panic!("connector record") };
        record
    })
}

#[test]
#[ignore = "engine.md, 17.10: Restore is not admitted by people (restore-request)"]
fn a_restore_of_an_applications_kind_takes_its_connectors_kept_value() {
    let (mut world, owner) = application(HISTORY);
    changed(&change(&mut world, owner, 1, 1, Box::new([app(b"v2")])), 2);
    changed(&change(&mut world, owner, 2, 2, Box::new([app(b"v3")])), 3);
    assert_eq!(
        connector_row(
            &world,
            connector::RecordKey::SettingReplaced { kind: 1, scope: PROJECT, number: 1, revision: 3 }
        ),
        Some(&connector::Record::SettingReplaced {
            kind: 1,
            scope: PROJECT,
            number: 1,
            revision: 3,
            value: b"v2".as_slice().into()
        })
    );
    let proposal = no_left_out(&restore(&mut world, owner, 3, 2, REASON));
    let reviewed = review(&mut world, owner, proposal);
    assert_eq!(reviewed.items.len(), 1);
    assert_eq!(reviewed.items.get(0).expect("app").edit, reads::RevisionEdit::Set);
    let key = reviewed.key.expect("connector proposal key");
    assert_eq!(
        connector_row(
            &world,
            connector::RecordKey::SettingProposal { kind: 1, scope: PROJECT, number: 1, proposal: key }
        ),
        Some(&connector::Record::SettingProposal {
            kind: 1,
            scope: PROJECT,
            number: 1,
            proposal: key,
            edit: connector::SettingEdit::Set(b"v2".as_slice().into())
        })
    );
    accept(&mut world, owner, proposal);
    assert_eq!(world.domain.setting(&world.env(), 1, 1, PROJECT, 1).as_deref(), Some(b"v2".as_slice()));
    world.close();
}

#[test]
#[ignore = "connectors.md, 17: a scoped default survives project deletion; Restore also awaits restore-request"]
fn a_restore_leaves_out_and_names_a_deleted_projects_items() {
    let (mut world, owner) = application(HISTORY);
    let items = Box::new([app(b"on"), project_policy_item(&world, PROJECT, 120), run_charter(&world, b"after")]);
    changed(&change(&mut world, owner, 1, 1, items), 2);
    world.send(root::Event::Core(core::Event::Act {
        reply_to: ReplyTo::new(Token::new(SEED)),
        connector: 1,
        key: [2; 16],
        carried: None,
        act: Box::new(people::Act::RemoveProject { project: PROJECT, grace: None }),
    }));
    world.advance(skein_lib::Duration::from_secs(6));
    let (proposal, left_out) = restored(&restore(&mut world, owner, 3, 1, REASON));
    assert_eq!(left_out.as_ref(), &[APP, settings::Setting::ProjectPolicy { project: PROJECT }]);
    let reviewed = review(&mut world, owner, proposal);
    assert_eq!(reviewed.items.len(), 1);
    assert_eq!(reviewed.items.get(0).expect("remaining charter").setting, settings::Setting::Charter(1));
    world.close();
}

#[test]
#[ignore = "engine.md, 17.10: history pruning sends no ForgetReplaced (restore-request)"]
fn forgetting_a_revision_erases_what_its_items_replaced_at_the_connector() {
    let (mut world, owner) = application(2);
    for (against, value) in [(1, b"v2".as_slice()), (2, b"v3"), (3, b"v4"), (4, b"v5")] {
        changed(
            &change(&mut world, owner, u8::try_from(against).expect("key"), against, Box::new([app(value)])),
            against + 1,
        );
    }
    assert!(!world.store.rows.contains_key(&key(settings::Key::History(2))));
    assert!(
        connector_row(
            &world,
            connector::RecordKey::SettingReplaced { kind: 1, scope: PROJECT, number: 1, revision: 2 }
        )
        .is_none()
    );
    assert_eq!(
        connector_row(
            &world,
            connector::RecordKey::SettingReplaced { kind: 1, scope: PROJECT, number: 1, revision: 3 }
        ),
        Some(&connector::Record::SettingReplaced {
            kind: 1,
            scope: PROJECT,
            number: 1,
            revision: 3,
            value: b"v2".as_slice().into()
        })
    );
    // Revision 3 has a replaced row. Observe its later erasure as well,
    // so the check of revision 2 (which replaced a default) cannot pass vacuously.
    changed(&change(&mut world, owner, 5, 5, Box::new([app(b"v6")])), 6);
    assert!(!world.store.rows.contains_key(&key(settings::Key::History(3))));
    assert!(
        connector_row(
            &world,
            connector::RecordKey::SettingReplaced { kind: 1, scope: PROJECT, number: 1, revision: 3 }
        )
        .is_none()
    );
    assert_eq!(
        connector_row(
            &world,
            connector::RecordKey::SettingReplaced { kind: 1, scope: PROJECT, number: 1, revision: 4 }
        ),
        Some(&connector::Record::SettingReplaced {
            kind: 1,
            scope: PROJECT,
            number: 1,
            revision: 4,
            value: b"v3".as_slice().into()
        })
    );
    world.close();
}

#[test]
#[ignore = "engine.md, 17.10: Restore is not admitted by people (restore-request)"]
fn a_pending_restore_survives_a_restart_and_is_accepted() {
    let (mut world, owner) = setup();
    let earlier = later(&mut world, owner);
    let proposal = no_left_out(&restore(&mut world, owner, 3, 2, REASON));
    let before = proposal_document(&mut world, owner, OWNER, proposal);
    restart(&mut world, configuration().0);
    assert_eq!(proposal_document(&mut world, owner, OWNER, proposal), before);
    accept(&mut world, owner, proposal);
    let committed = revision(&world, 4, false);
    for item in &earlier {
        assert!(committed.items.iter().any(|recorded| recorded.setting == item.setting && recorded.edit == item.edit));
    }
    assert!(!world.store.rows.contains_key(&proposal_set_key(OWNER, proposal)));
    world.close();
}
