//! The brief's section inventory and byte plan (domain/engine/briefs.md, section 1).
//! Core sections arrive as text. A connector's section is only its
//! number, kind, fixed name, token and reported size; its bytes stay with the connector.
//! The plan gives required sections first claim on the budget, then visits
//! optional sections by priority. A connector cuts its own content to the
//! size requested here and reports the resulting size before completion.

use alloc::boxed::Box;

use jig_domain_contract::parts::{Failure, Fate, Section};
use skein_lib::{List, Map, Queue, Set, Token};

/// A section the core owns and renders in its own words.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Core {
    /// The task's specification and result contract.
    Task,
    /// A standing task's current note (domain/tasks/standing.md, 6).
    Now,
    /// Its requesters' specifications, nearest first.
    Lineage,
    /// Dependencies' and inputs' results.
    Results,
    /// Delegates, their states and results that came since the last turn.
    Plan,
    /// Earlier attempts and why they failed.
    Attempts,
    /// Calls committed since the last turn when the run does not resume.
    Calls,
    /// Open questions, proposals and escalations waiting for this run.
    Waiting,
    /// The index of notes in scope.
    NotesIndex,
    /// The requester's whole preferences (domain/engine/briefs.md, 2.1).
    Preferences,
    /// The end of a transcript when it cannot be resumed whole.
    TranscriptTail,
}

/// The fixed name of a core section in a failure (domain/engine/briefs.md, 1.1).
#[must_use]
pub fn core_name(kind: Core) -> &'static [u8] {
    match kind {
        Core::Task => b"task",
        Core::Now => b"now",
        Core::Lineage => b"lineage",
        Core::Results => b"results",
        Core::Plan => b"plan",
        Core::Attempts => b"attempts",
        Core::Calls => b"calls",
        Core::Waiting => b"waiting",
        Core::NotesIndex => b"notes",
        Core::Preferences => b"preferences",
        Core::TranscriptTail => b"transcript",
    }
}

/// One requested section. A lower `priority` is kept before a higher one.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Planned {
    /// Text the core owns and may cut by this kind's rules.
    Core { kind: Core, text: Box<[u8]>, limit: u32, priority: u16, required: bool },
    /// Opaque content retained by a numbered connector.
    Connector { connector: u16, section: Section },
}

impl Planned {
    /// The size reported by the owner before cutting.
    #[must_use]
    pub fn size(&self) -> u32 {
        match self {
            Planned::Core { text, .. } => u32::try_from(text.len()).unwrap_or(u32::MAX),
            Planned::Connector { section, .. } => section.size,
        }
    }

    /// The largest rendered section this owner may deliver.
    #[must_use]
    pub fn limit(&self) -> u32 {
        match self {
            Planned::Core { limit, .. } => *limit,
            Planned::Connector { section, .. } => section.limit,
        }
    }

    /// Whether absence of this section fails the brief.
    #[must_use]
    pub fn required(&self) -> bool {
        match self {
            Planned::Core { required, .. } => *required,
            Planned::Connector { section, .. } => section.required,
        }
    }

    /// Which sections receive room first.
    #[must_use]
    pub fn priority(&self) -> u16 {
        match self {
            Planned::Core { priority, .. } => *priority,
            Planned::Connector { section, .. } => section.priority,
        }
    }
}

/// A section's position and its allotted size in a completed brief.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Placement {
    /// The position of its source in the incoming inventory.
    pub index: u32,
    /// The most bytes this section may take after cuts.
    pub size: u32,
}

/// What a connector must do with a section it holds.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ConnectorAction {
    /// Cut its section by its own rules to this size, then report the size.
    CutTo { connector: u16, token: Token, size: u32 },
    /// Release an optional section that has no room.
    Drop { connector: u16, token: Token },
}

/// The budget plan, preserving source order for the final assignment.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Plan {
    /// Every retained section and its allotment, in source order.
    pub order: Box<[Placement]>,
    /// Requests to connector owners, in the order sections were considered.
    pub connectors: Box<[ConnectorAction]>,
    /// The sum of the allotted sizes.
    pub size: u32,
}

/// Plan sections by requirement and priority within `budget`.
/// Sections can be cut by their owner. Returns `None` when the inventory cannot fit.
#[must_use]
pub fn plan(sections: &[Planned], budget: u32) -> Option<Plan> {
    let count = u32::try_from(sections.len()).ok()?;
    let mut seen = List::with_capacity(count);
    let mut sizes = List::with_capacity(count);
    for _ in sections {
        seen.push(false).ok()?;
        sizes.push(0_u32).ok()?;
    }
    let mut connectors = List::with_capacity(count);
    let mut left = budget;
    for _ in sections {
        let mut selected = None;
        for (index, section) in sections.iter().enumerate() {
            let index = u32::try_from(index).ok()?;
            if *seen.get(index)? {
                continue;
            }
            match selected {
                Some(previous) => {
                    let old = sections.get(usize::try_from(previous).ok()?)?;
                    if (section.required() && !old.required())
                        || (section.required() == old.required() && section.priority() < old.priority())
                    {
                        selected = Some(index);
                    }
                }
                None => selected = Some(index),
            }
        }
        let index = selected?;
        *seen.get_mut(index)? = true;
        let section = sections.get(usize::try_from(index).ok()?)?;
        let wanted = section.size();
        let allotted = wanted.min(section.limit()).min(left);
        *sizes.get_mut(index)? = allotted;
        left = left.checked_sub(allotted)?;
        match section {
            Planned::Core { .. } => {}
            Planned::Connector { connector, section: held } => {
                let action = if allotted == 0 && !section.required() {
                    ConnectorAction::Drop { connector: *connector, token: held.token }
                } else if allotted < wanted {
                    ConnectorAction::CutTo { connector: *connector, token: held.token, size: allotted }
                } else {
                    continue;
                };
                connectors.push(action).ok()?;
            }
        }
    }
    let mut order = List::with_capacity(count);
    for (index, section) in sections.iter().enumerate() {
        let index = u32::try_from(index).ok()?;
        let size = *sizes.get(index)?;
        if size > 0 || section.required() {
            order.push(Placement { index, size }).ok()?;
        }
    }
    Some(Plan { order: order.into_boxed(), connectors: connectors.into_boxed(), size: budget.checked_sub(left)? })
}

/// A connector's answer to the preparation context.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ConnectorPlan {
    pub connector: u16,
    pub sections: Box<[Section]>,
}

/// Slot placement and the connector sections it leaves out.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Placing {
    Placed { sections: Box<[Planned]>, dropped: Box<[ConnectorAction]> },
    Failed { failure: Failure, dropped: Box<[ConnectorAction]> },
}

/// Place the core's sections, planned documents and other connector sections, in that order
/// (jig/docs/design/domain/engine/briefs.md, 1.4). Each connector gives one plan.
#[must_use]
pub fn place(core: &[Planned], documents: &[u16], plans: &[ConnectorPlan], sections: u32) -> Placing {
    let numbers = u32::from(u16::MAX).saturating_add(1);
    let mut ordered = Map::with_capacity(numbers);
    let mut planned_documents = Map::with_capacity(numbers);
    // Planned documents keep their slots alongside the required sections.
    let mut needed = 0_u64;
    for section in core {
        if section.required() {
            needed = needed.saturating_add(1);
        }
    }
    for plan in plans {
        let valid = valid_plan(plan, documents, sections);
        let previous = ordered.insert(plan.connector, (plan, valid)).expect("bounded connector numbers");
        assert!(previous.is_none(), "one plan per connector");
        if !valid {
            continue;
        }
        for section in &plan.sections {
            if let Some(index) = section.document {
                let previous = planned_documents
                    .insert(index, Planned::Connector { connector: plan.connector, section: *section })
                    .expect("bounded document numbers");
                assert!(previous.is_none(), "one validated section per document");
                needed = needed.saturating_add(1);
            } else if section.required {
                needed = needed.saturating_add(1);
            }
        }
    }

    let mut dropped = Queue::uncapped();
    let Ok(needed) = u32::try_from(needed) else {
        return no_room(&ordered);
    };
    let Some(mut optional) = sections.checked_sub(needed) else {
        return no_room(&ordered);
    };
    // Optional core sections use the remaining slots from the front, before other offers.
    let mut placed = List::with_capacity(sections);
    for section in core {
        if !section.required() {
            if optional == 0 {
                continue;
            }
            optional = optional.saturating_sub(1);
        }
        placed.push(section.clone()).expect("allotted core slot");
    }
    for (_, section) in &planned_documents {
        placed.push(section.clone()).expect("allotted document slot");
    }
    for (connector, (plan, valid)) in &ordered {
        for section in &plan.sections {
            if !valid || (section.document.is_none() && !section.required && optional == 0) {
                dropped.push(ConnectorAction::Drop { connector: *connector, token: section.token });
            } else if section.document.is_none() {
                if !section.required {
                    optional = optional.saturating_sub(1);
                }
                placed
                    .push(Planned::Connector { connector: *connector, section: *section })
                    .expect("allotted connector slot");
            }
        }
    }
    Placing::Placed { sections: placed.into_boxed(), dropped: dropped.into_boxed() }
}

fn no_room(ordered: &Map<u16, (&ConnectorPlan, bool)>) -> Placing {
    let mut dropped = Queue::uncapped();
    for (connector, (plan, _)) in ordered {
        for section in &plan.sections {
            dropped.push(ConnectorAction::Drop { connector: *connector, token: section.token });
        }
    }
    Placing::Failed {
        failure: Failure {
            fate: Fate::Permanent,
            section: b"brief",
            words: Box::from(b"no room for its required sections".as_slice()),
        },
        dropped: dropped.into_boxed(),
    }
}

fn valid_plan(plan: &ConnectorPlan, documents: &[u16], sections: u32) -> bool {
    if plan.sections.len() > skein_lib::length::usize_of(sections) {
        return false;
    }
    let mut tokens = Set::with_capacity(sections);
    let mut planned_documents = Set::with_capacity(sections);
    for section in &plan.sections {
        if !tokens.insert(section.token).expect("measured plan") {
            return false;
        }
        if let Some(index) = section.document
            && (section.required
                || documents.get(usize::from(index)) != Some(&plan.connector)
                || !planned_documents.insert(index).expect("measured plan"))
        {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::{
        ConnectorAction, ConnectorPlan, Core, Failure, Fate, Placement, Placing, Planned, Section, place, plan,
    };
    use alloc::boxed::Box;
    use skein_lib::Token;

    #[test]
    fn required_sections_claim_budget_before_optional_sections() {
        for (kind, name) in [(Core::Task, b"task".as_slice()), (Core::Preferences, b"preferences")] {
            assert_eq!(super::core_name(kind), name);
            let sections = [
                Planned::Connector {
                    connector: 3,
                    section: Section {
                        name: b"section",
                        kind: 7,
                        token: Token::new(1),
                        size: 80,
                        limit: 80,
                        priority: 0,
                        required: false,
                        document: None,
                    },
                },
                Planned::Core { kind, text: Box::from(&b"task"[..]), limit: 30, priority: 9, required: true },
                Planned::Connector {
                    connector: 4,
                    section: Section {
                        name: b"section",
                        kind: 2,
                        token: Token::new(2),
                        size: 20,
                        limit: 20,
                        priority: 1,
                        required: true,
                        document: None,
                    },
                },
            ];
            let planned = plan(&sections, 30).unwrap();
            assert_eq!(planned.size, 30);
            assert_eq!(
                planned.order.as_ref(),
                [Placement { index: 0, size: 6 }, Placement { index: 1, size: 4 }, Placement { index: 2, size: 20 }]
            );
            assert_eq!(
                planned.connectors.as_ref(),
                [ConnectorAction::CutTo { connector: 3, token: Token::new(1), size: 6 }]
            );
        }
    }

    #[test]
    fn an_optional_connector_without_room_is_dropped_by_token() {
        let sections = [
            Planned::Core { kind: Core::Task, text: Box::from(&b"task"[..]), limit: 4, priority: 0, required: true },
            Planned::Connector {
                connector: 2,
                section: Section {
                    name: b"section",
                    kind: 9,
                    token: Token::new(5),
                    size: 100,
                    limit: 100,
                    priority: 0,
                    required: false,
                    document: None,
                },
            },
        ];
        let planned = plan(&sections, 4).unwrap();
        assert_eq!(planned.order.len(), 1);
        assert_eq!(planned.connectors.as_ref(), [ConnectorAction::Drop { connector: 2, token: Token::new(5) }]);
    }

    #[test]
    fn a_kind_limit_cuts_a_connector_with_room_left_in_the_brief() {
        let token = Token::new(6);
        let sections = [Planned::Connector {
            connector: 2,
            section: Section {
                name: b"section",
                kind: 4,
                token,
                size: 80,
                limit: 40,
                priority: 0,
                required: true,
                document: None,
            },
        }];
        let planned = plan(&sections, 200).expect("one bounded section");
        assert_eq!(planned.size, 40);
        assert_eq!(planned.order.as_ref(), [Placement { index: 0, size: 40 }]);
        assert_eq!(planned.connectors.as_ref(), [ConnectorAction::CutTo { connector: 2, token, size: 40 }]);
    }

    const FIRST: u16 = 2;
    const SECOND: u16 = 3;
    const FIRST_DOCUMENT: u16 = 0;
    const SECOND_DOCUMENT: u16 = 1;
    const OUTSIDE_DOCUMENT: u16 = 2;
    const DOCUMENTS: [u16; 2] = [FIRST, SECOND];
    const SECTION_LIMIT: u32 = 64; // Room for the short fixture text, not used by slot placement.

    const fn section(kind: u16, token: u64, required: bool, document: Option<u16>) -> Section {
        Section {
            token: Token::new(token),
            kind,
            name: b"section",
            size: 0,
            limit: SECTION_LIMIT,
            priority: 1,
            required,
            document,
        }
    }

    const DOC_FIRST: Section = section(1, 1, false, Some(FIRST_DOCUMENT));
    const DOC_SECOND: Section = section(2, 2, false, Some(SECOND_DOCUMENT));
    const CI: Section = section(3, 3, false, None);
    const HOME: Section = section(4, 4, true, None);
    const PULL: Section = section(5, 5, true, None);
    const OTHER: Section = section(6, 6, false, None);

    fn core(kind: Core, required: bool) -> Planned {
        Planned::Core { kind, text: Box::from(b"core".as_slice()), limit: SECTION_LIMIT, priority: 0, required }
    }

    fn offer(connector: u16, sections: &[Section]) -> ConnectorPlan {
        ConnectorPlan { connector, sections: Box::from(sections) }
    }

    fn drop_section(connector: u16, section: Section) -> ConnectorAction {
        ConnectorAction::Drop { connector, token: section.token }
    }

    #[test]
    fn slots_follow_core_then_charter_then_connector_and_plan_order() {
        const SLOTS: u32 = 10; // The seven offered sections fit with room to spare.
        let task = core(Core::Task, true);
        let waiting = core(Core::Waiting, false);
        let core = [task.clone(), waiting.clone()];
        let plans = [offer(SECOND, &[HOME, DOC_SECOND, PULL]), offer(FIRST, &[DOC_FIRST, CI])];
        assert_eq!(
            place(&core, &DOCUMENTS, &plans, SLOTS),
            Placing::Placed {
                sections: Box::new([
                    task,
                    waiting,
                    Planned::Connector { connector: FIRST, section: DOC_FIRST },
                    Planned::Connector { connector: SECOND, section: DOC_SECOND },
                    Planned::Connector { connector: FIRST, section: CI },
                    Planned::Connector { connector: SECOND, section: HOME },
                    Planned::Connector { connector: SECOND, section: PULL },
                ]),
                dropped: Box::new([]),
            }
        );
    }

    fn crowded() -> ([Planned; 3], [ConnectorPlan; 1]) {
        ([core(Core::Task, true), core(Core::Plan, false), core(Core::Attempts, false)], [offer(SECOND, &[HOME, PULL])])
    }

    #[test]
    fn required_connector_sections_displace_the_last_optional_core_section() {
        const SLOTS: u32 = 4; // One optional core section fits beside the three required ones.
        let (core, plans) = crowded();
        let [task, plan, _attempts] = &core;
        assert_eq!(
            place(&core, &[], &plans, SLOTS),
            Placing::Placed {
                sections: Box::new([
                    task.clone(),
                    plan.clone(),
                    Planned::Connector { connector: SECOND, section: HOME },
                    Planned::Connector { connector: SECOND, section: PULL },
                ]),
                dropped: Box::new([]),
            }
        );
    }

    #[test]
    fn required_sections_without_slots_fail_and_drop_every_offer() {
        const SLOTS: u32 = 2; // The plan is within its bound, but Task needs a third slot.
        let (core, plans) = crowded();
        assert_eq!(
            place(&core, &[], &plans, SLOTS),
            Placing::Failed {
                failure: Failure {
                    fate: Fate::Permanent,
                    section: b"brief",
                    words: Box::from(b"no room for its required sections".as_slice()),
                },
                dropped: Box::new([drop_section(SECOND, HOME), drop_section(SECOND, PULL)]),
            }
        );
    }

    #[test]
    fn optional_connectors_take_only_slots_left_after_required_sections() {
        const SLOTS: u32 = 3; // Task, its document and Home leave no slot for CI.
        let task = core(Core::Task, true);
        let core = core::slice::from_ref(&task);
        let plans = [offer(FIRST, &[DOC_FIRST, CI]), offer(SECOND, &[HOME])];
        assert_eq!(
            place(core, &DOCUMENTS, &plans, SLOTS),
            Placing::Placed {
                sections: Box::new([
                    task,
                    Planned::Connector { connector: FIRST, section: DOC_FIRST },
                    Planned::Connector { connector: SECOND, section: HOME },
                ]),
                dropped: Box::new([drop_section(FIRST, CI)]),
            }
        );
    }

    #[test]
    fn an_unplanned_document_leaves_its_slot_free() {
        const SLOTS: u32 = 3; // Task, one planned document and CI fill the brief.
        let task = core(Core::Task, true);
        let core = core::slice::from_ref(&task);
        let plans = [offer(FIRST, &[CI]), offer(SECOND, &[DOC_SECOND])];
        assert_eq!(
            place(core, &DOCUMENTS, &plans, SLOTS),
            Placing::Placed {
                sections: Box::new([
                    task,
                    Planned::Connector { connector: SECOND, section: DOC_SECOND },
                    Planned::Connector { connector: FIRST, section: CI },
                ]),
                dropped: Box::new([]),
            }
        );
    }

    fn assert_plan_dropped(invalid: &[Section], dropped: Box<[ConnectorAction]>, slots: u32) {
        let task = core(Core::Task, true);
        let core = core::slice::from_ref(&task);
        let plans = [offer(FIRST, invalid), offer(SECOND, &[OTHER])];
        assert_eq!(
            place(core, &DOCUMENTS, &plans, slots),
            Placing::Placed {
                sections: Box::new([task, Planned::Connector { connector: SECOND, section: OTHER }]),
                dropped,
            }
        );
    }

    const INVALID_PLAN_SLOTS: u32 = 4; // Task and Other leave two slots even if both invalid offers were accepted.

    #[test]
    fn a_repeated_token_invalidates_the_entire_plan() {
        assert_plan_dropped(
            &[HOME, Section { token: HOME.token, ..CI }],
            Box::new([drop_section(FIRST, HOME), drop_section(FIRST, HOME)]),
            INVALID_PLAN_SLOTS,
        );
    }

    #[test]
    fn another_connectors_document_invalidates_the_entire_plan() {
        assert_plan_dropped(
            &[HOME, DOC_SECOND],
            Box::new([drop_section(FIRST, HOME), drop_section(FIRST, DOC_SECOND)]),
            INVALID_PLAN_SLOTS,
        );
    }

    #[test]
    fn an_out_of_range_document_invalidates_the_entire_plan() {
        assert_plan_dropped(
            &[HOME, Section { document: Some(OUTSIDE_DOCUMENT), ..DOC_FIRST }],
            Box::new([drop_section(FIRST, HOME), drop_section(FIRST, DOC_FIRST)]),
            INVALID_PLAN_SLOTS,
        );
    }

    #[test]
    fn a_repeated_document_invalidates_the_entire_plan() {
        assert_plan_dropped(
            &[DOC_FIRST, Section { document: Some(FIRST_DOCUMENT), ..CI }],
            Box::new([drop_section(FIRST, DOC_FIRST), drop_section(FIRST, CI)]),
            INVALID_PLAN_SLOTS,
        );
    }

    #[test]
    fn a_required_document_invalidates_the_entire_plan() {
        assert_plan_dropped(
            &[HOME, Section { required: true, ..DOC_FIRST }],
            Box::new([drop_section(FIRST, HOME), drop_section(FIRST, DOC_FIRST)]),
            INVALID_PLAN_SLOTS,
        );
    }

    #[test]
    fn a_plan_larger_than_the_brief_is_dropped_whole() {
        assert_plan_dropped(
            &[HOME, DOC_FIRST, CI, PULL, OTHER],
            Box::new([
                drop_section(FIRST, HOME),
                drop_section(FIRST, DOC_FIRST),
                drop_section(FIRST, CI),
                drop_section(FIRST, PULL),
                drop_section(FIRST, OTHER),
            ]),
            INVALID_PLAN_SLOTS,
        );
    }

    #[test]
    fn different_connectors_can_place_sections_with_equal_tokens() {
        const SLOTS: u32 = 3; // Task and both offers fill the brief exactly.
        let task = core(Core::Task, true);
        let core = core::slice::from_ref(&task);
        let pull = Section { token: HOME.token, ..PULL };
        let plans = [offer(FIRST, &[HOME]), offer(SECOND, &[pull])];
        assert_eq!(
            place(core, &[], &plans, SLOTS),
            Placing::Placed {
                sections: Box::new([
                    task,
                    Planned::Connector { connector: FIRST, section: HOME },
                    Planned::Connector { connector: SECOND, section: pull },
                ]),
                dropped: Box::new([]),
            }
        );
    }
}
