use super::{Making, Refused, ask, copy_target, phrase};
use crate::{forge, tests::fixtures};
use skein_lib::List;
use temper_client_protocol_wire::{self as wire, source, v1 as w};

#[test]
fn address_lines_keep_independent_validation_and_repairs() {
    use super::{Line, lines};
    let parsed = lines(
        b" github.com/octo/vsr\n octo/vsr\ngithub/octo/vsr\na\nhttps://host.example/o/n.git\n\nwork/taken\n",
        &|_address, org, name| org == b"work" && name == b"taken",
    );
    assert_eq!(
        parsed.as_slice(),
        [
            Line::To {
                address: b"https://github.com/octo/vsr".as_slice().into(),
                org: b"octo".as_slice().into(),
                name: b"vsr".as_slice().into(),
            },
            Line::To {
                address: b"https://github.com/octo/vsr".as_slice().into(),
                org: b"octo".as_slice().into(),
                name: b"vsr".as_slice().into(),
            },
            Line::NotAHost {
                raw: b"github/octo/vsr".as_slice().into(),
                line: b"github.com/octo/vsr".as_slice().into(),
            },
            Line::Bad { raw: b"a".as_slice().into() },
            Line::To {
                address: b"https://host.example/o/n".as_slice().into(),
                org: b"o".as_slice().into(),
                name: b"n".as_slice().into(),
            },
            Line::Taken {
                raw: b"work/taken".as_slice().into(),
                org: b"work".as_slice().into(),
                name: b"taken".as_slice().into(),
            },
        ]
    );
}

fn copy(address: &[u8]) -> w::MakeFrom {
    w::MakeFrom::Copy(w::MakeCopy::new(&w::CEILINGS, w::MakeCopyParts { address: address.into() }).unwrap())
}

fn into() -> w::MakeInto {
    w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project: 7, home: true }).unwrap()
}

fn made() -> w::MakingMade {
    w::MakingMade::new(&w::CEILINGS, w::MakingMadeParts { id: 42, branch: b"trunk".as_slice().into() }).unwrap()
}

fn failed(step: w::MakeStep) -> w::MakingState {
    w::MakingState::Failed(
        w::MakingFailed::new(
            &w::CEILINGS,
            w::MakingFailedParts { step, reason: w::MakeFailure::Taken, words: b"name taken".as_slice().into() },
        )
        .unwrap(),
    )
}

#[test]
fn a_rows_makings_are_delivered_in_order() {
    let limits = forge::wire_limits(&fixtures::limits().forge).unwrap();
    let expected = [
        Making {
            org: b"a".as_slice().into(),
            name: b"one".as_slice().into(),
            from: w::MakeFrom::Empty,
            into: None,
            state: w::MakingState::Waiting,
        },
        Making {
            org: b"a".as_slice().into(),
            name: b"two".as_slice().into(),
            from: copy(b"https://source.invalid/team/two"),
            into: Some(into()),
            state: w::MakingState::Making,
        },
        Making {
            org: b"b".as_slice().into(),
            name: b"one".as_slice().into(),
            from: w::MakeFrom::Empty,
            into: Some(into()),
            state: w::MakingState::Made(made()),
        },
        Making {
            org: b"b".as_slice().into(),
            name: b"two".as_slice().into(),
            from: copy(b"https://source.invalid/team/two"),
            into: Some(into()),
            state: w::MakingState::Adopting(made()),
        },
        Making {
            org: b"c".as_slice().into(),
            name: b"one".as_slice().into(),
            from: copy(b"https://source.invalid/team/one"),
            into: None,
            state: failed(w::MakeStep::Repository),
        },
    ];
    let mut makings = List::with_capacity(5);
    for row in &expected {
        makings
            .push(
                w::RepositoryMaking::new(
                    &limits,
                    w::RepositoryMakingParts {
                        org: row.org.clone(),
                        name: row.name.clone(),
                        from: row.from.clone(),
                        into: row.into.clone(),
                        state: row.state.clone(),
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let mut forges = List::with_capacity(2);
    for (number, makings) in [(3, makings), (4, List::with_capacity(0))] {
        forges
            .push(
                w::ForgeRow::new(
                    &limits,
                    w::ForgeRowParts {
                        starts_with: w::StartsWith::Repository,
                        number,
                        host: b"git.example:3000".as_slice().into(),
                        kind: w::Kind::Forgejo,
                        mode: w::Mode::Managed,
                        connection: w::Connection::NotConnected,
                        offer: w::Offer::NotOffered,
                        first: None,
                        installations: List::with_capacity(0),
                        ended: None,
                        app: None,
                        web: None,
                        repositories: List::with_capacity(0),
                        owned: None,
                        admin: true,
                        read_only: false,
                        ended_stop: None,
                        makings,
                    },
                )
                .unwrap(),
            )
            .unwrap();
    }
    let record = jig_client::Family {
        family: wire::FORGE,
        version: wire::CLIENT_VERSION,
        bytes: wire::forges(&w::Forges::new(&limits, w::ForgesParts { forges, linked: false }).unwrap()),
    };
    let rows = forge::forges::rows(&record, &fixtures::limits().forge).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].makings.as_ref(), expected);
    assert!(rows[1].makings.is_empty());
}

#[test]
fn a_copys_target_is_its_sources_owner_and_name() {
    for address in [b"source.invalid:8443/team/widget".as_slice(), b"https://source.invalid:8443/team/widget.git"] {
        let source = source::parse(address).unwrap();
        let (org, name) = copy_target(&source);
        assert_eq!(org.as_ref(), b"team");
        assert_eq!(name.as_ref(), b"widget");
        let request = ask(3, &org, &name, Some(address), Some(into())).unwrap().into_parts();
        assert_eq!(request.forge, 3);
        assert_eq!(request.org, org);
        assert_eq!(request.name, name);
        assert_eq!(request.into, Some(into()));
        assert_eq!(request.from, copy(b"https://source.invalid:8443/team/widget"));
    }
}

#[test]
fn an_owner_that_is_no_org_needs_as() {
    for address in
        [b"source.invalid/_team/widget".as_slice(), b"source.invalid/abcdefghijklmnopqrstuvwxyzabcdefghijklmno/widget"]
    {
        let source = source::parse(address).unwrap();
        let (org, name) = copy_target(&source);
        assert_eq!(ask(3, &org, &name, Some(address), None), Err(Refused::Org));
        let request = ask(3, b"team", b"renamed", Some(address), None).unwrap();
        assert_eq!(request.org(), b"team");
        assert_eq!(request.name(), b"renamed");
        assert_eq!(request.from(), &copy(&source::address(&source)));
    }
}

#[test]
fn a_request_checks_names_and_the_source_before_it_is_formed() {
    for name in [b"".as_slice(), b".", b"..", b"a/b", b"a b", &[b'a'; 101]] {
        assert_eq!(ask(3, b"team", name, None, None), Err(Refused::Name));
    }
    for address in [
        b"http://source.invalid/team/widget".as_slice(),
        b"ssh://source.invalid/team/widget",
        b"git@source.invalid:team/widget",
    ] {
        assert_eq!(
            ask(3, b"team", b"widget", Some(address), None),
            Err(Refused::Source(source::SourceRefusal::Scheme))
        );
    }
    let request = ask(3, b"team", b"widget", None, None).unwrap();
    assert_eq!(request.from(), &w::MakeFrom::Empty);
    assert_eq!(request.into_parts().into, None);
}

#[test]
fn the_row_says_what_it_is() {
    let mut row = Making {
        org: b"team".as_slice().into(),
        name: b"widget".as_slice().into(),
        from: w::MakeFrom::Empty,
        into: None,
        state: w::MakingState::Waiting,
    };
    for state in [
        w::MakingState::Waiting,
        w::MakingState::Making,
        w::MakingState::Made(made()),
        w::MakingState::Adopting(made()),
    ] {
        row.state = state;
        row.from = w::MakeFrom::Empty;
        assert_eq!(phrase(&row).as_ref(), b"Making team/widget");
        row.from = copy(b"https://source.invalid:8443/owner/original");
        assert_eq!(phrase(&row).as_ref(), b"Copying team/widget from source.invalid:8443");
    }
    for (step, words) in [
        (w::MakeStep::Admin, b"team/widget failed at admin: name taken".as_slice()),
        (w::MakeStep::Org, b"team/widget failed at org: name taken"),
        (w::MakeStep::Repository, b"team/widget failed at repository: name taken"),
        (w::MakeStep::File, b"team/widget failed at file: name taken"),
        (w::MakeStep::Adoption, b"team/widget failed at adoption: name taken"),
    ] {
        row.state = failed(step);
        assert_eq!(phrase(&row).as_ref(), words);
    }
}

#[test]
fn welcome_copies_read_the_mocks_forms_in_order() {
    use super::{CopiesRefused as R, parse_copies};
    for input in [
        "source.invalid/team/widget",
        "https://source.invalid/team/widget",
        "http://source.invalid/team/widget",
        "source.invalid/team/widget.git",
        "source.invalid/team/widget///",
        "https://source.invalid/team/widget.git/",
    ] {
        assert_eq!(
            parse_copies(input.as_bytes()).unwrap().as_slice(),
            [Box::from(b"https://source.invalid/team/widget".as_slice())],
            "{input}"
        );
    }
    assert_eq!(
        parse_copies(b" \n acme/one \n\nhttp://source.invalid/team/two.git\r\n").unwrap().as_slice(),
        [
            Box::from(b"https://github.com/acme/one".as_slice()),
            Box::from(b"https://source.invalid/team/two".as_slice()),
        ]
    );
    assert_eq!(
        parse_copies(b"source.invalid:8443/o/n").unwrap().as_slice(),
        [Box::from(b"https://source.invalid:8443/o/n".as_slice())]
    );
    for input in ["a/b/c/d", "host.invalid//n", "git@host:o/n", " a ", "source.invalid:0/o/n", "host.invalid/o/n?x"] {
        assert_eq!(parse_copies(input.as_bytes()), Err(R::NotAnAddress(input.trim().as_bytes().into())), "{input}");
    }
    for input in ["", " \n\r\n", "https://\n.git\n///"] {
        assert_eq!(parse_copies(input.as_bytes()), Err(R::NoAddress));
    }
    let lines = skein_lib::bytes::concat(&[b"a/b\n".as_slice(); 17]);
    assert_eq!(parse_copies(lines.get(..64).unwrap()).unwrap().len(), 16);
    assert_eq!(parse_copies(&lines), Err(R::TooMany));
}
