//! Direct Core over settings rows from the reference root's real seed
//! (jig/docs/design/domain/engine/restart.md, jig/docs/design/domain/engine/settings.md, 7).
use crate::settings_support as support;
use jig_core_world::fixtures::{link, phase};
use jig_domain_core::{
    self as core, settings,
    start::{Action, Failure, Phase, SweepInput},
};
use jig_domain_people as people;
use jig_test_domain as root;
use skein_lib::{Duration, Env, Queue, Time, Wall};

struct Harness {
    core: core::Core,
    env: Env<core::Limits>,
    out: Queue<core::Request>,
}

impl Harness {
    fn stored(changed: bool, pending: bool) -> Self {
        let world = support::start(64);
        let (mut config, limits) = support::configuration();
        if changed {
            config.core.defaults.templates[0].grace = Some(Duration::from_secs(6));
        }
        let mut h = Self {
            core: core::Core::new(config.core, &limits.core).expect("configured settings Core"),
            env: Env { now: Time::ZERO, wall: Wall::from_nanos(10), limits: limits.core },
            out: Queue::uncapped(),
        };
        phase(&h.core.start_begin(pending, h.env.wall), Phase::LoadCore);
        for row in world.store.rows.values() {
            if let root::Record::Core(core::Record::Core(row)) = row {
                match row {
                    core::CoreRecord::Deployment(_) | core::CoreRecord::Settings(_) => {
                        assert_ne!(h.core.restore(&h.env, core::Record::Core(row.clone())), core::Restore::Refused);
                    }
                    core::CoreRecord::Projection(_)
                    | core::CoreRecord::Call(_)
                    | core::CoreRecord::RunProof(_)
                    | core::CoreRecord::Turn(_)
                    | core::CoreRecord::Terminal(_)
                    | core::CoreRecord::ProposalDecision(_)
                    | core::CoreRecord::EscalationDecision(_) => {}
                }
            }
        }
        h
    }
}

#[test]
fn changed_defaults_commit_the_release_revision_before_sweep_and_switch() {
    for pending in [false, true] {
        let mut h = Harness::stored(true, pending);
        let Action::Commit(release) = link(&mut h.core, &h.env, &mut h.out) else {
            panic!("changed defaults commit before Sweep")
        };
        let mut default = None;
        let mut revision = None;
        let mut copied = false;
        let mut erased = false;
        let mut extent = false;
        for _ in 0..h.out.len() {
            let write = match h.out.pop().expect("release output") {
                core::Request::Write(write) => write,
                core::Request::Ask { ask: core::Ask::Catalogue { .. }, .. } => continue,
                other => panic!("unexpected release output: {other:?}"),
            };
            match write {
                core::Write::Save(core::Record::Core(core::CoreRecord::Settings(row))) => match row {
                    settings::Record::Application { .. } => unreachable!("an application's record: apps-store"),
                    settings::Record::Default { setting, default: value } => {
                        assert_eq!(setting, settings::Setting::Template(1));
                        default = Some(value);
                    }
                    settings::Record::Revision { history, revision: value } => {
                        assert!(!history);
                        revision = Some(value);
                    }
                    settings::Record::Extent { oldest, .. } => {
                        assert_eq!(oldest, 1);
                        extent = true;
                    }
                    settings::Record::Changed { .. }
                    | settings::Record::Charter { .. }
                    | settings::Record::Proposal { .. } => panic!("unexpected settings release row"),
                },
                core::Write::CopySettingsRevision(number) => {
                    assert_eq!(number, 1);
                    copied = true;
                }
                core::Write::Erase(key) => {
                    assert_eq!(key, core::Key::Core(core::CoreKey::Settings(settings::Key::Held(1))));
                    erased = true;
                }
                core::Write::Save(_) | core::Write::DropTranscript { .. } => panic!("unexpected release write"),
            }
        }
        let default = default.expect("changed default");
        assert_eq!(default.revision, 2);
        let settings::Value::Template(template) = default.value else { panic!("template default") };
        assert_eq!(template.grace, Some(Duration::from_secs(6)));
        let revision = revision.expect("release revision");
        assert_eq!(revision.head.number, 2);
        assert_eq!(revision.head.against, 1);
        assert_eq!(revision.head.by, settings::Decider::Release);
        assert_eq!(revision.head.proposal, None);
        assert_eq!(
            revision.items.as_ref(),
            &[settings::RevisionItem {
                setting: settings::Setting::Template(1),
                edit: settings::Edit::Set(settings::Value::Template(template)),
                replaced: settings::Replaced::Default,
            }]
        );
        assert!(copied && erased && extent);
        assert_eq!(h.core.current.newest(), 2);
        phase(&h.core.start_resume(&h.env, release, &mut h.out), Phase::Sweep);
        phase(&h.core.start_sweep(&h.env, SweepInput::Finished, &mut h.out), Phase::Switch);
    }
}

#[test]
fn unchanged_settings_tell_the_catalogue_without_a_release_revision() {
    let mut h = Harness::stored(false, false);
    phase(&link(&mut h.core, &h.env, &mut h.out), Phase::Sweep);
    assert!(matches!(h.out.pop(), Some(core::Request::Ask { ask: core::Ask::Catalogue { .. }, .. })));
    assert!(h.out.is_empty());
    assert_eq!(h.core.current.newest(), 1);
    phase(&h.core.start_sweep(&h.env, SweepInput::Finished, &mut h.out), Phase::Switch);
}

#[test]
fn stored_settings_whole_fit_refuses_at_load_core_without_root_completion() {
    let mut h = Harness::stored(false, false);
    let mut charter = h.core.current.charter(1).expect("seed charter").clone();
    charter.policy.model.name = Box::from(b"unconfigured".as_slice());
    assert_eq!(
        h.core.restore(
            &h.env,
            core::Record::Core(core::CoreRecord::Settings(settings::Record::Changed {
                setting: settings::Setting::Charter(1),
                changed: settings::Changed { revision: 1, value: Some(settings::Value::Charter(Box::new(charter))) },
            }))
        ),
        core::Restore::Kept
    );
    let expected = Failure::Settings {
        failure: settings::Failure {
            setting: settings::Setting::Charter(1),
            fault: settings::Fault::Reference,
            detail: None,
        },
        changed: true,
    };
    assert!(
        matches!(h.core.start_done(&h.env, Phase::LoadCore, &mut h.out), Action::Refused(found) if found == expected)
    );
    assert_eq!(h.core.start_phase(), Some(Phase::LoadCore));
    assert!(h.out.is_empty());
}

#[test]
fn stored_proposals_require_their_links_at_link() {
    let mut h = Harness::stored(false, false);
    let record = people::Stored::Person {
        number: 1,
        identity: people::Identity {
            key: people::IdentityKey { provider: 0, subject: Box::new([1]) },
            login: Box::new([]),
            name: Box::new([]),
        },
    };
    drop(core::step(&mut h.core, &h.env, core::Event::People(people::Event::Restore { record })));
    assert_eq!(
        h.core.restore(
            &h.env,
            core::Record::Core(core::CoreRecord::Settings(settings::Record::Proposal {
                proposer: jig_domain_people::Proposer::Party(1),
                proposal: 1,
                value: settings::Proposal {
                    key: None,
                    applications: Box::default(),
                    against: 1,
                    items: Box::new([settings::Item {
                        setting: settings::Setting::Template(1),
                        edit: settings::Edit::Reset,
                    }])
                },
            }))
        ),
        core::Restore::Kept
    );
    let expected = core::start::Failure::Link {
        record: core::Key::Core(core::CoreKey::Settings(settings::Key::Proposal {
            proposer: people::Proposer::Party(1),
            proposal: 1,
        })),
        linked: core::Linked::Record(core::Key::Tasks(jig_domain_tasks::Key::PersonProposal(1))),
    };
    assert!(matches!(link(&mut h.core, &h.env, &mut h.out), Action::Refused(failure) if failure == expected));
    assert_eq!(h.core.start_phase(), Some(Phase::Link));
    assert_eq!(h.core.start_failure(), Some(expected));
    assert!(h.out.is_empty());
}
