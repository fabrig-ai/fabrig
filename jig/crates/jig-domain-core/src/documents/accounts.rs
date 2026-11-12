//! The party's accounts (`client/domain/contract/documents.md`, 1; `domain/accounts.md`, sections 3 and 5).
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).

pub use crate::below::documents::accounts::*;
use crate::{
    Core, Limits,
    documents::{Missing, Text, Window, deployment::account_state, inbox_deployment::line_bound},
};
use alloc::boxed::Box;
use jig_domain_accounts as accounts;
use skein_lib::List;

/// An account's facts as this reader may see them (client/domain/contract/documents.md, 1).
pub(super) fn fact_lines(core: &Core, account: u32, holder_reads: bool) -> List<accounts::AccountFact> {
    let mut facts = List::with_capacity(accounts::FACTS);
    for fact in core.accounts.account_facts(account) {
        if fact.shown_to(holder_reads) {
            facts.push(fact.clone()).expect("the child bounds account facts");
        }
    }
    facts
}

pub(super) fn source_lines(core: &Core, limits: &Limits, account: &accounts::Account) -> List<SourceLine> {
    let mut sources = List::with_capacity(limits.documents.lines);
    for (place, number) in account.sources().iter().take(line_bound(limits)) {
        let fact = core.accounts.source(*number).expect("an account's sources are kept");
        let spent_until = match fact.spent_until {
            Some(until) => Some(until.wall),
            None => None,
        };
        sources
            .push(SourceLine {
                name: match accounts::programs::name(&core.accounts, *number, account.holder()) {
                    Some(name) => Some(skein_lib::bytes::copy_of(name)),
                    None => None,
                },
                place: *place,
                number: *number,
                kind: fact.kind,
                state: account_state(fact.state),
                usable: source_usable(core, &fact),
                expiring: fact.expiring,
                attention: fact.attention,
                spent_until,
            })
            .expect("bounded source lines");
    }
    sources
}

pub(super) fn make(core: &Core, limits: &Limits, party: u64, owner: u64, window: &Window) -> Result<Accounts, Missing> {
    core.people.identity(party).ok_or(Missing::Unknown)?;
    if super::document_bound(&limits.documents).is_none() || window.rows > limits.documents.rows {
        return Err(Missing::TooLarge);
    }
    if owner != party {
        return Err(Missing::NotAllowed);
    }
    let mut accounts = List::with_capacity(limits.documents.lines);
    let mut cut = false;
    let views = core.accounts.account_views();
    let mut after = None;
    for _ in 0..views.len() {
        let Some((place, number)) = next_account(core, &views, accounts::Holder::Party(party), after) else { break };
        after = Some((place, number));
        let account = core.accounts.account(number).expect("a selected account");
        if accounts.room() == 0 {
            cut = true;
            continue;
        }
        let sources = source_lines(core, limits, account);
        accounts
            .push(AccountLine {
                list: list(core, number),
                number,
                place: account.place(),
                uses: uses(core, account),
                plan: account.plan(),
                provider: Some(account.provider()),
                provider_name: provider_name(core, account, limits.documents.text_bytes),
                manage_usage: manage_usage(core, account),
                display: super::small::text(account.display(), limits.documents.text_bytes),
                holder: account.holder(),
                cut: sources.len() < account.sources().len(),
                sources,
                facts: fact_lines(core, number, true),
            })
            .expect("bounded account lines");
    }
    let mut signing = List::with_capacity(limits.documents.lines);
    let mut settled = true;
    for pending in &core.accounts.pending() {
        if pending.by != party {
            continue;
        }
        settled = false;
        if signing.room() == 0 {
            cut = true;
            continue;
        }
        signing
            .push(SignInLine { source: pending.source, provider: pending.provider, holder: pending.holder })
            .expect("bounded sign-in lines");
    }
    let last_end = match core.accounts.last_end(party) {
        Some(ended) => Some(LastEnd { source: ended.source, end: ended.end }),
        None => None,
    };
    let (readiness, readiness_cut) = super::catalogue::rows(core, limits, party)?;
    let (program_hosts, hosts_cut) = super::programs::hosts(core, limits, accounts::Holder::Party(party));
    Ok(Accounts {
        program_hosts,
        accounts,
        signing,
        last_end,
        usable: serves_work(core, party),
        settled,
        cut: cut || hosts_cut,
        readiness,
        readiness_cut,
    })
}

/// A usable account serves its explicit party payers, or its default holder/deployment uses.
fn serves_work(core: &Core, party: u64) -> bool {
    for fact in &core.accounts.account_views() {
        let account = core.accounts.account(fact.account).expect("an account view names its account");
        if !account_usable(core, account) {
            continue;
        }
        if crate::runs::coverage::serves(&core.people, account, accounts::Use::Party(party), None) {
            return true;
        }
    }
    false
}

/// Program sources need a current worker login allowed by the host rule (accounts.md, 3.4).
fn source_usable(core: &Core, fact: &accounts::Fact) -> bool {
    if !fact.usable {
        return false;
    }
    match fact.kind {
        accounts::Kind::SignIn | accounts::Kind::Key | accounts::Kind::None => return fact.usable,
        accounts::Kind::Program => {}
    }
    let mut facts = List::with_capacity(core.host_fact_capacity());
    for host in &core.fleet.host_views() {
        if !host.connected
            || host.kind != jig_domain_fleet::HostKind::Worker
            || core.fleet.awaiting_logins(host.channel)
        {
            continue;
        }
        let placement = jig_domain_fleet::PlacementConstraint { channel: host.channel, kind: host.kind };
        core.host_facts(placement, core.host_rule(placement), &mut facts);
        for login in &facts {
            if login.source == fact.source && login.usable {
                return true;
            }
        }
    }
    false
}

/// Walk the holder's source order before document truncation (accounts.md, 3.4).
pub(super) fn account_usable(core: &Core, account: &accounts::Account) -> bool {
    for (_, number) in account.sources() {
        let fact = core.accounts.source(*number).expect("an account's sources are kept");
        if source_usable(core, &fact) {
            return true;
        }
        match fact.kind {
            accounts::Kind::Program => {}
            accounts::Kind::SignIn | accounts::Kind::Key | accounts::Kind::None => return false,
        }
    }
    false
}

/// Copy an account's explicit uses without changing the default/empty distinction.
pub(super) fn uses(core: &Core, account: &accounts::Account) -> Option<List<accounts::Use>> {
    let uses = account.uses()?;
    let mut result = List::with_capacity(accounts::USES);
    for payer in uses {
        match payer {
            accounts::Use::Project(project)
                if *project != jig_domain_people::DEPLOYMENT && !core.open_project(*project) =>
            {
                continue;
            }
            accounts::Use::Project(_) | accounts::Use::Party(_) => {}
        }
        result.push(*payer).expect("the child bounds uses");
    }
    Some(result)
}

/// The next holder account by place, with its number breaking ties deterministically.
pub(super) fn next_account(
    core: &Core,
    views: &List<accounts::Fact>,
    holder: accounts::Holder,
    after: Option<(u32, u32)>,
) -> Option<(u32, u32)> {
    let mut next = None;
    for fact in views {
        let account = core.accounts.account(fact.account).expect("an account view names its account");
        let position = (account.place(), fact.account);
        if account.holder() != holder
            || match after {
                Some(old) => position <= old,
                None => false,
            }
        {
            continue;
        }
        if match next {
            Some(old) => position < old,
            None => true,
        } {
            next = Some(position);
        }
    }
    next
}

/// Public configuration is independent of the holder's private account document.
pub(super) fn provider_name(core: &Core, account: &accounts::Account, bytes: u32) -> Option<Text> {
    let provider = core.settings.llm_providers.get(account.provider())?;
    Some(super::small::text(&provider.name, bytes))
}
pub(super) fn manage_usage(core: &Core, account: &accounts::Account) -> Option<Box<[u8]>> {
    core.settings.llm_providers.get(account.provider())?.manage_usage.clone()
}

/// Public list history (jig/docs/design/client/domain/contract/documents.md, 1).
/// Capability is separate from a first attempt's history (domain/accounts.md, 3.7).
pub(super) fn list(core: &Core, account: u32) -> Option<AccountList> {
    if !core.accounts.has_list(account) {
        return None;
    }
    Some(match core.accounts.list(account) {
        Some(record) => {
            AccountList { read: record.read, tried: record.tried, failure: record.failure, left_out: record.left_out }
        }
        None => AccountList { read: None, tried: None, failure: None, left_out: 0 },
    })
}

/// The largest omission among account rows this party may read, before document cuts.
pub(super) fn left_out(core: &Core, party: u64) -> u32 {
    let deployment = core.people.role(party, jig_domain_people::DEPLOYMENT).is_some();
    let mut largest = 0;
    for fact in &core.accounts.account_views() {
        let account = core.accounts.account(fact.account).expect("an account view names its account");
        if (account.holder() == accounts::Holder::Party(party)
            || (deployment && account.holder() == accounts::Holder::Deployment))
            && let Some(list) = list(core, fact.account)
        {
            largest = largest.max(list.left_out);
        }
    }
    largest
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_accounts_document_is_priced() {
        let limits = crate::tests::limits();
        let accounts = crate::documents::accounts::document_bound(&limits.documents).expect("accounts bound");
        assert!(accounts <= crate::documents::document_bound(&limits.documents).expect("document bound"));
    }
}
