//! Complete worker stories with real child domains and outside observations.
use temper_worker_domain::{host, wire};
use temper_worker_domain_world::whole::{Case, run, saved, unprepared, workstreams};
#[test]
fn a_prepared_run_delivers_the_complete_tree_and_answers() {
    assert_eq!(run(1, Case::Delivered).expect("world checks").cancellations, 0);
}
#[test]
fn cancelling_mid_delivery_waits_for_the_landing_before_answering() {
    assert_eq!(run(2, Case::CancelledDelivery).expect("world checks").cancellations, 1);
}
#[test]
fn contact_restored_within_the_grace_keeps_the_agent_running() {
    assert_eq!(run(3, Case::ContactWithin).expect("world checks").cancellations, 0);
}
#[test]
fn contact_lost_past_the_grace_saves_before_the_cancellation_answer() {
    assert_eq!(run(4, Case::ContactPast).expect("world checks").saves, 1);
}
#[test]
fn the_hello_carries_the_configured_stop_bound_whole() {
    assert_eq!(run(71, Case::Delivered).expect("world checks").stop_bound, temper_worker_domain_world::STOP_BOUND);
}
#[test]
fn stories_replay() {
    for case in [Case::Delivered, Case::CancelledDelivery, Case::ContactWithin, Case::ContactPast] {
        assert_eq!(run(71, case).expect("world checks"), run(71, case).expect("world checks"));
    }
}
#[test]
fn a_branch_the_forge_lacks_fails_the_prepare_naming_the_repository() {
    let (ending, preparation) = unprepared(5).expect("world checks");
    let host::Ending::Failed { failure: host::Failure::Unprepared(host::Preparation::Permanent { .. }), .. } = ending
    else {
        panic!("a permanent preparation failure: {ending:?}")
    };
    assert_eq!(preparation, Some(wire::Preparation::Missing { tag: 1, missing: wire::Missing::Branch }));
}
#[test]
fn the_hello_names_the_workstreams_the_checkout_holds() {
    assert_eq!(&*workstreams(6).expect("world checks"), [81]);
}

#[test]
fn a_save_says_whether_each_repository_started_from_saved_work() {
    // The fixture's repository has one initial commit, shared by its branches.
    let base = temper_worker_checkout_world::translate::commit(1).raw();
    let rows = [
        (wire::Start::Base { branch: Box::from(&b"main"[..]) }, false),
        (wire::Start::Saved { branch: Box::from(&b"saved"[..]) }, true),
        (wire::Start::Merge { branch: Box::from(&b"saved"[..]), base }, true),
        (wire::Start::Merge { branch: Box::from(&b"topic"[..]), base }, false),
    ];
    for (start, expected) in rows {
        let name = format!("{start:?}");
        let work = saved(8, start).expect("world checks");
        let saved = work.saved.expect("saved work");
        let [wire::SavedRepository { tag, from_saved, landing: wire::Landing::Landed { .. } }] = saved.as_ref() else {
            panic!("{name}: one repository's save landed: {saved:?}");
        };
        assert_eq!((*tag, *from_saved), (1, expected), "{name}");
    }
}

#[test]
fn a_host_answer_at_the_agents_reply_bound_reaches_it_whole() {
    use temper_worker_domain_world::whole::{REPLY_BYTES, host_answer};
    let smith_channel::Reply::Host(reply) = host_answer(91, REPLY_BYTES, false).expect("world checks") else {
        panic!("the agent receives the whole host answer");
    };
    assert!(!reply.error);
    assert_eq!(reply.text.as_ref(), vec![b'x'; usize::try_from(REPLY_BYTES).expect("reply length")]);
}

#[test]
fn a_host_answer_past_the_agents_reply_bound_reaches_it_as_too_large() {
    use temper_worker_domain_world::whole::{REPLY_BYTES, host_answer};
    for error in [false, true] {
        assert_eq!(host_answer(92, REPLY_BYTES + 1, error).expect("world checks"), smith_channel::Reply::TooLarge);
    }
}

#[test]
fn a_saved_retry_delivers_the_file_written_before_its_run_ended() {
    let report = temper_worker_domain_world::whole::retry(109, true).expect("world checks");
    let expected =
        [(b"code".to_vec(), b"original\n".to_vec()), (b"changes/3.md".to_vec(), b"repair kept\n".to_vec())].into();
    assert_eq!(report.tree, expected);
    assert_eq!(report.saves, 1);
}

#[test]
fn a_run_starting_fresh_from_its_brief_starts_clean() {
    let report = temper_worker_domain_world::whole::retry(110, false).expect("world checks");
    assert_eq!(report.tree, [(b"code".to_vec(), b"original\n".to_vec())].into());
    assert_eq!(report.saves, 0);
}

#[test]
fn a_reassigned_start_notes_the_fetch_and_check_out_for_each_attempt() {
    temper_worker_domain_world::whole::reassigned_start(118).expect("world checks");
}

#[test]
fn a_repair_that_delivered_before_parking_finishes_its_next_run_as_a_change() {
    temper_worker_domain_world::whole::delivered_across_park(116, true).expect("world checks");
}

#[test]
fn a_clean_repair_without_its_own_delivery_gets_nothing_when_finishing() {
    temper_worker_domain_world::whole::delivered_across_park(117, false).expect("world checks");
}

#[test]
fn a_landed_change_saves_nothing() {
    let report = run(111, Case::Delivered).expect("world checks");
    assert_eq!(report.saves, 0);
    assert!(report.kept.is_empty(), "finished runs leave no workspace mark");
}

#[test]
fn an_unchecked_save_keeps_the_producer_tree_and_a_checked_push_replaces_its_commit() {
    temper_worker_domain_world::whole::unchecked_save_then_delivery(101).expect("world checks");
}

#[test]
fn parts_relay_before_the_answer_and_are_acknowledged_and_replayed() {
    temper_worker_domain_world::whole::parts::relayed().expect("world checks");
}

#[test]
fn starts_keep_ordered_parts_and_drains_use_grace_only_when_requested() {
    temper_worker_domain_world::whole::parts::starts_and_graces().expect("world checks");
}

#[test]
fn a_drained_answer_resumes_with_parts_in_its_kept_workspace() {
    use temper_worker_domain_world::whole::parts::{InFlight, resumes};
    resumes(InFlight::Answer).expect("world checks");
}

#[test]
fn a_drained_check_resumes_with_parts_and_its_previous_results() {
    use temper_worker_domain_world::whole::parts::{InFlight, resumes};
    resumes(InFlight::Check).expect("world checks");
}

#[test]
fn a_drained_command_resumes_with_its_interrupted_part() {
    use temper_worker_domain_world::whole::parts::{InFlight, resumes};
    resumes(InFlight::Command).expect("world checks");
}

#[test]
fn a_shutdown_drains_the_turn_saves_its_work_and_waits_for_every_acknowledgement() {
    let report = run(112, Case::Drained).expect("world checks");
    assert_eq!((report.drains, report.cancellations, report.saves), (1, 0, 1));
}

#[test]
fn the_drain_bound_cancels_a_run_still_in_its_turn() {
    let report = run(113, Case::DrainExpired).expect("world checks");
    assert_eq!((report.drains, report.cancellations, report.saves), (1, 1, 1));
}

#[test]
fn a_link_drain_saves_the_turn_and_stays_linked_after_every_acknowledgement() {
    let report = run(118, Case::LinkDrained).expect("world checks");
    assert_eq!((report.drains, report.cancellations, report.saves), (1, 0, 1));
}

#[test]
fn the_link_drain_bound_counts_from_the_ask_and_cancels_a_turn_still_running() {
    let report = run(119, Case::LinkDrainExpired).expect("world checks");
    assert_eq!((report.drains, report.cancellations, report.saves), (1, 1, 1));
}

#[test]
fn a_link_pause_keeps_the_run_live_refuses_busy_and_a_resume_admits_again() {
    temper_worker_domain_world::whole::pause::pause_and_resume(120).expect("world checks");
}

#[test]
fn a_stop_during_a_link_drain_keeps_only_the_signals_deadlines() {
    temper_worker_domain_world::whole::pause::stop_during_link_drain(121).expect("world checks");
}

#[test]
fn the_contact_stop_bound_cuts_a_save_push_that_never_answers() {
    assert_eq!(temper_worker_domain_world::whole::contact_bound(114, true).expect("world checks").io_cuts, 1);
}

#[test]
fn the_contact_stop_bound_drops_a_stopping_agent_and_cuts_its_late_save() {
    assert_eq!(temper_worker_domain_world::whole::contact_bound(115, false).expect("world checks").drops, 1);
}
