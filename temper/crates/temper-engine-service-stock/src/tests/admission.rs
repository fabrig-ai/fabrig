//! Stock amendment admission (temper/docs/design/testing/settings-tests.md, 5–7).
//! Reads jig's `acts::policy_recipient` and `acts::policy_proposal_allowed`, and
//! the `authority::check_request` acceptance used by `Core::goal_accept_allowed`.
use jig_domain_authority as authority;
use jig_domain_core::acts::{Recipient, policy_proposal_allowed, policy_recipient};
use skein_lib::{List, Queue, Wall};

const OWN: &str = "an_owners_widening_is_never_proposed_to_itself";
const FUNDING: &str = "a_members_batch_is_addressed_to_a_role_that_can_fund_it";
const KINDS: [authority::ProposalKind; 5] = [
    authority::ProposalKind::Batch,
    authority::ProposalKind::Effect,
    authority::ProposalKind::Widen,
    authority::ProposalKind::Amend,
    authority::ProposalKind::Escalation,
];

fn stock_domain() -> authority::Domain {
    let engine = super::stock::engine(&super::stock::generated());
    let stock = crate::stock::checked(&engine.llm, engine.limits.core.tasks.result_bytes).expect("stock defaults");
    let policy = stock.policies.get(&1).expect("stock policy 1").policy.clone();
    let mut domain = authority::Domain::new(stock.rules, engine.limits.core.authority).expect("stock rules");
    assert_eq!(install(&mut domain, policy), authority::PolicyFact::Added { project: 1 });
    domain
}

fn install(domain: &mut authority::Domain, policy: authority::Policy) -> authority::PolicyFact {
    let mut facts = Queue::uncapped();
    authority::step(domain, authority::Event::Policy { project: 1, policy }, &mut facts);
    let fact = facts.pop().expect("one policy admission decision");
    assert!(facts.is_empty());
    fact
}

fn party_request(kind: authority::ProposalKind) -> Option<authority::RequestKind> {
    match kind {
        authority::ProposalKind::Batch => Some(authority::RequestKind::Create),
        authority::ProposalKind::Amend => Some(authority::RequestKind::Amend),
        authority::ProposalKind::Effect | authority::ProposalKind::Widen | authority::ProposalKind::Escalation => None,
    }
}

// Each case varies one bound. In a batch the immediate child takes one task and
// one level; none of its own delegation kinds is needed for these cases.
fn leaf(role: &authority::Role) -> authority::Authority {
    let mut given = role.authority.clone();
    given.budget.spend = 0;
    given.grants = Box::new([]);
    given.delegation = authority::Delegation { kinds: Box::new([]), tasks: 0, depth: 0 };
    given
}

fn action(kind: authority::ProposalKind, given: authority::Authority) -> authority::Action {
    match kind {
        authority::ProposalKind::Batch => authority::Action::Batch(Box::new([authority::Delegate {
            executor: authority::Executor::Charter(1),
            authority: given,
            symbolic: Box::new([]),
        }])),
        authority::ProposalKind::Amend => authority::Action::Amend(given),
        authority::ProposalKind::Effect | authority::ProposalKind::Widen | authority::ProposalKind::Escalation => {
            unreachable!("parties propose only batches and amendments")
        }
    }
}

fn accept(
    domain: &authority::Domain,
    number: u32,
    action: &authority::Action,
) -> (authority::Answer, Queue<authority::Finding>) {
    let role = domain.role(1, number).expect("addressed role");
    let ask = authority::PersonAsk {
        project: 1,
        role: number,
        pool: authority::Numbers { budget: role.period_spend, spent: 0, spent_below: 0, reserved: 0 },
        tasks_left: role.authority.delegation.tasks,
        request: authority::PersonRequest::Accept(action.clone()),
    };
    let mut findings = Queue::uncapped();
    let checked = authority::check_request(domain, &ask, &mut findings);
    (checked.answer, findings)
}

// An exclusion requires every finding to be precisely role funding, not just
// one spend finding accompanying an unrelated authority or permission failure.
fn funding_only(findings: &Queue<authority::Finding>) -> bool {
    let mut found = false;
    for finding in findings {
        match *finding {
            authority::Finding::Authority { source: authority::Source::Role, mut lacks } => {
                if !lacks.spend {
                    return false;
                }
                lacks.spend = false;
                if !lacks.is_empty() {
                    return false;
                }
                found = true;
            }
            authority::Finding::Spend { source: authority::Source::Role } => found = true,
            authority::Finding::Authority {
                source: authority::Source::Task | authority::Source::Project | authority::Source::Deployment,
                ..
            }
            | authority::Finding::Spend {
                source: authority::Source::Task | authority::Source::Project | authority::Source::Deployment,
            }
            | authority::Finding::Oversized
            | authority::Finding::UnknownProject
            | authority::Finding::UnknownRole
            | authority::Finding::Executor { .. }
            | authority::Finding::Tasks { .. }
            | authority::Finding::Arithmetic
            | authority::Finding::RunBudget
            | authority::Finding::RunCap
            | authority::Finding::Deadline
            | authority::Finding::Account
            | authority::Finding::Writer
            | authority::Finding::Tool
            | authority::Finding::Grant { .. }
            | authority::Finding::Price { .. }
            | authority::Finding::ResourceAccess
            | authority::Finding::ResourceUnavailable
            | authority::Finding::Reference
            | authority::Finding::Scope { .. }
            | authority::Finding::Required { .. }
            | authority::Finding::Failed { .. }
            | authority::Finding::Unguarded { .. }
            | authority::Finding::Unpermitted
            | authority::Finding::Undecidable
            | authority::Finding::PeriodSpend => return false,
        }
    }
    found
}

struct Counts {
    cases: u32,
    accepted: u32,
    own: u32,
    funding: u32,
    before_address: u32,
    nobody: u32,
    unbounded: u32,
}

impl Counts {
    fn new() -> Self {
        Self { cases: 0, accepted: 0, own: 0, funding: 0, before_address: 0, nobody: 0, unbounded: 0 }
    }

    #[expect(
        clippy::disallowed_macros,
        clippy::print_stderr,
        reason = "the settings walk reports its named exclusions and counts"
    )]
    fn print(&self, name: &str) {
        eprintln!(
            "{name}: cases={}, accepted={}, refused before address={}, nobody={}, unbounded deadlines={}, {OWN}={}, {FUNDING}={}",
            self.cases, self.accepted, self.before_address, self.nobody, self.unbounded, self.own, self.funding
        );
    }
}

#[expect(
    clippy::disallowed_macros,
    clippy::print_stderr,
    clippy::use_debug,
    reason = "each settings case and its decision is printed before continuing"
)]
fn judge(
    domain: &authority::Domain,
    name: &str,
    who: (u32, authority::ProposalKind),
    bound: &str,
    past: bool,
    action: &authority::Action,
    counts: &mut Counts,
) {
    let (proposer, kind) = who;
    eprintln!("{name}: proposer role {proposer}, {kind:?}, {bound}");
    counts.cases = counts.cases.checked_add(1).expect("bounded walk");
    if let Err(why) = policy_proposal_allowed(domain, 1, action) {
        assert!(past, "{name}: proposer role {proposer}, {kind:?}, {bound} was refused before address: {why:?}");
        eprintln!("refused before address: {why:?}");
        counts.before_address = counts.before_address.checked_add(1).expect("bounded walk");
        return;
    }
    let addressed = roles(domain, kind);
    if addressed.is_empty() {
        eprintln!("refused as nobody's");
        counts.nobody = counts.nobody.checked_add(1).expect("bounded walk");
        return;
    }
    for number in &addressed {
        if *number == proposer {
            counts.own = counts.own.checked_add(1).expect("bounded walk");
            eprintln!("addressed role {number}: left out under {OWN}");
            continue;
        }
        let (answer, findings) = accept(domain, *number, action);
        if answer != authority::Answer::Allow && funding_only(&findings) {
            counts.funding = counts.funding.checked_add(1).expect("bounded walk");
            eprintln!("addressed role {number}: left out under {FUNDING}: {answer:?}, {findings:?}");
            continue;
        }
        assert_eq!(
            answer,
            authority::Answer::Allow,
            "{name}: proposer role {proposer}, {kind:?}, {bound}, addressed role {number}: {findings:?}"
        );
        counts.accepted = counts.accepted.checked_add(1).expect("bounded walk");
    }
}

#[derive(Clone, Copy)]
enum Bound {
    Spend,
    Depth,
    Tasks,
    Grants,
    Deadline,
}

impl Bound {
    fn name(self, past: bool) -> &'static str {
        match self {
            Self::Spend => {
                if past {
                    "spend one past"
                } else {
                    "spend at bound"
                }
            }
            Self::Depth => {
                if past {
                    "depth one past"
                } else {
                    "depth at bound"
                }
            }
            Self::Tasks => {
                if past {
                    "tasks one past"
                } else {
                    "tasks at bound"
                }
            }
            Self::Grants => {
                if past {
                    "grants one past"
                } else {
                    "grants at bound"
                }
            }
            Self::Deadline => {
                if past {
                    "deadline one past"
                } else {
                    "deadline at bound"
                }
            }
        }
    }

    fn proposal(
        self,
        domain: &authority::Domain,
        role: &authority::Role,
        kind: authority::ProposalKind,
        past: bool,
    ) -> Option<authority::Action> {
        let mut given = leaf(role);
        let child = u32::from(kind == authority::ProposalKind::Batch);
        match self {
            Self::Spend => {
                given.budget.spend = role
                    .authority
                    .budget
                    .spend
                    .min(role.period_spend)
                    .checked_add(u64::from(past))
                    .expect("stock spend plus one");
            }
            Self::Depth => {
                given.delegation.depth = role
                    .authority
                    .delegation
                    .depth
                    .checked_sub(child)
                    .expect("proposer can create")
                    .checked_add(u32::from(past))
                    .expect("stock depth plus one");
            }
            Self::Tasks => {
                given.delegation.tasks = role
                    .authority
                    .delegation
                    .tasks
                    .checked_sub(child)
                    .expect("proposer can create")
                    .checked_add(u32::from(past))
                    .expect("stock tasks plus one");
            }
            Self::Grants => {
                let capacity = u32::try_from(role.authority.grants.len())
                    .expect("stock grants")
                    .checked_add(1)
                    .expect("one extra grant");
                let mut grants = List::with_capacity(capacity);
                for grant in &role.authority.grants {
                    grants.push(grant.clone()).expect("counted grants");
                }
                if past {
                    let mut grant = role.authority.grants.first().expect("proposing stock role has grants").clone();
                    // One new kind beyond all configured kinds, not a duplicate grant.
                    for allowed in &domain.rules().ceiling.grants {
                        grant.kind = grant.kind.max(allowed.kind);
                    }
                    grant.kind = grant.kind.checked_add(1).expect("stock kind plus one");
                    grants.push(grant).expect("one extra grant");
                }
                given.grants = grants.into_boxed();
            }
            Self::Deadline => match role.authority.budget.deadline {
                Some(deadline) => {
                    given.budget.deadline = Some(Wall::from_nanos(
                        deadline.as_nanos().checked_add(u64::from(past)).expect("stock deadline plus one"),
                    ));
                }
                None => {
                    if past {
                        return None;
                    }
                    given.budget.deadline = None;
                }
            },
        }
        Some(action(kind, given))
    }
}

#[expect(
    clippy::disallowed_macros,
    clippy::print_stderr,
    clippy::use_debug,
    reason = "the settings walk names unbounded deadlines as well as its cases"
)]
fn walk(domain: &authority::Domain, name: &str) -> Counts {
    let policy = domain.policy(1).expect("stock policy");
    let mut counts = Counts::new();
    for role in &policy.roles {
        for kind in KINDS {
            let Some(request) = party_request(kind) else { continue };
            if !role.requests.allows(request) {
                continue;
            }
            for bound in [Bound::Spend, Bound::Depth, Bound::Tasks, Bound::Grants, Bound::Deadline] {
                for past in [false, true] {
                    let Some(proposal) = bound.proposal(domain, role, kind, past) else {
                        eprintln!(
                            "{name}: proposer role {}, {kind:?}: deadline unbounded; no one-past case",
                            role.number
                        );
                        counts.unbounded = counts.unbounded.checked_add(1).expect("bounded walk");
                        continue;
                    };
                    judge(domain, name, (role.number, kind), bound.name(past), past, &proposal, &mut counts);
                }
            }
        }
    }
    counts.print(name);
    counts
}

fn roles(domain: &authority::Domain, kind: authority::ProposalKind) -> List<u32> {
    match policy_recipient(domain, 1, kind) {
        Some(Recipient::Roles { project, roles }) => {
            assert_eq!(project, 1);
            assert!(!roles.is_empty(), "a nobody address must be None");
            roles
        }
        Some(Recipient::Task(_) | Recipient::Party(_) | Recipient::Role { .. }) => {
            unreachable!("a policy address names all deciding roles")
        }
        None => List::with_capacity(0),
    }
}

#[test]
fn stock_proposals_are_accepted_by_each_addressed_role() {
    let domain = stock_domain();
    let member = domain.role(1, 2).expect("stock member");
    let mut given = leaf(member);
    given.budget.spend = member.authority.budget.spend;
    let proposal = action(authority::ProposalKind::Batch, given);
    assert_eq!(roles(&domain, authority::ProposalKind::Batch).as_slice(), &[0, 1]);
    for number in [0, 1] {
        let (answer, findings) = accept(&domain, number, &proposal);
        assert_eq!(answer, authority::Answer::Allow, "member at bound, addressed role {number}: {findings:?}");
    }
    let counts = walk(&domain, "stock policy 1");
    assert!(counts.cases > 0 && counts.accepted > 0);
}

#[expect(
    clippy::disallowed_macros,
    clippy::print_stderr,
    reason = "generated pool mutations report their names and observed decisions"
)]
fn narrow_pools(stock: &authority::Domain, role: &authority::Role, position: usize) {
    let policy = stock.policy(1).expect("stock policy");
    // Lower the configured pool by one, and also cross its action-funding
    // boundary when that is below the configured pool. The latter exercises
    // the funding exclusion rather than only an unchanged affordable address.
    let spending =
        if role.authority.budget.spend < role.period_spend { role.authority.budget.spend.checked_sub(1) } else { None };
    for (boundary, pool) in [("configured pool", role.period_spend.checked_sub(1)), ("authority spend", spending)] {
        let Some(pool) = pool else { continue };
        let name = format!("role {} pool {} -> {pool}, one below {boundary}", role.number, role.period_spend);
        eprintln!("mutation: {name}");
        let mut domain = stock_domain();
        let mut changed = policy.clone();
        changed.roles.get_mut(position).expect("enumerated role").period_spend = pool;
        assert_eq!(install(&mut domain, changed), authority::PolicyFact::Changed { project: 1 });
        for kind in KINDS {
            assert_eq!(roles(&domain, kind), roles(stock, kind), "pool narrowing: role {}, {kind:?}", role.number);
        }
        walk(&domain, &name);
    }
    if role.period_spend == 0 {
        eprintln!("mutation: role {} pool is zero; no narrower pool", role.number);
    }
}

#[test]
#[expect(
    clippy::disallowed_macros,
    clippy::print_stderr,
    clippy::use_debug,
    reason = "generated mutations report their names and observed decisions"
)]
fn narrowed_stock_policies_address_only_accepting_roles() {
    let stock = stock_domain();
    let policy = stock.policy(1).expect("stock policy");
    for (position, role) in policy.roles.iter().enumerate() {
        narrow_pools(&stock, role, position);
        for kind in KINDS {
            if !role.decides.allows(kind) {
                continue;
            }
            let name = format!("role {} no longer decides {kind:?}", role.number);
            eprintln!("mutation: {name}");
            let mut domain = stock_domain();
            let mut changed = policy.clone();
            let narrowed = changed.roles.get_mut(position).expect("enumerated role");
            // Discover this kind's bit through the public membership query.
            for bit in 0..u8::BITS {
                let one = 1_u8.checked_shl(bit).expect("u8 bit");
                if authority::Proposals(one).allows(kind) {
                    narrowed.decides.0 &= !one;
                }
            }
            let decision = install(&mut domain, changed);
            if policy.escalation_role == Some(role.number) && kind == authority::ProposalKind::Escalation {
                assert_eq!(
                    decision,
                    authority::PolicyFact::Refused { project: 1, reason: authority::PolicyRefusal::InvalidRole }
                );
                eprintln!("decision: {decision:?}; not walked");
                continue;
            }
            assert_eq!(decision, authority::PolicyFact::Changed { project: 1 }, "{name}");
            for checked_kind in KINDS {
                let original = roles(&stock, checked_kind);
                let mut expected = List::with_capacity(stock.limits().roles);
                for number in &original {
                    if checked_kind != kind || *number != role.number {
                        expected.push(*number).expect("subset of stock recipients");
                    }
                }
                let addressed = roles(&domain, checked_kind);
                eprintln!("decision: {checked_kind:?} addressed to {addressed:?}");
                assert_eq!(addressed, expected, "decides narrowing: role {}, {kind:?}", role.number);
            }
            walk(&domain, &name);
        }
    }
}

#[test]
#[ignore = "the address weighing its accepter's funding"]
fn a_members_batch_is_addressed_to_a_role_that_can_fund_it() {
    let domain = stock_domain();
    let member = domain.role(1, 2).expect("stock member");
    let mut given = leaf(member);
    given.budget.spend = member.authority.budget.spend.checked_add(1).expect("one past stock spend");
    let proposal = action(authority::ProposalKind::Batch, given);
    policy_proposal_allowed(&domain, 1, &proposal).expect("within policy and rules");
    for number in &roles(&domain, authority::ProposalKind::Batch) {
        let (answer, findings) = accept(&domain, *number, &proposal);
        assert_eq!(
            answer,
            authority::Answer::Allow,
            "proposer role 2, Batch, spend one past bound, addressed role {number}: {findings:?}"
        );
    }
}
