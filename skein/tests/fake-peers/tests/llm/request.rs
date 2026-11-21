//! Literal HTTP requests over the simulator.
use skein_fake_peers::llm::Peer;
pub use skein_fake_peers_kit::request::{call, pass, send};
use skein_io::kernel::{Done, Family, Op};
use skein_sim::{Pid, Sim};

pub fn request(
    sim: &mut Sim,
    server: Pid,
    client: Pid,
    peer: &mut Peer,
    path: &[u8],
    fields: &[u8],
    body: &[u8],
) -> Vec<u8> {
    for _ in 0..1000 {
        if peer.address().is_some() {
            break;
        }
        pass(sim, server, peer);
    }
    let complete = call(sim, server, client, peer, Op::Socket { family: Family::Ipv4 });
    let Done::Fd(fd) = complete.result.expect("socket opened") else { panic!("socket descriptor") };
    call(sim, server, client, peer, Op::Connect { fd, addr: peer.address().expect("listening peer") })
        .result
        .expect("connected to peer");
    let mut head = b"POST ".to_vec();
    head.extend_from_slice(path);
    head.extend_from_slice(
        b" HTTP/1.1\r\nhost: localhost\r\ncontent-type: application/json\r\nconnection: close\r\ncontent-length: ",
    );
    head.extend_from_slice(body.len().to_string().as_bytes());
    head.extend_from_slice(b"\r\n");
    head.extend_from_slice(fields);
    head.extend_from_slice(b"\r\n");
    send(sim, server, client, peer, fd, &head);
    send(sim, server, client, peer, fd, body);
    let mut response = Vec::new();
    loop {
        let complete = call(sim, server, client, peer, Op::Recv { fd, buf: Box::new([0; 1024]) });
        let Done::Count(count) = complete.result.expect("response read") else { panic!("read count") };
        if count == 0 {
            break;
        }
        let Op::Recv { buf, .. } = complete.kind else { panic!("receive buffer") };
        response.extend_from_slice(&buf[..usize::try_from(count).expect("read count")]);
    }
    call(sim, server, client, peer, Op::Close { fd }).result.expect("socket closed");
    response
}
