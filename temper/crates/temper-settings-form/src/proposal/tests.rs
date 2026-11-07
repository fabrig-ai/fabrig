//! Review cuts and retained outcomes in the shared proposal words.
use super::*;

#[test]
fn review_values_keep_their_sizes() {
    let limits = Limits::standard();
    for (value, expected) in [
        (
            w::ReviewValue::TooLarge(w::Size::new(&limits.settings, w::SizeParts { bytes: 65_537 }).unwrap()),
            b"      too large, 65537 bytes\n".as_slice(),
        ),
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
    let limits = Limits::standard();
    for (edit, expected) in [
        (
            w::SizedEdit::Set(
                w::ApplicationSize::new(&limits.settings, w::ApplicationSizeParts { size: 234 }).unwrap(),
            ),
            b"234 bytes".as_slice(),
        ),
        (w::SizedEdit::Remove, b"none".as_slice()),
        (w::SizedEdit::Reset, b"default".as_slice()),
    ] {
        let value = w::ApplicationReview::new(
            &limits.settings,
            w::ApplicationReviewParts {
                edit,
                now: Some(
                    w::ApplicationState::new(
                        &limits.settings,
                        w::ApplicationStateParts {
                            follows: w::ApplicationFollows::Default,
                            size: 123,
                            charters: List::with_capacity(0),
                        },
                    )
                    .unwrap(),
                ),
            },
        )
        .unwrap();
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
    let bounds = &limits.settings;
    let key = w::ProposalKey::new(
        bounds,
        w::ProposalKeyParts {
            proposer: w::Proposer::Party(w::ProposerParty::new(bounds, w::ProposerPartyParts { value: 7 }).unwrap()),
            number: 3,
        },
    )
    .unwrap();
    let mut items = List::with_capacity(3);
    for (number, edit) in
        [(1, w::RevisionItemEdit::Set), (2, w::RevisionItemEdit::Removed), (3, w::RevisionItemEdit::Reset)]
    {
        items
            .push(
                w::RevisionItemHead::new(
                    bounds,
                    w::RevisionItemHeadParts {
                        setting: w::Setting::Charter(
                            w::SettingNumber::new(bounds, w::SettingNumberParts { number }).unwrap(),
                        ),
                        edit,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let history = w::ProposalHistory::new(
        bounds,
        w::ProposalHistoryParts {
            reason: w::DocumentText::new(
                bounds,
                w::DocumentTextParts { bytes: b"Explain work\nKeep it readable".as_slice().into(), cut: false },
            )
            .unwrap(),
            action: w::RetainedAction::Settings(
                w::RetainedActionSettings::new(
                    bounds,
                    w::RetainedActionSettingsParts { projects: List::with_capacity(0), against: 1, items, made: None },
                )
                .unwrap(),
            ),
            number: 3,
            proposer: 7,
            as_holder: false,
            state: w::ProposalHistoryState::Withdrawn,
        },
    )
    .unwrap();
    let ended = w::Ended::new(
        bounds,
        w::EndedParts {
            key: w::ObjectKey::Proposal(
                w::ObjectKeyProposal::new(bounds, w::ObjectKeyProposalParts { value: key }).unwrap(),
            ),
            value: w::EndedValue::Proposal(
                w::EndedValueProposal::new(bounds, w::EndedValueProposalParts { value: history }).unwrap(),
            ),
            at: None,
            by: None,
        },
    )
    .unwrap();
    assert_eq!(ended_proposal_text(&ended, &limits).unwrap().as_ref(), b"proposal p7/3 of settings, withdrawn\nreason:\n  Explain work\n  Keep it readable\nitems:\n  charter 1: set\n  charter 2: removed\n  charter 3: reset\n");
}
