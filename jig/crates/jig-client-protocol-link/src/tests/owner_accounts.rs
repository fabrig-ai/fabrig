//! Complete accounts receiving admission and field preservation.
use super::{c, fixtures, w};
use crate::{Limits, Undecodable, Unusable, translate::watches};
use skein_lib::{Duration, List, Wall, Writer, length::len_u64};
fn list<T, const N: usize>(items: [T; N]) -> List<T> {
    List::from_boxed(Box::new(items))
}
fn source_lines() -> List<c::AccountSource> {
    list(source_rows())
}
fn source_rows() -> [c::AccountSource; 2] {
    [
        c::AccountSource {
            name: None,
            place: 2,
            number: 11,
            kind: c::AccountKind::SignIn,
            state: c::AccountState::Granted,
            usable: true,
            expiring: false,
            attention: true,
            spent_until: Some(Wall::from_nanos(101)),
        },
        c::AccountSource {
            name: None,
            place: 5,
            number: 12,
            kind: c::AccountKind::Key,
            state: c::AccountState::Later { until: None },
            usable: false,
            expiring: true,
            attention: false,
            spent_until: None,
        },
    ]
}
fn document(end: Option<c::AccountEnd>) -> c::Accounts {
    c::Accounts {
        readiness: List::with_capacity(0),
        readiness_cut: false,
        program_hosts: List::with_capacity(0),
        accounts: list([
            c::AccountLine {
                list: None,
                facts: fact_rows(),
                provider_name: None,
                manage_usage: None,
                number: 7,
                place: 3,
                uses: Some(list([c::AccountUse::Party(23), c::AccountUse::Project(29)])),
                plan: true,
                provider: Some(3),
                display: c::DocumentText {
                    bytes: c::Text::new(Box::from(b"Personal".as_slice()), 32).unwrap(),
                    cut: true,
                },
                holder: c::AccountHolder::Party(23),
                sources: source_lines(),
                cut: true,
            },
            c::AccountLine {
                list: None,
                facts: List::with_capacity(0),
                provider_name: None,
                manage_usage: None,
                number: 9,
                place: 4,
                uses: None,
                plan: false,
                provider: None,
                display: c::DocumentText { bytes: c::Text::new(Box::from([]), 32).unwrap(), cut: false },
                holder: c::AccountHolder::Deployment,
                sources: List::with_capacity(0),
                cut: false,
            },
        ]),
        signing: list([c::AccountSignIn { source: 19, provider: 3, holder: c::AccountHolder::Party(23) }]),
        lendable: false,
        lending: None,
        lent_end: None,
        last_end: match end {
            Some(end) => Some(c::AccountLastEnd { source: 11, end }),
            None => None,
        },
        usable: true,
        settled: false,
        cut: true,
    }
}
fn ends() -> Box<[c::AccountEnd]> {
    Box::new([
        c::AccountEnd::Made { account: 7, declined: false },
        c::AccountEnd::Renewed { account: 7, declined: true },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::NotLimited },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::Uses },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::NotKept { why: c::AccountKeyFailure::NotStored } },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::Limit { bound: c::AccountBound::HolderAccounts } },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::Held { provider: 3 } },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::Provider { provider: 3 } },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::Subject },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::Unknown },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::Numbers },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::Kind { kind: c::AccountKind::Program } },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::Closing },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::NotStarted },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::NotSignedIn },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::NotOurs },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::Cancelled },
        c::AccountEnd::Refused { refusal: c::AccountRefusal::Moving },
        c::AccountEnd::Failed { failure: c::AccountFailure::Later { after: Some(Duration::from_nanos(103)) } },
        c::AccountEnd::Failed { failure: c::AccountFailure::Later { after: None } },
        c::AccountEnd::Failed { failure: c::AccountFailure::SignInAgain },
        c::AccountEnd::Failed { failure: c::AccountFailure::MissingScope { scope: 31 } },
        c::AccountEnd::Failed { failure: c::AccountFailure::Misconfigured },
        c::AccountEnd::Failed {
            failure: c::AccountFailure::Unloaded { at: c::AccountPlace::File, why: c::AccountUnread::Stalled },
        },
        c::AccountEnd::Failed { failure: c::AccountFailure::Cancelled },
    ])
}
fn bytes(end: Option<w::AccountEnd>) -> Box<[u8]> {
    delivery(jig_client_test_records::accounts_record(end))
}
/// A list's pieces written `times` over, one after another.
fn repeated(pieces: &[u8], times: usize) -> Box<[u8]> {
    let mut out = Writer::new(pieces.len().checked_mul(times).unwrap());
    for _ in 0..times {
        out.put(pieces).unwrap();
    }
    out.finish()
}
fn line_piece(line: &w::AccountLine) -> Box<[u8]> {
    let mut out = Writer::new(usize::try_from(line.measure()).unwrap());
    line.encode(&mut out).unwrap();
    let bytes = out.finish();
    let mut piece = Writer::new(bytes.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut piece, &bytes).unwrap();
    piece.finish()
}
/// The bytes `document(None)` holds once decoded: its record, its two accounts, the first's two uses,
/// display and two sources, and its one sign-in.
fn held() -> u64 {
    let parts = [
        size_of::<c::Accounts>(),
        size_of::<c::AccountLine>(),
        size_of::<c::AccountLine>(),
        size_of::<c::AccountUse>(),
        size_of::<c::AccountUse>(),
        b"Personal".len(),
        size_of::<c::AccountSource>(),
        size_of::<c::AccountSource>(),
        size_of::<c::AccountSignIn>(),
    ];
    let mut bytes = 0_usize;
    for part in parts {
        bytes = bytes.checked_add(part).unwrap();
    }
    for fact in &fact_rows() {
        bytes = bytes.checked_add(size_of::<c::AccountFact>()).unwrap().checked_add(fact.label.len()).unwrap();
        if let c::AccountFactValue::Text(text) = &fact.value {
            bytes = bytes.checked_add(text.len()).unwrap();
        }
    }
    u64::try_from(bytes).unwrap()
}
fn delivery(record: w::Accounts) -> Box<[u8]> {
    let value = w::DeliveryValue::Accounts(record);
    let mut out = Writer::new(usize::try_from(value.measure()).unwrap());
    value.encode(&mut out).unwrap();
    out.finish()
}
#[test]
fn accounts_delivery_preserves_every_field_and_terminal_outcome() {
    let limits = fixtures::limits();
    assert_eq!(watches::subject(&c::Subject::Accounts, limits.wire()), Ok(w::Subject::Accounts));
    for (end, expected) in jig_client_test_records::account_end_records().into_iter().zip(ends()) {
        assert_eq!(
            watches::delivery(&bytes(Some(end)), &limits),
            Ok(c::Delivery::Accounts(Box::new(document(Some(expected)))))
        );
    }
    assert_eq!(watches::delivery(&bytes(None), &limits), Ok(c::Delivery::Accounts(Box::new(document(None)))));
}
#[test]
fn an_accounts_document_filling_its_decoded_room_is_admitted() {
    // The smallest document limit whose decoded room holds the fixture.
    let limits = fixtures::limits_for_document(held().div_ceil(2));
    assert_eq!(watches::delivery(&bytes(None), &limits), Ok(c::Delivery::Accounts(Box::new(document(None)))));
}
#[test]
fn an_accounts_document_past_its_decoded_room_is_refused() {
    let limits = fixtures::limits_for_document(held().div_ceil(2).checked_sub(1).unwrap());
    assert_eq!(watches::delivery(&bytes(None), &limits), Err(Undecodable::TooLarge));
}
#[test]
fn accounts_delivery_one_byte_over_the_receiving_limit_is_refused() {
    let bytes = bytes(None);
    let limits = fixtures::limits_for_document(len_u64(bytes.len()).checked_sub(1).unwrap());
    assert_eq!(watches::delivery(&bytes, &limits), Err(Undecodable::TooLarge));
}

#[test]
fn account_uses_keep_their_constant_bound_with_fewer_receiving_rows() {
    // The schema mirrors jig_domain_accounts::USES; receiving rows remain two.
    const USES: usize = 16;
    let limits = fixtures::limits();
    let mut line = jig_client_test_records::account_line_record().into_parts();
    line.uses = Some(list([(); USES].map(|()| {
        w::AccountUse::Party(w::AccountUseParty::new(limits.wire(), w::AccountUsePartyParts { party: 23 }).unwrap())
    })));
    let line = w::AccountLine::new(limits.wire(), line).unwrap();
    let mut record = jig_client_test_records::accounts_record(None).into_parts();
    record.accounts = line_piece(&line);
    let record = w::Accounts::new(limits.wire(), record).unwrap();
    let mut expected = document(None);
    let mut line = expected.accounts.get(0).unwrap().clone();
    line.uses = Some(list([c::AccountUse::Party(23); USES]));
    expected.accounts = list([line]);
    assert_eq!(watches::delivery(&delivery(record), &limits), Ok(c::Delivery::Accounts(Box::new(expected))));
}
#[test]
fn account_lists_longer_than_the_receiving_rows_are_kept() {
    // Receiving rows are two; the account counts and document room admit these whole lists.
    let limits = fixtures::limits();
    let mut line = jig_client_test_records::account_line_record().into_parts();
    line.sources = repeated(&line.sources, 2);
    let line = w::AccountLine::new(limits.wire(), line).unwrap();
    let mut record = jig_client_test_records::accounts_record(None).into_parts();
    record.accounts = repeated(&line_piece(&line), 3);
    record.signing = repeated(&record.signing, 3);
    let record = w::Accounts::new(limits.wire(), record).unwrap();
    let mut expected = document(None);
    let mut line = expected.accounts.get(0).unwrap().clone();
    let [granted, later] = source_rows();
    line.sources = list([granted.clone(), later.clone(), granted, later]);
    expected.accounts = list([line.clone(), line.clone(), line]);
    expected.signing =
        list([(); 3].map(|()| c::AccountSignIn { source: 19, provider: 3, holder: c::AccountHolder::Party(23) }));
    assert_eq!(watches::delivery(&delivery(record), &limits), Ok(c::Delivery::Accounts(Box::new(expected))));
}

fn configured_line() -> c::AccountLine {
    let limits = fixtures::limits();
    let mut line = jig_client_test_records::account_line_record().into_parts();
    line.provider_name = Some(
        w::DocumentText::new(
            limits.wire(),
            w::DocumentTextParts { bytes: Box::from(b"Arbitrary provider".as_slice()), cut: true },
        )
        .unwrap(),
    );
    line.manage_usage = Some(Box::from(b"https://usage.example/".as_slice()));
    let line = w::AccountLine::new(limits.wire(), line).unwrap();
    let encoded = {
        let mut writer = Writer::new(usize::try_from(line.measure()).unwrap());
        line.encode(&mut writer).unwrap();
        writer.finish()
    };
    let mut record = jig_client_test_records::accounts_record(None).into_parts();
    record.accounts = {
        let mut writer = Writer::new(encoded.len().checked_add(4).unwrap());
        skein_codec::piece::write(&mut writer, &encoded).unwrap();
        writer.finish()
    };
    let record = w::Accounts::new(limits.wire(), record).unwrap();
    let c::Delivery::Accounts(value) = watches::delivery(&delivery(record), &limits).unwrap() else {
        panic!("accounts delivery")
    };
    value.accounts.get(0).unwrap().clone()
}
#[test]
fn configured_provider_name_reaches_accounts_client() {
    assert_eq!(configured_line().provider_name.unwrap().bytes(), b"Arbitrary provider");
}
#[test]
fn provider_name_cut_reaches_accounts_client() {
    assert!(configured_line().provider_name.unwrap().cut);
}
#[test]
fn configured_manage_address_reaches_accounts_client() {
    assert_eq!(configured_line().manage_usage.as_deref(), Some(b"https://usage.example/".as_slice()));
}

#[test]
fn account_capacity_labels_decode_without_account_payloads() {
    let limits = fixtures::limits();
    for (wire, expected) in [
        (w::PersonCapacityAction::AccountSignIn, c::PersonCapacityAction::AccountSignIn),
        (w::PersonCapacityAction::AccountSignOut, c::PersonCapacityAction::AccountSignOut),
        (w::PersonCapacityAction::AccountChange, c::PersonCapacityAction::AccountChange),
    ] {
        let mut writer = Writer::new(usize::try_from(wire.measure()).unwrap());
        wire.encode(&mut writer).unwrap();
        let bytes = writer.finish();
        let decoded = w::PersonCapacityAction::decode(limits.wire(), &mut skein_lib::Reader::new(&bytes)).unwrap();
        assert_eq!(crate::translate::documents::person_capacity_action_fields(&decoded), expected);
    }
}

fn unsafe_manage_addresses() -> [&'static [u8]; 5] {
    [b"ftp://x", b"https://", b"https:///x", b"https://usage.example/a b", b"https://usage.example/\""]
}
#[test]
fn unsafe_manage_addresses_refuse_accounts_frames() {
    let limits = fixtures::limits();
    for address in unsafe_manage_addresses() {
        let mut line = jig_client_test_records::account_line_record().into_parts();
        line.manage_usage = Some(Box::from(address));
        let line = w::AccountLine::new(limits.wire(), line).unwrap();
        let mut record = jig_client_test_records::accounts_record(None).into_parts();
        record.accounts = line_piece(&line);
        let record = w::Accounts::new(limits.wire(), record).unwrap();
        assert_eq!(watches::delivery(&delivery(record), &limits), Err(Undecodable::Malformed));
    }
}
#[test]
fn unsafe_manage_addresses_refuse_deployment_frames() {
    let limits = fixtures::limits();
    for address in unsafe_manage_addresses() {
        let record = fixtures::with_account(Some(5));
        let bytes = skein_codec::piece::split(&mut skein_lib::Reader::new(record.accounts())).unwrap().unwrap();
        let mut account = w::Account::decode(limits.wire(), &mut skein_lib::Reader::new(&bytes)).unwrap().into_parts();
        account.manage_usage = Some(Box::from(address));
        let account = w::Account::new(limits.wire(), account).unwrap();
        let mut out = Writer::new(usize::try_from(account.measure()).unwrap());
        account.encode(&mut out).unwrap();
        let bytes = out.finish();
        let mut pieces = Writer::new(bytes.len().checked_add(4).unwrap());
        skein_codec::piece::write(&mut pieces, &bytes).unwrap();
        let mut record = record.into_parts();
        record.accounts = pieces.finish();
        let value = w::DeliveryValue::Deployment(w::Deployment::new(limits.wire(), record).unwrap());
        let mut out = Writer::new(usize::try_from(value.measure()).unwrap());
        value.encode(&mut out).unwrap();
        assert_eq!(watches::delivery(&out.finish(), &limits), Err(Undecodable::Malformed));
    }
}

#[test]
fn a_refused_worker_position_survives_wire_receipt() {
    let limits = fixtures::limits();
    let mut record = fixtures::deployment_record().into_parts();
    let mut expected = fixtures::deployment();
    let position = u32::MAX;
    record.hello_refusals = list([w::HelloRefusalLine::new(
        limits.wire(),
        w::HelloRefusalLineParts {
            channel: 9,
            at: 0,
            slots: 1,
            stop_bound: Duration::ZERO,
            why: w::HelloRefusal::Position(
                w::HelloRefusalPosition::new(limits.wire(), w::HelloRefusalPositionParts { worker: position })
                    .expect("position record"),
            ),
        },
    )
    .expect("hello refusal")]);
    expected.hello_refusals = list([c::HelloRefusalLine {
        channel: 9,
        at: Wall::EPOCH,
        slots: 1,
        stop_bound: Duration::ZERO,
        why: c::HelloRefusal::Position { worker: position },
    }]);
    let record = w::Deployment::new(limits.wire(), record).expect("deployment");
    assert_eq!(crate::translate::documents::deployment(record, &limits), Ok(expected));
}

#[test]
fn zero_account_receiving_counts_make_limits_unusable() {
    use jig_client_domain::AccountCounts;
    let bounds = fixtures::limits();
    for accounts in [
        AccountCounts { accounts: 0, ..bounds.client().accounts },
        AccountCounts { sources: 0, ..bounds.client().accounts },
        AccountCounts { sign_ins: 0, ..bounds.client().accounts },
    ] {
        let client = jig_client_domain::Limits { accounts, turn_text: bounds.client().turn_text, ..*bounds.client() };
        assert_eq!(Limits::new(client, bounds.exchanges()), Err(Unusable::Limits), "{accounts:?}");
    }
}

// The fixture has two accounts and two sources in its first account; repeat its sign-in to two too.
const ACCOUNT_COUNT: u32 = 2;
fn counted_accounts() -> Box<[u8]> {
    let limits = fixtures::limits();
    let mut record = jig_client_test_records::accounts_record(None).into_parts();
    record.signing = repeated(&record.signing, usize::try_from(ACCOUNT_COUNT).unwrap());
    delivery(w::Accounts::new(limits.wire(), record).unwrap())
}
fn count_limits() -> Limits {
    let bounds = fixtures::limits();
    let client = jig_client_domain::Limits {
        accounts: jig_client_domain::AccountCounts {
            accounts: ACCOUNT_COUNT,
            sources: ACCOUNT_COUNT,
            sign_ins: ACCOUNT_COUNT,
        },
        turn_text: bounds.client().turn_text,
        ..*bounds.client()
    };
    Limits::new(client, bounds.exchanges()).unwrap()
}
#[test]
fn account_lists_filling_their_receiving_counts_are_admitted() {
    let mut expected = document(None);
    let signing = expected.signing.get(0).unwrap().clone();
    expected.signing = list([signing.clone(), signing]);
    assert_eq!(watches::delivery(&counted_accounts(), &count_limits()), Ok(c::Delivery::Accounts(Box::new(expected))));
}
#[test]
fn account_lists_one_past_their_receiving_counts_are_refused() {
    let bounds = count_limits();
    let bytes = counted_accounts();
    for accounts in [
        jig_client_domain::AccountCounts { accounts: ACCOUNT_COUNT - 1, ..bounds.client().accounts },
        jig_client_domain::AccountCounts { sources: ACCOUNT_COUNT - 1, ..bounds.client().accounts },
        jig_client_domain::AccountCounts { sign_ins: ACCOUNT_COUNT - 1, ..bounds.client().accounts },
    ] {
        let client = jig_client_domain::Limits { accounts, turn_text: bounds.client().turn_text, ..*bounds.client() };
        let limits = Limits::new(client, bounds.exchanges()).unwrap();
        assert_eq!(watches::delivery(&bytes, &limits), Err(Undecodable::TooLarge), "{accounts:?}");
    }
}
// One more account than receiving rows, within the configured account count.
const DEPLOYMENT_ACCOUNTS: u32 = 3;
fn counted_deployment(count: u32) -> w::Deployment {
    let limits = fixtures::limits();
    let mut record = fixtures::with_account(Some(5)).into_parts();
    record.accounts = repeated(&record.accounts, usize::try_from(count).unwrap());
    w::Deployment::new(limits.wire(), record).unwrap()
}
fn expected_deployment(count: u32) -> c::Deployment {
    let mut expected = fixtures::account_expected(Some(5));
    let account = expected.accounts.get(0).unwrap().clone();
    expected.accounts = List::with_capacity(count);
    for _ in 0..count {
        expected.accounts.push(account.clone()).unwrap();
    }
    expected
}
fn deployment_count_limits() -> Limits {
    let bounds = fixtures::limits();
    let mut client = *bounds.client();
    client.accounts.accounts = DEPLOYMENT_ACCOUNTS;
    Limits::new(client, bounds.exchanges()).unwrap()
}
#[test]
fn deployment_accounts_filling_the_receiving_count_are_admitted() {
    let limits = deployment_count_limits();
    assert_eq!(
        crate::translate::documents::deployment(counted_deployment(DEPLOYMENT_ACCOUNTS), &limits),
        Ok(expected_deployment(DEPLOYMENT_ACCOUNTS))
    );
}
#[test]
fn deployment_accounts_one_past_the_receiving_count_are_refused() {
    let limits = deployment_count_limits();
    assert_eq!(
        crate::translate::documents::deployment(counted_deployment(DEPLOYMENT_ACCOUNTS + 1), &limits),
        Err(Undecodable::TooLarge)
    );
}
// The deployment reserves its rows-bounded host list; these accounts have no sources, uses or
// display, and each reserves two rows-bounded chip lists.
fn deployment_held() -> u64 {
    let chips = List::<c::Chip>::price(fixtures::limits().client().rows).unwrap();
    u64::try_from(size_of::<c::Deployment>())
        .unwrap()
        .checked_add(List::<c::Host>::price(fixtures::limits().client().rows).unwrap())
        .unwrap()
        .checked_add(List::<c::Account>::price(DEPLOYMENT_ACCOUNTS).unwrap())
        .unwrap()
        .checked_add(u64::from(DEPLOYMENT_ACCOUNTS).checked_mul(2).unwrap().checked_mul(chips).unwrap())
        .unwrap()
}
#[test]
fn a_deployment_filling_its_decoded_room_is_admitted() {
    // The smallest document limit whose decoded room holds the fixture.
    let limits = fixtures::limits_for_document(deployment_held().div_ceil(2));
    assert_eq!(
        crate::translate::documents::deployment(counted_deployment(DEPLOYMENT_ACCOUNTS), &limits),
        Ok(expected_deployment(DEPLOYMENT_ACCOUNTS))
    );
}
#[test]
fn a_deployment_past_its_decoded_room_is_refused() {
    let limits = fixtures::limits_for_document(deployment_held().div_ceil(2).checked_sub(1).unwrap());
    assert_eq!(
        crate::translate::documents::deployment(counted_deployment(DEPLOYMENT_ACCOUNTS), &limits),
        Err(Undecodable::TooLarge)
    );
}

fn fact_rows() -> List<c::AccountFact> {
    let mut facts = List::with_capacity(24);
    for _ in 0_u32..4 {
        for value in [
            c::AccountFactValue::Text(Box::from(b"you@example.com".as_slice())),
            c::AccountFactValue::Count(123),
            c::AccountFactValue::Instant(Wall::from_nanos(127)),
            c::AccountFactValue::Span(Duration::from_nanos(131)),
            c::AccountFactValue::Share(10_000),
            c::AccountFactValue::Window(c::AccountWindow {
                length: Some(Duration::from_nanos(137)),
                used: Some(10_000),
                resets: Some(Wall::from_nanos(139)),
                state: 2,
            }),
        ] {
            facts
                .push(c::AccountFact {
                    kind: 1,
                    label: Box::from(b"label".as_slice()),
                    value,
                    seen: Wall::from_nanos(149),
                    source: Some(11),
                    origin: 6,
                })
                .unwrap();
        }
    }
    facts
}

fn wire_deployment_account(facts: List<w::AccountFact>) -> w::Account {
    let limits = fixtures::limits();
    let deployment = fixtures::with_account(None);
    let bytes = skein_codec::piece::counted(&mut skein_lib::Reader::new(deployment.accounts()));
    let mut parts = w::Account::decode(limits.receiving(), &mut skein_lib::Reader::new(&bytes)).unwrap().into_parts();
    parts.facts = facts;
    w::Account::new(limits.wire(), parts).unwrap()
}

#[test]
#[expect(clippy::disallowed_methods, reason = "unit-test tables use ordinary Rust result projections")]
fn account_fact_values_reach_both_client_rows() {
    let limits = fixtures::limits();
    let line = jig_client_test_records::account_line_record();
    let deployment = wire_deployment_account(jig_client_test_records::account_fact_records());
    // Each row pays for its fact records, all labels and every text value.
    let facts = fact_rows();
    let mut fact_bytes = List::<c::AccountFact>::price(facts.len()).unwrap();
    for fact in &facts {
        fact_bytes = fact_bytes.checked_add(len_u64(fact.label.len())).unwrap();
        if let c::AccountFactValue::Text(text) = &fact.value {
            fact_bytes = fact_bytes.checked_add(len_u64(text.len())).unwrap();
        }
    }
    let line_bytes = fact_bytes
        .checked_add(List::<c::AccountUse>::price(2).unwrap())
        .unwrap()
        .checked_add(len_u64(b"Personal".len()))
        .unwrap()
        .checked_add(List::<c::AccountSource>::price(2).unwrap())
        .unwrap();
    let account_bytes =
        fact_bytes.checked_add(List::<c::Chip>::price(limits.client().rows).unwrap().checked_mul(2).unwrap()).unwrap();
    for (line_room, account_room, expected) in [
        (line_bytes, account_bytes, Ok(facts)),
        (
            line_bytes.checked_sub(1).unwrap(),
            account_bytes.checked_sub(1).unwrap(),
            Err(w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Bound }),
        ),
    ] {
        let mut room = line_room;
        assert_eq!(crate::translate::documents::account_line(&line, &limits, &mut room).map(|row| row.facts), expected,);
        let mut room = account_room;
        assert_eq!(
            crate::translate::documents::account_fields(&deployment, &limits, &mut room).map(|row| row.facts),
            expected,
        );
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "unit-test tables use ordinary Rust result projections")]
fn account_fact_shares_keep_their_basis_point_bound() {
    let limits = fixtures::limits();
    for (basis, refusal) in [(10_000, None), (10_001, Some(skein_codec::Reason::Bound))] {
        for (value, path) in [
            (
                w::AccountFactValue::Share(
                    w::AccountFactShare::new(limits.wire(), w::AccountFactShareParts { basis }).unwrap(),
                ),
                w::Path::AccountFactShareBasis,
            ),
            (
                w::AccountFactValue::Window(
                    w::AccountWindow::new(
                        limits.wire(),
                        w::AccountWindowParts { length: None, used: Some(basis), resets: None, state: 1 },
                    )
                    .unwrap(),
                ),
                w::Path::AccountWindowUsed,
            ),
        ] {
            let fact = w::AccountFact::new(
                limits.wire(),
                w::AccountFactParts { kind: 7, label: Box::new([]), value, seen: 0, source: None, origin: 6 },
            )
            .unwrap();
            let mut parts = jig_client_test_records::account_line_record().into_parts();
            parts.facts = list([fact.clone()]);
            let line = w::AccountLine::new(limits.wire(), parts).unwrap();
            let deployment = wire_deployment_account(list([fact]));
            let mut room = u64::MAX;
            for result in [
                crate::translate::documents::account_line(&line, &limits, &mut room).map(|row| row.facts),
                crate::translate::documents::account_fields(&deployment, &limits, &mut room).map(|row| row.facts),
            ] {
                assert_eq!(result.map(|_| ()), refusal.map_or(Ok(()), |reason| Err(w::Problem { path, reason })));
            }
        }
    }
}

#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    reason = "unit-test tables use ordinary Rust containers and collection"
)]
fn email_reads_only_an_email_fact_with_text() {
    let mut rows = vec![(None, None)];
    // Six value shapes, once each, from the full 24-fact fixture.
    for fact in fact_rows().as_slice().iter().take(6) {
        let expected = if let c::AccountFactValue::Text(text) = &fact.value { Some(text.clone()) } else { None };
        rows.push((Some(fact.clone()), expected));
    }
    let mut named = fact_rows().get(0).unwrap().clone();
    named.kind = c::FACT_NAME;
    rows.push((Some(named), None));
    for (fact, expected) in rows {
        let facts = List::from_boxed(fact.into_iter().collect());
        let mut line = document(None).accounts.get(0).unwrap().clone();
        line.facts = facts;
        let mut account = fixtures::account_expected(None).accounts.get(0).unwrap().clone();
        account.facts = line.facts.clone();
        assert_eq!(line.email(), expected.as_deref());
        assert_eq!(account.email(), expected.as_deref());
    }
}
