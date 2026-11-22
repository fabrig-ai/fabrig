use skein_fake_person_browser::{
    Lent,
    testing::{FIRST, Peer, Peers},
};
use skein_io::kernel::{Addr, Done, Error, Family, Op, Submit};
use skein_lib::{Queue, Token};
use skein_sim::{Config, Pid, Sim};

fn try_listen(sim: &mut Sim, pid: Pid, addr: Addr) -> Result<Done, Error> {
    let mut submissions = Queue::with_capacity(1);
    let mut completions = Queue::with_capacity(1);
    let mut call = |op, kind| {
        let op = Token::new(op);
        submissions.push(Submit { op, kind });
        sim.submit(pid, &mut submissions);
        sim.reap(pid, &mut completions);
        let complete = completions.pop().expect("a probe completion");
        assert_eq!(complete.op, op);
        complete.result
    };
    let Ok(Done::Fd(fd)) = call(1, Op::Socket { family: Family::Ipv4 }) else {
        panic!("a probe socket");
    };
    let result = call(2, Op::Bind { fd, addr }).and_then(|bound| {
        assert_eq!(bound, Done::Bound(addr));
        call(3, Op::Listen { fd, backlog: 1 })
    });
    assert_eq!(call(4, Op::Close { fd }), Ok(Done::Nothing));
    result
}

#[test]
fn a_lent_peer_becomes_ready_after_its_kernel_listen_is_processed() {
    let mut sim = Sim::new(7, Config::calm());
    let listener = sim.spawn_process();
    let probe = sim.spawn_process();
    let mut peer = Peer::new(None);
    let mut lent = [Lent { name: b"listener", peer: &mut peer }];
    let mut peers = Peers::new(&mut lent);
    let mut kernel = (Queue::with_capacity(64), Queue::with_capacity(64));

    let mut listens = 0;
    let mut address = None;
    for _ in 0..64 {
        sim.reap(listener, &mut kernel.0);
        for complete in &kernel.0 {
            if matches!(complete.kind, Op::Listen { .. }) {
                assert!(!peers.ready(), "the peer has yet to process Listen");
                listens += 1;
            }
            assert!(peers.completion(FIRST, complete).is_none(), "startup succeeds");
        }
        peers.drive_with(
            &mut kernel,
            sim.now(),
            sim.wall(),
            address.is_some(),
            |(arrived, _), host, delivered| {
                assert_eq!(host, FIRST);
                while delivered.room() > 0 {
                    let Some(complete) = arrived.pop() else { break };
                    delivered.push(complete);
                }
            },
            |(_, submissions), host, record| {
                assert_eq!(host, FIRST);
                submissions.push(record);
            },
        );
        if address.is_none() && peers.ready() {
            let [(name, bound)] = peers.addresses().try_into().expect("one lent listener");
            assert_eq!(name.as_ref(), b"listener");
            let bound = bound.parse().expect("numeric listener address");
            assert_eq!(try_listen(&mut sim, probe, bound), Err(Error::AddressInUse), "listener remains held");
            address = Some(bound);
        }
        sim.submit(listener, &mut kernel.1);
        if address.is_some() && peers.empty() && sim.in_flight(listener) == 0 {
            break;
        }
    }

    let address = address.expect("listener became ready");
    assert!(peers.empty(), "listener shutdown settles");
    assert!(kernel.0.is_empty());
    sim.assert_quiescent(listener);
    sim.assert_no_open_fds(listener);
    assert!(try_listen(&mut sim, probe, address).is_ok(), "shutdown releases the owned listener");
    sim.assert_quiescent(probe);
    sim.assert_no_open_fds(probe);
    assert_ne!(address.port(), 0);
    assert_eq!(listens, 1);
}
