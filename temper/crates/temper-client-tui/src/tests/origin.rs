//! Origins at the TUI's text boundary (client/tui/README.md, 1).
use crate::{Origin, Unaddressed};
use core::net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV6};
use skein_lib::bytes::{concat, copy_of};

pub(super) fn named(port: u16) -> Origin {
    Origin::Https { name: copy_of(b"dev.fabrig.ai"), port }
}

pub(super) fn long_name() -> Box<[u8]> {
    concat(&[&[b'a'; 63], b".", &[b'b'; 63], b".", &[b'c'; 63], b".", &[b'd'; 61]])
}

#[test]
fn origins_parse_print_and_keep_their_authority() {
    let http = Origin::Http(SocketAddr::from((Ipv4Addr::LOCALHOST, 8367)));
    let ipv6 = Origin::Http(SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::LOCALHOST, 9400, 0, 0)));
    for (bytes, expected, printed, authority) in [
        (
            b"http://127.0.0.1:8367".as_slice(),
            http.clone(),
            b"http://127.0.0.1:8367".as_slice(),
            b"127.0.0.1:8367".as_slice(),
        ),
        (b"http://127.0.0.1:8367/", http, b"http://127.0.0.1:8367", b"127.0.0.1:8367"),
        (b"http://[::1]:9400/", ipv6, b"http://[::1]:9400", b"[::1]:9400"),
        (b"https://Dev.Fabrig.AI", named(443), b"https://dev.fabrig.ai", b"dev.fabrig.ai:443"),
        (b"https://dev.fabrig.ai:443/", named(443), b"https://dev.fabrig.ai", b"dev.fabrig.ai:443"),
        (b"https://dev.fabrig.ai:8443", named(8443), b"https://dev.fabrig.ai:8443", b"dev.fabrig.ai:8443"),
    ] {
        let origin = Origin::parse(bytes).expect("origin");
        assert_eq!(origin, expected, "{bytes:?}");
        assert_eq!(origin.print().as_ref(), printed);
        assert_eq!(origin.authority().as_ref(), authority);
        assert_eq!(Origin::parse(&origin.print()), Ok(origin));
    }
    let name = long_name();
    assert_eq!(name.len(), 253);
    let at_bound = concat(&[b"https://", &name, b":65535/"]);
    let origin = Origin::Https { name: name.clone(), port: 65535 };
    assert_eq!(Origin::parse(&at_bound), Ok(origin.clone()));
    assert_eq!(Origin::parse(&origin.print()), Ok(origin));
    // Grow the last label, not any 63-byte label, to test the name's bound.
    let past_bound = concat(&[b"https://", &name, b"d:65535"]);
    assert_eq!(Origin::parse(&past_bound), Err(Unaddressed::NotAnAddress));
}

#[test]
fn origins_refuse_by_cause_without_accepting_url_parts() {
    use Unaddressed::{HttpName, HttpsAddress, NotAnAddress, OffLoopback};
    for (bytes, cause) in [
        (b"http://localhost:8367".as_slice(), HttpName),
        (b"http://dev.fabrig.ai:80", HttpName),
        (b"https://127.0.0.1", HttpsAddress),
        (b"https://127.0.0.1:443", HttpsAddress),
        (b"https://[::1]", HttpsAddress),
        (b"https://[::1]:8443", HttpsAddress),
        (b"http://10.0.0.1:8367", OffLoopback),
        (b"http://[2001:db8::1]:80", OffLoopback),
        (b"https://dev.fabrig.ai/x", NotAnAddress),
        (b"https://dev.fabrig.ai?q", NotAnAddress),
        (b"https://user@dev.fabrig.ai", NotAnAddress),
        (b"https://dev.fabrig.ai#f", NotAnAddress),
        (b"https://dev..fabrig.ai", NotAnAddress),
        (b"https://dev_fabrig.ai", NotAnAddress),
        (b"https://-dev.fabrig.ai", NotAnAddress),
        (b"https://dev-.fabrig.ai", NotAnAddress),
        (b"https://dev.fabrig.ai.", NotAnAddress),
        (b"https://dev.fabrig.ai:0", NotAnAddress),
        (b"https://dev.fabrig.ai:65536", NotAnAddress),
        (b"https://dev.fabrig.ai:", NotAnAddress),
        (b"http://127.0.0.1:0", NotAnAddress),
        (b"http://127.0.0.1:65536", NotAnAddress),
        (b"ftp://dev.fabrig.ai", NotAnAddress),
        (b"http://127.0.0.1", NotAnAddress),
        (b"http://localhost", HttpName),
        (b"http://dev.fabrig.ai", HttpName),
        (b"https://box.123", NotAnAddress),
        (b"https://123", NotAnAddress),
        (b"https://dev.fabrig.ai//", NotAnAddress),
        (b"https://", NotAnAddress),
        (b"https://\xFF", NotAnAddress),
    ] {
        assert_eq!(Origin::parse(bytes), Err(cause), "{bytes:?}");
    }
    let long_label = concat(&[b"https://", &[b'a'; 64], b".ai"]);
    assert_eq!(Origin::parse(&long_label), Err(NotAnAddress));
}

#[test]
fn named_places_and_approval_tabs_use_the_origin() {
    let code = super::approval::code();
    for (port, place, tab) in [
        (443, b"dev.fabrig.ai".as_slice(), b"https://dev.fabrig.ai/terminal".as_slice()),
        (8443, b"dev.fabrig.ai:8443", b"https://dev.fabrig.ai:8443/terminal"),
    ] {
        let origin = named(port);
        for boxed in [false, true] {
            assert_eq!(crate::place(&origin, boxed).as_ref(), place);
        }
        assert_eq!(crate::approval_address(&origin, code).as_ref(), tab);
    }
}

#[test]
fn origin_phrases_fit_the_longest_name_and_page() {
    use crate::words::Word;
    use skein_ui::phrases::Hole;
    let origin = Origin::Https { name: long_name(), port: 65535 };
    let printed = origin.print();
    let tab = crate::approval_address(&origin, super::approval::code());
    let path = temper_client_domain::address::print(&temper_client_domain::Address::Team {
        project: u32::MAX,
        role: Some(u64::MAX),
    });
    let page = crate::origin::page(&origin, &path);
    let words = super::fixtures::words();
    assert_eq!(
        words.say(Word::NoProject, &[Hole::Name(printed.clone())]),
        concat(&[b"Setting temper up makes your first project: open ", &printed, b" in a browser."])
    );
    assert_eq!(words.say(Word::ApprovalNoTab, &[Hole::Name(tab.clone())]), concat(&[b"No tab? Open ", &tab]));
    assert_eq!(words.say(Word::OpenAt, &[Hole::Name(page.clone())]), concat(&[b"Open ", &page, b" in your browser."]));
    let tui = crate::Tui::new(&origin);
    let root = super::approval::asked(Some(temper_client_domain::asked::State::Asked {
        code: super::approval::code(),
        expires: skein_lib::Wall::from_nanos(600_000_000_000),
    }));
    let mut view = super::fixtures::view();
    super::fixtures::build(&mut view, &tui, &root);
    let bounds = crate::limits(&super::fixtures::limits(), &skein_ui::Limits::EMPTY, 512).unwrap();
    super::limits::fits(view.tree(), &bounds);
}
