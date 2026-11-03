//! Interrupted calls at the readiness backend's seam (shell.md, 7.4), each
//! answered as the kernel would, without interrupting a real call.

use super::{Config, Done, Error, Family, Fd, Op, OpenHow, Queue, Readiness, Submit, Token};

const FD: Fd = Fd::new(123);

/// Puts `kind` in a slot as `start` does (a free slot, its token, the
/// record), attempts it through `answer`, which receives the call's number
/// from 1, then reaps it; returns the calls made and the completion's result.
fn attempted(
    kind: Op,
    resumed: bool,
    mut answer: impl FnMut(u32) -> Result<Done, Error>,
) -> (u32, Result<Done, Error>) {
    let mut kernel = Readiness::open(Config { operations: 1 }).expect("the poller is usable");
    let index = kernel.free.pop().expect("one free slot");
    let token = Token::new(1);
    kernel.tokens.insert(token, index);
    kernel.slot(index).record = Some(Submit { op: token, kind });
    let mut calls = 0_u32;
    let complete = kernel.attempt_with(index, resumed, true, |kind, _, _, _| {
        calls = calls.saturating_add(1);
        assert!(calls <= 3, "{kind:?}: an interrupted call exceeded the test's bound");
        answer(calls)
    });
    let no_waiters = kernel.waiters.is_empty();
    let kind = &kernel.slot(index).record.as_ref().expect("the record stays until reap").kind;
    assert!(complete, "{kind:?}: the attempt completes");
    assert!(no_waiters, "{kind:?}: no waiter was added");
    let mut out = Queue::with_capacity(1);
    kernel.reap(&mut out);
    let completion = out.pop().expect("one completed operation");
    assert_eq!(kernel.in_flight(), 0, "{:?}: nothing remains in flight", completion.kind);
    (calls, completion.result)
}

#[test]
fn an_interrupted_socket_accept_connect_or_open_completes_eintr_after_one_call() {
    for (kind, resumed) in [
        (Op::Socket { family: Family::Ipv4 }, false),
        (Op::Accept { fd: FD }, false),
        (Op::Connect { fd: FD, addr: "127.0.0.1:1234".parse().expect("loopback address") }, false),
        (Op::Connect { fd: FD, addr: "127.0.0.1:1234".parse().expect("loopback address") }, true),
        (Op::Open { root: FD, path: Box::from(*b"file"), how: OpenHow::Read }, false),
    ] {
        let operation = format!("{kind:?}, resumed={resumed}");
        let (calls, result) = attempted(kind, resumed, |_| Err(Error::Other(libc::EINTR)));
        assert_eq!(calls, 1, "{operation}");
        assert_eq!(result, Err(Error::Other(libc::EINTR)), "{operation}");
    }
}

#[test]
fn an_interrupted_close_completes_eintr_after_one_call() {
    let kind = Op::Close { fd: FD };
    let operation = format!("{kind:?}");
    let (calls, result) = attempted(kind, false, |_| Err(Error::Other(libc::EINTR)));
    assert_eq!(calls, 1, "{operation}");
    assert_eq!(result, Err(Error::Other(libc::EINTR)), "{operation}");
}

#[test]
fn an_interrupted_transfer_is_made_again_until_it_completes() {
    for kind in [
        Op::PipeWrite { fd: FD, bytes: Box::from(*b"x"), from: 0 },
        Op::Write { fd: FD, bytes: Box::from(*b"x"), from: 0, at: 0 },
        Op::Append { fd: FD, bytes: Box::from(*b"x"), from: 0 },
        Op::Send { fd: FD, bytes: Box::from(*b"x"), from: 0 },
        Op::PipeRead { fd: FD, buf: Box::from(*b" ") },
        Op::Read { fd: FD, buf: Box::from(*b" "), at: 0 },
        Op::Recv { fd: FD, buf: Box::from(*b" ") },
        Op::ReadSignal { fd: FD },
    ] {
        let operation = format!("{kind:?}");
        let (calls, result) =
            attempted(kind, false, |call| if call < 3 { Err(Error::Other(libc::EINTR)) } else { Ok(Done::Count(1)) });
        assert_eq!(calls, 3, "{operation}");
        assert_eq!(result, Ok(Done::Count(1)), "{operation}");
    }
}
