//! Milestone list and goal deliveries shared by the grouped views' tests.
use super as f;
use jig_client::{Down, Event, UnlinkedRequest, Up, Watchable, contract as c};
use skein_lib::{Env, List, Queue, Time, Token};
use temper_client_domain::{self as d, Limits};

pub const LABEL: u32 = 44;
pub const TASK: u64 = 1000;

pub fn event(root: &mut d::Domain, limits: &Limits, event: d::Event) -> Queue<d::Request> {
    let mut out = Queue::uncapped();
    d::step(root, &Env { now: Time::ZERO, wall: f::NOW, limits: *limits }, event, &mut out);
    out
}
pub fn offer(root: &mut d::Domain, me_watch: Token, limits: &Limits) -> Queue<d::Request> {
    let mut me = root.pages().me(root.client()).unwrap().clone();
    let mut template = f::goals::template(temper_client_domain::milestones::TEMPLATE, b"milestone", true);
    template.label = LABEL;
    let member = me.projects.get_mut(0).unwrap();
    let mut templates = List::uncapped();
    for held in &member.templates {
        templates.push(held.clone()).unwrap();
    }
    templates.push(template).unwrap();
    member.templates = templates;
    event(
        root,
        limits,
        d::Event::Client {
            event: Event::Up {
                up: Up::Watched { watch: me_watch, event: c::WatchEvent::Delivered(c::Delivery::Me(Box::new(me))) },
            },
        },
    )
}
#[must_use]
pub fn watch(out: &Queue<d::Request>, label: Option<u32>, parent: Option<u64>, window: u32) -> Token {
    for request in out {
        if let d::Request::Unlinked {
            request:
                UnlinkedRequest::Down {
                    down: Down::Open { watch, subject: Watchable::Jig(c::Subject::Tasks { query, window: size }) },
                },
        } = request
            && query.label == label
            && query.parent == parent
            && *size == window
        {
            return *watch;
        }
    }
    panic!("milestone fixture watch");
}
pub fn live(
    root: &mut d::Domain,
    limits: &Limits,
    watch: Token,
    lines: &[c::TaskLine],
    more: bool,
) -> Queue<d::Request> {
    event(
        root,
        limits,
        d::Event::Client {
            event: Event::Up {
                up: Up::Watched {
                    watch,
                    event: c::WatchEvent::Delivered(c::Delivery::Tasks(Box::new(c::Tasks {
                        rows: List::from_boxed(lines.into()),
                        more,
                    }))),
                },
            },
        },
    )
}
