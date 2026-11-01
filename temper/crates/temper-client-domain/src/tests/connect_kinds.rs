//! Kind choices make forges and follow their records (temper/docs/design/draft/playground-reset.md, 3).
use super::{connect as fixtures, fixtures as f};
use crate::{Address, Choice, Event, Place, Request, Step, forge};
use forge::{
    connect,
    forges::{Kind, Offer, State},
};
use jig_client::{self as client, Down, Up};
use skein_lib::{List, Queue, Secret, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

fn event(root: &mut crate::Domain, event: Event) -> Queue<Request> {
    let mut out = Queue::uncapped();
    crate::step(root, &f::env(&f::limits()), event, &mut out);
    out
}
fn row(number: u16, kind: w::Kind, mode: w::Mode) -> w::ForgeRow {
    w::ForgeRow::new(
        &w::CEILINGS,
        w::ForgeRowParts {
            skein_bools: [false; 4],
            number,
            kind,
            mode,
            connection: w::Connection::Connecting,
            offer: w::Offer::NotOffered,
            first: None,
            installations: List::with_capacity(0),
            ended: None,
            app: None,
            web: None,
            repositories: List::with_capacity(0),
            owned: None,
            ended_stop: None,
        },
    )
    .unwrap()
}
fn deliver(root: &mut crate::Domain, watch: Token, owned: w::Offer, rows: List<w::ForgeRow>) -> Queue<Request> {
    let record = w::Forges::new(&w::CEILINGS, w::ForgesParts { forges: rows, owned }).unwrap();
    event(
        root,
        Event::Client {
            event: client::Event::Up {
                up: Up::FamilyDelivered {
                    watch,
                    record: client::Family {
                        family: wire::FORGE,
                        version: wire::CLIENT_VERSION,
                        bytes: wire::forges(&record),
                    },
                },
            },
        },
    )
}
fn empty(owned: w::Offer) -> (crate::Domain, Token) {
    let (mut root, _, watch) = fixtures::opened(true);
    deliver(&mut root, watch, owned, List::with_capacity(0));
    (root, watch)
}
fn sent(out: &Queue<Request>) -> w::Request {
    for request in out {
        if let Request::Client {
            request: client::Request::Down { down: Down::Send { request: client::Ask::Family { record, .. }, .. } },
        } = request
        {
            return wire::decode_requesting(&record.bytes, &w::CEILINGS).unwrap().into_parts().request;
        }
    }
    panic!("family request sent");
}
#[test]
fn kind_choices_round_trip_in_the_address() {
    for (choice, value) in [(Choice::GitHub, "github"), (Choice::Owned, "owned"), (Choice::Forgejo, "forgejo")] {
        for address in [
            Address::ConnectForge { forge: Some(choice) },
            Address::Welcome { step: Step::Forge { forge: Some(choice) } },
        ] {
            let printed = crate::address::print(&address);
            assert!(printed.ends_with(value.as_bytes()));
            assert_eq!(crate::address::parse(&printed), Place::Address(address));
        }
    }
}
#[test]
fn an_empty_store_offers_github_and_the_owned_forgejo() {
    let (root, _) = empty(w::Offer::Offered);
    assert_eq!(
        connect::choices(root.forges(), None).as_slice(),
        &[Choice::GitHub, Choice::Owned, Choice::Forgejo, Choice::Elsewhere]
    );
    assert_eq!(connect::owned_offer(root.forges()), Offer::Offered);
}
#[test]
fn owned_is_not_offered_without_the_template() {
    let (root, _) = empty(w::Offer::NotOffered);
    assert_eq!(connect::choices(root.forges(), None).as_slice(), &[Choice::GitHub, Choice::Forgejo, Choice::Elsewhere]);
    let (root, _) = empty(w::Offer::NotUnderTheBox);
    assert_eq!(
        connect::choices(root.forges(), None).as_slice(),
        &[Choice::GitHub, Choice::Owned, Choice::Forgejo, Choice::Elsewhere]
    );
}
#[test]
fn connect_github_on_its_kind_sends_a_new_github_forge() {
    for back in [b"/welcome?at=forge".as_slice(), b"/forges/connect"] {
        let (mut root, _) = empty(w::Offer::Offered);
        let out = f::form_act(&mut root, forge::Act::ConnectManifest { forge: None, back: back.into() });
        let w::Request::Connect(request) = sent(&out) else { panic!("connect") };
        assert_eq!(*request.forge(), w::ConnectForge::Github);
        let w::How::GithubManifest(manifest) = request.how() else { panic!("manifest") };
        assert_eq!(manifest.back().as_deref(), Some(back));
    }
    let (mut root, _) = empty(w::Offer::Offered);
    f::form_act(&mut root, forge::Act::AppWrite { text: b"7".as_slice().into() });
    event(&mut root, Event::Forge(forge::Event::AppKey { value: Secret::new(b"fixture".as_slice().into()) }));
    let out = f::form_act(&mut root, forge::Act::ConnectApp { forge: None });
    let w::Request::Connect(request) = sent(&out) else { panic!("connect") };
    assert_eq!(*request.forge(), w::ConnectForge::Github);
    let w::How::GithubApp(_) = request.how() else { panic!("App") };
}
#[test]
fn run_on_this_machine_on_its_kind_sends_set_up() {
    let (mut root, _) = empty(w::Offer::Offered);
    let out = f::form_act(&mut root, forge::Act::SetUpAtOnce);
    let w::Request::SetUp(request) = sent(&out) else { panic!("set up") };
    assert_eq!(request.org(), b"work");
    assert_eq!(request.repository(), b"hello");
    assert_eq!(root.forge().set_up().unwrap().forge, None);
    assert_eq!(connect::following(root.forges()).unwrap().choice, Choice::Owned);
}
#[test]
fn a_managed_forgejo_sends_its_address_and_admin_token() {
    let (mut root, _) = empty(w::Offer::NotOffered);
    f::form_act(
        &mut root,
        forge::Act::AdminWrite {
            field: forge::admin::Field::Address,
            text: b"https://forge.example".as_slice().into(),
        },
    );
    event(&mut root, Event::Forge(forge::Event::AdminToken { value: Secret::new(b"fixture".as_slice().into()) }));
    let out = f::form_act(&mut root, forge::Act::ConnectAdmin { forge: None });
    let w::Request::Connect(request) = sent(&out) else { panic!("connect") };
    let w::ConnectForge::Forgejo(target) = request.forge() else { panic!("new Forgejo") };
    assert_eq!(target.address(), b"https://forge.example");
    let w::How::ForgejoAdmin(_) = request.how() else { panic!("admin") };
    assert!(!root.forge().admin_form().has_token());
}
#[test]
fn the_step_follows_the_forge_made() {
    for (choice, kind, mode) in [
        (Choice::GitHub, w::Kind::Github, w::Mode::Managed),
        (Choice::Owned, w::Kind::Forgejo, w::Mode::Owned),
        (Choice::Forgejo, w::Kind::Forgejo, w::Mode::Managed),
    ] {
        let (mut root, watch) = empty(w::Offer::Offered);
        let mut rows = List::with_capacity(1);
        rows.push(row(4, kind.clone(), mode.clone())).unwrap();
        deliver(&mut root, watch, w::Offer::Offered, rows);
        match choice {
            Choice::GitHub => {
                f::form_act(
                    &mut root,
                    forge::Act::ConnectManifest { forge: None, back: b"/forges/connect".as_slice().into() },
                );
            }
            Choice::Owned => {
                f::form_act(&mut root, forge::Act::SetUpAtOnce);
            }
            Choice::Forgejo => {
                f::form_act(
                    &mut root,
                    forge::Act::AdminWrite {
                        field: forge::admin::Field::Address,
                        text: b"https://forge.example".as_slice().into(),
                    },
                );
                event(
                    &mut root,
                    Event::Forge(forge::Event::AdminToken { value: Secret::new(b"fixture".as_slice().into()) }),
                );
                f::form_act(&mut root, forge::Act::ConnectAdmin { forge: None });
            }
            Choice::Forge(_) | Choice::Elsewhere => panic!("kind"),
        }
        let mut rows = List::with_capacity(4);
        rows.push(row(4, kind.clone(), mode.clone())).unwrap();
        rows.push(row(5, kind.clone(), mode.clone())).unwrap();
        rows.push(row(6, kind.clone(), mode.clone())).unwrap();
        rows.push(row(7, w::Kind::Forgejo, w::Mode::Fake)).unwrap();
        deliver(&mut root, watch, w::Offer::Offered, rows);
        assert_eq!(connect::step(Some(choice), root.forges(), None), connect::Shown::SettingUp(6));
        let personal = connect::personal(root.forges(), None).unwrap();
        assert_eq!(personal.number, 6);
        assert_eq!(personal.kind, if choice == Choice::GitHub { Kind::GitHub } else { Kind::Forgejo });
        let mut rows = List::with_capacity(1);
        rows.push(row(9, kind, mode)).unwrap();
        deliver(&mut root, watch, w::Offer::Offered, rows);
        assert_eq!(connect::following(root.forges()).unwrap().forge, Some(6));
    }
}
fn make_at(address: Address) -> (crate::Domain, Token, Queue<Request>) {
    let (mut root, watch) = empty(w::Offer::Offered);
    event(&mut root, Event::Went { place: Place::Address(address) });
    f::form_act(&mut root, forge::Act::ConnectManifest { forge: None, back: b"/forges/connect".as_slice().into() });
    let mut rows = List::with_capacity(1);
    rows.push(row(8, w::Kind::Github, w::Mode::Managed)).unwrap();
    let out = deliver(&mut root, watch, w::Offer::NotOffered, rows);
    (root, watch, out)
}
#[test]
fn a_made_forge_replaces_the_kind_address_with_its_row() {
    for (address, expected) in [
        (
            Address::ConnectForge { forge: Some(Choice::GitHub) },
            Address::ConnectForge { forge: Some(Choice::Forge(8)) },
        ),
        (Address::ConnectForge { forge: None }, Address::ConnectForge { forge: Some(Choice::Forge(8)) }),
        (
            Address::Welcome { step: Step::Forge { forge: None } },
            Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(8)) } },
        ),
    ] {
        let (mut root, watch, out) = make_at(address);
        let mut replaced = false;
        for request in &out {
            if let Request::Show { address, replace: true } = request {
                replaced |= *address == expected;
            }
        }
        assert!(replaced);
        event(&mut root, Event::Went { place: Place::Address(expected) });
        let mut rows = List::with_capacity(1);
        rows.push(row(8, w::Kind::Github, w::Mode::Managed)).unwrap();
        let out = deliver(&mut root, watch, w::Offer::NotOffered, rows);
        for request in &out {
            if let Request::Show { .. } = request {
                panic!("no repeated Show");
            }
        }
    }
}
#[test]
fn all_choices_after_a_made_forge_opens_choose_again() {
    let (mut root, _, _) = make_at(Address::ConnectForge { forge: Some(Choice::GitHub) });
    event(&mut root, Event::Went { place: Place::Address(Address::ConnectForge { forge: Some(Choice::Forge(8)) }) });
    event(&mut root, Event::Went { place: Place::Address(Address::ConnectForge { forge: None }) });
    assert_eq!(connect::following(root.forges()), None);
    assert_eq!(connect::step(None, root.forges(), None), connect::Shown::Choose);
    assert!(connect::choices(root.forges(), None).as_slice().contains(&Choice::GitHub));
    let State::Rows(..) = root.forges() else { panic!("rows") };
}
