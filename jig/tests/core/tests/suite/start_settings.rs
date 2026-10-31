//! Direct selected Core over settings rows from the reference root's real seed
//! (jig/docs/design/domain/engine.md, 6, 17.7).
use crate::settings_support as support;
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
                        assert_ne!(h.core.restore_core(row.clone(), &h.env.limits), core::Restored::Rejected);
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

    fn link(&mut self) -> Action {
        phase(&self.core.start_done(&self.env, Phase::LoadCore, &mut self.out), Phase::LoadInbox);
        let mut previous = Phase::LoadInbox;
        for connector in self.core.connectors.clone() {
            let next = Phase::RestoreConnector { connector };
            phase(&self.core.start_done(&self.env, previous, &mut self.out), next);
            previous = next;
        }
        phase(&self.core.start_done(&self.env, previous, &mut self.out), Phase::Link);
        self.core.start_done(&self.env, Phase::Link, &mut self.out)
    }
}

fn phase(action: &Action, expected: Phase) {
    assert!(matches!(action, Action::Step(found) if *found == expected));
}

#[test]
fn changed_defaults_commit_the_release_revision_before_sweep_and_switch() {
    for pending in [false, true] {
        let mut h = Harness::stored(true, pending);
        let Action::Commit(release) = h.link() else { panic!("changed defaults commit before Sweep") };
        let mut default = None;
        let mut revision = None;
        let mut copied = false;
        let mut erased = false;
        let mut extent = false;
        for _ in 0..h.out.len() {
            let core::Request::Write(write) = h.out.pop().expect("release write") else { panic!("release writes") };
            match write {
                core::Write::Save(core::Record::Core(core::CoreRecord::Settings(row))) => match row {
                    settings::Record::Default { setting, default: value } => {
                        assert_eq!(setting, settings::Setting::Template(1));
                        default = Some(value);
                    }
                    settings::Record::Revision { history, revision: value } => {
                        assert!(!history);
                        revision = Some(value);
                    }
                    settings::Record::Extent { oldest } => {
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
                core::Write::Save(_) => panic!("unexpected release write"),
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
            &[settings::Item {
                setting: settings::Setting::Template(1),
                edit: settings::Edit::Set(settings::Value::Template(template)),
            }]
        );
        assert!(copied && erased && extent);
        assert_eq!(h.core.current.newest(), 2);
        let early = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            h.core.start_sweep(&h.env, SweepInput::Finished, &mut h.out)
        }));
        assert!(early.is_err());
        phase(&h.core.start_resume(&h.env, release, &mut h.out), Phase::Sweep);
        phase(&h.core.start_sweep(&h.env, SweepInput::Finished, &mut h.out), Phase::Switch);
    }
}

#[test]
fn unchanged_settings_go_directly_to_sweep_without_a_release_commit() {
    let mut h = Harness::stored(false, false);
    phase(&h.link(), Phase::Sweep);
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
        h.core.restore_core(
            core::CoreRecord::Settings(settings::Record::Changed {
                setting: settings::Setting::Charter(1),
                changed: settings::Changed { revision: 1, value: Some(settings::Value::Charter(Box::new(charter))) },
            }),
            &h.env.limits
        ),
        core::Restored::Live
    );
    let expected = Failure::Settings {
        failure: settings::Failure { setting: settings::Setting::Charter(1), fault: settings::Fault::Reference },
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
    assert!(h.core.restore_people_row(&record));
    drop(core::step(&mut h.core, &h.env, core::Event::People(people::Event::Restore { record })));
    assert_eq!(
        h.core.restore_core(
            core::CoreRecord::Settings(settings::Record::Proposal {
                proposer: jig_domain_people::Proposer::Party(1),
                proposal: 1,
                value: settings::Proposal {
                    against: 1,
                    items: Box::new([settings::Item {
                        setting: settings::Setting::Template(1),
                        edit: settings::Edit::Reset,
                    }])
                },
            }),
            &h.env.limits
        ),
        core::Restored::Live
    );
    let _panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| h.link()))
        .expect_err("a proposal requires its retained links");
    assert_eq!(h.core.start_phase(), Some(Phase::Link));
    assert_eq!(h.core.start_failure(), None);
    assert!(h.out.is_empty());
}
