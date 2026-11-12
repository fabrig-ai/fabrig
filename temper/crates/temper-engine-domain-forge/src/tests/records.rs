//! Kept forge records (temper/docs/design/protocol/forge/engine.md, 1; temper/docs/design/domain/forge/adoption.md, 1).
use super::*;
use crate::settings as setting;

pub(super) fn record(number: u16) -> ForgeRecord {
    ForgeRecord {
        person: None,
        git_program: None,
        number,
        kind: Kind::Forgejo,
        mode: Mode::Managed,
        state: ForgeState::Kept,
        place: Place {
            api: Address {
                host: b"forge.example".as_slice().into(),
                port: 443,
                tls: Some(b"forge.example".as_slice().into()),
            },
            git: b"https://forge.example".as_slice().into(),
            web: Some(b"https://forge.example".as_slice().into()),
        },
        read_only: false,
        bot: Bot { login: b"temper-bot".as_slice().into(), name: b"Temper".as_slice().into(), email: Box::new([]) },
        asked: false,
        first: None,
    }
}

#[test]
fn a_restored_record_is_listed_and_told_to_the_routing() {
    let mut d = setup_domain(&[], false);
    for number in [3, 1, 2] {
        outputs(&mut d, Event::Restore { record: Stored::Forge(record(number)) });
    }
    let rows = d.forges().rows;
    assert_eq!(rows.len(), 3);
    for (number, row) in (1..=3).zip(rows.iter()) {
        assert_eq!(row.number, number);
        assert_eq!(row.mode, Mode::Managed);
        assert!(row.admin);
    }
    let sent = outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    let mut next = 1;
    for request in sent {
        if let Request::ForgeKept(row) = request {
            assert_eq!(row, record(next));
            next = next.checked_add(1).expect("three fixture forges");
        }
    }
    assert_eq!(next, 4);
    let link = Link { user: 71, person: 29 };
    for (number, state, person, expected) in [
        (1, Some(ForgeState::Kept), Some(link), Some(link)),
        (2, Some(ForgeState::Kept), None, None),
        (3, Some(ForgeState::Ended { secrets_erased: false, box_removed: false }), Some(link), None),
        (4, None, None, None),
    ] {
        if let Some(state) = state {
            let row = ForgeRecord { state, person, ..record(number) };
            outputs(&mut d, Event::Restore { record: Stored::Forge(row) });
        }
        assert_eq!(d.link(number), expected, "forge {number}");
    }
}

fn make(d: &mut Domain, out: &mut Queue<Request>) -> Option<u16> {
    let row = record(0);
    forges::make(d, row.kind, row.mode, row.place, row.git_program, row.read_only, row.bot, row.asked, row.first, out)
}

#[test]
fn make_numbers_next_and_never_reuses_after_erase() {
    let mut d = setup_domain(&[], false);
    outputs(&mut d, Event::Restore { record: Stored::ForgeNumber { last: 8 } });
    let mut out = Queue::uncapped();
    assert_eq!(make(&mut d, &mut out), Some(9));
    assert_eq!(
        out.pop(),
        Some(Request::Save { record: Stored::ForgeNumber { last: 9 }, release: None, read_afresh: None })
    );
    let row = d.record(9).expect("made record").clone();
    assert_eq!(out.pop(), Some(Request::Save { record: Stored::Forge(row.clone()), release: None, read_afresh: None }));
    assert_eq!(out.pop(), Some(Request::ForgeKept(row)));
    forges::erase(&mut d, 9, &mut out);
    assert_eq!(out.pop(), Some(Request::Erase { key: Key::Forge(9) }));
    assert!(d.record(9).is_none());
    assert!(d.configured(9).is_none());
    assert_eq!(make(&mut d, &mut out), Some(10));
    outputs(&mut d, Event::Restore { record: Stored::ForgeNumber { last: u16::MAX } });
    assert_eq!(make(&mut d, &mut out), None);
}

#[test]
fn make_refuses_past_the_forges_bound() {
    let mut d = setup_domain(&[], false);
    let mut out = Queue::uncapped();
    for number in 1..=LIMITS.forges {
        assert_eq!(make(&mut d, &mut out), Some(u16::try_from(number).expect("small bound")));
    }
    let before = out.len();
    assert_eq!(make(&mut d, &mut out), None);
    assert_eq!(out.len(), before, "refusal spends no number and emits nothing");
    forges::erase(&mut d, 1, &mut out);
    assert_eq!(
        make(&mut d, &mut out),
        Some(u16::try_from(LIMITS.forges.checked_add(1).expect("small bound")).expect("small bound"))
    );
}

#[test]
fn restore_refuses_a_record_past_the_forges_bound() {
    let mut d = setup_domain(&[], false);
    for number in 1..=LIMITS.forges.checked_add(1).expect("small bound") {
        outputs(&mut d, Event::Restore { record: Stored::Forge(record(u16::try_from(number).expect("small bound"))) });
    }
    assert_eq!(
        d.restore_fit_failure(),
        Some(&RestoreFit {
            family: Some(RecordKind::Forge),
            set: RestoreSet::Rows(RecordKind::Forge),
            held: u64::from(LIMITS.forges),
            additional: 1,
            limit: u64::from(LIMITS.forges),
        })
    );
}

#[test]
fn an_ended_record_is_out_of_every_list_and_refuses_calls() {
    let mut d = setup_domain(&configured(), true);
    let mut row = record(1);
    row.state = ForgeState::Ended { secrets_erased: false, box_removed: false };
    outputs(&mut d, Event::Restore { record: Stored::Forge(row.clone()) });
    outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
    assert_eq!(d.record(1), Some(&row));
    assert!(d.forge_ended(1));
    assert_eq!(d.connection(1), Connection::NotConnected);
    assert_eq!(d.offer(1), Offer::NotOffered);
    assert_eq!(d.forges().rows.len(), 1);
    assert_eq!(d.forges().rows.first().expect("other forge").number, 2);
    assert_eq!(d.home(5), None);
    assert!(d.context_repositories(5, 2).expect("bounded contexts").is_empty());
    assert!(d.adopted_ids(1).is_empty());
    assert!(d.readable_repositories(5).is_empty());
    assert!(d.writable_repositories(5).is_empty());
    assert_eq!(d.repository(REPO), Some(&adopted()), "historical lookup remains");
    let request = Token::new(104);
    outputs(
        &mut d,
        Event::Application {
            role: None,
            carried: None,
            request: Token::new(4),
            token: request,
            retention: Duration::from_secs(60),
            ask: Ask::SetUp {
                forge: Making::Kept(1),
                first: SetUpFirst::Empty {
                    playground: false,
                    org: b"org".as_slice().into(),
                    repository: b"repo".as_slice().into(),
                },
            },
        },
    );
    assert_eq!(d.application_answer(request), Some(Answered::SetUp(SetUpAnswer::Refused(SetUpRefusal::NotOffered))));
    let sent = outputs(&mut d, Event::Connection { forge: 1, state: Connection::Connecting });
    assert!(sent.is_empty());
    assert_eq!(d.connection(1), Connection::NotConnected);
}

#[test]
fn a_number_with_no_record_is_ended() {
    let d = setup_domain(&[], false);
    assert!(d.forge_ended(97));
    assert_eq!(d.connection(97), Connection::NotConnected);
    assert_eq!(d.offer(97), Offer::NotOffered);
}

#[test]
fn a_record_overrides_the_document_forge_of_its_number() {
    let mut d = setup_domain(&configured(), true);
    let row = record(1);
    outputs(&mut d, Event::Restore { record: Stored::Forge(row.clone()) });
    assert_eq!(d.configured(1), Some(&forges::configured_of(&row)));
    assert!(!d.forge_ended(2), "document forge remains admitted");
    assert_eq!(d.forges().rows.len(), 2);
}

#[test]
fn a_real_forge_blocks_the_playground_until_its_record_is_erased() {
    let mut d = setup_domain(&[], false);
    let mut row = record(1);
    row.mode = Mode::Fake;
    outputs(&mut d, Event::Restore { record: Stored::Forge(row.clone()) });
    assert!(!d.real_forge_kept());
    row.mode = Mode::Owned;
    row.state = ForgeState::Ended { secrets_erased: false, box_removed: false };
    outputs(&mut d, Event::Restore { record: Stored::Forge(row) });
    assert!(d.real_forge_kept());
    forges::erase(&mut d, 1, &mut Queue::uncapped());
    assert!(!d.real_forge_kept());
}

#[test]
fn made_and_restored_forges_supply_their_checkout_credential() {
    for restored in [false, true] {
        let mut d = setup_domain(&[], false);
        let mut out = Queue::uncapped();
        if restored {
            outputs(&mut d, Event::Restore { record: Stored::Forge(record(1)) });
        } else {
            assert_eq!(make(&mut d, &mut out), Some(1));
        }
        outputs(&mut d, Event::Restore { record: Stored::Repository(adopted()) });
        let item = d
            .workspace_item(items::Seed {
                modify: false,
                task: 1,
                parent: None,
                repository: REPO,
                run_branch: b"temper/1/r1".as_slice().into(),
                saved_branch: None,
            })
            .expect("adopted checkout");
        assert_eq!(item.credential, Some(1), "restored={restored}");
        forges::erase(&mut d, 1, &mut out);
        assert_eq!(d.identities.get(&1), None, "erasure frees the credential slot");
    }
}

pub(super) fn setting_rule() -> setting::LandingRule {
    setting::LandingRule {
        pattern: GatePattern {
            segments: Box::new([Box::from(b"forge".as_slice())]),
            last: GateLast::Open(Box::new([])),
        },
        enforce: true,
        ci: true,
        up_to_date: false,
        gates: Box::new([setting::RuleGate { number: 2, blocking: true, freshness: JudgeFreshness::Exact }]),
        approvals: Box::new([setting::RuleApproval { role: 1, people: 2, freshness: JudgeFreshness::Clean }]),
    }
}

#[test]
fn setting_rows_count_scopes_slots_and_proposals_with_checked_arithmetic() {
    let limits = Limits { judge_projects: 2, landing_rules: 3, setting_proposals: 4, ..LIMITS };
    assert_eq!(setting::rows(&limits), Some(150)); // (9 rules + 2 charters + 4 project values) * (2 + 4 + 4).
    assert_eq!(
        setting::rows(&Limits {
            judge_projects: 0,
            decided_per_project: 64,
            findings_per_project: 256,
            landing_rules: 0,
            setting_proposals: 0,
            setting_history: 0,
            ..LIMITS
        }),
        Some(4)
    );
    for limits in [
        Limits { judge_projects: u32::MAX, ..LIMITS },
        Limits { landing_rules: u32::MAX, ..LIMITS },
        Limits { setting_proposals: u32::MAX, ..LIMITS },
        Limits { setting_history: u32::MAX, ..LIMITS },
        Limits { judge_projects: 0, landing_rules: (u32::MAX - 4) / 2, setting_proposals: 1, ..LIMITS },
    ] {
        assert_eq!(setting::rows(&limits), None);
        let Err(error) = Domain::new(
            &limits,
            4,
            skein_forge_client::Config { namespace: b"test".as_slice().into(), writers: Box::new([]) },
            &[],
            &[],
            false,
        ) else {
            panic!("overflowing setting rows must refuse construction");
        };
        assert_eq!(error, skein_forge::api::Error::TooLarge(skein_forge::api::Excess::unnamed()));
    }
}

#[test]
fn setting_bytes_count_rule_allocations_and_inline_values_hold_none() {
    use core::mem::size_of;
    let rule = setting_rule();
    let bytes = 5 + size_of::<Box<[u8]>>() + size_of::<setting::RuleGate>() + size_of::<setting::RuleApproval>();
    let key = setting::SettingKey { kind: 1, scope: 0, number: 1, slot: setting::SettingSlot::Default };
    for (edit, expected) in [
        (
            setting::Edit::Set(setting::SizedValue { value: setting::Value::Rule(rule), size: 42 }),
            u64::try_from(bytes).unwrap(),
        ),
        (setting::Edit::Set(setting::SizedValue { value: setting::Value::Charter(7), size: 4 }), 0),
        (setting::Edit::Remove, 0),
        (setting::Edit::Reset, 0),
    ] {
        let row = Stored::Setting(setting::SettingRow { key, edit });
        assert_eq!(stored_key(&row), Key::Setting(key));
        assert_eq!(stored_bytes(&row), Some(expected));
    }
}
