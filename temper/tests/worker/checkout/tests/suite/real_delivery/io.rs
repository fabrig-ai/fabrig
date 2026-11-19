//! The real component and io on the checkout world's real kernel.
use skein_io::{self as io, kernel};
use skein_lib::{Duration, Queue, Time, Token};
use skein_shell::{Clock, Config, Kernel, Wait};
use std::path::Path;
use temper_worker_domain_checkout::git;
use temper_worker_protocol_git::{Component, Event, Identity, Limits, Outputs, Request, Settings};

const IO: io::Limits = io::Limits { sockets: 6, ..io::Limits::TEST };
const OWNER: Token = Token::new(1);

pub(super) struct Driver {
    component: Component,
    io: io::Io,
    kernel: Kernel,
    clock: Clock,
    root: kernel::Fd,
    out: Outputs,
    submissions: Queue<kernel::Submit>,
    completions: Queue<kernel::Complete>,
    events: Queue<io::Event>,
}
impl Driver {
    pub fn new(root: &Path) -> Self {
        let root_fd = skein_shell::open_root(root).expect("worker root");
        let settings = Settings {
            git: Some(b"/usr/bin/git".as_slice().into()),
            home: root.join("home").as_os_str().as_encoded_bytes().into(),
            root: root_fd,
            prefix: 6_u64 << 60,
            serial: 1000,
            largest_read: IO.largest_read(),
            largest_room: IO.largest_room(),
        };
        let limits = Limits {
            operations: super::LIMITS.workspaces,
            stdout_bytes: 4096,
            stderr_bytes: 4096,
            read_bytes: 64,
            sweep_entries: 2,
            sweep_bytes: 255,
        };
        let mut driver = Self {
            component: Component::new(&limits, &super::LIMITS, settings).expect("git limits"),
            io: io::Io::new(&IO),
            kernel: Kernel::open(Config {
                operations: io::operations(&IO).expect("io operations")
                    + temper_worker_protocol_git::KERNEL_SUBMISSIONS,
            })
            .expect("kernel"),
            clock: Clock::new(),
            root: root_fd,
            out: Outputs { events: Queue::uncapped(), io: Queue::uncapped(), kernel: Queue::uncapped() },
            submissions: Queue::uncapped(),
            completions: Queue::uncapped(),
            events: Queue::uncapped(),
        };
        driver.component.start(driver.clock.now().now, &mut driver.out);
        loop {
            driver.iterate();
            if let Some(event) = driver.out.events.pop() {
                assert_eq!(event, Event::Started, "git starts");
                break;
            }
        }
        driver
    }
    fn iterate(&mut self) {
        let now = self.clock.now();
        let env = now.with(IO);
        self.kernel.reap(&mut self.completions);
        let mut io_completions = Queue::uncapped();
        while let Some(complete) = self.completions.pop() {
            if self.component.owns(complete.op) {
                self.component.complete(now.now, complete, &mut self.out);
            } else {
                io_completions.push(complete);
            }
        }
        io::stage::up(&mut self.io, &env, &mut io_completions, &mut self.events, &mut self.submissions, now.now);
        assert!(io_completions.is_empty(), "unbounded submission room");
        while let Some(event) = self.events.pop() {
            self.component.up(now.now, event, &mut self.out);
        }
        if self.component.is_due(now.now) {
            self.component.fire(now.now, &mut self.out);
        }
        io::stage::down(&mut self.io, &env, &mut self.out.io, &mut self.submissions);
        while let Some(submit) = self.out.kernel.pop() {
            self.submissions.push(submit);
        }
        self.io.reclaim();
        self.component.reclaim();
        let wait = if self.out.events.is_empty() && !self.component.is_empty() {
            Wait::Until(now.now.saturating_add(Duration::from_millis(1)))
        } else {
            Wait::No
        };
        self.kernel.submit(&mut self.submissions, wait);
    }
    pub fn perform(&mut self, op: git::Op, identity: Identity) -> git::Done {
        let now = self.clock.now().now;
        let timeout = if op.is_remote() { super::LIMITS.remote_timeout } else { super::LIMITS.local_timeout };
        self.component.down(
            now,
            Request::Io { owner: OWNER, op, deadline: now.saturating_add(timeout), identity: Some(identity) },
            &mut self.out,
        );
        loop {
            self.iterate();
            if let Some(event) = self.out.events.pop() {
                let Event::Done { owner: OWNER, done } = event else { panic!("operation terminal: {event:?}") };
                return done;
            }
        }
    }
    pub fn close(&mut self) {
        self.component.down(Time::ZERO, Request::Shutdown, &mut self.out);
        while !self.component.is_empty() {
            self.iterate();
        }
        assert!(self.submissions.is_empty());
        assert_eq!(self.kernel.in_flight(), 0);
        skein_shell::close_startup_fd(self.root);
    }
}
