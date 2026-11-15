//! Switcher repositories (temper/docs/design/client/view.md, 3).
use super::{Held, held};
use crate::forge::{
    Playground, PlaygroundState,
    forges::{Adoption, Connection, Kind, Mode, Offer, Row, State},
};
use alloc::boxed::Box;

const PROJECT: u32 = 7;
const OTHER: u32 = 9;
// The fixture fills sixteen repository slots, retaining one as the displayed home.
const SIXTEEN: u32 = 16;

fn repository(project: u32, name: &[u8]) -> Adoption {
    Adoption { project, owner: b"work".as_slice().into(), name: name.into() }
}
fn row(number: u16, repositories: Box<[Adoption]>) -> Row {
    Row {
        starts_with: crate::forge::forges::StartsWith::Repository,
        number,
        host: b"git.example:3000".as_slice().into(),
        repositories,
        read_only: false,
        kind: Kind::Forgejo,
        mode: Mode::Owned,
        connection: Connection::NotConnected,
        offer: Offer::Offered,
        first: None,
        installations: Box::default(),
        ended: None,
        app: None,
        web: None,
        owned: None,
        fetched: None,
        admin: false,
        ended_stop: None,
        makings: Box::default(),
    }
}

fn displayed_forge(state: &State, number: u16) -> Option<&Row> {
    if let State::Rows(rows) = state {
        for row in rows {
            if row.number == number {
                return Some(row);
            }
        }
    }
    None
}

#[test]
fn absent_or_ended_watches_leave_repositories_unknown() {
    for state in [State::Unknown, State::Ended] {
        assert_eq!(held(&state, None, PROJECT), Held::Unknown);
    }
}

#[test]
fn only_the_projects_repositories_count() {
    let cases: [Box<[Adoption]>; 2] = [Box::default(), Box::new([repository(OTHER, b"elsewhere")])];
    for repositories in cases {
        let state = State::Rows(Box::new([row(3, repositories)]));
        assert_eq!(held(&state, None, PROJECT), Held::None);
    }
    let state = State::Rows(Box::new([row(3, Box::new([repository(PROJECT, b"hello")]))]));
    assert_eq!(
        held(&state, None, PROJECT),
        Held::One {
            forge: match &state {
                State::Rows(rows) => rows.first(),
                State::Unknown | State::Ended => None,
            },
            owner: b"work",
            name: b"hello"
        }
    );
}

#[test]
fn the_lowest_forge_and_its_first_repository_stand_in_for_home() {
    let state = State::Rows(Box::new([
        row(5, Box::new([repository(PROJECT, b"higher")])),
        row(
            3,
            Box::new([repository(OTHER, b"elsewhere"), repository(PROJECT, b"hello"), repository(PROJECT, b"next")]),
        ),
    ]));
    assert_eq!(
        held(&state, None, PROJECT),
        Held::Several { forge: displayed_forge(&state, 3), owner: b"work", name: b"hello", more: 2 }
    );
}

#[test]
fn sixteen_repositories_keep_the_home_and_fifteen_more() {
    let mut repositories = skein_lib::List::with_capacity(SIXTEEN);
    for _ in 0..SIXTEEN {
        repositories.push(repository(PROJECT, b"hello")).unwrap();
    }
    let repositories = repositories.into_boxed();
    let state = State::Rows(Box::new([row(3, repositories)]));
    assert_eq!(
        held(&state, None, PROJECT),
        Held::Several { forge: displayed_forge(&state, 3), owner: b"work", name: b"hello", more: 15 }
    );
}

#[test]
fn the_playground_names_widget_without_a_delivered_watch() {
    let playground = Playground {
        state: PlaygroundState::Running,
        forge: 3,
        project: Some(PROJECT),
        provider: 1,
        owner: b"playground".as_slice().into(),
        name: b"widget".as_slice().into(),
        cues: Box::default(),
    };
    for state in [State::Unknown, State::Ended, State::Rows(Box::default())] {
        assert_eq!(
            held(&state, Some(&playground), PROJECT),
            Held::One { forge: None, owner: b"playground", name: b"widget" }
        );
    }
    assert_eq!(held(&State::Unknown, Some(&playground), OTHER), Held::Unknown);
}
