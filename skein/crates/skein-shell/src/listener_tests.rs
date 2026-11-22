//! Decisions made before startup inspects inherited descriptors.

use super::{NO_ACCEPTCONN, listening, refused};
use crate::listeners::count;
use crate::{ListenerCheck, ListenersRefusal, MAX_INHERITED_LISTENERS};
use skein_io::kernel::Fd;

#[test]
fn listening_answers() {
    #[cfg(target_os = "linux")]
    const {
        assert!(!NO_ACCEPTCONN, "Linux keeps its listening check");
    };
    for (answer, no_option, expected) in [
        (Ok(0), false, Ok(false)),
        (Ok(0), true, Ok(false)),
        (Ok(4), false, Ok(true)),
        (Ok(4), true, Ok(true)),
        (Err(libc::EINVAL), false, Err(libc::EINVAL)),
        (Err(libc::EINVAL), true, Err(libc::EINVAL)),
        (Err(libc::ENOPROTOOPT), true, Ok(true)),
        (Err(libc::ENOPROTOOPT), false, Err(libc::ENOPROTOOPT)),
    ] {
        assert_eq!(listening(answer, no_option), expected, "answer {answer:?}, no option {no_option}");
    }
}

#[test]
fn refused_names_its_call() {
    let fd = Fd::new(3);
    for (check, name) in [
        (ListenerCheck::Stat, "fstat"),
        (ListenerCheck::Kind, "getsockopt SO_TYPE"),
        (ListenerCheck::Listening, "getsockopt SO_ACCEPTCONN"),
        (ListenerCheck::Address, "getsockname"),
    ] {
        assert_eq!(check.name(), name, "refused call's name");
        assert_eq!(
            refused(fd, check, libc::ENOTSOCK),
            ListenersRefusal::Check { fd, check, errno: libc::ENOTSOCK },
            "failed {name} names its check"
        );
        assert_eq!(refused(fd, check, libc::EBADF), ListenersRefusal::NotOpen { fd }, "{name}: descriptor closed");
        assert_eq!(refused(fd, check, libc::EAFNOSUPPORT), ListenersRefusal::NotIp { fd }, "{name}: not IP");
    }
}

#[test]
fn inherited_empty() {
    assert_eq!(count(b""), Ok(0), "no inherited listeners");
}

#[test]
fn inherited_zero() {
    assert_eq!(count(b"0"), Ok(0), "no inherited listeners");
}

#[test]
fn inherited_bad_count() {
    assert_eq!(count(b"+2"), Err(ListenersRefusal::BadCount));
}

#[test]
fn inherited_past_bound() {
    assert_eq!(count(b"1025"), Err(ListenersRefusal::TooMany { bound: MAX_INHERITED_LISTENERS }));
}
