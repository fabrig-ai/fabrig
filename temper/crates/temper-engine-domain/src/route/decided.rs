//! A task's live decision for its person (temper/docs/design/domain/forge/attention.md, 5).
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

/// HOW is render-only: the caller's current route, not part of the saved judgement
/// (temper/docs/design/domain/agent/tools.md, HOW names the one way its caller can ask).
pub(crate) fn how(domain: &Domain, task: u64) -> super::AskHow {
    let Some(row) = domain.core.tasks.task(task) else { return super::AskHow::None };
    match row.requester {
        tasks::Party::Task(_) => {
            if let Some(context) = domain.core.tasks.delegation(task)
                && context.authority.tools.0 & domain.core.settings.tools.message.0 != 0
            {
                return super::AskHow::Message;
            }
        }
        tasks::Party::Person(person) => {
            if let tasks::Executor::Agent { charter } = row.executor
                && let Some(charter) = domain.core.charter(charter)
                && charter.policy.tools.0 & domain.core.settings.tools.delegate.0 != 0
            {
                return super::AskHow::Person(person);
            }
        }
        tasks::Party::Deployment { .. } => {}
    }
    super::AskHow::None
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
