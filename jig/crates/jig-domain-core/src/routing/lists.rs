//! A read of an account's list, into settings and the account's record (domain/engine/settings.md, 12).
use super::party::dispatch::Routed;
use crate::{Core, Limits, settings as s};
use jig_domain_accounts as accounts;
use jig_domain_link as link;
use skein_lib::{Env, Queue};

pub(crate) fn answered(core: &mut Core, env: &Env<Limits>, routed: &mut Routed, read: u64, outcome: link::ListOutcome) {
    let Some(account) = core.accounts.reading_list(read) else { return };
    let held = core.accounts.account(account).expect("a read's account is held");
    let provider = held.provider();
    let party = match held.holder() {
        accounts::Holder::Party(party) => Some(party),
        accounts::Holder::Deployment => None,
    };
    let before = names(core, account);
    let event = match outcome {
        link::ListOutcome::Failed(failure) => {
            accounts::Event::ListFailed { read, failure: failed(failure), at: env.wall }
        }
        link::ListOutcome::Listed(rows) => {
            let Some(provider_facts) = core.settings.llm_providers.get(provider) else { return };
            let Some(list) = &provider_facts.list else { return };
            let models = s::catalogue::entries(&core.current);
            let change = s::catalogue::from_list(
                provider,
                list,
                &rows,
                core.accounts.listed_names(provider, account),
                env.wall,
                &models,
                &core.settings.endpoints,
            );
            // The root's deployment header and the account's record share this commit.
            let room =
                s::writes_bound(&env.limits).expect("settings writes checked at configuration").saturating_sub(2);
            if change.items.is_empty()
                || super::settings::dispatch::decide(
                    core,
                    env,
                    routed,
                    s::Decider::List { provider },
                    None,
                    s::Change { against: core.current.newest(), items: change.items, running: s::Running::Stays },
                    room,
                )
                .is_ok()
            {
                accounts::Event::Listed { read, names: change.names, left_out: change.left_out, at: env.wall }
            } else {
                accounts::Event::ListFailed { read, failure: accounts::ReadFailure::Room, at: env.wall }
            }
        }
    };
    let mut out = Queue::uncapped();
    super::step::step_accounts(core, env, event, &mut out);
    let mut out = super::children::tag_account(core, env, &mut routed.work, out);
    for _ in 0..out.len() {
        routed.out.push(out.pop().expect("list record output count"));
    }
    core.touch_account_lines(party);
    if names(core, account) != before {
        core.touch_accounts(party);
    }
}

// A first good read replaces the stock fallback, including an empty list.
fn names(core: &Core, account: u32) -> Option<u64> {
    match core.accounts.list(account) {
        Some(record) if record.read.is_some() => Some(record.names),
        Some(_) | None => None,
    }
}

fn failed(failure: link::ListFailure) -> accounts::ReadFailure {
    match failure {
        link::ListFailure::Refused { status } => accounts::ReadFailure::Refused { status },
        link::ListFailure::Unreachable => accounts::ReadFailure::Unreachable,
        link::ListFailure::Unreadable => accounts::ReadFailure::Unreadable,
        link::ListFailure::Deadline => accounts::ReadFailure::Deadline,
        link::ListFailure::Agent => accounts::ReadFailure::Agent,
    }
}
