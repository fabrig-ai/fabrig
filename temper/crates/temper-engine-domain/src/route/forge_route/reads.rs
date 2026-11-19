//! Forge read admission and refusals (temper/docs/design/domain/agent/agents.md).
//! Authority findings reach both the tool result and volatile operator facts.
use super::{
    CallAnswer, CallKey, Decision, Domain, Env, Limits, List, Queue, ReplyTo, Token, Work, authority, effect_access,
    forge, read_name,
};
use alloc::boxed::Box;
use skein_forge::api::{self, Answer, Error, Read, Repository};

type Refusal = (&'static str, &'static str, Box<CallAnswer>);

/// Admit one reviewer or agent read without losing the refusing layer.
pub(crate) fn read_call(
    domain: &mut Domain,
    env: &Env<Limits>,
    _decision: &mut Decision,
    to: ReplyTo,
    key: CallKey,
    repository: Repository,
    read: Read,
) {
    Domain::read_forge_call(domain, &env.limits, to, key, repository, read);
}

impl Domain {
    /// Admit and correlate a fresh forge read (domain/forge/keeping-up.md, 3).
    pub(crate) fn read_forge_call(
        domain: &mut Domain,
        limits: &Limits,
        to: ReplyTo,
        key: CallKey,
        repository: Repository,
        read: Read,
    ) {
        if let Err((layer, reason, answer)) = admit(domain, limits, key, repository, &read) {
            refused(domain, key, layer, reason, &answer);
            return super::super::host_route::read(domain, to, *answer);
        }
        let next = match domain.next_effect_owner.checked_add(1) {
            Some(serial) => match forge::root_read(serial) {
                Some(owner) => Some((serial, owner)),
                None => None,
            },
            None => None,
        };
        let Some((serial, owner)) = next else {
            let answer = CallAnswer::ForgeRead(Box::new(Err(Error::Busy)));
            refused(domain, key, "forge-route", "read correlation exhausted", &answer);
            return super::super::host_route::read(domain, to, answer);
        };
        domain.next_effect_owner = serial;
        let checks = if let Read::Checks { commit } = &read { Some(*commit) } else { None };
        assert!(
            domain.forge_reading.insert(owner, (to, key, repository, checks)) == Ok(None),
            "one fresh read correlation"
        );
        domain.work.push(Work::Forge(forge::Event::Client(skein_forge_client::Event::Read {
            owner,
            repository,
            read,
        })));
    }
}

fn local(reason: &'static str, error: Error) -> Refusal {
    ("forge-route", reason, Box::new(CallAnswer::ForgeRead(Box::new(Err(error)))))
}

fn admit(domain: &Domain, limits: &Limits, key: CallKey, repository: Repository, read: &Read) -> Result<(), Refusal> {
    let Some(adopted) = domain.forge.repository(repository) else {
        return Err(local("repository not adopted", Error::Missing));
    };
    let Some(context) = domain.core.tasks.delegation(key.task) else {
        return Err(local("task context missing", Error::Forbidden));
    };
    if context.project != adopted.project {
        return Err(local("repository belongs to another project", Error::Forbidden));
    }
    if !adopted.kinds.read {
        return Err(local("repository reads disabled", Error::Forbidden));
    }
    let Some(name) = read_name(adopted, read, limits.authority.segments) else {
        return Err(local("read name exceeds authority bounds", Error::Forbidden));
    };
    let mut findings = Queue::uncapped();
    let answer = domain.core.connector_read_admit(
        key.task,
        authority::Effect {
            connector: domain.config.forge_connector,
            kind: 1,
            name,
            state: [0; 32],
            price: None,
            access: effect_access(adopted, &forge::What::Repository, 1),
            additional: Box::new([]),
            guards: Box::new([]),
        },
        &mut findings,
    );
    if answer == authority::Answer::Allow {
        return Ok(());
    }
    let mut kept = List::with_capacity(findings.len());
    for _ in 0..findings.len() {
        let finding = findings.pop().expect("counted authority findings");
        kept.push(finding).expect("one slot per authority finding");
    }
    Err((
        "authority",
        "connector read denied",
        Box::new(CallAnswer::ToolDenied { answer, findings: kept.into_boxed() }),
    ))
}

fn refused(domain: &mut Domain, key: CallKey, layer: &'static str, reason: &'static str, answer: &CallAnswer) {
    let findings = if let CallAnswer::ToolDenied { findings, .. } = answer { findings.clone() } else { Box::new([]) };
    let error = if let CallAnswer::ForgeRead(result) = answer { result.as_ref().as_ref().err().copied() } else { None };
    super::super::diagnostics::read(domain, key, layer, reason, error, findings);
}

/// Only a change's current landing repair sees the receipt's candidate checks
/// (temper/docs/design/domain/forge/change/repairs.md, A landing's repair).
fn landing_failure(
    domain: &Domain,
    task: u64,
    repository: Repository,
    commit: api::Commit,
) -> Option<forge::LandingFailure> {
    let context = domain.core.tasks.delegation(task)?;
    let super::tasks::Party::Task(parent) = context.requester else { return None };
    let row = domain.forge.change(parent)?;
    let (child, super::forge_change::Delegate::Repair(why)) = row.delegate? else { return None };
    if row.terminal() || child != task || row.repository != repository {
        return None;
    }
    let failure = domain.forge.landing_failure(parent, why)?;
    if row.change.last_head != Some(commit) && failure.prefix != commit {
        return None;
    }
    Some(failure)
}

/// Busy reads wait in the host's bounded backoff (temper/docs/design/domain/forge/keeping-up.md, 3).
pub(super) fn read_answer(domain: &mut Domain, owner: Token, result: Result<Answer, Error>) {
    let Some((to, key, repository, checks)) = domain.forge_reading.remove(&owner) else { return };
    let mut result = result;
    // The forge client validates its answer's pins before this receipt-only overlay.
    if let Some(commit) = checks
        && let Ok(Answer::Checks(statuses)) = &mut result
        && let Some(failure) = landing_failure(domain, key.task, repository, commit)
    {
        forge::checks::append_gate_checks(statuses, failure, domain.limits.forge.client.rows);
    }
    if let Err(error) = &result {
        let reason = if *error == Error::Busy { "read admission busy" } else { "forge read failed" };
        let answer = CallAnswer::ForgeRead(Box::new(Err(*error)));
        refused(domain, key, "forge-client", reason, &answer);
    }
    if result == Err(Error::Busy) {
        let right = to.into_token();
        domain.call_renders.remove(&right);
        crate::stopping::call(domain, ReplyTo::new(right));
    } else {
        super::super::host_route::read(domain, to, CallAnswer::ForgeRead(Box::new(result)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_busy_branch_file_phase_discards_its_flight_without_rendering() {
        use crate::route::host_route::{Flight, FlightRender};
        let repository = Repository { forge: 1, repository: 1 };
        let render = FlightRender::ForgeRead(Box::new(Read::File {
            head: [7; 32],
            path: b"file".as_slice().into(),
            max_bytes: 2,
        }));
        let (mut domain, _) = crate::tests::resume_fixture::root();
        let right = Token::new(501);
        let owner = forge::root_read(1).expect("read owner");
        let key = CallKey { task: 1, attempt: 1, completion: 1, position: 1 };
        domain
            .call_renders
            .insert(
                right,
                Flight {
                    task: key.task,
                    attempt: key.attempt,
                    key: Some(key),
                    note_decoding: false,
                    render,
                    name: b"call".as_slice().into(),
                    tool: b"read_forge".as_slice().into(),
                    job: None,
                    page: None,
                },
            )
            .unwrap();
        domain.forge_reading.insert(owner, (ReplyTo::new(right), key, repository, None)).unwrap();
        read_answer(&mut domain, owner, Err(Error::Busy));
        assert!(domain.call_renders.is_empty(), "no partial context survives the host's full-call retry");
        assert!(domain.forge_reading.is_empty());
        let Some(Work::Fleet(jig_domain_fleet::Event::Relayed { to, answer })) = domain.work.pop() else {
            panic!("Busy returns to the host, without a render")
        };
        assert_eq!(to.into_token(), right);
        let Some(crate::boundary::Payload::Busy) = crate::assemble::take_payload(&mut domain, answer) else {
            panic!("the host receives Busy, not a partial answer")
        };
        assert!(domain.work.is_empty(), "no branch or file answer follows Busy");
    }
}
