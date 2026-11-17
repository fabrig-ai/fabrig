//! Forge-making acts and kept first intent (temper/docs/design/protocol/forge/engine.md, 2; temper/docs/design/protocol/setup/owned.md, 7).
use super::{Domain, setup_login, setup_repository};
use crate::{
    Ask, Bot, Connection, EmptyIntent, First, FirstIntent, FirstState, ForgeAsk, ForgeRecord, ForgeState, Kind, Limits,
    Making, Mode, Offer, Request, SetUpAnswer, SetUpFirst, SetUpRefusal,
};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::Carried;
use skein_lib::{Duration, Env, Queue, Token};
pub use temper_engine_domain_forge_vocabulary::make::*;

#[derive(Hash, Debug)]
pub(crate) enum Choosing {
    Owned { intent: FirstIntent, first: SetUpFirst },
    Fake,
}

pub(crate) fn choose(
    d: &mut Domain,
    request: Token,
    kind: crate::BoxKind,
    choosing: Choosing,
    out: &mut Queue<Request>,
) {
    let login = match d.setup_people.remove(&request) {
        Some(login) => login,
        None => Box::from(b"owner".as_slice()),
    };
    d.choosing.insert(request, choosing).expect("one chosen place per admitted act");
    out.push(Request::ChoosePlace { request, kind, login });
}

pub(crate) fn chosen(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    place: Option<crate::Place>,
    git_program: Option<Box<[u8]>>,
    out: &mut Queue<Request>,
) {
    let Some(choosing) = d.choosing.remove(&request) else { return };
    match choosing {
        Choosing::Owned { intent, first } => {
            let Some(place) = place else {
                d.setup_answer(env, request, SetUpAnswer::Refused(SetUpRefusal::Busy), out);
                return;
            };
            match owned(d, intent.clone(), place, out) {
                Ok(forge) => finish(d, forge, request, intent, first, out),
                Err(why) => d.setup_answer(env, request, SetUpAnswer::Refused(why), out),
            }
        }
        Choosing::Fake => crate::playground::chosen(d, request, place, git_program, out),
    }
}

pub(crate) fn bot() -> Bot {
    Bot { login: Box::from(BOT_LOGIN), name: Box::from(b"temper".as_slice()), email: Box::from(BOT_EMAIL) }
}

pub(crate) fn restore_first(d: &mut Domain, record: &ForgeRecord) {
    set_first(d, record.number, record.first.as_ref());
}
fn set_first(d: &mut Domain, forge: u16, first: Option<&FirstIntent>) {
    d.first.remove(&forge);
    if let Some(FirstIntent::Empty(intent)) = first {
        d.first
            .insert(
                forge,
                First {
                    playground: intent.playground,
                    org: intent.org.clone(),
                    repository: intent.repository.clone(),
                    label: intent.label.clone(),
                    state: FirstState::Making,
                },
            )
            .expect("kept forges bound first repositories");
    }
}

#[expect(clippy::too_many_arguments, reason = "the setup act and its caller travel together")]
pub(crate) fn setup(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    token: Token,
    retention: Duration,
    ask: Ask,
    carried: Option<Carried>,
    out: &mut Queue<Request>,
) {
    let (forge, first) = match ask {
        Ask::SetUp { forge, first } => (forge, first),
        Ask::Connect { .. }
        | Ask::Replace { .. }
        | Ask::RemoveForge { .. }
        | Ask::StartPlayground
        | Ask::EndPlayground
        | Ask::DismissMaking { .. }
        | Ask::MakeRepository { .. }
        | Ask::SetDraft(_)
        | Ask::Adopt(_)
        | Ask::Install { .. }
        | Ask::Accept { .. }
        | Ask::Reachable { .. } => unreachable!("setup routing"),
    };
    if let Some(value) = carried {
        out.push(Request::CarriedUnused { number: value.number });
    }
    let mut expired = None;
    for (token, (until, _)) in &d.setup_answers {
        if *until <= env.wall {
            expired = Some(*token);
            break;
        }
    }
    if let Some(token) = expired {
        d.setup_answers.remove(&token);
    }
    if d.setup_pending.len().saturating_add(d.setup_answers.len()) >= env.limits.applications {
        out.push(Request::ApplicationAgain { request, why: jig_domain_people::Again::Busy });
        return;
    }
    d.setup_pending.insert(request, (token, retention)).expect("setup answer room checked");
    let intent = match intent(&first, env.limits.name_bytes) {
        Ok(intent) => intent,
        Err(why) => {
            d.setup_answer(env, request, SetUpAnswer::Refused(why), out);
            return;
        }
    };
    let admitted = match forge {
        Making::New(ForgeAsk::Owned) => match offered(d) {
            Ok(()) => {
                choose(d, request, crate::BoxKind::Owned, Choosing::Owned { intent, first }, out);
                return;
            }
            Err(why) => Err(why),
        },
        Making::New(ForgeAsk::GitHub | ForgeAsk::Managed { .. } | ForgeAsk::Connected { .. }) => {
            Err(SetUpRefusal::NotOffered)
        }
        Making::Kept(forge) => match d.offer(forge) {
            Offer::NotOffered => Err(SetUpRefusal::NotOffered),
            Offer::NotUnderTheBox => Err(SetUpRefusal::NotUnderTheBox),
            Offer::Offered => {
                if setup_busy(d) {
                    Err(SetUpRefusal::Busy)
                } else {
                    Ok(forge)
                }
            }
        },
    };
    let forge = match admitted {
        Ok(forge) => forge,
        Err(why) => {
            d.setup_answer(env, request, SetUpAnswer::Refused(why), out);
            return;
        }
    };
    finish(d, forge, request, intent, first, out);
}

fn finish(
    d: &mut Domain,
    forge: u16,
    request: Token,
    intent: FirstIntent,
    first: SetUpFirst,
    out: &mut Queue<Request>,
) {
    if let Some(record) = d.record(forge)
        && record.first.as_ref() != Some(&intent)
    {
        let mut record = record.clone();
        record.first = Some(intent.clone());
        crate::forges::save(d, record, out);
    }
    set_first(d, forge, Some(&intent));
    out.push(Request::MarkedForges);
    out.push(Request::SetUp { forge, request, first });
}

fn intent(first: &SetUpFirst, name_bytes: u32) -> Result<FirstIntent, SetUpRefusal> {
    match first {
        SetUpFirst::Empty { org, repository, playground } => {
            if !setup_login(org)
                || !setup_repository(repository)
                || org.len() > skein_lib::usize_of(name_bytes)
                || repository.len() > skein_lib::usize_of(name_bytes)
                || (*playground && (org.as_ref() != b"playground" || repository.as_ref() != b"widget"))
            {
                return Err(SetUpRefusal::Invalid);
            }
            Ok(FirstIntent::Empty(EmptyIntent {
                playground: *playground,
                org: org.clone(),
                repository: repository.clone(),
                label: Box::from(b"temper-host".as_slice()),
            }))
        }
        SetUpFirst::Copies { addresses } => {
            if addresses.len() > skein_lib::usize_of(temper_client_protocol_wire::FIRST_COPIES) {
                return Err(SetUpRefusal::Invalid);
            }
            for address in addresses {
                if address.len() > temper_client_protocol_wire::source::ADDRESS_BYTES
                    || temper_client_protocol_wire::source::parse(address).is_err()
                {
                    return Err(SetUpRefusal::Invalid);
                }
            }
            Ok(FirstIntent::Copies { addresses: addresses.clone() })
        }
        SetUpFirst::Nothing => Ok(FirstIntent::Nothing),
    }
}

fn setup_busy(d: &Domain) -> bool {
    if !d.choosing.is_empty() {
        return true;
    }
    for (_, (state, _)) in &d.connections {
        match state {
            Connection::SettingUp { .. } => return true,
            Connection::NotConnected
            | Connection::NotSetUp
            | Connection::SetUpFailed { .. }
            | Connection::Connecting
            | Connection::Connected { .. }
            | Connection::Stopped { .. } => {}
        }
    }
    false
}

fn offered(d: &Domain) -> Result<(), SetUpRefusal> {
    if !d.under_box {
        return Err(SetUpRefusal::NotUnderTheBox);
    }
    for (number, record) in &d.records {
        if record.mode == Mode::Owned && record.state == ForgeState::Kept {
            return match d.connection(*number) {
                Connection::SettingUp { .. } => Err(SetUpRefusal::Busy),
                Connection::NotConnected
                | Connection::NotSetUp
                | Connection::SetUpFailed { .. }
                | Connection::Connecting
                | Connection::Connected { .. }
                | Connection::Stopped { .. } => Err(SetUpRefusal::NotOffered),
            };
        }
    }
    if setup_busy(d) || !crate::forges::room(d) {
        return Err(SetUpRefusal::Busy);
    }
    Ok(())
}
fn owned(
    d: &mut Domain,
    intent: FirstIntent,
    place: crate::Place,
    out: &mut Queue<Request>,
) -> Result<u16, SetUpRefusal> {
    match crate::forges::make(d, Kind::Forgejo, Mode::Owned, place, None, false, bot(), true, Some(intent), out) {
        Some(forge) => Ok(forge),
        None => Err(SetUpRefusal::Busy),
    }
}
