//! Admit a fenced attempt's pushes from its assigned workspace, and return
//! the answer after the record commits (temper/docs/design/domain/forge/writes.md, 2).
use super::{Decision, Delivery, Domain, Env, ForgeWorkspace, Limits, List, Queue, branch_what, emit, forge};
use crate::wire::{Answer, Asking};
use alloc::boxed::Box;
use skein_forge::api::{Commit, Repository};

#[derive(Hash, Debug)]
pub(crate) struct Claim {
    attempt: u64,
    destinations: Box<[Destination]>,
}

#[derive(Hash, Debug)]
struct Destination {
    tag: u64,
    repository: Repository,
    branch: Box<[u8]>,
    create_only: bool,
}

/// Keep only the destinations given to this attempt, not a second workspace.
/// Landing preparation names each prefix explicitly; a writable Base start
/// also lets checkout create that base if it is absent.
pub(crate) fn keep(domain: &mut Domain, task: u64, attempt: u64, workspace: &ForgeWorkspace, save: bool) {
    let mut destinations = List::uncapped();
    for item in &workspace.repositories {
        if item.prefixes.is_empty() && item.push.is_none() {
            continue;
        }
        if item.prefixes.is_empty() {
            let push = item.push.as_ref().expect("writable item");
            add(&mut destinations, item, push, false);
            if let crate::ForgeStart::Base(base) = &item.start {
                add(&mut destinations, item, base, true);
            }
        } else {
            for branch in &item.prefixes {
                add(&mut destinations, item, branch, false);
            }
        }
        if save && let Some(branch) = &item.save {
            add(&mut destinations, item, branch, false);
        }
    }
    if !destinations.is_empty() {
        domain
            .push_claims
            .insert(task, Claim { attempt, destinations: destinations.into_boxed() })
            .expect("one workspace per live task");
    }
}

fn add(destinations: &mut List<Destination>, item: &crate::ForgeRepository, branch: &[u8], create_only: bool) {
    destinations
        .push(Destination { tag: item.tag, repository: item.provider, branch: branch.into(), create_only })
        .expect("destinations bounded by the assigned items");
}

pub(crate) fn forget(domain: &mut Domain, task: u64, attempt: u64) {
    if let Some(claim) = domain.push_claims.get(&task)
        && claim.attempt == attempt
    {
        drop(domain.push_claims.remove(&task));
    }
}

#[expect(clippy::too_many_arguments, reason = "a fenced ask carries the worker's lease and proposed commit")]
pub(crate) fn ask(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    asking: Asking,
    tag: u64,
    branch: &[u8],
    expected: Option<Commit>,
    commit: Commit,
) {
    let task = asking.run.raw();
    let attempt = asking.attempt.raw();
    let answer = if let Some(name) = named(domain, &env.limits, task, attempt, tag, branch, expected.is_none()) {
        let mut out = Queue::uncapped();
        let answer = match domain.forge.admit_push(
            &env.with(env.limits.forge),
            &name,
            expected,
            commit,
            forge::Sender::Attempt { task, attempt },
            &mut out,
        ) {
            Ok(()) => Answer::Admitted,
            Err(words) => Answer::Unadmitted { words },
        };
        super::outputs(domain, env, decision, &mut out);
        answer
    } else {
        let words = skein_lib::bytes::concat(&[b"this attempt may not push to ", branch]);
        let words = forge::text::fit(&words, skein_lib::usize_of(forge::EFFECT_ACCOUNT_BYTES), b"...");
        Answer::Unadmitted { words }
    };
    emit(decision, &env.limits, Delivery::Host(Box::new(crate::HostDelivery::Asked { asking, answer })));
}

fn named(
    domain: &Domain,
    limits: &Limits,
    task: u64,
    attempt: u64,
    tag: u64,
    branch: &[u8],
    create: bool,
) -> Option<forge::Name> {
    if let Some(claim) = domain.push_claims.get(&task) {
        if claim.attempt == attempt {
            for destination in &claim.destinations {
                if destination.tag == tag
                    && destination.branch.as_ref() == branch
                    && (!destination.create_only || create)
                {
                    let Repository { forge, repository } = destination.repository;
                    return Some(forge::Name {
                        forge,
                        repository,
                        what: branch_what(branch, limits.forge.name_bytes)?,
                    });
                }
            }
        }
        return None;
    }
    // A reattached attempt is not assigned again. Its durable connector hold
    // is evidence for this exact destination, not for any other branch in the repository.
    let (forge, repository) = forge::untag(tag)?;
    let name = forge::Name { forge, repository, what: branch_what(branch, limits.forge.name_bytes)? };
    let hold = domain.forge.hold(&name)?;
    if hold.writer == Some(forge::Writer::Run { task, attempt }) {
        return Some(name);
    }
    for writer in &hold.design_writers {
        if writer.task == task && writer.attempt == attempt && !writer.rereading {
            return Some(name);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, Record, Request, StoreInput, StoreRequest, WorkerInput, WorkerRequest, Write};
    use skein_lib::Token;

    #[test]
    fn a_push_is_admitted_only_after_its_record_commits() {
        let (mut domain, env) = crate::tests::resume_fixture::root();
        let branch = b"temper/7";
        let provider = Repository { forge: 1, repository: 2 };
        let asking = Asking { channel: Token::new(7), run: Token::new(7), attempt: Token::new(1), number: 1 };
        let tag = 3;
        let workspace = ForgeWorkspace {
            repositories: Box::new([crate::ForgeRepository {
                prefixes: Box::new([]),
                delivers: false,
                save: None,
                tag,
                provider,
                name: b"repo".as_slice().into(),
                host: b"forge.example".as_slice().into(),
                owner: b"org".as_slice().into(),
                start: crate::ForgeStart::Base(b"main".as_slice().into()),
                push: Some(branch.as_slice().into()),
                landing: None,
                delivered: None,
                pin: None,
                identity: 1,
            }]),
        };
        keep(&mut domain, 7, 1, &workspace, false);
        let expected = Some([6; 32]);
        let commit = [7; 32];
        crate::step(
            &mut domain,
            &env,
            Event::Worker(WorkerInput::Admit { asking, tag, branch: branch.as_slice().into(), expected, commit }),
        );
        let mut out = Queue::uncapped();
        crate::release(&mut domain, &env, &mut out);
        let Some(Request::Store(StoreRequest::Commit { number, writes })) = out.pop() else {
            panic!("save before answer")
        };
        let name = forge::Name {
            forge: provider.forge,
            repository: provider.repository,
            what: branch_what(branch, env.limits.forge.name_bytes).unwrap(),
        };
        let mut saved = false;
        for write in &writes {
            if let Write::Save(Record::Forge { row }) = write
                && let forge::Stored::Writes(row) = row.as_ref()
            {
                assert_eq!(row.name, name);
                assert_eq!(
                    row.entries.as_ref(),
                    &[forge::Written {
                        order: 1,
                        sender: forge::Sender::Attempt { task: 7, attempt: 1 },
                        state: forge::WriteState::Admitted,
                        operation: forge::writes::Operation::Push { before: expected, after: commit },
                    }]
                );
                saved = true;
            }
        }
        assert!(saved, "the commit records this attempt's lease");
        crate::release(&mut domain, &env, &mut out);
        assert!(out.is_empty(), "the push must wait for the store's commit");
        crate::step(&mut domain, &env, Event::Store(StoreInput::Committed { number, fill: 0 }));
        crate::release(&mut domain, &env, &mut out);
        crate::release(&mut domain, &env, &mut out);
        assert_eq!(
            out.pop(),
            Some(Request::Worker(WorkerRequest::Deliver(Delivery::Host(Box::new(crate::HostDelivery::Asked {
                asking,
                answer: Answer::Admitted
            },)))))
        );
    }
}
