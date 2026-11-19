//! Run each native batch member through the fixture's own single-message route.
//! jig/docs/design/protocol/client/wire.md, 2.3.
use super::*;
impl Frame {
    pub(super) fn batch(&mut self, fd: Fd) {
        let request = &self.sockets.get(&fd).expect("batch socket").request;
        let end = request.windows(4).position(|b| b == b"\r\n\r\n").expect("batch head");
        let batch =
            w::Batch::decode(&w::CEILINGS, &mut skein_lib::Reader::new(&request[end + 4..])).expect("batch body");
        let mut frames = Vec::new();
        let mut complete = true;
        for (place, member) in batch.members.into_boxed().into_iter().enumerate() {
            let (route, body) = match member {
                w::Member::Read(read) => ("/jig/v1/read", encode!(read)),
                w::Member::Open(open) => ("/jig/v1/watch", encode!(open)),
                w::Member::Close(close) => ("/jig/v1/unwatch", encode!(close)),
            };
            let child = Fd::new(self.next_socket);
            self.next_socket += 1;
            let mut bytes = format!("POST {route} HTTP/1.1\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
            bytes.extend_from_slice(&body);
            self.sockets.insert(child, Socket { request: bytes, ..Socket::default() });
            let before = self.requests.len();
            self.http_request(child);
            self.requests.truncate(before); // A member is not another physical HTTP request.
            let socket = self.sockets.remove(&child).expect("member route finished");
            let response: Vec<_> = socket.reply.into_iter().collect();
            if response.is_empty() {
                complete = false; // A quiet member keeps this exchange unfinished, like its lone route.
                continue;
            }
            let end = response.windows(4).position(|b| b == b"\r\n\r\n").expect("member response head");
            let reply = response.starts_with(b"HTTP/1.1 200 ").then(|| response[end + 4..].into());
            let part = w::BatchReply { member: u16::try_from(place).expect("member"), reply };
            let bytes = encode!(part);
            frames.extend_from_slice(&part.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture").to_be_bytes());
            frames.extend_from_slice(&bytes);
        }
        if complete {
            self.reply(fd, &frames);
        } else {
            let response = &mut self.sockets.get_mut(&fd).expect("batch socket").reply;
            response.extend(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n");
            if !frames.is_empty() {
                response.extend(format!("{:x}\r\n", frames.len()).bytes());
                response.extend(frames);
                response.extend(b"\r\n");
            }
        }
    }
}
