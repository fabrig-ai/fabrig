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
                    w::SettingValue { value: temper_engine_smith::kinds::encode(&value) },
                ),
                ways::Edit::Remove => w::SettingEdit::Removed,
            };
            items
                .push(
                    w::ProposalItem { kind: item.kind, scope: item.scope, number: item.number, edit },
                )
                .ok()?;
        }
        let mut roles = List::with_capacity(limits.stock_way_roles);
        for entry in way.roles {
            roles
                .push(
                    w::WayRole {
                            charter: entry.role.charter,
                            words: entry.role.words,
                            budget: entry.role.budget,
                            running: entry.running,
                        },
                )
                .ok()?;
        }
        entries
            .push(
                w::StockWay {
                        name: way.name,
                        headline: way.headline,
                        line: way.line,
                        path: path(way.path, limits)?,
                        items,
                        delegation: way.delegation,
                        roles,
                    },
            )
            .ok()?;
    }
    let mut rows = List::with_capacity(limits.path_rules_rows);
    for row in document.rules.rows {
        rows.push(
            w::PathRule { paths: row.paths, review: row.review, sign_off: row.sign_off, lands: row.lands },
        )
        .ok()?;
    }
    let rules = w::PathRules { rows, note: document.rules.note };
    let panel = w::WaysPanel {
            project: document.project,
            ways: entries,
            current: document.current,
            own: path(document.own, limits)?,
            words: match document.words {
                Some(words) => Some(
                    w::ProjectWords { headline: words.headline, lede: words.lede },
                ),
                None => None,
            },
            rules,
            may_change: document.may_change,
        };
    Some(panel.encode(limits).ok()?)
}

pub(super) fn view_answer(view: ways::View, limits: &w::Limits) -> Option<Box<[u8]>> {
    let view = match view {
        ways::View::Way { name, line, path: value, current } => w::View::Way(
            w::WayView { name, line, path: path(value, limits)?, current },
        ),
        ways::View::Path(value) => w::View::Path(path(value, limits)?),
        ways::View::Proposal { before, after } => w::View::Proposal(
            w::ProposalView { before: path(before, limits)?, after: path(after, limits)? },
        ),
        ways::View::Charter { charter, card: value } => w::View::Charter(
            w::CharterView { charter, card: card(value, limits)? },
        ),
        ways::View::Rule { kind, number, card: value } => {
            w::View::Rule(w::RuleView { kind, number, card: card(value, limits)? })
        }
        ways::View::Roles(values) => {
            let mut roles = List::with_capacity(limits.roles_view_roles);
            for (charter, value) in values {
                roles
                    .push(
                        w::CharterView { charter, card: card(value, limits)? },
                    )
                    .ok()?;
            }
            w::View::Roles(w::RolesView { roles })
        }
        ways::View::NotDrawn(words) => w::View::NotDrawn(w::NotDrawn { words }),
    };
    let answer = w::ViewAnswer { view };
    Some(answer.encode(limits).ok()?)
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
                w::Opens::Charter(w::OpensCharter { charter })
            }
            ways::Opens::Rule { kind, scope, number } => {
                w::Opens::Rule(w::OpensRule { kind, scope, number })
            }
            ways::Opens::Gate { number, charter } => {
                w::Opens::Gate(w::OpensGate { number, charter })
            }
            ways::Opens::Tier(name) => w::Opens::Tier(w::OpensTier { name }),
            ways::Opens::Queue => w::Opens::Queue,
            ways::Opens::Turn => w::Opens::Turn,
        };
        stops
            .push(
                w::Stop {
                        actor,
                        step,
                        words: stop.words,
                        needed: stop.needed,
                        over: stop.over,
                        opens,
                        card: card(stop.card, limits)?,
                    },
            )
            .ok()?;
    }
    { let value = w::BranchPath { repository: value.repository, branch: value.branch, stops }; value.measure(limits).ok()?; Some(value) }
}

fn card(value: ways::Card, limits: &w::Limits) -> Option<w::Card> {
    let mut facts = List::with_capacity(limits.card_facts);
    for fact in value.facts {
        facts.push(w::Fact { label: fact.label, value: fact.value }).ok()?;
    }
    { let value = w::Card { who: value.who, name: value.name, facts }; value.measure(limits).ok()?; Some(value) }
}
