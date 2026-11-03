//! Forge-making acts and kept first intent (temper/docs/design/protocol/forge.md, 2.2; temper/docs/design/protocol/setup.md, 6.7).
use super::{Domain, setup_login, setup_repository};
use crate::{
    Ask, Bot, Connection, First, FirstIntent, FirstState, ForgeAsk, ForgeRecord, ForgeState, Kind, Limits, Making,
    Mode, Offer, Request, SetUpAnswer, SetUpProject, SetUpRefusal,
};
use alloc::boxed::Box;
use jig_domain_core::Carried;
use skein_lib::{Duration, Env, Queue, Token};

pub(crate) fn bot() -> Bot {
    Bot {
        login: Box::from(b"temper".as_slice()),
        name: Box::from(b"temper".as_slice()),
        email: Box::from(b"temper@temper.invalid".as_slice()),
    }
}

pub(crate) fn restore_first(d: &mut Domain, record: &ForgeRecord) {
    d.first.remove(&record.number);
    if let Some(intent) = &record.first {
        d.first
            .insert(
                record.number,
                First {
                    playground: intent.playground,
                    org: intent.org.clone(),
                    repository: intent.repository.clone(),
                    project: intent.project.clone(),
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
    let (forge, org, repository, project, playground) = match ask {
        Ask::SetUp { forge, org, repository, project, playground } => (forge, org, repository, project, playground),
        Ask::Connect { .. }
        | Ask::Replace { .. }
        | Ask::RemoveForge { .. }
        | Ask::StartPlayground
        | Ask::EndPlayground
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
    let invalid_project = match &project {
        SetUpProject::New(name) => {
            name.is_empty() || name.len() > usize::try_from(env.limits.name_bytes).expect("name bound fits")
        }
        SetUpProject::Existing(_) => false,
    };
    if !setup_login(&org)
        || !setup_repository(&repository)
        || org.len() > usize::try_from(env.limits.name_bytes).expect("name bound fits")
        || repository.len() > usize::try_from(env.limits.name_bytes).expect("name bound fits")
        || invalid_project
        || (playground && (org.as_ref() != b"playground" || repository.as_ref() != b"widget"))
    {
        d.setup_answer(env, request, SetUpAnswer::Refused(SetUpRefusal::Invalid), out);
        return;
    }
    let intent = FirstIntent {
        playground,
        org: org.clone(),
        repository: repository.clone(),
        project,
        label: Box::from(b"temper-host".as_slice()),
    };
    let admitted = match forge {
        Making::New(ForgeAsk::Owned) => owned(d, intent.clone(), out),
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
    if let Some(record) = d.record(forge)
        && record.first.as_ref() != Some(&intent)
    {
        let mut record = record.clone();
        record.first = Some(intent.clone());
        crate::forges::save(d, record, out);
    }
    d.first
        .insert(
            forge,
            First {
                playground,
                org: org.clone(),
                repository: repository.clone(),
                project: intent.project,
                label: intent.label,
                state: FirstState::Making,
            },
        )
        .expect("configured forges bound first repositories");
    out.push(Request::MarkedForges);
    out.push(Request::SetUp { forge, request, org, repository, playground });
}

fn setup_busy(d: &Domain) -> bool {
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

fn owned(d: &mut Domain, intent: FirstIntent, out: &mut Queue<Request>) -> Result<u16, SetUpRefusal> {
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
    let Some(place) = (match &d.machine {
        Some(machine) => machine.owned.clone(),
        None => None,
    }) else {
        return Err(SetUpRefusal::NotOffered);
    };
    if setup_busy(d) {
        return Err(SetUpRefusal::Busy);
    }
    match crate::forges::make(d, Kind::Forgejo, Mode::Owned, place, None, false, bot(), true, Some(intent), out) {
        Some(forge) => Ok(forge),
        None => Err(SetUpRefusal::Busy),
    }
}
