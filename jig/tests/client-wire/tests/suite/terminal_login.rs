//! Lent-terminal forms keep their wire tags and optional fields (domain/terminal-login.md, 5).
use jig_client_protocol_wire as w;
use skein_lib::Reader;
fn goldens<T: w::Record<Limits = w::Limits>>(rows: &[&[u8]]) {
    for bytes in rows {
        let mut input = Reader::new(bytes);
        let value = T::decode(&w::CEILINGS, &mut input).expect("golden record");
        assert!(input.is_empty());
        let encoded = value.encode(&w::CEILINGS).expect("golden encoding");
        assert_eq!(encoded.as_ref(), *bytes);
    }
}
#[test]
fn lent_terminal_forms_keep_their_goldens() {
    goldens::<w::Subject>(&[
        super::client_values::value("enum_subject_terminal_smallest.bin"),
        super::client_values::value("enum_subject_terminal_full.bin"),
    ]);
    goldens::<w::DeliveryValue>(&[
        super::client_values::value("enum_delivery_value_terminal_smallest.bin"),
        super::client_values::value("enum_delivery_value_terminal_full.bin"),
    ]);
    goldens::<w::Accounts>(&[
        super::client_values::value("record_accounts_smallest.bin"),
        super::client_values::value("record_accounts_full.bin"),
    ]);
    goldens::<w::Terminal>(&[
        super::client_values::value("record_terminal_smallest.bin"),
        super::client_values::value("record_terminal_full.bin"),
    ]);
    goldens::<w::TerminalLogin>(&[
        super::client_values::value("record_terminal_login_smallest.bin"),
        super::client_values::value("record_terminal_login_full.bin"),
    ]);
    goldens::<w::AccountLentEnd>(&[
        super::client_values::value("record_account_lent_end_smallest.bin"),
        super::client_values::value("record_account_lent_end_full.bin"),
    ]);
    goldens::<w::LoginEnd>(&[
        super::client_values::value("enum_login_end_signed_in_smallest.bin"),
        super::client_values::value("enum_login_end_signed_in_full.bin"),
        super::client_values::value("enum_login_end_not_finished_smallest.bin"),
        super::client_values::value("enum_login_end_not_finished_full.bin"),
        super::client_values::value("enum_login_end_cancelled_smallest.bin"),
        super::client_values::value("enum_login_end_cancelled_full.bin"),
        super::client_values::value("enum_login_end_refused_smallest.bin"),
        super::client_values::value("enum_login_end_refused_full.bin"),
    ]);
    goldens::<w::LoginExit>(&[
        super::client_values::value("enum_login_exit_exited_smallest.bin"),
        super::client_values::value("enum_login_exit_exited_full.bin"),
        super::client_values::value("enum_login_exit_killed_smallest.bin"),
        super::client_values::value("enum_login_exit_killed_full.bin"),
        super::client_values::value("enum_login_exit_not_started_smallest.bin"),
        super::client_values::value("enum_login_exit_not_started_full.bin"),
    ]);
    goldens::<w::AccountChange>(&[
        super::client_values::value("enum_account_change_login_program_smallest.bin"),
        super::client_values::value("enum_account_change_login_program_full.bin"),
        super::client_values::value("enum_account_change_cancel_login_smallest.bin"),
        super::client_values::value("enum_account_change_cancel_login_full.bin"),
        super::client_values::value("enum_account_change_login_ended_smallest.bin"),
        super::client_values::value("enum_account_change_login_ended_full.bin"),
    ]);
    goldens::<w::DocumentOutcome>(&[
        super::client_values::value("enum_document_outcome_account_login_asked_smallest.bin"),
        super::client_values::value("enum_document_outcome_account_login_asked_full.bin"),
    ]);
    goldens::<w::Refusal>(&[
        super::client_values::value("enum_refusal_no_terminal_smallest.bin"),
        super::client_values::value("enum_refusal_no_terminal_full.bin"),
        super::client_values::value("enum_refusal_login_under_way_smallest.bin"),
        super::client_values::value("enum_refusal_login_under_way_full.bin"),
        super::client_values::value("enum_refusal_not_lending_smallest.bin"),
        super::client_values::value("enum_refusal_not_lending_full.bin"),
    ]);
}
