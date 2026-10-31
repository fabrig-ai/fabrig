//! A graceful close driven by io, with an in-memory kernel that holds Recv.
use skein_io::{
    Io, Limits, Request,
    kernel::{Complete, Done, Error, Fd, Op, Submit},
};
use skein_lib::{Env, Queue, Time, Token, Wall};

pub(super) struct Closing {
    io: Io,
    env: Env<Limits>,
    events: Queue<skein_io::Event>,
    completions: Queue<Complete>,
    subs: Queue<Submit>,
    recv: Option<Submit>,
    socket: Option<Token>,
    closed: bool,
    pub cancelled: bool,
}
impl Closing {
    pub fn new(limits: Limits, owner: Token, now: Time) -> Self {
        let mut c = Self {
            io: Io::new(&limits),
            env: Env { now, wall: Wall::EPOCH, limits },
            events: Queue::with_capacity(32),
            completions: Queue::with_capacity(32),
            subs: Queue::with_capacity(32),
            recv: None,
            socket: None,
            closed: false,
            cancelled: false,
        };
        skein_io::down(
            &mut c.io,
            &c.env,
            Request::Connect { owner, addr: skein_io::Addr::Ip(([127, 0, 0, 1], 3000).into()) },
            &mut c.subs,
        );
        c.pump();
        assert!(c.recv.is_some(), "peer keeps its sending half open");
        skein_io::down(&mut c.io, &c.env, Request::Close { entity: c.socket.expect("connected") }, &mut c.subs);
        c.pump();
        assert_eq!(c.io.next_deadline(), Some(now.saturating_add(limits.close_timeout)));
        c
    }
    pub fn deadline(&self) -> Option<Time> {
        self.io.next_deadline()
    }
    pub fn advance(&mut self, now: Time) -> bool {
        self.env.now = now;
        self.pump();
        self.closed
    }
    fn complete(&mut self, submit: Submit, result: Result<Done, Error>) {
        self.completions.push(Complete { op: submit.op, kind: submit.kind, result });
    }
    /// Runs io's up pass, the kernel answering one submission an iteration, until io settles.
    fn pump(&mut self) {
        for _ in 0..64 {
            self.io.reclaim();
            skein_io::stage::up(
                &mut self.io,
                &self.env,
                &mut self.completions,
                &mut self.events,
                &mut self.subs,
                self.env.now,
            );
            if let Some(submit) = self.subs.pop() {
                match &submit.kind {
                    Op::Socket { .. } => self.complete(submit, Ok(Done::Fd(Fd::new(42)))),
                    Op::Connect { .. } | Op::Shutdown { .. } | Op::Close { .. } => {
                        self.complete(submit, Ok(Done::Nothing));
                    }
                    Op::Recv { .. } => {
                        assert!(self.recv.replace(submit).is_none());
                    }
                    Op::Cancel { target } => {
                        let recv = self.recv.take().expect("held receive");
                        assert_eq!(*target, recv.op);
                        self.cancelled = true;
                        self.complete(recv, Err(Error::Cancelled));
                        self.complete(submit, Ok(Done::Nothing));
                    }
                    Op::Bind { .. }
                    | Op::Listen { .. }
                    | Op::Accept { .. }
                    | Op::Send { .. }
                    | Op::Open { .. }
                    | Op::Read { .. }
                    | Op::Write { .. }
                    | Op::Append { .. }
                    | Op::Truncate { .. }
                    | Op::Sync { .. }
                    | Op::Lock { .. }
                    | Op::Stat { .. }
                    | Op::Rename { .. }
                    | Op::Remove { .. }
                    | Op::MakeDirectory { .. }
                    | Op::List { .. }
                    | Op::Spawn { .. }
                    | Op::Wait { .. }
                    | Op::Signal { .. }
                    | Op::Usage
                    | Op::Window { .. }
                    | Op::ReadSignal { .. }
                    | Op::PipeRead { .. }
                    | Op::PipeWrite { .. } => panic!("close path: {submit:?}"),
                }
                continue;
            }
            if self.io.is_ready() || !self.completions.is_empty() {
                continue;
            }
            while let Some(event) = self.events.pop() {
                match event {
                    skein_io::Event::Connecting { socket, .. } => self.socket = Some(socket),
                    skein_io::Event::Connected { .. } => (),
                    skein_io::Event::Closed { .. } => self.closed = true,
                    skein_io::Event::Listening { .. }
                    | skein_io::Event::Accepted { .. }
                    | skein_io::Event::Stream { .. }
                    | skein_io::Event::Output { .. }
                    | skein_io::Event::Spawned { .. }
                    | skein_io::Event::Exited { .. }
                    | skein_io::Event::Usage { .. }
                    | skein_io::Event::Resized
                    | skein_io::Event::Window { .. }
                    | skein_io::Event::Shutdown { .. }
                    | skein_io::Event::Failed { .. } => panic!("close event: {event:?}"),
                }
            }
            self.io.reclaim();
            return;
        }
        panic!("close did not settle");
    }
}
