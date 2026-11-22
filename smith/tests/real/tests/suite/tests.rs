use skein_shell::Host;
use smith_local_process_world::{Placement, World as Simulated, process::Proc};
use smith_real_world::{Scratch, address, listener_fds, testing};

#[test]
fn macos_real_peers_serve_on_their_distinct_held_listeners() {
    use smith_agent_process_world::fake;
    use smith_local_process_world::world::Authentication;

    let scratch = Scratch::new();
    let _held = fake::real_loopback_on(true);
    let provider = address(34_443);
    let issuer = address(34_444);
    assert_eq!(provider.ip(), std::net::Ipv4Addr::LOCALHOST);
    assert_eq!(issuer.ip(), std::net::Ipv4Addr::LOCALHOST);
    assert_ne!(provider.port(), 0);
    assert_ne!(issuer.port(), 0);
    assert_ne!(provider.port(), issuer.port(), "two held listeners cannot share a port");
    for (port, expected) in [(34_443, provider), (34_444, issuer)] {
        let fds = listener_fds(port);
        assert_eq!(skein_shell::listener_at(*fds.first().expect("held descriptor")), Ok(expected));
    }
    let scenario =
        Simulated::authenticated(12, testing::files(&scratch), Placement::InProcess, Authentication::SignIn).scenario;
    let mut world = testing::configured(scenario, &scratch);
    world.settle();
    let outcome = testing::outcome(&world).expect("drained real world");
    let peer = outcome
        .procs
        .iter()
        .find_map(|proc| match proc {
            Proc::Peer(peer) => Some(peer),
            _ => None,
        })
        .expect("LLM peer");
    let oauth = outcome
        .procs
        .iter()
        .find_map(|proc| match proc {
            Proc::Issuer(peer) => Some(peer.peer()),
            _ => None,
        })
        .expect("OAuth peer");
    assert_eq!(peer.address(), Some(provider), "LLM announces the held address");
    assert_eq!(oauth.address(), Some(issuer), "OAuth announces the held address");
    assert!(fake::replied(peer), "LLM answered its client");
    assert_eq!(smith_local_process_world::oauth::posts(oauth), 1, "issuer answered its client");
    assert!(outcome.procs.iter().all(Host::is_empty), "clients and peers drained");
}
