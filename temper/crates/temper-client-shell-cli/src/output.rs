//! Output is drained through io before the call exits.
use skein_io::{self as io, kernel::Fd};
use skein_lib::{
    Token,
    stream::{Down, Read, Up},
};
use std::collections::VecDeque;

pub(crate) struct Output {
    streams: [Token; 2],
    pending: [VecDeque<Box<[u8]>>; 2],
    waiting: [bool; 2],
    room: u32,
}
impl Output {
    pub(crate) fn new(io: &mut io::Io, stdout: Fd, stderr: Fd, room: u32) -> Self {
        Self {
            streams: [
                io.adopt_write_pipe(stdout, None).expect("stdout slot"),
                io.adopt_write_pipe(stderr, None).expect("stderr slot"),
            ],
            pending: Default::default(),
            waiting: [false; 2],
            room,
        }
    }
    pub(crate) fn push(&mut self, stderr: bool, bytes: Box<[u8]>) {
        if !bytes.is_empty() {
            let index = usize::from(stderr);
            assert!(self.pending[index].len() < 64, "bounded output lines");
            self.pending[index].push_back(bytes);
        }
    }
    pub(crate) fn event(&mut self, owner: Token, up: &Up) -> Option<io::Request> {
        let index = self.streams.iter().position(|stream| *stream == owner)?;
        match up {
            Up::Room => {
                let bytes = self.pending[index].pop_front().expect("output demand has a line");
                let end = bytes.len().min(usize::try_from(self.room).expect("u32 fits"));
                let chunk = bytes[..end].into();
                if end < bytes.len() {
                    self.pending[index].push_front(bytes[end..].into());
                }
                self.waiting[index] = false;
                Some(io::Request::Stream { stream: owner, down: Down::Send(chunk) })
            }
            Up::Failed(_) | Up::End => {
                self.pending[index].clear();
                self.waiting[index] = false;
                None
            }
            Up::Bytes(_) => unreachable!("output pipes never read"),
        }
    }
    pub(crate) fn requests(&mut self) -> Vec<io::Request> {
        let mut requests = Vec::new();
        for index in 0..self.streams.len() {
            if !self.pending[index].is_empty() && !self.waiting[index] {
                self.waiting[index] = true;
                requests.push(io::Request::Stream {
                    stream: self.streams[index],
                    down: Down::Demand { read: Read::Nothing, room: self.room },
                });
            }
        }
        requests
    }
    pub(crate) fn close(&mut self, abort: bool) -> Vec<io::Request> {
        self.pending.iter_mut().for_each(VecDeque::clear);
        self.waiting = [false; 2];
        self.streams
            .iter()
            .map(
                |entity| {
                    if abort { io::Request::Abort { entity: *entity } } else { io::Request::Close { entity: *entity } }
                },
            )
            .collect()
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.pending.iter().all(VecDeque::is_empty) && self.waiting == [false; 2]
    }
}
