//! A landing rule's card reads the forge's current rules
//! (temper/docs/design/domain/forge/change/gates.md, 3 and temper/docs/design/domain/forge/change/branch-path.md).
use super::{fact, not_drawn};
use crate::{Domain, ways};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, bytes};
use temper_engine_domain_forge::{GateLast, GatePattern, JudgeFreshness, settings};

pub(super) fn draw(domain: &Domain, project: u32, party: u64, kind: u32, number: u32) -> ways::View {
    let rules = domain.forge.landing_rules(project);
    let scope = if kind == 1 { 0 } else { project };
    for held in &rules {
        if held.scope == scope && held.number == number {
            let inputs = super::super::inputs(domain, project, party, &[], None, None);
            return ways::View::Rule { kind, number, card: card(&held.rule, kind, number, &inputs) };
        }
    }
    not_drawn(&[
        b"no rule ",
        Decimal::of(u64::from(kind)).as_bytes(),
        b"/",
        Decimal::of(u64::from(number)).as_bytes(),
        b".",
    ])
}

fn card(rule: &settings::LandingRule, kind: u32, number: u32, inputs: &ways::Inputs) -> ways::Card {
    let who = if rule.enforce {
        "a landing rule · holds a merge until it is met"
    } else {
        "a landing rule · advice, enforcing nothing"
    };
    let name = bytes::concat(&[
        if kind == 1 { b"the deployment's rule " } else { b"the project's rule " },
        Decimal::of(u64::from(number)).as_bytes(),
    ]);
    let mut facts = List::uncapped();
    let mut covering = covers(&rule.pattern);
    if ways::by_path(rule, &inputs.sets) {
        covering = bytes::concat(&[&covering, b", where a change touches ", &ways::set_names(rule, &inputs.sets)]);
    }
    facts.push(fact(b"Covers", &covering)).expect("rule facts");
    let mut asks = Box::from(b"".as_slice());
    if rule.ci {
        asks = b"CI".as_slice().into();
    }
    if rule.up_to_date {
        asks = bytes::concat(&[&asks, if asks.is_empty() { b"" } else { b", " }, b"up to date"]);
    }
    if !asks.is_empty() {
        facts.push(fact(b"Asks", &asks)).expect("rule facts");
    }
    let mut gates = Box::from(b"".as_slice());
    for gate in &rule.gates {
        gates = bytes::concat(&[
            &gates,
            if gates.is_empty() { b"" } else { b"; " },
            b"the reviewer, ",
            if rule.enforce && gate.blocking { b"blocking" } else { b"advice" },
            match gate.freshness {
                JudgeFreshness::Exact => b"",
                JudgeFreshness::Clean => b", at a clean head",
            },
        ]);
    }
    if !gates.is_empty() {
        facts.push(fact(b"Gates", &gates)).expect("rule facts");
    }
    let mut approvals = Box::from(b"".as_slice());
    for approval in &rule.approvals {
        approvals = bytes::concat(&[
            &approvals,
            if approvals.is_empty() { b"" } else { b"; " },
            &approval_words(approval, &inputs.roles),
        ]);
    }
    if !approvals.is_empty() {
        facts.push(fact(b"Approvals", &approvals)).expect("rule facts");
    }
    ways::Card {
        who: ways::cut_on_word(who.as_bytes(), ways::VALUE),
        name: ways::cut_on_word(&name, ways::VALUE),
        facts: facts.into_boxed(),
    }
}

fn covers(pattern: &GatePattern) -> Box<[u8]> {
    let forge = match pattern.segments.first() {
        Some(first) => first.as_ref() == b"forge",
        None => false,
    };
    let branch = match &pattern.last {
        GateLast::Exact(name) => bytes::concat(&[if forge { b"branch " } else { b"the branch " }, name]),
        GateLast::Open(prefix) => {
            if prefix.is_empty() {
                b"every branch".as_slice().into()
            } else {
                bytes::concat(&[b"branches beginning ", prefix])
            }
        }
    };
    if forge {
        let owner = match pattern.segments.get(2) {
            Some(owner) => owner.as_ref(),
            None => b"*",
        };
        let name = match pattern.segments.get(3) {
            Some(name) => name.as_ref(),
            None => b"*",
        };
        bytes::concat(&[owner, b"/", name, b"'s ", &branch])
    } else {
        branch
    }
}

fn approval_words(approval: &settings::RuleApproval, roles: &[ways::Role]) -> Box<[u8]> {
    for role in roles {
        if role.number == approval.role {
            return ways::approval_words(role, approval.people, false);
        }
    }
    let role = ways::Role {
        number: approval.role,
        name: bytes::concat(&[b"role ", Decimal::of(u64::from(approval.role)).as_bytes()]),
        yours: false,
        holders: 0,
    };
    ways::approval_words(&role, approval.people, false)
}
