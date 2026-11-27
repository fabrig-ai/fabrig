//! Conformance scenarios under chaos, one case per seed of each test
//! (`skein/docs/design/testing/sweeps.md`, 4). Each world checks the
//! contract in every mode; the gate checks outcome coverage across its seeds.

use std::{collections::BTreeSet, convert::Infallible};

use skein_conformance::{
    Cancelling, Check, Pairing, PathLimits, Race, Shortness, accept_past_the_descriptor_limit, address_in_use,
    backpressure, cancel_accept, cancel_connect, cancel_connect_established_while_away, cancel_read, cancel_recv,
    closed_before_accept, escapes_within, file_lifecycle, file_metadata, full_accept_queue, graceful_close, ipv6_only,
    lifecycle, list, listener_close_resets_waiting, make_directory, nested_roots, open_past_the_descriptor_limit,
    permissions, refused, remove, rename, reset_after_end_of_stream, send_after_peer_closed, unread_close_meets_recv,
    unread_close_meets_send, wrong_state,
};
use skein_conformance_sim::{
    CHAOS, Racing, Simulated, cancel_chaos, file_cancel_chaos, loopback_chaos, racing_accept, racing_recv,
};
use skein_io::kernel::Family;
use skein_sim::Config;
use skein_world::sweep::{Test, cases, gate};
use skein_world::this_test;

fn sweep(test: Test, mut run: impl FnMut(u64)) {
    cases::<_, Infallible, _>(test, 0..CHAOS, |seed| {
        run(seed);
        Ok(())
    })
    .unwrap_or_else(|case| panic!("{case}"));
}

fn checked<S: Check>(seed: u64, config: Config, scenario: fn(&mut Simulated) -> S) -> S {
    let outcome = scenario(&mut Simulated::new(seed, config));
    outcome.check();
    outcome
}

fn scenario_cases<S: Check>(test: Test, config: Config, scenario: fn(&mut Simulated) -> S) {
    sweep(test, |seed| {
        checked(seed, config, scenario);
    });
}

/// With the faults loopback can show.
fn chaos<S: Check>(test: Test, scenario: fn(&mut Simulated) -> S) {
    scenario_cases(test, loopback_chaos(), scenario);
}

#[test]
fn a_connection_lives_and_ends_over_ipv4() {
    chaos(this_test!("a_connection_lives_and_ends_over_ipv4"), |world| lifecycle(world, Family::Ipv4));
}

#[test]
fn a_connection_lives_and_ends_over_ipv6() {
    chaos(this_test!("a_connection_lives_and_ends_over_ipv6"), |world| lifecycle(world, Family::Ipv6));
}

#[test]
fn a_peer_closes_gracefully() {
    chaos(this_test!("a_peer_closes_gracefully"), graceful_close);
}

#[test]
fn a_send_after_the_peer_closed() {
    chaos(this_test!("a_send_after_the_peer_closed"), send_after_peer_closed);
}

#[test]
fn a_connect_where_nothing_listens() {
    chaos(this_test!("a_connect_where_nothing_listens"), refused);
}

#[test]
fn where_an_address_is_in_use() {
    chaos(this_test!("where_an_address_is_in_use"), address_in_use);
}

#[test]
fn ipv6_sockets_are_ipv6_only() {
    chaos(this_test!("ipv6_sockets_are_ipv6_only"), ipv6_only);
}

#[test]
fn records_wrong_for_the_socket_state() {
    chaos(this_test!("records_wrong_for_the_socket_state"), wrong_state);
}

#[test]
fn a_full_accept_queue() {
    chaos(this_test!("a_full_accept_queue"), full_accept_queue);
}

#[test]
fn a_close_with_bytes_unread_meets_a_recv() {
    chaos(this_test!("a_close_with_bytes_unread_meets_a_recv"), unread_close_meets_recv);
}

#[test]
fn a_close_with_bytes_unread_meets_a_send() {
    chaos(this_test!("a_close_with_bytes_unread_meets_a_send"), unread_close_meets_send);
}

#[test]
fn a_reset_after_the_end_of_stream() {
    chaos(this_test!("a_reset_after_the_end_of_stream"), reset_after_end_of_stream);
}

#[test]
fn a_client_closed_before_its_connection_is_accepted() {
    chaos(this_test!("a_client_closed_before_its_connection_is_accepted"), closed_before_accept);
}

#[test]
fn backpressure_stalls_a_sender_and_reading_resumes_it() {
    chaos(this_test!("backpressure_stalls_a_sender_and_reading_resumes_it"), backpressure);
}

#[test]
fn a_cancel_of_a_waiting_accept_loopback_chaos() {
    chaos(this_test!("a_cancel_of_a_waiting_accept_loopback_chaos"), cancel_accept);
}

#[test]
fn a_cancel_of_a_waiting_accept_cancel_chaos() {
    scenario_cases(this_test!("a_cancel_of_a_waiting_accept_cancel_chaos"), cancel_chaos(), cancel_accept);
}

#[test]
fn a_cancel_of_a_waiting_recv_loopback_chaos() {
    chaos(this_test!("a_cancel_of_a_waiting_recv_loopback_chaos"), cancel_recv);
}

#[test]
fn a_cancel_of_a_waiting_recv_cancel_chaos() {
    scenario_cases(this_test!("a_cancel_of_a_waiting_recv_cancel_chaos"), cancel_chaos(), cancel_recv);
}

#[test]
fn a_cancel_of_a_recv_racing_bytes_loopback_chaos_cancel_first() {
    chaos(this_test!("a_cancel_of_a_recv_racing_bytes_loopback_chaos_cancel_first"), racing_recv(Race::CancelFirst));
}

#[test]
fn a_cancel_of_a_recv_racing_bytes_cancel_chaos_cancel_first() {
    scenario_cases(
        this_test!("a_cancel_of_a_recv_racing_bytes_cancel_chaos_cancel_first"),
        cancel_chaos(),
        racing_recv(Race::CancelFirst),
    );
}

#[test]
fn a_cancel_of_a_recv_racing_bytes_loopback_chaos_arrived_away() {
    chaos(this_test!("a_cancel_of_a_recv_racing_bytes_loopback_chaos_arrived_away"), racing_recv(Race::ArrivedAway));
}

#[test]
fn a_cancel_of_a_recv_racing_bytes_cancel_chaos_arrived_away() {
    scenario_cases(
        this_test!("a_cancel_of_a_recv_racing_bytes_cancel_chaos_arrived_away"),
        cancel_chaos(),
        racing_recv(Race::ArrivedAway),
    );
}

#[test]
fn a_cancel_of_a_recv_racing_bytes_loopback_chaos_arrived_entered() {
    chaos(
        this_test!("a_cancel_of_a_recv_racing_bytes_loopback_chaos_arrived_entered"),
        racing_recv(Race::ArrivedEntered),
    );
}

#[test]
fn a_cancel_of_a_recv_racing_bytes_cancel_chaos_arrived_entered() {
    scenario_cases(
        this_test!("a_cancel_of_a_recv_racing_bytes_cancel_chaos_arrived_entered"),
        cancel_chaos(),
        racing_recv(Race::ArrivedEntered),
    );
}

#[test]
fn a_cancel_of_an_accept_racing_a_connect_loopback_chaos_cancel_first() {
    chaos(
        this_test!("a_cancel_of_an_accept_racing_a_connect_loopback_chaos_cancel_first"),
        racing_accept(Race::CancelFirst),
    );
}

#[test]
fn a_cancel_of_an_accept_racing_a_connect_cancel_chaos_cancel_first() {
    scenario_cases(
        this_test!("a_cancel_of_an_accept_racing_a_connect_cancel_chaos_cancel_first"),
        cancel_chaos(),
        racing_accept(Race::CancelFirst),
    );
}

#[test]
fn a_cancel_of_an_accept_racing_a_connect_loopback_chaos_arrived_away() {
    chaos(
        this_test!("a_cancel_of_an_accept_racing_a_connect_loopback_chaos_arrived_away"),
        racing_accept(Race::ArrivedAway),
    );
}

#[test]
fn a_cancel_of_an_accept_racing_a_connect_cancel_chaos_arrived_away() {
    scenario_cases(
        this_test!("a_cancel_of_an_accept_racing_a_connect_cancel_chaos_arrived_away"),
        cancel_chaos(),
        racing_accept(Race::ArrivedAway),
    );
}

#[test]
fn a_cancel_of_an_accept_racing_a_connect_loopback_chaos_arrived_entered() {
    chaos(
        this_test!("a_cancel_of_an_accept_racing_a_connect_loopback_chaos_arrived_entered"),
        racing_accept(Race::ArrivedEntered),
    );
}

#[test]
fn a_cancel_of_an_accept_racing_a_connect_cancel_chaos_arrived_entered() {
    scenario_cases(
        this_test!("a_cancel_of_an_accept_racing_a_connect_cancel_chaos_arrived_entered"),
        cancel_chaos(),
        racing_accept(Race::ArrivedEntered),
    );
}

#[test]
fn a_cancel_of_a_waiting_connect_loopback_chaos() {
    chaos(this_test!("a_cancel_of_a_waiting_connect_loopback_chaos"), cancel_connect);
}

#[test]
fn a_cancel_of_a_waiting_connect_cancel_chaos() {
    scenario_cases(this_test!("a_cancel_of_a_waiting_connect_cancel_chaos"), cancel_chaos(), cancel_connect);
}

#[test]
fn a_cancel_of_a_connect_established_while_its_client_was_away_loopback_chaos() {
    chaos(
        this_test!("a_cancel_of_a_connect_established_while_its_client_was_away_loopback_chaos"),
        cancel_connect_established_while_away,
    );
}

#[test]
fn a_cancel_of_a_connect_established_while_its_client_was_away_cancel_chaos() {
    scenario_cases(
        this_test!("a_cancel_of_a_connect_established_while_its_client_was_away_cancel_chaos"),
        cancel_chaos(),
        cancel_connect_established_while_away,
    );
}

#[test]
fn closing_a_listener_resets_the_connections_waiting_on_it() {
    chaos(this_test!("closing_a_listener_resets_the_connections_waiting_on_it"), listener_close_resets_waiting);
}

/// Simulator only: the ring's descriptor limit is the process's, which a
/// test cannot lower without `unsafe` (programming-model.md, 2.1).
#[test]
fn an_accept_past_the_descriptor_limit() {
    scenario_cases(
        this_test!("an_accept_past_the_descriptor_limit"),
        Config { max_fds: 4, ..loopback_chaos() },
        accept_past_the_descriptor_limit,
    );
}

/// The simulator draws among the pairings the contract allows. Over the
/// seeds, the races of a `Recv` and of an `Accept` each meet a cancel that
/// stops the target, one too late for it, and one the backend cannot
/// submit. A target interrupted as it is stopped needs a raced cancel to
/// land while its process is away, before anything arrives: targets that
/// wait for nothing meet it.
#[test]
fn the_simulator_meets_each_pairing_over_the_seeds() {
    let racing_targets: [Racing; 2] = [racing_recv, racing_accept];
    let waiting_targets: [fn(&mut Simulated) -> Cancelling; 2] = [cancel_accept, cancel_recv];
    let mut races = racing_targets.map(|target| (target, BTreeSet::new()));
    let mut waits = waiting_targets.map(|target| (target, BTreeSet::new()));
    let at_gate = gate();
    sweep(this_test!("the_simulator_meets_each_pairing_over_the_seeds"), |seed| {
        for (racing, seen) in &mut races {
            for race in [Race::CancelFirst, Race::ArrivedAway] {
                for config in [loopback_chaos(), cancel_chaos()] {
                    let outcome = checked(seed, config, racing(race));
                    if at_gate {
                        seen.insert(outcome.pairing());
                    }
                }
            }
        }
        for (waiting, seen) in &mut waits {
            let outcome = checked(seed, loopback_chaos(), *waiting);
            if at_gate {
                seen.insert(outcome.pairing());
            }
        }
    });
    if at_gate {
        for (_, seen) in races {
            let expected = BTreeSet::from([Pairing::Stopped, Pairing::Completed, Pairing::RanOn]);
            assert_eq!(seen, expected, "the pairings of a race");
        }
        for (_, seen) in waits {
            let expected = BTreeSet::from([Pairing::Stopped, Pairing::Interrupted]);
            assert_eq!(seen, expected, "the pairings of a target that waits for nothing");
        }
    }
}

/// The peer's reset, answering a `Send` after it closed, is sometimes late
/// in a chaos world, as over a network: more than one `Send` succeeds.
#[test]
fn a_late_reset_lets_more_than_one_send_succeed() {
    let mut late = 0_u32;
    let at_gate = gate();
    sweep(this_test!("a_late_reset_lets_more_than_one_send_succeed"), |seed| {
        let seen = checked(seed, loopback_chaos(), send_after_peer_closed);
        if at_gate && seen.sends.len() > 2 {
            late = late.checked_add(1).unwrap();
        }
    });
    if at_gate {
        assert!(late > 0, "no seed had a late reset");
    }
}

#[test]
fn a_file_is_made_written_at_offsets_read_back_and_stated() {
    chaos(this_test!("a_file_is_made_written_at_offsets_read_back_and_stated"), file_lifecycle);
}

#[test]
fn renames_over_across_and_beneath() {
    chaos(this_test!("renames_over_across_and_beneath"), rename);
}

#[test]
fn removes_of_files_directories_and_links() {
    chaos(this_test!("removes_of_files_directories_and_links"), remove);
}

#[test]
fn new_directories_and_one_removed_while_open() {
    chaos(this_test!("new_directories_and_one_removed_while_open"), |world| make_directory(world, PathLimits::LINUX));
}

#[test]
fn listings_whole_one_at_a_time_and_cut_short() {
    chaos(this_test!("listings_whole_one_at_a_time_and_cut_short"), list);
}

#[test]
fn a_root_beneath_a_root() {
    chaos(this_test!("a_root_beneath_a_root"), nested_roots);
}

#[test]
fn paths_that_leave_their_root_and_paths_that_stay() {
    chaos(this_test!("paths_that_leave_their_root_and_paths_that_stay"), |world| {
        escapes_within(world, PathLimits::LINUX)
    });
}

#[test]
fn what_the_owner_may_not_do() {
    chaos(this_test!("what_the_owner_may_not_do"), |world| permissions(world, PathLimits::LINUX));
}

/// Simulator only: the ring's descriptor limit is the process's.
#[test]
fn an_open_past_the_descriptor_limit() {
    scenario_cases(
        this_test!("an_open_past_the_descriptor_limit"),
        Config { max_fds: 4, ..loopback_chaos() },
        open_past_the_descriptor_limit,
    );
}

/// The simulator draws among the counts the contract allows a read and a
/// write: over the seeds, each is cut short, and each counts every byte,
/// which is what the ring does.
#[test]
fn the_simulator_cuts_reads_and_writes_short_and_whole_over_the_seeds() {
    let mut reads = Shortness::default();
    let mut writes = Shortness::default();
    let at_gate = gate();
    sweep(this_test!("the_simulator_cuts_reads_and_writes_short_and_whole_over_the_seeds"), |seed| {
        let seen = checked(seed, loopback_chaos(), file_lifecycle);
        if at_gate {
            reads = reads.and(seen.reads);
            writes = writes.and(seen.writes());
        }
    });
    if at_gate {
        let both = Shortness { short: true, full: true };
        assert_eq!((reads, writes), (both, both), "every outcome of a count, over the seeds");
    }
}

/// The simulator draws among the pairings the contract allows a `Cancel` of
/// a file's `Read`: too late for one that read, and stopping, or
/// interrupting, one that hung, as the ring stops one its worker has not
/// read yet; not submitted, the `Read` running on.
#[test]
fn a_cancel_of_a_read_of_a_file_meets_each_pairing_over_the_seeds() {
    let mut seen = BTreeSet::new();
    let at_gate = gate();
    sweep(this_test!("a_cancel_of_a_read_of_a_file_meets_each_pairing_over_the_seeds"), |seed| {
        let cancelling = checked(seed, file_cancel_chaos(), cancel_read);
        if at_gate {
            seen.insert(cancelling.pairing());
        }
    });
    if at_gate {
        let expected = BTreeSet::from([Pairing::Stopped, Pairing::Interrupted, Pairing::Completed, Pairing::RanOn]);
        assert_eq!(seen, expected, "the pairings of a cancelled Read of a file");
    }
}

#[test]
fn groups_and_usage_under_chaos_loopback_groups() {
    chaos(this_test!("groups_and_usage_under_chaos_loopback_groups"), |world| {
        skein_conformance::groups(world, b"process_fixture")
    });
}

#[test]
fn groups_and_usage_under_chaos_loopback_usage() {
    chaos(this_test!("groups_and_usage_under_chaos_loopback_usage"), |world| {
        skein_conformance::usage(world, b"process_fixture")
    });
}

#[test]
fn appends_continue_short_counts_and_cancel_meets_every_pairing() {
    let mut shorts = Shortness::default();
    let mut pairings = BTreeSet::new();
    let at_gate = gate();
    sweep(this_test!("appends_continue_short_counts_and_cancel_meets_every_pairing"), |seed| {
        let seen = checked(seed, loopback_chaos(), skein_conformance::appending);
        let cancel = checked(seed, file_cancel_chaos(), skein_conformance::cancel_append);
        if at_gate {
            shorts = shorts.and(seen.counts);
            pairings.insert(cancel.pairing());
        }
    });
    if at_gate {
        assert_eq!(shorts, Shortness { short: true, full: true });
        assert_eq!(
            pairings,
            BTreeSet::from([Pairing::Stopped, Pairing::Interrupted, Pairing::Completed, Pairing::RanOn])
        );
    }
}

#[test]
fn stat_keeps_the_owner_and_counts_hard_links_through_removal() {
    chaos(this_test!("stat_keeps_the_owner_and_counts_hard_links_through_removal"), file_metadata);
}

#[test]
fn a_directory_lock_under_chaos_keeps_the_contract() {
    chaos(this_test!("a_directory_lock_under_chaos_keeps_the_contract"), |world| {
        skein_conformance::lock(world, b"process_fixture")
    });
}

#[test]
fn a_no_replace_rename_refuses_a_taken_target_in_linux_order() {
    chaos(this_test!("a_no_replace_rename_refuses_a_taken_target_in_linux_order"), |world| {
        skein_conformance::no_replace(world, PathLimits::LINUX)
    });
}
