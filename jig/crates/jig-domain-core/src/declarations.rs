//! Construction-time connector kinds (jig/docs/design/domain/engine/vocabulary.md, 2.1).
use crate::Core;
use jig_domain_contract::{Connector, Declaration, StartProblem, effects::EffectKind};
use jig_domain_tasks as tasks;
use skein_lib::{Env, Queue, Set, Time, Wall};

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
