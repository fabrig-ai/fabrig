//! Accounts' people names use Members' ordered project reads (client/pages/accounts.md, 2).
#![expect(clippy::disallowed_methods, reason = "ordinary tests inspect emitted reads and fixture rows")]
use super::fixtures as f;
use crate::{
    Address,
    pages::{self, Request, accounts},
};
use jig_client::{Client, Event, Handle, Readable, Readout, Told, Watchable, contract as c};
use skein_lib::{List, Queue, Time, Token, Wall};

fn text(bytes: &[u8]) -> c::DocumentText {
    c::DocumentText::complete(f::acts::text(bytes))
}
fn client_event(client: &mut Client, event: Event) -> Queue<jig_client::UnlinkedRequest> {
    let mut out = Queue::uncapped();
    client.step_unlinked(Time::ZERO, Wall::EPOCH, event, &mut out);
    out
}
fn deliver(client: &mut Client, watch: u64, delivery: c::Delivery) {
    client_event(
        client,
        Event::Up {
            up: jig_client::Up::Watched { watch: Token::new(watch), event: c::WatchEvent::Delivered(delivery) },
        },
    );
}
fn document(uses: Option<List<c::AccountUse>>) -> c::Accounts {
    c::Accounts {
        accounts: List::from_boxed(Box::new([c::AccountLine {
            number: 1,
            place: 1,
            provider: Some(1),
            provider_name: None,
            display: text(b"Account"),
            holder: c::AccountHolder::Party(f::PARTY),
            plan: true,
            uses,
            sources: List::uncapped(),
            facts: List::uncapped(),
            manage_usage: None,
            list: None,
            cut: false,
        }])),
        program_hosts: List::uncapped(),
        readiness: List::uncapped(),
        readiness_cut: false,
        signing: List::uncapped(),
        lendable: false,
        lending: None,
        lent_end: None,
        last_end: None,
        usable: true,
        settled: true,
        cut: false,
    }
}
fn setup(uses: Option<List<c::AccountUse>>) -> (Client, c::Me, pages::Pages) {
    let mut client = Client::unlinked(&f::limits().client, f::SEED).unwrap();
    for event in [
        Event::StartUnlinked { kept: List::uncapped() },
        Event::SignIn { with: jig_client::With::Transport },
        Event::Up { up: jig_client::Up::Served { served: true, versions: None } },
        Event::Watch { handle: pages::me_handle(), subject: Watchable::Jig(c::Subject::Me) },
    ] {
        client_event(&mut client, event);
    }
    let mut me = f::chats::me();
    let mut first = me.projects.get(0).unwrap().clone();
    first.project = 9;
    let mut second = first.clone();
    second.project = 2;
    me.projects = List::from_boxed(Box::new([first, second]));
    deliver(&mut client, 0, c::Delivery::Me(Box::new(me.clone())));
    let out = client_event(
        &mut client,
        Event::Watch { handle: accounts::handle(), subject: Watchable::Jig(c::Subject::Accounts) },
    );
    let watch = out
        .iter()
        .find_map(|r| {
            if let jig_client::UnlinkedRequest::Down { down: jig_client::Down::Open { watch, .. } } = r {
                Some(*watch)
            } else {
                None
            }
        })
        .expect("accounts watch");
    client_event(
        &mut client,
        Event::Up {
            up: jig_client::Up::Watched {
                watch,
                event: c::WatchEvent::Delivered(c::Delivery::Accounts(Box::new(document(uses)))),
            },
        },
    );
    assert!(accounts::current_me(&client).is_some(), "me delivered");
    assert!(accounts::document(&client).is_some(), "accounts delivered");
    let mut pages = pages::Pages::new();
    step(&mut pages, &client, pages::Event::Told { told: Told::Changed { handle: pages::me_handle() } });
    (client, me, pages)
}
fn step(pages: &mut pages::Pages, client: &Client, event: pages::Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    pages::step(pages, &f::env(&f::limits()), client, event, &mut out);
    out
}
fn reads(out: &Queue<Request>) -> Box<[(Handle, u32)]> {
    out.iter()
        .filter_map(|request| {
            if let Request::Client(Event::Read { handle, read: Readable::Jig(c::Read::Project { project }) }) = request
            {
                Some((*handle, *project))
            } else {
                None
            }
        })
        .collect()
}
fn answer(pages: &mut pages::Pages, client: &Client, handle: Handle, result: c::ReadResult) -> Queue<Request> {
    step(pages, client, pages::Event::Told { told: Told::Page { handle, page: Readout::Jig(result) } })
}
fn project() -> c::Project {
    let limits = f::limits();
    let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).unwrap();
    let mut project =
        jig_client_protocol_link::translate::documents::project(jig_client_test_records::owner_project_record(), &link)
            .unwrap();
    project.number = 2;
    project.members = List::uncapped();
    for (party, name, login) in [(1, b"Ana".as_slice(), b"ana".as_slice()), (2, b"", b"ben"), (3, b"", b"")] {
        project
            .members
            .push(c::ProjectMember {
                party,
                name: text(name),
                login: text(login),
                kind: c::PartyKind::Person,
                role: c::HeldRole::Member,
            })
            .unwrap();
    }
    project
}
#[test]
fn accounts_read_people_in_project_order_and_refresh_when_projects_change() {
    let (mut client, mut me, mut pages) = setup(Some(List::from_boxed(Box::new([c::AccountUse::Party(1)]))));
    let out = step(&mut pages, &client, pages::Event::Went { address: Address::Accounts });
    let first = reads(&out);
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].1, 9, "me's order, not project number order");
    assert_eq!(pages.accounts().unwrap().party_name(1), None);
    assert!(
        reads(&step(&mut pages, &client, pages::Event::Link { link: jig_client::Link::Live })).is_empty(),
        "only one outstanding read"
    );
    let out = answer(&mut pages, &client, first[0].0, c::ReadResult::None(c::ReadNone::Refused));
    let second = reads(&out);
    assert_eq!(second.len(), 1);
    assert_eq!(second[0].1, 2);
    assert_eq!(pages.accounts().unwrap().party_name(1), None, "a refusal guesses no person");
    let out =
        answer(&mut pages, &client, second[0].0, c::ReadResult::Value(c::ReadValue::Project(Box::new(project()))));
    assert_eq!(reads(&out).as_ref(), []);
    let account = pages.accounts().unwrap();
    assert_eq!(account.party_name(1), Some(b"Ana".as_slice()));
    assert_eq!(account.party_name(2), Some(b"ben".as_slice()));
    assert_eq!(account.party_name(3), None);
    assert_eq!(account.party_name(9), None);
    me.projects.get_mut(0).unwrap().name = text(b"Changed");
    deliver(&mut client, 0, c::Delivery::Me(Box::new(me)));
    let out = step(&mut pages, &client, pages::Event::Told { told: Told::Changed { handle: pages::me_handle() } });
    assert_eq!(reads(&out).iter().map(|r| r.1).collect::<Box<[_]>>().as_ref(), [9]);
    assert_eq!(pages.accounts().unwrap().party_name(1), None, "new project list reads names anew");
    let out = step(&mut pages, &client, pages::Event::Went { address: Address::Members });
    assert!(pages.accounts().is_none());
    let own_read = reads(&out);
    assert_eq!(own_read.len(), 1, "Members owns its own visit");
    let out = answer(&mut pages, &client, first[0].0, c::ReadResult::None(c::ReadNone::Refused));
    assert!(reads(&out).is_empty(), "an old account read cannot continue after leaving");
}
#[test]
fn accounts_without_people_uses_send_no_project_reads() {
    for uses in [None, Some(List::uncapped()), Some(List::from_boxed(Box::new([c::AccountUse::Project(9)])))] {
        let (client, _, mut pages) = setup(uses);
        let out = step(&mut pages, &client, pages::Event::Went { address: Address::Accounts });
        assert_eq!(reads(&out).as_ref(), []);
        assert_eq!(pages.accounts().unwrap().party_name(1), None);
    }
}
