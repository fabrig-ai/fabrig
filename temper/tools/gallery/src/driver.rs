//! The one event path used by the sample worlds and their follow-up acts.
use jig_client::contract as c;
use skein_lib::{Env, Queue, Time, Token, Wall};
use temper_client_domain::{Domain, Event, Limits, Request};

pub fn limits() -> Limits {
    temper_client_shell_terminal::limits::native().client
}
pub fn step(root: &mut Domain, event: Event, out: &mut Queue<Request>) {
    temper_client_domain::step(root, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits() }, event, out);
}
pub fn deliver(root: &mut Domain, watch: Token, value: c::Delivery, out: &mut Queue<Request>) {
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::Watched { watch, event: c::WatchEvent::Delivered(value) },
            },
        },
        out,
    );
}
pub fn answer(root: &mut Domain, read: Token, value: c::ReadValue, out: &mut Queue<Request>) {
    step(
        root,
        Event::Client {
            event: jig_client::Event::Up {
                up: jig_client::Up::ReadDone { read, result: jig_client::Readout::Jig(c::ReadResult::Value(value)) },
            },
        },
        out,
    );
}
/// The transport obligations a sample answers; the other shell obligations carry no documents.
pub fn down(request: Request) -> Option<jig_client::Down> {
    match request {
        Request::Client { request } => match request {
            jig_client::Request::Down { down } => Some(down),
            // M4's Authorize place answers it
            jig_client::Request::Authorized(_)
            | jig_client::Request::Exchange { .. }
            | jig_client::Request::OpenStream { .. }
            | jig_client::Request::CloseStream
            | jig_client::Request::Save { .. }
            | jig_client::Request::Erase { .. }
            | jig_client::Request::Told { .. }
            | jig_client::Request::SignOut
            | jig_client::Request::ServiceCredential { .. } => None,
        },
        Request::Show { .. }
        | Request::Leave { .. }
        | Request::SignOut
        | Request::Dropped { .. }
        | Request::Fact { .. } => None,
    }
}
