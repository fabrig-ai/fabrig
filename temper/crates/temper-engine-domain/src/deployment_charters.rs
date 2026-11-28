//! The room a deployment's named charters leave a project's words
//! (temper/docs/design/protocol/temper-dir.md, 8.1).
use alloc::boxed::Box;
use jig_domain_core::{Charter, Core, settings::Setting};
use jig_domain_core_vocabulary::limits::{parts_owned_bytes, policy_owned_bytes};
use jig_domain_tasks::{REASON_NAMES, Reasons};
use skein_lib::List;
use temper_engine_domain_forge::{CharterRoom, DeploymentCharters};

use crate::room::{BRIEF_ROOM, HostPart};
use crate::{Domain, Limits};

impl Domain {
    pub(crate) fn give_deployment_charters(&mut self) {
        self.forge.give_deployment_charters(snapshot(self.engine.core(), &self.limits, &self.host_part));
    }
}

pub(crate) fn snapshot(core: &Core, limits: &Limits, host: &HostPart) -> DeploymentCharters {
    let bounds = crate::limits::core_limits(limits);
    let numbers = core.current.offered_charters();
    let mut offered = List::with_capacity(u32::try_from(numbers.len()).expect("bounded charter numbers"));
    for number in numbers {
        let Some(charter) = core.charter(number) else { continue };
        let Some(name) = &charter.name else { continue };
        let mut base = charter.clone();
        if let Some(overlay) = core.current.overlay(Setting::Charter(number)) {
            base.policy.instructions.clone_from(&overlay.instructions);
            base.parts.clone_from(&overlay.parts);
        } else {
            base.policy.instructions = Box::new([]);
            base.parts = Box::new([]);
        }
        let held = u32::try_from(base.parts.len()).expect("bounded overlay parts");
        let parts = bounds.tasks.charter_parts.saturating_sub(held);
        let bytes = if held > bounds.tasks.charter_parts { None } else { word_room(&base, bounds.run_bytes, host) };
        offered.push(CharterRoom { name: name.clone(), bytes, parts }).expect("bounded offered charters");
    }
    DeploymentCharters { offered: offered.into_boxed(), limit: bounds.tasks.charters }
}

fn word_room(base: &Charter, run_bytes: u32, host: &HostPart) -> Option<u64> {
    // All project words could be instructions. Reserve the join to the
    // overlay in that case; parts already pay their own joins in Jig's count.
    let join = if base.policy.instructions.is_empty() { 0 } else { 2 };
    let jig = policy_owned_bytes(&base.policy)?.checked_add(parts_owned_bytes(&base.parts)?)?;
    let mut room = u64::from(run_bytes).checked_sub(jig)?.saturating_sub(join);

    let owned = host.allowance.checked_sub(host.floor.owned)?;
    let encoded = host.allowance.checked_sub(host.floor.encoded)?;
    for name in REASON_NAMES {
        let cost = host.on_wake(base, Reasons::named(name).expect("jig's reason name"));
        room = room.min(owned.checked_sub(cost.owned)?.saturating_sub(join));
        room = room.min(encoded.checked_sub(cost.encoded)?.saturating_sub(join));
        if base.parts.is_empty() {
            break;
        }
    }
    if let Some(bound) = host.bound {
        // Smith terminates nonempty instructions with a blank line. If the
        // overlay has a tail, it is already charged. A project's part can
        // change that tail by a byte, less than the part's owned overhead.
        let tail = if base.policy.instructions.is_empty() && base.parts.is_empty() { 2 } else { 0 };
        let rendered = bound.checked_sub(BRIEF_ROOM)?.checked_sub(host.non_brief(base))?;
        room = room.min(rendered.saturating_sub(join).saturating_sub(tail));
    }
    Some(room)
}
