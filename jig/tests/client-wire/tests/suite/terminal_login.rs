//! Lent-terminal forms keep their wire tags and optional fields (domain/terminal-login.md, 5).
use jig_client_protocol_wire as w;
use skein_lib::Reader;
fn fixed_values<T: w::Record<Limits = w::Limits>>(rows: &[&[u8]]) {
    for bytes in rows {
        let mut input = Reader::new(bytes);
        let value = T::decode(&w::CEILINGS, &mut input).expect("fixed-rule value record");
        assert!(input.is_empty());
        let encoded = value.encode(&w::CEILINGS).expect("fixed-rule value encoding");
        assert_eq!(encoded.as_ref(), *bytes);
    }
}
#[test]
fn lent_terminal_forms_keep_their_fixed_values() {
    fixed_values::<w::Subject>(&[
        super::client_values::fixed_value("enum_subject_terminal_smallest.bin"),
        super::client_values::fixed_value("enum_subject_terminal_full.bin"),
    ]);
    fixed_values::<w::DeliveryValue>(&[
        super::client_values::fixed_value("enum_delivery_value_terminal_smallest.bin"),
        super::client_values::fixed_value("enum_delivery_value_terminal_full.bin"),
    ]);
    fixed_values::<w::Accounts>(&[
        super::client_values::fixed_value("record_accounts_smallest.bin"),
        super::client_values::fixed_value("record_accounts_full.bin"),
    ]);
    fixed_values::<w::Terminal>(&[
        super::client_values::fixed_value("record_terminal_smallest.bin"),
        super::client_values::fixed_value("record_terminal_full.bin"),
    ]);
    fixed_values::<w::TerminalLogin>(&[
        super::client_values::fixed_value("record_terminal_login_smallest.bin"),
        super::client_values::fixed_value("record_terminal_login_full.bin"),
    ]);
    fixed_values::<w::AccountLentEnd>(&[
        super::client_values::fixed_value("record_account_lent_end_smallest.bin"),
        super::client_values::fixed_value("record_account_lent_end_full.bin"),
    ]);
    fixed_values::<w::LoginEnd>(&[
        super::client_values::fixed_value("enum_login_end_signed_in_smallest.bin"),
        super::client_values::fixed_value("enum_login_end_signed_in_full.bin"),
        super::client_values::fixed_value("enum_login_end_not_finished_smallest.bin"),
        super::client_values::fixed_value("enum_login_end_not_finished_full.bin"),
        super::client_values::fixed_value("enum_login_end_cancelled_smallest.bin"),
        super::client_values::fixed_value("enum_login_end_cancelled_full.bin"),
        super::client_values::fixed_value("enum_login_end_refused_smallest.bin"),
        super::client_values::fixed_value("enum_login_end_refused_full.bin"),
    ]);
    fixed_values::<w::LoginExit>(&[
        super::client_values::fixed_value("enum_login_exit_exited_smallest.bin"),
        super::client_values::fixed_value("enum_login_exit_exited_full.bin"),
        super::client_values::fixed_value("enum_login_exit_killed_smallest.bin"),
        super::client_values::fixed_value("enum_login_exit_killed_full.bin"),
        super::client_values::fixed_value("enum_login_exit_not_started_smallest.bin"),
        super::client_values::fixed_value("enum_login_exit_not_started_full.bin"),
    ]);
    fixed_values::<w::AccountChange>(&[
        super::client_values::fixed_value("enum_account_change_login_program_smallest.bin"),
        super::client_values::fixed_value("enum_account_change_login_program_full.bin"),
        super::client_values::fixed_value("enum_account_change_cancel_login_smallest.bin"),
        super::client_values::fixed_value("enum_account_change_cancel_login_full.bin"),
        super::client_values::fixed_value("enum_account_change_login_ended_smallest.bin"),
        super::client_values::fixed_value("enum_account_change_login_ended_full.bin"),
    ]);
    fixed_values::<w::DocumentOutcome>(&[
        super::client_values::fixed_value("enum_document_outcome_account_login_asked_smallest.bin"),
        super::client_values::fixed_value("enum_document_outcome_account_login_asked_full.bin"),
    ]);
    fixed_values::<w::Refusal>(&[
        super::client_values::fixed_value("enum_refusal_no_terminal_smallest.bin"),
        super::client_values::fixed_value("enum_refusal_no_terminal_full.bin"),
        super::client_values::fixed_value("enum_refusal_login_under_way_smallest.bin"),
        super::client_values::fixed_value("enum_refusal_login_under_way_full.bin"),
        super::client_values::fixed_value("enum_refusal_not_lending_smallest.bin"),
        super::client_values::fixed_value("enum_refusal_not_lending_full.bin"),
    ]);
}
