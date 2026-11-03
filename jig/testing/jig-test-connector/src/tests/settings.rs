//! The connector's settings rules (jig/docs/design/domain/connectors.md, section 17).
use super::run;
use crate::{
    Domain, Event, Record, RecordKey, Request, RestoredEdit, RestoredItem, SettingDefault, SettingEdit, SettingItem,
    SettingJudged, SettingsKind,
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

fn restored(number: u32, edit: RestoredEdit) -> RestoredItem {
    RestoredItem { kind: KIND, scope: 0, number, edit }
}

fn proposal_record(proposal: u64, number: u32, edit: SettingEdit) -> Request {
    Request::Save { record: Record::SettingProposal { proposal, kind: KIND, scope: 0, number, edit } }
}

#[test]
fn a_restores_items_are_kept_from_the_values_replaced_and_answered_with_sizes() {
    let mut domain = connector();
    drop(kept(&mut domain, Box::from([set(2, b"!abc")])));
    drop(kept(&mut domain, Box::from([set(2, b"!abc")])));
    let kind = domain.config.settings.as_mut().expect("kind");
    kind.charters = true;
    kind.defaults = Box::from([(0, 1, Box::from(&b"!def"[..]))]);
    assert_eq!(
        run(
            &mut domain,
            Event::RestoreSettings {
                token: TOKEN,
                proposal: 12,
                items: Box::from([
                    restored(2, RestoredEdit::Set { revision: 9 }),
                    restored(3, RestoredEdit::Remove),
                    restored(1, RestoredEdit::Reset),
                    restored(4, RestoredEdit::Reset),
                ]),
            }
        ),
        Box::from([
            proposal_record(12, 2, SettingEdit::Set(Box::from(&b"!abc"[..]))),
            proposal_record(12, 3, SettingEdit::Remove),
            proposal_record(12, 1, SettingEdit::Reset),
            proposal_record(12, 4, SettingEdit::Reset),
            Request::SettingsRestored {
                token: TOKEN,
                judged: Box::from([
                    SettingJudged::Accept { size: 4, charters: Box::from([u32::from_be_bytes(*b"!abc")]) },
                    accept(0),
                    SettingJudged::Accept { size: 4, charters: Box::from([u32::from_be_bytes(*b"!def")]) },
                    accept(0),
                ]),
            },
        ])
    );
    assert_eq!(read(&mut domain, 3), Box::from([Request::SettingValue { token: TOKEN, value: None }]));
    assert_eq!(domain.settings.replaced.get(&((KIND, 0, 2), 9)).expect("retained value").as_ref(), b"!abc");
    domain.config.settings.as_mut().expect("kind").defaults = Box::from([(0, 1, Box::from(&b"new"[..]))]);
    drop(run(
        &mut domain,
        Event::KeepSettings {
            revision: 10,
            proposal: Some(12),
            items: Box::from([set(2, b"ignored"), set(3, b"ignored"), set(1, b"ignored"), set(4, b"ignored")]),
        },
    ));
    assert_eq!(
        read(&mut domain, 2),
        Box::from([Request::SettingValue { token: TOKEN, value: Some(Box::from(&b"!abc"[..])) }])
    );
    assert_eq!(
        read(&mut domain, 1),
        Box::from([Request::SettingValue { token: TOKEN, value: Some(Box::from(&b"new"[..])) }])
    );
    assert!(run(&mut domain, Event::DropProposalSettings { proposal: 12 }).is_empty());
}

#[test]
fn a_restore_item_past_its_kinds_bounds_is_refused() {
    let mut domain = connector();
    drop(kept(&mut domain, Box::from([set(2, b"1234567890123456")])));
    drop(kept(&mut domain, Box::from([set(2, b"current")])));
    domain.config.settings.as_mut().expect("kind").value_bytes = 15;
    let mut wrong_kind = restored(1, RestoredEdit::Remove);
    wrong_kind.kind += 1;
    let mut wrong_scope = restored(1, RestoredEdit::Reset);
    wrong_scope.scope = 1;
    let refused = SettingJudged::Refuse { why: Box::from(&b"setting outside kind bounds"[..]) };
    assert_eq!(
        run(
            &mut domain,
            Event::RestoreSettings {
                token: TOKEN,
                proposal: 12,
                items: Box::from([
                    restored(2, RestoredEdit::Set { revision: 9 }),
                    wrong_kind,
                    wrong_scope,
                    restored(2, RestoredEdit::Set { revision: 8 })
                ]),
            }
        ),
        Box::from([Request::SettingsRestored {
            token: TOKEN,
            judged: Box::from([
                refused.clone(),
                refused.clone(),
                refused,
                SettingJudged::Refuse { why: Box::from(&b"no value kept"[..]) },
            ])
        }])
    );
    assert!(run(&mut domain, Event::DropProposalSettings { proposal: 12 }).is_empty());
    domain.config.settings.as_mut().expect("kind").value_bytes = 16;
    let requests = run(
        &mut domain,
        Event::RestoreSettings {
            token: TOKEN,
            proposal: 12,
            items: Box::from([restored(2, RestoredEdit::Set { revision: 9 })]),
        },
    );
    assert_eq!(
        requests,
        Box::from([
            proposal_record(12, 2, SettingEdit::Set(Box::from(&b"1234567890123456"[..]))),
            Request::SettingsRestored { token: TOKEN, judged: Box::from([accept(16)]) },
        ])
    );
}

#[test]
fn restored_items_obey_the_proposals_scope_capacity() {
    let mut domain = connector();
    domain.config.settings.as_mut().expect("kind").per_scope = 1;
    assert_eq!(
        run(
            &mut domain,
            Event::RestoreSettings {
                token: TOKEN,
                proposal: 12,
                items: Box::from([restored(1, RestoredEdit::Reset), restored(2, RestoredEdit::Remove)]),
            }
        ),
        Box::from([
            proposal_record(12, 1, SettingEdit::Reset),
            Request::SettingsRestored {
                token: TOKEN,
                judged: Box::from([
                    accept(7),
                    SettingJudged::Refuse { why: Box::from(&b"setting outside kind bounds"[..]) },
                ])
            },
        ])
    );
    assert_eq!(
        run(&mut domain, Event::DropProposalSettings { proposal: 12 }),
        Box::from([Request::Erase {
            key: RecordKey::SettingProposal { proposal: 12, kind: KIND, scope: 0, number: 1 }
        },])
    );
}

#[test]
fn forgetting_erases_the_values_replaced_by_older_revisions_only() {
    let mut domain = connector();
    for (revision, items) in [
        (2, Box::from([set(1, b"a"), set(2, b"a")])),
        (3, Box::from([set(1, b"b"), set(2, b"b")])),
        (4, Box::from([set(1, b"c"), set(2, b"c")])),
        (u64::MAX, Box::from([set(1, b"d"), set(2, b"d")])),
    ] {
        drop(run(&mut domain, Event::KeepSettings { revision, proposal: None, items }));
    }
    drop(run(&mut domain, Event::KeepProposalSettings { proposal: 12, items: Box::from([set(1, b"proposal")]) }));
    assert!(run(&mut domain, Event::ForgetReplaced { before: 0 }).is_empty());
    assert_eq!(
        run(&mut domain, Event::ForgetReplaced { before: 4 }),
        Box::from([
            Request::Erase { key: RecordKey::SettingReplaced { kind: KIND, scope: 0, number: 1, revision: 3 } },
            Request::Erase { key: RecordKey::SettingReplaced { kind: KIND, scope: 0, number: 2, revision: 3 } },
        ])
    );
    assert_eq!(domain.settings.replaced.len(), 4);
    for number in [1, 2] {
        assert!(!domain.settings.replaced.contains_key(&((KIND, 0, number), 3)));
        for revision in [4, u64::MAX] {
            assert!(domain.settings.replaced.contains_key(&((KIND, 0, number), revision)));
        }
        assert_eq!(
            read(&mut domain, number),
            Box::from([Request::SettingValue { token: TOKEN, value: Some(Box::from(&b"d"[..])) }])
        );
    }
    assert!(run(&mut domain, Event::ForgetReplaced { before: 4 }).is_empty());
    assert_eq!(run(&mut domain, Event::ForgetReplaced { before: u64::MAX }).len(), 2);
    assert_eq!(domain.settings.replaced.len(), 2);
    assert_eq!(
        run(&mut domain, Event::DropProposalSettings { proposal: 12 }),
        Box::from([Request::Erase {
            key: RecordKey::SettingProposal { proposal: 12, kind: KIND, scope: 0, number: 1 }
        },])
    );
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
        Box::from([
            Request::Save {
                record: Record::SettingReplaced {
                    kind: KIND,
                    scope: 0,
                    number: 1,
                    revision: 9,
                    value: Box::from(&b"changed"[..])
                }
            },
            Request::Save { record: Record::SettingChange { kind: KIND, scope: 0, number: 1, value: None } },
        ])
    );
    assert_eq!(read(&mut domain, 1), Box::from([Request::SettingValue { token: TOKEN, value: None }]));
    drop(kept(&mut domain, Box::from([set(1, b"")])));
    assert_eq!(read(&mut domain, 1), Box::from([Request::SettingValue { token: TOKEN, value: Some(Box::from([])) }]));
    assert_eq!(
        kept(&mut domain, Box::from([item(1, SettingEdit::Reset)])),
        Box::from([
            Request::Save {
                record: Record::SettingReplaced { kind: KIND, scope: 0, number: 1, revision: 9, value: Box::from([]) }
            },
            Request::Erase { key: RecordKey::SettingChange { kind: KIND, scope: 0, number: 1 } },
        ])
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
            Record::SettingReplaced { kind, scope, number: 1, revision: 3, value: value.clone() },
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
fn a_kept_change_keeps_the_value_it_replaced_by_its_revision() {
    let mut domain = connector();
    assert_eq!(
        run(&mut domain, Event::KeepSettings { revision: 2, proposal: None, items: Box::from([set(1, b"a")]) }),
        Box::from([Request::Save {
            record: Record::SettingChange { kind: KIND, scope: 0, number: 1, value: Some(Box::from(&b"a"[..])) },
        }])
    );
    assert!(domain.settings.replaced.is_empty());
    assert_eq!(
        run(&mut domain, Event::KeepSettings { revision: 3, proposal: None, items: Box::from([set(1, b"b")]) }),
        Box::from([
            Request::Save {
                record: Record::SettingReplaced {
                    kind: KIND,
                    scope: 0,
                    number: 1,
                    revision: 3,
                    value: Box::from(&b"a"[..])
                },
            },
            Request::Save {
                record: Record::SettingChange { kind: KIND, scope: 0, number: 1, value: Some(Box::from(&b"b"[..])) },
            },
        ])
    );
    assert_eq!(domain.settings.replaced.len(), 1);
    assert_eq!(domain.settings.replaced.get(&((KIND, 0, 1), 3)).expect("revision 3 replaced a change").as_ref(), b"a");
}

#[test]
fn a_change_over_a_default_or_a_removal_keeps_nothing_replaced() {
    let mut domain = connector();
    let items = [set(1, b"a"), item(2, SettingEdit::Remove), set(2, b"b"), set(3, b"c")];
    for (index, item) in items.into_iter().enumerate() {
        let revision = u64::try_from(index).expect("four items") + 2;
        let value = match &item.edit {
            SettingEdit::Set(value) => Some(value.clone()),
            SettingEdit::Remove => None,
            SettingEdit::Reset => unreachable!("these changes set or remove"),
        };
        let number = item.number;
        let saved = run(&mut domain, Event::KeepSettings { revision, proposal: None, items: Box::from([item]) });
        assert_eq!(
            saved,
            Box::from([Request::Save { record: Record::SettingChange { kind: KIND, scope: 0, number, value } }])
        );
        assert!(domain.settings.replaced.is_empty());
    }
    let saved = run(&mut domain, Event::SaveSettingsDefaults { revision: Some(6) });
    assert_eq!(
        saved,
        Box::from([Request::Save {
            record: Record::SettingDefault { kind: KIND, scope: 0, number: 1, value: Box::from(&b"default"[..]) }
        }])
    );
    assert!(domain.settings.replaced.is_empty());
}

#[test]
fn replaced_values_are_restored_at_a_restart() {
    let mut domain = connector();
    let mut records = alloc::collections::BTreeMap::new();
    for (revision, bytes) in [(2, &b"a"[..]), (3, &b"b"[..]), (u64::MAX, &b"c"[..])] {
        let saved =
            run(&mut domain, Event::KeepSettings { revision, proposal: None, items: Box::from([set(1, bytes)]) });
        for request in saved {
            let Request::Save { record } = request else { panic!("kept changes save records") };
            records.insert(record.key(), record);
        }
    }
    let mut restarted = connector();
    for record in records.into_values() {
        assert!(run(&mut restarted, Event::Restore { record }).is_empty());
    }
    assert_eq!(restarted.settings.replaced, domain.settings.replaced);
    assert_eq!(restarted.settings.replaced.len(), 2);
    assert_eq!(read(&mut restarted, 1), read(&mut domain, 1));
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

#[test]
fn replaced_values_have_their_own_scope_capacity_for_each_revision() {
    let mut domain = connector();
    domain.config.settings.as_mut().expect("kind").per_scope = 1;
    drop(kept(&mut domain, Box::from([set(1, b"current")])));
    for revision in [3, 4] {
        let record =
            Record::SettingReplaced { kind: KIND, scope: 0, number: 2, revision, value: Box::from(&b"old"[..]) };
        assert!(run(&mut domain, Event::Restore { record: record.clone() }).is_empty());
        assert!(
            run(&mut domain, Event::Restore { record }).is_empty(),
            "restoring the same identity uses no extra slot"
        );
        assert_eq!(
            run(
                &mut domain,
                Event::Restore {
                    record: Record::SettingReplaced {
                        kind: KIND,
                        scope: 0,
                        number: 3,
                        revision,
                        value: Box::from(&b"other"[..]),
                    }
                }
            ),
            Box::from([Request::RestoreRefused { kind: KIND, scope: 0, number: 3 }]),
        );
    }
    assert_eq!(domain.settings.replaced.len(), 2);
}
