use crate::lines::joined;
use crate::pipe::{self, Command, Line, Reader};
use skein_io::kernel::Exit;
use skein_lib::List;
use temper_forge_setup::Stage;
#[test]
fn a_bounded_reader_discards_a_long_line_through_its_feed() {
    let rows: &[(&[u8], &[Line])] = &[
        (b"\n".as_slice(), &[Line::Whole(b"\n".as_slice().into())]),
        (b"123\n", &[Line::Whole(b"123\n".as_slice().into())]),
        (b"1234\nstart", &[Line::TooLong]),
        (b"12345\nx\n", &[Line::TooLong, Line::Whole(b"x\n".as_slice().into())]),
        (b"12345", &[]),
        (b"123", &[]),
    ];
    for (input, expected) in rows {
        let mut reader = Reader::new(4);
        let mut got = List::with_capacity(4);
        for byte in *input {
            if let Some(line) = reader.byte(*byte) {
                got.push(line).unwrap();
            }
        }
        assert_eq!(got.as_slice(), *expected, "{input:?}");
    }
}
#[test]
fn only_canonical_start_and_stop_lines_are_known() {
    for (line, expected) in [
        (b"start 0\n".as_slice(), Command::Start(0)),
        (b"stop 0\n", Command::Stop(0)),
        (b"stop 65535\n", Command::Stop(65535)),
        (b"stop 3\n", Command::Stop(3)),
        (b"start 65535\n", Command::Start(65535)),
        (b"start 3\n", Command::Start(3)),
        (b"start 65536\n", Command::Unknown),
        (b"start 01\n", Command::Unknown),
        (b"start +1\n", Command::Unknown),
        (b"start -1\n", Command::Unknown),
        (b"start  1\n", Command::Unknown),
        (b"start 1 \n", Command::Unknown),
        (b"start 1\r\n", Command::Unknown),
        (b"start 1", Command::Unknown),
        (b"start \n", Command::Unknown),
        (b"start 999999999999999999999\n", Command::Unknown),
        (b"other 1\n", Command::Unknown),
    ] {
        assert_eq!(pipe::command(line), expected, "{line:?}");
        if let Some(suffix) = line.strip_prefix(b"start ") {
            let stopped = joined(&[b"stop ", suffix]);
            let expected = match expected {
                Command::Start(forge) => Command::Stop(forge),
                Command::Unknown => Command::Unknown,
                Command::Stop(_) => unreachable!(),
            };
            assert_eq!(pipe::command(&stopped), expected, "{stopped:?}");
        }
    }
}
#[test]
fn box_lines_and_failure_words_have_the_wire_spelling_and_bound() {
    for (stage, line) in [
        (Stage::Fetch, b"step 3 fetch\n".as_slice()),
        (Stage::Prepare, b"step 3 prepare\n"),
        (Stage::Run, b"step 3 run\n"),
    ] {
        assert_eq!(&*pipe::step(3, stage), line);
    }
    assert_eq!(&*pipe::started(65535), b"started 65535\n");
    assert_eq!(&*pipe::stopped(65535), b"stopped 65535\n");
    for (why, expected) in [
        (pipe::exited(b"prepare 3", Exit::Code(1)), b"prepare 3 exited 1".as_slice()),
        (pipe::exited(b"forgejo 3", Exit::Signal(9)), b"forgejo 3 killed by signal 9"),
        (pipe::not_started(b"prepare 3"), b"prepare 3 did not start"),
        (pipe::not_written().into(), b"record not written"),
    ] {
        assert_eq!(&*why, expected);
        assert_eq!(&*pipe::failed(3, &why, 64), &*joined(&[b"failed 3 ", expected, b"\n"]));
    }
    for bound in [64, 65, 256] {
        for forge in [0, 65535] {
            for length in [0, 40, 64, 256, 1000] {
                let mut why = List::with_capacity(u32::try_from(length).unwrap());
                for _ in 0..length {
                    why.push(b'x').unwrap();
                }
                let line = pipe::failed(forge, why.as_slice(), bound);
                assert!(line.len() <= usize::try_from(bound).unwrap());
                assert_eq!(line.last(), Some(&b'\n'));
                assert_eq!(
                    line.len(),
                    (pipe::failed(forge, &[], bound).len() + length).min(usize::try_from(bound).unwrap())
                );
            }
        }
    }
}
