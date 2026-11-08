use super::{Request, answer};

#[test]
fn help_names_its_binary_in_its_refusals() {
    for (request, expected) in [
        (Request::Page(Box::from(b"nope".as_slice())), "temper-client: no page nope; temper-client help lists them\n"),
        (Request::Search(Box::from(b"zzzznotadocword".as_slice())), "temper-client: nothing matches\n"),
        (Request::Search(Box::from(b" \t".as_slice())), "temper-client: nothing to search for\n"),
        (Request::Search(Box::from(b"a a a a a a a a a".as_slice())), "temper-client: at most 8 words\n"),
        (
            Request::Search(Box::from(b"123456789012345678901234567890123".as_slice())),
            "temper-client: a word is at most 32 bytes\n",
        ),
    ] {
        assert_eq!(answer("temper-client", &request), Err(expected.into()));
    }
}
