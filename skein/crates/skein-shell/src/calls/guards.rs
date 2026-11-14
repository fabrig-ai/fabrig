//! Path-only admission with supplied descriptor flags (runtime/readiness.md, 5).

use super::{Fd, Op, check_path_flags, path_descriptors};

const FD: Fd = Fd::new(123);

#[test]
fn ordinary_reads_and_path_only_stat_and_close_pass_the_guard() {
    let record = Op::Read { fd: FD, buf: Box::from([0; 1]), at: 0 };
    assert_eq!(path_descriptors(&record), [Some(FD), None]);
    check_path_flags(Ok(0));
    check_path_flags(Err(libc::EBADF));
    assert_eq!(path_descriptors(&Op::Stat { fd: FD }), [None, None]);
    assert_eq!(path_descriptors(&Op::Close { fd: FD }), [None, None]);
}

#[test]
fn every_file_data_operation_routes_its_descriptor_through_the_flags_guard() {
    for operation in [
        Op::Read { fd: FD, buf: Box::from([0; 1]), at: 0 },
        Op::Write { fd: FD, bytes: Box::from(*b"x"), from: 0, at: 0 },
        Op::Append { fd: FD, bytes: Box::from(*b"x"), from: 0 },
        Op::Truncate { fd: FD, length: 0 },
        Op::Sync { fd: FD },
        Op::Lock { fd: FD },
    ] {
        assert_eq!(path_descriptors(&operation), [Some(FD), None], "{operation:?}");
    }
}
