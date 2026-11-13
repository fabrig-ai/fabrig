//! Offers and logins are memory only (domain/terminal-login.md, 2–4).
use crate::{Answer, Domain, Holder, Limits, Refusal, Request, TERMINALS};
use alloc::boxed::Box;
use skein_lib::{Env, List, Map, Queue, Token};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TerminalLogin {
    pub number: u32,
    pub provider: u32,
    pub name: Box<[u8]>,
    pub cancelled: bool,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LoginExit {
    Exited { code: u8 },
    Killed { signal: u8 },
    NotStarted,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LoginEnd {
    SignedIn { source: u32 },
    NotFinished { exit: Option<LoginExit> },
    Cancelled,
    Refused { refusal: Refusal },
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct LentEnd {
    pub number: u32,
    pub end: LoginEnd,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct Offer {
    watcher: Token,
    terminal: u64,
}
#[derive(Hash, Debug)]
struct UnderWay {
    login: TerminalLogin,
    offer: Offer,
}
/// The parent admits watches and parties; one list per offering party and one end per party.
#[derive(Hash, Debug)]
pub(crate) struct Lending {
    offers: Map<u64, List<Offer>>,
    logins: Map<u64, UnderWay>,
    ends: Map<u64, LentEnd>,
    next: u32,
}
impl Lending {
    pub(crate) fn new() -> Self {
        Self { offers: Map::uncapped(), logins: Map::uncapped(), ends: Map::uncapped(), next: 1 }
    }
}
impl Domain {
    #[must_use]
    pub fn terminal_count(&self, party: u64) -> u32 {
        match self.lending.offers.get(&party) {
            Some(offers) => offers.len(),
            None => 0,
        }
    }
    /// Called only for a watch admitted by the parent, within the per-party bound.
    pub fn offer_terminal(&mut self, party: u64, watcher: Token, terminal: u64) {
        if !self.lending.offers.contains_key(&party) {
            assert!(self.lending.offers.insert(party, List::with_capacity(TERMINALS)).is_ok(), "admitted party");
        }
        self.lending
            .offers
            .get_mut(&party)
            .expect("offering party")
            .push(Offer { watcher, terminal })
            .expect("parent admitted terminal bound");
    }
    /// Withdrawing the selected watch ends its login, never declaring a source.
    pub fn withdraw_terminal(&mut self, party: u64, watcher: Token) {
        let Some(offers) = self.lending.offers.remove(&party) else { return };
        let mut remaining = List::with_capacity(TERMINALS);
        for offer in offers.into_boxed() {
            if offer.watcher != watcher {
                remaining.push(offer).expect("withdrawal only removes offers");
            }
        }
        if !remaining.is_empty() {
            assert!(self.lending.offers.insert(party, remaining).is_ok(), "offering party");
        }
        if let Some(login) = self.lending.logins.get(&party)
            && login.offer.watcher == watcher
        {
            let end = if login.login.cancelled { LoginEnd::Cancelled } else { LoginEnd::NotFinished { exit: None } };
            finish(self, party, end);
        }
    }
    #[must_use]
    pub fn lending(&self, party: u64) -> Option<&TerminalLogin> {
        Some(&self.lending.logins.get(&party)?.login)
    }
    #[must_use]
    pub fn terminal_login(&self, party: u64, terminal: u64) -> Option<&TerminalLogin> {
        let pending = self.lending.logins.get(&party)?;
        if pending.offer.terminal != terminal {
            return None;
        }
        Some(&pending.login)
    }
    #[must_use]
    pub fn lent_end(&self, party: u64) -> Option<LentEnd> {
        self.lending.ends.get(&party).copied()
    }
}

/// The declaration's checks, allowing this holder's live source for a later check.
fn startable(domain: &Domain, limits: &Limits, holder: Holder, provider: u32, name: &[u8]) -> Result<(), Refusal> {
    if crate::SourceName::new(name).is_none() {
        return Err(Refusal::Uses);
    }
    match domain.providers.get(provider) {
        Some(offered) if holder != Holder::Deployment && !offered.offers.programs.is_empty() => {}
        Some(_) | None => return Err(Refusal::Provider { provider }),
    }
    let mut reserved = false;
    for (number, issued) in &domain.issued {
        if issued.name.as_bytes() == name {
            if issued.holder != holder {
                return Err(Refusal::NameTaken);
            }
            if domain.programs.contains_key(number) {
                return Err(Refusal::Limit { bound: crate::Bound::Sources });
            }
            reserved = true;
            if domain.sources.contains_key(number) {
                return Ok(());
            }
        }
    }
    if !reserved && domain.issued.len() >= limits.program_names.min(domain.issued.capacity()) {
        return Err(Refusal::Limit { bound: crate::Bound::Sources });
    }
    domain.room_for_account(limits, holder)
}

pub(crate) fn ask(
    domain: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    holder: Holder,
    provider: u32,
    name: Box<[u8]>,
    out: &mut Queue<Request>,
) {
    let result = begin(domain, &env.limits, holder, provider, name);
    let answer = match result {
        Ok(number) => Answer::LoginAsked { number },
        Err(refusal) => Answer::Refused { refusal },
    };
    out.push(Request::Answer { request, answer });
}
fn begin(domain: &mut Domain, limits: &Limits, holder: Holder, provider: u32, name: Box<[u8]>) -> Result<u32, Refusal> {
    startable(domain, limits, holder, provider, &name)?;
    let party = match holder {
        Holder::Party(party) => party,
        Holder::Deployment => return Err(Refusal::Provider { provider }),
    };
    let offers = domain.lending.offers.get(&party).ok_or(Refusal::NoTerminal)?;
    let offer = *offers.last().ok_or(Refusal::NoTerminal)?;
    if domain.lending.logins.contains_key(&party) {
        return Err(Refusal::LoginUnderWay);
    }
    let number = domain.lending.next;
    domain.lending.next = number.checked_add(1).ok_or(Refusal::Numbers)?;
    let login = TerminalLogin { number, provider, name, cancelled: false };
    assert!(domain.lending.logins.insert(party, UnderWay { login, offer }).is_ok(), "one login per offering party");
    Ok(number)
}
fn party(domain: &Domain, holder: Holder, number: u32) -> Result<u64, Refusal> {
    let party = match holder {
        Holder::Party(party) => party,
        Holder::Deployment => return Err(Refusal::NotLending),
    };
    let pending = domain.lending.logins.get(&party).ok_or(Refusal::NotLending)?;
    if pending.login.number != number {
        return Err(Refusal::NotLending);
    }
    Ok(party)
}
pub(crate) fn cancel(domain: &mut Domain, holder: Holder, number: u32) -> Result<(), Refusal> {
    let party = party(domain, holder, number)?;
    let pending = domain.lending.logins.get_mut(&party).expect("checked login");
    pending.login.cancelled = true;
    let mut offered = false;
    if let Some(offers) = domain.lending.offers.get(&party) {
        for offer in offers {
            offered |= offer.watcher == pending.offer.watcher;
        }
    }
    if !offered {
        finish(domain, party, LoginEnd::Cancelled);
    }
    Ok(())
}
pub(crate) fn reported(
    domain: &mut Domain,
    env: &Env<Limits>,
    holder: Holder,
    number: u32,
    exit: LoginExit,
    out: &mut Queue<Request>,
) -> Result<(), Refusal> {
    let party = party(domain, holder, number)?;
    let pending = domain.lending.logins.get(&party).expect("checked login");
    let end = if pending.login.cancelled {
        LoginEnd::Cancelled
    } else {
        match exit {
            LoginExit::Exited { code: 0 } => signed_in(domain, env, holder, party, out),
            LoginExit::Exited { .. } | LoginExit::Killed { .. } | LoginExit::NotStarted => {
                LoginEnd::NotFinished { exit: Some(exit) }
            }
        }
    };
    finish(domain, party, end);
    Ok(())
}
fn signed_in(domain: &mut Domain, env: &Env<Limits>, holder: Holder, party: u64, out: &mut Queue<Request>) -> LoginEnd {
    let pending = domain.lending.logins.get(&party).expect("reported login");
    let name = pending.login.name.clone();
    let provider = pending.login.provider;
    if let Some(source) = crate::programs::source_named(domain, &name)
        && crate::programs::owner(domain, &name) == Some(holder)
    {
        return match crate::programs::check(domain, holder, source, out) {
            Ok(()) => LoginEnd::SignedIn { source },
            Err(refusal) => LoginEnd::Refused { refusal },
        };
    }
    match crate::programs::declare(domain, env, holder, provider, None, name, out) {
        Ok(source) => LoginEnd::SignedIn { source },
        Err(crate::programs::DeclarationRefusal::Account(refusal)) => LoginEnd::Refused { refusal },
        Err(crate::programs::DeclarationRefusal::Taken) => LoginEnd::Refused { refusal: Refusal::ProgramNameReserved },
        Err(crate::programs::DeclarationRefusal::Name) => LoginEnd::Refused { refusal: Refusal::Uses },
        Err(crate::programs::DeclarationRefusal::Names) => {
            LoginEnd::Refused { refusal: Refusal::Limit { bound: crate::Bound::Sources } }
        }
    }
}
fn finish(domain: &mut Domain, party: u64, end: LoginEnd) {
    let pending = domain.lending.logins.remove(&party).expect("ending login is under way");
    let ended = LentEnd { number: pending.login.number, end };
    assert!(domain.lending.ends.insert(party, ended).is_ok(), "one last end per admitted party");
}
