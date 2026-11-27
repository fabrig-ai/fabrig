//! Worker job test inputs and observations (temper/docs/design/protocol/tooling.md, 2).
use crate::Worker;
use alloc::boxed::Box;
use skein_io::{self as io, kernel};
use skein_lib::{Time, Token};
use temper_worker_domain::{
    self as root,
    tooling::{Event, Request},
};

pub mod fixtures;
pub use crate::worker::testing::{jobs_deadline, jobs_empty, jobs_fire};

/// Take the built worker's hello at the domain boundary, without running its service.
#[must_use]
pub fn startup_hello(service: &mut crate::worker_alone::Service) -> Option<jig_domain_link::Hello> {
    let worker = &mut service.worker;
    let mut out = skein_lib::Queue::uncapped();
    root::step(&mut worker.root, &worker.env, root::Event::Ready, &mut out);
    if let Some(root::Request::Hello(hello)) = out.pop() { Some(hello) } else { None }
}

pub const JOB: Token = Token::new(81);

pub fn launch_args(worker: &mut Worker, args: Box<[Box<[u8]>]>, deadline: Time) -> (Token, kernel::Spawn) {
    request_command(worker, args, deadline);
    take_spawn(worker)
}

pub(crate) fn request_command(worker: &mut Worker, args: Box<[Box<[u8]>]>, deadline: Time) {
    worker.job_request(
        JOB,
        Request::Spawn {
            owner: JOB,
            program: args.first().expect("the command is argv[0]").clone(),
            args,
            env: Box::new([
                (b"PATH".as_slice().into(), b"/usr/bin:/bin".as_slice().into()),
                (b"HOME".as_slice().into(), b"workspaces/7".as_slice().into()),
                (b"LANG".as_slice().into(), b"C.UTF-8".as_slice().into()),
                (b"TEMPER_JOB".as_slice().into(), b"workspaces/7".as_slice().into()),
            ]),
            dir: b"workspaces/7".as_slice().into(),
            deadline,
        },
    );
}

pub(crate) fn take_spawn(worker: &mut Worker) -> (Token, kernel::Spawn) {
    let io::Request::Spawn { owner, spawn } = worker.io().pop().expect("job spawn") else { unreachable!("job spawn") };
    (owner, spawn)
}

pub fn event(worker: &mut Worker) -> Event {
    let root::Event::Job { owner, event } = worker.events().pop().expect("job terminal") else {
        unreachable!("job terminal")
    };
    assert_eq!(owner, JOB, "terminal belongs to the fixture job");
    event
}
