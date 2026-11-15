//! Deployment facts (temper/docs/design/client/pages/README.md and temper/docs/design/client/forge/records.md).
use super::{Limits, Request, Side, wire_limits};
use alloc::boxed::Box;
use jig_client::{Handle, Link, Readable, Readout, Told, contract as c};
use skein_lib::Queue;
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Answer {
    pub welcome: bool,
    pub owned: bool,
    pub program_login: Option<Box<[u8]>>,
    pub push: Option<[u8; 65]>,
}
#[derive(Debug)]
pub(super) struct State {
    pub answer: Option<Answer>,
    pub pending: Option<Handle>,
    pub live: bool,
}
impl State {
    pub const fn new() -> Self {
        Self { answer: None, pending: None, live: false }
    }
}
pub(crate) fn told(side: &mut Side, told: &Told, limits: Limits, out: &mut Queue<Request>) -> bool {
    match told {
        Told::Link { link } => {
            if *link == Link::Live && !side.deployment.live {
                side.deployment.live = true;
                read(side, out);
            } else if *link != Link::Live {
                side.deployment.live = false;
            }
            false
        }
        Told::Page { handle, page } if Some(*handle) == side.deployment.pending => {
            side.deployment.pending = None;
            let answer = match page {
                Readout::Family { family, record: Some(record) } if *family == wire::FORGE => decode(record, limits),
                Readout::Family { .. } | Readout::NotServed { .. } | Readout::Limit | Readout::Jig(_) => None,
            };
            let answer = match answer {
                Some(answer) => answer,
                None => match side.deployment.answer.as_ref() {
                    Some(_) => return true,
                    None => Answer { welcome: false, owned: false, program_login: None, push: None },
                },
            };
            if side.deployment.answer.as_ref() != Some(&answer) {
                side.deployment.answer = Some(answer);
                side.changed();
            }
            true
        }
        Told::Page { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Text { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => false,
    }
}
fn decode(record: &[u8], limits: Limits) -> Option<Answer> {
    let answer = wire::decode_deployment_answer(record, &wire_limits(&limits)?)?.into_parts();
    Some(Answer {
        welcome: answer.welcome,
        owned: answer.owned,
        program_login: answer.program_login,
        push: answer.push,
    })
}
fn read(side: &mut Side, out: &mut Queue<Request>) {
    side.serial = side.serial.checked_add(1).expect("forge handles fit");
    let handle = Handle { asker: crate::owner::Owner::Forge.asker(), number: side.serial };
    side.deployment.pending = Some(handle);
    let record = w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::Deployment })
        .expect("fieldless deployment read");
    out.push(Request::Client {
        asked: jig_client::Event::Read {
            handle,
            read: Readable::Family {
                scope: c::Scope::Deployment,
                window: c::Window { rows: 0, after: None },
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::reading(&record),
                },
            },
        },
    });
}
