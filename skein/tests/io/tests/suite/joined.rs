//! Joined sockets with both owners in one process (io.md, 3.5 and 8).
use skein_io_world::joined::{closing, names_full, taken, unbound};
use skein_sim::Config;
const SEEDS: [u64; 3] = [1, 2, 3];

#[test]
fn a_connect_to_an_unbound_name_is_refused() {
    for seed in SEEDS {
        let outcome = unbound(seed, Config::chaos()).run();
        assert!(outcome.trace.is_empty(), "seed {seed}: no kernel submission");
    }
}

#[test]
fn a_second_listener_on_a_taken_name_fails_with_other() {
    for seed in SEEDS {
        assert!(taken(seed, Config::chaos()).run().trace.is_empty());
    }
}

#[test]
fn named_listens_past_the_names_limit_are_refused_busy() {
    for seed in SEEDS {
        assert!(names_full(seed, Config::chaos()).run().trace.is_empty());
    }
}

#[test]
fn a_closing_listener_refuses_its_waiting_connectors() {
    for seed in SEEDS {
        assert!(closing(seed, Config::chaos()).run().trace.is_empty());
    }
}

#[test]
fn a_named_listener_accepts_binds_and_rejects() {
    for seed in SEEDS {
        let outcome = skein_io_world::joined::accept(seed, Config::chaos()).run();
        assert!(outcome.trace.is_empty());
        assert!(
            outcome.logs[0].iter().any(|line| line.contains("Failed(Reset)")),
            "the rejected connector hears Reset"
        );
    }
}
#[test]
fn two_named_listeners_share_one_accept_batch() {
    for seed in SEEDS {
        assert!(skein_io_world::joined::batch(seed, Config::chaos()).run().trace.is_empty());
    }
}
#[test]
fn connectors_wait_without_room_or_slot_and_one_aborts() {
    for seed in SEEDS {
        assert!(skein_io_world::joined::waiting(seed, Config::chaos()).run().trace.is_empty());
    }
}
#[test]
fn bytes_go_both_ways_over_a_joined_pair_under_small_intakes() {
    for seed in SEEDS {
        assert!(skein_io_world::joined::exchange(seed, Config::chaos()).run().trace.is_empty());
    }
}
#[test]
fn a_joined_reader_that_does_not_demand_stops_its_peer() {
    for seed in SEEDS {
        assert!(skein_io_world::joined::backpressure(seed, Config::chaos()).run().trace.is_empty());
    }
}
#[test]
fn an_abort_is_heard_as_reset_by_the_joined_peer() {
    for seed in SEEDS {
        let outcome = skein_io_world::joined::abort(seed, Config::chaos()).run();
        assert!(outcome.trace.is_empty());
        assert!(outcome.logs[0].iter().any(|line| line.contains("Failed(Reset)")), "the aborted peer hears Reset");
    }
}

#[test]
fn a_response_in_the_middle_of_an_upload_reaches_the_joined_peer() {
    for seed in SEEDS {
        assert!(skein_io_world::joined::refusal_mid_upload(seed, Config::chaos()).run().trace.is_empty());
    }
}
#[test]
fn a_joined_close_the_peer_never_ends_meets_its_deadline() {
    for seed in SEEDS {
        assert!(skein_io_world::joined::close_deadline(seed, Config::chaos()).run().trace.is_empty());
    }
}
#[test]
fn joined_closes_and_aborts_at_random_moments_settle() {
    for seed in SEEDS {
        assert!(skein_io_world::joined::random(seed, Config::chaos()).run().trace.is_empty());
    }
}

#[test]
fn the_same_owners_read_the_same_over_either_pair() {
    use skein_io_world::owner::{Reads, Total};
    use skein_io_world::scenarios::plan;
    for seed in SEEDS {
        let first: Box<[u8]> = Box::from(&b"one sends a message longer than either intake"[..]);
        let second: Box<[u8]> = Box::from(&b"two answers with another long message"[..]);
        let served = skein_io_world::owner::Plan {
            total: Total::Known(first.len()),
            ..plan("served", second.clone(), 16, Reads::Mixed(7))
        };
        let dialer = skein_io_world::owner::Plan {
            total: Total::Known(second.len()),
            ..plan("dialer", first.clone(), 16, Reads::Mixed(7))
        };
        let [loopback, joined] = skein_io_world::joined::either(seed, [served, dialer]);
        assert_eq!(loopback, joined, "seed {seed}");
        assert_eq!(joined.received, [first.into_vec(), second.into_vec()]);
        assert_eq!(joined.ended, [true, true]);
        assert_eq!(joined.failed, [None, None]);
    }
}

#[test]
fn an_ip_and_a_named_listener_accept_and_exchange_independently() {
    use skein_io_world::owner::{Dial, Owner, Reads, Serve, ServeClose, Target, Total};
    use skein_io_world::referee::{Expect, Referee};
    use skein_io_world::scenarios::{END, limits, local, plan};
    use skein_io_world::world::World;
    use skein_lib::Time;

    for seed in SEEDS {
        let mut serves = Vec::new();
        let mut dials = Vec::new();
        let mut expectations = Vec::new();
        for (listener, addr, server, client, upload, reply) in [
            ("ip", local(0), "ip-server", "ip-client", b"IP upload".as_slice(), b"IP reply".as_slice()),
            (
                "named",
                skein_io_world::joined::name(b"mixed"),
                "named-server",
                "named-client",
                b"named upload".as_slice(),
                b"named reply".as_slice(),
            ),
        ] {
            serves.push(Serve {
                name: listener,
                addr,
                answers: vec![Some(skein_io_world::owner::Plan {
                    total: Total::Known(upload.len()),
                    ..plan(server, Box::from(reply), 16, Reads::Mixed(7))
                })],
                close: ServeClose::At(Time::from_nanos(50_000_000)),
            });
            dials.push(Dial {
                to: Target::Named(listener),
                at: Time::from_nanos(20_000_000),
                plan: skein_io_world::owner::Plan {
                    total: Total::Known(reply.len()),
                    ..plan(client, Box::from(upload), 16, Reads::Mixed(7))
                },
            });
            for (conn, bytes) in [(server, upload), (client, reply)] {
                expectations.push(Expect::Receives { at: 0, conn, bytes: Box::from(bytes), by: END });
                expectations.push(Expect::Ends { at: 0, conn, by: END });
                expectations.push(Expect::Closed { at: 0, conn, by: END, optional: false });
            }
        }
        let mut world = World::new(seed, Config::calm(), Referee::new(seed, expectations));
        world.spawn(limits(6), Owner::new(seed, serves, dials));
        let outcome = world.run();
        assert_eq!(
            outcome.logs[0]
                .iter()
                .take_while(|line| !line.contains("Closed {"))
                .filter(|line| line.contains("Listening {"))
                .count(),
            2,
            "seed {seed}: both listeners open before either closes"
        );
        assert!(!outcome.trace.is_empty(), "the IP pair uses the simulated kernel");
    }
}
