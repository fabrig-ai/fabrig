//! Stock ways and documents (temper/docs/design/protocol/configuration/settings.md, 8).
use crate::{Card, Path};
use alloc::boxed::Box;
/// A stock way (protocol/configuration/settings.md, 8).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct StockWay {
    pub name: Box<[u8]>,
    pub headline: Box<[u8]>,
    pub line: Box<[u8]>,
    /// The project's rules it sets, by number, each of 1 to `RULES`; every other number of 1 to `RULES` it removes.
    pub rules: Box<[(u32, temper_engine_domain_forge::settings::LandingRule)]>,
    pub ways: Option<Laid>,
    /// The classes table the way sets whole; none for a world's way that sets none.
    pub classes: Option<temper_protocol_reaches::Classes>,
    /// Whether the way needs charter 5, role 0 and job 3 in the project's policy's delegation.
    pub delegation: bool,
}
/// The members of kind 7 a way lays over the project's own.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Laid {
    pub drafts: bool,
    pub goals: bool,
    pub way: u32,
}
pub const RULES: u32 = 4;

/// An item of temper's kinds, as jig's `ChangeSettings` carries one.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Item {
    pub kind: u32,
    pub scope: u32,
    pub number: u32,
    pub edit: Edit,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Edit {
    Set(Value),
    Remove,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct WayEntry {
    pub name: Box<[u8]>,
    pub headline: Box<[u8]>,
    pub line: Box<[u8]>,
    pub path: Path,
    pub items: Box<[Item]>,
    pub delegation: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct WaysDocument {
    pub project: u32,
    pub ways: Box<[WayEntry]>,
    pub current: Option<u32>,
    pub own: Path,
    pub words: Option<temper_engine_domain_forge::settings::ProjectWords>,
    pub may_change: bool,
}

/// A view's answer (client/pages/chat/conversation.md, 3).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum View {
    Way { name: Box<[u8]>, line: Box<[u8]>, path: Path, current: bool },
    Path(Path),
    Proposal { before: Path, after: Path },
    Charter { charter: u32, card: Card },
    Rule { kind: u32, number: u32, card: Card },
    Roles(Box<[(u32, Card)]>),
    NotDrawn(Box<[u8]>), // one line, at most 512 bytes
}

use skein_lib::{List, Map};
use temper_engine_domain_forge::settings::{Held, ProjectWays, TemplateGoal, Value, project_stock};

/// The way's rules in number order, then kind 7 and kind 8 where the way lays them.
#[must_use]
pub fn items(way: &StockWay, project: u32, ways: Option<&ProjectWays>) -> Box<[Item]> {
    let mut items = List::uncapped();
    for number in 1..=RULES {
        let mut edit = Edit::Remove;
        for (set_number, rule) in &way.rules {
            if *set_number == number {
                edit = Edit::Set(Value::Rule(rule.clone()));
                break;
            }
        }
        items.push(Item { kind: 2, scope: project, number, edit }).expect("four rule items");
    }
    if let Some(laid) = &way.ways {
        let mut value = own_ways(ways);
        value.drafts = laid.drafts;
        value.goals = laid.goals;
        value.way = laid.way;
        items
            .push(Item { kind: 7, scope: project, number: 1, edit: Edit::Set(Value::Ways(value)) })
            .expect("one ways item");
    }
    if let Some(table) = &way.classes {
        items
            .push(Item { kind: 8, scope: project, number: 1, edit: Edit::Set(Value::Classes(table.clone())) })
            .expect("one classes item");
    }
    items.into_boxed()
}

/// Held rules after the edits: deployment order, then the project's numbers.
/// Removed project rules fall back to the stock's every-project rule.
#[must_use]
pub fn laid(rules: &[Held], stock: &[Held], project: u32, items: &[Item]) -> Box<[Held]> {
    let mut result = List::uncapped();
    let mut own = Map::uncapped();
    for held in rules {
        if held.scope == 0 {
            result.push(held.clone()).expect("held deployment rules");
        } else if held.scope == project {
            own.insert(held.number, held.rule.clone()).expect("held project rules");
        }
    }
    for item in items {
        if item.kind != 2 || item.scope != project {
            continue;
        }
        match &item.edit {
            Edit::Set(value) => match value {
                Value::Rule(rule) => {
                    own.insert(item.number, rule.clone()).expect("edited project rule");
                }
                Value::Charter(_)
                | Value::Goal(_)
                | Value::Questions(_)
                | Value::CharterWords(_)
                | Value::Classes(_)
                | Value::Dependency(_) | Value::Follow(_)
                | Value::ProjectWords(_)
                | Value::Ways(_) => {}
            },
            Edit::Remove => {
                own.remove(&item.number);
                for held in stock {
                    if held.number == item.number {
                        own.insert(item.number, held.rule.clone()).expect("stock project rule");
                        break;
                    }
                }
            }
        }
    }
    for (&number, rule) in &own {
        result.push(Held { scope: project, number, rule: rule.clone() }).expect("project rules by number");
    }
    result.into_boxed()
}

/// Kind 7 after the edits, falling back to the stock where removed.
#[must_use]
pub fn laid_ways(ways: Option<&ProjectWays>, project: u32, items: &[Item]) -> ProjectWays {
    let mut result = own_ways(ways);
    for item in items {
        if item.kind != 7 || item.scope != project || item.number != 1 {
            continue;
        }
        match &item.edit {
            Edit::Set(value) => match value {
                Value::Ways(ways) => result = *ways,
                Value::Rule(_)
                | Value::Charter(_)
                | Value::Goal(_)
                | Value::Questions(_)
                | Value::CharterWords(_)
                | Value::Classes(_)
                | Value::Dependency(_) | Value::Follow(_)
                | Value::ProjectWords(_) => {}
            },
            Edit::Remove => result = own_ways(None),
        }
    }
    result
}

/// A goal template's work with a decision's items laid over it, kind 11
/// (temper/docs/design/protocol/configuration/settings.md, 2): an item of kind 11 at scope 0
/// and the template's number sets it, its removal leaves neither.
#[must_use]
pub fn laid_goal(goal: Option<TemplateGoal>, template: u32, items: &[Item]) -> TemplateGoal {
    let mut result = goal.unwrap_or_default();
    for item in items {
        if item.kind != 11 || item.scope != 0 || item.number != template {
            continue;
        }
        match &item.edit {
            Edit::Set(value) => match value {
                Value::Goal(goal) => result = *goal,
                Value::Rule(_)
                | Value::Charter(_)
                | Value::Questions(_)
                | Value::Ways(_)
                | Value::CharterWords(_)
                | Value::Classes(_)
                | Value::Dependency(_) | Value::Follow(_)
                | Value::ProjectWords(_) => {}
            },
            Edit::Remove => result = TemplateGoal::default(),
        }
    }
    result
}

fn own_ways(ways: Option<&ProjectWays>) -> ProjectWays {
    match ways {
        Some(ways) => *ways,
        None => match project_stock(7).expect("kind 7 has stock").value {
            Value::Ways(ways) => ways,
            Value::Rule(_)
            | Value::Charter(_)
            | Value::Goal(_)
            | Value::Questions(_)
            | Value::CharterWords(_)
            | Value::Classes(_)
            | Value::Dependency(_) | Value::Follow(_)
            | Value::ProjectWords(_) => unreachable!("kind 7 is ways"),
        },
    }
}

/// The index of the first way with the same drawn stops.
#[must_use]
pub fn current(ways: &[Path], own: &Path) -> Option<u32> {
    for (index, way) in ways.iter().enumerate() {
        if way.same(own) {
            return u32::try_from(index).ok();
        }
    }
    None
}

/// The watched subject of a project's ways, outside every other subject's bits.
#[must_use]
pub const fn subject(project: u32) -> u64 {
    let [a, b, c, d] = project.to_le_bytes();
    (1 << 62) | (1 << 38) | u64::from_le_bytes([a, b, c, d, 0, 0, 0, 0])
}
