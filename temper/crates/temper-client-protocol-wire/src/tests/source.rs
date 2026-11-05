//! Shared address grammar (temper/docs/design/draft/repository-copies.md, 5.4).
use crate::source::{Source, SourceRefusal, address, org, parse, repository};

#[test]
fn source_forms_parse_and_render_the_same_address() {
    for (host, canonical, forms) in [
        (
            b"source.invalid".as_slice(),
            b"https://source.invalid/Owner/Name".as_slice(),
            [
                "source.invalid/Owner/Name",
                "https://source.invalid/Owner/Name",
                "source.invalid/Owner/Name.git",
                "https://source.invalid/Owner/Name.git",
            ],
        ),
        (
            b"source.invalid:8443".as_slice(),
            b"https://source.invalid:8443/Owner/Name".as_slice(),
            [
                "source.invalid:8443/Owner/Name",
                "https://source.invalid:8443/Owner/Name",
                "source.invalid:8443/Owner/Name.git",
                "https://source.invalid:8443/Owner/Name.git",
            ],
        ),
    ] {
        let expected = Source { host, owner: b"Owner", name: b"Name" };
        for form in forms {
            assert_eq!(parse(form.as_bytes()), Ok(expected), "{form}");
            let rendered = address(&expected);
            assert_eq!(rendered.as_ref(), canonical);
            assert_eq!(parse(&rendered), Ok(expected));
        }
    }
}

#[test]
fn invalid_source_forms_report_the_refused_part() {
    for (input, why) in [
        ("http://source.invalid/work/repo", SourceRefusal::Scheme),
        ("ssh://source.invalid/work/repo", SourceRefusal::Scheme),
        ("git@source.invalid:work/repo", SourceRefusal::Scheme),
        ("source.invalid/work", SourceRefusal::Segments),
        ("source.invalid/work/repo/extra", SourceRefusal::Segments),
        ("source.invalid//repo", SourceRefusal::Segments),
        ("source.invalid/work/", SourceRefusal::Segments),
        ("/work/repo", SourceRefusal::Segments),
        ("source.invalid/./repo", SourceRefusal::Name),
        ("source.invalid/../repo", SourceRefusal::Name),
        ("source.invalid/work/.", SourceRefusal::Name),
        ("source.invalid/work/..", SourceRefusal::Name),
        ("source.invalid/work/.git", SourceRefusal::Name),
        ("source.invalid/work/repo?x", SourceRefusal::Name),
        ("source.invalid/work/repo#x", SourceRefusal::Name),
        ("source.invalid/work/repo%20", SourceRefusal::Name),
        ("bad_host.invalid/work/repo", SourceRefusal::Host),
        ("-bad.invalid/work/repo", SourceRefusal::Host),
        ("bad-.invalid/work/repo", SourceRefusal::Host),
        ("bad..invalid/work/repo", SourceRefusal::Host),
        ("127.0.0.1/work/repo", SourceRefusal::Host),
        ("[::1]/work/repo", SourceRefusal::Host),
        ("user@source.invalid/work/repo", SourceRefusal::Host),
        ("source.invalid:/work/repo", SourceRefusal::Host),
        ("source.invalid:0/work/repo", SourceRefusal::Host),
        ("source.invalid:65536/work/repo", SourceRefusal::Host),
        ("source.invalid:abc/work/repo", SourceRefusal::Host),
    ] {
        assert_eq!(parse(input.as_bytes()), Err(why), "{input}");
    }
}

#[test]
fn source_name_and_dns_bounds_hold_on_both_sides() {
    // The draft's name bound is 100 bytes; DNS labels hold at 63, names at 253.
    for (length, accepted) in [(SOURCE_NAME_BYTES, true), (SOURCE_NAME_BYTES + 1, false)] {
        let name = super::filled(length);
        for input in [
            skein_lib::bytes::concat(&[b"source.invalid/work/", &name]),
            skein_lib::bytes::concat(&[b"source.invalid/", &name, b"/repo"]),
            skein_lib::bytes::concat(&[b"source.invalid/work/", &name, b".git"]),
        ] {
            assert_eq!(parse(&input).is_ok(), accepted, "{input:?}");
        }
    }
    for (host, accepted) in [
        (skein_lib::bytes::concat(&[&super::filled(DNS_LABEL_BYTES), b".invalid"]), true),
        (skein_lib::bytes::concat(&[&super::filled(DNS_LABEL_BYTES + 1), b".invalid"]), false),
        (dns_name(DNS_LAST_LABEL_BYTES), true),
        (dns_name(DNS_LAST_LABEL_BYTES + 1), false),
    ] {
        let input = skein_lib::bytes::concat(&[&host, b"/work/repo"]);
        assert_eq!(parse(&input).is_ok(), accepted, "{input:?}");
    }
}

fn dns_name(last: usize) -> Box<[u8]> {
    let label = super::filled(DNS_LABEL_BYTES);
    skein_lib::bytes::concat(&[&label, b".", &label, b".", &label, b".", &super::filled(last)])
}

#[test]
fn destination_names_follow_their_distinct_grammars() {
    for (name, is_org, is_repository) in [
        (b"".as_slice(), false, false),
        (b".", false, false),
        (b"..", false, false),
        (b"a._-09Z", true, true),
        (b"_repo", false, true),
        (b"a/b", false, false),
        (b"a b", false, false),
        (b"\xff", false, false),
        (&[b'a'; ORG_BYTES], true, true),
        (&[b'a'; ORG_BYTES + 1], false, true),
        (&[b'a'; SOURCE_NAME_BYTES], false, true),
        (&[b'a'; SOURCE_NAME_BYTES + 1], false, false),
    ] {
        assert_eq!(org(name), is_org, "{name:?}");
        assert_eq!(repository(name), is_repository, "{name:?}");
    }
}

// The draft's 5.4 name bound and skein's given::login bound.
const SOURCE_NAME_BYTES: usize = 100;
const ORG_BYTES: usize = 40;
// DNS labels are at most 63 bytes. Three full labels, three dots and a final
// 61-byte label fill the DNS name's 253 bytes.
const DNS_LABEL_BYTES: usize = 63;
const DNS_LAST_LABEL_BYTES: usize = 61;
