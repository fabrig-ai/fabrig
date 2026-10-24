//! A departure is emitted only from the return answer, never from rendering or restoration.
use crate::{Address, Domain, Limits, ModelWay, Request, Step, WayKind, owner::Owner};
use jig_client::{Answered, Ask, Event, Key, Outcome, Told, contract as c};
use skein_lib::{Env, Queue};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub(crate) fn told(root: &mut Domain, env: &Env<Limits>, told: &Told, out: &mut Queue<Request>) -> bool {
    let Told::Answered { key, outcome } = told else { return false };
    if let Some((pending, _)) = &root.returning
        && pending == key
    {
        let (_, address) = root.returning.take().expect("matching return");
        if let Outcome::Answered(Answered::Family(record)) = outcome
            && record.family == wire::FORGE
            && record.version == wire::CLIENT_VERSION
            && let Some(answer) = return_answer(&record.bytes)
            && *answer.outcome() == w::Outcome::Returning
        {
            out.push(Request::Leave { to: address, replace: false });
        }
        return true;
    }
    if !root.departing || root.returning.is_some() {
        return false;
    }
    let Some(page) = root.pages.connect_model() else { return false };
    if page.elsewhere() {
        return false;
    }
    let Some(pending) = &page.request.pending else { return false };
    if pending.key != *key || pending.name.is_none() || pending.outcome.is_some() {
        return false;
    }
    let c::Ask::SignIn { provider, .. } = pending.ask else { return false };
    let Outcome::Answered(Answered::Jig(c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::AccountSignInStarted {
        source,
        address,
    })))) = outcome
    else {
        return false;
    };
    let path = match root.pages.address() {
        Address::Welcome { step: Step::Model { .. } } => Address::Welcome { step: Step::Ready },
        Address::ConnectModel { .. } => {
            Address::ConnectModel { way: Some(ModelWay { provider, kind: WayKind::SignIn, source: Some(*source) }) }
        }
        Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Adopt { .. } => return false,
    };
    let limits = crate::forge::wire_limits(&env.limits.forge).expect("checked wire bounds");
    let returning = w::Return::new(&limits, w::ReturnParts { source: *source, path: crate::address::print(&path) })
        .expect("printed return path fits");
    let requesting = w::Requesting::new(&limits, w::RequestingParts { request: w::Request::Return(returning) })
        .expect("return request fits");
    root.pages.serial = root.pages.serial.checked_add(1).expect("page keys fit");
    let key = Key { asker: Owner::Pages.asker(), number: root.pages.serial };
    root.returning = Some((key, address.clone()));
    crate::route::client(
        root,
        env,
        Event::Send {
            key,
            name: None,
            request: Ask::Family {
                scope: c::Scope::Own,
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::requesting(&requesting),
                },
            },
        },
        out,
    );
    false
}

fn return_answer(bytes: &[u8]) -> Option<w::Answer> {
    let mut reader = skein_lib::Reader::new(bytes);
    let answer = w::Answer::decode(&w::CEILINGS, &mut reader).ok()?;
    if reader.is_empty() { Some(answer) } else { None }
}
