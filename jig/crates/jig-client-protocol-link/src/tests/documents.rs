use super::*;
use crate::translate::{Undecodable, documents, live};
#[test]
fn a_me_record_becomes_the_contract_s_me() {
    assert_eq!(documents::frame(fixtures::frame_record(), &fixtures::limits()), Ok(fixtures::frame()));
}
#[test]
fn a_task_one_row_past_the_client_s_limit_is_refused() {
    assert_eq!(documents::task(fixtures::task_with_lineage(3), &fixtures::limits()), Err(Undecodable::TooLarge));
}
#[test]
fn a_run_text_keeps_its_turn_retry_and_offsets() {
    assert_eq!(live::run_text(fixtures::text_record(), &fixtures::limits()), Ok(fixtures::run_text_expected()));
}

#[test]
fn a_chip_in_each_phase_becomes_the_contract_s_chip() {
    for (record, expected) in fixtures::phases() {
        assert_eq!(documents::chip(record, &fixtures::limits()), Ok(expected));
    }
}

fn a_task_keeps_every_supplied_field() {
    assert_eq!(documents::task(fixtures::task_record(), &fixtures::limits()), Ok(fixtures::task()));
}

fn a_tasks_keeps_every_supplied_field() {
    assert_eq!(documents::tasks(fixtures::tasks_record(), &fixtures::limits()), Ok(fixtures::tasks()));
}

fn a_owner_deployment_keeps_every_supplied_field() {
    assert_eq!(documents::deployment(fixtures::deployment_record(), &fixtures::limits()), Ok(fixtures::deployment()));
}

fn a_owner_project_keeps_every_supplied_field() {
    assert_eq!(documents::project(fixtures::project_record(), &fixtures::limits()), Ok(fixtures::project()));
}

fn a_run_state_keeps_every_supplied_field() {
    assert_eq!(live::run_state(fixtures::run_state_record(), &fixtures::limits()), Ok(fixtures::run_state()));
}

fn a_current_call_keeps_every_supplied_field() {
    assert_eq!(live::current_call(fixtures::current_call_record(), &fixtures::limits()), Ok(fixtures::current_call()));
}
fn a_task_piece_with_a_partial_header_is_malformed() {
    assert_eq!(
        documents::task(fixtures::task_with_piece(&[1, 0, 0]), &fixtures::limits()),
        Err(Undecodable::Malformed)
    );
}
fn a_task_piece_past_its_end_is_malformed() {
    assert_eq!(
        documents::task(fixtures::task_with_piece(&[5, 0, 0, 0, 1]), &fixtures::limits()),
        Err(Undecodable::Malformed)
    );
}
#[test]
fn a_document_at_its_measured_limit_is_admitted() {
    let record = fixtures::frame_record();
    let limits = fixtures::limits_for_document(u64::from(record.measure()));
    assert_eq!(documents::frame(record, &limits), Ok(fixtures::frame()));
}
#[test]
fn a_document_one_byte_past_its_measured_limit_is_refused() {
    let record = fixtures::frame_record();
    let limits = fixtures::limits_for_document(u64::from(record.measure()).checked_sub(1).unwrap());
    assert_eq!(documents::frame(record, &limits), Err(Undecodable::TooLarge));
}
#[test]
fn document_decodes_borrow_the_kept_bounds() {
    let limits = fixtures::limits();
    let bounds = limits.receiving();
    assert_eq!(bounds.document_text_bytes, limits.client().text.max(512));
    assert_eq!(bounds.call_input, limits.client().text);
    assert_eq!(documents::frame(fixtures::frame_record(), &limits), Ok(fixtures::frame()));
    assert_eq!(documents::frame(fixtures::frame_record(), &limits), Ok(fixtures::frame()));
    assert!(core::ptr::eq(bounds, limits.receiving()), "decoders retain the same receiving bounds");
}
#[test]
fn invalid_receiving_limits_are_refused_at_construction() {
    let limits = fixtures::limits();
    let client = jig_client_domain::Limits { text: u32::MAX, ..*limits.client() };
    assert_eq!(Limits::new(client, limits.exchanges()), Err(Unusable::Limits));
}

#[test]
fn a_run_text_without_optional_values_keeps_their_absence() {
    assert_eq!(live::run_text(fixtures::run_text_record(), &fixtures::limits()), Ok(fixtures::run_text()));
}
#[test]
fn a_task_at_its_row_bound_is_admitted() {
    assert_eq!(
        documents::task(fixtures::task_at_row_bound(), &fixtures::limits()),
        Ok(fixtures::task_with_two_chips())
    );
}
#[test]
fn a_run_text_one_byte_past_its_text_limit_is_refused() {
    assert_eq!(
        live::run_text(fixtures::run_text_one_past_text_limit(), &fixtures::limits()),
        Err(Undecodable::TooLarge)
    );
}
#[test]
fn an_account_later_keeps_its_optional_wall() {
    for until in [None, Some(0), Some(u64::MAX)] {
        assert_eq!(
            documents::deployment(fixtures::with_account(until), &fixtures::limits()),
            Ok(fixtures::account_expected(until))
        );
    }
}

#[test]
fn a_raw_live_call_keeps_its_provider_retry_and_start() {
    assert_eq!(live::live_call(fixtures::raw_live_call(), &fixtures::limits()), Ok(fixtures::raw_live_call_expected()));
}
#[test]
fn a_decoded_live_call_keeps_its_committed_call() {
    assert_eq!(
        live::live_call(fixtures::decoded_live_call(), &fixtures::limits()),
        Ok(fixtures::decoded_live_call_expected())
    );
}

#[test]
fn malformed_task_pieces_are_refused() {
    table([
        ("a_task_piece_with_a_partial_header_is_malformed", a_task_piece_with_a_partial_header_is_malformed),
        ("a_task_piece_past_its_end_is_malformed", a_task_piece_past_its_end_is_malformed),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    let mut failures = [None; N];
    let mut passed = true;
    for ((name, case), failure) in cases.into_iter().zip(&mut failures) {
        if std::panic::catch_unwind(case).is_err() {
            *failure = Some(name);
            passed = false;
        }
    }
    assert!(passed, "{failures:?}");
}

#[test]
fn documents_keep_every_supplied_field() {
    table([
        ("a_task_keeps_every_supplied_field", a_task_keeps_every_supplied_field),
        ("a_tasks_keeps_every_supplied_field", a_tasks_keeps_every_supplied_field),
        ("a_owner_deployment_keeps_every_supplied_field", a_owner_deployment_keeps_every_supplied_field),
        ("a_owner_project_keeps_every_supplied_field", a_owner_project_keeps_every_supplied_field),
        ("a_run_state_keeps_every_supplied_field", a_run_state_keeps_every_supplied_field),
        ("a_current_call_keeps_every_supplied_field", a_current_call_keeps_every_supplied_field),
    ]);
}

#[test]
fn a_resource_path_at_its_segment_bound_is_decoded_past_the_row_bound() {
    let mut expected = fixtures::task();
    let c::TaskContractDocument::Approval { part, .. } = &mut expected.result_contract else { panic!("approval") };
    let mut path = skein_lib::List::with_capacity(4);
    for _ in 0_u32..4 {
        path.push(Box::from(b"segment".as_slice())).unwrap();
    }
    part.key = c::PartKey::Path(path);
    assert_eq!(documents::task(fixtures::task_with_resource_path(4), &fixtures::limits()), Ok(expected));
}

#[test]
fn a_resource_path_one_segment_past_its_bound_is_refused() {
    assert_eq!(documents::task(fixtures::task_with_resource_path(5), &fixtures::limits()), Err(Undecodable::TooLarge));
}

#[test]
fn an_authority_uses_its_own_bound_beyond_the_document_rows() {
    let limits = fixtures::limits();
    assert_eq!(limits.client().rows, 2);
    let (record, expected) = fixtures::task_with_authority_grants(limits.client().authority);
    assert_eq!(documents::task(record, &limits), Ok(expected));
    let (record, _) = fixtures::task_with_authority_grants(limits.client().authority + 1);
    assert_eq!(documents::task(record, &limits), Err(Undecodable::TooLarge));
}

fn catalogue_frame(address: &[u8]) -> w::Me {
    let limits = *fixtures::limits().wire();
    let row = w::ProviderOffer::new(
        &limits,
        w::ProviderOfferParts {
            provider: 37,
            key_bytes: 256,
            name: w::DocumentText::new(
                &limits,
                w::DocumentTextParts { bytes: b"Arbitrary".as_slice().into(), cut: false },
            )
            .unwrap(),
            skein_bools: [true, true, false, false, true],
            programs: skein_lib::List::with_capacity(0),
            manage_usage: Some(address.into()),
        },
    )
    .unwrap();
    let mut writer = skein_lib::Writer::new(usize::try_from(row.measure()).unwrap());
    row.encode(&mut writer).unwrap();
    let bytes = writer.finish();
    let mut writer = skein_lib::Writer::new(bytes.len().checked_add(4).unwrap());
    skein_codec::piece::write(&mut writer, &bytes).unwrap();
    let mut frame = fixtures::frame_record().into_parts();
    frame.providers = writer.finish();
    w::Me::new(&limits, frame).unwrap()
}
#[test]
fn catalogue_addresses_are_checked_before_publication() {
    for address in [b"ftp://x".as_slice(), b"https:///", b"https://x/\""] {
        assert_eq!(documents::frame(catalogue_frame(address), &fixtures::limits()), Err(Undecodable::Malformed));
    }
}
#[test]
fn catalogue_keeps_the_configured_provider_words() {
    let frame = documents::frame(catalogue_frame(b"https://usage.example/"), &fixtures::limits()).unwrap();
    let row = frame.providers.get(0).unwrap();
    assert_eq!(row.provider, 37);
    assert_eq!(row.name.bytes(), b"Arbitrary");
    assert!(row.sign_in && row.key && row.signed_in_plan);
}

fn repeated_catalogue(count: usize) -> w::Me {
    let mut frame = catalogue_frame(b"https://usage.example/").into_parts();
    let mut writer = skein_lib::Writer::new(frame.providers.len().checked_mul(count).unwrap());
    for _ in 0..count {
        writer.put(&frame.providers).unwrap();
    }
    frame.providers = writer.finish();
    w::Me::new(fixtures::limits().wire(), frame).unwrap()
}
#[test]
fn catalogue_repeated_providers_are_refused() {
    assert_eq!(documents::frame(repeated_catalogue(2), &fixtures::limits()), Err(Undecodable::Malformed));
}
#[test]
fn catalogue_row_capacity_is_bounded() {
    assert_eq!(documents::frame(repeated_catalogue(3), &fixtures::limits()), Err(Undecodable::TooLarge));
}

fn redirect_frame(bound: u32) -> w::Me {
    let mut parts = fixtures::frame_record().into_parts();
    parts.redirect_bytes = bound;
    w::Me::new(fixtures::limits().wire(), parts).unwrap()
}
#[test]
fn the_redirect_bound_at_the_wire_bound_is_advertised() {
    let limits = fixtures::limits();
    assert_eq!(
        documents::frame(redirect_frame(limits.wire().keyed_value_some), &limits).unwrap().redirect_bytes,
        limits.wire().keyed_value_some
    );
}
#[test]
fn a_redirect_bound_past_the_wire_bound_is_refused() {
    let limits = fixtures::limits();
    assert_eq!(
        documents::frame(redirect_frame(limits.wire().keyed_value_some + 1), &limits),
        Err(Undecodable::TooLarge)
    );
}

#[test]
fn task_settings_revisions_cross_into_the_client_document() {
    for settings in [
        None,
        Some(c::TaskSettings { task: 7, run: None, newest: 9 }),
        Some(c::TaskSettings { task: 7, run: Some(6), newest: 9 }),
    ] {
        let mut parts = fixtures::task_record().into_parts();
        parts.settings = None;
        if let Some(value) = settings {
            parts.settings = Some(
                w::TaskSettings::new(
                    &w::CEILINGS,
                    w::TaskSettingsParts { task: value.task, run: value.run, newest: value.newest },
                )
                .expect("settings"),
            );
        }
        let record = w::Task::new(&w::CEILINGS, parts).expect("task record");
        let task = documents::task(record, &fixtures::limits()).expect("task document");
        assert_eq!(task.settings, settings);
    }
}

#[test]
fn chip_phases_refuse_missing_end_time_and_retry_notice() {
    let limits = fixtures::limits();
    let backing_off = w::ChipPhase::Active(
        w::ChipPhaseActive::new(
            limits.wire(),
            w::ChipPhaseActiveParts {
                value: w::RunStage::BackingOff(
                    w::RunStageBackingOff::new(limits.wire(), w::RunStageBackingOffParts { until: 0 }).unwrap(),
                ),
            },
        )
        .unwrap(),
    );
    for (phase, compact) in [(w::ChipPhase::Ended, w::Phase::Ended), (backing_off, w::Phase::BackingOff)] {
        for present in [false, true] {
            let w::Fact::Chip(root) = fixtures::chip_root_fact() else { panic!("chip root") };
            let mut root = root.into_parts();
            let mut fields = root.value.into_parts();
            fields.phase = compact.clone();
            root.value = w::ChipFields::new(limits.wire(), fields).unwrap();
            let mut details = fixtures::wire_owner_chip_details_row().into_parts();
            let mut fields = details.value.into_parts();
            if present {
                fields.ended_at = Some(0);
                fields.retry = Some(w::RetryNotice::Refused(
                    w::RetryNoticeRefused::new(limits.wire(), w::RetryNoticeRefusedParts { count: 0 }).unwrap(),
                ));
            }
            details.value = w::ChipDetailsFields::new(limits.wire(), fields).unwrap();
            let chip = fixtures::chip_from_rows(&[
                w::Fact::Chip(w::ChipRow::new(limits.wire(), root).unwrap()),
                w::Fact::ChipPhase(
                    w::ChipPhaseRow::new(
                        limits.wire(),
                        w::ChipPhaseRowParts {
                            parent: Some(0),
                            value: w::ChipPhaseOwnerFields::new(
                                limits.wire(),
                                w::ChipPhaseOwnerFieldsParts { phase: phase.clone(), details: 2 },
                            )
                            .unwrap(),
                        },
                    )
                    .unwrap(),
                ),
                w::Fact::ChipDetails(Box::new(w::ChipDetailsRow::new(limits.wire(), details).unwrap())),
            ]);
            let decoded = documents::chip(chip, &limits);
            if present {
                assert!(decoded.is_ok(), "zero-valued required facts are supplied");
            } else {
                assert_eq!(decoded, Err(Undecodable::Malformed));
            }
        }
    }
}

#[test]
fn receiving_inventory_bounds_keep_their_schema_ceilings() {
    let base = fixtures::limits();
    let client = jig_client_domain::Limits {
        rows: 32,
        text: 8192,
        turn_text: 8192,
        turn_bytes: 8192,
        document_bytes: u64::from(w::CEILINGS.owned_record),
        ask_bytes: u64::from(u32::MAX),
        ..*base.client()
    };
    let limits = Limits::new(client, base.exchanges()).unwrap();
    assert_eq!(limits.wire().create_tasks, 1_048_576);
    assert_eq!(limits.wire().charter_instructions, 1_114_240);
    assert_eq!(limits.wire().propose_settings_items, 1_114_240);
    assert_eq!(limits.wire().change_settings_items, 1_114_240);
    assert_eq!(limits.wire().charter_alternatives, 16);
    assert_eq!(limits.wire().policy_edit_templates_overrides, 16);
    assert_eq!(limits.wire().setting_model_efforts, 16);
    assert_eq!(limits.wire().setting_policy_templates, 16);
    assert_eq!(limits.receiving().settings_document_settings, 256);
    assert_eq!(limits.wire().model_set_names_names_item, 4096);
    assert_eq!(limits.wire().setting_model_name, 4096);
    assert_eq!(limits.wire().setting_model_program_some, 32);
    assert_eq!(limits.wire().setting_model_effort_some, 32);
    assert_eq!(limits.wire().setting_model_efforts_item, 32);
}

#[test]
fn hold_accounts_decode_whole_past_the_text_preview_bound() {
    let limits = fixtures::limits();
    assert!(limits.client().text < 512);
    let cases: [(Box<[u8]>, bool); 2] = [(Box::new([b'x'; 512]), true), (Box::new([b'x'; 513]), false)];
    for (bytes, accepted) in cases {
        let mut parts = jig_client_test_records::hold_record().into_parts();
        parts.words = Some(
            w::DocumentText::new(&w::CEILINGS, w::DocumentTextParts { bytes: bytes.clone(), cut: false }).unwrap(),
        );
        let mut task = fixtures::task_record().into_parts();
        task.hold = Some(w::ChipHold::new(&w::CEILINGS, parts).unwrap());
        let decoded = documents::task(w::Task::new(&w::CEILINGS, task).unwrap(), &limits);
        if accepted {
            let words = decoded.unwrap().hold.unwrap().words.unwrap();
            assert_eq!(words.bytes.bytes(), bytes.as_ref());
            assert!(!words.cut);
        } else {
            assert_eq!(decoded, Err(Undecodable::TooLarge));
        }
    }
}
