//! Areas, their holders and which parts they fill (client/pages/team.md, 1).
use super::{Facts, Parts};
use alloc::boxed::Box;
use skein_lib::{List, Set};
use temper_client_protocol_wire::v1 as w;

/// The findings document's ownership facts, independent of its finding rows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Areas {
    pub rows: List<Area>,
    pub triager: Option<w::AreaOwner>,
    pub holder: w::Holder,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Area {
    pub name: Box<[u8]>,
    pub words: Box<[u8]>,
    pub owner: w::AreaOwner,
    pub holder: Option<w::Holder>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AreaName {
    Named(Box<[u8]>),
    AnythingElse,
}
/// One table row; a listed role's first row carries its spend and selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AreaPart {
    pub name: AreaName,
    pub words: Box<[u8]>,
    pub owner: Option<w::AreaOwner>,
    pub holder: Option<w::Holder>,
    pub role: Option<u64>,
    pub first: bool,
    pub current: bool,
}

pub(super) fn gather(document: &w::Findings) -> Areas {
    let mut rows = List::with_capacity(document.areas().len());
    for row in document.areas() {
        rows.push(Area {
            name: row.name().into(),
            words: row.words().into(),
            owner: row.owner().clone(),
            holder: row.holder().clone(),
        })
        .expect("one area per document row");
    }
    Areas { rows, triager: document.triager().clone(), holder: document.holder().clone() }
}

pub(super) fn parts(facts: &Facts, parts: &mut Parts) {
    parts.areas = facts.areas.is_some();
    if let Some(areas) = &facts.areas {
        parts.areas_table = !areas.rows.is_empty() || areas.triager.is_some();
        parts.areas_empty = !parts.areas_table;
        parts.who_routes = parts.areas_table;
        parts.lede_areas = parts.areas_table;
        if parts.areas_table {
            parts.area_rows = List::with_capacity(areas.rows.len().saturating_add(1));
            for area in &areas.rows {
                parts
                    .area_rows
                    .push(AreaPart {
                        name: AreaName::Named(area.name.clone()),
                        words: area.words.clone(),
                        owner: Some(area.owner.clone()),
                        holder: area.holder.clone(),
                        role: None,
                        first: false,
                        current: false,
                    })
                    .expect("areas and their last row");
            }
            parts
                .area_rows
                .push(AreaPart {
                    name: AreaName::AnythingElse,
                    words: Box::default(),
                    owner: None,
                    holder: Some(areas.holder.clone()),
                    role: None,
                    first: false,
                    current: false,
                })
                .expect("space for the last row");
        }
    }
    let mut seen = Set::with_capacity(parts.area_rows.len());
    parts.owns = List::with_capacity(parts.area_rows.len());
    for row in parts.area_rows.iter_mut() {
        if let Some(w::Holder::Task(task)) = &row.holder {
            for role in &facts.roles {
                if role.task == task.number() {
                    row.role = Some(role.task);
                    row.first = seen.insert(role.task).expect("one key per table row");
                    row.current = row.first && parts.card == Some(role.task);
                    if parts.card == Some(role.task) {
                        parts.owns.push(row.name.clone()).expect("at most every row");
                    }
                    break;
                }
            }
        }
    }
    parts.standing = List::with_capacity(facts.roles.len());
    for role in &facts.roles {
        if !seen.contains(&role.task) {
            parts.standing.push(role.task).expect("at most every listed role");
        }
    }
    parts.standing_roles = !parts.standing.is_empty() || facts.roles_cut;
    parts.box_areas = List::with_capacity(facts.people.len());
    for person in &facts.people {
        let mut names = List::with_capacity(parts.area_rows.len());
        for row in &parts.area_rows {
            if let Some(w::Holder::Party(party)) = &row.holder
                && party.number() == person.party
            {
                names.push(row.name.clone()).expect("at most every row");
            }
        }
        if !names.is_empty() {
            parts.box_areas.push((person.party, names)).expect("one entry per person");
        }
    }
}
