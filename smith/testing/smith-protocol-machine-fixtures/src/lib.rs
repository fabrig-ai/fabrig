//! Small machine limits for deterministic scenarios (testing-strategy.md, 1).
//! Shared fixtures (smith/docs/design/testing.md, section 7).

#![forbid(unsafe_code)]

use skein_io::{Event, Request, kernel};
use skein_lib::{Duration, Env, Queue, Time, Token, Wall, stream::Up};
use smith_protocol_machine::{Below, BelowEvent, Component, FromDomain, Limits, ToDomain};

/// Small file, process and output admission edges.
pub const SMALL: Limits = Limits {
    operations: 2,
    roots: 1,
    path_bytes: 64,
    file_bytes: 16,
    tool_payload: 64,
    entries: 4,
    entry_bytes: 512,
    processes: 2,
    output_bytes: 64,
    search_hits: 8,
    search_bytes: 256,
    search_line_bytes: 512,
    env_bytes: 512,
    group_stop: Duration::from_millis(10),
};

const CHILD: Token = Token::new(40);
/// Fake parent pipe attached to a check's stdout.
pub const STDOUT: Token = Token::new(41);
/// Fake parent pipe attached to a check's stderr.
pub const STDERR: Token = Token::new(42);

/// One real check machine, with deterministic time and fake process/pipe completions.
pub struct CheckProcess {
    owner: Token,
    machine: Component,
    env: Env<Limits>,
    above: Queue<ToDomain>,
    below: Queue<Below>,
}

impl CheckProcess {
    /// Start and acknowledge one check's spawn in a writable workspace.
    #[must_use]
    pub fn start(request: FromDomain, now: Time) -> Self {
        let mut check = Self::request(request, now);
        check.event(BelowEvent::Process(Event::Spawned {
            owner: check.owner,
            child: CHILD,
            pipes: Box::new([STDOUT, STDERR]),
        }));
        check.clear_reads();
        check
    }

    /// Refuse the spawn and settle the unopened process, without attaching pipes.
    #[must_use]
    pub fn unstarted(request: FromDomain, now: Time) -> ToDomain {
        let mut check = Self::request(request, now);
        check.event(BelowEvent::Process(Event::Failed { owner: check.owner, error: skein_io::Error::Other }));
        check.settle()
    }

    fn request(request: FromDomain, now: Time) -> Self {
        let FromDomain::Check { owner, program, tail, .. } = &request else { panic!("check request") };
        let limits = Limits { output_bytes: *tail, ..SMALL };
        let mut machine = Component::new(
            &limits,
            smith_protocol_machine::Environment::encode(&[], &limits).expect("environment"),
            Box::from(b"/usr/bin/rg".as_slice()),
        );
        machine.workspace(&smith_domain::run::Workspace {
            directories: Box::new([smith_domain::run::Directory {
                results: None,
                hooks: None,
                host: b"/host".as_slice().into(),
                name: b"code".as_slice().into(),
                root: program.root,
                writable: true,
                git: true,
                conflicts: Box::new([]),
            }]),
        });
        let mut check = Self {
            owner: *owner,
            machine,
            env: Env { now, wall: Wall::EPOCH, limits },
            above: Queue::uncapped(),
            below: Queue::uncapped(),
        };
        check.machine.from_domain(&check.env, request, &mut check.above, &mut check.below);
        assert!(matches!(check.below.pop(), Some(Below::Spawn { .. })));
        check
    }

    /// Set the injected time for the next completion or timer.
    pub fn at(&mut self, now: Time) {
        self.env.now = now;
    }

    /// Time at which this machine next needs to be driven.
    #[must_use]
    pub fn deadline(&self) -> Time {
        self.machine.next_deadline().expect("live check deadline")
    }

    /// Deliver exactly one read from the selected output pipe.
    pub fn bytes(&mut self, pipe: Token, bytes: &[u8]) {
        self.event(BelowEvent::Pipe(Event::Stream { owner: pipe, up: Up::Bytes(bytes.into()) }));
        self.clear_reads();
    }

    /// Run the check machine's timer at the injected time.
    pub fn fire(&mut self) {
        self.machine.fire(&self.env, &mut self.above, &mut self.below);
    }

    /// No stop or other machine effect has been requested.
    #[must_use]
    pub fn quiet(&self) -> bool {
        self.below.is_empty()
    }

    /// Consume the request to terminate the process group, leaving the pipes open.
    pub fn assert_stopped(&mut self) {
        assert_eq!(
            self.below.pop(),
            Some(Below::Process(Request::Signal {
                child: CHILD,
                to: kernel::Target::Group,
                signal: kernel::Signal::Terminate,
                reason: kernel::Reason::Unknown,
            }))
        );
        assert!(self.quiet());
    }

    /// Report the leader's exit; a final pipe read may still arrive.
    pub fn exited(&mut self, exit: kernel::Exit) {
        self.event(BelowEvent::Process(Event::Exited { owner: self.owner, exit }));
        if let Some(effect) = self.below.pop() {
            assert_eq!(effect, Below::Process(Request::Close { entity: CHILD }));
        }
    }

    /// Settle the pipes and process, then take the one terminal.
    #[must_use]
    pub fn finish(mut self) -> ToDomain {
        for pipe in [STDOUT, STDERR] {
            self.event(BelowEvent::Pipe(Event::Closed { owner: pipe }));
        }
        self.settle()
    }

    fn settle(mut self) -> ToDomain {
        self.event(BelowEvent::Process(Event::Closed { owner: self.owner }));
        let answer = self.above.pop().expect("one check terminal");
        assert!(self.above.is_empty());
        self.machine.close(&mut self.below);
        assert!(self.machine.closed());
        answer
    }

    fn event(&mut self, event: BelowEvent) {
        self.machine.from_below(&self.env, event, &mut self.above, &mut self.below);
    }

    fn clear_reads(&mut self) {
        while let Some(request) = self.below.pop() {
            assert!(matches!(request, Below::Process(Request::Stream { .. })), "unexpected stop: {request:?}");
        }
    }
}
