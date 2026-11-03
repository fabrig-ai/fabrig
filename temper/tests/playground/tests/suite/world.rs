//! The browser's calm starting world (temper/docs/design/playground.md, 4).

use skein_io::kernel::Exit;
use skein_lib::Wall;
use skein_sim::{Faults, Trace};
use skein_sim_world::Host;
use temper_playground::{programs::Process, stories::A};
use temper_playground_world::{digests, printed_link, start_idle};

const SEED: u64 = 7;
const OTHER_SEED: u64 = 8;
const WALL: Wall = Wall::EPOCH;

#[test]
fn story_a_starts_with_no_box_calm_and_prints_its_link() {
    let mut started = start_idle(&A, SEED, WALL);
    let link = printed_link(&started.printed);
    let (_, secret) = link.split_once("/jig/v1/sign-in/link?secret=").expect("the service's sign-in link");
    assert!(!secret.is_empty() && !secret.contains(['\r', '\n']), "one whole link line");

    assert!(
        started.world.processes().iter().all(|process| {
            matches!(process, Process::Service(_) | Process::Peer(_) | Process::Forge(_) | Process::Git(_))
        }),
        "only the service, its two fake peers and git; no box process"
    );
    for process in started.world.processes() {
        let exit = process.exit();
        assert!(
            exit.is_none() || (matches!(process, Process::Git(_)) && exit == Some(Exit::Code(0))),
            "calm startup keeps the service and fake peers running; only hosted git may end, with Code(0); got {exit:?}"
        );
    }
    let (sim, _, processes) = started.world.parts();
    let Process::Service(service) = &processes[0] else { panic!("the service is process zero") };
    assert!(service.service.end().is_none(), "the service waits for its visitor");
    assert_eq!(sim.config().faults, Faults::NONE);
    assert_eq!(sim.config().trace, Trace::Off);
    assert!(sim.trace().is_empty(), "no trace retained");
}

#[test]
fn the_clients_listener_is_at_the_documents_address() {
    let mut started = start_idle(&A, SEED, WALL);
    let document: serde_json::Value = serde_json::from_slice(A.deployment).expect("story A's deployment document");
    let address = document["application"]["clients"]["listener"]["socket"].as_str().expect("clients socket");
    let expected = address.parse::<std::net::SocketAddr>().expect("clients address");
    let Process::Service(service) = started.world.process(0) else { panic!("the service is process zero") };
    let bound: Vec<_> = service
        .service
        .listening()
        .iter()
        .filter_map(|note| {
            if let temper_service::Note::Listening { listener: temper_service::Listening::Clients, addr } = note {
                Some(*addr)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(bound, [expected], "the service's bound clients listener");
    let link = printed_link(&started.printed);
    let (authority, _) =
        link.strip_prefix("http://").expect("plaintext playground link").split_once('/').expect("URL path");
    assert_eq!(authority, address);
}

#[test]
fn one_seed_makes_one_world() {
    let first = start_idle(&A, SEED, WALL);
    let second = start_idle(&A, SEED, WALL);
    assert_eq!(digests(&first.world), digests(&second.world), "one seed replays the world");
    assert_eq!(first.printed, second.printed, "one seed replays its output");
    let other = start_idle(&A, OTHER_SEED, WALL);
    assert_ne!(printed_link(&first.printed), printed_link(&other.printed), "another seed changes the sign-in link");
}
