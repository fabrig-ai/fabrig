//! Both settings owners across changes, proposals and a release (connectors.md, 15, 17).

use crate::settings_support::*;
use jig_core_world::{
    client_peer::ClientPeer,
    world::{Boot, Setup, World},
};
use jig_domain_core::{
    documents, reads,
    settings::{self, application},
};
use jig_domain_people as people;
use jig_test_connector as connector;
use jig_test_domain as root;

const SETTING: settings::Setting = settings::Setting::Application { kind: 1, scope: 1, number: 1 };

fn config(default: &[u8]) -> (root::Config, root::Limits) {
    let (mut config, limits) = configuration();
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
        defaults: Box::new([(1, 1, default.into())]),
    });
    (config, limits)
}

fn item(edit: settings::Edit) -> Box<[settings::Item]> {
    Box::new([settings::Item { setting: SETTING, edit }])
}

fn set(value: &[u8]) -> Box<[settings::Item]> {
    item(settings::Edit::Set(settings::Value::Application(value.into())))
}

fn value(world: &mut World) -> Option<Box<[u8]>> {
    world.domain.setting(&world.env(), 1, 1, 1, 1)
}

fn connector_row(world: &World, key: connector::RecordKey) -> Option<&connector::Record> {
    world.store.rows.get(&root::Key::Connector { number: 1, key }).map(|row| {
        let root::Record::Connector { record, .. } = row else { panic!("connector-owned record") };
        record
    })
}

fn state(world: &mut World, signed: u64, follows: application::Follows, size: u32, charters: &[u32]) {
    let document = inventory(world, signed);
    let line = document.settings.iter().find(|line| line.setting == SETTING).expect("application setting");
    assert_eq!(
        line.follows,
        if follows == application::Follows::Default { documents::Follows::Default } else { documents::Follows::Change }
    );
    assert_eq!(
        line.last,
        match follows {
            application::Follows::Default => None,
            application::Follows::Changed { revision } | application::Follows::Removed { revision } => Some(revision),
        }
    );
    assert_eq!(line.application, Some(application::State { follows, size, charters: charters.into() }));
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one story follows both settings owners through changes, proposals and restart"
)]
fn changes_proposals_and_changed_defaults_keep_both_owners_in_step() {
    let (configuration, limits) = config(b"d");
    let mut world =
        World::new(Setup { configuration: Some(configuration), limits, boot: Boot::Empty, ..Setup::default() });
    world.client_peer = Some(ClientPeer::default());
    assert!(world.domain.ready());
    let owner = sign_in(&mut world);
    assert_eq!(value(&mut world).as_deref(), Some(b"d".as_slice()));
    state(&mut world, owner, application::Follows::Default, 1, &[]);

    changed(&change(&mut world, owner, 1, 1, set(b"on")), 2);
    assert_eq!(value(&mut world).as_deref(), Some(b"on".as_slice()));
    assert_eq!(
        connector_row(&world, connector::RecordKey::SettingChange { kind: 1, scope: 1, number: 1 }),
        Some(&connector::Record::SettingChange { kind: 1, scope: 1, number: 1, value: Some(b"on".as_slice().into()) })
    );
    state(&mut world, owner, application::Follows::Changed { revision: 2 }, 2, &[]);

    assert_eq!(
        change(&mut world, owner, 2, 2, set(b"!try again")),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::Judged {
            at: 0,
            setting: SETTING,
            why: b"refused: try again".as_slice().into()
        }))
    );
    assert_eq!(
        change(&mut world, owner, 3, 2, set(b"on")),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::SettingsUnchanged
        }))
    );
    assert_eq!(world.domain.settings().newest(), 2);
    assert_eq!(value(&mut world).as_deref(), Some(b"on".as_slice()));

    changed(&change(&mut world, owner, 4, 2, set(&1_u32.to_be_bytes())), 3);
    state(&mut world, owner, application::Follows::Changed { revision: 3 }, 4, &[1]);
    let removal = removing_charter(&world);
    assert_eq!(
        change(&mut world, owner, 5, 3, removal),
        people::Reply::Outcome(people::Outcome::Refused(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Settings { setting: SETTING, fault: settings::Fault::Reference }
        }))
    );

    let (member, signed) = party(&mut world, b"settings-member", Some((1, people::Role::Member)));
    let accepted = proposed(&propose(&mut world, signed, 1, 3, set(b"ok")));
    let accepted_key = accepted;
    assert!(
        matches!(connector_row(&world, connector::RecordKey::SettingProposal { proposal: accepted_key, kind: 1, scope: 1, number: 1 }), Some(connector::Record::SettingProposal { edit: connector::SettingEdit::Set(bytes), .. }) if bytes.as_ref() == b"ok")
    );
    let stale = proposed(&propose(&mut world, owner, 6, 3, set(b"no")));
    let stale_key = stale;
    assert!(
        connector_row(
            &world,
            connector::RecordKey::SettingProposal { proposal: stale_key, kind: 1, scope: 1, number: 1 }
        )
        .is_some()
    );
    restart(&mut world, config(b"d").0);
    assert_eq!(world.domain.settings().newest(), 3);
    assert_eq!(value(&mut world).as_deref(), Some(1_u32.to_be_bytes().as_slice()));
    assert!(matches!(
        decide(&mut world, owner, 7, member, accepted, people::ProposalDecision::Accept, Some(people::Running::Stays)),
        people::Reply::Outcome(people::Outcome::ProposalDecided { choice: people::ProposalChoice::Accepted, .. })
    ));
    assert_eq!(world.domain.settings().newest(), 4);
    assert_eq!(value(&mut world).as_deref(), Some(b"ok".as_slice()));
    state(&mut world, owner, application::Follows::Changed { revision: 4 }, 2, &[]);
    assert!(inventory(&mut world, owner).pending.is_empty());
    let accepted_revision = revision(&world, 4, false);
    assert_eq!(
        accepted_revision.head.proposal,
        Some((people::Proposer::Party(member), u32::try_from(accepted).expect("proposal number")))
    );
    assert_eq!(accepted_revision.head.by, settings::Decider::Party(OWNER));
    let reads::Answer::Ended(history) =
        read(&mut world, owner, reads::Read::Proposal { proposer: people::Proposer::Party(OWNER), proposal: stale })
    else {
        panic!("withdrawn proposal history")
    };
    let reads::EndedValue::PersonProposal { proposal, decision: Some(decision) } = history.value else {
        panic!("withdrawn proposal decision")
    };
    assert!(matches!(proposal.state, jig_domain_tasks::PersonProposalState::Withdrawn { .. }));
    assert_eq!(decision.choice, people::ProposalChoice::Withdrawn);
    for proposal in [accepted_key, stale_key] {
        assert!(
            connector_row(&world, connector::RecordKey::SettingProposal { proposal, kind: 1, scope: 1, number: 1 })
                .is_none()
        );
    }

    changed(&change(&mut world, owner, 8, 4, item(settings::Edit::Reset)), 5);
    assert_eq!(value(&mut world).as_deref(), Some(b"d".as_slice()));
    restart(&mut world, config(b"new").0);
    assert_eq!(world.domain.settings().newest(), 6);
    let release = revision(&world, 6, false);
    assert_eq!(release.head.by, settings::Decider::Release);
    assert_eq!(release.head.against, 5);
    assert_eq!(release.head.proposal, None);
    assert_eq!(
        release.applications.as_ref(),
        &[application::RevisionRecorded {
            kind: 1,
            scope: 1,
            number: 1,
            edit: application::SizedEdit::Set { size: 3 },
            replaced: application::Replaced::Default
        }]
    );
    assert_eq!(value(&mut world).as_deref(), Some(b"new".as_slice()));
    assert_eq!(
        connector_row(&world, connector::RecordKey::SettingDefault { kind: 1, scope: 1, number: 1 }),
        Some(&connector::Record::SettingDefault { kind: 1, scope: 1, number: 1, value: b"new".as_slice().into() })
    );
    state(&mut world, owner, application::Follows::Default, 3, &[]);
    world.close();
    assert_eq!(world.domain.settings().newest(), 6);
    assert_eq!(value(&mut world).as_deref(), Some(b"new".as_slice()));
}
