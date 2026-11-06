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

#[test]
fn resumed_cache_work_goes_before_a_child_s_exit() {
    use crate::{
        domain::{
            render::{
                revision::Oid,
                vocabulary::{As, Blob, Request},
            },
            vocabulary::Request as RootRequest,
        },
        protocol::{self, Asked},
    };
    use skein_io::{self as io, kernel::Exit};
    use skein_lib::{Token, bytes};

    const CHILD: Token = Token::new(200);
    const STDOUT: Token = Token::new(201);
    const STDIN: Token = Token::new(202);
    const STDERR: Token = Token::new(203);
    const BODY: &[u8] = b"abc";

    let mut service = fixtures::service();
    crate::protocol_up(&mut service);
    drop(service.calls.pop());
    let oid = bytes::copy_of(b"0123456789abcdef0123456789abcdef01234567");
    protocol::down(
        &mut service.protocol,
        &service.protocol_env,
        RootRequest::Render(Request::Read {
            request: fixtures::OWNER,
            repository: bytes::copy_of(b"/repo"),
            commit: Oid(oid.clone()),
            blobs: Box::new([Blob {
                oid: Oid(oid),
                size: u64::try_from(BODY.len()).expect("fixture body"),
                kind: As::Cached,
            }]),
        }),
        &mut service.calls,
        &mut service.asked,
    );
    let Some(Asked::Io(io::Request::Spawn { owner, .. })) = service.asked.pop() else { panic!("cat-file spawn") };
    service.told.push(io::Event::Spawned { owner, child: CHILD, pipes: Box::new([STDOUT, STDIN, STDERR]) });
    service.told.push(io::Event::Exited { owner, exit: Exit::Code(1) });
    // Spawned makes cache creation runnable; the next event can cancel it.
    crate::protocol_up(&mut service);
    crate::protocol_up(&mut service);

    let mut created = false;
    while let Some(request) = service.asked.pop() {
        if let Asked::File(io::file::Request::Create { .. }) = request {
            created = true;
        }
    }
    assert!(created, "resumed cache creation reaches below before the next event can cancel it");
}
