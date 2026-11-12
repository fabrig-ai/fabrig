use super::fixtures;
use crate::output::{self, Markers};
use temper_worker_domain_checkout::git::{Fault, Kind, Missing};

#[test]
fn a_marker_cut_across_reads_is_found() {
    let rows: [(&[&[u8]], bool); 4] = [
        (&[b"one\n<<<<", b"<<< ours\ntwo\n"], true),
        (&[b"=====", b"==\r\n"], true),
        (&[b">>>>>>>", b" theirs"], true),
        (&[b"<<<<<<", b"<\n"], false),
    ];
    for (reads, expected) in rows {
        let mut markers = Markers::START;
        for read in reads {
            markers.feed(read);
        }
        assert_eq!(markers.finish(), expected, "{reads:?}");
    }
}
#[test]
fn markers_of_six_and_eight_characters_are_no_markers() {
    for bytes in [
        b"<<<<<< ours\n".as_slice(),
        b"<<<<<<<< ours\n",
        b"======\n",
        b"========\n",
        b">>>>>> theirs\n",
        b">>>>>>>> theirs\n",
        b"x<<<<<<< ours\n",
        b"=======suffix\n",
        b"<<<<<<<\n",
    ] {
        let mut markers = Markers::START;
        markers.feed(bytes);
        assert!(!markers.finish(), "{bytes:?}");
    }
}
#[test]
fn one_carriage_return_before_the_newline_is_dropped() {
    for (bytes, expected) in [
        (b"=======\r\n".as_slice(), true),
        (b"=======\r", true),
        (b"=======\r\r\n", false),
        (b"<<<<<<< ours\r\n", true),
        (b">>>>>>> theirs\r", true),
    ] {
        let mut markers = Markers::START;
        markers.feed(bytes);
        assert_eq!(markers.finish(), expected, "{bytes:?}");
    }
}
#[test]
fn nul_lists_are_complete_for_commits_and_bounded_prefixes_for_merges() {
    let mut checkout = fixtures::checkout();
    checkout.conflicts = 2;
    checkout.path_bytes = 4;
    for (bytes, accepted) in [
        (b"".as_slice(), true),
        (b"one\0two\0", true),
        (b"four\0", true),
        (b"one\0two\0x\0", false),
        (b"longer\0", false),
        (b"one", false),
        (b"\0", false),
    ] {
        let parsed = output::paths(bytes, &checkout);
        assert_eq!(parsed.is_some(), accepted, "{bytes:?}");
    }
    assert_eq!(
        output::paths(b"one\0two\0", &checkout),
        Some(Box::from([Box::from(&b"one"[..]), Box::from(&b"two"[..])]))
    );
    let two: Box<[Box<[u8]>]> = Box::from([Box::from(b"one".as_slice()), Box::from(b"two".as_slice())]);
    for (bytes, expected) in [
        (b"one\0two\0x\0".as_slice(), Some(two.clone())),
        (b"one\0two\0longer\0", Some(two)),
        (b"one\0cut", Some(Box::from([Box::from(b"one".as_slice())]))),
        (b"longer\0", None),
        (b"one\0longer\0", None),
        (b"\0", None),
    ] {
        assert_eq!(output::conflict_paths(bytes, &checkout), expected, "{bytes:?}");
    }
}
#[test]
fn stderr_lines_decide_last_to_first_the_first_match_wins() {
    let rows: &[(&[u8], Kind, Fault)] = &[
        (b"fatal: Authentication failed for 'x'\nfatal: early EOF\n", Kind::Store, Fault::Unreachable),
        (b"fatal: early EOF\nfatal: Authentication failed for 'x'\nerror: unrelated\n", Kind::Fetch, Fault::Refused),
        (
            b"fatal: unable to access 'x': The requested URL returned error: 404\n",
            Kind::Push,
            Fault::Missing { missing: Missing::Repository },
        ),
        (b"fatal: unable to access 'x': The requested URL returned error: 401\n", Kind::Create, Fault::Refused),
        (b"fatal: unable to access 'x': The requested URL returned error: 403\n", Kind::Fetch, Fault::Refused),
        (b"fatal: unable to access 'x': The requested URL returned error: 500\n", Kind::Store, Fault::Unreachable),
        (b"fatal: couldn't find remote ref x\n", Kind::Store, Fault::Broken),
        (b"fatal: couldn't find remote ref x\n", Kind::Fetch, Fault::Missing { missing: Missing::Branch }),
        (
            b"fatal: remote error: upload-pack: not our ref x\n",
            Kind::Fetch,
            Fault::Missing { missing: Missing::Commit },
        ),
        (b"fatal: repository 'x' not found\n", Kind::Store, Fault::Missing { missing: Missing::Repository }),
        (b"fatal: could not read Username for 'x'\n", Kind::Fetch, Fault::Refused),
        (b"fatal: the remote end hung up unexpectedly\n", Kind::Push, Fault::Unreachable),
        (b"error: RPC failed; x\n", Kind::Push, Fault::Unreachable),
        (b"remote: fatal: early EOF\n", Kind::Push, Fault::Broken),
        (b"fatal: early EOF\n", Kind::Merge, Fault::Unreachable),
        (b"fatal: early EOF\n", Kind::Commit, Fault::Broken),
        (b"fatal: early EOF\n", Kind::CheckOut, Fault::Unreachable),
        (b"fatal: early EOF\n", Kind::Make, Fault::Broken),
    ];
    for (tail, kind, expected) in rows {
        assert_eq!(output::classify(tail, *kind), *expected, "{tail:?}, {kind:?}");
    }
}

#[test]
fn lazy_fetch_failures_are_remote_and_local_failures_stay_broken() {
    use crate::{
        invocation::Command,
        sequence::{self, Next},
    };
    use temper_worker_domain_checkout::git;
    for (kind, command, op) in [
        (
            Kind::CheckOut,
            Command::CheckOut { commit: fixtures::COMMIT },
            git::Op::CheckOut { at: fixtures::at(), commit: fixtures::COMMIT, clean: true, identity: 1, account: None },
        ),
        (
            Kind::Merge,
            Command::Merge { theirs: fixtures::COMMIT },
            git::Op::Merge { at: fixtures::at(), theirs: fixtures::COMMIT, identity: 1, account: None },
        ),
    ] {
        for (tail, expected) in [
            (b"fatal: unable to access 'x': The requested URL returned error: 401\n".as_slice(), Fault::Refused),
            (b"fatal: unable to access 'x': Could not resolve host: x\n", Fault::Unreachable),
            (b"fatal: repository 'x' not found\n", Fault::Missing { missing: Missing::Repository }),
            (b"fatal: Unable to create 'x/index.lock': File exists.\n", Fault::Broken),
        ] {
            assert_eq!(output::classify(tail, kind), expected, "{kind:?}");
            let Next::End(git::Done::FailedWithOutput { fault, .. }) =
                sequence::after(&op, &command, 128, b"", tail, 0, &fixtures::checkout())
            else {
                panic!("failed command")
            };
            assert_eq!(fault, expected, "{kind:?}");
            let local = if kind == Kind::CheckOut { Command::Clean } else { Command::Unmerged };
            let Next::End(git::Done::FailedWithOutput { fault, .. }) =
                sequence::after(&op, &local, 128, b"", tail, 0, &fixtures::checkout())
            else {
                panic!("failed local command")
            };
            assert_eq!(fault, Fault::Broken, "secondary command is local");
        }
    }
}
