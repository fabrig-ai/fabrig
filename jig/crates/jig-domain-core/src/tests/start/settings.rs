//! Stored settings and policy fits precede linking (domain/engine/restart.md, domain/engine/settings.md, 7).
use crate::{
    CoreRecord, HeldLimit, Limits, Restored,
    settings::{Changed, CharterVersion, Fault, Head, Record, Revision, Setting, Value},
    tests::start::{Action, Core, Env, Failure, Key, Phase, Queue, Time, Wall, config, limits, people},
};
use skein_lib::List;

fn loaded() -> (Core, Env<Limits>) {
    let mut env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() };
    env.limits.authority.projects = 2;
    let mut core = Core::new(config(401), &env.limits).expect("configured core");
    {
        let _begin = core.start_begin(false, env.wall);
    }
    let mut header = core.counters.deployment();
    header.commits = 1;
    header.people = 1;
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
                by: crate::settings::Decider::FirstStart,
                proposal: None,
                moved: false,
            },
            items: Box::new([]),
            applications: Box::default(),
        },
    }
}

fn settings_rows(core: &Core) -> List<Record> {
    let mut rows = List::with_capacity(core.current.defaults.len().checked_add(3).expect("settings rows"));
    for (&setting, default) in &core.current.defaults {
        rows.push(Record::Default { setting, default: default.clone() }).expect("stored defaults");
    }
    rows.push(head(1)).expect("stored head");
    rows.push(Record::Extent { oldest: 0, keys: 1 }).expect("stored extent");
    rows
}

fn policy() -> people::PolicyValue {
    crate::admission::policy::snapshot(&config(401).seed_projects[0].policy.policy, &[], &[]).expect("stored policy")
}

#[test]
fn stored_policy_names_and_project_counters_are_writers_rules() {
    for (project, name) in [(0, "Zero"), (1, ""), (11, "Project")] {
        let (mut started, env) = loaded();
        assert!(started.restore_policy(&env.limits, project, name.into(), policy()));
        assert_eq!(started.names.get(&project).expect("stored name").as_ref(), name);
        assert!(started.authority.policy(project).is_some());
        assert_eq!(started.start_failure(), None);
    }
    {
        let (mut core, env) = loaded();
        assert!(core.restore_policy(&env.limits, 1, "Same".into(), policy()));
        assert!(core.restore_policy(&env.limits, 2, "Same".into(), policy()));
        {
            assert_eq!(core.names.get(&1), core.names.get(&2));
        }
    }
}

#[test]
fn stored_policy_byte_and_template_bounds_refuse_without_failing_the_restart_script() {
    for templates in [false, true] {
        let (mut core, mut env) = loaded();
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
        assert_eq!(core.start_failure(), Some(Failure::ProjectPolicy { project: 1 }));
        let Action::Refused(failure) = core.start_done(&env, Phase::LoadCore, &mut Queue::uncapped()) else {
            panic!("policy fit refused")
        };
        assert_eq!(failure, Failure::ProjectPolicy { project: 1 });
    }
}

#[test]
fn every_policy_table_reports_the_stored_key_at_capacity() {
    enum Table {
        Projects,
        Names,
        Permissions,
        Templates,
    }
    for table in [Table::Projects, Table::Names, Table::Permissions, Table::Templates] {
        let (mut core, env) = loaded();
        match table {
            Table::Projects => core.projects = List::with_capacity(0),
            Table::Names => core.names = skein_lib::Map::with_capacity(0),
            Table::Permissions => core.permission_roles = skein_lib::Map::with_capacity(0),
            Table::Templates => core.template_overrides = skein_lib::Map::with_capacity(0),
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
#[ignore = "fails under Cranelift, which cannot unwind yet: run it on LLVM (2026-11-12)"]
fn duplicate_policy_rows_assert_before_replacing_the_key() {
    let (mut core, env) = loaded();
    assert!(core.restore_policy(&env.limits, 1, "First".into(), policy()));
    let duplicate = std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
        core.restore_policy(&env.limits, 1, "Second".into(), policy())
    }));
    let _panic = duplicate.expect_err("duplicate policy asserts");
    assert_eq!(core.names.get(&1).expect("stored name").as_ref(), "First");
}

#[test]
fn settings_finish_drops_cross_record_revision_rules() {
    enum Rule {
        DefaultAhead,
        ChangeAhead,
        Version(u32),
        ExtentAtHead,
    }
    for rule in [Rule::DefaultAhead, Rule::ChangeAhead, Rule::Version(2), Rule::Version(1), Rule::ExtentAtHead] {
        {
            let (mut core, env) = loaded();
            let mut rows = settings_rows(&core);
            match rule {
                Rule::DefaultAhead => {
                    let Record::Default { default, .. } = rows.get_mut(0).expect("default row") else {
                        panic!("default")
                    };
                    default.revision = 2;
                }
                Rule::ChangeAhead => rows
                    .push(Record::Changed {
                        setting: Setting::Charter(1),
                        changed: Changed {
                            revision: 2,
                            value: Some(Value::Charter(Box::new(core.charter(1).expect("charter").clone()))),
                        },
                    })
                    .expect("one stored change"),
                Rule::Version(revision) => rows
                    .push(Record::Charter {
                        charter: 1,
                        revision,
                        value: CharterVersion {
                            overlay: None,
                            charter: Some(Box::new(core.charter(1).expect("charter").clone())),
                        },
                    })
                    .expect("one stored version"),
                Rule::ExtentAtHead => {
                    let row = rows.get_mut(rows.len() - 1).expect("extent");
                    *row = Record::Extent { oldest: 1, keys: 1 };
                }
            }
            for row in rows.into_boxed() {
                assert_eq!(restore(&mut core, &env, row), Restored::Live);
            }
            {
                super::step_phase(core.start_done(&env, Phase::LoadCore, &mut Queue::uncapped()), Phase::LoadInbox);
                assert_eq!(core.start_failure(), None);
            }
        }
    }
}

#[test]
fn whole_settings_fit_is_checked_by_load_core() {
    let (mut core, env) = loaded();
    let mut rows = settings_rows(&core);
    let mut charter = core.charter(1).expect("charter").clone();
    charter.policy.model.name = Box::from(b"unconfigured".as_slice());
    rows.push(Record::Changed {
        setting: Setting::Charter(1),
        changed: Changed { revision: 1, value: Some(Value::Charter(Box::new(charter))) },
    })
    .expect("one stored change");
    for row in rows.into_boxed() {
        assert_eq!(restore(&mut core, &env, row), Restored::Live);
    }
    let expected = Failure::Settings {
        failure: crate::settings::Failure { setting: Setting::Charter(1), fault: Fault::Reference, detail: None },
        changed: true,
    };
    let mut out = Queue::uncapped();
    let Action::Refused(found) = core.start_done(&env, Phase::LoadCore, &mut out) else {
        panic!("settings fit refused")
    };
    assert_eq!(found, expected);
    assert_eq!(core.start_phase(), Some(Phase::LoadCore));
    assert!(out.is_empty());
}

#[test]
fn held_rows_past_each_limit_keep_their_refusal_detail() {
    for versions in [false, true] {
        let (mut core, mut env) = loaded();
        let (row, which) = if versions {
            env.limits.settings.versions = 0;
            (
                Record::Charter { charter: 7, revision: 2, value: CharterVersion { overlay: None, charter: None } },
                HeldLimit::Versions,
            )
        } else {
            env.limits.settings.revisions = 0;
            assert_eq!(restore(&mut core, &env, head(1)), Restored::Live);
            (head(2), HeldLimit::Revisions)
        };
        assert_eq!(restore(&mut core, &env, row), Restored::Rejected);
        assert_eq!(core.start_failure(), Some(Failure::HeldPastLimit { which, held: 1, limit: 0 }));
    }
}

#[test]
fn stored_policies_fill_the_project_table_and_name_the_first_overflow() {
    let (mut core, env) = loaded();
    let limit = env.limits.authority.projects;
    for project in 1..=limit {
        assert!(core.restore_policy(&env.limits, project, "Same".into(), policy()));
    }
    let project = limit + 1;
    assert!(!core.restore_policy(&env.limits, project, "Same".into(), policy()));
    assert_eq!(
        core.start_failure(),
        Some(Failure::LiveSet {
            set: crate::start::LiveSet::Projects,
            key: Key::People(people::Key::Policy(project)),
            held: u64::from(project),
            limit: u64::from(limit),
        })
    );
}

#[test]
fn authority_child_policy_bounds_refuse_at_the_row_entrance() {
    let (mut core, env) = loaded();
    let mut value = policy();
    let role = value.roles.first().expect("stored role").clone();
    value.roles = Box::new([role.clone(), role]);
    assert!(!core.restore_policy(&env.limits, 7, "Project".into(), value));
    assert_eq!(core.start_failure(), Some(Failure::ProjectPolicy { project: 7 }));
}

#[test]
fn setting_values_keep_their_byte_bound_at_the_row_entrance() {
    let (mut core, mut env) = loaded();
    let charter = core.charter(1).expect("charter").clone();
    let bytes = crate::limits::policy_owned_bytes(&charter.policy).expect("charter bytes");
    env.limits.run_bytes = u32::try_from(bytes).expect("small charter");
    let record = Record::Changed {
        setting: Setting::Charter(1),
        changed: Changed { revision: 1, value: Some(Value::Charter(Box::new(charter))) },
    };
    assert_eq!(restore(&mut core, &env, record.clone()), Restored::Live);
    let (mut core, _) = loaded();
    env.limits.run_bytes -= 1;
    assert_eq!(restore(&mut core, &env, record), Restored::Rejected);
    assert_eq!(
        core.start_failure(),
        Some(Failure::Settings {
            failure: crate::settings::Failure {
                setting: Setting::Charter(1),
                fault: Fault::Shape,
                detail: Some(people::SettingDetail {
                    member: people::SettingMember::StoredPolicy,
                    document: 0,
                    segment: 0,
                    rule: people::SettingRule::Maximum,
                    supplied: u32::try_from(bytes).expect("small charter"),
                    bound: env.limits.run_bytes,
                }),
            },
            changed: true,
        })
    );
}

#[test]
fn a_stored_head_does_not_rejudge_revision_order() {
    let Record::Revision { mut revision, history } = head(1) else { panic!("stored head") };
    revision.head.against = revision.head.number;
    {
        let (mut core, env) = loaded();
        assert_eq!(restore(&mut core, &env, Record::Revision { history, revision: revision.clone() }), {
            Restored::Live
        });
    }
}

#[test]
fn restored_refusals_keep_the_first_cause_across_families() {
    let (mut core, mut env) = loaded();
    env.limits.settings.versions = 0;
    let row = Record::Charter { charter: 7, revision: 2, value: CharterVersion { overlay: None, charter: None } };
    assert_eq!(restore(&mut core, &env, row), Restored::Rejected);
    env.limits.policy_bytes = 0;
    assert!(!core.restore_policy(&env.limits, 1, "Project".into(), policy()));
    assert_eq!(core.start_failure(), Some(Failure::HeldPastLimit { which: HeldLimit::Versions, held: 1, limit: 0 }));
    assert_eq!(core.start_failure(), Some(Failure::HeldPastLimit { which: HeldLimit::Versions, held: 1, limit: 0 }));
}

#[test]
fn an_empty_settings_range_keeps_the_supplied_defaults() {
    let (mut core, env) = loaded();
    let before = core.current.rules().clone();
    super::through_link(&mut core, &env, &mut Queue::uncapped());
    assert_eq!(core.current.rules(), &before);
    assert_eq!(core.start_failure(), None);
}

#[test]
fn an_overlay_is_checked_whole_over_the_new_default_before_start() {
    use crate::settings::{Default, Overlay};
    let (mut core, env) = loaded();
    let rows = settings_rows(&core);
    let mut default = core.current.charter(1).unwrap().clone();
    default.policy.instructions = Box::new([]);
    let room = u64::from(env.limits.run_bytes) - crate::policy_owned_bytes(&default.policy).unwrap();
    default.policy.instructions = skein_lib::bytes::zeroed(usize::try_from(room).unwrap());
    core.current
        .defaults
        .insert(Setting::Charter(1), Default { value: Value::Charter(Box::new(default)), revision: 1 })
        .unwrap();
    for row in rows.into_boxed() {
        assert_ne!(restore(&mut core, &env, row), Restored::Rejected);
    }
    assert!(core.restore_setting(
        Record::Changed {
            setting: Setting::Charter(1),
            changed: Changed {
                revision: 1,
                value: Some(Value::Overlay(Box::new(Overlay {
                    instructions: b"B".as_slice().into(),
                    ..Overlay::empty()
                })))
            },
        },
        &env.limits
    ));
    assert!(!core.settings_restored(&env));
    let Some(Failure::Settings { failure, changed }) = core.start_failure() else { panic!("named settings failure") };
    assert_eq!(failure.setting, Setting::Charter(1));
    assert_eq!(failure.fault, Fault::Shape);
    assert!(changed);
}
