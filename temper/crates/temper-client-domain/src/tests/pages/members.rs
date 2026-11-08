//! Members' ordered one-shot reads (temper/docs/design/client/README.md, 3.5).
#![expect(clippy::disallowed_methods, reason = "ordinary tests select reads and inspect outcomes")]
use super::{
    accounts::{deliver, event},
    fixtures,
};
use crate::{Address, Event, Place, Request, pages::members::Read};
use jig_client::{Down, Readable, UnlinkedRequest, contract as c};
use skein_lib::{List, Queue, Token};

fn reads(out: &Queue<Request>) -> Box<[(Token, u32)]> {
    out.iter().filter_map(|r| {
        if let Request::Unlinked { request: UnlinkedRequest::Down {
            down: Down::Read { read, what: Readable::Jig(c::Read::Project { project }) },
        }} = r { Some((*read, *project)) } else { None }
    }).collect()
}
fn state(read: &Read) -> &'static str {
    match read {
        Read::Unread => "Unread",
        Read::Refused => "Refused",
        Read::Project(_) => "Project",
    }
}
fn answer(root: &mut crate::Domain, read: Token, why: c::ReadNone) -> Queue<Request> {
    event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone { read, result: jig_client::Readout::Jig(c::ReadResult::None(why)) },
            },
        },
    )
}
#[test]
fn members_read_in_order_refresh_keep_refusals_and_release() {
    for change_pending in [false, true] {
        let mut root = crate::unlinked(&fixtures::limits(), fixtures::SEED);
        let watch = fixtures::start(&mut root, List::uncapped());
        deliver(&mut root, watch, fixtures::frame(fixtures::PARTY));
        let mut me = root.pages().me(root.client()).unwrap().clone();
        let first = c::MeProject {
            project: 9,
            name: c::DocumentText::complete(c::Text::new(b"First".as_slice().into(), 64).unwrap()),
            role: c::HeldRole::Owner,
            pool: None,
            templates: List::uncapped(),
            creating: None,
            models: List::uncapped(),
            defaults: List::uncapped(),
            public: None,
            left_out: 0,
        };
        me.projects = List::uncapped();
        me.projects.push(first.clone()).unwrap();
        let mut second = first.clone();
        second.project = 2;
        me.projects.push(second).unwrap();
        deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me.clone()))));
        let out = event(&mut root, Event::Went { place: Place::Address(Address::Members) });
        let pending = reads(&out);
        assert_eq!(pending.len(), 1, "only one project read is outstanding");
        assert_eq!(pending[0].1, first.project);
        assert_eq!(state(root.pages().members().unwrap().read(2)), "Unread");
        let mut third = first.clone();
        third.project = 3;
        if change_pending {
            me.projects.push(third.clone()).unwrap();
            let out = deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me.clone()))));
            assert!(reads(&out).is_empty(), "a changed project list waits for the outstanding read");
        }
        let out = answer(&mut root, pending[0].0, c::ReadNone::Refused);
        let next = reads(&out);
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].1, if change_pending { first.project } else { 2 });
        if !change_pending {
            assert_eq!(state(root.pages().members().unwrap().read(first.project)), "Refused");
            let out = answer(&mut root, next[0].0, c::ReadNone::Unknown);
            assert!(reads(&out).is_empty());
            me.projects.push(third).unwrap();
            let out = deliver(&mut root, watch, c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))));
            assert_eq!(reads(&out).iter().map(|r| r.1).collect::<Box<[_]>>().as_ref(), [first.project]);
        }
        if change_pending {
            let out = answer(&mut root, next[0].0, c::ReadNone::Refused);
            let second = reads(&out);
            assert_eq!(second.iter().map(|r| r.1).collect::<Box<[_]>>().as_ref(), [2]);
            let out = answer(&mut root, second[0].0, c::ReadNone::Refused);
            let third = reads(&out);
            assert_eq!(third.iter().map(|r| r.1).collect::<Box<[_]>>().as_ref(), [3]);
            assert_eq!(state(root.pages().members().unwrap().read(first.project)), "Refused");
            assert_eq!(state(root.pages().members().unwrap().read(2)), "Refused");
        }
        let out = event(&mut root, Event::Went { place: Place::Address(Address::Accounts) });
        assert!(root.pages().members().is_none());
        assert!(reads(&out).is_empty());
        let out = answer(&mut root, next[0].0, c::ReadNone::Refused);
        assert!(reads(&out).is_empty(), "a late answer cannot continue a released page");
    }
}
