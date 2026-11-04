use super::{Making, Refused, ask, copy_target, phrase};
use crate::{forge, tests::fixtures};
use skein_lib::List;
use temper_client_protocol_wire::{self as wire, source, v1 as w};

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
                        number,
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
        bytes: wire::forges(&w::Forges::new(&limits, w::ForgesParts { forges }).unwrap()),
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
