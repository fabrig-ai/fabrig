use super::{directory_access_with, private_metadata, read_bounded};
use crate::startup::{DirectoryAccess, Unread};
use skein_io::{
    file::Unsafe,
    kernel::{Kind, Stat},
};
use std::path::Path;

#[test]
fn directory_access_refuses_a_nul_before_a_call() {
    let access = directory_access_with(Path::new("nul\0inside"), |_| panic!("a NUL path must not reach the kernel"));
    assert_eq!(access, DirectoryAccess::Denied(libc::EINVAL));
}

const SECRET: &[u8; 6] = b"secret";

fn read_secret(size: i64, max: u32) -> Result<Box<[u8]>, Unread> {
    let mut chunks = SECRET.chunks(3);
    read_bounded(Kind::File, size, max, |buffer| {
        let bytes = chunks.next().unwrap_or_default();
        buffer[..bytes.len()].copy_from_slice(bytes);
        Ok(bytes.len())
    })
}

#[test]
fn startup_read_checks_its_bound_against_metadata_and_read_lengths() {
    assert_eq!(read_secret(6, 6).expect("six bytes at the bound").as_ref(), SECRET);
    assert_eq!(
        read_bounded(Kind::File, 6, 5, |_| panic!("known oversize files are not read")),
        Err(Unread::TooLarge { max: 5 })
    );
    assert_eq!(
        read_secret(5, 5),
        Err(Unread::TooLarge { max: 5 }),
        "bytes beyond the observed size still count toward the bound"
    );
}

#[test]
fn startup_private_file_refuses_wrong_owner_before_other_unsafe_metadata() {
    let expected = 41;
    let found = 42;
    let safe = Stat { kind: Kind::File, size: 6, mode: 0o600, owner: expected, links: 1 };
    let unsafe_entry = Stat { kind: Kind::Symlink, size: 6, mode: 0o640, owner: found, links: 2 };
    assert_eq!(private_metadata(safe, expected), Ok(()));
    for (stat, refusal) in [
        (Stat { owner: found, ..safe }, Unsafe::Owner { found, expected }),
        (unsafe_entry, Unsafe::Owner { found, expected }),
        (Stat { owner: expected, ..unsafe_entry }, Unsafe::Link),
        (Stat { owner: expected, kind: Kind::Directory, ..unsafe_entry }, Unsafe::Kind(Kind::Directory)),
        (Stat { owner: expected, kind: Kind::File, ..unsafe_entry }, Unsafe::Mode(0o040)),
        (Stat { links: 2, ..safe }, Unsafe::Links(2)),
    ] {
        assert_eq!(private_metadata(stat, expected), Err(Unread::Unsafe(refusal)), "metadata {stat:?}");
    }
}
