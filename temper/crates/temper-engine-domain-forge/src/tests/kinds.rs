//! The forge's held kinds (temper/docs/design/protocol/configuration/settings.md, 2;
//! temper/docs/design/domain/forge/change/gates.md, 3, and temper/docs/design/domain/forge/README.md, 9).
use super::fixtures::{CHANGE, opening_row};
use super::*;
use crate::settings::{
    Asked, Bounds, DefaultState, Edit, Held, Judged, LandingRule, Refusal, SettingKey, SettingRow, SettingSlot,
    SizedValue, Stock, TemplateGoal, Value,
};
use core::slice::from_ref;
use jig_domain_core::settings::application::{Restored, RestoredEdit};
use skein_lib::List;

const GOAL: TemplateGoal = TemplateGoal { design: true, close_out: false };

const BOUNDS: Bounds =
    Bounds { landing_rules: 4, gates: 2, roles: 2, charters: 8, segments: 8, segment_bytes: 32, templates: 4 };

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
    d.keep_settings(&env(), 2, None, from_ref(&asked), &mut out);
    assert_eq!(out.pop(), Some(save(&asked, SettingSlot::Change)));
    assert!(out.is_empty());
    let Value::Rule(expected) = rule(false).value else { panic!("rule fixture") };
    let held = d.landing_rules(1);
    assert_eq!(held.len(), 2);
    assert_eq!(held.first().expect("deployment rule").scope, 0);
    assert_eq!(&held[1..], &[Held { scope: 1, number: 1, rule: expected }]);
    assert_eq!(d.charters(), ForgeCharters { change: 2, review: 4 });
    assert_eq!(d.setting(2, 1, 1), Some(&rule(false)));

    // Number order follows the keys even when the write order differs.
    d.keep_settings(
        &env(),
        2,
        None,
        &[item(2, 1, 3, Edit::Set(rule(true))), item(2, 1, 2, Edit::Set(rule(false)))],
        &mut out,
    );
    for (held, number) in project_rules(&d, 1).iter().zip([1, 2, 3]) {
        assert_eq!(held.number, number);
    }
}

fn project_rules(d: &Domain, project: u32) -> Box<[Held]> {
    let mut rules = List::uncapped();
    for held in d.landing_rules(project) {
        if held.scope != 0 {
            rules.push(held).expect("bounded project rules");
        }
    }
    rules.into_boxed()
}

fn given_rule() -> SizedValue {
    SizedValue {
        value: Value::Rule(LandingRule {
            pattern: GatePattern {
                segments: Box::new([b"branch".as_slice().into()]),
                last: GateLast::Open(Box::new([])),
            },
            enforce: true,
            ci: true,
            up_to_date: true,
            gates: Box::new([crate::settings::RuleGate {
                number: 1,
                blocking: true,
                freshness: JudgeFreshness::Clean,
            }]),
            approvals: Box::new([]),
        }),
        size: 64,
    }
}

fn given_stock() -> Box<[Stock]> {
    let mut defaults = List::uncapped();
    for item in stock() {
        defaults.push(item).expect("bounded stock");
    }
    defaults.push(Stock { kind: 2, scope: 0, number: 1, value: given_rule() }).expect("given rule fits stock");
    defaults.into_boxed()
}

#[test]
fn every_project_holds_its_given_rule_until_it_sets_its_own() {
    for edit in [Edit::Reset, Edit::Remove] {
        let mut d = domain();
        let states = d.give_settings_defaults(given_stock());
        for state in &states {
            assert_ne!(state.kind, 2, "the given rule is not jig state");
        }
        let Value::Rule(given) = given_rule().value else { panic!("given rule") };
        for project in [1, 2] {
            assert_eq!(project_rules(&d, project).as_ref(), &[Held { scope: project, number: 1, rule: given.clone() }]);
        }
        let mut own = rule(true);
        let Value::Rule(own_rule) = &mut own.value else { panic!("own rule") };
        own_rule.up_to_date = false;
        let mut out = Queue::uncapped();
        d.keep_settings(&env(), 2, None, &[item(2, 1, 1, Edit::Set(own.clone()))], &mut out);
        assert_eq!(d.setting(2, 1, 1), Some(&own));
        let Value::Rule(own_rule) = &own.value else { panic!("own rule") };
        assert_eq!(
            project_rules(&d, 1).as_ref(),
            &[Held { scope: 1, number: 1, rule: own_rule.clone() }],
            "own rule replaces the given rule"
        );
        assert_eq!(d.setting(2, 2, 1), Some(&given_rule()));
        assert_eq!(project_rules(&d, 2).as_ref(), &[Held { scope: 2, number: 1, rule: given.clone() }]);
        assert_eq!(
            d.judge_settings(&BOUNDS, &[item(2, 1, 1, Edit::Reset)]).as_ref(),
            &[Judged::Accept { size: 64, charter: None }]
        );
        d.keep_settings(&env(), 3, None, &[item(2, 1, 1, edit.clone())], &mut out);
        assert_eq!(d.setting(2, 1, 1), Some(&given_rule()), "{edit:?}");
        assert_eq!(project_rules(&d, 1).as_ref(), &[Held { scope: 1, number: 1, rule: given.clone() }]);
        d.keep_settings(&env(), 4, None, &[item(2, 2, 2, Edit::Set(own.clone()))], &mut out);
        let Value::Rule(own_rule) = own.value else { panic!("own rule") };
        assert_eq!(
            project_rules(&d, 2).as_ref(),
            &[Held { scope: 2, number: 1, rule: given }, Held { scope: 2, number: 2, rule: own_rule },]
        );
        d.save_settings_defaults(&mut out);
        let mut restarted = domain();
        for (key, edit) in &d.setting_rows {
            restore(&mut restarted, &item(key.kind, key.scope, key.number, edit.clone()), key.slot);
        }
        outputs(&mut restarted, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
        assert_eq!(restarted.landing_rules(1), d.landing_rules(1), "given rule survives restore before stock");
        assert_eq!(restarted.landing_rules(2), d.landing_rules(2), "own rule survives restore");
        d.erase_project_settings(2, &mut out);
        assert!(!d.held_rules.projects.contains_key(&2));
        for (key, _) in &d.setting_rows {
            assert!(key.kind != 2 || key.scope != 2);
        }
    }
}

#[test]
fn items_that_leave_a_setting_as_it_is_are_the_same() {
    let mut d = stocked();
    let set = item(2, 1, 1, Edit::Set(rule(false)));
    let removed = item(2, 1, 2, Edit::Remove);
    let mut out = Queue::uncapped();
    d.keep_settings(&env(), 2, None, &[set.clone(), removed.clone()], &mut out);
    let reset = item(1, 0, 1, Edit::Reset);
    assert_eq!(d.judge_settings(&BOUNDS, &[set, removed, reset]).as_ref(), &[const { Judged::Same }; 3]);
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
        2,
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
    d.keep_settings(&env(), 3, None, &items, &mut erased);
    for (item, previous) in items.iter().zip([charter(6), rule(false)]) {
        assert_eq!(
            erased.pop(),
            Some(save(&Asked { edit: Edit::Set(previous), ..item.clone() }, SettingSlot::Replaced(3)))
        );
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
        &[const { Judged::Refuse(Refusal::Charter) }; 2]
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
    d.keep_settings(&env(), 2, Some(7), &supplied, &mut out);
    assert_eq!(out.pop(), Some(save(&items[0], SettingSlot::Change)));
    assert_eq!(out.pop(), Some(save(&item(1, 0, 1, Edit::Set(rule(true))), SettingSlot::Replaced(2))));
    assert_eq!(out.pop(), Some(save(&items[1], SettingSlot::Change)));
    assert_eq!(out.pop(), Some(save(&item(3, 0, 1, Edit::Set(charter(2))), SettingSlot::Replaced(2))));
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
    d.give_settings_defaults(given_stock());
    let mut repository = adopted();
    repository.project = 1;
    outputs(&mut d, Event::Restore { record: Stored::Repository(repository) });
    outputs(&mut d, Event::Restore { record: Stored::Change(Box::new(opening_row())) });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    outputs(&mut d, Event::ReadAfresh);
    d.offered_change(CHANGE);
    assert_eq!(d.next_wall_deadline(), None);
    let mut out = Queue::uncapped();
    d.keep_settings(&env(), 2, None, &[item(2, 1, 1, Edit::Set(rule(false)))], &mut out);
    assert_eq!(d.next_wall_deadline(), Some(env().wall));
    d.offered_change(CHANGE);
    d.keep_settings(
        &env(),
        2,
        None,
        &[item(2, 2, 1, Edit::Set(rule(false))), item(3, 0, 1, Edit::Set(charter(6)))],
        &mut out,
    );
    assert_eq!(d.next_wall_deadline(), None);
    d.keep_settings(&env(), 2, None, &[item(2, 1, 1, Edit::Set(rule(false)))], &mut out);
    assert_eq!(d.next_wall_deadline(), None, "unchanged rules do not wake");
    for edit in [Edit::Reset, Edit::Set(rule(false)), Edit::Remove] {
        d.keep_settings(&env(), 3, None, &[item(2, 1, 1, edit)], &mut out);
        assert_eq!(d.next_wall_deadline(), Some(env().wall), "own and given rules wake the project");
        d.offered_change(CHANGE);
    }
    d.keep_settings(&env(), 2, None, &[item(1, 0, 1, Edit::Remove)], &mut out);
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
    assert_eq!(project_rules(&d, 1).len(), 1);
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
    assert!(project_rules(&d, 1).is_empty());
    assert!(d.proposal_settings(7).is_empty());
    assert_eq!(d.setting(1, 0, 2), Some(&rule(false)));
    assert_eq!(project_rules(&d, 2).len(), 1);
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
    assert!(project_rules(&d, 1).is_empty());
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    for held in d.landing_rules(1) {
        assert_ne!(held.scope, 0);
    }
    assert_eq!(project_rules(&d, 1).len(), 1);
    assert_eq!(project_rules(&d, 1)[0].number, 2);
    assert_eq!(d.charters(), ForgeCharters { change: 6, review: 4 });
    d.give_settings_defaults(stock());
    assert_eq!(d.charters().change, 6, "stored changes override current stock");
}

#[test]
fn a_kept_change_keeps_the_rule_it_replaced_by_its_revision() {
    let mut d = stocked();
    let first = item(1, 0, 1, Edit::Set(rule(true)));
    let second = item(1, 0, 1, Edit::Set(rule(false)));
    let mut out = Queue::uncapped();
    d.keep_settings(&env(), 2, None, from_ref(&first), &mut out);
    assert_eq!(out.pop(), Some(save(&first, SettingSlot::Replaced(2))));
    assert_eq!(out.pop(), Some(save(&first, SettingSlot::Change)));
    assert!(out.is_empty());
    d.keep_settings(&env(), 3, None, from_ref(&second), &mut out);
    assert_eq!(out.pop(), Some(save(&first, SettingSlot::Replaced(3))));
    assert_eq!(out.pop(), Some(save(&second, SettingSlot::Change)));
    assert!(out.is_empty());
    assert_eq!(d.setting_rows.get(&key(&first, SettingSlot::Replaced(2))), Some(&first.edit));
    assert_eq!(d.setting_rows.get(&key(&first, SettingSlot::Replaced(3))), Some(&first.edit));
    assert_eq!(d.setting(1, 0, 1), Some(&rule(false)));
}

#[test]
fn a_change_over_the_stock_default_keeps_its_value_by_revision() {
    let mut d = stocked();
    let default = item(3, 0, 1, Edit::Set(charter(2)));
    let changed = item(3, 0, 1, Edit::Set(charter(6)));
    let reset = item(3, 0, 1, Edit::Reset);
    let mut out = Queue::uncapped();
    d.keep_settings(&env(), 2, None, from_ref(&changed), &mut out);
    assert_eq!(out.pop(), Some(save(&default, SettingSlot::Replaced(2))));
    assert_eq!(out.pop(), Some(save(&changed, SettingSlot::Change)));
    assert!(out.is_empty());
    d.keep_settings(&env(), 3, None, from_ref(&reset), &mut out);
    assert_eq!(out.pop(), Some(save(&changed, SettingSlot::Replaced(3))));
    assert_eq!(out.pop(), Some(Request::Erase { key: Key::Setting(key(&reset, SettingSlot::Change)) }));
    assert!(out.is_empty());
    d.keep_settings(&env(), 4, None, from_ref(&changed), &mut out);
    assert_eq!(out.pop(), Some(save(&default, SettingSlot::Replaced(4))));
    assert_eq!(out.pop(), Some(save(&changed, SettingSlot::Change)));
    assert!(out.is_empty());
    for revision in [2, 4] {
        assert_eq!(d.setting_rows.get(&key(&changed, SettingSlot::Replaced(revision))), Some(&default.edit));
    }
}

#[test]
fn a_change_over_a_removed_rule_keeps_nothing_replaced() {
    let mut d = stocked();
    let removed = item(1, 0, 1, Edit::Remove);
    let changed = item(1, 0, 1, Edit::Set(rule(false)));
    let mut out = Queue::uncapped();
    d.keep_settings(&env(), 2, None, from_ref(&removed), &mut out);
    assert_eq!(out.pop(), Some(save(&item(1, 0, 1, Edit::Set(rule(true))), SettingSlot::Replaced(2))));
    assert_eq!(out.pop(), Some(save(&removed, SettingSlot::Change)));
    assert!(out.is_empty());
    d.keep_settings(&env(), 3, None, from_ref(&changed), &mut out);
    assert_eq!(out.pop(), Some(save(&changed, SettingSlot::Change)));
    assert!(out.is_empty());
}

#[test]
fn an_accepted_proposal_keeps_the_current_value_it_replaced() {
    let mut d = stocked();
    let previous = item(1, 0, 1, Edit::Set(rule(true)));
    let proposed = item(1, 0, 1, Edit::Remove);
    restore(&mut d, &previous, SettingSlot::Change);
    restore(&mut d, &proposed, SettingSlot::Proposal(7));
    let mut out = Queue::uncapped();
    d.keep_settings(&env(), 3, Some(7), from_ref(&previous), &mut out);
    assert_eq!(out.pop(), Some(save(&previous, SettingSlot::Replaced(3))));
    assert_eq!(out.pop(), Some(save(&proposed, SettingSlot::Change)));
    assert_eq!(out.pop(), Some(Request::Erase { key: Key::Setting(key(&proposed, SettingSlot::Proposal(7))) }));
    assert!(out.is_empty());
    assert_eq!(d.setting(1, 0, 1), None);
}

#[test]
fn replaced_rows_are_restored_and_change_nothing_held() {
    let mut with = domain();
    let mut without = domain();
    let rows = [
        (item(1, 0, 1, Edit::Set(rule(true))), SettingSlot::Default),
        (item(1, 0, 1, Edit::Set(rule(false))), SettingSlot::Change),
        (item(2, 1, 1, Edit::Set(rule(false))), SettingSlot::Change),
        (item(3, 0, 1, Edit::Set(charter(6))), SettingSlot::Change),
        (item(3, 0, 2, Edit::Set(charter(4))), SettingSlot::Default),
    ];
    for (item, slot) in &rows {
        restore(&mut with, item, *slot);
        restore(&mut without, item, *slot);
    }
    let replaced = [
        item(1, 0, 1, Edit::Set(rule(true))),
        item(2, 1, 1, Edit::Set(rule(true))),
        item(2, 2, 2, Edit::Set(rule(true))),
        item(3, 0, 1, Edit::Set(charter(2))),
        item(3, 0, 2, Edit::Set(charter(8))),
    ];
    for item in &replaced {
        restore(&mut with, item, SettingSlot::Replaced(3));
        assert_eq!(with.setting_rows.get(&key(item, SettingSlot::Replaced(3))), Some(&item.edit));
    }
    for d in [&mut with, &mut without] {
        outputs(d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    }
    for project in [1, 2] {
        assert_eq!(with.landing_rules(project), without.landing_rules(project));
    }
    assert_eq!(with.charters(), without.charters());
    for item in &replaced {
        assert_eq!(
            with.setting(item.kind, item.scope, item.number),
            without.setting(item.kind, item.scope, item.number)
        );
    }
    assert!(with.proposal_settings(3).is_empty());
    for d in [&mut with, &mut without] {
        d.give_settings_defaults(stock());
        d.save_settings_defaults(&mut Queue::uncapped());
    }
    assert_eq!(with.charters(), without.charters());
    assert_eq!(with.landing_rules(1), without.landing_rules(1));
    for (key, _) in &without.setting_rows {
        match key.slot {
            SettingSlot::Default | SettingSlot::Change | SettingSlot::Proposal(_) => {}
            SettingSlot::Replaced(_) => panic!("saving defaults keeps nothing replaced"),
        }
    }
}

#[test]
fn a_restore_keeps_the_replaced_rule_as_the_proposals_item_and_answers_its_size() {
    let mut d = stocked();
    let previous = item(2, 1, 1, Edit::Set(rule(true)));
    let current = item(2, 1, 1, Edit::Set(rule(false)));
    d.keep_settings(&env(), 2, None, from_ref(&previous), &mut Queue::uncapped());
    d.keep_settings(&env(), 3, None, from_ref(&current), &mut Queue::uncapped());
    let mut out = Queue::uncapped();
    let restored = Restored { kind: 2, scope: 1, number: 1, edit: RestoredEdit::Set { revision: 3 } };
    assert_eq!(
        d.restore_settings(&BOUNDS, 7, from_ref(&restored), &mut out).as_ref(),
        &[Judged::Accept { size: 32, charter: None }]
    );
    assert_eq!(out.pop(), Some(save(&previous, SettingSlot::Proposal(7))));
    assert!(out.is_empty());
    assert_eq!(d.proposal_settings(7).as_ref(), &[(key(&previous, SettingSlot::Proposal(7)), previous.edit.clone())]);
    assert_eq!(d.setting(2, 1, 1), Some(&rule(false)));
    assert_eq!(d.setting_rows.get(&key(&previous, SettingSlot::Replaced(3))), Some(&previous.edit));
}

#[test]
fn a_restore_of_the_forges_charter_names_the_charter() {
    let mut d = stocked();
    let previous = item(3, 0, 1, Edit::Set(charter(BOUNDS.charters)));
    restore(&mut d, &previous, SettingSlot::Replaced(3));
    restore(&mut d, &previous, SettingSlot::Change);
    let mut out = Queue::uncapped();
    assert_eq!(
        d.restore_settings(
            &BOUNDS,
            7,
            &[Restored { kind: 3, scope: 0, number: 1, edit: RestoredEdit::Set { revision: 3 } }],
            &mut out,
        )
        .as_ref(),
        &[Judged::Accept { size: 4, charter: Some(BOUNDS.charters) }],
        "an equal current value is accepted rather than judged the same"
    );
    assert_eq!(out.pop(), Some(save(&previous, SettingSlot::Proposal(7))));
    assert!(out.is_empty());
    assert_eq!(d.proposal_settings(7).as_ref(), &[(key(&previous, SettingSlot::Proposal(7)), previous.edit)]);
}

#[test]
fn a_restore_item_with_no_replaced_row_is_refused() {
    let mut d = stocked();
    let previous = item(2, 1, 1, Edit::Set(rule(true)));
    for slot in [SettingSlot::Default, SettingSlot::Change, SettingSlot::Proposal(3), SettingSlot::Replaced(4)] {
        restore(&mut d, &previous, slot);
    }
    let mut out = Queue::uncapped();
    let items = [
        Restored { kind: 2, scope: 1, number: 1, edit: RestoredEdit::Set { revision: 3 } },
        Restored { kind: 1, scope: 1, number: 1, edit: RestoredEdit::Set { revision: 4 } },
        Restored { kind: 2, scope: 2, number: 1, edit: RestoredEdit::Set { revision: 4 } },
        Restored { kind: 2, scope: 1, number: 2, edit: RestoredEdit::Set { revision: 4 } },
    ];
    assert_eq!(
        d.restore_settings(&BOUNDS, 7, &items, &mut out).as_ref(),
        &[const { Judged::Refuse(Refusal::Replaced) }; 4]
    );
    assert_eq!(Refusal::Replaced.words().as_ref(), b"the replaced value is no longer kept");
    assert!(out.is_empty());
    assert!(d.proposal_settings(7).is_empty());
}

#[test]
fn a_restore_keeps_removals_and_resets_and_answers_the_defaults_sizes() {
    let mut d = stocked();
    let items = [
        Restored { kind: 2, scope: 1, number: 1, edit: RestoredEdit::Remove },
        Restored { kind: 1, scope: 0, number: 1, edit: RestoredEdit::Reset },
        Restored { kind: 3, scope: 0, number: 1, edit: RestoredEdit::Reset },
        Restored { kind: 2, scope: 1, number: 2, edit: RestoredEdit::Reset },
    ];
    let mut out = Queue::uncapped();
    assert_eq!(
        d.restore_settings(&BOUNDS, 7, &items, &mut out).as_ref(),
        &[
            Judged::Accept { size: 0, charter: None },
            Judged::Accept { size: 32, charter: None },
            Judged::Accept { size: 4, charter: Some(2) },
            Judged::Accept { size: 0, charter: None },
        ]
    );
    let asked = [
        item(2, 1, 1, Edit::Remove),
        item(1, 0, 1, Edit::Reset),
        item(3, 0, 1, Edit::Reset),
        item(2, 1, 2, Edit::Reset),
    ];
    for item in &asked {
        assert_eq!(out.pop(), Some(save(item, SettingSlot::Proposal(7))));
        assert_eq!(d.setting_rows.get(&key(item, SettingSlot::Proposal(7))), Some(&item.edit));
    }
    assert!(out.is_empty());
    assert_eq!(d.charters(), ForgeCharters { change: 2, review: 4 });
    assert_eq!(d.setting(1, 0, 1), Some(&rule(true)));
}

#[test]
fn a_restore_checks_the_kinds_bounds_and_keeps_only_accepted_items_in_order() {
    let mut d = stocked();
    let previous = item(2, 1, 1, Edit::Set(rule(false)));
    let invalid = item(3, 0, 1, Edit::Set(charter(BOUNDS.charters + 1)));
    restore(&mut d, &previous, SettingSlot::Replaced(3));
    restore(&mut d, &invalid, SettingSlot::Replaced(3));
    let items = [
        Restored { kind: 3, scope: 0, number: 1, edit: RestoredEdit::Set { revision: 3 } },
        Restored { kind: 2, scope: 1, number: 1, edit: RestoredEdit::Set { revision: 3 } },
        Restored { kind: 3, scope: 0, number: 2, edit: RestoredEdit::Remove },
        Restored { kind: 2, scope: 1, number: BOUNDS.landing_rules + 1, edit: RestoredEdit::Reset },
    ];
    let mut out = Queue::uncapped();
    assert_eq!(
        d.restore_settings(&BOUNDS, 7, &items, &mut out).as_ref(),
        &[
            Judged::Refuse(Refusal::Charter),
            Judged::Accept { size: 32, charter: None },
            Judged::Refuse(Refusal::Charter),
            Judged::Refuse(Refusal::Number),
        ]
    );
    assert_eq!(out.pop(), Some(save(&previous, SettingSlot::Proposal(7))));
    assert!(out.is_empty());
    assert_eq!(d.proposal_settings(7).len(), 1);
    d.drop_proposal_settings(7, &mut out);
    assert_eq!(out.pop(), Some(Request::Erase { key: Key::Setting(key(&previous, SettingSlot::Proposal(7))) }));
    assert!(out.is_empty());
    assert!(d.proposal_settings(7).is_empty());
}

#[test]
fn forgetting_erases_replaced_rows_of_older_revisions_only() {
    let mut d = stocked();
    let items = [
        item(1, 0, 1, Edit::Set(rule(false))),
        item(2, 1, 1, Edit::Set(rule(true))),
        item(3, 0, 1, Edit::Set(charter(6))),
    ];
    let slots = [
        SettingSlot::Default,
        SettingSlot::Change,
        SettingSlot::Proposal(2),
        SettingSlot::Replaced(2),
        SettingSlot::Replaced(3),
        SettingSlot::Replaced(4),
    ];
    for item in &items {
        for slot in slots {
            restore(&mut d, item, slot);
        }
    }
    let mut out = Queue::uncapped();
    d.forget_replaced(0, &mut out);
    assert!(out.is_empty());
    d.forget_replaced(3, &mut out);
    for item in &items {
        assert_eq!(out.pop(), Some(Request::Erase { key: Key::Setting(key(item, SettingSlot::Replaced(2))) }));
        for slot in slots {
            let expected = if slot == SettingSlot::Replaced(2) { None } else { Some(&item.edit) };
            assert_eq!(d.setting_rows.get(&key(item, slot)), expected);
        }
    }
    assert!(out.is_empty());
    d.forget_replaced(3, &mut out);
    assert!(out.is_empty(), "forgetting twice emits no duplicate erases");
    assert_eq!(d.setting(1, 0, 1), Some(&rule(false)));
    assert_eq!(d.setting(2, 1, 1), Some(&rule(true)));
    assert_eq!(d.proposal_settings(2).len(), 3);
}

fn words() -> crate::settings::ProjectWords {
    crate::settings::ProjectWords { headline: Box::from(b"headline".as_slice()), lede: Box::from(b"lede".as_slice()) }
}

fn project_values() -> [(u32, SizedValue); 6] {
    [
        (12, SizedValue { value: Value::ProjectAreas(crate::settings::ProjectAreas { areas: Box::new([]) }), size: 4 }),
        (
            13,
            SizedValue {
                value: Value::ProjectTriager(crate::settings::ProjectTriager {
                    owner: crate::settings::AreaOwner::Charter(14),
                }),
                size: 5,
            },
        ),
        (10, SizedValue { value: Value::ProjectWords(words()), size: 20 }),
        (6, SizedValue { value: Value::Questions(temper_engine_domain_forge_change::Route::Foreman), size: 1 }),
        (
            7,
            SizedValue {
                value: Value::Ways(crate::settings::ProjectWays { drafts: true, goals: false, changes: true, way: 0 }),
                size: 7,
            },
        ),
        (
            8,
            SizedValue {
                value: Value::Classes(temper_protocol_reaches::Classes::stock(
                    temper_protocol_reaches::Way::YouApprove,
                )),
                size: temper_protocol_reaches::classes_record(&temper_protocol_reaches::Classes::stock(
                    temper_protocol_reaches::Way::YouApprove,
                ))
                .unwrap()
                .measure(),
            },
        ),
    ]
}

#[test]
fn a_deleted_project_loses_its_questions_and_ways() {
    let mut d = stocked();
    let line = b"charter line".as_slice();
    let charter = item(9, 0, 1, Edit::Set(SizedValue { value: Value::CharterWords(Box::from(line)), size: 20 }));
    restore(&mut d, &charter, SettingSlot::Change);
    let goal = item(11, 0, 1, Edit::Set(SizedValue { value: Value::Goal(GOAL), size: 2 }));
    for slot in [SettingSlot::Default, SettingSlot::Change, SettingSlot::Proposal(7), SettingSlot::Replaced(3)] {
        restore(&mut d, &goal, slot);
    }
    for (kind, value) in project_values() {
        let project = item(kind, 1, 1, Edit::Set(value.clone()));
        for slot in [SettingSlot::Default, SettingSlot::Change, SettingSlot::Proposal(7), SettingSlot::Replaced(3)] {
            restore(&mut d, &project, slot);
        }
        restore(&mut d, &item(kind, 2, 1, Edit::Set(value)), SettingSlot::Change);
    }
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let sent = outputs(&mut d, Event::ReleaseProject { project: 1, request: Token::new(9) });
    for (kind, value) in project_values() {
        for slot in [SettingSlot::Default, SettingSlot::Change, SettingSlot::Proposal(7), SettingSlot::Replaced(3)] {
            let key = SettingKey { kind, scope: 1, number: 1, slot };
            assert!(!d.setting_rows.contains_key(&key), "deleted {key:?}");
            assert!(sent.contains(&Request::Erase { key: Key::Setting(key) }), "erase {key:?}");
        }
        assert_eq!(d.setting(kind, 1, 1), None);
        assert_eq!(d.setting(kind, 2, 1), Some(&value));
    }
    // Four slots of each of the four project kinds, then the release acknowledgement.
    assert_eq!(sent.len(), 17);
    assert_eq!(d.charter_words(1), Some(line));
    assert_eq!(d.template_goal(1), Some(GOAL));
    for slot in [SettingSlot::Default, SettingSlot::Change, SettingSlot::Proposal(7), SettingSlot::Replaced(3)] {
        assert_eq!(d.setting_rows.get(&key(&goal, slot)), Some(&goal.edit));
    }
    assert_eq!(d.charters(), ForgeCharters { change: 2, review: 4 });
}

#[test]
fn project_questions_reads_none_until_set_then_the_route() {
    use temper_engine_domain_forge_change::Route;
    let mut d = stocked();
    assert_eq!(d.project_questions(1), Route::None);
    assert_eq!(d.setting(6, 1, 1), None);
    assert_eq!(d.setting(7, 1, 1), None);
    let mut out = Queue::uncapped();
    for route in [Route::None, Route::Foreman, Route::Up] {
        let value = SizedValue { value: Value::Questions(route), size: 1 };
        let asked = item(6, 1, 1, Edit::Set(value));
        d.keep_settings(&env(), 2, None, from_ref(&asked), &mut out);
        assert_eq!(d.project_questions(1), route);
        assert_eq!(d.project_questions(2), Route::None);
        assert_eq!(d.judge_settings(&BOUNDS, from_ref(&asked)).as_ref(), &[Judged::Same]);
    }
    for edit in [Edit::Remove, Edit::Reset] {
        d.keep_settings(&env(), 3, None, &[item(6, 1, 1, edit.clone())], &mut out);
        assert_eq!(d.project_questions(1), Route::None);
        assert_eq!(d.judge_settings(&BOUNDS, &[item(6, 1, 1, edit)]).as_ref(), &[Judged::Same]);
    }
    assert_eq!(d.project_words(1), None);
    assert_eq!(d.charter_words(1), None);
    let words = words();
    let line = b"charter line".as_slice();
    let charter = item(9, 0, 1, Edit::Set(SizedValue { value: Value::CharterWords(Box::from(line)), size: 20 }));
    let project = item(10, 1, 1, Edit::Set(SizedValue { value: Value::ProjectWords(words.clone()), size: 20 }));
    for asked in [&charter, &project] {
        assert_eq!(d.judge_settings(&BOUNDS, from_ref(asked)).as_ref(), &[Judged::Accept { size: 20, charter: None }]);
        d.keep_settings(&env(), 4, None, from_ref(asked), &mut out);
    }
    assert_eq!(d.project_words(1), Some(&words));
    assert_eq!(d.project_words(2), None);
    assert_eq!(d.charter_words(1), Some(line));
    // A default answers through the same accessor, and a removal hides it.
    let mut defaults = List::uncapped();
    for default in stock() {
        defaults.push(default).unwrap();
    }
    defaults
        .push(Stock {
            kind: 9,
            scope: 0,
            number: 1,
            value: SizedValue { value: Value::CharterWords(Box::from(line)), size: 20 },
        })
        .unwrap();
    d.give_settings_defaults(defaults.into_boxed());
    for (edit, charter_size, charter_line) in [(Edit::Remove, 0, None), (Edit::Reset, 20, Some(line))] {
        for (asked, size) in [(item(9, 0, 1, edit.clone()), charter_size), (item(10, 1, 1, edit), 0)] {
            assert_eq!(d.judge_settings(&BOUNDS, from_ref(&asked)).as_ref(), &[Judged::Accept { size, charter: None }]);
            d.keep_settings(&env(), 5, None, from_ref(&asked), &mut out);
        }
        assert_eq!(d.project_words(1), None);
        assert_eq!(d.charter_words(1), charter_line);
    }
    assert_eq!(d.template_goal(1), None);
    let value = SizedValue { value: Value::Goal(GOAL), size: 2 };
    let asked = item(11, 0, 1, Edit::Set(value.clone()));
    d.keep_settings(&env(), 4, None, from_ref(&asked), &mut out);
    assert_eq!(d.template_goal(1), Some(GOAL));
    assert_eq!(d.template_goal(2), None);
    assert_eq!(
        d.judge_settings(&BOUNDS, &[item(11, 0, 1, Edit::Remove)]).as_ref(),
        &[Judged::Accept { size: 0, charter: None }]
    );
    d.keep_settings(&env(), 5, None, &[item(11, 0, 1, Edit::Remove)], &mut out);
    assert_eq!(d.template_goal(1), None);
    d.give_settings_defaults(Box::new([Stock { kind: 11, scope: 0, number: 1, value }]));
    assert_eq!(d.template_goal(1), None, "removal hides the stock default");
    assert_eq!(
        d.judge_settings(&BOUNDS, &[item(11, 0, 1, Edit::Reset)]).as_ref(),
        &[Judged::Accept { size: 2, charter: None }]
    );
    d.keep_settings(&env(), 6, None, &[item(11, 0, 1, Edit::Reset)], &mut out);
    assert_eq!(d.template_goal(1), Some(GOAL), "reset reveals the stock default");
}

#[test]
fn project_values_keep_replace_restore_and_propose_as_rules_do() {
    let mut d = stocked();
    let mut out = Queue::uncapped();
    for (kind, value) in [(11, SizedValue { value: Value::Goal(GOAL), size: 2 })].into_iter().chain(project_values()) {
        let stock = temper_protocol_reaches::Classes::stock(temper_protocol_reaches::Way::AgentsReview);
        assert_eq!(d.project_classes(1), stock);
        let scope = u32::from(kind != 11);
        let asked = item(kind, scope, 1, Edit::Set(value.clone()));
        d.keep_settings(&env(), 2, None, from_ref(&asked), &mut out);
        if let Value::Classes(classes) = &value.value {
            assert_eq!(d.project_classes(1), *classes);
            assert_eq!(d.project_classes(2), stock);
        }
        assert_eq!(d.judge_settings(&BOUNDS, from_ref(&asked)).as_ref(), &[Judged::Same]);
        d.keep_settings(&env(), 3, None, &[item(kind, scope, 1, Edit::Remove)], &mut out);
        assert_eq!(d.project_classes(1), stock);
        assert_eq!(d.setting_rows.get(&key(&asked, SettingSlot::Replaced(3))), Some(&asked.edit));
        assert_eq!(
            d.restore_settings(
                &BOUNDS,
                7,
                &[Restored { kind, scope, number: 1, edit: RestoredEdit::Set { revision: 3 } }],
                &mut out
            )
            .as_ref(),
            &[Judged::Accept { size: value.size, charter: None }]
        );
        d.keep_settings(&env(), 4, Some(7), from_ref(&asked), &mut out);
        assert_eq!(d.setting(kind, scope, 1), Some(&value));
        assert!(d.proposal_settings(7).is_empty());
    }
}

#[test]
fn dropped_defaults_name_the_stored_ones_the_stock_no_longer_gives() {
    let mut d = domain();
    for (stored, slot) in [
        (item(1, 0, 2, Edit::Set(rule(false))), SettingSlot::Default),
        (item(1, 0, 1, Edit::Set(rule(true))), SettingSlot::Default),
        (item(2, 0, 2, Edit::Set(rule(false))), SettingSlot::Default),
        (item(1, 0, 3, Edit::Set(rule(false))), SettingSlot::Change),
    ] {
        restore(&mut d, &stored, slot);
    }
    d.give_settings_defaults(stock());
    assert_eq!(d.dropped_settings_defaults().as_ref(), &[(1, 0, 2)]);
}
