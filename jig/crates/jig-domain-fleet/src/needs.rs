//! Agent admission and model eligibility for configured charters (domain/engine.md, section 8).
use crate::{Bounds, Figures, HostKind, Kinds};
use alloc::boxed::Box;
use skein_lib::{Duration, List, length::len_u64};

/// A charter version's name (engine.md, 17.3).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Version {
    pub charter: u32,
    pub revision: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Needs {
    pub charters: Box<[CharterNeeds]>,
    pub spend: u64,
    pub message_bytes: u32,
    pub start_messages: u32,
    pub unacknowledged: u32,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CharterNeeds {
    pub version: Version,
    pub kinds: Kinds,
    pub models: Box<[ModelNeed]>,
    pub turns: u32,
    pub time: Duration,
    pub waiting: Duration,
    pub change_reserve: Option<Duration>,
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ModelNeed {
    pub name: Box<[u8]>,
    pub window: u32,
    pub output: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AgentShortfall {
    Model { charter: u32, model: u32 },
    Window { charter: u32, model: u32 },
    Output { charter: u32, model: u32 },
    Turns { charter: u32 },
    Spend,
    Time { charter: u32 },
    Waiting { charter: u32 },
    Checks { charter: u32 },
    Message,
    Inbox,
    Unacknowledged,
}

pub(crate) fn owned_bytes(needs: &Needs) -> Option<u64> {
    let mut bytes = len_u64(needs.charters.len()).checked_mul(u64::try_from(size_of::<CharterNeeds>()).ok()?)?;
    for charter in &needs.charters {
        bytes = bytes
            .checked_add(len_u64(charter.models.len()).checked_mul(u64::try_from(size_of::<ModelNeed>()).ok()?)?)?;
        for model in &charter.models {
            bytes = bytes.checked_add(len_u64(model.name.len()))?;
        }
    }
    Some(bytes)
}

/// The first agent bound that cannot hold this host kind's configured charters.
///
/// Checks applicable charters in their configured order, then deployment-wide
/// bounds if any charter applies. Missing models narrow placement eligibility;
/// only models the host declares have their window and output checked here.
/// This is the same decision used for hello admission (domain/engine.md, section 8).
#[must_use]
pub fn shortfall(needs: &Needs, bounds: &Bounds, kind: HostKind) -> Option<AgentShortfall> {
    let mut applicable = false;
    for charter in &needs.charters {
        if !charter.kinds.allows(kind) {
            continue;
        }
        applicable = true;
        if let Some(why) = charter_shortfall(charter, bounds) {
            return Some(why);
        }
    }
    if applicable {
        if bounds.spend < needs.spend {
            return Some(AgentShortfall::Spend);
        }
        if bounds.message_bytes < needs.message_bytes {
            return Some(AgentShortfall::Message);
        }
        if bounds.inbox < needs.start_messages {
            return Some(AgentShortfall::Inbox);
        }
        if bounds.unacknowledged < needs.unacknowledged {
            return Some(AgentShortfall::Unacknowledged);
        }
    }
    None
}

/// The first shortfall within one charter version, in admission order.
fn charter_shortfall(charter: &CharterNeeds, bounds: &Bounds) -> Option<AgentShortfall> {
    for (index, need) in charter.models.iter().enumerate() {
        let number = u32::try_from(index).expect("configured model count");
        let mut held = None;
        for model in &bounds.models {
            if model.name == need.name {
                held = Some(model);
                break;
            }
        }
        let Some(held) = held else { continue };
        if held.window < need.window {
            return Some(AgentShortfall::Window { charter: charter.version.charter, model: number });
        }
        if held.output < need.output {
            return Some(AgentShortfall::Output { charter: charter.version.charter, model: number });
        }
    }
    if bounds.turns < charter.turns {
        return Some(AgentShortfall::Turns { charter: charter.version.charter });
    }
    if bounds.time < charter.time {
        return Some(AgentShortfall::Time { charter: charter.version.charter });
    }
    if bounds.waiting < charter.waiting {
        return Some(AgentShortfall::Waiting { charter: charter.version.charter });
    }
    if let Some(reserve) = charter.change_reserve
        && bounds.checks > reserve
    {
        return Some(AgentShortfall::Checks { charter: charter.version.charter });
    }
    None
}

/// The amounts of a shortfall `shortfall` selected for this host kind: None
/// for `Model`, which has none, or a `why` no version `kind` allows gives.
#[must_use]
pub fn figures(needs: &Needs, bounds: &Bounds, kind: HostKind, why: AgentShortfall) -> Option<Figures> {
    let amounts = match why {
        AgentShortfall::Model { .. } => return None,
        AgentShortfall::Window { model, .. } => {
            return model_figures(charter_need(needs, bounds, kind, why)?, bounds, model, false);
        }
        AgentShortfall::Output { model, .. } => {
            return model_figures(charter_need(needs, bounds, kind, why)?, bounds, model, true);
        }
        AgentShortfall::Turns { .. } => {
            (u64::from(charter_need(needs, bounds, kind, why)?.turns), u64::from(bounds.turns))
        }
        AgentShortfall::Time { .. } => {
            (charter_need(needs, bounds, kind, why)?.time.as_millis(), bounds.time.as_millis())
        }
        AgentShortfall::Waiting { .. } => {
            (charter_need(needs, bounds, kind, why)?.waiting.as_millis(), bounds.waiting.as_millis())
        }
        AgentShortfall::Checks { .. } => {
            (charter_need(needs, bounds, kind, why)?.change_reserve?.as_millis(), bounds.checks.as_millis())
        }
        AgentShortfall::Spend => (needs.spend, bounds.spend),
        AgentShortfall::Message => (u64::from(needs.message_bytes), u64::from(bounds.message_bytes)),
        AgentShortfall::Inbox => (u64::from(needs.start_messages), u64::from(bounds.inbox)),
        AgentShortfall::Unacknowledged => (u64::from(needs.unacknowledged), u64::from(bounds.unacknowledged)),
    };
    Some(Figures { required: amounts.0, offered: amounts.1 })
}

#[expect(clippy::manual_find, reason = "step code scans without closures")]
fn charter_need<'a>(
    needs: &'a Needs,
    bounds: &Bounds,
    kind: HostKind,
    why: AgentShortfall,
) -> Option<&'a CharterNeeds> {
    for charter in &needs.charters {
        if charter.kinds.allows(kind) && charter_shortfall(charter, bounds) == Some(why) {
            return Some(charter);
        }
    }
    None
}

fn model_figures(charter: &CharterNeeds, bounds: &Bounds, model: u32, output: bool) -> Option<Figures> {
    let need = charter.models.get(usize::try_from(model).ok()?)?;
    for held in &bounds.models {
        if held.name == need.name {
            return Some(if output {
                Figures { required: u64::from(need.output), offered: u64::from(held.output) }
            } else {
                Figures { required: u64::from(need.window), offered: u64::from(held.window) }
            });
        }
    }
    None
}

/// One eligibility bit per configured charter, bounded by `Needs`' owned bytes.
/// The parent admits charter numbers; an unconfigured number has no needs here.
pub(crate) fn eligibility(needs: &Needs, models: &[crate::ModelBound]) -> Box<[bool]> {
    let mut eligible = List::with_capacity(u32::try_from(needs.charters.len()).expect("configured charter count"));
    for charter in &needs.charters {
        let mut all = true;
        for need in &charter.models {
            let mut declared = false;
            for model in models {
                if model.name == need.name {
                    declared = true;
                    break;
                }
            }
            if !declared {
                all = false;
                break;
            }
        }
        eligible.push(all).expect("one bit per configured charter");
    }
    eligible.into_boxed()
}

pub(crate) fn allows(needs: &Needs, eligible: &[bool], version: Version) -> bool {
    let mut known = false;
    for (index, need) in needs.charters.iter().enumerate() {
        known |= need.version.charter == version.charter;
        if need.version == version {
            return *eligible.get(index).expect("one bit per configured charter");
        }
    }
    !known
}

/// Declared models in the configured needs' order, without retaining peer-owned names.
pub(crate) fn declarations(needs: &Needs, models: &[crate::ModelBound]) -> Box<[bool]> {
    let mut count = 0_u32;
    for charter in &needs.charters {
        count = count.checked_add(u32::try_from(charter.models.len()).expect("model count")).expect("model count");
    }
    let mut result = List::with_capacity(count);
    for charter in &needs.charters {
        for need in &charter.models {
            let mut declared = false;
            for model in models {
                if model.name == need.name && model.window >= need.window && model.output >= need.output {
                    declared = true;
                }
            }
            result.push(declared).expect("one bit per model need");
        }
    }
    result.into_boxed()
}

pub(crate) fn allows_models(needs: &Needs, declarations: &[bool], models: &[ModelNeed]) -> bool {
    for model in models {
        let mut at = 0_usize;
        let mut found = false;
        for charter in &needs.charters {
            for configured in &charter.models {
                if configured.name == model.name
                    && configured.window >= model.window
                    && configured.output >= model.output
                    && declarations.get(at) == Some(&true)
                {
                    found = true;
                }
                at = at.checked_add(1).expect("bounded configured model index");
            }
        }
        if !found {
            return false;
        }
    }
    true
}

/// The same catalogue bounds as the core's configured models (configuration.md, 6).
pub const CATALOGUE_MODELS: u32 = 64;
pub const CATALOGUE_NAME_BYTES: u32 = 4096;

pub(crate) fn empty_evidence() -> Bounds {
    Bounds {
        program_endpoints: Box::new([]),
        models: Box::new([]),
        turns: 0,
        spend: 0,
        time: Duration::ZERO,
        waiting: Duration::ZERO,
        checks: Duration::ZERO,
        message_bytes: 0,
        inbox: 0,
        unacknowledged: 0,
    }
}
pub(crate) fn evidence(catalogue: &[Box<[u8]>], bounds: &Bounds) -> Bounds {
    let mut result = empty_evidence();
    result.turns = bounds.turns;
    result.spend = bounds.spend;
    result.time = bounds.time;
    result.waiting = bounds.waiting;
    result.checks = bounds.checks;
    result.message_bytes = bounds.message_bytes;
    result.inbox = bounds.inbox;
    result.unacknowledged = bounds.unacknowledged;
    let mut models = List::with_capacity(CATALOGUE_MODELS);
    for name in catalogue {
        for model in &bounds.models {
            if *name == model.name {
                models.push(model.clone()).expect("catalogue model bound");
                break;
            }
        }
    }
    result.models = models.into_boxed();
    result
}
pub(crate) fn replace(domain: &mut crate::Domain, limits: &crate::Limits, needs: Needs) {
    assert!(
        match owned_bytes(&needs) {
            Some(bytes) => bytes <= crate::limits::needs_bytes(limits, domain.versions).expect("needs bound"),
            None => false,
        },
        "replacement needs within their bound"
    );
    domain.needs = needs;
    for (_, id) in &domain.tokens {
        let channel = domain.channels.get_mut(*id).expect("connected channel");
        channel.eligible = eligibility(&domain.needs, &channel.evidence.models);
        channel.declarations = declarations(&domain.needs, &channel.evidence.models);
        for (at, charter) in domain.needs.charters.iter().enumerate() {
            let bounds = &channel.evidence;
            let mut fits = bounds.turns >= charter.turns
                && bounds.time >= charter.time
                && bounds.waiting >= charter.waiting
                && bounds.spend >= domain.needs.spend
                && bounds.message_bytes >= domain.needs.message_bytes
                && bounds.inbox >= domain.needs.start_messages
                && bounds.unacknowledged >= domain.needs.unacknowledged;
            if let Some(reserve) = charter.change_reserve {
                fits &= bounds.checks <= reserve;
            }
            for need in &charter.models {
                for model in &bounds.models {
                    if model.name == need.name {
                        fits &= model.window >= need.window && model.output >= need.output;
                    }
                }
            }
            let eligible = channel.eligible.get_mut(at).expect("one bit per charter");
            *eligible &= fits;
        }
    }
    domain.placing = true;
}

/// The selected version's host kinds and non-model bounds. Probes check only
/// their retained models separately, so an unused alternative cannot block them.
pub(crate) fn fits(needs: &Needs, version: Version, bounds: &Bounds, kind: HostKind) -> bool {
    let mut known = false;
    for charter in &needs.charters {
        known |= charter.version.charter == version.charter;
        if charter.version != version {
            continue;
        }
        return charter.kinds.allows(kind)
            && bounds.turns >= charter.turns
            && bounds.time >= charter.time
            && bounds.waiting >= charter.waiting
            && bounds.spend >= needs.spend
            && bounds.message_bytes >= needs.message_bytes
            && bounds.inbox >= needs.start_messages
            && bounds.unacknowledged >= needs.unacknowledged
            && match charter.change_reserve {
                Some(reserve) => bounds.checks <= reserve,
                None => true,
            };
    }
    // The parent admits charter versions. An unconfigured charter has no needs.
    !known
}
