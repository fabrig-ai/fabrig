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

fn append_gate_checks(statuses: &mut Box<[api::Status]>, failure: forge::LandingFailure, rows: u32) {
    if failure.jobs.is_empty() {
        return;
    }
    let bound = usize::try_from(rows).expect("row bound fits usize");
    let gate_rows = failure.jobs.len().min(bound);
    let forge_rows = statuses.len().min(bound.saturating_sub(gate_rows));
    let cut = forge_rows < statuses.len() || gate_rows < failure.jobs.len();
    let mut combined = List::with_capacity(rows);
    for status in statuses.iter().take(forge_rows) {
        combined.push(status.clone()).expect("forge rows leave room for gate jobs");
    }
    for job in failure.jobs.iter().take(gate_rows) {
        let mut context = List::uncapped();
        for byte in b"temper/gate/".iter().chain(job.name.iter()) {
            context.push(*byte).expect("context is internal receipt text");
        }
        combined
            .push(api::Status {
                at: skein_lib::Time::ZERO,
                context: context.into_boxed(),
                check: api::Check::Failed,
                description: if cut {
                    Box::from(b"Row bound: gate jobs take precedence; remaining checks omitted.".as_slice())
                } else {
                    Box::new([])
                },
                url: failure.run.url.clone(),
                job: Some(api::JobAttempt {
                    head: failure.prefix,
                    run: failure.run.id,
                    job: job.job,
                    attempt: job.attempt,
                }),
            })
            .expect("gate jobs have reserved rows");
    }
    *statuses = combined.into_boxed();
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
        append_gate_checks(statuses, failure, domain.limits.forge.client.rows);
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
    fn gate_jobs_reserve_rows_and_report_displaced_forge_checks() {
        let failure = forge::LandingFailure {
            gate: b"gate".as_slice().into(),
            run: forge::ReceiptRun { id: 11, number: 3, url: b"https://forge.example/run/3".as_slice().into() },
            prefix: [7; 32],
            base: None,
            jobs: Box::new([forge::ReceiptJob { job: 17, attempt: 2, name: b"test".as_slice().into() }]),
        };
        let first = api::Status {
            at: skein_lib::Time::ZERO,
            context: b"build".as_slice().into(),
            check: api::Check::Passed,
            description: Box::new([]),
            url: Box::new([]),
            job: None,
        };
        let second = api::Status { context: b"fmt".as_slice().into(), ..first.clone() };
        for (rows, kept) in [(3, 2), (2, 1), (1, 0)] {
            let mut statuses: Box<[api::Status]> = Box::new([first.clone(), second.clone()]);
            append_gate_checks(&mut statuses, failure.clone(), rows);
            assert_eq!(statuses.len(), kept + 1);
            let (gate, own) = statuses.split_last().expect("gate job");
            assert_eq!(own, &([first.clone(), second.clone()])[..kept]);
            assert_eq!(gate.context.as_ref(), b"temper/gate/test");
            assert_eq!(gate.check, api::Check::Failed);
            assert_eq!(gate.url, failure.run.url);
            assert_eq!(gate.job, Some(api::JobAttempt { head: failure.prefix, run: 11, job: 17, attempt: 2 }));
            assert_eq!(gate.description.is_empty(), kept == 2);
            if kept < 2 {
                assert_eq!(
                    gate.description.as_ref(),
                    b"Row bound: gate jobs take precedence; remaining checks omitted."
                );
            }
        }
    }
}
