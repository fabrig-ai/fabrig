//! Stock ways given to the engine at each start (protocol/configuration/settings.md, 8).
use alloc::boxed::Box;
use skein_lib::bytes;
use temper_engine_domain_forge::{
    GateLast, GatePattern, JudgeFreshness,
    settings::{LandingRule, RuleApproval},
};
use temper_engine_domain_ways::{Laid, StockWay};
use temper_protocol_reaches::{Classes, Way};

/// The project's three stock ways, from more of the person to more of their agents.
#[must_use]
pub fn ways() -> Box<[StockWay]> {
    let mut approve = branch(GateLast::Open(Box::default()), true);
    approve.approvals = Box::new([RuleApproval { role: 0, people: 1, freshness: JudgeFreshness::Clean }]);
    Box::new([
        StockWay {
            name: bytes::copy_of(b"You approve"),
            headline: bytes::copy_of(b"You approve every change"),
            line: bytes::copy_of(b"An agent makes each change; nothing reaches main until you approve it."),
            rules: Box::new([(1, approve)]),
            ways: None,
            classes: Some(Classes::stock(Way::YouApprove)),
            delegation: false,
        },
        StockWay {
            name: bytes::copy_of(b"Agents review"),
            headline: bytes::copy_of(b"Agents review every change, and code lands it"),
            line: bytes::copy_of(b"An agent makes each change, another reviews it, and code lands it."),
            rules: Box::new([(1, super::review_rule())]),
            ways: None,
            classes: Some(Classes::stock(Way::AgentsReview)),
            delegation: false,
        },
        StockWay {
            name: bytes::copy_of(b"Draft first"),
            headline: bytes::copy_of(b"Draft first, then agents build it on its own branch"),
            line: bytes::copy_of(
                b"You and an agent write a design draft; agents build it on its own branch, and it lands whole.",
            ),
            rules: Box::new([
                (1, branch(GateLast::Exact(bytes::copy_of(b"main")), true)),
                (2, branch(GateLast::Open(bytes::copy_of(b"temper")), false)),
            ]),
            ways: Some(Laid { drafts: true, goals: true, way: 4 }),
            classes: Some(Classes::stock(Way::DraftFirst)),
            delegation: true,
        },
    ])
}

fn branch(last: GateLast, checked: bool) -> LandingRule {
    LandingRule {
        sets: Box::new([]),
        pattern: GatePattern { segments: Box::new([bytes::copy_of(b"branch")]), last },
        enforce: true,
        ci: checked,
        up_to_date: checked,
        gates: Box::default(),
        approvals: Box::default(),
    }
}
