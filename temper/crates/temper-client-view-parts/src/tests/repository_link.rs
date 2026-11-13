//! Repository addresses (temper/docs/design/client/forge/deployment.md).
#![expect(clippy::disallowed_methods, clippy::disallowed_macros, reason = "ordinary tests compare bounded addresses")]
use super::fixtures as f;
use crate::{forge::repository::address, forge_sign_in::Forges};
use temper_client_domain::{
    Address,
    forge::forges::{Connection, Kind, Row, State},
};
use temper_client_protocol_wire::v1 as w;
fn row() -> Row {
    let rows = f::connect_forge::rows(f::connect_forge::connected(), w::Offer::Offered);
    let root = f::forge_root(Address::Forges, rows);
    let State::Rows(rows) = root.forges() else { panic!("forge rows") };
    rows[1].clone()
}
#[test]
fn an_address_for_each_kind_of_forge() {
    let mut facts = Forges::new();
    facts.read("3 forgejo https://sign-in.example/root/ owned");
    let empty = Forges::new();
    let mut row = row();
    row.connection = Connection::NotConnected;
    for (kind, web, signed, expected) in [
        (
            Kind::GitHub,
            Some(b"https://github.com".as_slice()),
            false,
            Some(b"https://github.com/work/hello".as_slice()),
        ),
        (
            Kind::GitHub,
            Some(b"https://github.com/".as_slice()),
            true,
            Some(b"https://github.com/work/hello".as_slice()),
        ),
        (
            Kind::Forgejo,
            Some(b"https://forge.example/root".as_slice()),
            false,
            Some(b"https://forge.example/root/work/hello".as_slice()),
        ),
        (
            Kind::Forgejo,
            Some(b"https://forge.example/root/".as_slice()),
            false,
            Some(b"https://forge.example/root/work/hello".as_slice()),
        ),
        (
            Kind::Forgejo,
            None,
            true,
            Some(b"https://sign-in.example/root/user/oauth2/temper?redirect_to=%2Fwork%2Fhello".as_slice()),
        ),
        (
            Kind::Forgejo,
            Some(b"https://unused.example".as_slice()),
            true,
            Some(b"https://sign-in.example/root/user/oauth2/temper?redirect_to=%2Fwork%2Fhello".as_slice()),
        ),
        (Kind::Forgejo, None, false, None),
        (Kind::GitHub, None, true, None),
        (Kind::GitHub, Some(b"javascript:alert(1)".as_slice()), false, None),
    ] {
        row.kind = kind;
        row.web = web.map(Into::into);
        assert_eq!(address(&row, if signed { &facts } else { &empty }, b"work", b"hello").as_deref(), expected);
        let mut approval = f::needs::approval_row();
        approval.kind = kind;
        approval.forge = row.number;
        approval.owner = b"work".as_slice().into();
        approval.name = b"hello".as_slice().into();
        approval.address = match web {
            Some(web) => skein_lib::bytes::concat(&[
                web.strip_suffix(b"/").unwrap_or(web),
                if kind == Kind::GitHub { b"/work/hello/pull/24/files" } else { b"/work/hello/pulls/24/files" },
            ]),
            None => Box::default(),
        };
        let expected = if signed && kind == Kind::Forgejo {
            Some(
                b"https://sign-in.example/root/user/oauth2/temper?redirect_to=%2Fwork%2Fhello%2Fpulls%2F24%2Ffiles"
                    .as_slice()
                    .into(),
            )
        } else {
            expected.map(|url| {
                skein_lib::bytes::concat(&[
                    url,
                    if kind == Kind::GitHub { b"/pull/24/files" } else { b"/pulls/24/files" },
                ])
            })
        };
        assert_eq!(crate::common::approval::address(&approval, if signed { &facts } else { &empty }), expected);
    }
}
#[test]
fn a_name_cannot_leave_its_segment_or_the_query() {
    let mut facts = Forges::new();
    facts.read("3 forgejo https://forge.example owned");
    let mut row = row();
    row.web = Some(b"https://forge.example".as_slice().into());
    for (name, path, query) in [
        (b"a/b".as_slice(), "a%2Fb", "a%252Fb"),
        (b"a?b", "a%3Fb", "a%253Fb"),
        (b"a b", "a%20b", "a%2520b"),
        (b"%2F", "%252F", "%25252F"),
        (b"..", "%252E%252E", "%25252E%25252E"),
        (b".", "%252E", "%25252E"),
        (b"a..b", "a..b", "a..b"),
        (b"a#b", "a%23b", "a%2523b"),
        (b"\xc3\xa9", "%C3%A9", "%25C3%25A9"),
    ] {
        let expected = format!("https://forge.example/{path}/{path}");
        assert_eq!(address(&row, &Forges::new(), name, name).as_deref(), Some(expected.as_bytes()));
        let expected = format!("https://forge.example/user/oauth2/temper?redirect_to=%2F{query}%2F{query}");
        assert_eq!(address(&row, &facts, name, name).as_deref(), Some(expected.as_bytes()));
        let mut approval = f::needs::approval_row();
        approval.owner = name.into();
        approval.name = name.into();
        let expected = format!("{expected}%2Fpulls%2F24%2Ffiles");
        assert_eq!(crate::common::approval::address(&approval, &facts).as_deref(), Some(expected.as_bytes()));
    }
}

#[test]
fn repository_addresses_stop_at_the_url_bound() {
    let mut row = row();
    row.web = Some(b"https://forge.example".as_slice().into());
    let mut facts = Forges::new();
    facts.read("3 forgejo https://forge.example owned");
    let empty = Forges::new();
    let bound = usize::try_from(temper_client_domain::forge::change::ADDRESS_BYTES).unwrap();
    for (sign_in, prefix) in
        [(&empty, "https://forge.example/o/"), (&facts, "https://forge.example/user/oauth2/temper?redirect_to=%2Fo%2F")]
    {
        let mut name = vec![b'n'; bound.checked_sub(prefix.len()).unwrap()];
        let url = address(&row, sign_in, b"o", &name).expect("at the URL bound");
        assert_eq!(url.len(), bound);
        name.push(b'n');
        assert!(address(&row, sign_in, b"o", &name).is_none(), "one byte past the URL bound");
    }
}

#[test]
fn an_empty_owner_or_name_has_no_address() {
    let mut facts = Forges::new();
    facts.read("3 forgejo https://forge.example owned");
    let empty = Forges::new();
    let mut row = row();
    row.kind = Kind::Forgejo;
    row.web = Some(b"https://forge.example".as_slice().into());
    for (owner, name) in [(b"".as_slice(), b"hello".as_slice()), (b"work", b""), (b"", b"")] {
        for sign_in in [&empty, &facts] {
            assert!(address(&row, sign_in, owner, name).is_none(), "empty repository segment");
        }
    }
}
