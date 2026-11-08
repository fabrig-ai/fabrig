//! A landing rule's card reads the forge's current rules
//! (temper/docs/design/domain/forge.md, 8.9 and 8.13).
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
            return ways::View::Rule { kind, number, card: card(&held.rule, kind, number, &inputs.roles) };
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

fn card(rule: &settings::LandingRule, kind: u32, number: u32, roles: &[ways::Role]) -> ways::Card {
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
    facts.push(fact(b"Covers", &covers(&rule.pattern))).expect("rule facts");
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
            &approval_words(approval, roles),
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

// The approve stop's words are private to the derivation (forge.md, 8.13).
fn approval_words(approval: &settings::RuleApproval, roles: &[ways::Role]) -> Box<[u8]> {
    let mut name = bytes::concat(&[b"role ", Decimal::of(u64::from(approval.role)).as_bytes()]);
    let mut yours = false;
    for role in roles {
        if role.number == approval.role {
            name.clone_from(&role.name);
            yours = role.yours;
            break;
        }
    }
    let mut words = if yours {
        b"you approve".as_slice().into()
    } else {
        let article = match name.first() {
            Some(b'a' | b'e' | b'i' | b'o' | b'u' | b'A' | b'E' | b'I' | b'O' | b'U') => b"an ".as_slice(),
            Some(_) | None => b"a ".as_slice(),
        };
        bytes::concat(&[article, &name, b" approves"])
    };
    if approval.people > 1 {
        words = bytes::concat(&[&words, " ×".as_bytes(), Decimal::of(u64::from(approval.people)).as_bytes()]);
    }
    words
}
