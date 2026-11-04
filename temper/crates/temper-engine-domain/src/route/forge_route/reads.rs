//! Forge read admission and refusals (temper/docs/design/domain/agent.md, 8).
//! Authority findings reach both the tool result and volatile operator facts.
use super::{
    CallAnswer, CallKey, Decision, Domain, Env, Limits, List, Queue, ReplyTo, Token, Work, authority, effect_access,
    forge, read_name,
};
use alloc::boxed::Box;
use skein_forge::api::{Answer, Error, Read, Repository};

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
    if let Err((layer, reason, answer)) = admit(domain, env, key, repository, &read) {
        refused(domain, key, layer, reason, &answer);
        return super::super::host_route::read(domain, to, *answer);
    }
    let serial = match domain.next_effect_owner.checked_add(1) {
        Some(serial) if serial < 1_u64 << 61_u32 => Some(serial),
        Some(_) | None => None,
    };
    let Some(serial) = serial else {
        let answer = CallAnswer::ForgeRead(Box::new(Err(Error::Busy)));
        refused(domain, key, "forge-route", "read correlation exhausted", &answer);
        return super::super::host_route::read(domain, to, answer);
    };
    domain.next_effect_owner = serial;
    let owner = Token::new(serial | (1_u64 << 61_u32));
    assert!(domain.forge_reading.insert(owner, (to, key)) == Ok(None), "one fresh read correlation");
    domain.work.push(Work::Forge(forge::Event::Client(skein_forge_client::Event::Read { owner, repository, read })));
}

fn local(reason: &'static str, error: Error) -> Refusal {
    ("forge-route", reason, Box::new(CallAnswer::ForgeRead(Box::new(Err(error)))))
}

fn admit(domain: &Domain, env: &Env<Limits>, key: CallKey, repository: Repository, read: &Read) -> Result<(), Refusal> {
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
    let Some(name) = read_name(adopted, read, env.limits.authority.segments) else {
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

/// Busy reads wait in the host's bounded backoff (temper/docs/design/domain/forge.md, 5.3).
pub(super) fn read_answer(domain: &mut Domain, owner: Token, result: Result<Answer, Error>) {
    let Some((to, key)) = domain.forge_reading.remove(&owner) else { return };
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
