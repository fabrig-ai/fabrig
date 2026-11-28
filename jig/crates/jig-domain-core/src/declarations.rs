//! Construction-time connector kinds (jig/docs/design/domain/engine/vocabulary.md, 2.1).
use crate::Core;
use alloc::boxed::Box;
use jig_domain_contract::{Connector, Declaration, StartProblem, effects::EffectKind};
use jig_domain_tasks as tasks;
use skein_lib::{Env, List, Queue, Set, Time, Wall};

impl Core {
    pub(crate) fn effect_kind(&self, connector: u16, kind: u16) -> Option<&EffectKind> {
        for declaration in &self.declarations {
            if declaration.connector == Connector(connector) {
                for effect in &declaration.effects {
                    if effect.kind == kind {
                        return Some(effect);
                    }
                }
                return None;
            }
        }
        None
    }
}

/// Broadcast recipients derived once from configuration and declarations.
#[derive(Hash, Debug)]
pub(crate) struct Recipients {
    pub(crate) resources: Box<[u16]>,
    pub(crate) effects: Box<[u16]>,
    pub(crate) projections: Box<[u16]>,
    pub(crate) sections: Set<u16>,
}

pub(crate) fn recipients(configured: &[u16], declarations: &[Declaration]) -> Recipients {
    let count = u32::try_from(configured.len()).expect("bounded connectors");
    let mut resources = List::with_capacity(count);
    let mut effects = List::with_capacity(count);
    let mut projections = List::with_capacity(count);
    let mut sections = Set::with_capacity(count);
    for &number in configured {
        for declaration in declarations {
            if declaration.connector != Connector(number) {
                continue;
            }
            if !declaration.resources.is_empty() {
                resources.push(number).expect("configured connector bound");
            }
            if !declaration.effects.is_empty() {
                effects.push(number).expect("configured connector bound");
            }
            if declaration.projects_goals {
                projections.push(number).expect("configured connector bound");
            }
            if declaration.sections {
                sections.insert(number).expect("configured connector bound");
            }
            break;
        }
    }
    Recipients {
        resources: resources.into_boxed(),
        effects: effects.into_boxed(),
        projections: projections.into_boxed(),
        sections,
    }
}

// A kind number is a u16, so this set can hold every distinct number.
const NUMBERS: u32 = 65_536;

pub(crate) fn check(declarations: &[Declaration], configured: &[u16], limit: u32) -> Result<(), StartProblem> {
    let mut named = Set::with_capacity(NUMBERS);
    let mut resources = 0_usize;
    for declaration in declarations {
        let connector = declaration.connector;
        if !configured.contains(&connector.0) || !named.insert(connector).expect("all connector numbers fit") {
            return Err(StartProblem::Declared { connector });
        }
        let mut kinds = Set::with_capacity(NUMBERS);
        for resource in &declaration.resources {
            if !kinds.insert(resource.kind).expect("all resource kind numbers fit") {
                return Err(StartProblem::ResourceKind { connector, kind: resource.kind });
            }
        }
        let mut kinds = Set::with_capacity(NUMBERS);
        for effect in &declaration.effects {
            if !kinds.insert(effect.kind).expect("all effect kind numbers fit") {
                return Err(StartProblem::EffectKind { connector, kind: effect.kind });
            }
        }
        resources = resources.saturating_add(declaration.resources.len());
    }
    for &number in configured {
        let connector = Connector(number);
        if !named.contains(&connector) {
            return Err(StartProblem::Undeclared { connector });
        }
    }
    if resources > skein_lib::length::usize_of(limit) {
        return Err(StartProblem::HoldKinds { limit });
    }
    Ok(())
}

pub(crate) fn install(domain: &mut tasks::Domain, limits: &tasks::Limits, declarations: &[Declaration]) {
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: *limits };
    let mut out = Queue::uncapped();
    for declaration in declarations {
        tasks::step(
            domain,
            &env,
            tasks::Event::Kinds { connector: declaration.connector.0, kinds: declaration.resources.clone() },
            &mut out,
        );
    }
    assert!(out.is_empty(), "declared hold rules decide nothing");
}
