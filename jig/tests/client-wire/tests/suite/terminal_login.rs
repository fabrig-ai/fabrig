//! Lent-terminal forms keep their wire tags and optional fields (domain/terminal-login.md, 5).
use jig_client_protocol_wire::v19 as w;
use skein_lib::{Reader, Writer};
fn goldens<T>(
    rows: &[&[u8]],
    decode: fn(&w::Limits, &mut Reader<'_>) -> Result<T, w::Problem>,
    measure: fn(&T) -> u32,
    encode: fn(&T, &mut Writer) -> Result<(), skein_lib::Overflow>,
) {
    for bytes in rows {
        let mut input = Reader::new(bytes);
        let value = decode(&w::CEILINGS, &mut input).expect("golden record");
        assert!(input.is_empty());
        let mut out = Writer::new(skein_lib::length::usize_of(measure(&value)));
        encode(&value, &mut out).expect("measured");
        assert_eq!(out.finish().as_ref(), *bytes);
    }
}
#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one table pins the lent-terminal records, variants and optional fields against their goldens"
)]
fn lent_terminal_forms_keep_their_goldens() {
    goldens(
        &[
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_subject_terminal_smallest.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_subject_terminal_full.bin"),
        ],
        w::Subject::decode,
        w::Subject::measure,
        w::Subject::encode,
    );
    goldens(
        &[
            include_bytes!(
                "../../../../crates/jig-client-protocol-wire/golden/enum_delivery_value_terminal_smallest.bin"
            ),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_delivery_value_terminal_full.bin"),
        ],
        w::DeliveryValue::decode,
        w::DeliveryValue::measure,
        w::DeliveryValue::encode,
    );
    goldens(
        &[
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/record_accounts_smallest.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/record_accounts_full.bin"),
        ],
        w::Accounts::decode,
        w::Accounts::measure,
        w::Accounts::encode,
    );
    goldens(
        &[
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/record_terminal_smallest.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/record_terminal_full.bin"),
        ],
        w::Terminal::decode,
        w::Terminal::measure,
        w::Terminal::encode,
    );
    goldens(
        &[
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/record_terminal_login_smallest.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/record_terminal_login_full.bin"),
        ],
        w::TerminalLogin::decode,
        w::TerminalLogin::measure,
        w::TerminalLogin::encode,
    );
    goldens(
        &[
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/record_account_lent_end_smallest.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/record_account_lent_end_full.bin"),
        ],
        w::AccountLentEnd::decode,
        w::AccountLentEnd::measure,
        w::AccountLentEnd::encode,
    );
    goldens(
        &[
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_login_end_signed_in_smallest.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_login_end_signed_in_full.bin"),
            include_bytes!(
                "../../../../crates/jig-client-protocol-wire/golden/enum_login_end_not_finished_smallest.bin"
            ),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_login_end_not_finished_full.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_login_end_cancelled_smallest.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_login_end_cancelled_full.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_login_end_refused_smallest.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_login_end_refused_full.bin"),
        ],
        w::LoginEnd::decode,
        w::LoginEnd::measure,
        w::LoginEnd::encode,
    );
    goldens(
        &[
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_login_exit_exited_smallest.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_login_exit_exited_full.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_login_exit_killed_smallest.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_login_exit_killed_full.bin"),
            include_bytes!(
                "../../../../crates/jig-client-protocol-wire/golden/enum_login_exit_not_started_smallest.bin"
            ),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_login_exit_not_started_full.bin"),
        ],
        w::LoginExit::decode,
        w::LoginExit::measure,
        w::LoginExit::encode,
    );
    goldens(
        &[
            include_bytes!(
                "../../../../crates/jig-client-protocol-wire/golden/enum_account_change_login_program_smallest.bin"
            ),
            include_bytes!(
                "../../../../crates/jig-client-protocol-wire/golden/enum_account_change_login_program_full.bin"
            ),
            include_bytes!(
                "../../../../crates/jig-client-protocol-wire/golden/enum_account_change_cancel_login_smallest.bin"
            ),
            include_bytes!(
                "../../../../crates/jig-client-protocol-wire/golden/enum_account_change_cancel_login_full.bin"
            ),
            include_bytes!(
                "../../../../crates/jig-client-protocol-wire/golden/enum_account_change_login_ended_smallest.bin"
            ),
            include_bytes!(
                "../../../../crates/jig-client-protocol-wire/golden/enum_account_change_login_ended_full.bin"
            ),
        ],
        w::AccountChange::decode,
        w::AccountChange::measure,
        w::AccountChange::encode,
    );
    goldens(
        &[
            include_bytes!(
                "../../../../crates/jig-client-protocol-wire/golden/enum_document_outcome_account_login_asked_smallest.bin"
            ),
            include_bytes!(
                "../../../../crates/jig-client-protocol-wire/golden/enum_document_outcome_account_login_asked_full.bin"
            ),
        ],
        w::DocumentOutcome::decode,
        w::DocumentOutcome::measure,
        w::DocumentOutcome::encode,
    );
    goldens(
        &[
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_refusal_no_terminal_smallest.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_refusal_no_terminal_full.bin"),
            include_bytes!(
                "../../../../crates/jig-client-protocol-wire/golden/enum_refusal_login_under_way_smallest.bin"
            ),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_refusal_login_under_way_full.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_refusal_not_lending_smallest.bin"),
            include_bytes!("../../../../crates/jig-client-protocol-wire/golden/enum_refusal_not_lending_full.bin"),
        ],
        w::Refusal::decode,
        w::Refusal::measure,
        w::Refusal::encode,
    );
}
