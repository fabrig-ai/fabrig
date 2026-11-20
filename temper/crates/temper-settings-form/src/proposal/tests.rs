//! Review cuts and retained outcomes in the shared proposal words.
use super::*;

#[test]
fn review_values_keep_their_sizes() {
    let limits = Limits::standard();
    for (value, expected) in [
        (w::ReviewValue::TooLarge(w::Size { bytes: 65_537 }), b"      too large, 65537 bytes\n".as_slice()),
        (w::ReviewValue::None, b"      none\n".as_slice()),
    ] {
        let mut measure = Text::measure();
        let setting = view::Setting::Charter(view::SettingNumber { number: 1 });
        review_value(setting, &value, &limits, &mut measure).unwrap();
        let mut out = Text::writing_exact(measure);
        review_value(setting, &value, &limits, &mut out).unwrap();
        assert_eq!(out.finish().as_ref(), expected);
    }
}

#[test]
fn application_reviews_keep_the_sizes_and_edits_they_carry() {
    for (edit, expected) in [
        (w::SizedEdit::Set(w::ApplicationSize { size: 234 }), b"234 bytes".as_slice()),
        (w::SizedEdit::Remove, b"none".as_slice()),
        (w::SizedEdit::Reset, b"default".as_slice()),
    ] {
        let value = w::ApplicationReview {
            edit,
            now: Some(w::ApplicationState {
                follows: w::ApplicationFollows::Default,
                size: 123,
                charters: List::with_capacity(0),
            }),
        };
        let mut measure = Text::measure();
        application_review(&value, &mut measure);
        let mut out = Text::writing_exact(measure);
        application_review(&value, &mut out);
        let actual = out.finish();
        assert!(actual.starts_with(b"      123 bytes\n    would be:\n      "));
        assert_eq!(
            actual.strip_prefix(b"      123 bytes\n    would be:\n      ").unwrap().strip_suffix(b"\n").unwrap(),
            expected
        );
    }
}

#[test]
fn a_withdrawn_party_proposal_keeps_its_reason_and_heads() {
    let limits = Limits::standard();
    let key = w::ProposalKey { proposer: w::Proposer::Party(w::ProposerParty { value: 7 }), number: 3 };
    let mut items = List::with_capacity(3);
    for (number, edit) in
        [(1, w::RevisionItemEdit::Set), (2, w::RevisionItemEdit::Removed), (3, w::RevisionItemEdit::Reset)]
    {
        items.push(w::RevisionItemHead { setting: w::Setting::Charter(w::SettingNumber { number }), edit }).unwrap();
    }
    let history = w::ProposalHistory {
        reason: w::DocumentText { bytes: b"Explain work\nKeep it readable".as_slice().into(), cut: false },
        action: w::RetainedAction::Settings(w::RetainedActionSettings {
            projects: List::with_capacity(0),
            against: 1,
            items,
            made: None,
        }),
        number: 3,
        proposer: 7,
        as_holder: false,
        state: w::ProposalHistoryState::Withdrawn,
    };
    let ended = w::Ended {
        key: w::ObjectKey::Proposal(w::ObjectKeyProposal { value: key }),
        value: w::EndedValue::Proposal(w::EndedValueProposal { value: history }),
        at: None,
        by: None,
    };
    assert_eq!(ended_proposal_text(&ended, &limits).unwrap().as_ref(), b"proposal p7/3 of settings, withdrawn\nreason:\n  Explain work\n  Keep it readable\nitems:\n  charter 1: set\n  charter 2: removed\n  charter 3: reset\n");
}

#[test]
fn a_worker_review_prints_its_head_and_whole_value() {
    let limits = Limits::standard();
    let setting = view::Setting::Worker(view::SettingNumber { number: 2 });
    let value = w::ReviewValue::Value(w::SettingSet {
        value: w::SettingValue::Worker(Box::new(w::WorkerSetting {
            number: 2,
            name: b"w1".as_slice().into(),
            places: 4,
            digest: None,
            withdrawn: false,
        })),
    });
    let mut measure = Text::measure();
    render::setting_text(setting, &mut measure);
    review_value(setting, &value, &limits, &mut measure).unwrap();
    let mut out = Text::writing_exact(measure);
    render::setting_text(setting, &mut out);
    review_value(setting, &value, &limits, &mut out).unwrap();
    let said = out.finish();
    assert!(skein_lib::bytes::find(&said, b"worker 2").is_some());
    assert!(skein_lib::bytes::find(&said, br#""number":2,"name":"w1","places":4"#).is_some());
}
