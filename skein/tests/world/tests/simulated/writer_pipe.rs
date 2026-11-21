use skein_io::kernel::Complete;
use skein_lib::{Time, Wall};
use skein_world::{Host, PipeWriter};
use skein_world_tests::hosted::Script;
use std::io::Write;

#[test]
fn a_writer_refuses_a_full_capture_and_writes_after_its_pipe_close() {
    let (mut writer, capture) = PipeWriter::new(skein_io::kernel::Fd::new(50), 4);
    assert_eq!(writer.write(b"abcdef").expect("partial capture"), 4);
    assert_eq!(writer.write(b"x").expect_err("bounded capture").kind(), std::io::ErrorKind::WouldBlock);
    assert_eq!(writer.flush().expect_err("not yet submitted").kind(), std::io::ErrorKind::WouldBlock);
    let mut host = capture.host(Script::new(&[]));
    host.iterate(Time::ZERO, Wall::from_nanos(0));
    host.drain();
    let write = host.submissions().pop().expect("captured pipe write");
    host.completions().push(Complete {
        op: write.op,
        kind: write.kind,
        result: Err(skein_io::kernel::Error::BrokenPipe),
    });
    host.iterate(Time::ZERO, Wall::from_nanos(0));
    host.drain();
    assert_eq!(writer.write(b"x").expect_err("closed writer").kind(), std::io::ErrorKind::BrokenPipe);
    assert_eq!(host.failure(), Some(skein_io::kernel::Error::BrokenPipe));
    let close = host.submissions().pop().expect("failed pipe still closes");
    host.completions().push(Complete { op: close.op, kind: close.kind, result: Ok(skein_io::kernel::Done::Nothing) });
    host.iterate(Time::ZERO, Wall::from_nanos(0));
    host.drain();
    assert!(host.is_empty());
    writer.flush().expect("no captured bytes remain");
}

#[test]
fn a_pipe_host_gives_the_digest_of_the_host_it_wraps() {
    let script = skein_world_tests::server().noting(3);
    let digest = script.state_digest();
    assert!(digest.is_some());
    let (_writer, capture) = PipeWriter::new(skein_io::kernel::Fd::new(50), 64);
    assert_eq!(capture.host(script).state_digest(), digest);
}
