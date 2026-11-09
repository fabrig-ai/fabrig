//! A task's live decision for its person (temper/docs/design/domain/forge.md, 5.9).
use super::{CallAnswer, CallKey, Decision, Domain, Env, Limits, ReplyTo, Work, decide_call, forge};
use jig_domain_tasks as tasks;

pub(crate) fn call(
    domain: &mut Domain,
    env: &Env<Limits>,
    decision: &mut Decision,
    to: ReplyTo,
    key: CallKey,
    call: forge::DecidedCall,
) {
    if let Some(party) = person(domain, key.task)
        && let Some(context) = domain.core.tasks.delegation(key.task)
    {
        domain.work.push(Work::Forge(forge::Event::Decided {
            to,
            key: crate::translate::named_call(key),
            party,
            project: context.project,
            task: key.task,
            always: always(domain, key.task),
            call,
        }));
    } else {
        decide_call(domain, &env.limits, decision, to, key, CallAnswer::Decided { took: None, reach: None });
    }
}

fn person(domain: &Domain, task: u64) -> Option<u64> {
    let row = domain.core.tasks.task(task)?;
    match row.requester {
        tasks::Party::Person(person) => Some(person),
        tasks::Party::Deployment { .. } => None,
        tasks::Party::Task(_) => {
            let chat = domain.core.tasks.task(domain.core.tasks.served_chat(task)?)?;
            match chat.requester {
                tasks::Party::Person(person) => Some(person),
                tasks::Party::Task(_) | tasks::Party::Deployment { .. } => None,
            }
        }
    }
}

fn always(domain: &Domain, task: u64) -> bool {
    let Some(row) = domain.core.tasks.task(task) else { return false };
    let words = row.spec.words.as_ref();
    words == b"Asks: always"
        || words.starts_with(b"Asks: always\n")
        || words.ends_with(b"\nAsks: always")
        || skein_lib::bytes::find(words, b"\nAsks: always\n").is_some()
}
