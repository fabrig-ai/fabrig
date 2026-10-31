//! Stored settings and policy fits precede linking (domain/engine.md, 6, 17.7).
use super::{Action, Core, Env, Failure, Key, Phase, Queue, Time, Wall, config, limits, people};
use crate::{CoreRecord, HeldLimit, Limits, RestartRequest, Restored, settings};
use settings::{Changed, CharterVersion, Edit, Fault, Head, Item, Record, Revision, Setting, Value};

fn loaded(selected: bool) -> (Core, Env<Limits>) {
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    env.limits.authority.projects = 2;
    let mut core = Core::new(config(401), &env.limits).expect("configured core");
    if selected {
        let _begin = core.start_begin(false, env.wall);
    }
    let mut header = core.counters.deployment();
    header.commits = 1;
    header.projects = 10;
    assert_eq!(core.restore_core(CoreRecord::Deployment(header), &env.limits), Restored::Deployment { commits: 1 });
    (core, env)
}

fn restore(core: &mut Core, env: &Env<Limits>, row: Record) -> Restored {
    core.restore_core(CoreRecord::Settings(row), &env.limits)
}

fn head(number: u32) -> Record {
    Record::Revision {
        history: false,
        revision: Revision {
            head: Head {
                number,
                against: number.saturating_sub(1),
                at: Wall::EPOCH,
                by: settings::Decider::FirstStart,
                proposal: None,
                moved: false,
            },
            items: Box::new([]),
        },
    }
}

fn stored_defaults(core: &Core) -> Vec<Record> {
    core.current
        .defaults
        .iter()
        .map(|(&setting, default)| Record::Default { setting, default: default.clone() })
        .collect()
}

fn settings_rows(core: &Core) -> Vec<Record> {
    let mut rows = stored_defaults(core);
    rows.push(head(1));
    rows.push(Record::Extent { oldest: 0 });
    rows
}

fn policy() -> people::PolicyValue {
    crate::policy::snapshot(&config(401).seed_projects[0].policy.policy, &[], &[]).expect("stored policy")
}

#[test]
fn stored_policy_names_and_project_counters_are_writers_rules() {
    for (project, name) in [(1, ""), (11, "Project")] {
        let (mut default, env) = loaded(false);
        assert!(!default.restore_policy(&env.limits, project, name.into(), policy()));
        let (mut selected, env) = loaded(true);
        assert!(selected.restore_policy(&env.limits, project, name.into(), policy()));
        assert_eq!(selected.names.get(&project).map(AsRef::as_ref), Some(name));
        assert!(selected.authority.policy(project).is_some());
        assert_eq!(selected.start_failure(), None);
    }
    for selected in [false, true] {
        let (mut core, env) = loaded(selected);
        assert!(core.restore_policy(&env.limits, 1, "Same".into(), policy()));
        assert_eq!(core.restore_policy(&env.limits, 2, "Same".into(), policy()), selected);
        if selected {
            assert_eq!(core.names.get(&1), core.names.get(&2));
        }
    }
}

#[test]
fn stored_policy_byte_and_template_bounds_refuse_without_failing_the_restart_script() {
    for templates in [false, true] {
        let (mut core, mut env) = loaded(true);
        let mut value = policy();
        if templates {
            env.limits.templates = 0;
            value.templates = Box::new([people::TemplateOverride {
                template: 1,
                executor: None,
                priority: None,
                budget: None,
                deadline: None,
            }]);
        } else {
            env.limits.policy_bytes = people::policy_bytes(&value).expect("policy bytes") - 1;
        }
        assert!(!core.restore_policy(&env.limits, 1, "Project".into(), value));
        assert_eq!(core.start_failure(), Some(Failure::ProjectPolicy { project: 1 }));
        assert_eq!(core.restore_failure(), Some(RestartRequest::ProjectPolicy { project: 1 }));
        assert_eq!(core.restart_failure(), None);
        assert!(matches!(
            core.start_done(&env, Phase::LoadCore, &mut Queue::uncapped()),
            Action::Refused(Failure::ProjectPolicy { project: 1 })
        ));
    }
}

#[test]
fn every_policy_table_reports_the_stored_key_at_capacity() {
    for table in 0..4 {
        let (mut core, env) = loaded(true);
        match table {
            0 => core.projects = skein_lib::List::with_capacity(0),
            1 => core.names = skein_lib::Map::with_capacity(0),
            2 => core.permission_roles = skein_lib::Map::with_capacity(0),
            3 => core.template_overrides = skein_lib::Map::with_capacity(0),
            _ => unreachable!(),
        }
        assert!(!core.restore_policy(&env.limits, 7, "Project".into(), policy()));
        assert_eq!(
            core.start_failure(),
            Some(Failure::LiveSet {
                set: crate::start::LiveSet::Projects,
                key: Key::People(people::Key::Policy(7)),
                held: 1,
                limit: 0,
            })
        );
    }
}

#[test]
fn duplicate_policy_rows_assert_before_replacing_the_key() {
    let (mut core, env) = loaded(true);
    assert!(core.restore_policy(&env.limits, 1, "First".into(), policy()));
    let duplicate = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        core.restore_policy(&env.limits, 1, "Second".into(), policy())
    }));
    assert!(duplicate.is_err());
    assert_eq!(core.names.get(&1).map(AsRef::as_ref), Some("First"));
}

#[test]
fn settings_finish_drops_cross_record_revision_rules() {
    enum Rule {
        DefaultAhead,
        ChangeAhead,
        VersionAhead,
        VersionAtOrigin,
        ExtentAtHead,
    }
    for rule in [Rule::DefaultAhead, Rule::ChangeAhead, Rule::VersionAhead, Rule::VersionAtOrigin, Rule::ExtentAtHead] {
        for selected in [false, true] {
            let (mut core, env) = loaded(selected);
            let mut rows = settings_rows(&core);
            match rule {
                Rule::DefaultAhead => {
                    let Record::Default { default, .. } = rows.first_mut().expect("default row") else {
                        panic!("default")
                    };
                    default.revision = 2;
                }
                Rule::ChangeAhead => rows.push(Record::Changed {
                    setting: Setting::Charter(1),
                    changed: Changed {
                        revision: 2,
                        value: Some(Value::Charter(Box::new(core.charter(1).expect("charter").clone()))),
                    },
                }),
                Rule::VersionAhead | Rule::VersionAtOrigin => rows.push(Record::Charter {
                    charter: 1,
                    revision: if matches!(rule, Rule::VersionAhead) { 2 } else { 1 },
                    value: CharterVersion { charter: Some(Box::new(core.charter(1).expect("charter").clone())) },
                }),
                Rule::ExtentAtHead => {
                    let row = rows.last_mut().expect("extent");
                    *row = Record::Extent { oldest: 1 };
                }
            }
            for row in rows {
                assert_eq!(restore(&mut core, &env, row), Restored::Live);
            }
            if selected {
                super::step_phase(core.start_done(&env, Phase::LoadCore, &mut Queue::uncapped()), Phase::LoadInbox);
                assert_eq!(core.start_failure(), None);
            } else {
                assert!(!core.settings_restored(&env));
            }
        }
    }
}

#[test]
fn whole_settings_fit_is_checked_by_load_core() {
    let (mut core, env) = loaded(true);
    let mut rows = settings_rows(&core);
    let mut charter = core.charter(1).expect("charter").clone();
    charter.policy.model.name = Box::from(b"unconfigured".as_slice());
    rows.push(Record::Changed {
        setting: Setting::Charter(1),
        changed: Changed { revision: 1, value: Some(Value::Charter(Box::new(charter))) },
    });
    for row in rows {
        assert_eq!(restore(&mut core, &env, row), Restored::Live);
    }
    let expected = Failure::Settings {
        failure: settings::Failure { setting: Setting::Charter(1), fault: Fault::Reference },
        changed: true,
    };
    let mut out = Queue::uncapped();
    assert!(matches!(core.start_done(&env, Phase::LoadCore, &mut out), Action::Refused(found) if found == expected));
    assert_eq!(core.start_phase(), Some(Phase::LoadCore));
    assert_eq!(core.restart_failure(), None);
    assert!(out.is_empty());
}

#[test]
fn held_rows_past_each_limit_keep_their_typed_cause() {
    for versions in [false, true] {
        let (mut core, mut env) = loaded(true);
        let (row, which) = if versions {
            env.limits.settings.versions = 0;
            (Record::Charter { charter: 7, revision: 2, value: CharterVersion { charter: None } }, HeldLimit::Versions)
        } else {
            env.limits.settings.revisions = 0;
            assert_eq!(restore(&mut core, &env, head(1)), Restored::Live);
            (head(2), HeldLimit::Revisions)
        };
        assert_eq!(restore(&mut core, &env, row), Restored::Rejected);
        assert_eq!(core.start_failure(), Some(Failure::HeldPastLimit { which, held: 1, limit: 0 }));
        assert_eq!(core.restart_failure(), None);
    }
}

#[test]
fn a_proposal_without_its_tasks_counterpart_is_a_writer_rule() {
    for selected in [false, true] {
        let (mut core, env) = loaded(selected);
        for row in settings_rows(&core) {
            assert_eq!(restore(&mut core, &env, row), Restored::Live);
        }
        let row = Record::Proposal {
            proposer: people::Proposer::Party(1),
            proposal: 1,
            value: settings::Proposal {
                against: 1,
                items: Box::new([Item { setting: Setting::Charter(1), edit: Edit::Reset }]),
            },
        };
        assert_eq!(restore(&mut core, &env, row), Restored::Live);
        assert!(!core.settings_proposals_consistent());
        if selected {
            super::through_link(&mut core, &env, &mut Queue::uncapped());
            assert_eq!(core.start_failure(), None);
        } else {
            let _out = core.commit_restored_settings(&env, settings::writes_bound(&env.limits).expect("write bound"));
            assert_eq!(
                core.restore_failure(),
                Some(RestartRequest::Settings {
                    failure: settings::Failure { setting: Setting::Rules, fault: Fault::Shape },
                    changed: false,
                })
            );
        }
    }
}
