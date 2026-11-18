//! Paced batch frames, one call through the body's end (protocol/client/default-http.md, 6).
use crate::{Limits, call::Call};
use alloc::boxed::Box;
use skein_http::server as http;
use skein_lib::{Env, Queue, Writer, bytes, stream::Down};
#[derive(Debug)]
pub(crate) struct Batch {
    frames: Queue<Box<[u8]>>,
    writing: bool,
    done: bool,
}
impl Call {
    fn batch_open(&mut self) {
        if self.batch.is_some() {
            return;
        }
        self.batch = Some(Batch { frames: Queue::with_capacity(64), writing: false, done: false });
        self.requests.push(http::Request::Discard);
        self.requests.push(http::Request::Respond(http::Response {
            status: 200,
            headers: Box::new([
                skein_http::Header {
                    name: bytes::copy_of(b"content-type"),
                    value: bytes::copy_of(b"application/octet-stream"),
                },
                skein_http::Header { name: bytes::copy_of(b"cache-control"), value: bytes::copy_of(b"no-store") },
                skein_http::Header {
                    name: bytes::copy_of(b"x-content-type-options"),
                    value: bytes::copy_of(b"nosniff"),
                },
            ]),
            body: http::Body::Chunked,
            close: false,
        }));
    }
    pub(super) fn part(&mut self, env: &Env<Limits>, message: Box<[u8]>) {
        self.batch_open();
        let batch = self.batch.as_mut().expect("opened batch");
        assert!(!batch.done, "parts precede Done");
        let unwritten = batch.frames.len().saturating_add(u32::from(batch.writing));
        assert!(unwritten < 64, "one frame per batch member");
        let length = u32::try_from(message.len()).expect("bounded batch reply");
        let mut frame = Writer::new(message.len().checked_add(4).expect("bounded frame"));
        frame.put(&length.to_be_bytes()).expect("frame length");
        frame.put(&message).expect("frame record");
        batch.frames.push(frame.finish());
        self.batch_flush(&env.limits);
    }
    pub(super) fn batch_done(&mut self, env: &Env<Limits>) {
        self.batch_open();
        self.batch.as_mut().expect("opened batch").done = true;
        self.batch_flush(&env.limits);
    }
    fn batch_flush(&mut self, limits: &Limits) {
        let batch = self.batch.as_mut().expect("batch writer");
        if batch.writing {
            return;
        }
        if let Some(frame) = batch.frames.pop() {
            batch.writing = true;
            self.response = frame;
            self.offset = 0;
            self.room(limits);
        } else if batch.done {
            self.requests.push(http::Request::Reply(Down::Finish));
        }
    }
    pub(super) fn batch_send(&mut self, limits: &Limits) {
        let end = self.offset.saturating_add(skein_lib::length::usize_of(limits.send)).min(self.response.len());
        self.requests.push(http::Request::Reply(Down::Send(bytes::copy_of(
            self.response.get(self.offset..end).expect("frame slice"),
        ))));
        self.offset = end;
        if end == self.response.len() {
            self.response = Box::new([]);
            self.batch.as_mut().expect("batch writer").writing = false;
            self.batch_flush(limits);
        } else {
            self.room(limits);
        }
    }
}
