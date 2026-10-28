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
fn a_nul_list_past_the_checkout_s_limits_is_refused() {
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
}
#[test]
fn stderr_lines_decide_last_to_first_the_first_match_wins() {
    let rows: &[(&[u8], Kind, Fault)] = &[
        (b"fatal: Authentication failed for 'x'\nfatal: early EOF\n", Kind::Clone, Fault::Unreachable),
        (b"fatal: early EOF\nfatal: Authentication failed for 'x'\nerror: unrelated\n", Kind::Fetch, Fault::Refused),
        (
            b"fatal: unable to access 'x': The requested URL returned error: 404\n",
            Kind::Push,
            Fault::Missing { missing: Missing::Repository },
        ),
        (b"fatal: unable to access 'x': The requested URL returned error: 401\n", Kind::Create, Fault::Refused),
        (b"fatal: unable to access 'x': The requested URL returned error: 403\n", Kind::Fetch, Fault::Refused),
        (b"fatal: unable to access 'x': The requested URL returned error: 500\n", Kind::Clone, Fault::Unreachable),
        (b"fatal: couldn't find remote ref x\n", Kind::Clone, Fault::Broken),
        (b"fatal: couldn't find remote ref x\n", Kind::Fetch, Fault::Missing { missing: Missing::Branch }),
        (
            b"fatal: remote error: upload-pack: not our ref x\n",
            Kind::Fetch,
            Fault::Missing { missing: Missing::Commit },
        ),
        (b"fatal: repository 'x' not found\n", Kind::Clone, Fault::Missing { missing: Missing::Repository }),
        (b"fatal: could not read Username for 'x'\n", Kind::Fetch, Fault::Refused),
        (b"fatal: the remote end hung up unexpectedly\n", Kind::Push, Fault::Unreachable),
        (b"error: RPC failed; x\n", Kind::Push, Fault::Unreachable),
        (b"remote: fatal: early EOF\n", Kind::Push, Fault::Broken),
        (b"fatal: early EOF\n", Kind::Merge, Fault::Broken),
        (b"fatal: early EOF\n", Kind::Commit, Fault::Broken),
        (b"fatal: early EOF\n", Kind::CheckOut, Fault::Broken),
        (b"fatal: early EOF\n", Kind::Make, Fault::Broken),
    ];
    for (tail, kind, expected) in rows {
        assert_eq!(output::classify(tail, *kind), *expected, "{tail:?}, {kind:?}");
    }
}
