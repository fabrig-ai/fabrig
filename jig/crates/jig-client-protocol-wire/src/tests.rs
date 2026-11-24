//! Client wire refusals and round trips (protocol/client/wire.md, section 1).
mod approval;
mod fixtures;
mod link_sign_in;
mod task_refusal;
mod unread;
mod versions;
use crate::*;
use skein_lib::Reader;

/// A version is its schema's family line (protocol/client/versions.md, section 1).
#[test]
fn version_constants_match_their_schemas() {
    let client = &skein_codegen::values(CLIENT_SCHEMA).schema;
    assert_eq!(VERSION, u32::from(client.version), "jig-client-wire VERSION must equal its family version");
    let public = &skein_codegen::values(PUBLIC_SCHEMA).schema;
    assert_eq!(PUBLIC_VERSION, public.version, "jig-public-wire PUBLIC_VERSION must equal its family version");
}

/// A kept record holds the largest request (protocol/client/small-store.md, 1).
#[test]
fn kept_holds_the_widest_table_entry_and_round_trips_a_ceiling_request() {
    let widest = TableEntry::encoded_bound(&CEILINGS).expect("representable table entry");
    assert!(u64::from(CEILINGS.kept_record) >= widest, "Kept must hold TableEntry's widest encoding");
    let table = TableEntry {
        asker: u32::MAX,
        number: u64::MAX,
        key: [255; 16],
        scope: u64::MAX,
        first_sent: Some(u64::MAX),
        owner: Owner::Family(Family { number: u16::MAX }),
        version: u32::MAX,
        request: fixtures::bytes(CEILINGS.table_entry_request),
    };
    assert_eq!(u64::from(table.measure(&CEILINGS).expect("bounded fixture")), widest);
    let record = fixtures::encoded(&table);
    let kept = Kept { store_version: 1, record };
    let bytes = fixtures::encoded(&kept);
    let decoded = Kept::decode(&CEILINGS, &mut Reader::new(&bytes)).expect("kept round trip");
    assert_eq!(decoded, kept);
    assert_eq!(TableEntry::decode(&CEILINGS, &mut Reader::new(decoded.record.as_ref())), Ok(table));
}

fn a_keyed_request_past_its_bound_is_refused_naming_it() {
    let limits = fixtures::limits();
    let request = fixtures::bytes(limits.keyed_request.checked_add(1).expect("small bound"));
    let refused = (Keyed { key: [7; 16], owner: Owner::Jig, version: 1, request, value: None }).encode(&limits);
    assert_eq!(refused, Err(Problem { path: paths::KEYED_REQUEST, reason: skein_codec::Reason::Bound }));
}

fn an_owned_record_keeps_its_owners_version() {
    let limits = fixtures::limits();
    let owned = fixtures::owned();
    let bytes = fixtures::encoded(&owned);
    assert_eq!(Owned::decode(&limits, &mut Reader::new(&bytes)), Ok(owned));
}

fn a_refusal_with_its_facts_round_trips_unchanged() {
    let limits = fixtures::limits();
    let refused = fixtures::refused_moved();
    let bytes = fixtures::encoded(&refused);
    assert_eq!(Refused::decode(&limits, &mut Reader::new(&bytes)), Ok(refused));
}

fn each_answer_kind_round_trips_unchanged() {
    let limits = fixtures::limits();
    for answer in [
        KeyedAnswer::NotServed(fixtures::not_served()),
        KeyedAnswer::FinalAnswer(fixtures::owned()),
        KeyedAnswer::Busy,
        KeyedAnswer::AnswersFull(AnswersFull { held: u32::MAX, room_at: u64::MAX }),
        KeyedAnswer::NotReady,
        KeyedAnswer::SignedOut,
    ] {
        let bytes = fixtures::encoded(&answer);
        assert_eq!(KeyedAnswer::decode(&limits, &mut Reader::new(&bytes)), Ok(answer));
    }
}

#[test]
fn a_batch_round_trips_its_read_open_and_close() {
    let limits = fixtures::stream_limits();
    let close = WatchClose { stream: [7; 16], watch: 23 };
    let mut members = skein_lib::List::with_capacity(3);
    for member in
        [Member::Read(fixtures::stream_owned(&limits)), Member::Open(fixtures::open_tree()), Member::Close(close)]
    {
        members.push(member).expect("three members");
    }
    let batch = Batch { members };
    let bytes = fixtures::encoded(&batch);
    assert_eq!(Batch::decode(&limits, &mut Reader::new(&bytes)), Ok(batch));
}

#[test]
fn batch_replies_preserve_the_member_and_optional_answer() {
    let limits = fixtures::stream_limits();
    for reply in [
        None,
        Some(fixtures::encoded(&ReadAnswer::Value(fixtures::stream_owned(&limits)))),
        Some(fixtures::encoded(&OpenAnswer::Admitted)),
        Some(fixtures::encoded(&CloseAnswer::Closed)),
    ] {
        let reply = BatchReply { member: 2, reply };
        let bytes = fixtures::encoded(&reply);
        assert_eq!(BatchReply::decode(&limits, &mut Reader::new(&bytes)), Ok(reply));
    }
}

#[test]
fn a_batch_accepts_64_members_and_refuses_65() {
    let limits = fixtures::limits();
    let close = WatchClose { stream: [7; 16], watch: 23 };
    for count in [64, 65] {
        let members = fixtures::repeated(Member::Close(close.clone()), count);
        let bytes = fixtures::encoded_batch_members(&members);
        let decoded = Batch::decode(&limits, &mut Reader::new(&bytes));
        if count == 64 {
            let batch = Batch { members };
            assert_eq!(decoded, Ok(batch));
        } else {
            let expected = Err(Problem { path: paths::BATCH_MEMBERS, reason: skein_codec::Reason::Bound });
            assert_eq!(decoded, expected);
            assert_eq!((Batch { members }).encode(&limits).err(), expected.err());
        }
    }
}

#[test]
fn a_batch_reply_can_hold_the_largest_members_answer() {
    let largest = [
        ReadAnswer::encoded_bound(&CEILINGS).expect("read answer bound"),
        OpenAnswer::encoded_bound(&CEILINGS).expect("open answer bound"),
        CloseAnswer::encoded_bound(&CEILINGS).expect("close answer bound"),
    ]
    .into_iter()
    .max()
    .expect("three member answer kinds");
    assert!(u64::from(CEILINGS.batch_reply_reply_some) >= largest, "a batch must carry an answer at its ceiling");
}

fn a_bearer_token_past_its_bound_is_refused() {
    let limits = fixtures::limits();
    let token = fixtures::bytes(limits.signed_in_token_some.checked_add(1).expect("small bound"));
    assert_eq!(
        (SignedIn { token: Some(skein_lib::Secret::new(token)), party: 17, expires: 29 }).encode(&CEILINGS),
        Err(Problem { path: paths::SIGNED_IN_TOKEN, reason: skein_codec::Reason::Bound })
    );
}

fn a_cursor_past_its_bound_is_refused() {
    let limits = fixtures::limits();
    let value = fixtures::bytes(limits.cursor_value.checked_add(1).expect("small bound"));
    assert_eq!(
        (Cursor { value }).encode(&limits),
        Err(Problem { path: paths::CURSOR_VALUE, reason: skein_codec::Reason::Bound })
    );
}

fn each_read_none_round_trips_unchanged() {
    let limits = fixtures::limits();
    for none in [
        ReadNone::Unknown,
        ReadNone::Refused,
        ReadNone::Gone,
        ReadNone::Busy,
        ReadNone::SignedOut,
        ReadNone::Malformed,
        ReadNone::TooLinked(ReadTooLinked { joins: 4 }),
        ReadNone::TooLarge(TurnTooLarge { bytes: 8192 }),
    ] {
        let answer = ReadAnswer::None(fixtures::read_none(none));
        let bytes = fixtures::encoded(&answer);
        assert_eq!(ReadAnswer::decode(&limits, &mut Reader::new(&bytes)), Ok(answer));
    }
}

fn a_read_value_round_trips_with_its_owner_version() {
    let limits = fixtures::limits();
    let answer = ReadAnswer::Value(fixtures::owned());
    let bytes = fixtures::encoded(&answer);
    assert_eq!(ReadAnswer::decode(&limits, &mut Reader::new(&bytes)), Ok(answer));
}

#[test]
fn a_page_one_row_past_its_limit_is_refused() {
    let limits = fixtures::limits();
    let rows = fixtures::funding_rows(limits.spend_page_rows.checked_add(1).expect("small bound"));
    assert_eq!(
        (SpendPage { rows, before: None, after: None, cut: false }).encode(&limits),
        Err(Problem { path: paths::SPEND_PAGE_ROWS, reason: skein_codec::Reason::Bound })
    );
}

fn an_activity_line_carries_its_turn_bytes_unchanged() {
    let limits = fixtures::limits();
    let line = fixtures::activity_line(Box::from([0, 255, 1]));
    let bytes = fixtures::encoded(&line);
    assert_eq!(DecodedActivityLine::decode(&limits, &mut Reader::new(&bytes)), Ok(line));
}

#[test]
fn a_page_whose_rows_do_not_split_to_its_end_is_refused_as_malformed() {
    let limits = fixtures::limits_with_activity_bytes(4096);
    for rows in fixtures::malformed_activity_pieces() {
        let page = fixtures::activity_page(rows);
        let bytes = fixtures::encoded(&page);
        assert_eq!(
            fixtures::decode_activity_page(&limits, &bytes, 8),
            Err(Problem { path: paths::ACTIVITY_PAGE_ROWS, reason: skein_codec::Reason::Malformed })
        );
    }
}

#[test]
fn a_page_of_many_small_rows_and_one_of_a_few_large_rows_fit_one_bound() {
    let limits = fixtures::limits_with_activity_bytes(4096);
    for (count, width) in [(4, 16), (2, 512)] {
        let rows = fixtures::activity_rows(count, width);
        let bytes = fixtures::encoded_activity_page(&rows);
        assert_eq!(fixtures::decode_activity_page(&limits, &bytes, 8), Ok(rows));
    }
}

fn me_at_its_ceilings_round_trips_unchanged() {
    let limits = fixtures::document_limits();
    let frame = fixtures::frame_at_ceilings(&limits);
    let bytes = fixtures::encoded(&frame);
    assert_eq!(Me::decode(&limits, &mut Reader::new(&bytes)), Ok(frame));
}

#[test]
fn a_task_one_byte_past_its_text_bound_is_refused() {
    let mut limits = fixtures::document_limits();
    let task = fixtures::task(&limits);
    let bytes = fixtures::encoded(&task);
    limits.document_text_bytes -= 1;
    let expected = Err(Problem { path: paths::DOCUMENT_TEXT_BYTES, reason: skein_codec::Reason::Bound });
    assert_eq!(Task::decode(&limits, &mut Reader::new(&bytes)), expected);
    assert_eq!(task.encode(&limits).err(), expected.err());
}

fn a_run_state_without_writing_or_call_round_trips() {
    let limits = fixtures::document_limits();
    let run = fixtures::run_state();
    let bytes = fixtures::encoded(&run);
    assert_eq!(RunState::decode(&limits, &mut Reader::new(&bytes)), Ok(run));
}

fn a_chip_with_each_phase_round_trips_unchanged() {
    let limits = fixtures::document_limits();
    for phase in fixtures::chip_phases() {
        let rows = fixtures::chip_facts(&limits, phase);
        let chip = fixtures::chip(&rows);
        let bytes = fixtures::encoded(&chip);
        assert_eq!(fixtures::decode_chip_facts(&limits, &bytes), rows);
    }
}

fn a_hello_naming_two_owners_round_trips_unchanged() {
    let limits = fixtures::stream_limits();
    let hello = fixtures::hello();
    let bytes = fixtures::encoded(&hello);
    assert_eq!(Hello::decode(&limits, &mut Reader::new(&bytes)), Ok(hello));
}

fn a_stream_event_at_its_ceiling_round_trips_unchanged() {
    let limits = fixtures::stream_limits();
    for event in fixtures::stream_events(&limits) {
        let bytes = fixtures::encoded(&event);
        assert_eq!(StreamEvent::decode(&limits, &mut Reader::new(&bytes)), Ok(event));
    }
}

#[test]
fn an_open_for_a_tree_carries_its_window() {
    let limits = fixtures::stream_limits();
    let open = fixtures::open_tree();
    let bytes = fixtures::encoded(&open);
    let decoded = Open::decode(&limits, &mut Reader::new(&bytes)).expect("whole open exchange");
    assert_eq!(
        Subject::decode(&limits, &mut Reader::new(decoded.subject.record.as_ref())),
        Ok(fixtures::tree_subject())
    );
    assert_eq!(decoded, open);
}

fn each_watch_end_but_dropped_round_trips() {
    let limits = fixtures::stream_limits();
    for why in [
        WatchEnd::Closed,
        WatchEnd::Unknown,
        WatchEnd::NotAllowed,
        WatchEnd::Busy,
        WatchEnd::SubjectEnded,
        WatchEnd::SignedOut,
        WatchEnd::Stopping,
    ] {
        let ended = fixtures::ended(why);
        let bytes = fixtures::encoded(&ended);
        assert_eq!(WatchEnded::decode(&limits, &mut Reader::new(&bytes)), Ok(ended));
    }
}

#[test]
fn an_account_waiting_to_ask_again_carries_the_wall_it_asks_at() {
    let limits = fixtures::stream_limits();
    for until in [None, Some(0), Some(u64::MAX)] {
        let state = fixtures::waiting_account_state(until);
        let account = fixtures::account(state.clone());
        let bytes = fixtures::encoded(&account);
        let decoded = Account::decode(&limits, &mut Reader::new(&bytes)).expect("whole account document");
        assert_eq!((&decoded.state), &state);
        assert_eq!(decoded, account);
    }
}

#[test]
fn a_ended_goal_holding_keeps_its_opaque_segments() {
    let limits = fixtures::stream_limits();
    let holding = fixtures::ended_goal_holding();
    let bytes = fixtures::encoded(&holding);
    assert_eq!(GoalHolding::decode(&limits, &mut Reader::new(&bytes)), Ok(holding));
}

fn small_store_family() -> Family {
    Family { number: u16::MAX }
}

#[test]
fn a_kept_record_of_another_family_version_is_refused() {
    let limits = fixtures::limits();
    let kept = Kept { store_version: u16::MAX, record: Box::from([255, 0]) };
    let mut bytes = fixtures::encoded(&kept);
    bytes[0] = 2;
    bytes[1] = 0;
    assert_eq!(
        Kept::decode(&limits, &mut Reader::new(&bytes)),
        Err(Problem { path: paths::KEPT_VERSION, reason: skein_codec::Reason::Version })
    );
}

fn a_table_entry_never_sent_round_trips_with_no_wall() {
    let limits = fixtures::limits();
    let family = small_store_family();
    let entry = TableEntry {
        asker: 65536,
        number: 1,
        key: [7; 16],
        scope: u64::MAX,
        first_sent: None,
        owner: Owner::Family(family),
        version: u32::MAX,
        request: Box::from([255, 0]),
    };
    let bytes = fixtures::encoded(&entry);
    let decoded = TableEntry::decode(&limits, &mut Reader::new(&bytes)).expect("whole table entry");
    assert_eq!((&decoded.first_sent), &None);
    assert_eq!(decoded, entry);
}

fn each_read_value_variant_round_trips_unchanged() {
    let limits = fixtures::limits();
    for (name, value) in &fixtures::read_values(&limits) {
        let bytes = fixtures::encoded(value);
        assert_eq!(ReadValue::decode(&limits, &mut Reader::new(&bytes)), Ok(value.clone()), "{name}");
    }
}

fn a_stream_text_event_round_trips_with_its_watch() {
    let limits = fixtures::stream_limits();
    let event = StreamEvent::Text(fixtures::stream_text(&limits, fixtures::raw_live_call(&limits)));
    let bytes = fixtures::encoded(&event);
    assert_eq!(StreamEvent::decode(&limits, &mut Reader::new(&bytes)), Ok(event));
}

fn each_delivery_value_variant_round_trips_unchanged() {
    let limits = fixtures::limits();
    for (name, value) in &fixtures::delivery_values(&limits) {
        let bytes = fixtures::encoded(value);
        assert_eq!(DeliveryValue::decode(&limits, &mut Reader::new(&bytes)), Ok(value.clone()), "{name}");
    }
}

#[test]
fn each_turn_bytes_variant_round_trips_with_the_carried_ceiling() {
    let limits = fixtures::limits();
    let rows = [
        ("carried at 8 MiB", TurnBytes::Carried(TurnCarried { bytes: fixtures::bytes(8_388_608) })),
        ("too large", TurnBytes::TooLarge(TurnTooLarge { bytes: u64::MAX })),
    ];
    for (name, value) in rows {
        let bytes = fixtures::encoded(&value);
        assert_eq!(TurnBytes::decode(&limits, &mut Reader::new(&bytes)), Ok(value), "{name}");
    }
}

fn a_sign_in_not_ready_round_trips_unchanged() {
    let limits = fixtures::limits();
    let answer = SignInAnswer::NotReady;
    let bytes = fixtures::encoded(&answer);
    assert_eq!(SignInAnswer::decode(&limits, &mut Reader::new(&bytes)), Ok(answer), "sign-in not_ready");
}

fn a_sign_out_not_ready_round_trips_unchanged() {
    let limits = fixtures::limits();
    let answer = SignOutAnswer::NotReady;
    let bytes = fixtures::encoded(&answer);
    assert_eq!(SignOutAnswer::decode(&limits, &mut Reader::new(&bytes)), Ok(answer), "sign-out not_ready");
}

#[test]
fn wire_values_round_trip_unchanged() {
    table([
        ("a_compound_s_operations_round_trip_as_pieces", a_compound_s_operations_round_trip_as_pieces),
        ("version_not_served_is_tag_zero_of_each_answer", version_not_served_is_tag_zero_of_each_answer),
        (
            "a_family_record_round_trips_with_its_scope_and_window",
            a_family_record_round_trips_with_its_scope_and_window,
        ),
        ("an_owned_record_keeps_its_owners_version", an_owned_record_keeps_its_owners_version),
        ("a_refusal_with_its_facts_round_trips_unchanged", a_refusal_with_its_facts_round_trips_unchanged),
        ("each_answer_kind_round_trips_unchanged", each_answer_kind_round_trips_unchanged),
        ("each_read_none_round_trips_unchanged", each_read_none_round_trips_unchanged),
        ("a_read_value_round_trips_with_its_owner_version", a_read_value_round_trips_with_its_owner_version),
        ("an_activity_line_carries_its_turn_bytes_unchanged", an_activity_line_carries_its_turn_bytes_unchanged),
        ("me_at_its_ceilings_round_trips_unchanged", me_at_its_ceilings_round_trips_unchanged),
        ("a_run_state_without_writing_or_call_round_trips", a_run_state_without_writing_or_call_round_trips),
        ("a_chip_with_each_phase_round_trips_unchanged", a_chip_with_each_phase_round_trips_unchanged),
        ("a_hello_naming_two_owners_round_trips_unchanged", a_hello_naming_two_owners_round_trips_unchanged),
        ("a_stream_event_at_its_ceiling_round_trips_unchanged", a_stream_event_at_its_ceiling_round_trips_unchanged),
        ("each_watch_end_but_dropped_round_trips", each_watch_end_but_dropped_round_trips),
        ("a_table_entry_never_sent_round_trips_with_no_wall", a_table_entry_never_sent_round_trips_with_no_wall),
        ("each_read_value_variant_round_trips_unchanged", each_read_value_variant_round_trips_unchanged),
        ("a_stream_text_event_round_trips_with_its_watch", a_stream_text_event_round_trips_with_its_watch),
        ("each_delivery_value_variant_round_trips_unchanged", each_delivery_value_variant_round_trips_unchanged),
        ("a_sign_in_not_ready_round_trips_unchanged", a_sign_in_not_ready_round_trips_unchanged),
        ("a_sign_out_not_ready_round_trips_unchanged", a_sign_out_not_ready_round_trips_unchanged),
    ]);
}

#[test]
fn wire_fields_past_their_bounds_are_refused() {
    table([
        ("a_keyed_request_past_its_bound_is_refused_naming_it", a_keyed_request_past_its_bound_is_refused_naming_it),
        ("a_bearer_token_past_its_bound_is_refused", a_bearer_token_past_its_bound_is_refused),
        ("a_cursor_past_its_bound_is_refused", a_cursor_past_its_bound_is_refused),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    skein_rows::check(cases, |case| case());
}

fn version_not_served_is_tag_zero_of_each_answer() {
    let refusal = fixtures::not_served();
    let values = [
        fixtures::encoded(&KeyedAnswer::NotServed(refusal.clone())),
        fixtures::encoded(&ReadAnswer::NotServed(refusal.clone())),
        fixtures::encoded(&OpenAnswer::NotServed(refusal.clone())),
        fixtures::encoded(&WatchEnd::NotServed(refusal)),
    ];
    for bytes in values {
        assert_eq!(bytes.first(), Some(&0));
    }
}
const FAMILY_PROJECT: u32 = 7;
fn a_family_record_round_trips_with_its_scope_and_window() {
    let limits = fixtures::limits();
    for scope in [Scope::Own, Scope::Deployment, Scope::Project(ScopeProject { value: FAMILY_PROJECT })] {
        let after = Some(Cursor { value: Box::from([1, 2, 3]) });
        let window = Window { rows: 3, after };
        let record = FamilyRecord { scope, window, record: Box::from([255, 0]) };
        let bytes = fixtures::encoded(&record);
        assert_eq!(FamilyRecord::decode(&limits, &mut Reader::new(&bytes)), Ok(record));
    }
}

#[test]
fn each_merged_operation_target_round_trips_unchanged() {
    let limits = fixtures::limits();
    for (name, value) in fixtures::merged_targets() {
        let bytes = fixtures::encoded(&value);
        assert_eq!(Ask::decode(&limits, &mut Reader::new(&bytes)), Ok(value), "{name}");
    }
}

fn a_compound_s_operations_round_trip_as_pieces() {
    use skein_lib::Writer;
    let limits = fixtures::limits();
    let values = fixtures::compound_operations();
    let mut size = 0_usize;
    for value in &values {
        size = size
            .checked_add(usize::try_from(value.measure(&CEILINGS).expect("bounded fixture")).unwrap())
            .unwrap()
            .checked_add(4)
            .unwrap();
    }
    let mut bytes = Writer::new(size);
    for value in &values {
        bytes.put(&value.measure(&CEILINGS).expect("bounded fixture").to_be_bytes()).unwrap();
        value.write(&CEILINGS, &mut bytes).unwrap();
    }
    let record = Compound { project: fixtures::COMPOUND_PROJECT, operations: bytes.finish() };
    let bytes = fixtures::encoded(&record);
    let restored = Compound::decode(&limits, &mut Reader::new(&bytes)).unwrap();
    assert_eq!(restored, record);
    let mut reader = Reader::new(restored.operations.as_ref());
    for expected in values {
        let bytes = skein_codec::piece::split(&mut reader).unwrap().unwrap();
        assert_eq!(Operation::decode(&limits, &mut Reader::new(&bytes)), Ok(expected));
    }
    assert!(reader.is_empty());
}

#[test]
fn a_me_project_without_its_name_cannot_decode() {
    let limits = fixtures::limits();
    let mut prefix = skein_lib::Writer::new(8);
    prefix.put(&0_u32.to_be_bytes()).expect("left-out count");
    prefix.put(&u32::MAX.to_be_bytes()).expect("project number");
    assert_eq!(
        MeProject::decode(&limits, &mut Reader::new(&prefix.finish())),
        Err(Problem { path: paths::DOCUMENT_TEXT_BYTES, reason: skein_codec::Reason::Short })
    );
}

// protocol/client/README.md, 3: Keyed.value holds at most 128 KiB of a moved file.
const MOVED_FILE_BYTES: usize = 131_072;
#[test]
fn a_keyed_value_at_the_move_bound_round_trips() {
    let limits = CEILINGS;
    let value = skein_lib::Secret::new(skein_lib::bytes::zeroed(MOVED_FILE_BYTES));
    let keyed = Keyed { key: [7; 16], owner: Owner::Jig, version: VERSION, request: Box::new([]), value: Some(value) };
    let bytes = fixtures::encoded(&keyed);
    assert_eq!(Keyed::decode(&limits, &mut Reader::new(&bytes)), Ok(keyed));
}
#[test]
fn a_keyed_value_one_byte_past_the_move_bound_is_refused() {
    let value = skein_lib::Secret::new(skein_lib::bytes::zeroed(MOVED_FILE_BYTES + 1));
    assert_eq!(
        (Keyed { key: [7; 16], owner: Owner::Jig, version: VERSION, request: Box::new([]), value: Some(value) })
            .encode(&CEILINGS),
        Err(Problem { path: paths::KEYED_VALUE, reason: skein_codec::Reason::Bound })
    );
}

#[test]
fn authorization_query_and_each_answer_round_trip_at_their_bounds() {
    let limits = CEILINGS;
    let record = Authorize { query: fixtures::bytes(8192) };
    let bytes = fixtures::encoded(&record);
    assert_eq!(Authorize::decode(&limits, &mut Reader::new(&bytes)), Ok(record));
    assert_eq!(
        (Authorize { query: fixtures::bytes(8193) }).encode(&CEILINGS),
        Err(Problem { path: paths::AUTHORIZE_QUERY, reason: skein_codec::Reason::Bound })
    );
    assert_eq!(
        (AuthorizeGo { location: fixtures::bytes(16385) }).encode(&CEILINGS),
        Err(Problem { path: paths::AUTHORIZE_GO_LOCATION, reason: skein_codec::Reason::Bound })
    );
    let go = AuthorizeGo { location: fixtures::bytes(16384) };
    for answer in
        [AuthorizeAnswer::Go(go), AuthorizeAnswer::SignedOut, AuthorizeAnswer::NotReady, AuthorizeAnswer::Busy]
    {
        let bytes = fixtures::encoded(&answer);
        assert_eq!(AuthorizeAnswer::decode(&limits, &mut Reader::new(&bytes)), Ok(answer));
    }
    for why in [
        AuthorizeWhy::Malformed,
        AuthorizeWhy::UnknownClient,
        AuthorizeWhy::UnknownRedirect,
        AuthorizeWhy::NoProvider,
        AuthorizeWhy::NotAPerson,
    ] {
        let answer = AuthorizeAnswer::Refused(AuthorizeRefused { why });
        let bytes = fixtures::encoded(&answer);
        assert_eq!(AuthorizeAnswer::decode(&limits, &mut Reader::new(&bytes)), Ok(answer));
    }
}

#[test]
fn lent_login_names_are_bounded_at_32_bytes() {
    for length in [32, 33] {
        let name = skein_lib::bytes::zeroed(length);
        let terminal =
            (TerminalLogin { number: 1, provider: 2, name: name.clone(), cancelled: false }).encode(&CEILINGS);
        let change = (AccountChangeLoginProgram { provider: 2, name }).encode(&CEILINGS);
        assert_eq!(terminal.is_ok(), length == 32);
        assert_eq!(change.is_ok(), length == 32);
    }
}

#[test]
fn a_stop_reason_is_bounded_and_utf8() {
    let stop = Stop { project: 3, task: 8, reason: Box::from(b"hold".as_slice()) };
    let bytes = stop.encode(&CEILINGS).expect("valid reason");
    let limits = Limits { stop_reason: 3, ..CEILINGS };
    let bound = Err(paths::STOP_REASON.problem(skein_codec::Reason::Bound));
    assert_eq!(Stop::decode(&limits, &mut Reader::new(&bytes)), bound);
    assert_eq!(stop.encode(&limits).err(), bound.err());

    let mut invalid = bytes;
    *invalid.last_mut().expect("reason's last byte") = 0xff;
    assert_eq!(
        Stop::decode(&CEILINGS, &mut Reader::new(&invalid)),
        Err(paths::STOP_REASON.problem(skein_codec::Reason::Utf8)),
    );
}
