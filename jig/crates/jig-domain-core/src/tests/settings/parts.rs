//! Charter parts are checked at changes, start and restore (jig/docs/design/domain/tasks/messages.md, 6).
use crate::settings::{Changed, Current, Edit, Failure, Fault, Item, Setting, Value};
use crate::tests::*;
use tasks::Reasons;

fn repeated_parts(count: u32) -> Box<[CharterPart]> {
    let mut parts = List::with_capacity(count);
    for _ in 0..count {
        parts.push(part(Reasons::FIRST, 1)).unwrap();
    }
    parts.into_boxed()
}

fn part(on: Reasons, bytes: usize) -> CharterPart {
    CharterPart { on, instructions: skein_lib::bytes::zeroed(bytes) }
}

#[test]
fn charter_parts_keep_their_shape_at_change_start_and_restore() {
    let mut limits = limits();
    // Room for eight small parts; the fixture's ordinary run bound is smaller.
    limits.run_bytes = 1024;
    let mut configured = config(1);
    configured.defaults.charters[0].policy.resume = false;
    let base = configured.defaults.charters[0].clone();
    let current = Current::new(config(1).defaults, &limits, &[]);
    let at_bytes = usize::try_from(
        u64::from(limits.run_bytes)
            - policy_owned_bytes(&base.policy).unwrap()
            - parts_owned_bytes(&[part(Reasons::FIRST, 0)]).unwrap(),
    )
    .unwrap();
    // The configured part count is eight (protocol/limits.md); test both sides.
    let count = limits.tasks.charter_parts;
    let none: Box<[CharterPart]> = Box::new([]);
    let past_parts = Box::new([part(Reasons::FIRST, at_bytes + 1)]);
    let supplied = policy_owned_bytes(&base.policy)
        .unwrap()
        .checked_add(parts_owned_bytes(past_parts.as_slice()).unwrap())
        .unwrap();
    let past_detail = Some(people::SettingDetail {
        member: people::SettingMember::StoredPolicy,
        document: 0,
        segment: 0,
        rule: people::SettingRule::Maximum,
        supplied: u32::try_from(supplied).unwrap(),
        bound: limits.run_bytes,
    });
    for (name, parts, resume, fault, detail) in [
        ("none", none, false, None, None),
        ("none may resume", Box::new([]), true, None, None),
        ("two", Box::new([part(Reasons::FIRST, 1), part(Reasons::QUESTION, 1)]), false, None, None),
        ("at count", repeated_parts(count), false, None, None),
        ("past count", repeated_parts(count.checked_add(1).unwrap()), false, Some(Fault::Count), None),
        ("empty on", Box::new([part(Reasons::EMPTY, 1)]), false, Some(Fault::Shape), None),
        ("unknown on", Box::new([part(Reasons(1 << 14), 1)]), false, Some(Fault::Shape), None),
        ("resume", Box::new([part(Reasons::FIRST, 1)]), true, None, None),
        ("at run bytes", Box::new([part(Reasons::FIRST, at_bytes)]), false, None, None),
        ("past run bytes", past_parts, false, Some(Fault::Shape), past_detail),
    ] {
        let mut charter = base.clone();
        charter.parts = parts;
        charter.policy.resume = resume;
        let value = Value::Charter(Box::new(charter));
        let expected = match fault {
            Some(fault) => Err(Failure { setting: Setting::Charter(1), fault, detail }),
            None => Ok(()),
        };
        let items = [Item { setting: Setting::Charter(1), edit: Edit::Set(value.clone()) }];
        assert_eq!(
            crate::settings::check_change(&current, &items, &configured.settings, &configured.connectors, &limits),
            expected,
            "{name}: change"
        );
        assert_eq!(
            crate::settings::check::restore_fit(Setting::Charter(1), &value, &limits),
            expected,
            "{name}: restore"
        );
        if let Some(fault) = fault {
            check_refused_change(&items, &limits, fault, detail, name);
        }
        let mut at_start = Current::new(config(1).defaults, &limits, &[]);
        at_start.defaults.get_mut(&Setting::Charter(1)).unwrap().value = value.clone();
        assert_eq!(
            crate::settings::check(&at_start, &configured.settings, &configured.connectors, &limits),
            expected,
            "{name}: defaults"
        );
        let mut overlay = Current::new(config(1).defaults, &limits, &[]);
        overlay.changes.insert(Setting::Charter(1), Changed { value: Some(value), revision: 2 }).unwrap();
        assert_eq!(
            crate::settings::check(&overlay, &configured.settings, &configured.connectors, &limits),
            expected,
            "{name}: overlay"
        );
    }
}

fn check_refused_change(
    items: &[Item],
    limits: &Limits,
    fault: Fault,
    detail: Option<people::SettingDetail>,
    name: &str,
) {
    use crate::routing::{party::dispatch::Routed, settings::dispatch};
    let (mut core, _) = core();
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut out = Queue::uncapped();
    let mut routed = Routed::new(&mut out);
    let result = dispatch::decide_judged(
        &mut core,
        &env,
        &mut routed,
        1,
        dispatch::Judged {
            change: crate::settings::Change {
                against: 1,
                items: items.into(),
                running: crate::settings::Running::Stays,
            },
            room: crate::settings::writes_bound(limits).unwrap(),
            kept: Box::new([]),
            proposal: None,
        },
    );
    assert_eq!(
        result,
        Err(people::Refusal::NotWellFormed {
            at: 0,
            why: people::Malformed::Settings { setting: Setting::Charter(1), fault, detail },
        }),
        "{name}: malformed reply at the charter item"
    );
}

#[test]
fn overlay_parts_are_checked_with_the_defaults_policy_and_parts() {
    use crate::settings::{Overlay, overlaid};
    let mut limits = limits();
    limits.run_bytes = 2048;
    let mut configured = config(1);
    configured.defaults.charters[0].policy.resume = false;
    configured.defaults.charters[0].parts = repeated_parts(1);
    let base = configured.defaults.charters[0].clone();
    let current = Current::new(configured.defaults, &limits, &[]);
    let room = usize::try_from(
        u64::from(limits.run_bytes)
            - policy_owned_bytes(&base.policy).unwrap()
            - parts_owned_bytes(&base.parts).unwrap()
            - parts_owned_bytes(&[part(Reasons::FIRST, 0)]).unwrap(),
    )
    .unwrap();
    let at_parts: Box<[CharterPart]> = Box::new([part(Reasons::FIRST, room)]);
    for (name, parts, resume, fault) in [
        ("at bytes", at_parts, false, None),
        ("past bytes", Box::new([part(Reasons::FIRST, room + 1)]), false, Some(Fault::Shape)),
        ("combined count", repeated_parts(limits.tasks.charter_parts), false, Some(Fault::Count)),
        ("resume", repeated_parts(1), true, Some(Fault::Shape)),
        ("unknown reason", Box::new([part(Reasons(1 << 14), 0)]), false, Some(Fault::Shape)),
    ] {
        let overlay = Overlay { parts, resume: Some(resume), ..Overlay::empty() };
        let value = Value::Overlay(Box::new(overlay.clone()));
        let result = crate::settings::check_change(
            &current,
            &[Item { setting: Setting::Charter(1), edit: Edit::Set(value) }],
            &configured.settings,
            &configured.connectors,
            &limits,
        );
        assert_eq!(
            match &result {
                Ok(()) => None,
                Err(failure) => Some(failure.fault),
            },
            fault,
            "{name}"
        );
        if let Err(failure) = result {
            assert_eq!(failure.setting, Setting::Charter(1), "{name}");
        } else {
            let made = overlaid(&base, &overlay);
            assert_eq!(
                policy_owned_bytes(&made.policy).unwrap() + parts_owned_bytes(&made.parts).unwrap(),
                u64::from(limits.run_bytes)
            );
        }
    }
}
