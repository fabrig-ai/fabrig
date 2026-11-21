//! Local synthetic GitHub connection frames; no forge is contacted.
use super::{World, sample_forge_answer};
use skein_lib::{List, Queue, Token};
use temper_client_domain::{Domain, Request};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub(super) fn forges(world: World) -> Box<[u8]> {
    let World::FirstRun { step } = world else { unreachable!("GitHub sample") };
    let mut parts = w::ForgeRow {
        starts_with: w::StartsWith::Repository,
        makings: List::with_capacity(0),
        host: b"git.example:3000".as_slice().into(),
        number: 2,
        kind: w::Kind::Github,
        mode: w::Mode::Connected,
        connection: w::Connection::Connected(w::Connected {
            id: 248_113_907,
            login: b"temper-7f3k2q[bot]".as_slice().into(),
        }),
        offer: w::Offer::NotOffered,
        owned: None,
        admin: false,
        read_only: false,
        ended_stop: None,
        installations: List::with_capacity(1),
        ended: None,
        first: None,
        app: Some(w::AppWay::Manifest),
        web: Some(b"http://github.test".as_slice().into()),
        repositories: List::with_capacity(1),
    };
    if step >= 21 {
        parts
            .installations
            .push(w::Installation {
                id: 61_022_417,
                account: Some(w::Account { id: 10, login: b"sam".as_slice().into() }),
                suspended: false,
                stray: false,
            })
            .expect("installation room");
    }
    if step == 22 {
        parts
            .repositories
            .push(w::Adoption { owner: b"sam".as_slice().into(), name: b"widget".as_slice().into(), project: 1 })
            .expect("adoption room");
    }
    let mut forges = List::with_capacity(1);
    forges.push(parts).expect("row room");
    wire::forges(&w::Forges { forges, full: List::with_capacity(0), linked: false })
}
pub(super) fn answer_page(root: &mut Domain, attempt: Token, out: &mut Queue<Request>) {
    let mut rows = List::with_capacity(4);
    for (id, name, private) in [
        (1, b"widget".as_slice(), false),
        (2, b"notes".as_slice(), true),
        (3, b"dotfiles".as_slice(), false),
        (4, b"site".as_slice(), false),
    ] {
        rows.push(w::Reached {
            id,
            owner: w::Account { id: 10, login: b"sam".as_slice().into() },
            name: name.into(),
            private,
        })
        .expect("repository room");
    }
    sample_forge_answer(root, attempt, w::Outcome::Page(w::Page { rows, more: false }), out);
}
