//! The connector's settings rules (jig/docs/design/domain/connectors.md, section 17).
use super::run;
use crate::{
    Domain, Event, Record, RecordKey, Request, SettingDefault, SettingEdit, SettingItem, SettingJudged, SettingsKind,
};
use skein_lib::Token;

const KIND: u32 = 71;
const TOKEN: Token = Token::new(1);

fn connector() -> Domain {
    let mut domain = super::domain();
    domain.config.settings = Some(SettingsKind {
        kind: KIND,
        project: false,
        value_bytes: 16,
        per_scope: 4,
        charters: false,
        defaults: Box::from([(0, 1, Box::from(&b"default"[..]))]),
    });
    domain
}
fn item(number: u32, edit: SettingEdit) -> SettingItem {
    SettingItem { kind: KIND, scope: 0, number, edit }
}
fn set(number: u32, bytes: &[u8]) -> SettingItem {
    item(number, SettingEdit::Set(Box::from(bytes)))
}
fn judge(domain: &mut Domain, proposal: Option<u64>, items: Box<[SettingItem]>) -> Box<[Request]> {
    run(domain, Event::JudgeSettings { token: TOKEN, revision: 9, proposal, items })
}
fn kept(domain: &mut Domain, items: Box<[SettingItem]>) -> Box<[Request]> {
    run(domain, Event::KeepSettings { revision: 9, proposal: None, items })
}
fn read(domain: &mut Domain, number: u32) -> Box<[Request]> {
    run(domain, Event::ReadSetting { token: TOKEN, kind: KIND, scope: 0, number })
}
fn accept(size: u32) -> SettingJudged {
    SettingJudged::Accept { size, charters: Box::from([]) }
}
fn judgments(judged: Box<[SettingJudged]>) -> Box<[Request]> {
    Box::from([Request::SettingsJudged { token: TOKEN, judged }])
}

#[test]
fn judges_in_order_by_bytes_size_and_current_value_without_mutating() {
    let mut domain = connector();
    assert_eq!(
        judge(&mut domain, None, Box::from([set(1, b"default"), set(2, b""), set(3, b"1234567890123456")])),
        judgments(Box::from([SettingJudged::Same, accept(0), accept(16)]))
    );
    assert_eq!(read(&mut domain, 2), Box::from([Request::SettingValue { token: TOKEN, value: None }]));
    assert_eq!(
        judge(&mut domain, None, Box::from([set(1, b"!no thanks"), set(2, b"!")])),
        judgments(Box::from([
            SettingJudged::Refuse { why: Box::from(&b"refused: no thanks"[..]) },
            SettingJudged::Refuse { why: Box::from(&b"refused: "[..]) },
        ]))
    );
}

#[test]
fn charters_are_the_first_four_big_endian_bytes_only_when_declared() {
    let mut domain = connector();
    assert_eq!(judge(&mut domain, None, Box::from([set(2, &[1, 2, 3, 4, 5])])), judgments(Box::from([accept(5)])));
    domain.config.settings.as_mut().expect("kind").charters = true;
    assert_eq!(
        judge(&mut domain, None, Box::from([set(2, &[1, 2, 3]), set(3, &[1, 2, 3, 4, 5])])),
        judgments(Box::from([accept(3), SettingJudged::Accept { size: 5, charters: Box::from([0x0102_0304]) }]))
    );
}

#[test]
fn bounds_refuse_wrong_kind_scope_and_oversized_value() {
    let mut domain = connector();
    let mut wrong_kind = set(1, b"value");
    wrong_kind.kind = KIND + 1;
    let mut wrong_scope = set(1, b"value");
    wrong_scope.scope = 1;
    let refused = SettingJudged::Refuse { why: Box::from(&b"setting outside kind bounds"[..]) };
    assert_eq!(
        judge(&mut domain, None, Box::from([wrong_kind, wrong_scope, set(1, b"12345678901234567")])),
        judgments(Box::from([refused.clone(), refused.clone(), refused]))
    );
    let mut none = super::domain();
    assert_eq!(
        judge(&mut none, None, Box::from([item(1, SettingEdit::Remove)])),
        judgments(Box::from([SettingJudged::Refuse { why: Box::from(&b"setting outside kind bounds"[..]) }]))
    );
}

#[test]
fn keep_distinguishes_removed_empty_and_reset_and_reads_the_newest() {
    let mut domain = connector();
    assert_eq!(
        kept(&mut domain, Box::from([set(1, b"changed")])),
        Box::from([Request::Save {
            record: Record::SettingChange { kind: KIND, scope: 0, number: 1, value: Some(Box::from(&b"changed"[..])) }
        }])
    );
    assert_eq!(
        read(&mut domain, 1),
        Box::from([Request::SettingValue { token: TOKEN, value: Some(Box::from(&b"changed"[..])) }])
    );
    assert_eq!(
        judge(&mut domain, None, Box::from([item(1, SettingEdit::Reset), item(2, SettingEdit::Reset)])),
        judgments(Box::from([accept(7), SettingJudged::Same]))
    );
    assert_eq!(
        kept(&mut domain, Box::from([item(1, SettingEdit::Remove)])),
        Box::from([Request::Save { record: Record::SettingChange { kind: KIND, scope: 0, number: 1, value: None } }])
    );
    assert_eq!(read(&mut domain, 1), Box::from([Request::SettingValue { token: TOKEN, value: None }]));
    drop(kept(&mut domain, Box::from([set(1, b"")])));
    assert_eq!(read(&mut domain, 1), Box::from([Request::SettingValue { token: TOKEN, value: Some(Box::from([])) }]));
    assert_eq!(
        kept(&mut domain, Box::from([item(1, SettingEdit::Reset)])),
        Box::from([Request::Erase { key: RecordKey::SettingChange { kind: KIND, scope: 0, number: 1 } }])
    );
    assert_eq!(
        read(&mut domain, 1),
        Box::from([Request::SettingValue { token: TOKEN, value: Some(Box::from(&b"default"[..])) }])
    );
    assert_eq!(
        judge(&mut domain, None, Box::from([item(1, SettingEdit::Reset), item(1, SettingEdit::Remove)])),
        judgments(Box::from([SettingJudged::Same, accept(0)]))
    );
}

#[test]
fn reset_judges_the_default_including_a_missing_default_as_removal() {
    let mut domain = connector();
    drop(kept(&mut domain, Box::from([set(2, b"changed")])));
    assert_eq!(judge(&mut domain, None, Box::from([item(2, SettingEdit::Reset)])), judgments(Box::from([accept(0)])));
    domain.config.settings.as_mut().expect("kind").defaults = Box::from([(0, 1, Box::from(&b"!default"[..]))]);
    assert_eq!(
        judge(&mut domain, None, Box::from([item(1, SettingEdit::Reset)])),
        judgments(Box::from([SettingJudged::Refuse { why: Box::from(&b"refused: default"[..]) }]))
    );
}

#[test]
fn a_proposal_keeps_unjudged_bytes_and_acceptance_uses_them_then_drops_it() {
    let mut domain = connector();
    drop(run(
        &mut domain,
        Event::KeepProposalSettings { proposal: 12, items: Box::from([set(1, b"kept"), set(2, b"!later")]) },
    ));
    assert_eq!(
        read(&mut domain, 1),
        Box::from([Request::SettingValue { token: TOKEN, value: Some(Box::from(&b"default"[..])) }])
    );
    assert_eq!(
        judge(&mut domain, Some(12), Box::from([set(1, b"!ignored"), set(2, b"ignored")])),
        judgments(Box::from([accept(4), SettingJudged::Refuse { why: Box::from(&b"refused: later"[..]) }]))
    );
    assert_eq!(run(&mut domain, Event::DropProposalSettings { proposal: 12 }).len(), 2);
    assert!(run(&mut domain, Event::DropProposalSettings { proposal: 12 }).is_empty());
    assert_eq!(
        judge(&mut domain, Some(12), Box::from([set(1, b"ignored")])),
        judgments(Box::from([SettingJudged::Refuse { why: Box::from(&b"proposal setting missing"[..]) }]))
    );
    drop(run(&mut domain, Event::KeepProposalSettings { proposal: 13, items: Box::from([set(1, b"kept")]) }));
    assert_eq!(
        run(
            &mut domain,
            Event::KeepSettings { revision: 10, proposal: Some(13), items: Box::from([set(1, b"ignored")]) }
        ),
        Box::from([
            Request::Save {
                record: Record::SettingChange { kind: KIND, scope: 0, number: 1, value: Some(Box::from(&b"kept"[..])) }
            },
            Request::Erase { key: RecordKey::SettingProposal { proposal: 13, kind: KIND, scope: 0, number: 1 } },
        ])
    );
}

#[test]
fn a_proposals_reset_follows_the_default_at_acceptance() {
    let mut domain = connector();
    drop(run(
        &mut domain,
        Event::KeepProposalSettings { proposal: 12, items: Box::from([item(1, SettingEdit::Reset)]) },
    ));
    drop(kept(&mut domain, Box::from([set(1, b"changed")])));
    domain.config.settings.as_mut().expect("kind").defaults = Box::from([(0, 1, Box::from(&b"new"[..]))]);
    assert_eq!(judge(&mut domain, Some(12), Box::from([set(1, b"ignored")])), judgments(Box::from([accept(3)])));
    drop(run(
        &mut domain,
        Event::KeepSettings { revision: 10, proposal: Some(12), items: Box::from([set(1, b"ignored")]) },
    ));
    assert_eq!(
        read(&mut domain, 1),
        Box::from([Request::SettingValue { token: TOKEN, value: Some(Box::from(&b"new"[..])) }])
    );
}

fn default(number: u32, size: u32, differs: bool) -> SettingDefault {
    SettingDefault { kind: KIND, scope: 0, number, size, charters: Box::from([]), differs }
}
#[test]
fn defaults_compare_same_changed_added_and_gone_and_save_erases_gone() {
    let mut domain = connector();
    domain.config.settings.as_mut().expect("kind").defaults =
        Box::from([(0, 1, Box::from(&b"one"[..])), (0, 2, Box::from(&b"two"[..])), (0, 3, Box::from(&b"gone"[..]))]);
    drop(run(&mut domain, Event::SaveSettingsDefaults { revision: None }));
    domain.config.settings.as_mut().expect("kind").defaults = Box::from([
        (0, 1, Box::from(&b"one"[..])),
        (0, 2, Box::from(&b"new two"[..])),
        (0, 4, Box::from(&b"added"[..])),
    ]);
    assert_eq!(
        run(&mut domain, Event::SettingsDefaults),
        Box::from([Request::SettingsDefaults {
            defaults: Box::from([default(1, 3, false), default(2, 7, true), default(3, 0, true), default(4, 5, true)])
        }])
    );
    let saved = run(&mut domain, Event::SaveSettingsDefaults { revision: Some(10) });
    assert_eq!(saved.len(), 4);
    assert_eq!(saved[0], Request::Erase { key: RecordKey::SettingDefault { kind: KIND, scope: 0, number: 3 } });
    assert_eq!(
        run(&mut domain, Event::SettingsDefaults),
        Box::from([Request::SettingsDefaults {
            defaults: Box::from([default(1, 3, false), default(2, 7, false), default(4, 5, false)])
        }])
    );
}

#[test]
fn defaults_report_charters_without_judging_their_bytes() {
    let mut domain = connector();
    let kind = domain.config.settings.as_mut().expect("kind");
    kind.charters = true;
    kind.defaults = Box::from([(0, 1, Box::from(&b"!abc"[..]))]);
    assert_eq!(
        run(&mut domain, Event::SettingsDefaults),
        Box::from([Request::SettingsDefaults {
            defaults: Box::from([SettingDefault {
                kind: KIND,
                scope: 0,
                number: 1,
                size: 4,
                charters: Box::from([u32::from_be_bytes(*b"!abc")]),
                differs: true
            }])
        }])
    );
}

#[test]
fn restore_refuses_unowned_kind_scope_oversized_values_and_scope_capacity() {
    let mut domain = connector();
    for (kind, scope, value) in [(KIND + 1, 0, &b"x"[..]), (KIND, 1, &b"x"[..]), (KIND, 0, &b"12345678901234567"[..])] {
        let value: Box<[u8]> = Box::from(value);
        for record in [
            Record::SettingDefault { kind, scope, number: 1, value: value.clone() },
            Record::SettingChange { kind, scope, number: 1, value: Some(value.clone()) },
            Record::SettingProposal { proposal: 1, kind, scope, number: 1, edit: SettingEdit::Set(value.clone()) },
        ] {
            assert_eq!(
                run(&mut domain, Event::Restore { record }),
                Box::from([Request::RestoreRefused { kind, scope, number: 1 }])
            );
        }
    }
    domain.config.settings.as_mut().expect("kind").per_scope = 1;
    for number in [1, 2] {
        let restored = run(
            &mut domain,
            Event::Restore { record: Record::SettingChange { kind: KIND, scope: 0, number, value: None } },
        );
        if number == 1 {
            assert!(restored.is_empty());
        } else {
            assert_eq!(restored, Box::from([Request::RestoreRefused { kind: KIND, scope: 0, number: 2 }]));
        }
    }
    domain.config.settings = None;
    assert_eq!(
        run(
            &mut domain,
            Event::Restore { record: Record::SettingChange { kind: KIND, scope: 0, number: 1, value: None } }
        ),
        Box::from([Request::RestoreRefused { kind: KIND, scope: 0, number: 1 }])
    );
}

#[test]
fn invalid_defaults_refuse_start_with_their_identity() {
    for (scope, bytes, count) in [(1, &b"x"[..], 4), (0, &b"12345678901234567"[..], 4), (0, &b"x"[..], 0)] {
        let mut domain = connector();
        let kind = domain.config.settings.as_mut().expect("kind");
        kind.defaults = Box::from([(scope, 1, Box::from(bytes))]);
        kind.per_scope = count;
        assert_eq!(
            run(&mut domain, Event::SettingsDefaults),
            Box::from([Request::RestoreRefused { kind: KIND, scope, number: 1 }])
        );
    }
}

#[test]
fn restore_does_not_rejudge_saved_values_and_removals_mask_defaults() {
    let mut domain = connector();
    assert!(
        run(
            &mut domain,
            Event::Restore {
                record: Record::SettingChange {
                    kind: KIND,
                    scope: 0,
                    number: 1,
                    value: Some(Box::from(&b"!stored"[..]))
                }
            }
        )
        .is_empty()
    );
    assert_eq!(
        read(&mut domain, 1),
        Box::from([Request::SettingValue { token: TOKEN, value: Some(Box::from(&b"!stored"[..])) }])
    );
    drop(run(
        &mut domain,
        Event::Restore { record: Record::SettingChange { kind: KIND, scope: 0, number: 1, value: None } },
    ));
    assert_eq!(read(&mut domain, 1), Box::from([Request::SettingValue { token: TOKEN, value: None }]));
}

#[test]
fn project_release_erases_settings_and_proposals_in_that_scope_only() {
    let mut domain = connector();
    let kind = domain.config.settings.as_mut().expect("kind");
    kind.project = true;
    kind.defaults = Box::from([(1, 1, Box::from(&b"one"[..])), (2, 1, Box::from(&b"two"[..]))]);
    drop(run(&mut domain, Event::SaveSettingsDefaults { revision: None }));
    let project_item = |scope| SettingItem { kind: KIND, scope, number: 1, edit: SettingEdit::Remove };
    drop(kept(&mut domain, Box::from([project_item(1), project_item(2)])));
    drop(run(
        &mut domain,
        Event::KeepProposalSettings { proposal: 1, items: Box::from([project_item(1), project_item(2)]) },
    ));
    assert_eq!(
        run(&mut domain, Event::ReleaseProject { project: 1 }),
        Box::from([
            Request::Erase { key: RecordKey::SettingChange { kind: KIND, scope: 1, number: 1 } },
            Request::Erase { key: RecordKey::SettingDefault { kind: KIND, scope: 1, number: 1 } },
            Request::Erase { key: RecordKey::SettingProposal { proposal: 1, kind: KIND, scope: 1, number: 1 } },
            Request::ProjectReleased { project: 1 },
        ])
    );
    let saved = run(&mut domain, Event::SaveSettingsDefaults { revision: None });
    assert_eq!(saved.len(), 1);
    assert_eq!(
        saved[0],
        Request::Save {
            record: Record::SettingDefault { kind: KIND, scope: 2, number: 1, value: Box::from(&b"two"[..]) }
        }
    );
}

#[test]
fn stored_defaults_and_changes_share_scope_capacity_in_either_restore_order() {
    for default_first in [true, false] {
        let mut domain = connector();
        domain.config.settings.as_mut().expect("kind").per_scope = 1;
        let default = Record::SettingDefault { kind: KIND, scope: 0, number: 91, value: Box::from(&b"old"[..]) };
        let change = Record::SettingChange { kind: KIND, scope: 0, number: 91, value: None };
        let (first, second) = if default_first { (default, change) } else { (change, default) };
        assert!(run(&mut domain, Event::Restore { record: first }).is_empty());
        assert!(run(&mut domain, Event::Restore { record: second }).is_empty());
        for record in [
            Record::SettingDefault { kind: KIND, scope: 0, number: 92, value: Box::from(&b"other"[..]) },
            Record::SettingChange { kind: KIND, scope: 0, number: 92, value: None },
        ] {
            assert_eq!(
                run(&mut domain, Event::Restore { record }),
                Box::from([Request::RestoreRefused { kind: KIND, scope: 0, number: 92 }])
            );
        }
    }
}
