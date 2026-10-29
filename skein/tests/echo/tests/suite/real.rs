//! The echo on the real loop (testing-strategy.md, 2.8; examples.md, 7): the
//! echo as it ships and its fake clients, on one shared kernel, in one
//! thread and one loop over the shell's kernel, on loopback, with the
//! referee's deadlines on the real clock. It shows what only the real
//! kernel can: each backend under the whole service, and real sockets
//! between real processes' worth of io. It does not replay; what fails here
//! is rerun in the simulated worlds, where it does.
//!
//! The forced ring story fails on a machine where `io_uring` cannot be used,
//! saying so, as the ring's own tests do.

use skein_echo_client::Plan;
use skein_echo_world::proc::Proc;
use skein_echo_world::referee::{EchoReferee, Expect, Shutdown};
use skein_echo_world::scenarios::{client_limits, plan, server};
use skein_io::Addr;
use skein_lib::{Duration, Rng};
use skein_shell::{Backend, Clock};
use skein_world::{Deadline, Expired, real};

// The echo's real loop on the ring (shell.md, 9).
#[cfg(target_os = "linux")]
#[test]
fn the_echo_answers_and_shuts_down_on_the_real_ring() {
    answers_and_shuts_down(Backend::Ring);
}

#[test]
fn the_echo_answers_and_shuts_down_on_the_readiness_backend() {
    answers_and_shuts_down(Backend::Readiness);
}

fn answers_and_shuts_down(backend: Backend) {
    let clock = Clock::new();
    let start = clock.now().now;
    let mut rng = Rng::new(7);
    let limits = server();
    let listen = Addr::Ip("127.0.0.1:0".parse().expect("an address"));
    let plans = [Plan { lines: 1, ..plan(start, rng.next_u64()) }];
    let by = start.saturating_add(Duration::from_secs(5));
    let expect = vec![Expect::Served { at: 1, conn: 0, by }, Expect::Finished { at: 1, conn: 0, by }];

    let mut world =
        real::World::new_controlled(|controls| EchoReferee::new(7, expect, Shutdown::WhenDone, controls)).on(backend);
    world.spawn_signals(|signal| {
        let mut proc = Proc::echo(limits, listen, rng.next_u64());
        match &mut proc {
            Proc::Echo { svc, .. } => svc.svc.adopt_signals(signal).expect("signal slot"),
            Proc::Client { .. } => unreachable!("echo"),
        }
        proc
    });
    world.spawn(|| Proc::client(client_limits(1), &plans));
    let deadline = Deadline::arm("the real echo exchange and shutdown", Duration::from_secs(10));
    let outcome = world.run(&clock, Duration::from_secs(20));
    if deadline.passed() {
        match deadline.expired() {
            Expired::Starved(starved) => {
                drop(outcome);
                starved.pass();
            }
            Expired::Failed(figures) => panic!("over within ten seconds: {figures}"),
        }
    }
}
