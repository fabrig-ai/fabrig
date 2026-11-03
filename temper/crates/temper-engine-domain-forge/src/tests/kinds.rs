//! The forge's held kinds (temper/docs/design/protocol/configuration.md, 13.2;
//! temper/docs/design/domain/forge.md, 8.9 and section 16).
use super::fixtures::{CHANGE, opening_row};
use super::*;
use crate::settings::{
    Asked, Bounds, DefaultState, Edit, Held, Judged, LandingRule, Refusal, SettingKey, SettingRow, SettingSlot,
    SizedValue, Stock, Value,
};
use core::slice::from_ref;

const BOUNDS: Bounds = Bounds { landing_rules: 4, gates: 2, roles: 2, charters: 8, segments: 8, segment_bytes: 32 };

fn rule(ci: bool) -> SizedValue {
    SizedValue {
        value: Value::Rule(LandingRule {
            pattern: GatePattern {
                segments: Box::new([Box::from(b"forge".as_slice())]),
                last: GateLast::Open(Box::new([])),
            },
            enforce: true,
            ci,
            up_to_date: true,
            gates: Box::new([]),
            approvals: Box::new([]),
        }),
        size: 32,
    }
}

fn charter(number: u32) -> SizedValue {
    SizedValue { value: Value::Charter(number), size: 4 }
}

fn stock() -> Box<[Stock]> {
    Box::new([
        Stock { kind: 1, scope: 0, number: 1, value: rule(true) },
        Stock { kind: 3, scope: 0, number: 1, value: charter(2) },
        Stock { kind: 3, scope: 0, number: 2, value: charter(4) },
    ])
}

fn stocked() -> Domain {
    let mut d = domain();
    d.give_settings_defaults(stock());
    d
}

fn item(kind: u32, scope: u32, number: u32, edit: Edit) -> Asked {
    Asked { kind, scope, number, edit }
}

fn key(item: &Asked, slot: SettingSlot) -> SettingKey {
    SettingKey { kind: item.kind, scope: item.scope, number: item.number, slot }
}

fn save(item: &Asked, slot: SettingSlot) -> Request {
    Request::Save {
        record: Stored::Setting(SettingRow { key: key(item, slot), edit: item.edit.clone() }),
        release: None,
        read_afresh: None,
    }
}

fn restore(d: &mut Domain, item: &Asked, slot: SettingSlot) {
    assert!(
        outputs(
            d,
            Event::Restore { record: Stored::Setting(SettingRow { key: key(item, slot), edit: item.edit.clone() }) }
        )
        .is_empty()
    );
}

#[test]
fn a_change_of_rules_is_judged_kept_and_held() {
    let mut d = stocked();
    let asked = item(2, 1, 1, Edit::Set(rule(false)));
    assert_eq!(d.judge_settings(&BOUNDS, from_ref(&asked)).as_ref(), &[Judged::Accept { size: 32, charter: None }]);
    let mut out = Queue::uncapped();
    d.keep_settings(&env(), None, from_ref(&asked), &mut out);
    assert_eq!(out.pop(), Some(save(&asked, SettingSlot::Change)));
    assert!(out.is_empty());
    let Value::Rule(expected) = rule(false).value else { panic!("rule fixture") };
    let (deployment, project) = d.landing_rules(1);
    assert_eq!(deployment.len(), 1);
    assert_eq!(project, &[Held { scope: 1, number: 1, rule: expected }]);
    assert_eq!(d.charters(), ForgeCharters { change: 2, review: 4 });
    assert_eq!(d.setting(2, 1, 1), Some(&rule(false)));

    // Number order follows the keys even when the write order differs.
    d.keep_settings(
        &env(),
        None,
        &[item(2, 1, 3, Edit::Set(rule(true))), item(2, 1, 2, Edit::Set(rule(false)))],
        &mut out,
    );
    for (held, number) in d.landing_rules(1).1.iter().zip([1, 2, 3]) {
        assert_eq!(held.number, number);
    }
}

#[test]
fn items_that_leave_a_setting_as_it_is_are_the_same() {
    let mut d = stocked();
    let set = item(2, 1, 1, Edit::Set(rule(false)));
    let removed = item(2, 1, 2, Edit::Remove);
    let mut out = Queue::uncapped();
    d.keep_settings(&env(), None, &[set.clone(), removed.clone()], &mut out);
    let reset = item(1, 0, 1, Edit::Reset);
    assert_eq!(d.judge_settings(&BOUNDS, &[set, removed, reset]).as_ref(), &[Judged::Same; 3]);
    let mut sized = rule(false);
    sized.size += 1;
    assert_eq!(
        d.judge_settings(&BOUNDS, &[item(2, 1, 1, Edit::Set(sized))]).as_ref(),
        &[Judged::Accept { size: 33, charter: None }]
    );
    assert_eq!(d.setting(2, 1, 2), None);
}

#[test]
fn a_reset_is_judged_as_its_default() {
    let mut d = stocked();
    let mut out = Queue::uncapped();
    d.keep_settings(
        &env(),
        None,
        &[item(3, 0, 1, Edit::Set(charter(6))), item(1, 0, 2, Edit::Set(rule(false)))],
        &mut out,
    );
    let items = [item(3, 0, 1, Edit::Reset), item(1, 0, 2, Edit::Reset)];
    assert_eq!(
        d.judge_settings(&BOUNDS, &items).as_ref(),
        &[Judged::Accept { size: 4, charter: Some(2) }, Judged::Accept { size: 0, charter: None }]
    );
    let mut erased = Queue::uncapped();
    d.keep_settings(&env(), None, &items, &mut erased);
    for item in &items {
        assert_eq!(erased.pop(), Some(Request::Erase { key: Key::Setting(key(item, SettingSlot::Change)) }));
    }
    assert!(erased.is_empty());
    assert_eq!(d.charters().change, 2);
    assert_eq!(d.setting(1, 0, 2), None);
}

#[test]
fn a_refused_item_names_what_fails() {
    let d = stocked();
    assert_eq!(
        d.judge_settings(
            &BOUNDS,
            &[item(3, 0, 1, Edit::Set(charter(BOUNDS.charters + 1))), item(3, 0, 2, Edit::Remove)]
        )
        .as_ref(),
        &[Judged::Refuse(Refusal::Charter); 2]
    );
}

#[test]
fn an_accepted_proposal_keeps_its_kept_values() {
    let mut d = stocked();
    let items = [item(2, 1, 1, Edit::Set(rule(false))), item(1, 0, 1, Edit::Remove), item(3, 0, 1, Edit::Reset)];
    let mut out = Queue::uncapped();
    d.keep_proposal_settings(7, &items, &mut out);
    for item in &items {
        assert_eq!(out.pop(), Some(save(item, SettingSlot::Proposal(7))));
    }
    assert!(out.is_empty());
    assert_eq!(d.proposal_settings(7).len(), 3);
    assert_eq!(d.setting(2, 1, 1), None, "a proposal does not change current values");
    let supplied = [
        item(2, 1, 1, Edit::Set(rule(true))),
        item(1, 0, 1, Edit::Set(rule(false))),
        item(3, 0, 1, Edit::Set(charter(6))),
        item(2, 1, 2, Edit::Set(rule(true))),
    ];
    d.keep_settings(&env(), Some(7), &supplied, &mut out);
    assert_eq!(out.pop(), Some(save(&items[0], SettingSlot::Change)));
    assert_eq!(out.pop(), Some(save(&items[1], SettingSlot::Change)));
    assert_eq!(out.pop(), Some(save(&supplied[3], SettingSlot::Change)), "missing proposal item uses supplied edit");
    for (key, _) in [
        (key(&items[1], SettingSlot::Proposal(7)), Edit::Remove),
        (key(&items[0], SettingSlot::Proposal(7)), items[0].edit.clone()),
        (key(&items[2], SettingSlot::Proposal(7)), Edit::Reset),
    ] {
        assert_eq!(out.pop(), Some(Request::Erase { key: Key::Setting(key) }));
    }
    assert!(out.is_empty());
    assert!(d.proposal_settings(7).is_empty());
    assert_eq!(d.setting(2, 1, 1), Some(&rule(false)));
    assert_eq!(d.setting(1, 0, 1), None);
    assert_eq!(d.charters().change, 2);
}

#[test]
fn a_dropped_proposal_erases_its_items() {
    let mut d = stocked();
    let asked = item(2, 1, 1, Edit::Reset);
    let mut out = Queue::uncapped();
    d.keep_proposal_settings(7, from_ref(&asked), &mut out);
    d.keep_proposal_settings(8, from_ref(&asked), &mut out);
    assert_eq!(out.pop(), Some(save(&asked, SettingSlot::Proposal(7))));
    assert_eq!(out.pop(), Some(save(&asked, SettingSlot::Proposal(8))));
    assert_eq!(d.proposal_settings(7).as_ref(), &[(key(&asked, SettingSlot::Proposal(7)), Edit::Reset)]);
    d.drop_proposal_settings(7, &mut out);
    assert_eq!(out.pop(), Some(Request::Erase { key: Key::Setting(key(&asked, SettingSlot::Proposal(7))) }));
    assert!(out.is_empty());
    assert!(d.proposal_settings(7).is_empty());
    assert_eq!(d.proposal_settings(8).len(), 1);
}

#[test]
fn defaults_say_which_differ_and_are_saved() {
    let mut d = domain();
    let same = item(1, 0, 1, Edit::Set(rule(true)));
    let different = item(3, 0, 1, Edit::Set(charter(6)));
    let obsolete = item(1, 0, 2, Edit::Set(rule(false)));
    for item in [&same, &different, &obsolete] {
        restore(&mut d, item, SettingSlot::Default);
    }
    assert_eq!(d.setting(3, 0, 1), Some(&charter(6)), "stored default before stock");
    assert_eq!(
        d.give_settings_defaults(stock()).as_ref(),
        &[
            DefaultState { kind: 1, scope: 0, number: 1, size: 32, charter: None, differs: false },
            DefaultState { kind: 3, scope: 0, number: 1, size: 4, charter: Some(2), differs: true },
            DefaultState { kind: 3, scope: 0, number: 2, size: 4, charter: Some(4), differs: true },
        ]
    );
    assert_eq!(d.setting(1, 0, 2), None, "old defaults do not survive current stock");
    let mut out = Queue::uncapped();
    d.save_settings_defaults(&mut out);
    assert_eq!(out.pop(), Some(Request::Erase { key: Key::Setting(key(&obsolete, SettingSlot::Default)) }));
    assert_eq!(out.pop(), Some(save(&item(3, 0, 1, Edit::Set(charter(2))), SettingSlot::Default)));
    assert_eq!(out.pop(), Some(save(&item(3, 0, 2, Edit::Set(charter(4))), SettingSlot::Default)));
    assert!(out.is_empty());
    d.save_settings_defaults(&mut out);
    assert!(out.is_empty(), "saved defaults match");
    let mut replacement = stock();
    replacement[0].value.size += 1;
    assert!(d.give_settings_defaults(replacement)[0].differs, "size is part of a default");
    assert!(d.give_settings_defaults(Box::new([])).is_empty());
    assert_eq!(d.setting(1, 0, 1), None);
    assert_eq!(d.charters(), ForgeCharters { change: 0, review: 0 });
    d.save_settings_defaults(&mut out);
    assert_eq!(out.len(), 3, "empty stock erases every saved default");
}

#[test]
fn a_change_of_rules_wakes_the_live_changes_it_covers() {
    let mut d = stocked();
    let mut repository = adopted();
    repository.project = 1;
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    outputs(&mut d, Event::Restore { record: Stored::Change(opening_row()) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    d.offered_change(CHANGE);
    assert_eq!(d.next_wall_deadline(), None);
    let mut out = Queue::uncapped();
    d.keep_settings(&env(), None, &[item(2, 1, 1, Edit::Set(rule(false)))], &mut out);
    assert_eq!(d.next_wall_deadline(), Some(env().wall));
    d.offered_change(CHANGE);
    d.keep_settings(
        &env(),
        None,
        &[item(2, 2, 1, Edit::Set(rule(false))), item(3, 0, 1, Edit::Set(charter(6)))],
        &mut out,
    );
    assert_eq!(d.next_wall_deadline(), None);
    d.keep_settings(&env(), None, &[item(2, 1, 1, Edit::Set(rule(false)))], &mut out);
    assert_eq!(d.next_wall_deadline(), None, "unchanged rules do not wake");
    d.keep_settings(&env(), None, &[item(1, 0, 1, Edit::Remove)], &mut out);
    assert_eq!(d.next_wall_deadline(), Some(env().wall), "deployment rules cover the change");
}

#[test]
fn a_released_project_erases_its_rules() {
    let mut d = stocked();
    let project = item(2, 1, 1, Edit::Set(rule(false)));
    for slot in [SettingSlot::Default, SettingSlot::Change, SettingSlot::Proposal(7)] {
        restore(&mut d, &project, slot);
    }
    let deployment = item(1, 0, 2, Edit::Set(rule(false)));
    restore(&mut d, &deployment, SettingSlot::Change);
    let other = item(2, 2, 1, Edit::Set(rule(true)));
    restore(&mut d, &other, SettingSlot::Change);
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert_eq!(d.landing_rules(1).1.len(), 1);
    let sent = outputs(&mut d, Event::ReleaseProject { project: 1, request: Token::new(9) });
    assert_eq!(
        sent.as_ref(),
        &[
            Request::Erase { key: Key::Setting(key(&project, SettingSlot::Default)) },
            Request::Erase { key: Key::Setting(key(&project, SettingSlot::Change)) },
            Request::Erase { key: Key::Setting(key(&project, SettingSlot::Proposal(7))) },
            Request::ProjectReleased { request: Token::new(9) },
        ]
    );
    assert!(d.landing_rules(1).1.is_empty());
    assert!(d.proposal_settings(7).is_empty());
    assert_eq!(d.setting(1, 0, 2), Some(&rule(false)));
    assert_eq!(d.landing_rules(2).1.len(), 1);
    assert_eq!(d.charters(), ForgeCharters { change: 2, review: 4 });
}

#[test]
fn restored_rows_are_held_at_the_end_of_the_restore() {
    let mut d = domain();
    restore(&mut d, &item(1, 0, 1, Edit::Set(rule(true))), SettingSlot::Default);
    restore(&mut d, &item(1, 0, 1, Edit::Remove), SettingSlot::Change);
    restore(&mut d, &item(2, 1, 2, Edit::Set(rule(false))), SettingSlot::Change);
    restore(&mut d, &item(3, 0, 1, Edit::Set(charter(2))), SettingSlot::Default);
    restore(&mut d, &item(3, 0, 1, Edit::Set(charter(6))), SettingSlot::Change);
    restore(&mut d, &item(3, 0, 2, Edit::Set(charter(4))), SettingSlot::Default);
    restore(&mut d, &item(3, 0, 2, Edit::Set(charter(8))), SettingSlot::Proposal(7));
    assert!(d.landing_rules(1).1.is_empty());
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert!(d.landing_rules(1).0.is_empty());
    assert_eq!(d.landing_rules(1).1.len(), 1);
    assert_eq!(d.landing_rules(1).1[0].number, 2);
    assert_eq!(d.charters(), ForgeCharters { change: 6, review: 4 });
    d.give_settings_defaults(stock());
    assert_eq!(d.charters().change, 6, "stored changes override current stock");
}
