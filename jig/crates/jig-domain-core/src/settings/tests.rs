use crate::{
    Core, CoreRecord, Limits, Restored,
    settings::*,
    tests::{RESTORE_POLICY_BYTES, RESTORE_PROJECT, config, limits, restoring_sized_policy},
};
use jig_domain_tasks as tasks;
use skein_lib::{Duration, Env, Time};

fn template() -> Template {
    Template {
        resources: Box::new([]),
        number: 1,
        name: Box::from(b"chat".as_slice()),
        executor: people::TemplateExecutor::Charter(1),
        contract: tasks::Contract::Report { words: 1 },
        shallower: 0,
        budget: people::TemplateBudget::Given,
        deadline: people::TemplateDeadline::None,
        tracked: None,
        flags: tasks::Flags::PLAIN,
        label: 0,
        grace: None,
        llm: None,
    }
}
fn charter(current: &mut Current) -> &mut Charter {
    match &mut current.defaults.get_mut(&Setting::Charter(1)).unwrap().value {
        Value::Charter(value) => value,
        Value::Template(_) | Value::Policy(_) | Value::Rules(_) => unreachable!(),
        Value::Overlay(_) | Value::Model(_) | Value::Application(_) => unreachable!("an application's setting: apps-core"),
    }
}
#[test]
fn lookups_apply_sets_removals_additions_and_resets_without_a_third_set() {
    let mut current = Current::new(config(1).defaults, &limits(), &[]);
    let original = current.charter(1).unwrap().clone();
    let mut changed = original.clone();
    changed.spend += 1;
    current
        .changes
        .insert(Setting::Charter(1), Changed { value: Some(Value::Charter(Box::new(changed.clone()))), revision: 2 })
        .unwrap();
    assert_eq!(current.charter(1), Some(&changed));
    current.changes.insert(Setting::Charter(1), Changed { value: None, revision: 3 }).unwrap();
    assert!(current.charter(1).is_none());
    let mut added = original.clone();
    added.number = 2;
    current
        .changes
        .insert(Setting::Charter(2), Changed { value: Some(Value::Charter(Box::new(added.clone()))), revision: 4 })
        .unwrap();
    assert_eq!(current.charter(2), Some(&added));
    current.changes.remove(&Setting::Charter(1));
    assert_eq!(current.charter(1), Some(&original));
    assert_eq!(current.offered().as_slice(), &[Setting::Charter(1), Setting::Charter(2), Setting::Rules]);
}
fn fault_case(fault: Fault) {
    let mut config = config(1);
    let limits = limits();
    let mut policy = config.seed_projects[0].policy.clone();
    config.defaults.templates = Box::new([template()]);
    let mut current = Current::new(config.defaults, &limits, &[]);
    let setting = match fault {
        Fault::Shape => {
            let count = limits.run_bytes.checked_add(1).expect("one past the run bound");
            let mut bytes = List::with_capacity(count);
            for _ in 0..count {
                bytes.push(b'x').expect("instruction byte count");
            }
            charter(&mut current).policy.instructions = bytes.into_boxed();
            Setting::Charter(1)
        }
        Fault::Count => {
            let original = current.charter(1).unwrap().clone();
            let extra = limits.tasks.charters.checked_add(1).expect("one past the charter bound");
            for number in 2..=extra {
                let mut value = original.clone();
                value.number = number;
                current
                    .defaults
                    .insert(Setting::Charter(number), Default { value: Value::Charter(Box::new(value)), revision: 1 })
                    .unwrap();
            }
            Setting::Charter(extra)
        }
        Fault::Reference => {
            let mut value = template();
            value.executor = people::TemplateExecutor::Charter(999);
            current
                .changes
                .insert(Setting::Template(1), Changed { value: Some(Value::Template(Box::new(value))), revision: 2 })
                .unwrap();
            Setting::Template(1)
        }
        Fault::Rules => {
            policy.policy.period_spend =
                current.rules().period_spend.checked_add(1).expect("one past the policy period bound");
            current
                .changes
                .insert(Setting::Policy(1), Changed { value: Some(Value::Policy(Box::new(policy))), revision: 2 })
                .unwrap();
            Setting::Policy(1)
        }
        Fault::Budget => {
            let value = charter(&mut current);
            value.policy.turns = value.policy.reserve_turns;
            Setting::Charter(1)
        }
        Fault::Grace => {
            let mut value = template();
            value.grace = Some(Duration::from_secs(86_400));
            current
                .changes
                .insert(Setting::Template(1), Changed { value: Some(Value::Template(Box::new(value))), revision: 2 })
                .unwrap();
            Setting::Template(1)
        }
        Fault::Owner => {
            policy.policy.roles[0].requests = authority::Requests(authority::Requests::ALL.0 & !1);
            current
                .changes
                .insert(Setting::Policy(1), Changed { value: Some(Value::Policy(Box::new(policy))), revision: 2 })
                .unwrap();
            Setting::Policy(1)
        }
        Fault::Judged => unreachable!("an application's setting: apps-core"),
    };
    let detail = if fault == Fault::Shape {
        let charter = current.charter(1).unwrap();
        Some(people::SettingDetail {
            member: people::SettingMember::StoredPolicy,
            document: 0,
            segment: 0,
            rule: people::SettingRule::Maximum,
            supplied: u32::try_from(
                crate::limits::policy_owned_bytes(&charter.policy)
                    .unwrap()
                    .checked_add(crate::limits::parts_owned_bytes(&charter.parts).unwrap())
                    .unwrap(),
            )
            .unwrap(),
            bound: limits.run_bytes,
        })
    } else {
        None
    };
    assert_eq!(
        check(&current, &config.settings, &config.connectors, &limits),
        Err(Failure { setting, fault, detail }),
        "{fault:?}"
    );
}

#[test]
fn the_whole_check_names_shape_and_its_setting() {
    fault_case(Fault::Shape);
}
#[test]
fn the_whole_check_names_count_and_its_setting() {
    fault_case(Fault::Count);
}
#[test]
fn the_whole_check_names_reference_and_its_setting() {
    fault_case(Fault::Reference);
}
#[test]
fn the_whole_check_names_rules_and_its_setting() {
    fault_case(Fault::Rules);
}
#[test]
fn the_whole_check_names_budget_and_its_setting() {
    fault_case(Fault::Budget);
}
#[test]
fn the_whole_check_names_grace_and_its_setting() {
    fault_case(Fault::Grace);
}
#[test]
fn the_whole_check_names_owner_and_its_setting() {
    fault_case(Fault::Owner);
}

#[test]
fn pruning_a_lowered_history_limit_uses_only_the_commits_remaining_room() {
    let (writes, oldest) = revision_writes(None, List::with_capacity(0), 1, 1, 20, 2, 4);
    assert_eq!(oldest, 4);
    assert_eq!(writes.len(), 4);
    let crate::Write::Save(crate::Record::Core(CoreRecord::Settings(Record::Extent { oldest: stored, .. }))) =
        &writes.as_slice()[3]
    else {
        panic!("the extent closes the pruning commit")
    };
    assert_eq!(*stored, 4);
    let (writes, oldest) = revision_writes(None, List::with_capacity(0), oldest, 1, 20, 2, 32);
    assert_eq!(oldest, 18);
    assert_eq!(writes.len(), 15);
    let (writes, oldest) = revision_writes(None, List::with_capacity(0), oldest, 1, 20, 2, 32);
    assert!(writes.is_empty());
    assert_eq!(oldest, 18);
}

#[test]
fn the_whole_check_refuses_unconfigured_model_bits() {
    let config = config(1);
    let mut current = Current::new(config.defaults, &limits(), &[]);
    let mut rules = current.rules().clone();
    rules.ceiling.models = authority::Models::Named(2);
    current
        .changes
        .insert(Setting::Rules, Changed { value: Some(Value::Rules(Box::new(rules))), revision: 2 })
        .unwrap();
    assert_eq!(
        check(&current, &config.settings, &config.connectors, &limits()),
        Err(Failure { setting: Setting::Rules, fault: Fault::Reference, detail: None })
    );
}
#[test]
fn the_whole_check_refuses_unconfigured_contract_connectors() {
    let config = config(1);
    let mut current = Current::new(config.defaults, &limits(), &[]);
    let mut value = template();
    value.contract = tasks::Contract::Change { connector: u16::MAX, kind: 1, words: 1 };
    current
        .changes
        .insert(Setting::Template(1), Changed { value: Some(Value::Template(Box::new(value))), revision: 2 })
        .unwrap();
    assert_eq!(
        check(&current, &config.settings, &config.connectors, &limits()),
        Err(Failure { setting: Setting::Template(1), fault: Fault::Reference, detail: None })
    );
}

#[test]
fn decision_edits_apply_each_kind_over_defaults_and_additions() {
    let limits = limits();
    let mut config = config(1);
    config.defaults.templates = Box::new([template()]);
    let policy = config.seed_projects[0].policy.clone();
    config.defaults.policies = Map::with_capacity(1);
    config.defaults.policies.insert(1, policy).unwrap();
    let mut current = Current::new(config.defaults, &limits, &[]);
    for setting in [Setting::Charter(1), Setting::Template(1), Setting::Policy(1), Setting::Rules] {
        let original = current.value(setting).unwrap().clone();
        let items = [Item { setting, edit: Edit::Set(original.clone()) }];
        assert!(!changes_current(&current, &items));
        apply(&mut current, &items, 2);
        assert_eq!(current.value(setting), Some(&original));
        assert_eq!(current.changes.get(&setting).unwrap().revision, 2);
        let reset = [Item { setting, edit: Edit::Reset }];
        let writes = apply(&mut current, &reset, 3);
        let mut saved_reset = false;
        for write in writes.as_slice() {
            if let crate::Write::Save(crate::Record::Core(CoreRecord::Settings(Record::Default {
                setting: saved,
                default,
            }))) = write
            {
                saved_reset |= *saved == setting && default.revision == 3;
            }
        }
        assert!(saved_reset);
        assert_eq!(current.lookup(setting).1, Some((false, 3)));
        assert!(!current.changed(setting));
        if setting == Setting::Rules {
            assert_eq!(
                change_shape(&[Item { setting, edit: Edit::Removed }], &limits),
                Err(Failure { setting, fault: Fault::Shape, detail: None })
            );
        } else {
            let removed = [Item { setting, edit: Edit::Removed }];
            assert!(changes_current(&current, &removed));
            apply(&mut current, &removed, 4);
            assert!(current.value(setting).is_none());
            assert!(!changes_current(&current, &removed));
            apply(&mut current, &reset, 5);
            assert_eq!(current.value(setting), Some(&original));
        }
        let added = match original {
            Value::Charter(mut value) => {
                value.number = 2;
                Some((Setting::Charter(2), Value::Charter(value)))
            }
            Value::Template(mut value) => {
                value.number = 2;
                Some((Setting::Template(2), Value::Template(value)))
            }
            Value::Policy(value) => Some((Setting::Policy(2), Value::Policy(value))),
            Value::Rules(_) => None,
            Value::Overlay(_) | Value::Model(_) | Value::Application(_) => unreachable!("an application's setting: apps-core"),
        };
        if let Some((setting, value)) = added {
            for erase in [Edit::Removed, Edit::Reset] {
                apply(&mut current, &[Item { setting, edit: Edit::Set(value.clone()) }], 6);
                assert_eq!(current.value(setting), Some(&value));
                apply(&mut current, &[Item { setting, edit: erase }], 7);
                assert!(!current.changed(setting));
                assert!(current.value(setting).is_none());
            }
        }
    }
    assert!(!changes_current(&current, &[]));
}

#[test]
fn zero_numbered_edits_and_repeated_settings_are_malformed() {
    let limits = limits();
    for setting in [Setting::Charter(0), Setting::Template(0), Setting::Policy(0)] {
        for edit in [Edit::Removed, Edit::Reset] {
            assert_eq!(
                change_shape(&[Item { setting, edit }], &limits),
                Err(Failure { setting, fault: Fault::Shape, detail: None })
            );
        }
    }
    let setting = Setting::Template(1);
    assert_eq!(
        change_shape(&[Item { setting, edit: Edit::Removed }, Item { setting, edit: Edit::Reset }], &limits),
        Err(Failure { setting, fault: Fault::Shape, detail: None })
    );
}

#[test]
fn current_lookup_keeps_each_kinds_default_change_and_removal_origin() {
    let mut config = config(1);
    config.defaults.templates = Box::new([template()]);
    config.defaults.policies = Map::with_capacity(1);
    config.defaults.policies.insert(1, config.seed_projects[0].policy.clone()).unwrap();
    let mut current = Current::new(config.defaults, &limits(), &[]);
    for setting in [Setting::Charter(1), Setting::Template(1), Setting::Policy(1), Setting::Rules] {
        let default = current.value(setting).unwrap().clone();
        assert_eq!(current.lookup(setting), (Some(&default), Some((false, 1))));
        current.changes.insert(setting, Changed { value: Some(default.clone()), revision: 2 }).unwrap();
        assert_eq!(current.lookup(setting), (Some(&default), Some((true, 2))));
        current.changes.insert(setting, Changed { value: None, revision: 3 }).unwrap();
        assert_eq!(current.lookup(setting), (None, Some((true, 3))));
        let _writes = apply(&mut current, &[Item { setting, edit: Edit::Reset }], 4);
        assert_eq!(current.lookup(setting), (Some(&default), Some((false, 4))));
    }
    assert_eq!(current.lookup(Setting::Template(999)), (None, None));
}

fn stored_policy_change(bytes: u64) -> (Core, Limits, Record) {
    let (mut core, limits, configured) = restoring_sized_policy(bytes);
    let mut defaults = List::with_capacity(core.current.defaults.len());
    for (&setting, default) in &core.current.defaults {
        defaults.push(Record::Default { setting, default: default.clone() }).expect("stored default count");
    }
    for record in defaults.into_boxed() {
        let _restored = core.restore_core(CoreRecord::Settings(record), &limits);
    }
    for record in [
        Record::Revision {
            history: false,
            revision: Revision { head: core.current.head.clone(), items: Box::new([]), applications: Box::default() },
        },
        Record::Extent { oldest: 0, keys: 1 },
    ] {
        let _restored = core.restore_core(CoreRecord::Settings(record), &limits);
    }
    let changed = Record::Changed {
        setting: Setting::Policy(RESTORE_PROJECT),
        changed: Changed { revision: 1, value: Some(Value::Policy(Box::new(configured))) },
    };
    (core, limits, changed)
}

#[test]
fn oversized_settings_policy_restore_keeps_its_named_failure() {
    let oversized = RESTORE_POLICY_BYTES.checked_add(1).expect("one past the policy byte budget");
    let (mut core, limits, record) = stored_policy_change(oversized);
    assert_eq!(core.restore_core(CoreRecord::Settings(record), &limits), Restored::Rejected);
    let failure = crate::start::Failure::Settings {
        failure: Failure { setting: Setting::Policy(RESTORE_PROJECT), fault: Fault::Shape, detail: None },
        changed: true,
    };
    assert_eq!(core.start_failure(), Some(failure.clone()));
    assert_eq!(core.start_failure(), Some(failure.clone()));
    assert!(!core.current.changed(Setting::Policy(RESTORE_PROJECT)));
    assert!(!core.start_opened());
}

#[test]
fn settings_policy_restore_admits_the_policy_byte_budget() {
    let (mut core, limits, record) = stored_policy_change(RESTORE_POLICY_BYTES);
    assert_eq!(core.restore_core(CoreRecord::Settings(record), &limits), Restored::Live);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    assert!(core.settings_restored(&env));
    assert_eq!(core.start_failure(), None);
    assert!(core.current.policy(RESTORE_PROJECT).is_some());
}

fn version_head(core: &Core, revision: u32) -> Head {
    Head { number: revision, against: revision.saturating_sub(1), ..core.current.head.clone() }
}
fn held_fixture() -> (Core, Limits) {
    let (mut core, limits) = crate::tests::core();
    let mut charter = core.charter(1).unwrap().clone();
    charter.number = 7;
    charter.spend = 1;
    core.current.defaults.remove(&Setting::Charter(1)).unwrap();
    core.current
        .defaults
        .insert(Setting::Charter(7), Default { value: Value::Charter(Box::new(charter)), revision: 1 })
        .unwrap();
    (core, limits)
}
fn version_edit(core: &mut Core, limits: &Limits, number: u32, charter: u32, edit: Edit, keep_previous: bool) {
    let previous = core.current.newest();
    let items = [Item { setting: Setting::Charter(charter), edit }];
    let plan = core.held.plan(&core.current, &items, number, if keep_previous { Some(previous) } else { None }, limits);
    assert!(plan.versions() <= limits.settings.versions);
    let mut out = skein_lib::Queue::with_capacity(64);
    core.held.retain(&core.current, &plan, &mut out);
    if keep_previous {
        core.held.heads.insert(previous, core.current.head.clone()).unwrap();
    }
    let _writes = apply(&mut core.current, &items, number);
    core.current.head = version_head(core, number);
}
fn version_value(core: &Core, charter: u32, spend: u64) -> Edit {
    let source = match core.charter(charter) {
        Some(charter) => charter,
        None => core.charter(7).unwrap(),
    };
    let mut value = source.clone();
    value.number = charter;
    value.spend = spend;
    Edit::Set(Value::Charter(Box::new(value)))
}
fn held_versions() -> (Core, Limits) {
    let (mut core, limits) = held_fixture();
    let edit = version_value(&core, 7, 3);
    version_edit(&mut core, &limits, 3, 7, edit, true);
    core.current.head = version_head(&core, 4);
    let edit = version_value(&core, 7, 5);
    version_edit(&mut core, &limits, 5, 7, edit, true);
    core.current.head = version_head(&core, 6);
    (core, limits)
}
#[test]
fn a_revision_reads_the_latest_version_at_or_before_it() {
    let (core, _) = held_versions();
    for (revision, expected) in [(1, 1), (4, 3), (6, 5)] {
        assert_eq!(core.charter_at(7, revision).unwrap().spend, expected);
    }
    assert_eq!(
        core.charter_at(7, 1),
        core.held.versions.get(&VersionKey { charter: 7, revision: 1 }).unwrap().charter.as_deref()
    );
    assert!(core.charter_at(7, 2).is_none());
}
#[test]
fn a_removal_is_a_version_from_which_the_charter_is_not_offered() {
    let (mut core, limits) = held_fixture();
    let edit = version_value(&core, 8, 2);
    version_edit(&mut core, &limits, 2, 8, edit, false);
    core.current.head = version_head(&core, 3);
    version_edit(&mut core, &limits, 4, 8, Edit::Removed, true);
    core.current.head = version_head(&core, 5);
    assert!(!core.current.changed(Setting::Charter(8)));
    assert_eq!(core.charter_at(8, 3).unwrap().spend, 2);
    assert!(core.charter_at(8, 5).is_none());
    assert!(core.charter(8).is_none());
    assert!(core.held.versions.get(&VersionKey { charter: 8, revision: 4 }).unwrap().charter.is_none());
}
#[test]
fn versions_no_held_revision_reads_are_dropped() {
    let (mut core, limits) = held_versions();
    core.held.heads.remove(&1).unwrap();
    let plan = core.held.plan(&core.current, &[], core.current.newest(), None, &limits);
    let mut out = skein_lib::Queue::with_capacity(64);
    core.held.retain(&core.current, &plan, &mut out);
    assert!(!core.held.versions.contains_key(&VersionKey { charter: 7, revision: 1 }));
    assert!(core.held.versions.contains_key(&VersionKey { charter: 7, revision: 3 }));
    assert_eq!(core.charter_at(7, 4).unwrap().spend, 3);
}

#[test]
fn releasing_multiple_revisions_writes_one_extent_after_all_copies_and_erases() {
    let (mut core, mut limits) = held_versions();
    limits.settings.revisions = 2;
    limits.settings.versions = 2;
    core.current.oldest = 0;
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = skein_lib::Queue::with_capacity(release_writes(&limits).unwrap());
    core.release_revision(&env, 4, &mut out);
    core.release_revision(&env, 1, &mut out);

    core.finish_extent(0, &mut out);
    assert_eq!(out.len(), 7, "two copies, two Held erases, two Charter erases and one extent");
    let mut extents = 0_u32;
    for request in &out {
        if let crate::Request::Write(crate::Write::Save(crate::Record::Core(CoreRecord::Settings(Record::Extent {
            oldest,
            ..
        })))) = request
        {
            extents += 1;
            assert_eq!(*oldest, 1);
        }
    }
    assert_eq!(extents, 1);
    for _ in 0_u32..6 {
        let Some(crate::Request::Write(write)) = out.pop() else { panic!("revision write") };
        if let crate::Write::Save(crate::Record::Core(CoreRecord::Settings(Record::Extent { .. }))) = write {
            panic!("extent follows every revision write");
        }
    }
    let Some(crate::Request::Write(crate::Write::Save(crate::Record::Core(CoreRecord::Settings(Record::Extent {
        oldest: 1,
        ..
    }))))) = out.pop()
    else {
        panic!("extent follows every revision write")
    };
    assert!(out.is_empty());
}
#[test]
fn a_reset_is_a_version_made_at_the_reset() {
    let (mut core, limits) = held_fixture();
    let original = core.charter(7).unwrap().clone();
    let edit = version_value(&core, 7, 3);
    version_edit(&mut core, &limits, 3, 7, edit, false);
    core.current.head = version_head(&core, 4);
    version_edit(&mut core, &limits, 5, 7, Edit::Reset, true);
    assert!(!core.current.changed(Setting::Charter(7)));
    assert_eq!(core.current.lookup(Setting::Charter(7)).1, Some((false, 5)));
    core.current.head = version_head(&core, 6);
    let edit = version_value(&core, 7, 7);
    version_edit(&mut core, &limits, 7, 7, edit, true);
    assert_eq!(core.charter_at(7, 4).unwrap().spend, 3);
    assert_eq!(core.charter_at(7, 6), Some(&original));
    assert_eq!(core.charter_at(7, 7).unwrap().spend, 7);
}
#[test]
fn an_erased_removal_bars_older_versions_from_later_revisions() {
    let (mut core, limits) = held_fixture();
    let edit = version_value(&core, 8, 2);
    version_edit(&mut core, &limits, 2, 8, edit, false);
    core.current.head = version_head(&core, 3);
    version_edit(&mut core, &limits, 4, 8, Edit::Removed, true);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    core.tell_replaced_settings(&env, false, true);
    assert!(core.tasks.charter_models(8, 3).is_some());
    assert!(core.tasks.charter_models(8, 4).is_none());
    core.current.head = version_head(&core, 5);
    let edit = version_value(&core, 8, 6);
    version_edit(&mut core, &limits, 6, 8, edit, true);
    assert_eq!(core.charter_at(8, 3).unwrap().spend, 2);
    assert!(core.charter_at(8, 5).is_none());
    assert_eq!(core.charter_at(8, 6).unwrap().spend, 6);
    core.held.heads.remove(&3).unwrap();
    let plan = core.held.plan(&core.current, &[], core.current.newest(), None, &limits);
    let mut out = skein_lib::Queue::with_capacity(64);
    core.held.retain(&core.current, &plan, &mut out);
    assert!(!core.held.versions.contains_key(&VersionKey { charter: 8, revision: 2 }));
    assert!(core.charter_at(8, 5).is_none());
}

#[test]
fn a_barrier_leaves_with_the_last_older_value_even_while_absence_is_held() {
    let (mut core, limits) = held_fixture();
    let edit = version_value(&core, 8, 2);
    version_edit(&mut core, &limits, 2, 8, edit, false);
    core.current.head = version_head(&core, 3);
    version_edit(&mut core, &limits, 4, 8, Edit::Removed, true);
    core.held.heads.insert(4, version_head(&core, 4)).unwrap();
    core.current.head = version_head(&core, 5);
    core.held.heads.remove(&3).unwrap();
    let plan = core.held.plan(&core.current, &[], 5, None, &limits);
    let mut out = skein_lib::Queue::with_capacity(64);
    core.held.retain(&core.current, &plan, &mut out);
    assert!(core.held.versions.is_empty());
    assert!(core.charter_at(8, 4).is_none());
    assert!(core.charter_at(8, 5).is_none());
}

#[test]
fn removal_markers_fit_beside_offered_charters_and_held_versions() {
    let (mut core, mut limits) = held_fixture();
    limits.tasks.charters = 2;
    limits.settings.versions = 1;
    let mut unchanged = core.charter(7).unwrap().clone();
    unchanged.number = 9;
    core.current
        .defaults
        .insert(Setting::Charter(9), Default { value: Value::Charter(Box::new(unchanged)), revision: 1 })
        .unwrap();
    let items = [
        Item { setting: Setting::Charter(7), edit: Edit::Removed },
        Item { setting: Setting::Charter(8), edit: version_value(&core, 8, 2) },
    ];
    let plan = core.held.plan(&core.current, &items, 2, Some(1), &limits);
    assert_eq!(plan.versions(), 1, "the newest removal consumes no extra version");
    let mut out = skein_lib::Queue::with_capacity(64);
    core.held.retain(&core.current, &plan, &mut out);
    core.held.heads.insert(1, core.current.head.clone()).unwrap();
    let _writes = apply(&mut core.current, &items, 2);
    core.current.head = version_head(&core, 2);
    let models = charters::held_models(&core, &limits);
    assert_eq!(models.len(), 4);
    assert_eq!(core.held.versions.len(), 1);
    assert!(core.charter_at(7, 1).is_some());
    assert!(core.charter_at(7, 2).is_none());
    let mut tasks_out = skein_lib::Queue::uncapped();
    tasks::step(
        &mut core.tasks,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.tasks },
        tasks::Event::Charters { newest: 2, charters: models, efforts: charters::efforts(&core.settings) },
        &mut tasks_out,
    );
    assert!(tasks_out.is_empty());
    assert!(core.tasks.charter_models(7, 1).is_some());
    assert!(core.tasks.charter_models(7, 2).is_none());
    assert!(core.tasks.charter_models(8, 2).is_some());
    assert!(core.tasks.charter_models(9, 2).is_some());
}

fn restore_settings_fixture() -> (Core, Limits) {
    let limits = limits();
    let mut core = Core::new(config(41), &limits).unwrap();
    let _start = core.start_begin(false, Wall::EPOCH);
    let mut defaults = List::with_capacity(core.current.defaults.len());
    for (&setting, default) in &core.current.defaults {
        let mut default = default.clone();
        if let Setting::Charter(_) = setting {
            default.revision = 2;
        }
        defaults.push(Record::Default { setting, default }).unwrap();
    }
    for row in defaults.into_boxed() {
        assert!(core.restore_setting(row, &limits));
    }
    assert!(core.restore_setting(Record::Extent { oldest: 0, keys: 1 }, &limits));
    (core, limits)
}

fn restore_head(core: &mut Core, limits: &Limits, number: u32) -> bool {
    let head = version_head(core, number);
    core.restore_setting(
        Record::Revision {
            history: false,
            revision: Revision { head, items: Box::new([]), applications: Box::default() },
        },
        limits,
    )
}

#[test]
fn two_held_rows_and_a_charter_row_restore_before_tasks() {
    for order in [[1, 2], [2, 1]] {
        let (mut core, limits) = restore_settings_fixture();
        let charter = core.charter(1).unwrap().clone();
        for number in order {
            assert!(restore_head(&mut core, &limits, number));
        }
        assert!(core.restore_setting(
            Record::Charter {
                charter: 1,
                revision: 1,
                value: CharterVersion { overlay: None, charter: Some(Box::new(charter.clone())) },
            },
            &limits
        ));
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        assert!(core.settings_restored(&env));
        assert_eq!(core.current.newest(), 2);
        assert_eq!(core.held.heads.len(), 1);
        assert_eq!(core.held.heads.get(&1).unwrap().number, 1);
        assert_eq!(
            core.held.versions.get(&VersionKey { charter: 1, revision: 1 }).unwrap().charter.as_deref(),
            Some(&charter)
        );
        assert!(core.tasks.charter_models(1, 1).is_some());
        assert!(!restore_head(&mut core, &limits, 3), "the settings range has ended");
    }
}

#[test]
fn a_charter_row_past_the_limit_refuses_without_retaining_it() {
    let (mut core, mut limits) = restore_settings_fixture();
    limits.settings.versions = 0;
    assert!(restore_head(&mut core, &limits, 2));
    assert!(!core.restore_setting(
        Record::Charter {
            charter: 1,
            revision: 1,
            value: CharterVersion { overlay: None, charter: Some(Box::new(core.charter(1).unwrap().clone())) },
        },
        &limits
    ));
    assert!(core.held.versions.is_empty());
    let refusal = crate::start::Failure::HeldPastLimit { which: crate::HeldLimit::Versions, held: 1, limit: 0 };
    assert_eq!(core.start_failure(), Some(refusal));
}

#[test]
fn a_held_revision_left_without_holders_is_released_by_the_start() {
    let (mut core, limits) = restore_settings_fixture();
    assert!(restore_head(&mut core, &limits, 1));
    assert!(restore_head(&mut core, &limits, 2));
    let mut old = core.charter(1).unwrap().clone();
    old.policy.instructions = b"An orphan version".as_slice().into();
    assert!(core.restore_setting(
        Record::Charter { charter: 1, revision: 1, value: CharterVersion { overlay: None, charter: Some(Box::new(old)) } },
        &limits
    ));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let mut out = core.commit_restored_settings(&env, limits.routes);
    let mut copied = false;
    let mut erased_head = false;
    let mut erased_version = false;
    let mut saved_extent = false;
    while let Some(request) = out.pop() {
        if let crate::Request::Write(write) = request {
            if let crate::Write::CopySettingsRevision(number) = &write {
                copied |= *number == 1;
            }
            if let crate::Write::Erase(key) = &write {
                erased_head |= *key == crate::Key::Core(crate::CoreKey::Settings(Key::Held(1)));
                erased_version |=
                    *key == crate::Key::Core(crate::CoreKey::Settings(Key::Charter { charter: 1, revision: 1 }));
            }
            if let crate::Write::Save(crate::Record::Core(CoreRecord::Settings(Record::Extent { oldest, .. }))) = write
            {
                saved_extent |= oldest == 1;
            }
        }
    }
    assert!(core.held.heads.is_empty());
    assert!(core.held.versions.is_empty());
    assert!(copied);
    assert!(erased_head);
    assert!(erased_version);
    assert!(saved_extent);
}

#[test]
fn a_settings_commit_before_the_first_start_seed_keeps_the_supplied_defaults() {
    let limits = limits();
    let configured = config(41);
    let rules = configured.defaults.rules.clone();
    let mut core = Core::new(configured, &limits).unwrap();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    let out = core.commit_restored_settings(&env, limits.routes);
    assert!(out.is_empty());
    assert_eq!(core.current.rules(), &rules);
    let mut writes = skein_lib::Queue::uncapped();
    assert!(restart::first_start(&mut core, &env, &mut writes));
    assert!(!writes.is_empty());
    assert_eq!(core.current.newest(), 1);
}

#[test]
fn a_charter_with_a_zero_paced_span_is_refused_as_shape() {
    let config = config(1);
    let limits = limits();
    let mut current = Current::new(config.defaults, &limits, &[]);
    let mut invalid = current.charter(1).unwrap().clone();
    invalid.wake.news = tasks::WakeRule::Paced { span: Duration::ZERO };
    let setting = Setting::Charter(1);
    let expected = Err(Failure { setting, fault: Fault::Shape, detail: None });
    let value = Value::Charter(Box::new(invalid));
    assert_eq!(check::shape(setting, &value, &limits), expected);
    let items = [Item { setting, edit: Edit::Set(value.clone()) }];
    assert_eq!(check_change(&current, &items, &config.settings, &config.connectors, &limits), expected);
    current.defaults.get_mut(&setting).unwrap().value = value;
    assert_eq!(check(&current, &config.settings, &config.connectors, &limits), expected);
}

#[test]
fn default_wake_reads_current_settings_and_falls_back_without_a_charter() {
    let limits = limits();
    let mut core = Core::new(config(1), &limits).unwrap();
    let agent = tasks::Executor::Agent { charter: 1 };
    let mut changed = core.charter(1).unwrap().clone();
    changed.wake.results = tasks::ResultsWake::Each;
    let expected = changed.wake;
    core.current
        .changes
        .insert(Setting::Charter(1), Changed { value: Some(Value::Charter(Box::new(changed))), revision: 2 })
        .unwrap();
    charters::rebuild(&mut core.current, &limits);
    assert_eq!(crate::admission::amendments::default_wake(&core, &agent), expected);
    for executor in [
        tasks::Executor::Agent { charter: 999 },
        tasks::Executor::Procedure { connector: 1, code: 1 },
        tasks::Executor::Job { kind: 1 },
        tasks::Executor::Person(tasks::PersonAddress::Person(1)),
    ] {
        assert_eq!(crate::admission::amendments::default_wake(&core, &executor), tasks::WakePolicy::DEFAULT);
    }
    core.current.changes.insert(Setting::Charter(1), Changed { value: None, revision: 3 }).unwrap();
    charters::rebuild(&mut core.current, &limits);
    assert_eq!(crate::admission::amendments::default_wake(&core, &agent), tasks::WakePolicy::DEFAULT);
}

fn delegation_rules(count: u32) -> Value {
    let mut rules = config(1).defaults.rules;
    let mut kinds = List::with_capacity(count);
    for number in 1..=count {
        kinds.push(authority::Executor::Procedure(number)).expect("delegation count");
    }
    rules.ceiling.delegation.kinds = kinds.into_boxed();
    Value::Rules(Box::new(rules))
}

#[test]
fn delegation_count_accepts_the_limit_and_names_overflow() {
    let config = config(1);
    let limits = limits();
    let mut current = Current::new(config.defaults, &limits, &[]);
    for count in [limits.authority.executors, limits.authority.executors + 1] {
        let value = delegation_rules(count);
        let expected = if count <= limits.authority.executors {
            Ok(())
        } else {
            Err(Failure { setting: Setting::Rules, fault: Fault::Count, detail: None })
        };
        let items = [Item { setting: Setting::Rules, edit: Edit::Set(value.clone()) }];
        assert_eq!(check_change(&current, &items, &config.settings, &config.connectors, &limits), expected);
        assert_eq!(check::restore_fit(Setting::Rules, &value, &limits), expected);
        current.changes.insert(Setting::Rules, Changed { value: Some(value), revision: 2 }).unwrap();
        assert_eq!(check(&current, &config.settings, &config.connectors, &limits), expected);
    }
}

#[test]
fn malformed_rules_within_the_delegation_limit_remain_shape() {
    let limits = limits();
    let Value::Rules(mut rules) = delegation_rules(limits.authority.executors) else { unreachable!() };
    rules.minimum_run_spend = rules.maximum_run_spend;
    assert_eq!(
        check::shape(Setting::Rules, &Value::Rules(rules), &limits),
        Err(Failure { setting: Setting::Rules, fault: Fault::Shape, detail: None })
    );
}

#[test]
fn delegation_count_restore_names_overflow() {
    let limits = limits();
    let value = delegation_rules(limits.authority.executors + 1);
    assert_eq!(
        check::restore_fit(Setting::Rules, &value, &limits),
        Err(Failure { setting: Setting::Rules, fault: Fault::Count, detail: None })
    );
}

fn catalogue(name: &[u8], efforts: &[&[u8]]) -> ModelName {
    let mut result = List::with_capacity(u32::try_from(efforts.len()).unwrap());
    for effort in efforts {
        result.push(Box::from(*effort)).unwrap();
    }
    ModelName { name: name.into(), efforts: result.into_boxed() }
}

#[test]
fn remap_template_choices_follow_model_and_effort_names_and_mark_departures() {
    let kept = [catalogue(b"first", &[b"low", b"high"]), catalogue(b"left", &[])];
    for (names, expected) in [
        (
            [catalogue(b"new", &[]), catalogue(b"first", &[b"high", b"low"])],
            tasks::ModelEffort { model: 1, effort: Some(0) },
        ),
        (
            [catalogue(b"new", &[]), catalogue(b"other", &[])],
            tasks::ModelEffort { model: tasks::ModelEffort::GONE, effort: Some(1) },
        ),
        (
            [catalogue(b"first", &[b"low"]), catalogue(b"new", &[])],
            tasks::ModelEffort { model: tasks::ModelEffort::GONE, effort: Some(1) },
        ),
    ] {
        let mut template = template();
        template.llm = Some(tasks::LlmChoice {
            later: None,
            account: None,
            main: tasks::ModelEffort { model: 0, effort: Some(1) },
            subagents: Box::new([tasks::ModelEffort { model: 1, effort: None }]),
        });
        let mut value = Value::Template(Box::new(template));
        remap(&mut value, &moves(&kept, &names), &kept, &names);
        let Value::Template(template) = value else { panic!("template") };
        let choice = template.llm.unwrap();
        assert_eq!(choice.main, expected);
        assert_eq!(choice.subagents[0], tasks::ModelEffort { model: tasks::ModelEffort::GONE, effort: None });
    }
}

#[test]
fn remap_named_masks_follow_names_and_drop_departures_while_all_stays() {
    let kept = [catalogue(b"first", &[]), catalogue(b"left", &[])];
    let names = [catalogue(b"new", &[]), catalogue(b"first", &[])];
    for (before, after) in
        [(authority::Models::Named(3), authority::Models::Named(2)), (authority::Models::All, authority::Models::All)]
    {
        let mut rules = config(1).defaults.rules;
        rules.ceiling.models = before;
        let mut value = Value::Rules(Box::new(rules));
        remap(&mut value, &moves(&kept, &names), &kept, &names);
        let Value::Rules(rules) = value else { panic!("rules") };
        assert_eq!(rules.ceiling.models, after);
    }
}

fn catalogue_fixture() -> (Current, crate::Settings, Limits, CatalogueModel) {
    let config = config(1);
    let mut settings = config.settings;
    settings.endpoints =
        Box::new([crate::EndpointFacts { providers: Box::new([1]), number: 1, dialect: 3, program: None }]);
    let value = CatalogueModel {
        first_read: None,
        number: 9,
        model: settings.models[0].clone(),
        shown: Some(b"Shown".as_slice().into()),
        description: Some(b"Description".as_slice().into()),
        picker: true,
        withdrawn: false,
        source: ModelSource::Person,
        estimated: true,
    };
    (Current::new(config.defaults, &limits(), &[]), settings, limits(), value)
}
fn catalogue_item(value: CatalogueModel) -> Item {
    Item { setting: Setting::Model(value.number), edit: Edit::Set(Value::Model(Box::new(value))) }
}
#[test]
fn a_model_setting_is_checked_whole() {
    let (current, settings, limits, value) = catalogue_fixture();
    assert_eq!(check_change(&current, &[catalogue_item(value.clone())], &settings, &[1, 2], &limits), Ok(()));
    for (member, fault) in [
        ("zero", Fault::Shape),
        ("past", Fault::Shape),
        ("name", Fault::Shape),
        ("provider", Fault::Reference),
        ("endpoint", Fault::Reference),
        ("dialect", Fault::Shape),
        ("program", Fault::Shape),
        ("window", Fault::Shape),
        ("output", Fault::Shape),
    ] {
        let mut value = value.clone();
        match member {
            "zero" => value.number = 0,
            "past" => value.number = 65,
            "name" => value.model.name = b"bad/name".as_slice().into(),
            "provider" => value.model.provider = 99,
            "endpoint" => value.model.endpoint = 99,
            "dialect" => value.model.dialect = 99,
            "program" => value.model.program = Some(b"other".as_slice().into()),
            "window" => value.model.window = 0,
            "output" => value.model.output = 0,
            _ => unreachable!(),
        }
        let number = value.number;
        assert_eq!(
            check_change(&current, &[catalogue_item(value)], &settings, &[1, 2], &limits),
            Err(Failure { setting: Setting::Model(number), fault, detail: None }),
            "{member}"
        );
    }
}
#[test]
fn a_number_keeps_its_name() {
    let (mut current, settings, limits, value) = catalogue_fixture();
    for default in [true, false] {
        if default {
            current
                .defaults
                .insert(Setting::Model(9), Default { value: Value::Model(Box::new(value.clone())), revision: 1 })
                .unwrap();
        } else {
            current.defaults.remove(&Setting::Model(9));
            apply(&mut current, &[catalogue_item(value.clone())], 2);
        }
        let mut renamed = value.clone();
        renamed.model.name = b"different".as_slice().into();
        assert_eq!(
            check_change(&current, &[catalogue_item(renamed)], &settings, &[1, 2], &limits),
            Err(Failure { setting: Setting::Model(9), fault: Fault::Shape, detail: None })
        );
    }
}
#[test]
fn removing_a_model_withdraws_it_and_keeps_its_number() {
    let (mut current, settings, mut limits, mut value) = catalogue_fixture();
    limits.run_bytes = 1024;
    // Withdrawal is checked with another named model left in the charter.
    let mut remaining = value.clone();
    remaining.number = 1;
    remaining.model.name = b"remaining".as_slice().into();
    charter(&mut current).policy.alternatives = Box::new([remaining.model.clone()]);
    apply(&mut current, &[catalogue_item(value.clone()), catalogue_item(remaining)], 2);
    let items = [Item { setting: Setting::Model(9), edit: Edit::Removed }];
    assert_eq!(check_change(&current, &items, &settings, &[1, 2], &limits), Ok(()));
    assert!(changes_current(&current, &items));
    let writes = apply(&mut current, &items, 3);
    value.withdrawn = true;
    assert_eq!(current.value(Setting::Model(9)), Some(&Value::Model(Box::new(value.clone()))));
    assert!(current.offered().as_slice().contains(&Setting::Model(9)));
    assert_eq!(current.changes.get(&Setting::Model(9)).unwrap().revision, 3);
    assert!(!changes_current(&current, &items));
    assert_eq!(writes.len(), 1);
}
#[test]
fn resetting_a_model_with_no_default_is_out_of_shape() {
    let (mut current, settings, limits, value) = catalogue_fixture();
    apply(&mut current, &[catalogue_item(value.clone())], 2);
    let items = [Item { setting: Setting::Model(9), edit: Edit::Reset }];
    assert_eq!(
        check_change(&current, &items, &settings, &[1, 2], &limits),
        Err(Failure { setting: Setting::Model(9), fault: Fault::Shape, detail: None })
    );
    current.defaults.insert(Setting::Model(9), Default { value: Value::Model(Box::new(value)), revision: 1 }).unwrap();
    assert_eq!(check_change(&current, &items, &settings, &[1, 2], &limits), Ok(()));
}
#[test]
fn valid_name_takes_the_shapes_a_provider_names_a_model_by() {
    const NAME_BYTES: u32 = 128;
    for name in [b"claude-opus-5-5".as_slice(), b"openai:gpt_6.1", b"A0", b"-._:"] {
        assert!(catalogue::valid_name(name));
    }
    let mut name = List::with_capacity(NAME_BYTES.checked_add(1).unwrap());
    for _ in 0..NAME_BYTES {
        name.push(b'a').unwrap();
    }
    assert!(catalogue::valid_name(name.as_slice()));
    for name in [b"".as_slice(), b"space name", b"slash/name", b"new\nline", "modèle".as_bytes(), &[0]] {
        assert!(!catalogue::valid_name(name));
    }
    name.push(b'a').unwrap();
    assert!(!catalogue::valid_name(name.as_slice()));
    assert!(crate::Model::hole().is_hole());
    assert!(!config(1).settings.models[0].is_hole());
}

#[test]
fn overlay_replaces_fields_and_appends_text_and_parts() {
    let current = Current::new(config(1).defaults, &limits(), &[]);
    let mut default = current.charter(1).unwrap().clone();
    default.policy.instructions = Box::from(b"A".as_slice());
    let part = |text: &[u8]| charters::CharterPart { on: tasks::Reasons::FIRST, instructions: Box::from(text) };
    default.parts = Box::new([part(b"p1"), part(b"p2")]);
    let mut overlay = Overlay { shell: Some(!default.policy.shell), instructions: Box::from(b"B".as_slice()),
        parts: Box::new([part(b"q1")]), ..Overlay::empty() };
    let mut expected = default.clone();
    expected.policy.shell = !default.policy.shell;
    expected.policy.instructions = Box::from(b"A\n\nB".as_slice());
    expected.parts = Box::new([part(b"p1"), part(b"p2"), part(b"q1")]);
    assert_eq!(overlaid(&default, &overlay), expected);
    overlay.instructions = Box::new([]);
    assert_eq!(overlaid(&default, &overlay).policy.instructions.as_ref(), b"A");
    default.policy.instructions = Box::new([]);
    overlay.instructions = Box::from(b"B".as_slice());
    assert_eq!(overlaid(&default, &overlay).policy.instructions.as_ref(), b"B");
}

#[test]
fn overlay_requires_a_default_and_keeps_its_stored_identity() {
    let configured = config(1);
    let mut limits = limits();
    limits.tasks.charters = 2;
    let mut current = Current::new(configured.defaults, &limits, &[]);
    let value = Value::Overlay(Box::new(Overlay::empty()));
    let mut item = Item { setting: Setting::Charter(2), edit: Edit::Set(value.clone()) };
    let failure = check_change(&current, &[item.clone()], &configured.settings, &configured.connectors, &limits).unwrap_err();
    assert_eq!((failure.setting, failure.fault), (item.setting, Fault::Reference));
    item.setting = Setting::Charter(1);
    check_change(&current, &[item.clone()], &configured.settings, &configured.connectors, &limits).unwrap();
    assert!(changes_current(&current, &[item.clone()]), "even an empty overlay follows its default");
    drop(apply(&mut current, &[item.clone()], 2));
    charters::rebuild(&mut current, &limits);
    assert_eq!(current.value(item.setting), Some(&value));
    assert!(!changes_current(&current, &[item]));
    assert_eq!(current.replaced(Setting::Charter(1)), Replaced::Value(value));
    let default = current.default_value(Setting::Charter(1)).unwrap().clone();
    for edit in [Edit::Reset, Edit::Set(default.clone()), Edit::Removed] {
        drop(apply(&mut current, &[Item { setting: Setting::Charter(1), edit: edit.clone() }], 3));
        charters::rebuild(&mut current, &limits);
        assert!(current.overlay(Setting::Charter(1)).is_none());
        match edit {
            Edit::Reset | Edit::Set(_) => assert_eq!(current.value(Setting::Charter(1)), Some(&default)),
            Edit::Removed => assert!(current.charter(1).is_none()),
        }
    }
}
