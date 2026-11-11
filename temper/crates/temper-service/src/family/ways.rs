//! Paths, watched ways and read views (temper/docs/design/client/forge/records.md;
//! temper/docs/design/client/forge/served.md).
use alloc::boxed::Box;
use skein_lib::List;
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_engine_domain::ways;

/// Encode the project's ways, refusing a document beyond any wire bound.
#[must_use]
pub fn ways_panel(document: ways::WaysDocument, limits: &w::Limits) -> Option<Box<[u8]>> {
    let mut entries = List::with_capacity(limits.ways_panel_ways);
    for way in document.ways {
        let mut items = List::with_capacity(limits.stock_way_items);
        for item in way.items {
            let edit = match item.edit {
                ways::Edit::Set(value) => w::SettingEdit::Set(
                    w::SettingValue::new(
                        limits,
                        w::SettingValueParts { value: temper_engine_smith::kinds::encode(&value) },
                    )
                    .ok()?,
                ),
                ways::Edit::Remove => w::SettingEdit::Removed,
            };
            items
                .push(
                    w::ProposalItem::new(
                        limits,
                        w::ProposalItemParts { kind: item.kind, scope: item.scope, number: item.number, edit },
                    )
                    .ok()?,
                )
                .ok()?;
        }
        entries
            .push(
                w::StockWay::new(
                    limits,
                    w::StockWayParts {
                        name: way.name,
                        headline: way.headline,
                        line: way.line,
                        path: path(way.path, limits)?,
                        items,
                        delegation: way.delegation,
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    let panel = w::WaysPanel::new(
        limits,
        w::WaysPanelParts {
            project: document.project,
            ways: entries,
            current: document.current,
            own: path(document.own, limits)?,
            words: match document.words {
                Some(words) => Some(
                    w::ProjectWords::new(limits, w::ProjectWordsParts { headline: words.headline, lede: words.lede })
                        .ok()?,
                ),
                None => None,
            },
            rules: w::PathRules::new(limits, w::PathRulesParts { rows: List::with_capacity(0), note: Box::new([]) })
                .ok()?,
            may_change: document.may_change,
        },
    )
    .ok()?;
    Some(wire::ways_panel(&panel))
}

pub(super) fn view_answer(view: ways::View, limits: &w::Limits) -> Option<Box<[u8]>> {
    let view = match view {
        ways::View::Way { name, line, path: value, current } => w::View::Way(
            w::WayView::new(limits, w::WayViewParts { name, line, path: path(value, limits)?, current }).ok()?,
        ),
        ways::View::Path(value) => w::View::Path(path(value, limits)?),
        ways::View::Proposal { before, after } => w::View::Proposal(
            w::ProposalView::new(
                limits,
                w::ProposalViewParts { before: path(before, limits)?, after: path(after, limits)? },
            )
            .ok()?,
        ),
        ways::View::Charter { charter, card: value } => w::View::Charter(
            w::CharterView::new(limits, w::CharterViewParts { charter, card: card(value, limits)? }).ok()?,
        ),
        ways::View::Rule { kind, number, card: value } => {
            w::View::Rule(w::RuleView::new(limits, w::RuleViewParts { kind, number, card: card(value, limits)? }).ok()?)
        }
        ways::View::Roles(values) => {
            let mut roles = List::with_capacity(limits.roles_view_roles);
            for (charter, value) in values {
                roles
                    .push(
                        w::CharterView::new(limits, w::CharterViewParts { charter, card: card(value, limits)? })
                            .ok()?,
                    )
                    .ok()?;
            }
            w::View::Roles(w::RolesView::new(limits, w::RolesViewParts { roles }).ok()?)
        }
        ways::View::NotDrawn(words) => w::View::NotDrawn(w::NotDrawn::new(limits, w::NotDrawnParts { words }).ok()?),
    };
    let answer = w::ViewAnswer::new(limits, w::ViewAnswerParts { view }).ok()?;
    Some(wire::view_answer(&answer))
}

fn path(value: ways::Path, limits: &w::Limits) -> Option<w::BranchPath> {
    let mut stops = List::with_capacity(limits.branch_path_stops);
    for stop in value.stops {
        let actor = match stop.actor {
            ways::Actor::Person => w::Actor::Person,
            ways::Actor::Agent => w::Actor::Agent,
            ways::Actor::Code => w::Actor::Code,
            ways::Actor::Bar => w::Actor::Bar,
        };
        let step = match stop.step {
            ways::Step::Ask => w::PathStep::Ask,
            ways::Step::Draft => w::PathStep::Draft,
            ways::Step::Plan => w::PathStep::Plan,
            ways::Step::Produce => w::PathStep::Produce,
            ways::Step::ProduceMany => w::PathStep::ProduceMany,
            ways::Step::Check => w::PathStep::Check,
            ways::Step::Review => w::PathStep::Review,
            ways::Step::Repair => w::PathStep::Repair,
            ways::Step::Approve => w::PathStep::Approve,
            ways::Step::Conflict => w::PathStep::Conflict,
            ways::Step::CatchUp => w::PathStep::CatchUp,
            ways::Step::Resolve => w::PathStep::Resolve,
            ways::Step::Merge => w::PathStep::Merge,
            ways::Step::Prepare => w::PathStep::Prepare,
            ways::Step::Prove => w::PathStep::Prove,
            ways::Step::Turn => w::PathStep::Turn,
            ways::Step::Verify => w::PathStep::Verify,
            ways::Step::Branch => w::PathStep::Branch,
            ways::Step::Design => w::PathStep::Design,
            ways::Step::CloseOut => w::PathStep::CloseOut,
        };
        let opens = match stop.opens {
            ways::Opens::Nothing => w::Opens::Nothing,
            ways::Opens::Charter(charter) => {
                w::Opens::Charter(w::OpensCharter::new(limits, w::OpensCharterParts { charter }).ok()?)
            }
            ways::Opens::Rule { kind, scope, number } => {
                w::Opens::Rule(w::OpensRule::new(limits, w::OpensRuleParts { kind, scope, number }).ok()?)
            }
            ways::Opens::Gate { number, charter } => {
                w::Opens::Gate(w::OpensGate::new(limits, w::OpensGateParts { number, charter }).ok()?)
            }
            ways::Opens::Tier(name) => w::Opens::Tier(w::OpensTier::new(limits, w::OpensTierParts { name }).ok()?),
            ways::Opens::Queue => w::Opens::Queue,
            ways::Opens::Turn => w::Opens::Turn,
        };
        stops
            .push(
                w::Stop::new(
                    limits,
                    w::StopParts {
                        actor,
                        step,
                        words: stop.words,
                        needed: stop.needed,
                        over: stop.over,
                        opens,
                        card: card(stop.card, limits)?,
                    },
                )
                .ok()?,
            )
            .ok()?;
    }
    w::BranchPath::new(limits, w::BranchPathParts { repository: value.repository, branch: value.branch, stops }).ok()
}

fn card(value: ways::Card, limits: &w::Limits) -> Option<w::Card> {
    let mut facts = List::with_capacity(limits.card_facts);
    for fact in value.facts {
        facts.push(w::Fact::new(limits, w::FactParts { label: fact.label, value: fact.value }).ok()?).ok()?;
    }
    w::Card::new(limits, w::CardParts { who: value.who, name: value.name, facts }).ok()
}
