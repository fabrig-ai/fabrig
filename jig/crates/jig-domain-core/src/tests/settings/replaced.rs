//! Revisions keep the states their items replaced (jig/docs/design/domain/engine.md, 17.10).
use crate::routing::{party::dispatch::Routed, settings::dispatch};
use crate::settings::{self as s, application as app};
use crate::tests::*;

fn fixture() -> (Core, Limits) {
    let mut limits = limits();
    limits.tasks.charters = 2;
    let mut configured = config(41);
    configured.kinds = Box::new([s::Kind {
        number: 1,
        scope: s::KindScope::Deployment,
        connector: 1,
        value_bytes: 64,
        per_scope: 2,
        charters: 0,
    }]);
    configured.defaults.templates = Box::new([Template {
        number: 2,
        name: b"work".as_slice().into(),
        resources: Box::default(),
        executor: people::TemplateExecutor::Chosen,
        contract: tasks::Contract::Report { words: 32 },
        shallower: 0,
        budget: people::TemplateBudget::Given,
        deadline: people::TemplateDeadline::None,
        tracked: None,
        flags: tasks::Flags::PLAIN,
        label: 0,
        grace: None,
        llm: None,
    }]);
    let mut core = seeded_core(configured, &limits).unwrap();
    send_task(&mut core, &limits, tasks::Event::Check);
    send_task(&mut core, &limits, tasks::Event::Restored);
    (core, limits)
}

fn revision(mut out: Queue<Request>) -> s::Revision {
    while let Some(request) = out.pop() {
        if let Request::Write(Write::Save(Record::Core(CoreRecord::Settings(s::Record::Revision {
            history: false,
            revision,
        })))) = request
        {
            return revision;
        }
    }
    panic!("committed revision");
}

fn commit(core: &mut Core, limits: &Limits, items: Box<[s::Item]>, kept: Box<[app::Kept]>) -> s::Revision {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut routed = Routed::new(&mut Queue::uncapped());
    let against = core.current.newest();
    assert_eq!(
        dispatch::decide_judged(
            core,
            &env,
            &mut routed,
            1,
            s::Change { against, items, running: s::Running::Stays },
            s::writes_bound(limits).unwrap(),
            kept,
            None,
        ),
        Ok(against.checked_add(1).unwrap())
    );
    revision(routed.out)
}

#[test]
fn a_revision_records_what_each_item_replaced() {
    let (mut core, limits) = fixture();
    let mut charter = core.current.charter(1).unwrap().clone();
    charter.spend = charter.spend.checked_add(1).unwrap();
    let first_value = s::Value::Charter(Box::new(charter.clone()));
    let mut added = charter.clone();
    added.number = 2;
    assert_eq!(core.current.lookup(s::Setting::Charter(2)), (None, None));
    let first = commit(
        &mut core,
        &limits,
        Box::new([
            s::Item { setting: s::Setting::Charter(1), edit: s::Edit::Set(first_value.clone()) },
            s::Item { setting: s::Setting::Template(2), edit: s::Edit::Removed },
            s::Item { setting: s::Setting::Charter(2), edit: s::Edit::Set(s::Value::Charter(Box::new(added))) },
        ]),
        Box::default(),
    );
    assert_eq!(first.items.len(), 3);
    assert_eq!(first.items[0].replaced, s::Replaced::Default);
    assert_eq!(first.items[1].replaced, s::Replaced::Default);
    assert_eq!(first.items[2].replaced, s::Replaced::NotOffered);
    charter.spend = charter.spend.checked_add(1).unwrap();
    let second = commit(
        &mut core,
        &limits,
        Box::new([
            s::Item { setting: s::Setting::Charter(1), edit: s::Edit::Set(s::Value::Charter(Box::new(charter))) },
            s::Item { setting: s::Setting::Template(2), edit: s::Edit::Reset },
        ]),
        Box::default(),
    );
    assert_eq!(second.items.len(), 2);
    assert_eq!(second.items[0].replaced, s::Replaced::Value(first_value));
    assert_eq!(second.items[1].replaced, s::Replaced::Removed);
}

#[test]
fn a_release_revision_records_its_items_as_default() {
    let configured = config(41);
    let limits = limits();
    let stored = s::Current::new(configured.defaults, &limits, &[]);
    let mut configured = config(41);
    configured.defaults.charters[0].spend = configured.defaults.charters[0].spend.checked_add(1).unwrap();
    let mut core = Core::new(configured, &limits).unwrap();
    let _begun = core.start_begin(false, Wall::EPOCH);
    for (&setting, default) in &stored.defaults {
        assert!(core.restore_setting(s::Record::Default { setting, default: default.clone() }, &limits));
    }
    assert!(core.restore_setting(
        s::Record::Revision {
            history: false,
            revision: s::Revision { head: stored.head.clone(), items: Box::default(), applications: Box::default() },
        },
        &limits
    ));
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
    assert!(core.settings_restored(&env));
    let recorded = revision(core.commit_restored_settings(&env, s::writes_bound(&limits).unwrap()));
    assert_eq!(recorded.head.by, s::Decider::Release);
    assert_eq!(recorded.items.len(), 1);
    assert_eq!(recorded.items[0].setting, s::Setting::Charter(1));
    for item in &recorded.items {
        assert_eq!(item.replaced, s::Replaced::Default);
    }
}

#[test]
fn an_application_item_records_the_state_it_replaced() {
    let (mut core, limits) = fixture();
    let setting = s::Setting::Application { kind: 1, scope: 0, number: 1 };
    core.current
        .applications
        .insert(setting, app::State { follows: app::Follows::Default, size: 3, charters: Box::default() })
        .unwrap();
    for (size, expected) in [
        (10, app::Replaced::Default),
        (20, app::Replaced::Changed { size: 10 }),
        (0, app::Replaced::Changed { size: 20 }),
        (10, app::Replaced::Removed),
    ] {
        let bytes: Box<[u8]> = skein_lib::bytes::zeroed(usize::try_from(size).unwrap());
        let edit = if size == 0 { s::Edit::Removed } else { s::Edit::Set(s::Value::Application(bytes.clone())) };
        let follows = if size == 0 {
            app::Follows::Removed { revision: core.current.newest().checked_add(1).unwrap() }
        } else {
            app::Follows::Changed { revision: core.current.newest().checked_add(1).unwrap() }
        };
        let recorded = commit(
            &mut core,
            &limits,
            Box::new([s::Item { setting, edit }]),
            Box::new([app::Kept {
                item: app::Item {
                    kind: 1,
                    scope: 0,
                    number: 1,
                    edit: if size == 0 { app::Edit::Remove } else { app::Edit::Set(bytes) },
                },
                state: app::State { follows, size, charters: Box::default() },
            }]),
        );
        assert!(recorded.items.is_empty());
        assert_eq!(recorded.applications.len(), 1);
        assert_eq!(recorded.applications[0].replaced, expected);
    }
}
