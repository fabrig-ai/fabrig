//! Charter parts are checked at changes, start and restore (jig/docs/design/domain/tasks.md, 8.6).
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
    // The configured part count is eight (configuration.md, 6); test both sides.
    let count = limits.tasks.charter_parts;
    let none: Box<[CharterPart]> = Box::new([]);
    for (name, parts, resume, fault) in [
        ("none", none, false, None),
        ("none may resume", Box::new([]), true, None),
        ("two", Box::new([part(Reasons::FIRST, 1), part(Reasons::QUESTION, 1)]), false, None),
        ("at count", repeated_parts(count), false, None),
        ("past count", repeated_parts(count.checked_add(1).unwrap()), false, Some(Fault::Count)),
        ("empty on", Box::new([part(Reasons::EMPTY, 1)]), false, Some(Fault::Shape)),
        ("unknown on", Box::new([part(Reasons(1 << 14), 1)]), false, Some(Fault::Shape)),
        ("resume", Box::new([part(Reasons::FIRST, 1)]), true, Some(Fault::Shape)),
        ("at run bytes", Box::new([part(Reasons::FIRST, at_bytes)]), false, None),
        ("past run bytes", Box::new([part(Reasons::FIRST, at_bytes + 1)]), false, Some(Fault::Shape)),
    ] {
        let mut charter = base.clone();
        charter.parts = parts;
        charter.policy.resume = resume;
        let supplied = policy_owned_bytes(&charter.policy)
            .unwrap()
            .checked_add(parts_owned_bytes(&charter.parts).unwrap())
            .unwrap();
        let value = Value::Charter(Box::new(charter));
        let detail = if name == "past run bytes" {
            Some(people::SettingDetail {
                member: people::SettingMember::StoredPolicy,
                document: 0,
                segment: 0,
                rule: people::SettingRule::Maximum,
                supplied: u32::try_from(supplied).unwrap(),
                bound: limits.run_bytes,
            })
        } else {
            None
        };
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
