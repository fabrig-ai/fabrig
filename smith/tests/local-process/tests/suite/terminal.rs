use skein_io::kernel;
use skein_lib::{Time, Wall};
use skein_sim::{Config, Event, Sim, Summary};
use skein_world::Host;
use smith_local_process_world::terminal::{Action, Terminal};

#[test]
fn a_refusal_records_a_failed_write_before_exit_and_drains() {
    for error in [kernel::Error::Other(5), kernel::Error::Other(4)] {
        refusal_write_failure(error, false);
    }
}

#[test]
fn a_refusal_records_a_failed_write_after_exit_and_drains() {
    for error in [kernel::Error::Other(5), kernel::Error::Other(4)] {
        refusal_write_failure(error, true);
    }
}

#[test]
#[should_panic(expected = "terminal write failed")]
fn an_interactive_terminal_panics_on_a_failed_write() {
    let mut person = Terminal::attached(kernel::Fd::new(10), vec![Action::Eof]);
    let (_, write) = start_eof(&mut person);
    person.completions().push(kernel::Complete {
        op: write.op,
        kind: write.kind,
        result: Err(kernel::Error::Other(5)),
    });
    person.iterate(Time::ZERO, Wall::EPOCH);
}

fn start_eof(person: &mut Terminal) -> (kernel::Submit, kernel::Submit) {
    assert_eq!(person.failed_write(), None);
    person.iterate(Time::ZERO, Wall::EPOCH);
    let read = person.submissions().pop().expect("terminal read");
    assert!(matches!(read.kind, kernel::Op::PipeRead { .. }));
    let write = person.submissions().pop().expect("terminal EOF write");
    assert!(matches!(&write.kind, kernel::Op::PipeWrite { bytes, from: 0, .. } if bytes.as_ref() == [4]));
    assert!(person.submissions().is_empty());
    (read, write)
}

fn refusal_write_failure(error: kernel::Error, exit_first: bool) {
    let stream = kernel::Fd::new(10);
    let mut person =
        Terminal::attached_to_a_refusal(stream, vec![Action::Eof, Action::Send(b"more input\n".as_slice().into())]);
    let (read, write) = start_eof(&mut person);
    let exit = kernel::Exit::Code(1);
    if exit_first {
        person.finished(exit);
        person.iterate(Time::ZERO, Wall::EPOCH);
        assert!(!person.is_empty(), "exit waits for terminal completions");
        assert!(person.submissions().is_empty());
    }
    person.completions().push(kernel::Complete { op: write.op, kind: write.kind, result: Err(error) });
    person.iterate(Time::ZERO, Wall::EPOCH);
    assert_eq!(person.failed_write(), Some(error));
    assert!(person.submissions().is_empty(), "failed input is not retried and the remaining script stops");
    assert!(!person.is_empty(), "write failure still waits for terminal output to drain");

    person.completions().push(kernel::Complete { op: read.op, kind: read.kind, result: Ok(kernel::Done::Count(0)) });
    person.iterate(Time::ZERO, Wall::EPOCH);
    assert!(!person.is_empty(), "output end still waits for exit and close completion");
    if !exit_first {
        assert_eq!(person.exit(), None, "write failure does not manufacture an exit");
        assert!(person.submissions().is_empty(), "output end waits for exit before closing");
        person.finished(exit);
        person.iterate(Time::ZERO, Wall::EPOCH);
    }
    let close = person.submissions().pop().expect("terminal close after output end and exit");
    assert!(matches!(close.kind, kernel::Op::Close { fd } if fd == stream));
    assert!(person.submissions().is_empty());
    assert!(!person.is_empty(), "close completion is still outstanding");
    person.completions().push(kernel::Complete { op: close.op, kind: close.kind, result: Ok(kernel::Done::Nothing) });
    person.iterate(Time::ZERO, Wall::EPOCH);
    assert!(person.is_empty(), "refusal drains every terminal operation");
    assert_eq!(person.exit(), Some(exit));
    assert_eq!(person.failed_write(), Some(error));
    assert!(person.shown().is_empty());
    assert!(person.errors().is_empty());
}

#[test]
fn a_piped_person_shows_whole_output_in_at_most_sixteen_reads_then_closes() {
    let bytes: Vec<u8> = (0..60_000).map(|index| u8::try_from(index % 251).expect("byte")).collect();
    let mut sim = Sim::new(7, Config { buffer: 60_000, ..Config::calm() });
    let process = sim.spawn_process();
    let input = sim.open_inherited_write(process);
    let output = sim.open_inherited_read(process);
    assert_eq!(sim.peer_feed(process, output, &bytes), bytes.len());
    sim.peer_close(process, output);
    let mut person = Terminal::piped(input, output, vec![]);
    person.finished(kernel::Exit::Code(0));
    for _ in 0..32 {
        person.iterate(Time::ZERO, Wall::EPOCH);
        assert!(!person.is_empty(), "person waits for the output close completion");
        let closing =
            person.submissions().iter().any(|submit| matches!(submit.kind, kernel::Op::Close { fd } if fd == output));
        sim.submit(process, person.submissions());
        sim.reap(process, person.completions());
        if closing {
            person.iterate(Time::ZERO, Wall::EPOCH);
            break;
        }
    }
    assert_eq!(person.shown(), bytes);
    let reads = sim
        .trace()
        .iter()
        .filter(|entry| matches!(entry.event, Event::Submit { kind: Summary::PipeRead { fd, .. }, .. } if fd == output))
        .count();
    assert!(reads <= 16, "whole output and EOF took {reads} pipe reads");
    assert!(person.is_empty());
    sim.assert_quiescent(process);
    sim.assert_no_open_fds(process);
}
