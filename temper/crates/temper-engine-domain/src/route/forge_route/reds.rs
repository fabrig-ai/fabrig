//! A check's goal-local snapshot and report (temper/docs/design/domain/forge/reds.md, 5.1, 5.7 and 5.8).
use super::super::findings;
use super::{Domain, List, branch_what, forge, tasks};
use alloc::boxed::Box;
use skein_lib::{Decimal, Env, Queue};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RunRed {
    pub key: Box<[u8]>,
    pub head: Box<[u8]>,
}

/// Read only the reds applying to this live, fenced attempt.
#[must_use]
pub fn reds_for_run(domain: &Domain, task: u64, attempt: u64) -> Box<[RunRed]> {
    let mut rows = List::with_capacity(domain.limits.forge.branch_reds);
    if !domain.core.current_proof(task, attempt) {
        return rows.into_boxed();
    }
    if let Some(goal) = goal_for_run(domain, task) {
        for (_, red) in domain.forge.reds(goal) {
            rows.push(RunRed { key: red.key.clone(), head: red.head.clone() }).expect("bounded goal reds");
        }
    }
    rows.into_boxed()
}

fn goal_for_run(domain: &Domain, task: u64) -> Option<u64> {
    domain.core.tasks.task(task)?;
    if domain.forge.holds_goal_branch(task) {
        return Some(task);
    }
    let change = change_for_run(domain, task)?;
    goal_for_change(domain, change)
}

fn goal_for_change(domain: &Domain, change: &forge::ChangeRow) -> Option<u64> {
    let name = forge::Name {
        forge: change.repository.forge,
        repository: change.repository.repository,
        what: branch_what(&change.base, domain.limits.forge.name_bytes)?,
    };
    domain.forge.goal_branch_holder(&name)
}

fn change_for_run(domain: &Domain, task: u64) -> Option<&forge::ChangeRow> {
    let mut current = domain.core.tasks.task(task)?;
    for _ in 0..=current.depth {
        if let Some(change) = domain.forge.change(current.number) {
            return Some(change);
        }
        let tasks::Party::Task(parent) = current.requester else { break };
        current = domain.core.tasks.task(parent)?;
    }
    None
}

/// Report decisions belong to domain/forge/reds.md, 5.7 and 5.8.
pub(crate) fn check_results(
    domain: &mut Domain,
    env: &Env<crate::Limits>,
    decision: &mut crate::Decision,
    task: u64,
    attempt: u64,
    results: crate::wire::CheckResults,
) {
    if !super::super::current_proof(domain, task, attempt) {
        return;
    }
    let Some(run) = domain.core.tasks.task(task) else { return };
    match run.phase {
        tasks::Phase::Waiting
        | tasks::Phase::Active(_)
        | tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Active(_), .. } => {}
        tasks::Phase::Closing(_) | tasks::Phase::Ended(_) | tasks::Phase::Held { was: tasks::Was::Closing(_), .. } => {
            return;
        }
    }
    let project = run.project;
    let Some(repository) = domain.forge.repository_tag(project, results.tag) else { return };
    let mut out = Queue::uncapped();
    match results.verdict {
        crate::wire::Verdict::Passed => {
            if let Some(change) = change_for_run(domain, task)
                && change.repository == repository.provider
                && let Some(goal) = goal_for_change(domain, change)
            {
                domain.forge.observe_reds(change.task, goal, &results.passed, &results.failed, &mut out);
            }
        }
        crate::wire::Verdict::Refused => {}
    }
    super::outputs(domain, env, decision, &mut out);
    for flake in &results.flakes {
        let signature = forge::FindingSignature {
            repository: repository.provider,
            source: b"check".as_slice().into(),
            ids: Box::new([flake.key.clone()]),
            first: Box::default(),
        };
        let title = skein_lib::bytes::concat(&[b"Flake in a delivery's check: ", &flake.key]);
        let title = title.get(..character_start(&title, 200)).expect("title prefix").into();
        let body = flake_body(&repository, task, attempt, &results, flake);
        match domain.forge.prepare_check_flake(
            &env.limits.forge,
            env.wall,
            forge::Filer::Task(task),
            project,
            signature,
            title,
            body,
        ) {
            Ok(forge::CheckFlake::Filed(write)) => {
                drop(findings::commit(domain, env, decision, write, false));
            }
            Ok(forge::CheckFlake::Counted(write)) => {
                domain.forge.commit_finding(&env.limits.forge, write, &mut out);
                super::outputs(domain, env, decision, &mut out);
            }
            Err(_) => {}
        }
    }
}

fn flake_body(
    repository: &forge::Repository,
    task: u64,
    attempt: u64,
    results: &crate::wire::CheckResults,
    flake: &crate::wire::Flake,
) -> Box<[u8]> {
    let mut body = List::uncapped();
    append(&mut body, b"A test failed in a delivery's check and passed when its process ran again, with nothing else of the check running.\nRepository: ");
    if repository.owner.is_empty() || repository.name.is_empty() {
        for words in [
            b"forge ".as_slice(),
            Decimal::of(u64::from(repository.provider.forge)).as_bytes(),
            b", repository ",
            Decimal::of(u64::from(repository.provider.repository)).as_bytes(),
        ] {
            append(&mut body, words);
        }
    } else {
        append(&mut body, &repository.owner);
        append(&mut body, b"/");
        append(&mut body, &repository.name);
    }
    for words in [
        b"\nTest: ".as_slice(),
        &flake.key,
        b"\nSeen in task ",
        Decimal::of(task).as_bytes(),
        b"'s check ",
        Decimal::of(results.check).as_bytes(),
        b" (attempt ",
        Decimal::of(attempt).as_bytes(),
        b"): tree ",
    ] {
        append(&mut body, words);
    }
    append_hex(&mut body, &results.tree);
    append(&mut body, b", base ");
    match &results.base {
        Some(base) => append_hex(&mut body, base),
        None => append(&mut body, b"none"),
    }
    append(&mut body, b".\nIts log under load:\n");
    append(&mut body, &flake.log);
    findings::cut_words(body.as_slice(), 4096)
}

fn append(out: &mut List<u8>, words: &[u8]) {
    for &byte in words {
        out.push(byte).expect("bounded check words");
    }
}

fn append_hex(out: &mut List<u8>, bytes: &[u8]) {
    for &byte in bytes {
        append(out, &[skein_lib::bytes::hex_digit(byte >> 4), skein_lib::bytes::hex_digit(byte & 15)]);
    }
}

fn character_start(words: &[u8], limit: usize) -> usize {
    let mut end = words.len().min(limit);
    while end > 0
        && match words.get(end) {
            Some(byte) => byte & 0xc0 == 0x80,
            None => false,
        }
    {
        end = end.saturating_sub(1);
    }
    end
}
