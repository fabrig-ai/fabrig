use super::fixtures;
use crate::{
    Address, ChatsFilter,
    pages::{self, Act, Event, Pages, Request, switcher},
};
use jig_client::{Told, contract as c};
use skein_lib::{List, Queue, Token};

const OTHER: u32 = 2;
const UNKNOWN: u32 = 3;
// The owner's project and OTHER are the fixture's two membership rows.
const PROJECT_ROWS: u32 = 2;

struct Fixture {
    root: crate::Domain,
    pages: Pages,
    watch: Token,
    out: Queue<Request>,
}

fn fixture() -> Fixture {
    let limits = fixtures::limits();
    let mut root = crate::unlinked(&limits, fixtures::SEED);
    let watch = fixtures::start(&mut root, List::with_capacity(0));
    Fixture { root, pages: Pages::new(), watch, out: Queue::uncapped() }
}

fn event(f: &mut Fixture, event: Event) {
    pages::step(&mut f.pages, &fixtures::env(&fixtures::limits()), f.root.client(), event, &mut f.out);
}

fn deliver(f: &mut Fixture, me: c::Me) {
    crate::step(
        &mut f.root,
        &fixtures::env(&fixtures::limits()),
        crate::Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: f.watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut Queue::uncapped(),
    );
    event(f, Event::Told { told: Told::Changed { handle: pages::me_handle() } });
}

fn two_projects() -> c::Me {
    let mut me = fixtures::chats::me();
    let owner = me.projects.get(0).expect("first project").clone();
    let mut other = owner.clone();
    other.project = OTHER;
    me.projects = List::with_capacity(PROJECT_ROWS);
    me.projects.push(owner).expect("first");
    me.projects.push(other).expect("second");
    me
}

#[test]
fn project_addresses_wait_until_me_lists_their_project() {
    for address in [
        Address::Home { project: OTHER },
        Address::Settings { project: OTHER },
        Address::Repositories { project: OTHER },
        Address::Configure { project: OTHER },
    ] {
        let mut f = fixture();
        event(&mut f, Event::Went { address });
        assert_eq!(f.pages.project(), None);
        deliver(&mut f, fixtures::chats::me());
        assert_eq!(f.pages.project(), Some(fixtures::PROJECT));
        deliver(&mut f, two_projects());
        assert_eq!(f.pages.project(), Some(OTHER));
    }
}

#[test]
fn going_to_project_pages_opens_the_named_project() {
    for address in [
        Address::Home { project: OTHER },
        Address::Settings { project: OTHER },
        Address::Repositories { project: OTHER },
        Address::Configure { project: OTHER },
    ] {
        let mut f = fixture();
        deliver(&mut f, two_projects());
        event(&mut f, Event::Went { address });
        assert_eq!(f.pages.project(), Some(OTHER));
    }
}

#[test]
fn unlisted_project_addresses_keep_the_open_project() {
    for address in [
        Address::Home { project: UNKNOWN },
        Address::Settings { project: UNKNOWN },
        Address::Repositories { project: UNKNOWN },
        Address::Configure { project: UNKNOWN },
    ] {
        let mut f = fixture();
        deliver(&mut f, two_projects());
        event(&mut f, Event::Act { act: Act::Project { project: OTHER } });
        event(&mut f, Event::Went { address });
        deliver(&mut f, two_projects());
        assert_eq!(f.pages.project(), Some(OTHER));
    }
}

#[test]
fn switching_projects_keeps_home_settings_configure_or_chats() {
    for (address, shown) in [
        (Address::Home { project: fixtures::PROJECT }, Some(Address::Home { project: OTHER })),
        (Address::Changes { project: fixtures::PROJECT }, Some(Address::Changes { project: OTHER })),
        (Address::Settings { project: fixtures::PROJECT }, Some(Address::Settings { project: OTHER })),
        (Address::Configure { project: fixtures::PROJECT }, Some(Address::Configure { project: OTHER })),
        (Address::Chats { filter: ChatsFilter::All }, None),
        (Address::NewChat, None),
    ] {
        let mut f = fixture();
        deliver(&mut f, two_projects());
        event(&mut f, Event::Went { address: address.clone() });
        while f.out.pop().is_some() {}
        event(&mut f, Event::Act { act: Act::Project { project: OTHER } });
        assert_eq!(f.pages.project(), Some(OTHER));
        assert_eq!(f.pages.address(), &address);
        let mut requested = None;
        while let Some(request) = f.out.pop() {
            match request {
                Request::Show { address } => requested = Some(address),
                Request::Client(_) | Request::Replace { .. } => {}
            }
        }
        assert_eq!(requested, shown);
    }
}

#[test]
fn project_pages_keep_the_switchers_rename_form() {
    for address in [
        Address::Home { project: fixtures::PROJECT },
        Address::Changes { project: fixtures::PROJECT },
        Address::Settings { project: fixtures::PROJECT },
        Address::Repositories { project: fixtures::PROJECT },
        Address::Configure { project: fixtures::PROJECT },
    ] {
        let mut f = fixture();
        deliver(&mut f, two_projects());
        event(&mut f, Event::Act { act: Act::Switcher(switcher::Act::Rename) });
        event(
            &mut f,
            Event::Act { act: Act::Switcher(switcher::Act::RenameWords { words: b"Changed".as_slice().into() }) },
        );
        event(&mut f, Event::Went { address });
        assert_eq!(f.pages.switcher().renaming(), Some(fixtures::PROJECT));
        assert_eq!(f.pages.switcher().rename_words(), b"Changed");
    }
}

#[test]
fn choosing_a_project_keeps_the_repositories_tab() {
    let mut f = fixture();
    deliver(&mut f, two_projects());
    event(&mut f, Event::Went { address: Address::Repositories { project: fixtures::PROJECT } });
    while f.out.pop().is_some() {}
    event(&mut f, Event::Act { act: Act::Project { project: OTHER } });
    assert_eq!(f.pages.project(), Some(OTHER));
    assert_eq!(f.out.pop(), Some(Request::Show { address: Address::Repositories { project: OTHER } }));
}

#[test]
fn choosing_a_project_stays_on_forges_and_accounts() {
    for address in [Address::Forges, Address::Accounts] {
        let mut f = fixture();
        deliver(&mut f, two_projects());
        event(&mut f, Event::Went { address: address.clone() });
        while f.out.pop().is_some() {}
        event(&mut f, Event::Act { act: Act::Project { project: OTHER } });
        assert_eq!(f.pages.project(), Some(OTHER));
        assert_eq!(f.pages.address(), &address);
        assert!(f.out.is_empty());
    }
}
#[test]
fn choosing_a_project_keeps_the_builders_tab() {
    let mut f = fixture();
    deliver(&mut f, two_projects());
    event(&mut f, Event::Went { address: Address::Builders { project: fixtures::PROJECT } });
    while f.out.pop().is_some() {}
    event(&mut f, Event::Act { act: Act::Project { project: OTHER } });
    assert_eq!(f.pages.project(), Some(OTHER));
    assert_eq!(f.out.pop(), Some(Request::Show { address: Address::Builders { project: OTHER } }));
}
