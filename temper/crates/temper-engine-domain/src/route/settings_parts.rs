//! Settings parts held for one asking run (temper/docs/design/domain/agent/ask-temper.md, 2).
use crate::{CallAnswer, CallKey, Domain, KindValue, PartsAnswer, Tool, Work};
use alloc::boxed::Box;
use jig_domain_core::{self as core, settings as s};
use jig_domain_tasks as tasks;
use skein_lib::{List, ReplyTo};

/// One run's parts, typed as the decoder gave them (domain/agent/ask-temper.md, 2).
#[derive(Hash, Debug)]
pub(crate) struct Held {
    attempt: u64,
    against: u32,
    items: List<s::Item>,
    kinds: List<KindValue>,
}

fn asking(domain: &Domain, task: u64) -> bool {
    let Some(row) = domain.core.tasks.task(task) else { return false };
    match row.executor {
        tasks::Executor::Agent { charter } => match domain.core.charter(charter) {
            Some(charter) => crate::asking(charter.policy.tools),
            None => false,
        },
        tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } | tasks::Executor::Person(_) => false,
    }
}

/// Keep the restored asking runs that may have lost parts, once per run.
pub(crate) fn restored(domain: &mut Domain) {
    for (&task, proof) in &domain.core.proofs {
        if asking(domain, task) {
            domain.settings_restarted.insert((task, proof.attempt)).expect("one restart guard per live task");
        }
    }
}

/// Reclaim runs that ended before another run holds parts.
fn prune(domain: &mut Domain) {
    let mut ended = List::with_capacity(domain.settings_parts.len());
    for (&task, held) in &domain.settings_parts {
        if !super::current_proof(domain, task, held.attempt) {
            ended.push(task).expect("one ended entry per held run");
        }
    }
    for task in &ended {
        assert!(domain.settings_parts.remove(task).is_some(), "ended entry still held");
    }
    let mut ended = List::with_capacity(domain.settings_restarted.len());
    for &(task, attempt) in &domain.settings_restarted {
        if !super::current_proof(domain, task, attempt) {
            ended.push((task, attempt)).expect("one ended guard per run");
        }
    }
    for run in &ended {
        assert!(domain.settings_restarted.remove(run), "ended guard still held");
    }
}
fn answer(domain: &mut Domain, to: ReplyTo, parts: PartsAnswer) {
    super::host_route::read(domain, to, CallAnswer::Parts(Box::new(parts)));
}

/// Hold a part, or give the whole set to the proposal's path.
pub(crate) fn call(domain: &mut Domain, to: ReplyTo, key: CallKey, tool: Tool) {
    let Tool::ProposeSettings { against, items, reason, kinds, more } = tool else { unreachable!("whole proposal") };
    prune(domain);
    let asking = asking(domain, key.task);
    if more && !asking {
        answer(domain, to, PartsAnswer::NotOffered);
        return;
    }
    if !asking {
        propose(domain, to, key, Tool::ProposeSettings { against, items, reason, kinds, more }, false);
        return;
    }
    if domain.settings_restarted.remove(&(key.task, key.attempt)) && !more {
        answer(domain, to, PartsAnswer::Restarted);
        return;
    }
    let most = s::capacity(&crate::limits::core_limits(&domain.limits))
        .expect("settings capacity")
        .checked_mul(2)
        .expect("item union")
        .checked_add(1)
        .expect("one project policy");
    let count = u32::try_from(items.len()).expect("decoded items");
    let mut total = count;
    if let Some(held) = domain.settings_parts.get(&key.task) {
        for item in &items {
            for previous in &held.items {
                if item.setting == previous.setting {
                    answer(domain, to, PartsAnswer::Held { setting: item.setting });
                    return;
                }
            }
        }
        if !more || held.against == against {
            total = total.checked_add(held.items.len()).expect("bounded proposal union");
        }
    }
    if asking && total > most {
        answer(domain, to, PartsAnswer::PastBound { most, would: total });
        return;
    }
    let mut held = domain.settings_parts.remove(&key.task);
    let mut dropped = None;
    if let Some(previous) = &held
        && previous.against != against
    {
        if !more {
            let refusal =
                PartsAnswer::Revision { held: previous.items.len(), against: previous.against, named: against };
            answer(domain, to, refusal);
            return;
        }
        dropped = Some((previous.items.len(), previous.against));
        held = None;
    }
    let mut held = held.unwrap_or(Held {
        attempt: key.attempt,
        against,
        items: List::with_capacity(most),
        kinds: List::with_capacity(most),
    });
    for item in items {
        held.items.push(item).expect("checked proposal item bound");
    }
    for value in kinds {
        held.kinds.push(value).expect("at most one application value per item");
    }
    if more {
        let total = held.items.len();
        assert!(
            domain.settings_parts.insert(key.task, held).expect("one entry per live task").is_none(),
            "previous parts were consumed or dropped"
        );
        answer(domain, to, PartsAnswer::Holding { count, total, against, dropped });
        return;
    }
    let items = held.items.into_boxed();
    let kinds = held.kinds.into_boxed();
    propose(domain, to, key, Tool::ProposeSettings { against, items, reason, kinds, more: false }, asking);
}

fn propose(domain: &mut Domain, to: ReplyTo, key: CallKey, tool: Tool, asking: bool) {
    let Tool::ProposeSettings { against, items, reason, kinds, .. } = tool else { unreachable!("whole proposal") };
    if let Some(jig_domain_people::Refusal::Judged { why, .. }) = super::room_refusal(domain, &items) {
        super::host_route::rendered(domain, to, core::SettledAnswer::Host { error: true, body: why });
        return;
    }
    if asking
        && let Some(context) = domain.core.tasks.delegation(key.task)
        && let Some(why) = super::kinds::call_approval_refusal(domain, context.project, &kinds)
    {
        answer(domain, to, PartsAnswer::Approvals { why });
        return;
    }
    domain.kinds = kinds;
    domain.work.push(Work::Core(core::Event::NamedAction {
        to,
        key,
        action: core::NamedAction::ProposeSettings { against, items, reason },
    }));
}
