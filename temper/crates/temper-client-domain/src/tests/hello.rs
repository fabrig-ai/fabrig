//! The rail waits for the stream's hello (client/root.md, 1.4).
use super::fixtures as f;
use crate::{Address, Event, Request, WorkFilter};
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{Duration, List, Queue, Reader};

fn project_reads(out: &mut Queue<Request>) -> u32 {
    let mut count = 0_u32;
    while let Some(request) = out.pop() {
        if let Request::Client { request: jig_client::Request::Exchange { message, .. } } = request {
            let members = match message {
                jig_client::Message::Batch(bytes) => {
                    w::Batch::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap().members
                }
                jig_client::Message::Read(bytes) => List::from_boxed(Box::new([w::Member::Read(
                    w::Owned::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap(),
                )])),
                jig_client::Message::Authorize(_)
                | jig_client::Message::LinkAsk(_)
                | jig_client::Message::ApprovalAsk(_)
                | jig_client::Message::ApprovalRead(_)
                | jig_client::Message::ApprovalDecide(_)
                | jig_client::Message::SignInApproval(_)
                | jig_client::Message::Request(_)
                | jig_client::Message::Watch(_)
                | jig_client::Message::Unwatch(_)
                | jig_client::Message::StreamGone(_)
                | jig_client::Message::SignInLink(_)
                | jig_client::Message::SignInService
                | jig_client::Message::SignOut => List::with_capacity(0),
            };
            for member in &members {
                if let w::Member::Read(read) = member
                    && read.owner == w::Owner::Jig
                    && let w::Read::Project(project) =
                        w::Read::decode(&w::CEILINGS, &mut Reader::new(read.record.as_ref())).unwrap()
                {
                    assert_eq!(project.project, f::PROJECT);
                    count = count.checked_add(1).unwrap();
                }
            }
        }
    }
    count
}
#[test]
fn the_rails_project_is_read_once_in_the_hellos_batch() {
    let limits = f::limits();
    let env = f::env(&limits);
    let address = Address::Work { project: f::PROJECT, filter: WorkFilter::All };
    let mut root = crate::new(&limits, f::SEED);
    let mut out = Queue::uncapped();
    crate::step(
        &mut root,
        &env,
        Event::Start {
            address: address.clone(),
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
        &mut out,
    );
    crate::settle(&mut root, &env, &mut out);
    let Some(Request::Client { request: jig_client::Request::OpenStream { .. } }) = out.pop() else {
        panic!("stream alone")
    };
    assert!(out.is_empty(), "nothing of the frame or page before the hello");
    crate::step(&mut root, &env, opened(), &mut out);
    crate::settle(&mut root, &env, &mut out);
    assert_eq!(project_reads(&mut out), 1, "the hello carries the rail's only project read");
    crate::step(&mut root, &env, Event::Went { place: crate::Place::Address(address) }, &mut out);
    crate::settle(&mut root, &env, &mut out);
    assert_eq!(project_reads(&mut out), 0, "opening again does not repeat it");
}

fn opened() -> Event {
    let mut served = List::with_capacity(2);
    for (owner, version) in [
        (w::Owner::Jig, w::VERSION),
        (
            w::Owner::Family(w::Family { number: temper_client_protocol_wire::FORGE }),
            temper_client_protocol_wire::CLIENT_VERSION,
        ),
    ] {
        served.push(w::Served { owner, first: version, last: version }).unwrap();
    }
    let hello = jig_client_test_records::opened_with_layouts(
        w::Hello { stream: [7; 16], served, heartbeat: Duration::from_secs(30) },
        jig_client_test_records::frame_record(),
        &[jig_client_test_records::family_layout(
            temper_client_protocol_wire::FORGE,
            temper_client_protocol_wire::CLIENT_LAYOUT,
        )],
    );
    Event::Client {
        event: jig_client::Event::Streamed {
            message: jig_client::Streamed::Record(hello.encode(&w::CEILINGS).expect("fixture hello")),
        },
    }
}

#[test]
fn the_hello_marks_a_different_forge_layout_not_served() {
    use temper_client_protocol_wire::{CLIENT_LAYOUT, CLIENT_VERSION, FORGE};
    const OTHER_LAYOUT: [u8; 32] = {
        let mut layout = CLIENT_LAYOUT;
        layout[0] ^= 1;
        layout
    };
    const OWNERS: &[jig_client::Spoken] =
        &[jig_client::Spoken { family: FORGE, version: CLIENT_VERSION, layout: OTHER_LAYOUT }];
    let mut limits = f::limits();
    limits.client.domain.owners = OWNERS;
    let env = f::env(&limits);
    let mut root = crate::new(&limits, f::SEED);
    let mut out = Queue::uncapped();
    crate::step(
        &mut root,
        &env,
        Event::Start {
            address: Address::Work { project: f::PROJECT, filter: WorkFilter::All },
            kept: List::with_capacity(0),
            with: Some(jig_client::With::Transport),
        },
        &mut out,
    );
    crate::settle(&mut root, &env, &mut out);
    crate::step(&mut root, &env, opened(), &mut out);
    let link = root.client().link();
    assert_eq!(link.state, jig_client::Link::Live);
    assert_eq!(link.not_served, &[FORGE]);
    assert_eq!(root.forge().served(), crate::forge::Served::OutOfDate);
}
