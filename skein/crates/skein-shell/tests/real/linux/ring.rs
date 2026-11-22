#[expect(unsafe_code, reason = "the fixture queues a blocked signal on its calling thread")]
mod tests {
    use skein_io::kernel::{Done, Op, ServiceSignal, Submit};
    use skein_lib::{Duration, Queue, Token};
    use skein_shell::testing::Ring;
    use skein_shell::{Clock, Config, Wait, open_termination_signals};

    #[test]
    fn blocked_termination_signal_arrives_through_the_ring() {
        let fd = open_termination_signals().expect("block the signals and open signalfd");
        let mut kernel = Ring::open(Config { operations: 2 }).expect("open the ring");
        let mut submits = Queue::with_capacity(1);
        submits.push(Submit { op: Token::new(1), kind: Op::ReadSignal { fd } });
        kernel.submit(&mut submits, Wait::No);

        // SAFETY: pthread_self returns this live test thread's identifier.
        let thread = unsafe { libc::pthread_self() };
        // SAFETY: SIGTERM is blocked on this thread, so pthread_kill queues
        // it for the signalfd instead of invoking its default action.
        assert_eq!(unsafe { libc::pthread_kill(thread, libc::SIGTERM) }, 0);
        let deadline = Clock::new().now().now.saturating_add(Duration::from_secs(1));
        let mut answers = Queue::with_capacity(1);
        for _ in 0..3 {
            kernel.submit(&mut submits, Wait::Until(deadline));
            kernel.reap(&mut answers);
            if !answers.is_empty() {
                break;
            }
        }
        let answer = answers.pop().expect("the signal read completed");
        assert_eq!(answer.kind, Op::ReadSignal { fd });
        assert_eq!(
            answer.result,
            Ok(Done::Signalled(skein_io::kernel::Signalled::Termination {
                signal: ServiceSignal::Terminate,
                sender: Some(skein_io::kernel::Sender::new(u64::from(std::process::id())))
            }))
        );

        submits.push(Submit { op: Token::new(2), kind: Op::Close { fd } });
        kernel.submit(&mut submits, Wait::No);
        let deadline = Clock::new().now().now.saturating_add(Duration::from_secs(1));
        for _ in 0..3 {
            kernel.submit(&mut submits, Wait::Until(deadline));
            kernel.reap(&mut answers);
            if !answers.is_empty() {
                break;
            }
        }
        let answer = answers.pop().expect("the signal descriptor closed");
        assert_eq!(answer.kind, Op::Close { fd });
        assert_eq!(answer.result, Ok(Done::Nothing));
    }
}
