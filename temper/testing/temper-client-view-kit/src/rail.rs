use super as f;
use jig_client::contract as c;
use skein_lib::{Env, List, Queue, Time, Token};
use temper_client_domain::{self as d, Address, Event};

pub const GOAL_LABEL: u32 = 2;

pub const GOAL_PRIORITY: u32 = 1;

pub const TEXT_BYTES: u32 = 64;

#[expect(
    clippy::disallowed_methods,
    reason = "the shared test fixture keeps its semantic inspection and setup unchanged"
)]
pub fn with_goal_fact(root: &mut d::Domain, me_watch: Token, delivered: Option<bool>) {
    let mut me = root.pages().me(root.client()).unwrap().clone();
    let mut flags = super::chats::line().flags;
    flags.conversation = false;
    flags.listed = true;
    let goal = c::Template {
        resources: List::uncapped(),
        number: jig_client::shapes::GOAL,
        name: f::task::document_words(b"goal"),
        executor: c::TemplateExecutor::Charter(1),
        contract: c::TaskContractDocument::Report { words: TEXT_BYTES },
        shallower: 1,
        budget: c::TemplateBudget::AsAllowed { cap: None },
        deadline: c::TemplateDeadline::None,
        tracked: Some(GOAL_LABEL),
        flags,
        label: GOAL_LABEL,
        grace: None,
        llm: None,
    };
    let mut asking = goal.clone();
    asking.number = 5;
    asking.executor = c::TemplateExecutor::Charter(11);
    me.projects.get_mut(0).unwrap().templates = List::from_boxed(Box::new([goal, asking]));
    let mut out = Queue::uncapped();
    d::step(
        root,
        &Env { now: Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched {
                    watch: me_watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))),
                },
            },
        },
        &mut out,
    );
    if let Some(has_goal) = delivered {
        let watch = out
            .iter()
            .find_map(|request| match request {
                d::Request::Unlinked {
                    request:
                        jig_client::UnlinkedRequest::Down {
                            down:
                                jig_client::Down::Open {
                                    watch,
                                    subject: jig_client::Watchable::Jig(c::Subject::Tasks { query, window: 1 }),
                                },
                        },
                } if query.tracked == Some(true) => Some(*watch),
                d::Request::Client { .. }
                | d::Request::Unlinked { .. }
                | d::Request::Show { .. }
                | d::Request::Leave { .. }
                | d::Request::SignOut
                | d::Request::Dropped { .. }
                | d::Request::Screen { .. }
                | d::Request::KeepLook { .. }
                | d::Request::KeepSignIn { .. }
                | d::Request::Tab { .. }
                | d::Request::Copy { .. }
                | d::Request::Push(_)
                | d::Request::Fact { .. } => None,
            })
            .expect("goal fact watch");
        let mut rows = List::uncapped();
        if has_goal {
            let mut row = super::chats::line();
            row.tracked = Some(GOAL_PRIORITY);
            row.flags.conversation = false;
            rows.push(row).unwrap();
        }
        deliver_document(root, watch, c::Delivery::Tasks(Box::new(c::Tasks { rows, more: false })));
    }
}

pub fn deliver_document(root: &mut d::Domain, watch: Token, document: c::Delivery) {
    f::root_event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(document) },
            },
        },
    );
}

#[expect(
    clippy::disallowed_methods,
    reason = "the shared test fixture keeps its semantic inspection and setup unchanged"
)]
pub fn with_people(root: &mut d::Domain, people: u32) {
    let address = root.pages().address().clone();
    f::root_event(root, Event::Went { place: d::Place::Address(Address::Forges) });
    let mut out = Queue::uncapped();
    d::step(
        root,
        &Env { now: Time::ZERO, wall: f::NOW, limits: f::domain_limits() },
        Event::Went { place: d::Place::Address(Address::Settings { project: f::PROJECT }) },
        &mut out,
    );
    let read = out
        .iter()
        .find_map(|request| {
            if let d::Request::Unlinked {
                request:
                    jig_client::UnlinkedRequest::Down {
                        down: jig_client::Down::Read { read, what: jig_client::Readable::Jig(c::Read::Project { .. }) },
                    },
            } = request
            {
                Some(*read)
            } else {
                None
            }
        })
        .expect("rail's project read");
    let limits = f::domain_limits();
    let link = jig_client_protocol_link::Limits::new(limits.client.domain, limits.client.exchanges).unwrap();
    let mut document =
        jig_client_protocol_link::translate::documents::project(jig_client_test_records::owner_project_record(), &link)
            .unwrap();
    document.number = f::PROJECT;
    let reader = &root.pages().me(root.client()).unwrap().party;
    let member = c::ProjectMember {
        party: reader.number,
        kind: c::PartyKind::Person,
        name: reader.name.clone(),
        login: reader.login.clone(),
        role: c::HeldRole::Owner,
    };
    document.members = List::with_capacity(people);
    for number in 0..people {
        document.members.push(c::ProjectMember { party: u64::from(number), ..member.clone() }).unwrap();
    }
    f::root_event(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone {
                    read,
                    result: jig_client::Readout::Jig(c::ReadResult::Value(c::ReadValue::Project(Box::new(document)))),
                },
            },
        },
    );
    f::root_event(root, Event::Went { place: d::Place::Address(address) });
}
