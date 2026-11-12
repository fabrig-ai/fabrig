//! The current families change in place, with settings heads kept after decisions.
use super::{ask_round_trip, core_round_trip, fixtures, people_round_trip, saved_tasks};
use jig_domain_core as c;
use jig_domain_people as p;
use jig_domain_tasks as t;
use skein_lib::Wall;
fn settings_action(count: u32) -> t::PersonAction {
    let mut items = skein_lib::List::with_capacity(count);
    for number in 1..=count {
        items
            .push(t::SettingHead { setting: t::Setting::Template(number), edit: t::SettingEdit::Removed })
            .expect("bounded settings heads");
    }
    t::PersonAction::Settings { against: 1, items: items.into_boxed() }
}
#[test]
fn settings_asks_running_choices_and_refusals_round_trip_in_people_13() {
    let proposed = p::Ask::Propose {
        project: 0,
        action: p::Action::Settings { against: 3, digest: [9; 32] },
        reason: b"review".as_slice().into(),
    };
    ask_round_trip(proposed.clone());
    for running in [None, Some(p::Running::Stays), Some(p::Running::Moves)] {
        ask_round_trip(p::Ask::DecideProposal {
            project: 0,
            proposer: p::Proposer::Party(9),
            proposal: 17,
            decision: p::ProposalDecision::Accept,
            running,
        });
    }
    for why in [
        p::Refusal::Limit { which: p::LimitKind::SettingsProposals },
        p::Refusal::NotWellFormed { at: 0, why: p::Malformed::Running },
    ] {
        people_round_trip(p::Stored::Answer {
            key: p::RequestKey { person: 9, key: [7; 16] },
            ask: Box::new(proposed.clone()),
            models: Box::new([]),
            outcome: p::Outcome::Refused(why),
            at: Wall::EPOCH,
        });
    }
}
#[test]
fn settings_heads_and_each_terminal_round_trip_in_tasks_9() {
    for state in [
        t::PersonProposalState::Pending { since: Wall::EPOCH },
        t::PersonProposalState::SettingsAccepted { by: t::Party::Person(10), revision: 2 },
        t::PersonProposalState::Rejected {
            by: t::Party::Person(10),
            reason: b"later".as_slice().into(),
            message: 18,
            at: Wall::EPOCH,
        },
        t::PersonProposalState::Withdrawn { at: Wall::EPOCH },
    ] {
        let row = t::PersonProposal {
            number: 17,
            proposer: 9,
            project: 0,
            action: settings_action(t::SETTINGS_ITEMS),
            reason: b"review".as_slice().into(),
            state: state.clone(),
        };
        let row = match state {
            t::PersonProposalState::Pending { .. } => t::Stored::PersonProposal(Box::new(row)),
            t::PersonProposalState::SettingsAccepted { .. }
            | t::PersonProposalState::Accepted { .. }
            | t::PersonProposalState::Rejected { .. }
            | t::PersonProposalState::Withdrawn { .. } => t::Stored::DecidedProposal(Box::new(row)),
        };
        assert_eq!(saved_tasks(&row), Ok(row));
    }
    let mut row = fixtures::task_person_proposal();
    row.action = settings_action(t::SETTINGS_ITEMS + 1);
    let limits = super::tasks_limits();
    crate::codec::tasks::proposals::person_proposal(&row, &limits, None)
        .expect_err("129 heads exceed the schema bound");
}
#[test]
fn settings_decision_kind_round_trips_in_core_6() {
    core_round_trip(c::CoreRecord::ProposalDecision(c::ProposalDecisionRecord {
        project: 0,
        proposer: p::Proposer::Party(9),
        proposal: 17,
        kind: t::ProposalKind::Settings,
        by: 10,
        at: Wall::EPOCH,
        choice: p::ProposalChoice::Accepted,
        created: None,
    }));
}

#[test]
fn hold_owner_call_parts_and_placed_hold_revisions_round_trip_in_core_6() {
    for owner in [p::HoldOwner::Party { person: 9 }, p::HoldOwner::Task { task: 7 }] {
        core_round_trip(c::CoreRecord::Call(c::CallRecord {
            key: c::CallKey { task: 3, attempt: 1, completion: 2, position: 0 },
            part: c::CallPart::HoldOwner { task: 7, hold: 31, revision: 2, owner },
            settled: None,
        }));
    }
    core_round_trip(c::CoreRecord::Call(c::CallRecord {
        key: c::CallKey { task: 3, attempt: 1, completion: 2, position: 0 },
        part: c::CallPart::Moved { task: 7, now: p::Moved::PlacedHold { revision: 2 } },
        settled: None,
    }));
}

#[test]
fn application_settings_asks_and_judged_refusals_round_trip_at_their_bounds() {
    let limits = super::people_limits();
    for number in [0, u32::MAX] {
        for count in [0, limits.ask_change_settings_projects] {
            let mut projects = skein_lib::List::with_capacity(count);
            for _ in 0..count {
                projects.push(number).unwrap();
            }
            ask_round_trip(p::Ask::ChangeSettings {
                against: number,
                digest: [0xa5; 32],
                projects: projects.into_boxed(),
            });
        }
        for setting in [p::Setting::Rules, p::Setting::Application { kind: number, scope: number, number }] {
            for count in [0, limits.refusal_judged_why] {
                let why = skein_lib::bytes::zeroed(usize::try_from(count).unwrap());
                people_round_trip(p::Stored::Answer {
                    key: p::RequestKey { person: 9, key: [7; 16] },
                    ask: Box::new(p::Ask::ChangeSettings {
                        against: number,
                        digest: [0xa5; 32],
                        projects: Box::new([]),
                    }),
                    models: Box::new([]),
                    outcome: p::Outcome::Refused(p::Refusal::Judged { at: number, setting, why }),
                    at: Wall::EPOCH,
                });
            }
            people_round_trip(p::Stored::Answer {
                key: p::RequestKey { person: 9, key: [7; 16] },
                ask: Box::new(p::Ask::ChangeSettings { against: number, digest: [0xa5; 32], projects: Box::new([]) }),
                models: Box::new([]),
                outcome: p::Outcome::Refused(p::Refusal::NotWellFormed {
                    at: number,
                    why: p::Malformed::Settings { setting, fault: p::SettingFault::Judged, detail: None },
                }),
                at: Wall::EPOCH,
            });
        }
    }
}

#[test]
fn judged_refusal_words_use_the_people_words_bound() {
    let mut domain = fixtures::CORE_LIMITS;
    domain.people.words = 7;
    let limits = crate::codec::people::limits(&domain).unwrap();
    assert_eq!(limits.refusal_judged_why, 7);
    for count in [7, 8] {
        let row = p::Stored::Answer {
            key: p::RequestKey { person: 9, key: [7; 16] },
            ask: Box::new(p::Ask::ChangeSettings { against: 1, digest: [0; 32], projects: Box::new([]) }),
            models: Box::new([]),
            outcome: p::Outcome::Refused(p::Refusal::Judged {
                at: 0,
                setting: p::Setting::Application { kind: 1, scope: 0, number: 1 },
                why: skein_lib::bytes::zeroed(count),
            }),
            at: Wall::EPOCH,
        };
        assert_eq!(crate::codec::people::encode(&row, &limits, &[]).is_ok(), count == 7);
    }
}

fn restore_left_out(count: u32) -> Box<[p::Setting]> {
    let mut settings = skein_lib::List::with_capacity(count);
    for at in 0..count {
        let setting = if at % 2 == 0 {
            p::Setting::ProjectPolicy { project: u32::MAX }
        } else {
            p::Setting::Application { kind: u32::MAX, scope: u32::MAX, number: u32::MAX }
        };
        settings.push(setting).unwrap();
    }
    settings.into_boxed()
}
fn restore_answer(ask: p::Ask, outcome: p::Outcome) -> p::Stored {
    p::Stored::Answer {
        key: p::RequestKey { person: 9, key: [7; 16] },
        ask: Box::new(ask),
        models: Box::new([]),
        outcome,
        at: Wall::EPOCH,
    }
}
#[test]
fn restore_asks_outcomes_and_forgotten_refusals_round_trip_at_their_bounds() {
    let limits = super::people_limits();
    assert_eq!(limits.ask_restore_reason, fixtures::CORE_LIMITS.people.words);
    assert_eq!(limits.outcome_restored_left_out, t::SETTINGS_ITEMS + 1);
    for revision in [0, u32::MAX] {
        for count in [0, limits.ask_restore_reason] {
            let ask = p::Ask::Restore { revision, reason: skein_lib::bytes::zeroed(usize::try_from(count).unwrap()) };
            ask_round_trip(ask.clone());
            for count in [0, limits.outcome_restored_left_out] {
                people_round_trip(restore_answer(
                    ask.clone(),
                    p::Outcome::Restored { proposal: u64::MAX, to: u32::MAX, left_out: restore_left_out(count) },
                ));
            }
            for refusal in [
                p::Refusal::Forgotten { oldest: u32::MAX },
                p::Refusal::NotWellFormed {
                    at: u32::MAX,
                    why: p::Malformed::Settings {
                        setting: p::Setting::ProjectPolicy { project: u32::MAX },
                        fault: p::SettingFault::Shape,
                        detail: None,
                    },
                },
            ] {
                people_round_trip(restore_answer(ask.clone(), p::Outcome::Refused(refusal)));
            }
        }
    }
}

#[test]
fn a_restore_reason_and_left_out_one_past_their_bounds_are_refused() {
    let mut domain = fixtures::CORE_LIMITS;
    domain.people.words = 7;
    let limits = crate::codec::people::limits(&domain).unwrap();
    let wider = crate::people_v16::Limits {
        ask_restore_reason: limits.ask_restore_reason + 1,
        outcome_restored_left_out: limits.outcome_restored_left_out + 1,
        ..limits
    };
    for (reason, count, passes) in
        [(7, limits.outcome_restored_left_out, true), (8, 0, false), (0, limits.outcome_restored_left_out + 1, false)]
    {
        let row = restore_answer(
            p::Ask::Restore { revision: u32::MAX, reason: skein_lib::bytes::zeroed(reason) },
            p::Outcome::Restored { proposal: u64::MAX, to: u32::MAX, left_out: restore_left_out(count) },
        );
        let encoded = crate::codec::people::encode(&row, &limits, &[]);
        if passes {
            let bytes = encoded.unwrap();
            assert_eq!(crate::codec::people::decode(&bytes, &limits, &[]), Ok(row));
        } else {
            assert_eq!(encoded.unwrap_err().reason, skein_codec::Reason::Bound);
            let bytes = crate::codec::people::encode(&row, &wider, &[]).unwrap();
            assert_eq!(
                crate::codec::people::decode(&bytes, &limits, &[]).unwrap_err().reason,
                skein_codec::Reason::Bound
            );
        }
    }
}
