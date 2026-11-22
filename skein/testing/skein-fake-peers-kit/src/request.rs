//! Literal HTTP requests through a socket while the independent peer progresses.
use skein_fake_peers::llm::Peer;
use skein_io::kernel::{Complete, Done, Op, Submit};
use skein_lib::{Queue, Token};
use skein_shell::Host;
use skein_sim::{Pid, Sim};

pub fn pass(sim: &mut Sim, server: Pid, peer: &mut Peer) {
    sim.reap(server, peer.completions());
    peer.iterate(sim.now(), sim.wall());
    sim.submit(server, peer.submissions());
    sim.advance();
}

pub fn call(sim: &mut Sim, server: Pid, client: Pid, peer: &mut Peer, kind: Op) -> Complete {
    let mut submissions = Queue::with_capacity(1);
    let mut completions = Queue::with_capacity(1);
    submissions.push(Submit { op: Token::new(1), kind });
    sim.submit(client, &mut submissions);
    for _ in 0..1000 {
        sim.reap(client, &mut completions);
        if let Some(complete) = completions.pop() {
            return complete;
        }
        pass(sim, server, peer);
    }
    panic!("the scripted socket operation must settle");
}

pub fn send(sim: &mut Sim, server: Pid, client: Pid, peer: &mut Peer, fd: skein_io::kernel::Fd, bytes: &[u8]) {
    let mut from = 0;
    while usize::try_from(from).expect("request length") < bytes.len() {
        let complete = call(sim, server, client, peer, Op::Send { fd, bytes: bytes.into(), from });
        let Done::Count(sent) = complete.result.expect("literal request sent") else { panic!("send count") };
        from += sent;
    }
}
