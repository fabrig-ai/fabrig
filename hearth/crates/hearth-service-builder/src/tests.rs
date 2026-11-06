mod fixtures;
mod valid;
#[test]
fn a_second_output_is_refused_and_stays_with_its_caller() {
    let mut service = fixtures::service();
    assert_eq!(service.adopt_output_file(fixtures::OUTPUT), Ok(()), "one client output");
    assert_eq!(
        service.adopt_output_pipe(fixtures::OTHER_OUTPUT),
        Err(fixtures::OTHER_OUTPUT),
        "a second descriptor stays with its caller"
    );
}

#[test]
fn a_beneath_request_reaches_io_with_its_roots_descriptor() {
    use crate::protocol::{Asked, Beneath};
    use skein_lib::bytes;
    let (mut service, root) = fixtures::with_source_root();
    service.asked.push(Asked::Beneath(Beneath::OpenDirectory {
        owner: fixtures::OWNER,
        root,
        name: bytes::copy_of(b"child"),
        no_follow: true,
    }));
    crate::io_down(&mut service);
    let submit = service.file_submissions.pop().expect("open directory");
    assert_eq!(
        submit.kind,
        skein_io::kernel::Op::Open {
            root: fixtures::SOURCE_ROOT,
            path: bytes::copy_of(b"child"),
            how: skein_io::kernel::OpenHow::DirectoryNoFollow
        }
    );
}

#[test]
fn a_beneath_rename_reaches_io_with_both_descriptors() {
    use crate::protocol::{Asked, Beneath};
    use skein_lib::bytes;
    let (mut service, from_root) = fixtures::with_source_root();
    service.asked.push(Asked::Beneath(Beneath::Rename {
        owner: fixtures::OWNER,
        from_root,
        from: bytes::copy_of(b"index"),
        to_root: fixtures::OUTPUT,
        to: bytes::copy_of(b"digest"),
    }));
    crate::io_down(&mut service);
    assert_eq!(
        service.file_submissions.pop().expect("rename").kind,
        skein_io::kernel::Op::Rename {
            from_dir: fixtures::SOURCE_ROOT,
            from: bytes::copy_of(b"index"),
            to_dir: fixtures::OUTPUT,
            to: bytes::copy_of(b"digest"),
            no_replace: false,
        }
    );
}

#[test]
fn a_file_operation_reaches_the_kernel_under_its_own_token() {
    use crate::protocol::{Asked, Beneath};
    use skein_lib::{Time, Wall, bytes};
    let (mut service, root) = fixtures::with_source_root();
    service.asked.push(Asked::Beneath(Beneath::OpenDirectory {
        owner: fixtures::OWNER,
        root,
        name: bytes::copy_of(b"child"),
        no_follow: true,
    }));
    crate::io_down(&mut service);
    crate::iterate(&mut service, Time::ZERO, Wall::EPOCH);
    let submit = service.submissions().pop().expect("open directory reaches the kernel");
    assert!(skein_io::file_layer::owns(submit.op), "the operation keeps the file lane its owner minted");
}
