//! Pure helper protocol and Landlock decisions (runtime/kernel-cgroups.md, 1).

use super::{Record, Spec, abi_error, admit_abi, gaps, rule_rights};
use skein_io::kernel::{Reach, Rights, Unenforced};

#[test]
fn spec_preserves_command_bytes_and_named_descriptors() {
    let spec = Spec {
        program: Box::from(b"./a\xff".as_slice()),
        args: Box::new([Box::from(b"".as_slice()), Box::from(b"spaces \xfe".as_slice())]),
        env: Box::new([Box::from(b"A=b c".as_slice()), Box::from(b"EMPTY=".as_slice())]),
        named: Box::new([0, 2, 5, 9]),
    };
    assert_eq!(Spec::decode(&spec.encode().expect("bounded spec")), Ok(spec), "all command bytes survive");
}

#[test]
fn the_largest_admitted_command_fits_the_spec_pipe() {
    // Program's length prefix consumes four of COMMAND's counted bytes.
    let spec = Spec {
        program: vec![b'x'; Reach::COMMAND.checked_sub(4).expect("prefix fits")].into_boxed_slice(),
        args: Box::new([]),
        env: Box::new([]),
        named: Box::new([]),
    };
    let encoded = spec.encode().expect("the largest admitted command encodes");
    assert!(encoded.len() <= 65_536, "a complete spec fits a 64 KiB pipe");
    assert_eq!(Spec::decode(&encoded), Ok(spec), "the maximum command survives");
}

#[test]
fn reports_preserve_stage_and_errno() {
    for record in [Record::Restricted, Record::Restrict(libc::EPERM), Record::Command(libc::EACCES)] {
        assert_eq!(Record::decode(&record.encode()), Some(record), "report {record:?}");
    }
}

#[test]
fn abi_admits_the_floor_or_names_what_is_missing() {
    for (answer, expected) in [
        (Err(abi_error(libc::ENOSYS)), Err(Unenforced::Absent)),
        (Err(abi_error(libc::EOPNOTSUPP)), Err(Unenforced::Disabled)),
        (Ok(5), Err(Unenforced::Abi { found: 5, needs: 6 })),
        (Ok(6), Ok(())),
        (Ok(7), Ok(())),
    ] {
        assert_eq!(admit_abi(answer, 6), expected, "ABI answer {answer:?}");
    }
}

#[test]
fn a_rule_keeps_only_rights_its_object_can_have() {
    for (rights, directory, expected) in [
        (Rights::FULL, true, Rights::FULL.0),
        (Rights::FULL, false, Rights::FILE.0 & !Rights::IOCTL_DEV.0),
        (Rights::READ_DIR, false, 0),
    ] {
        assert_eq!(rule_rights(rights, directory), u64::from(expected), "{rights:?}, directory {directory}");
    }
}

#[test]
fn descriptor_gaps_keep_only_standards_named_and_report() {
    for (named, report, expected) in [
        (vec![0, 1, 2, 5, 9], 12, vec![(3, 4), (6, 8), (10, 11), (13, u32::MAX)]),
        (vec![9, 5, 5], 12, vec![(3, 4), (6, 8), (10, 11), (13, u32::MAX)]),
        (vec![], 3, vec![(4, u32::MAX)]),
    ] {
        assert_eq!(gaps(&named, report), expected, "named {named:?}, report {report}");
    }
}
