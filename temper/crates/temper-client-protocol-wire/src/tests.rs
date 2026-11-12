//! Version, bounds and total outcome round trips (client/forge/records.md).
mod fixtures;
mod source;
use crate::v1;
use alloc::boxed::Box;
use skein_lib::{List, Reader};

#[test]
fn an_adopt_of_another_version_is_refused() {
    let mut bytes = crate::requesting(
        &v1::Requesting::new(
            &fixtures::limits(),
            v1::RequestingParts { request: v1::Request::Adopt(fixtures::adopt()) },
        )
        .expect("request"),
    );
    *bytes.get_mut(1).expect("the record starts with a u16 version") =
        u8::try_from(crate::CLIENT_VERSION + 1).expect("test version fits");
    let problem =
        v1::Requesting::decode(&fixtures::limits(), &mut Reader::new(&bytes)).expect_err("another version is refused");
    assert_eq!(problem, v1::Problem { path: v1::Path::RequestingVersion, reason: skein_codec::Reason::Version });
}

#[test]
fn an_owner_one_byte_past_its_bound_is_refused() {
    let parts = fixtures::adopt().into_parts();
    v1::Adopt::new(&fixtures::limits(), v1::AdoptParts { owner: Box::new([b'a'; 16]), ..parts.clone() })
        .expect("at the bound");
    let made = v1::Adopt::new(&fixtures::limits(), v1::AdoptParts { owner: Box::new([b'a'; 17]), ..parts });
    assert_eq!(
        made.expect_err("past the owner bound"),
        v1::Problem { path: v1::Path::AdoptOwner, reason: skein_codec::Reason::Bound }
    );
}

#[test]
fn checks_one_past_their_count_are_refused() {
    let mut checks = List::with_capacity(3);
    for number in 1..=3 {
        checks.push(number).expect("counted room");
    }
    let parts = v1::AdoptParts { checks, ..fixtures::adopt().into_parts() };
    let problem = v1::Adopt::new(&fixtures::limits(), parts).expect_err("past the checks count");
    assert_eq!(problem, v1::Problem { path: v1::Path::AdoptChecks, reason: skein_codec::Reason::Bound });
}

#[test]
fn each_outcome_round_trips_unchanged() {
    let limits = fixtures::limits();
    for (row, outcome) in fixtures::outcome_rows().into_iter().enumerate() {
        let answer = v1::Answer::new(&limits, v1::AnswerParts { outcome }).expect("an outcome");
        let bytes = crate::answer(&answer);
        let back = v1::Answer::decode(&limits, &mut Reader::new(&bytes)).expect("decodes");
        assert_eq!(back, answer, "outcome row {row}");
    }
}

#[test]
#[expect(clippy::too_many_lines, reason = "one expected literal pins every wire bound to its source")]
fn limits_take_each_bound_from_its_source_field() {
    // A table full at every bound (temper/docs/design/protocol/configuration/settings.md, 2).
    const ITEMS: u32 = 20;
    const SETS: u8 = 6;
    const WORDS_BYTES: usize = 120;
    const NAME_BYTES: usize = 40;
    const PATTERNS: u32 = 6;
    const PATTERN_BYTES: usize = 80;
    assert_eq!(
        fixtures::limits(),
        v1::Limits {
            change_panel_approvals: 8192,
            change_panel_unmapped_reviews: 256,
            landing_exact_value: 8,
            landing_open_value: 8,
            landing_pattern_segments: 3,
            landing_pattern_segments_item: 8,
            landing_rule_gates: 2,
            landing_rule_approvals: 1,
            verifying_tiers: 32,
            verifying_tiers_item: 8192,
            red_tiers: 32,
            red_tiers_item: 8192,
            cleared_tiers: 32,
            cleared_tiers_item: 8192,
            authority_hold_reason: 512,
            owned_forge_forgejo: 64,
            owned_forge_runner_some: 64,
            owned_forge_password: 512,
            adoption_owner: 16,
            adoption_name: 16,
            forge_row_host: crate::ADDRESS_BYTES,
            forge_row_web_some: crate::ADDRESS_BYTES,
            forge_row_repositories: 16,
            forge_row_makings: crate::MAKINGS,
            make_copy_address: 512,
            make_repository_org: 16,
            make_repository_name: 16,
            dismiss_making_org: 16,
            dismiss_making_name: 16,
            making_made_branch: 16,
            making_failed_words: 256,
            repository_making_org: 16,
            repository_making_name: 16,
            reached_name: 16,
            page_rows: 100,
            install_back_some: 512,
            git_hub_manifest_back_some: 512,
            link_address: crate::ADDRESS_BYTES,
            forges_forges: crate::FORGES,
            dismissals_dismissed: crate::DISMISSED,
            set_up_failed_why: crate::SET_UP_WHY_BYTES,
            group_failed_why: crate::SET_UP_WHY_BYTES,
            connected_login: 16,
            account_login: 16,
            forge_row_installations: crate::INSTALLATIONS,
            connect_stopped_why: crate::CONNECT_WHY_BYTES,
            stopped_reason: 16,
            pull_address: crate::ADDRESS_BYTES,
            receipted_address: 2048,
            gate_proof_name: 32,
            member_owner: 1024,
            member_name: 1024,
            member_branch: 1024,
            member_landing: 1024,
            member_gates: 8,
            landing_panel_members: 8,
            landing_line_changes: 8,
            landing_line_repositories: 8,
            landing_line_repositories_item: 2048,
            landing_line_landed: 8,
            landings_panel_landings: 64,
            queued_ahead: crate::TASKS_AHEAD,
            change_panel_owner: 16,
            change_panel_name: 16,
            change_panel_branch: 16,
            change_panel_landing: 16,
            change_panel_gates: crate::GATES,
            question_line: 256,
            questions_open: 8,
            managed_forge_host: 255,
            managed_forge_tls_some: 255,
            managed_forge_git: crate::ADDRESS_BYTES,
            managed_forge_web_some: crate::ADDRESS_BYTES,
            managed_forge_git_program_some: 4096,
            playground_answer_owner: 16,
            playground_answer_name: 16,
            playground_answer_cues: crate::CUES,
            playground_answer_cues_item: crate::CUE_BYTES,
            first_ask_org: 16,
            first_ask_repository: 16,
            first_ask_label: 64,
            first_empty_org: 16,
            first_empty_repository: 16,
            first_repository_org: 16,
            first_repository_repository: 16,
            first_repository_label: 64,
            adopt_owner: 16,
            adopt_name: 16,
            adopt_prefix: 16,
            adopt_landing: 16,
            adopt_checks: 2,
            rule_branch: 16,
            rule_contexts: 2,
            rule_contexts_item: 16,
            adopted_owner: 16,
            adopted_name: 16,
            core_refused_refusal: 64,
            ..v1::CEILINGS
        }
    );
    let bounds = crate::Bounds { checks: 65, ..fixtures::BOUNDS };
    assert_eq!(crate::limits(&bounds), None, "past the checks ceiling");

    let limits = fixtures::limits();
    let item = v1::ClassItem::new(
        &limits,
        v1::ClassItemParts {
            subject: v1::ClassSubject::Kind(
                v1::ReachKindOf::new(&limits, v1::ReachKindOfParts { kind: v1::ReachKind::Implementation })
                    .expect("reach kind"),
            ),
            words: filled(WORDS_BYTES),
            class: v1::ReachClass::Ask,
        },
    )
    .expect("words at the bound");
    let mut sets = List::with_capacity(u32::from(SETS));
    for number in 1..=SETS {
        let mut patterns = List::with_capacity(PATTERNS);
        for _ in 0..PATTERNS {
            patterns.push(filled(PATTERN_BYTES)).expect("counted patterns");
        }
        sets.push(
            v1::PathSet::new(&limits, v1::PathSetParts { number, name: filled(NAME_BYTES), patterns })
                .expect("name and patterns at their bounds"),
        )
        .expect("counted sets");
    }
    for (count, problem) in [
        (ITEMS, None),
        (
            ITEMS.checked_add(1).expect("one past items"),
            Some(v1::Problem { path: v1::Path::ProjectClassesItems, reason: skein_codec::Reason::Bound }),
        ),
    ] {
        let mut items = List::with_capacity(count);
        for _ in 0..count {
            items.push(item.clone()).expect("counted items");
        }
        let result = v1::ProjectClasses::new(&limits, v1::ProjectClassesParts { items, sets: sets.clone() });
        match problem {
            None => {
                let classes = result.expect("classes at every bound");
                let mut writer = skein_lib::Writer::new(usize::try_from(classes.measure()).expect("measured table"));
                classes.encode(&mut writer).expect("measured room");
                let bytes = writer.finish();
                assert_eq!(v1::ProjectClasses::decode(&limits, &mut Reader::new(&bytes)), Ok(classes));
            }
            Some(problem) => assert_eq!(result, Err(problem)),
        }
    }
    let reach = v1::Reach::new(
        &limits,
        v1::ReachParts {
            kind: v1::ReachKind::Implementation,
            class: v1::ReachClass::AskAway,
            reason: v1::ReachReason::Set(
                v1::ReachSet::new(&limits, v1::ReachSetParts { number: SETS }).expect("reason set"),
            ),
            under: u8::try_from(ITEMS).expect("item number"),
            sets: 0b11_1111,
            matched: 0b10_0101,
            away: true,
        },
    )
    .expect("reach");
    let mut writer = skein_lib::Writer::new(usize::try_from(reach.measure()).expect("measured reach"));
    reach.encode(&mut writer).expect("measured room");
    assert_eq!(v1::Reach::decode(&limits, &mut Reader::new(&writer.finish())), Ok(reach));
}

#[test]
fn each_owner_round_trips_its_number() {
    for (owner, number) in [(crate::Owner::Forge, 1), (crate::Owner::Pages, 2), (crate::Owner::Runner, 3)] {
        assert_eq!(crate::number(owner), number);
        assert_eq!(crate::owner(number), Some(owner));
    }
}

#[test]
fn an_unknown_owner_number_has_no_owner() {
    assert_eq!(crate::owner(65_535), None);
}

#[test]
fn a_change_watch_round_trips_and_refuses_trailing_bytes() {
    let limits = fixtures::limits();
    let subject = v1::ChangeSubject::new(&limits, v1::ChangeSubjectParts { task: 17 }).expect("change task");
    let watch = v1::Watching::new(&limits, v1::WatchingParts { watch: v1::Watch::Change(subject) }).expect("watch");
    let bytes = crate::watching(&watch);
    assert_eq!(crate::decode_watching(&bytes, &limits), Some(watch));
    let trailing = fixtures::trailing(&bytes);
    assert_eq!(crate::decode_watching(&trailing, &limits), None);
}

#[test]
fn each_change_panel_round_trips_unchanged() {
    let limits = fixtures::limits();
    for (row, panel) in fixtures::panel_rows().as_slice().iter().enumerate() {
        assert_eq!(crate::decode_change_panel(&crate::change_panel(panel), &limits).as_ref(), Some(panel), "row {row}");
    }
}

#[test]
fn a_change_panel_of_another_version_is_refused() {
    let limits = fixtures::limits();
    let panel = v1::ChangePanel::new(&limits, fixtures::panel_parts()).expect("panel");
    let mut bytes = crate::change_panel(&panel);
    *bytes.get_mut(1).expect("u16 version") = u8::try_from(crate::CLIENT_VERSION + 1).expect("test version fits");
    assert_eq!(
        v1::ChangePanel::decode(&limits, &mut Reader::new(&bytes)).expect_err("another version"),
        v1::Problem { path: v1::Path::ChangePanelVersion, reason: skein_codec::Reason::Version }
    );
    assert_eq!(crate::decode_change_panel(&bytes, &limits), None);
}

#[test]
fn a_change_panel_refuses_trailing_bytes_and_a_zero_queue_place() {
    let limits = fixtures::limits();
    for (row, place, trailing) in [("first", 1, false), ("zero", 0, false), ("trailing", 1, true)] {
        let parts = v1::ChangePanelParts {
            step: v1::Step::Queued(fixtures::queued(place, 0, &limits)),
            ..fixtures::panel_parts()
        };
        let panel = v1::ChangePanel::new(&limits, parts).expect("schema panel");
        let bytes = crate::change_panel(&panel);
        let bytes = if trailing { fixtures::trailing(&bytes) } else { bytes };
        assert_eq!(
            crate::decode_change_panel(&bytes, &limits),
            if place == 0 || trailing { None } else { Some(panel) },
            "{row}"
        );
    }
}

#[test]
fn change_panel_bounds_accept_at_and_refuse_one_past() {
    let limits = fixtures::limits();
    for (field, path, bound) in [
        (fixtures::PanelBound::Address, v1::Path::PullAddress, crate::ADDRESS_BYTES),
        (fixtures::PanelBound::Ahead, v1::Path::QueuedAhead, crate::TASKS_AHEAD),
        (fixtures::PanelBound::Gates, v1::Path::ChangePanelGates, crate::GATES),
        (fixtures::PanelBound::Approvals, v1::Path::ChangePanelApprovals, limits.change_panel_approvals),
        (fixtures::PanelBound::Counted, v1::Path::ApprovalCounted, limits.approval_counted),
        (fixtures::PanelBound::Unmapped, v1::Path::ChangePanelUnmappedReviews, limits.change_panel_unmapped_reviews),
        (fixtures::PanelBound::Owner, v1::Path::ChangePanelOwner, fixtures::BOUNDS.field_bytes),
        (fixtures::PanelBound::Name, v1::Path::ChangePanelName, fixtures::BOUNDS.field_bytes),
        (fixtures::PanelBound::Branch, v1::Path::ChangePanelBranch, fixtures::BOUNDS.field_bytes),
        (fixtures::PanelBound::Landing, v1::Path::ChangePanelLanding, fixtures::BOUNDS.field_bytes),
    ] {
        for count in [bound, bound + 1] {
            let made = fixtures::panel_at_bound(field, count);
            if count == bound {
                let panel = made.expect("at bound");
                assert_eq!(crate::decode_change_panel(&crate::change_panel(&panel), &limits), Some(panel), "{path:?}");
            } else {
                assert_eq!(made.expect_err("past bound"), v1::Problem { path, reason: skein_codec::Reason::Bound });
            }
        }
    }
}

#[test]
fn requesting_round_trips_and_refuses_trailing_and_version_three_adopt_bytes() {
    let limits = fixtures::limits();
    for request in [
        v1::Request::Adopt(fixtures::adopt()),
        v1::Request::SetUp(
            v1::SetUp::new(
                &limits,
                v1::SetUpParts {
                    forge: kept(u16::MAX),
                    first: v1::SetUpFirst::Empty(
                        v1::FirstEmpty::new(
                            &v1::CEILINGS,
                            v1::FirstEmptyParts {
                                playground: true,
                                org: b"work".as_slice().into(),
                                repository: b"hello".as_slice().into(),
                            },
                        )
                        .expect("setup names fit wire ceilings"),
                    ),
                },
            )
            .expect("forge number"),
        ),
    ] {
        let record = v1::Requesting::new(&limits, v1::RequestingParts { request }).expect("request");
        let bytes = crate::requesting(&record);
        assert_eq!(crate::decode_requesting(&bytes, &limits), Some(record));
        assert_eq!(crate::decode_requesting(&fixtures::trailing(&bytes), &limits), None);
    }
    // Version 3's Adopt was versioned directly, without a request discriminator.
    let adopt = fixtures::adopt();
    let mut writer = skein_lib::Writer::new(usize::try_from(adopt.measure()).expect("length") + 2);
    writer.put(&[0, 3]).expect("old version");
    adopt.encode(&mut writer).expect("old adoption body");
    assert_eq!(crate::decode_requesting(&writer.finish(), &limits), None);
}

#[test]
fn forges_watch_round_trips_and_refuses_trailing_bytes() {
    let limits = fixtures::limits();
    let watch = v1::Watching::new(&limits, v1::WatchingParts { watch: v1::Watch::Forges }).expect("forges watch");
    let bytes = crate::watching(&watch);
    assert_eq!(crate::decode_watching(&bytes, &limits), Some(watch));
    assert_eq!(crate::decode_watching(&fixtures::trailing(&bytes), &limits), None);
}

fn forges_rows(numbers: &[u16], connection: &v1::Connection) -> v1::Forges {
    let limits = fixtures::limits();
    let mut forges = List::with_capacity(u32::try_from(numbers.len()).expect("small list"));
    for number in numbers {
        forges
            .push(
                v1::ForgeRow::new(
                    &limits,
                    v1::ForgeRowParts {
                        starts_with: v1::StartsWith::Repository,
                        makings: List::with_capacity(0),
                        host: b"git.example:3000".as_slice().into(),
                        number: *number,
                        kind: v1::Kind::Forgejo,
                        mode: v1::Mode::Owned,
                        connection: connection.clone(),
                        offer: v1::Offer::Offered,
                        first: None,
                        installations: List::with_capacity(0),
                        ended: None,
                        app: None,
                        web: None,
                        repositories: List::with_capacity(0),
                        owned: None,
                        admin: false,
                        read_only: false,
                        ended_stop: None,
                    },
                )
                .expect("row"),
            )
            .expect("row capacity");
    }
    v1::Forges::new(&limits, v1::ForgesParts { forges, linked: false }).expect("forges")
}

#[test]
fn each_forge_offer_round_trips() {
    let limits = fixtures::limits();
    let mut forges = List::with_capacity(3);
    for (number, offer) in [(1, v1::Offer::NotOffered), (2, v1::Offer::NotUnderTheBox), (3, v1::Offer::Offered)] {
        let row = v1::ForgeRow::new(
            &limits,
            v1::ForgeRowParts {
                starts_with: v1::StartsWith::Repository,
                makings: List::with_capacity(0),
                host: b"git.example:3000".as_slice().into(),
                number,
                kind: v1::Kind::Forgejo,
                mode: v1::Mode::Owned,
                connection: v1::Connection::NotSetUp,
                offer,
                first: None,
                installations: List::with_capacity(0),
                ended: None,
                app: None,
                web: None,
                repositories: List::with_capacity(0),
                owned: None,
                admin: false,
                read_only: false,
                ended_stop: None,
            },
        )
        .expect("forge row");
        forges.push(row).expect("three rows");
    }
    for linked in [false, true] {
        let record = v1::Forges::new(&limits, v1::ForgesParts { forges: forges.clone(), linked }).expect("forges");
        let bytes = crate::forges(&record);
        let decoded = crate::decode_forges(&bytes, &limits).expect("forges decode");
        assert_eq!(decoded.linked(), linked);
        assert_eq!(decoded, record);
        assert_eq!(crate::decode_forges(&fixtures::trailing(&bytes), &limits), None);
    }
}

#[test]
fn each_forge_connection_round_trips_at_its_bounds() {
    let limits = fixtures::limits();
    let failed = v1::SetUpFailed::new(&limits, v1::SetUpFailedParts { why: fixtures::field_bytes(256), at: None })
        .expect("why at bound");
    let connected = v1::Connected::new(&limits, v1::ConnectedParts { id: u64::MAX, login: fixtures::field_bytes(16) })
        .expect("login at bound");
    let stopped =
        v1::Stopped::new(&limits, v1::StoppedParts { reason: fixtures::field_bytes(16) }).expect("reason at bound");
    for connection in [
        v1::Connection::NotConnected,
        v1::Connection::NotSetUp,
        v1::Connection::SetUpFailed(failed),
        v1::Connection::Connecting,
        v1::Connection::Connected(connected),
        v1::Connection::Stopped(stopped),
    ] {
        let record = forges_rows(&[u16::MAX], &connection);
        let bytes = crate::forges(&record);
        assert_eq!(crate::decode_forges(&bytes, &limits), Some(record));
        assert_eq!(crate::decode_forges(&fixtures::trailing(&bytes), &limits), None);
    }
    for step in [None, Some(v1::SetUpStep::Fetch), Some(v1::SetUpStep::Prepare), Some(v1::SetUpStep::Run)] {
        let connection = v1::Connection::SettingUp(
            v1::SettingUp::new(&limits, v1::SettingUpParts { step, fetched: None }).expect("setup step"),
        );
        let record = forges_rows(&[1], &connection);
        assert_eq!(crate::decode_forges(&crate::forges(&record), &limits), Some(record));
    }
}

#[test]
fn a_setting_up_carries_its_fetch() {
    let limits = fixtures::limits();
    for binary in [v1::SetUpBinary::Forgejo, v1::SetUpBinary::Runner] {
        for total in [Some(122_057_320), None] {
            let fetched = v1::SetUpFetched::new(
                &limits,
                v1::SetUpFetchedParts { binary: binary.clone(), done: 1_048_576, total },
            )
            .expect("fetched bytes");
            let connection = v1::Connection::SettingUp(
                v1::SettingUp::new(
                    &limits,
                    v1::SettingUpParts { step: Some(v1::SetUpStep::Fetch), fetched: Some(fetched) },
                )
                .expect("fetching"),
            );
            let record = forges_rows(&[1], &connection);
            let bytes = crate::forges(&record);
            assert_eq!(crate::decode_forges(&bytes, &limits), Some(record), "{binary:?}, total {total:?}");
        }
    }
}

#[test]
fn forge_fields_refuse_one_past_their_bounds_on_construction_and_decode() {
    let limits = fixtures::limits();
    let mut writer = skein_lib::Writer::new(4 + 257);
    writer.put(&257_u32.to_be_bytes()).expect("why length");
    writer.put(&[b'w'; 257]).expect("why bytes");
    assert_eq!(
        v1::SetUpFailed::decode(&limits, &mut Reader::new(&writer.finish())).expect_err("why past ceiling"),
        v1::Problem { path: v1::Path::SetUpFailedWhy, reason: skein_codec::Reason::Bound }
    );
    assert_eq!(
        v1::SetUpFailed::new(&limits, v1::SetUpFailedParts { why: fixtures::field_bytes(257), at: None })
            .expect_err("past bound"),
        v1::Problem { path: v1::Path::SetUpFailedWhy, reason: skein_codec::Reason::Bound }
    );
    assert_eq!(
        v1::Connected::new(&limits, v1::ConnectedParts { id: 1, login: fixtures::field_bytes(17) })
            .expect_err("past bound"),
        v1::Problem { path: v1::Path::ConnectedLogin, reason: skein_codec::Reason::Bound }
    );
    assert_eq!(
        v1::Stopped::new(&limits, v1::StoppedParts { reason: fixtures::field_bytes(17) }).expect_err("past bound"),
        v1::Problem { path: v1::Path::StoppedReason, reason: skein_codec::Reason::Bound }
    );
    // A record legal at schema ceilings must still respect the receiving side's bounds.
    for connection in [
        v1::Connection::Connected(
            v1::Connected::new(&v1::CEILINGS, v1::ConnectedParts { id: 1, login: fixtures::field_bytes(17) })
                .expect("schema login"),
        ),
        v1::Connection::Stopped(
            v1::Stopped::new(&v1::CEILINGS, v1::StoppedParts { reason: fixtures::field_bytes(17) })
                .expect("schema reason"),
        ),
    ] {
        let row = v1::ForgeRow::new(
            &v1::CEILINGS,
            v1::ForgeRowParts {
                starts_with: v1::StartsWith::Repository,
                makings: List::with_capacity(0),
                host: b"git.example:3000".as_slice().into(),
                number: 1,
                kind: v1::Kind::Github,
                mode: v1::Mode::Connected,
                connection,
                offer: v1::Offer::NotOffered,
                first: None,
                installations: List::with_capacity(0),
                ended: None,
                app: None,
                web: None,
                repositories: List::with_capacity(0),
                owned: None,
                admin: false,
                read_only: false,
                ended_stop: None,
            },
        )
        .expect("schema row");
        let mut forges = List::with_capacity(1);
        forges.push(row).expect("one row");
        let record = v1::Forges::new(&v1::CEILINGS, v1::ForgesParts { forges, linked: false }).expect("schema forges");
        assert_eq!(crate::decode_forges(&crate::forges(&record), &limits), None);
    }
}

#[test]
fn forges_refuse_seventeen_rows_unsorted_rows_and_duplicate_numbers() {
    let limits = fixtures::limits();
    let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
    let at = forges_rows(&numbers, &v1::Connection::NotConnected);
    assert_eq!(crate::decode_forges(&crate::forges(&at), &limits), Some(at.clone()));
    let mut parts = at.into_parts();
    let mut rows = List::with_capacity(17);
    for row in parts.forges.into_boxed() {
        rows.push(row).expect("room");
    }
    rows.push(
        v1::ForgeRow::new(
            &limits,
            v1::ForgeRowParts {
                starts_with: v1::StartsWith::Repository,
                makings: List::with_capacity(0),
                host: b"git.example:3000".as_slice().into(),
                number: 17,
                kind: v1::Kind::Github,
                mode: v1::Mode::Fake,
                connection: v1::Connection::NotSetUp,
                offer: v1::Offer::NotOffered,
                first: None,
                installations: List::with_capacity(0),
                ended: None,
                app: None,
                web: None,
                repositories: List::with_capacity(0),
                owned: None,
                admin: false,
                read_only: false,
                ended_stop: None,
            },
        )
        .expect("row"),
    )
    .expect("extra room");
    parts.forges = rows;
    assert_eq!(
        v1::Forges::new(&limits, parts).expect_err("past bound"),
        v1::Problem { path: v1::Path::ForgesForges, reason: skein_codec::Reason::Bound }
    );
    // A peer can send seventeen rows even though the constructor refuses them.
    let at = forges_rows(&numbers, &v1::Connection::NotConnected);
    let row = at.forges().get(0).expect("row");
    let length = at.measure().checked_add(row.measure()).expect("seventeen rows fit");
    let mut writer = skein_lib::Writer::new(usize::try_from(length).expect("length"));
    writer.put(&u16::try_from(crate::CLIENT_VERSION).expect("version fits").to_be_bytes()).expect("version");
    writer.put(&17_u32.to_be_bytes()).expect("row count");
    for row in at.forges() {
        row.encode(&mut writer).expect("row bytes");
    }
    row.encode(&mut writer).expect("extra row bytes");
    writer.put(&[u8::from(at.linked())]).expect("linked");
    assert_eq!(crate::decode_forges(&writer.finish(), &limits), None);
    for numbers in [&[2, 1][..], &[1, 1][..]] {
        let record = forges_rows(numbers, &v1::Connection::NotSetUp);
        assert_eq!(crate::decode_forges(&crate::forges(&record), &limits), None);
    }
}

#[test]
fn forge_login_and_stop_reason_round_trip_at_schema_ceilings() {
    let limits = v1::CEILINGS;
    for count in [1024, 1025] {
        let connected =
            v1::Connected::new(&limits, v1::ConnectedParts { id: u64::MAX, login: fixtures::field_bytes(count) });
        let stopped = v1::Stopped::new(&limits, v1::StoppedParts { reason: fixtures::field_bytes(count) });
        if count == 1025 {
            connected.expect_err("login past schema ceiling");
            stopped.expect_err("reason past schema ceiling");
            continue;
        }
        let connected = connected.expect("login ceiling");
        let stopped = stopped.expect("reason ceiling");
        for kind in [v1::Kind::Forgejo, v1::Kind::Github] {
            for mode in [v1::Mode::Connected, v1::Mode::Managed, v1::Mode::Owned, v1::Mode::Fake] {
                for connection in
                    [v1::Connection::Connected(connected.clone()), v1::Connection::Stopped(stopped.clone())]
                {
                    let row = v1::ForgeRow::new(
                        &limits,
                        v1::ForgeRowParts {
                            starts_with: v1::StartsWith::Repository,
                            makings: List::with_capacity(0),
                            host: b"git.example:3000".as_slice().into(),
                            number: u16::MAX,
                            kind: kind.clone(),
                            mode: mode.clone(),
                            connection,
                            offer: v1::Offer::NotOffered,
                            first: None,
                            installations: List::with_capacity(0),
                            ended: None,
                            app: None,
                            web: None,
                            repositories: List::with_capacity(0),
                            owned: None,
                            admin: false,
                            read_only: false,
                            ended_stop: None,
                        },
                    )
                    .expect("forge row");
                    let mut forges = List::with_capacity(1);
                    forges.push(row).expect("one forge");
                    let record = v1::Forges::new(&limits, v1::ForgesParts { forges, linked: false }).expect("forges");
                    assert_eq!(crate::decode_forges(&crate::forges(&record), &limits), Some(record));
                }
            }
        }
    }
}

#[test]
fn playground_start_outcomes_round_trip_at_each_boundary() {
    let limits = v1::CEILINGS;
    let mut outcomes = List::with_capacity(8);
    for outcome in [
        v1::Outcome::Running,
        v1::Outcome::StartRefused(
            v1::StartRefused::new(&limits, v1::StartRefusedParts { why: v1::StartRefusal::NotOffered })
                .expect("refusal"),
        ),
    ] {
        outcomes.push(outcome).expect("outcome room");
    }
    for why in [v1::StartFailure::Account, v1::StartFailure::Adoption, v1::StartFailure::NotConnected] {
        outcomes
            .push(v1::Outcome::StartFailed(
                v1::StartFailed::new(&limits, v1::StartFailedParts { why }).expect("failure"),
            ))
            .expect("outcome room");
    }
    for length in [0, 1, 256] {
        let group = v1::GroupFailed::new(&limits, v1::GroupFailedParts { why: skein_lib::bytes::zeroed(length) })
            .expect("bounded why");
        outcomes
            .push(v1::Outcome::StartFailed(
                v1::StartFailed::new(&limits, v1::StartFailedParts { why: v1::StartFailure::Group(group) })
                    .expect("group failure"),
            ))
            .expect("outcome room");
    }
    for outcome in outcomes.into_boxed() {
        let record = v1::Answer::new(&limits, v1::AnswerParts { outcome }).expect("answer");
        let bytes = crate::answer(&record);
        let mut reader = Reader::new(&bytes);
        assert_eq!(v1::Answer::decode(&limits, &mut reader), Ok(record));
        assert!(reader.is_empty());
    }
    v1::GroupFailed::new(&limits, v1::GroupFailedParts { why: skein_lib::bytes::zeroed(257) })
        .expect_err("past ceiling");
    let group =
        v1::GroupFailed::new(&limits, v1::GroupFailedParts { why: skein_lib::bytes::zeroed(256) }).expect("ceiling");
    let record = v1::Answer::new(
        &limits,
        v1::AnswerParts {
            outcome: v1::Outcome::StartFailed(
                v1::StartFailed::new(&limits, v1::StartFailedParts { why: v1::StartFailure::Group(group) })
                    .expect("failure"),
            ),
        },
    )
    .expect("answer");
    let bytes = crate::answer(&record);
    // Version (2), Outcome (1), StartFailure (1), then WHY's u32 length.
    let mut writer = skein_lib::Writer::new(bytes.len().checked_add(1).expect("extra byte"));
    writer.put(&bytes[..4]).expect("header");
    writer.put(&257_u32.to_be_bytes()).expect("length");
    writer.put(&bytes[8..]).expect("why");
    writer.put(b"x").expect("extra byte");
    let bytes = writer.finish();
    v1::Answer::decode(&limits, &mut Reader::new(&bytes)).expect_err("past ceiling on decode");
}

#[test]
fn pasted_apps_round_trip_and_check_nonzero_id() {
    let limits = fixtures::limits();
    for (id, read_only) in [(0, None), (1, None), (1, Some(false)), (1, Some(true)), (u64::MAX, Some(true))] {
        let app = v1::GitHubApp::new(&limits, v1::GitHubAppParts { id, read_only }).unwrap();
        let connect =
            v1::Connect::new(&limits, v1::ConnectParts { forge: kept(1), how: v1::How::GithubApp(app) }).unwrap();
        let record =
            v1::Requesting::new(&limits, v1::RequestingParts { request: v1::Request::Connect(connect) }).unwrap();
        let decoded = crate::decode_requesting(&crate::requesting(&record), &limits);
        assert_eq!(decoded, if id == 0 { None } else { Some(record) });
    }
}

#[test]
fn connect_outcomes_round_trip() {
    let limits = fixtures::limits();
    let mut outcomes = List::with_capacity(10);
    for outcome in [
        v1::Outcome::Connecting,
        v1::Outcome::Link(
            v1::Link::new(&limits, v1::LinkParts { address: Box::from(b"https://forge.test/install".as_slice()) })
                .unwrap(),
        ),
    ] {
        outcomes.push(outcome).unwrap();
    }
    for why in [
        v1::ForgeRefusal::Role,
        v1::ForgeRefusal::NoSuchForge,
        v1::ForgeRefusal::NotTaken,
        v1::ForgeRefusal::Connected,
        v1::ForgeRefusal::NotConnected,
        v1::ForgeRefusal::Busy,
        v1::ForgeRefusal::NoKey,
        v1::ForgeRefusal::FirstIsBot,
    ] {
        outcomes
            .push(v1::Outcome::ConnectRefused(
                v1::ConnectRefused::new(&limits, v1::ConnectRefusedParts { why }).unwrap(),
            ))
            .unwrap();
    }
    for outcome in outcomes.into_boxed() {
        let record = v1::Answer::new(&limits, v1::AnswerParts { outcome }).unwrap();
        assert_eq!(v1::Answer::decode(&limits, &mut Reader::new(&crate::answer(&record))).unwrap(), record);
    }
}

// client/forge/records.md: at most 64 installations and a 256-byte end.
const INSTALLATIONS: u32 = 64;
const CONNECT_WHY: usize = 256;
fn installation_row(count: u32, why_bytes: usize) -> v1::ForgeRowParts {
    let mut installations = List::with_capacity(count);
    for id in 0..count {
        installations
            .push(
                v1::Installation::new(
                    &v1::CEILINGS,
                    v1::InstallationParts {
                        id: u64::from(id),
                        account: Some(
                            v1::Account::new(
                                &v1::CEILINGS,
                                v1::AccountParts { id: 42, login: Box::from(b"acme".as_slice()) },
                            )
                            .unwrap(),
                        ),
                        suspended: true,
                        stray: false,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    v1::ForgeRowParts {
        starts_with: v1::StartsWith::Repository,
        makings: List::with_capacity(0),
        host: b"git.example:3000".as_slice().into(),
        number: 1,
        kind: v1::Kind::Github,
        mode: v1::Mode::Connected,
        connection: v1::Connection::NotConnected,
        offer: v1::Offer::NotOffered,
        first: None,
        installations,
        ended: Some(
            v1::ConnectStopped::new(&v1::CEILINGS, v1::ConnectStoppedParts { why: filled(why_bytes) }).unwrap(),
        ),
        app: None,
        web: None,
        repositories: List::with_capacity(0),
        owned: None,
        admin: false,
        read_only: false,
        ended_stop: None,
    }
}
// Encode the fields explicitly to feed invalid lengths to the receiving boundary.
fn installation_row_bytes(parts: v1::ForgeRowParts, why: &[u8]) -> Box<[u8]> {
    // Number (2), host length (4) and bytes, four enum tags, first tag, starts-with tag, list length (4), end tag, why length (4),
    // app tag, web tag, repositories length (4), owned tag, admin bool, typed stop tag, read-only bool,
    // makings length (4).
    const ROW_FRAMING_BYTES: usize = 35;
    let mut size = why.len().checked_add(ROW_FRAMING_BYTES).unwrap().checked_add(parts.host.len()).unwrap();
    for installation in &parts.installations {
        size = size.checked_add(usize::try_from(installation.measure()).unwrap()).unwrap();
    }
    let mut writer = skein_lib::Writer::new(size);
    writer.put(&parts.number.to_be_bytes()).unwrap();
    writer.put(&u32::try_from(parts.host.len()).unwrap().to_be_bytes()).unwrap();
    writer.put(&parts.host).unwrap();
    parts.kind.encode(&mut writer).unwrap();
    parts.mode.encode(&mut writer).unwrap();
    parts.connection.encode(&mut writer).unwrap();
    parts.offer.encode(&mut writer).unwrap();
    writer.put(&[0]).unwrap(); // no first repository
    parts.starts_with.encode(&mut writer).unwrap();
    writer.put(&parts.installations.len().to_be_bytes()).unwrap();
    for installation in &parts.installations {
        installation.encode(&mut writer).unwrap();
    }
    writer.put(&[1]).unwrap(); // an end
    writer.put(&u32::try_from(why.len()).unwrap().to_be_bytes()).unwrap();
    writer.put(why).unwrap();
    writer.put(&[0]).unwrap(); // no App way
    writer.put(&[0]).unwrap(); // no web address
    writer.put(&0_u32.to_be_bytes()).unwrap(); // no adoptions
    writer.put(&[0]).unwrap(); // no owned details
    writer.put(&[0, 0, 0]).unwrap(); // admin false, no typed stop, read-only false
    writer.put(&0_u32.to_be_bytes()).unwrap(); // no repository makings
    writer.finish()
}
#[test]
fn installation_rows_round_trip_and_refuse_lengths_past_the_ceilings() {
    let limits = v1::CEILINGS;
    for count in [0, INSTALLATIONS] {
        let row = v1::ForgeRow::new(&limits, installation_row(count, CONNECT_WHY)).unwrap();
        let mut writer = skein_lib::Writer::new(usize::try_from(row.measure()).unwrap());
        row.encode(&mut writer).unwrap();
        assert_eq!(v1::ForgeRow::decode(&limits, &mut Reader::new(&writer.finish())), Ok(row));
    }
    for (count, why_bytes, accepted) in [
        (INSTALLATIONS, CONNECT_WHY, true),
        (INSTALLATIONS + 1, CONNECT_WHY, false),
        (INSTALLATIONS, CONNECT_WHY + 1, false),
    ] {
        let parts = installation_row(count, 0);
        let bytes = installation_row_bytes(parts, &filled(why_bytes));
        assert_eq!(v1::ForgeRow::decode(&limits, &mut Reader::new(&bytes)).is_ok(), accepted);
    }
    v1::ForgeRow::new(&limits, installation_row(INSTALLATIONS + 1, 0)).unwrap_err();
    v1::ConnectStopped::new(&limits, v1::ConnectStoppedParts { why: filled(CONNECT_WHY + 1) }).unwrap_err();
}
fn filled(bytes: usize) -> Box<[u8]> {
    let mut writer = skein_lib::Writer::new(bytes);
    for _ in 0..bytes {
        writer.put(b"x").unwrap();
    }
    writer.finish()
}

#[test]
fn manifest_connects_and_app_ways_round_trip() {
    let limits = fixtures::limits();
    for back in [None, Some(Box::from(b"/forges/connect?forge=1".as_slice()))] {
        let how = v1::GitHubManifest::new(&limits, v1::GitHubManifestParts { back }).unwrap();
        let connect =
            v1::Connect::new(&limits, v1::ConnectParts { forge: kept(1), how: v1::How::GithubManifest(how) }).unwrap();
        let record =
            v1::Requesting::new(&limits, v1::RequestingParts { request: v1::Request::Connect(connect) }).unwrap();
        assert_eq!(crate::decode_requesting(&crate::requesting(&record), &limits), Some(record));
    }
    for app in [None, Some(v1::AppWay::Manifest), Some(v1::AppWay::Pasted)] {
        let mut parts = installation_row(0, 0);
        parts.app = app;
        let row = v1::ForgeRow::new(&limits, parts).unwrap();
        let mut writer = skein_lib::Writer::new(usize::try_from(row.measure()).unwrap());
        row.encode(&mut writer).unwrap();
        assert_eq!(v1::ForgeRow::decode(&limits, &mut Reader::new(&writer.finish())), Ok(row));
    }
}

mod forgejo_admin;
mod forgejo_tokens;

#[test]
fn setup_failure_round_trips_each_step_and_none() {
    let limits = fixtures::limits();
    for at in [
        None,
        Some(v1::FailedStep::Fetch),
        Some(v1::FailedStep::Prepare),
        Some(v1::FailedStep::Run),
        Some(v1::FailedStep::Connect),
    ] {
        let value =
            v1::SetUpFailed::new(&limits, v1::SetUpFailedParts { why: b"stopped".as_slice().into(), at }).unwrap();
        let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).unwrap());
        value.encode(&mut writer).unwrap();
        let bytes = writer.finish();
        let mut reader = Reader::new(&bytes);
        assert_eq!(v1::SetUpFailed::decode(&limits, &mut reader).unwrap(), value);
        assert!(reader.is_empty());
    }
}

#[test]
fn admin_rows_round_trip_the_form_stop_and_first_label() {
    let limits = v1::CEILINGS;
    let mut stops = List::with_capacity(13);
    for stop in [None, Some(v1::AdminStop::NotAdmin), Some(v1::AdminStop::Unmarked), Some(v1::AdminStop::Other)] {
        stops.push(stop).unwrap();
    }
    for stage in [
        v1::SetupStage::Admin,
        v1::SetupStage::Bot,
        v1::SetupStage::Token,
        v1::SetupStage::Org,
        v1::SetupStage::Repository,
        v1::SetupStage::File,
        v1::SetupStage::App,
        v1::SetupStage::Installation,
        v1::SetupStage::State,
    ] {
        stops
            .push(Some(v1::AdminStop::Refused(
                v1::AdminRefused::new(&limits, v1::AdminRefusedParts { status: 403, stage }).unwrap(),
            )))
            .unwrap();
    }
    for stop in stops.into_boxed() {
        for admin in [false, true] {
            let mut parts = installation_row(0, 0);
            parts.admin = admin;
            parts.ended_stop = stop.clone();
            parts.first = Some(first_with_label(filled(64)).unwrap());
            let row = v1::ForgeRow::new(&limits, parts).unwrap();
            let mut writer = skein_lib::Writer::new(usize::try_from(row.measure()).unwrap());
            row.encode(&mut writer).unwrap();
            assert_eq!(v1::ForgeRow::decode(&limits, &mut Reader::new(&writer.finish())), Ok(row));
        }
    }
}
fn first_with_label(label: Box<[u8]>) -> Result<v1::FirstRepository, v1::Problem> {
    v1::FirstRepository::new(
        &v1::CEILINGS,
        v1::FirstRepositoryParts {
            playground: true,
            org: b"work".as_slice().into(),
            repository: b"hello".as_slice().into(),
            state: v1::FirstState::Making,
            label,
        },
    )
}
#[test]
fn first_label_accepts_64_bytes_and_refuses_one_more() {
    first_with_label(filled(64)).unwrap();
    assert_eq!(
        first_with_label(filled(65)),
        Err(v1::Problem { path: v1::Path::FirstRepositoryLabel, reason: skein_codec::Reason::Bound })
    );
}

#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire goldens are test inputs")]
fn landing_panels_round_trip_smallest_and_full_goldens() {
    for bytes in [
        include_bytes!("../golden/record_landing_panel_smallest.bin").as_slice(),
        include_bytes!("../golden/record_landing_panel_full.bin").as_slice(),
    ] {
        let panel = crate::decode_landing_panel(bytes, &v1::CEILINGS).expect("landing golden");
        assert_eq!(crate::landing_panel(&panel).as_ref(), bytes);
        assert_eq!(crate::decode_landing_panel(&fixtures::trailing(bytes), &v1::CEILINGS), None);
    }
}
#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire goldens are test inputs")]
fn landings_panels_round_trip_smallest_and_full_goldens() {
    for bytes in [
        include_bytes!("../golden/record_landings_panel_smallest.bin").as_slice(),
        include_bytes!("../golden/record_landings_panel_full.bin").as_slice(),
    ] {
        let panel = crate::decode_landings_panel(bytes, &v1::CEILINGS).expect("landings golden");
        assert_eq!(crate::landings_panel(&panel).as_ref(), bytes);
        assert_eq!(crate::decode_landings_panel(&fixtures::trailing(bytes), &v1::CEILINGS), None);
    }
}
#[test]
fn landing_watches_round_trip_without_trailing_bytes() {
    let limits = fixtures::limits();
    for watch in [
        v1::Watch::Landing(v1::LandingSubject::new(&limits, v1::LandingSubjectParts { task: 61 }).expect("task")),
        v1::Watch::Landings(
            v1::LandingsSubject::new(&limits, v1::LandingsSubjectParts { project: 1, ended: true }).expect("project"),
        ),
    ] {
        let watching = v1::Watching::new(&limits, v1::WatchingParts { watch }).expect("watch");
        let bytes = crate::watching(&watching);
        assert_eq!(crate::decode_watching(&bytes, &limits), Some(watching));
        assert_eq!(crate::decode_watching(&fixtures::trailing(&bytes), &limits), None);
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire goldens are test inputs")]
fn changes_panel_round_trips() {
    for bytes in [
        include_bytes!("../golden/record_changes_panel_smallest.bin").as_slice(),
        include_bytes!("../golden/record_changes_panel_full.bin").as_slice(),
    ] {
        let panel = crate::decode_changes_panel(bytes, &v1::CEILINGS).expect("changes golden");
        assert_eq!(crate::changes_panel(&panel).as_ref(), bytes);
        assert_eq!(crate::decode_changes_panel(&fixtures::trailing(bytes), &v1::CEILINGS), None);
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire goldens are test inputs")]
fn ways_panel_round_trips() {
    let limits = fixtures::limits();
    let subject = v1::WaysSubject::new(&limits, v1::WaysSubjectParts { project: 1 }).expect("project");
    let watch = v1::Watching::new(&limits, v1::WatchingParts { watch: v1::Watch::Ways(subject) }).expect("watch");
    let bytes = crate::watching(&watch);
    assert_eq!(crate::decode_watching(&bytes, &limits), Some(watch));
    assert_eq!(crate::decode_watching(&fixtures::trailing(&bytes), &limits), None);
    for parts in [
        v1::ProjectWordsParts { headline: b"h".as_slice().into(), lede: b"l".as_slice().into() },
        v1::ProjectWordsParts { headline: Box::new([b'h'; 120]), lede: Box::new([b'l'; 480]) },
    ] {
        let expected_size = parts.headline.len() + parts.lede.len() + 8;
        let words = v1::ProjectWords::new(&limits, parts).expect("project words within their byte bounds");
        let mut writer = skein_lib::Writer::new(usize::try_from(words.measure()).expect("words size"));
        words.encode(&mut writer).expect("measured words fit");
        let bytes = writer.finish();
        assert_eq!(bytes.len(), expected_size, "two text lengths and their bytes");
        let mut reader = Reader::new(&bytes);
        assert_eq!(v1::ProjectWords::decode(&limits, &mut reader), Ok(words.clone()));
        assert!(reader.is_empty());
        let panel = crate::decode_ways_panel(include_bytes!("../golden/record_ways_panel_smallest.bin"), &limits)
            .expect("smallest ways panel");
        let panel = v1::WaysPanel::new(&limits, v1::WaysPanelParts { words: Some(words), ..panel.into_parts() })
            .expect("ways panel with project words");
        let bytes = crate::ways_panel(&panel);
        assert_eq!(crate::decode_ways_panel(&bytes, &limits), Some(panel));
        assert_eq!(crate::decode_ways_panel(&fixtures::trailing(&bytes), &limits), None);
    }
    for bytes in [
        include_bytes!("../golden/record_ways_panel_smallest.bin").as_slice(),
        include_bytes!("../golden/record_ways_panel_full.bin").as_slice(),
    ] {
        let record = crate::decode_ways_panel(bytes, &v1::CEILINGS).expect("ways_panel golden");
        assert_eq!(crate::ways_panel(&record).as_ref(), bytes);
        assert_eq!(crate::decode_ways_panel(&fixtures::trailing(bytes), &v1::CEILINGS), None);
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire goldens are test inputs")]
fn view_answer_round_trips() {
    let limits = fixtures::limits();
    let view = v1::ViewRead::new(&limits, v1::ViewReadParts { project: 1, address: b"way Quick".as_slice().into() })
        .expect("view address");
    let read = v1::Reading::new(&limits, v1::ReadingParts { read: v1::Read::View(view) }).expect("read");
    let bytes = crate::reading(&read);
    assert_eq!(crate::decode_reading(&bytes, &limits), Some(read));
    assert_eq!(crate::decode_reading(&fixtures::trailing(&bytes), &limits), None);
    for bytes in [
        include_bytes!("../golden/record_view_answer_smallest.bin").as_slice(),
        include_bytes!("../golden/record_view_answer_full.bin").as_slice(),
    ] {
        let record = crate::decode_view_answer(bytes, &v1::CEILINGS).expect("view_answer golden");
        assert_eq!(crate::view_answer(&record).as_ref(), bytes);
        assert_eq!(crate::decode_view_answer(&fixtures::trailing(bytes), &v1::CEILINGS), None);
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire goldens are test inputs")]
fn project_status_round_trips_and_refuses_trailing_bytes() {
    for bytes in [
        include_bytes!("../golden/record_project_status_smallest.bin").as_slice(),
        include_bytes!("../golden/record_project_status_full.bin").as_slice(),
    ] {
        let status = crate::decode_project_status(bytes, &v1::CEILINGS).expect("project status golden");
        assert_eq!(crate::project_status(&status).as_ref(), bytes);
        assert_eq!(crate::decode_project_status(&fixtures::trailing(bytes), &v1::CEILINGS), None);
    }
}

#[test]
fn watching_changes_round_trips() {
    let limits = fixtures::limits();
    let subject = v1::ChangesSubject::new(&limits, v1::ChangesSubjectParts { project: 1 }).expect("project");
    let watching = v1::Watching::new(&limits, v1::WatchingParts { watch: v1::Watch::Changes(subject) }).expect("watch");
    let bytes = crate::watching(&watching);
    assert_eq!(crate::decode_watching(&bytes, &limits), Some(watching));
    assert_eq!(crate::decode_watching(&fixtures::trailing(&bytes), &limits), None);
}

fn kept(forge: u16) -> v1::Making {
    v1::Making::Kept(v1::Kept::new(&v1::CEILINGS, v1::KeptParts { forge }).unwrap())
}
fn making_new(forge: v1::ForgeAsk) -> v1::Making {
    v1::Making::New(v1::NewForge::new(&v1::CEILINGS, v1::NewForgeParts { forge }).unwrap())
}
fn managed(host: &[u8], port: u16) -> v1::ForgeAsk {
    v1::ForgeAsk::Managed(
        v1::ManagedForge::new(
            &v1::CEILINGS,
            v1::ManagedForgeParts {
                git_program: None,
                host: host.into(),
                port,
                tls: Some(filled(255)),
                git: filled(2048),
                web: Some(filled(2048)),
            },
        )
        .unwrap(),
    )
}
fn making_requests(forge: v1::Making) -> [v1::Request; 2] {
    let limits = fixtures::limits();
    [
        v1::Request::SetUp(
            v1::SetUp::new(
                &limits,
                v1::SetUpParts {
                    forge: forge.clone(),
                    first: v1::SetUpFirst::Empty(
                        v1::FirstEmpty::new(
                            &v1::CEILINGS,
                            v1::FirstEmptyParts {
                                org: b"work".as_slice().into(),
                                repository: b"hello".as_slice().into(),
                                playground: false,
                            },
                        )
                        .expect("setup names fit wire ceilings"),
                    ),
                },
            )
            .unwrap(),
        ),
        v1::Request::Connect(
            v1::Connect::new(
                &limits,
                v1::ConnectParts {
                    forge,
                    how: v1::How::ForgejoAdmin(
                        v1::ForgejoAdmin::new(&limits, v1::ForgejoAdminParts { first: None }).unwrap(),
                    ),
                },
            )
            .unwrap(),
        ),
    ]
}
#[test]
fn making_round_trips_kept_and_new() {
    let limits = fixtures::limits();
    for making in [
        kept(u16::MAX),
        making_new(v1::ForgeAsk::Github),
        making_new(v1::ForgeAsk::Owned),
        making_new(managed(&[b'h'; 255], u16::MAX)),
    ] {
        for request in making_requests(making) {
            let record = v1::Requesting::new(&limits, v1::RequestingParts { request }).unwrap();
            let bytes = crate::requesting(&record);
            assert_eq!(crate::decode_requesting(&bytes, &limits), Some(record));
            assert_eq!(crate::decode_requesting(&fixtures::trailing(&bytes), &limits), None);
        }
    }
}
#[test]
fn a_managed_forge_with_an_empty_host_is_refused() {
    let limits = fixtures::limits();
    for (host, port) in [(b"".as_slice(), 443), (b"a\0b", 443), (b"a/b", 443), (b"host", 0)] {
        for request in making_requests(making_new(managed(host, port))) {
            let record = v1::Requesting::new(&limits, v1::RequestingParts { request }).unwrap();
            assert_eq!(crate::decode_requesting(&crate::requesting(&record), &limits), None);
        }
    }
}
#[test]
fn remove_and_playground_requests_round_trip() {
    let limits = fixtures::limits();
    for request in [
        v1::Request::RemoveForge(v1::RemoveForge::new(&limits, v1::RemoveForgeParts { forge: u16::MAX }).unwrap()),
        v1::Request::StartPlayground,
        v1::Request::EndPlayground,
    ] {
        let record = v1::Requesting::new(&limits, v1::RequestingParts { request }).unwrap();
        let bytes = crate::requesting(&record);
        assert_eq!(crate::decode_requesting(&bytes, &limits), Some(record));
        assert_eq!(crate::decode_requesting(&fixtures::trailing(&bytes), &limits), None);
    }
    for outcome in [
        v1::Outcome::Removed,
        v1::Outcome::RemoveRefused(
            v1::RemoveRefused::new(&limits, v1::RemoveRefusedParts { why: v1::ForgeRefusal::Busy }).unwrap(),
        ),
        v1::Outcome::Reset,
        v1::Outcome::EndRefused(
            v1::EndRefused::new(&limits, v1::EndRefusedParts { why: v1::EndRefusal::NotOffered }).unwrap(),
        ),
    ] {
        let answer = v1::Answer::new(&limits, v1::AnswerParts { outcome }).unwrap();
        assert_eq!(v1::Answer::decode(&limits, &mut Reader::new(&crate::answer(&answer))), Ok(answer));
    }
    let read = v1::Reading::new(&limits, v1::ReadingParts { read: v1::Read::Playground }).unwrap();
    assert_eq!(crate::decode_reading(&crate::reading(&read), &limits), Some(read));
}
#[test]
fn revision_setting_read_appends_after_the_deployment_tag() {
    let limits = fixtures::limits();
    let revision = v1::Read::RevisionSetting(
        v1::RevisionSettingRead::new(
            &limits,
            v1::RevisionSettingReadParts { kind: 1, scope: 2, number: 3, revision: 4 },
        )
        .unwrap(),
    );
    for (read, bytes) in [
        (v1::Read::Deployment, &[0, 16, 3][..]),
        (revision, &[0, 16, 4, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4][..]),
    ] {
        let record = v1::Reading::new(&limits, v1::ReadingParts { read }).unwrap();
        assert_eq!(crate::reading(&record).as_ref(), bytes);
        assert_eq!(crate::decode_reading(bytes, &limits), Some(record));
    }
}

#[test]
fn deployment_read_and_answer_round_trip() {
    let limits = fixtures::limits();
    let read = v1::Reading::new(&limits, v1::ReadingParts { read: v1::Read::Deployment }).unwrap();
    let bytes = crate::reading(&read);
    assert_eq!(crate::decode_reading(&bytes, &limits), Some(read));
    for (welcome, owned) in [(false, false), (false, true), (true, false), (true, true)] {
        let answer =
            v1::DeploymentAnswer::new(&limits, v1::DeploymentAnswerParts { welcome, owned, program_login: None })
                .unwrap();
        let bytes = crate::deployment_answer(&answer);
        assert_eq!(crate::decode_deployment_answer(&bytes, &limits), Some(answer));
        assert_eq!(crate::decode_deployment_answer(&fixtures::trailing(&bytes), &limits), None);
    }
}

#[test]
fn program_environment_read_and_answer_round_trip() {
    let limits = v1::CEILINGS;
    let read = v1::Reading::new(&limits, v1::ReadingParts { read: v1::Read::ProgramEnvironment }).unwrap();
    let bytes = crate::reading(&read);
    assert_eq!(crate::decode_reading(&bytes, &limits), Some(read));
    for words in [b"".as_slice(), b"TEMPER_CONFIG='/config' TEMPER_DATA='/data' TEMPER_CACHE='/cache'"] {
        let answer =
            v1::ProgramEnvironmentAnswer::new(&limits, v1::ProgramEnvironmentAnswerParts { environment: words.into() })
                .unwrap();
        let bytes = crate::program_environment_answer(&answer);
        assert_eq!(crate::decode_program_environment_answer(&bytes, &limits), Some(answer));
        assert_eq!(crate::decode_program_environment_answer(&fixtures::trailing(&bytes), &limits), None);
    }
}

#[test]
fn playground_answer_round_trips_at_its_bounds() {
    let limits = fixtures::limits();
    for state in [v1::PlaygroundState::Starting, v1::PlaygroundState::Running, v1::PlaygroundState::Resetting] {
        for project in [None, Some(u32::MAX)] {
            for (count, cue_bytes, field_bytes, accepted) in [
                (0, 1, 0, true),
                (crate::CUES, 256, 16, true),
                (crate::CUES + 1, 1, 16, false),
                (1, 257, 16, false),
                (1, 1, 17, false),
            ] {
                let mut cues = List::with_capacity(count);
                for _ in 0..count {
                    cues.push(filled(cue_bytes)).unwrap();
                }
                let made = v1::PlaygroundAnswer::new(
                    &limits,
                    v1::PlaygroundAnswerParts {
                        forge: u16::MAX,
                        project,
                        provider: u32::MAX,
                        owner: filled(field_bytes),
                        name: filled(field_bytes),
                        state: state.clone(),
                        cues,
                    },
                );
                assert_eq!(made.is_ok(), accepted);
                if let Ok(answer) = made {
                    let bytes = crate::playground_answer(&answer);
                    assert_eq!(crate::decode_playground_answer(&bytes, &limits), Some(answer));
                    assert_eq!(crate::decode_playground_answer(&fixtures::trailing(&bytes), &limits), None);
                }
            }
        }
    }
    // Decode enforces the receiving side's field bounds too.
    for (owner_bytes, name_bytes) in [(17, 16), (16, 17)] {
        let answer = v1::PlaygroundAnswer::new(
            &v1::CEILINGS,
            v1::PlaygroundAnswerParts {
                forge: 1,
                project: None,
                provider: 3,
                owner: filled(owner_bytes),
                name: filled(name_bytes),
                state: v1::PlaygroundState::Starting,
                cues: List::with_capacity(0),
            },
        )
        .unwrap();
        assert_eq!(crate::decode_playground_answer(&crate::playground_answer(&answer), &limits), None);
    }
    for cue in [b"".as_slice(), &[255]] {
        let mut cues = List::with_capacity(1);
        cues.push(cue.into()).unwrap();
        let made = v1::PlaygroundAnswer::new(
            &limits,
            v1::PlaygroundAnswerParts {
                forge: 1,
                project: None,
                provider: 3,
                owner: b"work".as_slice().into(),
                name: b"hello".as_slice().into(),
                state: v1::PlaygroundState::Running,
                cues,
            },
        );
        if let Ok(answer) = made {
            assert_eq!(crate::decode_playground_answer(&crate::playground_answer(&answer), &limits), None);
        } else {
            assert_eq!(cue, &[255]);
        }
    }
}

#[test]
fn managed_git_program_is_optional_bounded_and_nonempty() {
    let limits = fixtures::limits();
    for length in [0_usize, 4096, 4097] {
        let v1::ForgeAsk::Managed(value) = managed(b"host", 80) else { unreachable!() };
        let mut parts = value.into_parts();
        parts.git_program = Some(filled(length));
        let value = v1::ManagedForge::new(&limits, parts);
        if length > 4096 {
            assert!(value.is_err());
            continue;
        }
        for request in making_requests(making_new(v1::ForgeAsk::Managed(value.unwrap()))) {
            let record = v1::Requesting::new(&limits, v1::RequestingParts { request }).unwrap();
            assert_eq!(crate::decode_requesting(&crate::requesting(&record), &limits).is_some(), length != 0);
        }
    }
}

#[test]
fn post_landing_tiers_accept_their_bounds_and_refuse_one_more() {
    let limits = fixtures::limits();
    for (count, bytes) in [(32, 8192), (33, 8192), (32, 8193)] {
        let mut tiers = List::with_capacity(count);
        for _ in 0..count {
            tiers.push(fixtures::field_bytes(bytes)).unwrap();
        }
        let verifying = v1::Verifying::new(&limits, v1::VerifyingParts { merge: [8; 32], tiers: tiers.clone() });
        let red = v1::Red::new(&limits, v1::RedParts { merge: [8; 32], tiers: tiers.clone() });
        let cleared = v1::Cleared::new(&limits, v1::ClearedParts { merge: [8; 32], tiers, uncleared: false });
        let within = count == 32 && bytes == 8192;
        assert_eq!(verifying.is_ok(), within);
        assert_eq!(red.is_ok(), within);
        assert_eq!(cleared.is_ok(), within);
    }
}

#[test]
fn tier_strings_round_trip_at_the_bound_and_refuse_one_more_byte() {
    for size in [8192_usize, 8193] {
        let mut tiers = List::with_capacity(1);
        let line: Box<[u8]> = if size == 8192 { Box::new([b'x'; 8192]) } else { Box::new([b'x'; 8193]) };
        tiers.push(line).unwrap();
        let value = v1::Red::new(&v1::CEILINGS, v1::RedParts { merge: [3; 32], tiers });
        if size == 8192 {
            let value = value.unwrap();
            let mut writer = skein_lib::Writer::new(usize::try_from(value.measure()).unwrap());
            value.encode(&mut writer).unwrap();
            let bytes = writer.finish();
            assert_eq!(v1::Red::decode(&v1::CEILINGS, &mut Reader::new(&bytes)).unwrap(), value);
        } else {
            assert_eq!(value.unwrap_err().reason, skein_codec::Reason::Bound);
        }
    }
}

#[test]
fn settings_reads_and_answers_require_one_complete_bounded_record() {
    use crate::{self as wire, v1 as w};
    for read in [
        w::Read::Setting(
            w::SettingRead::new(&w::CEILINGS, w::SettingReadParts { kind: 2, scope: 7, number: 1 }).unwrap(),
        ),
        w::Read::ProposalItems(
            w::ProposalItemsRead::new(&w::CEILINGS, w::ProposalItemsReadParts { key: u64::MAX }).unwrap(),
        ),
    ] {
        let record = w::Reading::new(&w::CEILINGS, w::ReadingParts { read }).unwrap();
        let bytes = wire::reading(&record);
        assert_eq!(wire::decode_reading(&bytes, &w::CEILINGS), Some(record));
        assert!(wire::decode_reading(&bytes[..bytes.len() - 1], &w::CEILINGS).is_none());
    }
    let record =
        w::SettingAnswer::new(&w::CEILINGS, w::SettingAnswerParts { value: Some(Box::new([6, 0, 0, 0])) }).unwrap();
    let bytes = wire::setting_answer(&record);
    assert_eq!(wire::decode_setting_answer(&bytes, &w::CEILINGS), Some(record));
    let mut limits = w::CEILINGS;
    limits.setting_answer_value_some = 3;
    assert!(wire::decode_setting_answer(&bytes, &limits).is_none());
    let mut trailing = skein_lib::Writer::new(bytes.len().checked_add(1).unwrap());
    trailing.put(&bytes).unwrap();
    trailing.put(&[0]).unwrap();
    let trailing = trailing.finish();
    assert!(wire::decode_setting_answer(&trailing, &w::CEILINGS).is_none());
    let mut items = List::with_capacity(1);
    items
        .push(
            w::ProposalItem::new(
                &w::CEILINGS,
                w::ProposalItemParts { kind: 1, scope: 0, number: 1, edit: w::SettingEdit::Reset },
            )
            .unwrap(),
        )
        .unwrap();
    let record =
        w::ProposalItemsAnswer::new(&w::CEILINGS, w::ProposalItemsAnswerParts { items, owned_approval: true }).unwrap();
    let bytes = wire::proposal_items_answer(&record);
    assert_eq!(wire::decode_proposal_items_answer(&bytes, &w::CEILINGS), Some(record));
    limits.proposal_items_answer_items = 0;
    assert!(wire::decode_proposal_items_answer(&bytes, &limits).is_none());
    assert!(wire::decode_proposal_items_answer(&bytes[..bytes.len() - 1], &w::CEILINGS).is_none());
    let mut trailing = skein_lib::Writer::new(bytes.len().checked_add(1).unwrap());
    trailing.put(&bytes).unwrap();
    trailing.put(&[0]).unwrap();
    let trailing = trailing.finish();
    assert!(wire::decode_proposal_items_answer(&trailing, &w::CEILINGS).is_none());
}

#[test]
fn repository_makings_round_trip_at_the_bound_and_refuse_one_more() {
    let limits = fixtures::limits();
    let making = v1::RepositoryMaking::new(
        &limits,
        v1::RepositoryMakingParts {
            org: filled(MAKING_FIELD_BYTES),
            name: filled(MAKING_FIELD_BYTES),
            from: v1::MakeFrom::Copy(
                v1::MakeCopy::new(&limits, v1::MakeCopyParts { address: filled(COPY_ADDRESS_BYTES) }).unwrap(),
            ),
            into: Some(v1::MakeInto::new(&limits, v1::MakeIntoParts { project: u32::MAX, home: true }).unwrap()),
            state: v1::MakingState::Made(
                v1::MakingMade::new(&limits, v1::MakingMadeParts { id: u64::MAX, branch: filled(MAKING_FIELD_BYTES) })
                    .unwrap(),
            ),
        },
    )
    .unwrap();
    for (count, accepted) in [(crate::MAKINGS, true), (crate::MAKINGS + 1, false)] {
        let mut parts = installation_row(0, 0);
        parts.makings = List::with_capacity(count);
        for _ in 0..count {
            parts.makings.push(making.clone()).unwrap();
        }
        let row = v1::ForgeRow::new(&limits, parts);
        assert_eq!(row.is_ok(), accepted);
        if let Ok(row) = row {
            let mut rows = List::with_capacity(1);
            rows.push(row).unwrap();
            let record = v1::Forges::new(&limits, v1::ForgesParts { forges: rows, linked: false }).unwrap();
            let encoded = crate::forges(&record);
            assert_eq!(crate::decode_forges(&encoded, &limits), Some(record));
            let mut smaller = limits;
            smaller.forge_row_makings = crate::MAKINGS - 1;
            assert_eq!(crate::decode_forges(&encoded, &smaller), None);
        }
    }
}

#[test]
fn make_repository_names_and_address_take_their_wire_bounds() {
    let limits = fixtures::limits();
    assert_eq!(
        v1::MakeCopy::new(&limits, v1::MakeCopyParts { address: filled(COPY_ADDRESS_BYTES + 1) }).unwrap_err(),
        v1::Problem { path: v1::Path::MakeCopyAddress, reason: skein_codec::Reason::Bound }
    );
    for (org_bytes, name_bytes, accepted) in [
        (MAKING_FIELD_BYTES, MAKING_FIELD_BYTES, true),
        (MAKING_FIELD_BYTES + 1, MAKING_FIELD_BYTES, false),
        (MAKING_FIELD_BYTES, MAKING_FIELD_BYTES + 1, false),
    ] {
        // Construct at the schema ceilings to check the receiving side as well.
        let copy = v1::MakeCopy::new(&v1::CEILINGS, v1::MakeCopyParts { address: filled(COPY_ADDRESS_BYTES) }).unwrap();
        for from in [v1::MakeFrom::Empty, v1::MakeFrom::Copy(copy)] {
            for into in [None, Some(v1::MakeInto::new(&limits, v1::MakeIntoParts { project: 1, home: true }).unwrap())]
            {
                let parts = v1::MakeRepositoryParts {
                    forge: u16::MAX,
                    org: filled(org_bytes),
                    name: filled(name_bytes),
                    from: from.clone(),
                    into,
                };
                assert_eq!(v1::MakeRepository::new(&limits, parts.clone()).is_ok(), accepted);
                let request = v1::MakeRepository::new(&v1::CEILINGS, parts).unwrap();
                let record = v1::Requesting::new(
                    &v1::CEILINGS,
                    v1::RequestingParts { request: v1::Request::MakeRepository(request) },
                )
                .unwrap();
                let bytes = crate::requesting(&record);
                assert_eq!(crate::decode_requesting(&bytes, &limits).is_some(), accepted);
            }
        }
    }
}

// fixtures::BOUNDS.field_bytes, and T3's schema bound on MakeCopy.address.
const MAKING_FIELD_BYTES: usize = 16;
const COPY_ADDRESS_BYTES: usize = 512;

#[test]
fn dismiss_and_dismissals_round_trip() {
    let limits = fixtures::limits();
    assert_eq!(crate::DISMISSED, 8192, "client/forge/records.md: a party's fact bound");
    for what in [
        v1::PersonFact::KeepAsking(
            v1::KeepAsking::new(
                &limits,
                v1::KeepAskingParts {
                    project: u32::MAX,
                    subject: v1::ClassSubject::Set(
                        v1::ReachSet::new(&limits, v1::ReachSetParts { number: u8::MAX }).unwrap(),
                    ),
                    at: u64::MAX,
                },
            )
            .unwrap(),
        ),
        v1::PersonFact::ToBegin(v1::ToBegin::new(&limits, v1::ToBeginParts { project: u32::MAX }).unwrap()),
        v1::PersonFact::GoalFrom(
            v1::GoalFrom::new(
                &limits,
                v1::GoalFromParts { project: u32::MAX, chat: u64::MAX, goal: u64::MAX, at: u64::MAX },
            )
            .unwrap(),
        ),
    ] {
        let dismiss = v1::Dismiss::new(&limits, v1::DismissParts { what: what.clone() }).unwrap();
        let request =
            v1::Requesting::new(&limits, v1::RequestingParts { request: v1::Request::Dismiss(dismiss) }).unwrap();
        assert_eq!(crate::decode_requesting(&crate::requesting(&request), &limits), Some(request));
        let watch = v1::Watching::new(&limits, v1::WatchingParts { watch: v1::Watch::Dismissed }).unwrap();
        assert_eq!(crate::decode_watching(&crate::watching(&watch), &limits), Some(watch));
        let answer = v1::Answer::new(&limits, v1::AnswerParts { outcome: v1::Outcome::Dismissed }).unwrap();
        assert_eq!(v1::Answer::decode(&limits, &mut Reader::new(&crate::answer(&answer))), Ok(answer));
        for count in [0, crate::DISMISSED, crate::DISMISSED + 1] {
            let mut dismissed = List::with_capacity(count);
            let mut writer =
                skein_lib::Writer::new(6 + usize::try_from(count).unwrap() * usize::try_from(what.measure()).unwrap());
            writer.put(&u16::try_from(crate::CLIENT_VERSION).unwrap().to_be_bytes()).unwrap();
            writer.put(&count.to_be_bytes()).unwrap();
            for _ in 0..count {
                dismissed.push(what.clone()).unwrap();
                what.encode(&mut writer).unwrap();
            }
            let bytes = writer.finish();
            let made = v1::Dismissals::new(&limits, v1::DismissalsParts { dismissed });
            if count > crate::DISMISSED {
                assert_eq!(
                    made.unwrap_err(),
                    v1::Problem { path: v1::Path::DismissalsDismissed, reason: skein_codec::Reason::Bound }
                );
                assert_eq!(crate::decode_dismissals(&bytes, &limits), None);
            } else {
                let record = made.unwrap();
                assert_eq!(crate::dismissals(&record), bytes);
                assert_eq!(crate::decode_dismissals(&bytes, &limits), Some(record));
                assert_eq!(crate::decode_dismissals(&fixtures::trailing(&bytes), &limits), None);
            }
        }
    }
}

#[test]
fn setup_first_choices_round_trip_in_requests_and_rows() {
    let limits = fixtures::limits();
    assert_eq!(v1::CEILINGS.copy_addresses_addresses, crate::FIRST_COPIES);
    for first in [
        v1::SetUpFirst::Empty(
            v1::FirstEmpty::new(
                &limits,
                v1::FirstEmptyParts {
                    org: b"work".as_slice().into(),
                    repository: b"hello".as_slice().into(),
                    playground: true,
                },
            )
            .unwrap(),
        ),
        v1::SetUpFirst::Copies(
            v1::CopyAddresses::new(&limits, v1::CopyAddressesParts { addresses: List::with_capacity(0) }).unwrap(),
        ),
        v1::SetUpFirst::Copies(
            v1::CopyAddresses::new(
                &limits,
                v1::CopyAddressesParts {
                    addresses: {
                        let mut addresses = List::with_capacity(crate::FIRST_COPIES);
                        for _ in 0..crate::FIRST_COPIES {
                            addresses.push(skein_lib::bytes::zeroed(crate::source::ADDRESS_BYTES)).unwrap();
                        }
                        addresses
                    },
                },
            )
            .unwrap(),
        ),
        v1::SetUpFirst::Nothing,
    ] {
        let request = v1::Requesting::new(
            &limits,
            v1::RequestingParts {
                request: v1::Request::SetUp(
                    v1::SetUp::new(&limits, v1::SetUpParts { forge: kept(3), first: first.clone() }).unwrap(),
                ),
            },
        )
        .unwrap();
        assert_eq!(crate::decode_requesting(&crate::requesting(&request), &limits), Some(request));
        let mut row = installation_row(0, 0);
        row.starts_with = match first {
            v1::SetUpFirst::Empty(_) => v1::StartsWith::Repository,
            v1::SetUpFirst::Copies(copies) => v1::StartsWith::Copies(copies),
            v1::SetUpFirst::Nothing => v1::StartsWith::Nothing,
        };
        let mut rows = List::with_capacity(1);
        rows.push(v1::ForgeRow::new(&limits, row).unwrap()).unwrap();
        let record = v1::Forges::new(&limits, v1::ForgesParts { forges: rows, linked: false }).unwrap();
        assert_eq!(crate::decode_forges(&crate::forges(&record), &limits), Some(record));
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire goldens are test inputs")]
fn aways_round_trip_and_refuse_trailing_bytes() {
    for bytes in [
        include_bytes!("../golden/record_aways_smallest.bin").as_slice(),
        include_bytes!("../golden/record_aways_full.bin").as_slice(),
    ] {
        let record = crate::decode_aways(bytes, &v1::CEILINGS).expect("aways golden");
        assert_eq!(crate::aways(&record).as_ref(), bytes);
        assert_eq!(crate::decode_aways(&fixtures::trailing(bytes), &v1::CEILINGS), None);
    }
}

#[test]
#[expect(clippy::disallowed_macros, reason = "committed wire goldens are test inputs")]
fn decisions_round_trip_and_refuse_trailing_bytes() {
    for bytes in [
        include_bytes!("../golden/record_decisions_smallest.bin").as_slice(),
        include_bytes!("../golden/record_decisions_full.bin").as_slice(),
    ] {
        let record = crate::decode_decisions(bytes, &v1::CEILINGS).expect("decisions golden");
        assert_eq!(crate::decisions(&record).as_ref(), bytes);
        assert_eq!(crate::decode_decisions(&fixtures::trailing(bytes), &v1::CEILINGS), None);
    }
}

#[test]
fn away_requests_round_trip() {
    let limits = fixtures::limits();
    for on in [false, true] {
        for until in [None, Some(u64::MAX)] {
            let value = v1::SetAway::new(&limits, v1::SetAwayParts { project: u32::MAX, on, until }).unwrap();
            let request =
                v1::Requesting::new(&limits, v1::RequestingParts { request: v1::Request::SetAway(value) }).unwrap();
            let bytes = crate::requesting(&request);
            assert_eq!(crate::decode_requesting(&bytes, &limits), Some(request));
            assert_eq!(crate::decode_requesting(&fixtures::trailing(&bytes), &limits), None);
        }
    }
    let value = v1::Overturn::new(&limits, v1::OverturnParts { project: u32::MAX, number: u32::MAX }).unwrap();
    let request = v1::Requesting::new(&limits, v1::RequestingParts { request: v1::Request::Overturn(value) }).unwrap();
    let bytes = crate::requesting(&request);
    assert_eq!(crate::decode_requesting(&bytes, &limits), Some(request));
    assert_eq!(crate::decode_requesting(&fixtures::trailing(&bytes), &limits), None);
}

#[test]
fn away_watches_and_outcomes_round_trip() {
    let limits = fixtures::limits();
    for watch in [
        v1::Watch::Away,
        v1::Watch::Decided(v1::DecidedSubject::new(&limits, v1::DecidedSubjectParts { project: u32::MAX }).unwrap()),
    ] {
        let watch = v1::Watching::new(&limits, v1::WatchingParts { watch }).unwrap();
        let bytes = crate::watching(&watch);
        assert_eq!(crate::decode_watching(&bytes, &limits), Some(watch));
        assert_eq!(crate::decode_watching(&fixtures::trailing(&bytes), &limits), None);
    }
    for outcome in [
        v1::Outcome::Away,
        v1::Outcome::Overturned,
        v1::Outcome::OverturnRefused(
            v1::OverturnRefused::new(&limits, v1::OverturnRefusedParts { reason: v1::OverturnRefusal::Already })
                .unwrap(),
        ),
        v1::Outcome::OverturnRefused(
            v1::OverturnRefused::new(&limits, v1::OverturnRefusedParts { reason: v1::OverturnRefusal::Unknown })
                .unwrap(),
        ),
    ] {
        let answer = v1::Answer::new(&limits, v1::AnswerParts { outcome }).unwrap();
        assert_eq!(v1::Answer::decode(&limits, &mut Reader::new(&crate::answer(&answer))), Ok(answer));
    }
}

#[test]
fn aways_rows_are_bounded_at_construction_and_decode() {
    let limits = fixtures::limits();
    let row = v1::AwayRow::new(
        &limits,
        v1::AwayRowParts {
            project: u32::MAX,
            since: u64::MAX,
            until: Some(u64::MAX),
            landed: u64::MAX,
            off: Some(u64::MAX),
        },
    )
    .unwrap();
    for count in [0, v1::CEILINGS.aways_rows, v1::CEILINGS.aways_rows + 1] {
        let mut rows = List::with_capacity(count);
        let mut writer =
            skein_lib::Writer::new(6 + usize::try_from(count).unwrap() * usize::try_from(row.measure()).unwrap());
        writer.put(&u16::try_from(crate::CLIENT_VERSION).unwrap().to_be_bytes()).unwrap();
        writer.put(&count.to_be_bytes()).unwrap();
        for _ in 0..count {
            rows.push(row.clone()).unwrap();
            row.encode(&mut writer).unwrap();
        }
        let bytes = writer.finish();
        let made = v1::Aways::new(&limits, v1::AwaysParts { rows });
        if count > v1::CEILINGS.aways_rows {
            assert_eq!(
                made.unwrap_err(),
                v1::Problem { path: v1::Path::AwaysRows, reason: skein_codec::Reason::Bound }
            );
            assert_eq!(crate::decode_aways(&bytes, &limits), None);
        } else {
            let record = made.unwrap();
            assert_eq!(crate::aways(&record), bytes);
            assert_eq!(crate::decode_aways(&bytes, &limits), Some(record));
            assert_eq!(crate::decode_aways(&fixtures::trailing(&bytes), &limits), None);
        }
    }
}

#[test]
fn decisions_rows_are_bounded_at_construction_and_decode() {
    let limits = fixtures::limits();
    let row = v1::Decision::new(
        &limits,
        v1::DecisionParts {
            number: u32::MAX,
            task: u64::MAX,
            at: u64::MAX,
            question: Box::new([b'q'; 400]),
            took: Box::new([b't'; 120]),
            why: Box::new([b'w'; 400]),
            overturned: true,
            reach: v1::Reach::new(
                &v1::CEILINGS,
                v1::ReachParts {
                    kind: v1::ReachKind::Wording,
                    class: v1::ReachClass::Decide,
                    reason: v1::ReachReason::Kind,
                    under: 0,
                    sets: 0,
                    matched: 0,
                    away: false,
                },
            )
            .unwrap(),
        },
    )
    .unwrap();
    for count in [0, 128, 128 + 1] {
        let mut rows = List::with_capacity(count);
        let mut writer =
            skein_lib::Writer::new(15 + usize::try_from(count).unwrap() * usize::try_from(row.measure()).unwrap());
        writer.put(&u16::try_from(crate::CLIENT_VERSION).unwrap().to_be_bytes()).unwrap();
        writer.put(&u32::MAX.to_be_bytes()).unwrap();
        writer.put(&count.to_be_bytes()).unwrap();
        for _ in 0..count {
            rows.push(row.clone()).unwrap();
            row.encode(&mut writer).unwrap();
        }
        writer.put(&0_u32.to_be_bytes()).unwrap();
        writer.put(&[0]).unwrap();
        let bytes = writer.finish();
        let made = v1::Decisions::new(
            &limits,
            v1::DecisionsParts { project: u32::MAX, rows, asked: List::with_capacity(0), cut: false },
        );
        if count > 128 {
            assert_eq!(
                made.unwrap_err(),
                v1::Problem { path: v1::Path::DecisionsRows, reason: skein_codec::Reason::Bound }
            );
            assert_eq!(crate::decode_decisions(&bytes, &limits), None);
        } else {
            let record = made.unwrap();
            assert_eq!(crate::decisions(&record), bytes);
            assert_eq!(crate::decode_decisions(&bytes, &limits), Some(record));
            assert_eq!(crate::decode_decisions(&fixtures::trailing(&bytes), &limits), None);
        }
    }
}

// temper/docs/design/domain/trial.md, 6.
#[test]
fn trial_requests_watches_and_outcomes_round_trip() {
    let limits = fixtures::limits();
    for again in [false, true] {
        let request = v1::Requesting::new(
            &limits,
            v1::RequestingParts {
                request: v1::Request::TryAccount(
                    v1::TryAccount::new(&limits, v1::TryAccountParts { account: u32::MAX, again }).unwrap(),
                ),
            },
        )
        .unwrap();
        let bytes = crate::requesting(&request);
        assert_eq!(crate::decode_requesting(&bytes, &limits), Some(request));
        assert_eq!(crate::decode_requesting(&fixtures::trailing(&bytes), &limits), None);
    }
    let watch = v1::Watching::new(&limits, v1::WatchingParts { watch: v1::Watch::Trials }).unwrap();
    let bytes = crate::watching(&watch);
    assert_eq!(crate::decode_watching(&bytes, &limits), Some(watch));
    assert_eq!(crate::decode_watching(&fixtures::trailing(&bytes), &limits), None);
    for outcome in [v1::Outcome::Tried, v1::Outcome::NotTheirs] {
        let answer = v1::Answer::new(&limits, v1::AnswerParts { outcome }).unwrap();
        assert_eq!(v1::Answer::decode(&limits, &mut Reader::new(&crate::answer(&answer))), Ok(answer));
    }
}

#[test]
fn trial_rows_round_trip_each_step_state_and_failure() {
    let limits = fixtures::limits();
    let mut states = List::uncapped();
    for state in [
        v1::TrialState::Running,
        v1::TrialState::Answered(
            v1::TrialAnswered::new(&limits, v1::TrialAnsweredParts { seconds: u32::MAX }).unwrap(),
        ),
        v1::TrialState::Failed(
            v1::TrialFailed::new(&limits, v1::TrialFailedParts { why: v1::TrialWhy::Refused }).unwrap(),
        ),
        v1::TrialState::Failed(
            v1::TrialFailed::new(&limits, v1::TrialFailedParts { why: v1::TrialWhy::Waited }).unwrap(),
        ),
    ] {
        states.push(state).unwrap();
    }
    for class in [
        v1::TrialClass::Transient,
        v1::TrialClass::Permanent,
        v1::TrialClass::Run,
        v1::TrialClass::Agent,
        v1::TrialClass::Lost,
        v1::TrialClass::Invalid,
        v1::TrialClass::Refused,
    ] {
        states
            .push(v1::TrialState::Failed(
                v1::TrialFailed::new(
                    &limits,
                    v1::TrialFailedParts {
                        why: v1::TrialWhy::Ended(v1::TrialEnded::new(&limits, v1::TrialEndedParts { class }).unwrap()),
                    },
                )
                .unwrap(),
            ))
            .unwrap();
    }
    for state in &states {
        for step in [v1::TrialStep::Starts, v1::TrialStep::Answers] {
            for (task, took) in [(None, None), (Some(u64::MAX), Some(u32::MAX))] {
                let row = v1::TrialRow::new(
                    &limits,
                    v1::TrialRowParts { account: u32::MAX, task, took, step: step.clone(), state: state.clone() },
                )
                .unwrap();
                let mut rows = List::with_capacity(1);
                rows.push(row).unwrap();
                let document = v1::Trials::new(&limits, v1::TrialsParts { rows }).unwrap();
                let mut writer = skein_lib::Writer::new(usize::try_from(document.measure()).unwrap());
                document.encode(&mut writer).unwrap();
                let bytes = writer.finish();
                let mut reader = Reader::new(&bytes);
                assert_eq!(v1::Trials::decode(&limits, &mut reader), Ok(document));
                assert!(reader.is_empty());
            }
        }
    }
}

#[test]
fn approvals_round_trip_each_review_of_yours_and_their_watch() {
    let limits = v1::CEILINGS;
    let mut rows = List::with_capacity(4);
    let size =
        v1::ApprovalSize::new(&limits, v1::ApprovalSizeParts { files: 4, additions: 142, deletions: 18, cut: false })
            .expect("approval size");
    let saturated = v1::ApprovalSize::new(
        &limits,
        v1::ApprovalSizeParts { files: u32::MAX, additions: u32::MAX, deletions: u32::MAX, cut: true },
    )
    .expect("approval size at its bounds");
    for (yours, goal, size) in [
        (v1::ApprovalYours::Counts, Some(u64::MAX), None),
        (v1::ApprovalYours::NoAccount, None, Some(size)),
        (v1::ApprovalYours::TooFew, None, Some(saturated)),
        (v1::ApprovalYours::OtherRole, None, None),
    ] {
        rows.push(
            v1::ApprovalRow::new(
                &limits,
                v1::ApprovalRowParts {
                    task: u64::MAX,
                    project: u32::MAX,
                    goal,
                    forge: u16::MAX,
                    kind: v1::Kind::Forgejo,
                    owner: Box::new([b'o'; 1024]),
                    name: Box::new([b'n'; 1024]),
                    pull: u64::MAX,
                    address: Box::new([b'a'; 2048]),
                    title: Box::new([b't'; 256]),
                    since: u64::MAX,
                    scope: 0,
                    rule: 1,
                    role: 2,
                    required: 3,
                    counted: 1,
                    sets: List::from_boxed(Box::new([v1::ApprovalSet::new(
                        &limits,
                        v1::ApprovalSetParts { number: 1, name: Box::new([b's'; 40]) },
                    )
                    .expect("set at its name bound")])),
                    whole: true,
                    yours,
                    size,
                },
            )
            .expect("approval row at its byte bounds"),
        )
        .expect("four rows fit");
    }
    let document = v1::Approvals::new(&limits, v1::ApprovalsParts { rows, more: 7 }).expect("approvals");
    let bytes = crate::approvals(&document);
    assert_eq!(crate::decode_approvals(&bytes, &limits), Some(document));
    assert_eq!(crate::decode_approvals(&fixtures::trailing(&bytes), &limits), None, "trailing byte refused");
    let fewer_rows = v1::Limits { approvals_rows: 3, ..limits };
    assert_eq!(crate::decode_approvals(&bytes, &fewer_rows), None, "rows past the limit refused");
    let watch = v1::Watching::new(&limits, v1::WatchingParts { watch: v1::Watch::Approvals }).expect("watch");
    let bytes = crate::watching(&watch);
    assert_eq!(crate::decode_watching(&bytes, &limits), Some(watch));
}
