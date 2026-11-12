//! A charter's brief room, by its host part's costs
//! (temper/docs/design/domain/agent/tools.md).

use alloc::boxed::Box;
use jig_domain_core::{Charter, ToolFamilies};
use skein_lib::{Decimal, Writer};

/// A run's least brief room (temper/docs/design/domain/agent/tools.md).
pub const BRIEF_ROOM: u64 = 2_048;

/// A charter's non-brief host part in smith's bytes, built by temper-engine-smith.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HostPart {
    /// Smith's host part in the receiving profile.
    pub bound: u64,
    /// Smith's own text and the largest finish and deliver schemas.
    pub fixed: u64,
    /// Host tools declared whatever the charter's tools, at the largest offer.
    pub declared: u64,
    /// Settings tools, paid where the charter names their family.
    pub settings: u64,
    /// The asking family's see and document declarations.
    pub asking: u64,
    /// The asking form of `propose_settings`, paid with the settings family.
    pub asking_settings: u64,
    /// Smith's families, paid where the policy turns them on.
    pub inspect: u64,
    pub modify: u64,
    pub shell: u64,
    pub agents: u64,
}

/// A charter that leaves its brief less than [`BRIEF_ROOM`].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Short {
    pub charter: u32,
    /// Zero where the non-brief part alone passes the bound.
    pub room: u64,
    pub bound: u64,
}

impl HostPart {
    /// Fixed parts, declarations and the instructions with every charter part joined,
    /// with each enabled family.
    #[must_use]
    pub fn non_brief(&self, charter: &Charter) -> u64 {
        let policy = &charter.policy;
        let asking = crate::asking(policy.tools);
        let mut bytes = self.fixed.saturating_add(if asking { self.asking } else { self.declared });
        let mut instructions = skein_lib::len_u64(policy.instructions.len());
        let mut newline = policy.instructions.ends_with(b"\n");
        for part in &charter.parts {
            instructions = instructions.saturating_add(2).saturating_add(skein_lib::len_u64(part.instructions.len()));
            newline = part.instructions.is_empty() || part.instructions.ends_with(b"\n");
        }
        if instructions != 0 {
            bytes = bytes.saturating_add(instructions);
            // Smith ends the joined literal instructions with a blank line.
            bytes = bytes.saturating_add(if newline { 1 } else { 2 });
        }
        for (enabled, cost) in [
            (
                policy.tools.0 & ToolFamilies::JIG.settings != 0,
                if asking { self.asking_settings } else { self.settings },
            ),
            (policy.inspect, self.inspect),
            (policy.modify, self.modify),
            (policy.shell, self.shell),
            (policy.agents, self.agents),
        ] {
            if enabled {
                bytes = bytes.saturating_add(cost);
            }
        }
        if policy.agents && !policy.alternatives.is_empty() {
            // Smith counts the alternative-model offer as host text; the main model and efforts add none.
            bytes = bytes.saturating_add(skein_lib::len_u64(b"run's main LLM unless you name one of these: ".len()));
            let mut rest = policy.alternatives.len();
            for model in &policy.alternatives {
                bytes = bytes.saturating_add(skein_lib::len_u64(model.name.len())).saturating_add(2);
                rest = rest.saturating_sub(1);
                bytes = bytes.saturating_add(match rest {
                    0 => 0,
                    1 => 4,
                    _ => 2,
                });
            }
            bytes = bytes.saturating_add(skein_lib::len_u64(b".\n".len()));
        }
        bytes
    }

    /// The host bound less the non-brief part, zero at or past the bound.
    #[must_use]
    pub fn room(&self, charter: &Charter) -> u64 {
        self.bound.saturating_sub(self.non_brief(charter))
    }

    /// Refuse a charter whose brief has less than its least room.
    pub fn check(&self, charter: &Charter) -> Result<(), Short> {
        let room = self.room(charter);
        if room < BRIEF_ROOM { Err(Short { charter: charter.number, room, bound: self.bound }) } else { Ok(()) }
    }
}

impl Short {
    /// Name the charter, its room and bound, with thousands' commas.
    #[must_use]
    pub fn words(&self) -> Box<[u8]> {
        let charter = Decimal::of(u64::from(self.charter));
        let room = Decimal::of(self.room);
        let bound = Decimal::of(self.bound);
        let parts: [&[u8]; 7] = [
            b"charter ",
            charter.as_bytes(),
            b" leaves its brief ",
            room.as_bytes(),
            b" bytes of its host part's ",
            bound.as_bytes(),
            b"; it needs at least 2,048",
        ];
        let mut length = 0_usize;
        for part in parts {
            length = length.checked_add(part.len()).expect("bounded refusal text");
        }
        for digits in [charter.as_bytes(), room.as_bytes(), bound.as_bytes()] {
            length = length
                .checked_add(digits.len().saturating_sub(1).checked_div(3).expect("nonzero divisor"))
                .expect("bounded comma count");
        }
        let mut out = Writer::new(length);
        for (index, part) in parts.into_iter().enumerate() {
            if index == 1 || index == 3 || index == 5 {
                for (offset, digit) in part.iter().enumerate() {
                    if offset != 0 && part.len().saturating_sub(offset).is_multiple_of(3) {
                        out.put(b",").expect("measured comma");
                    }
                    out.put(core::slice::from_ref(digit)).expect("measured digit");
                }
            } else {
                out.put(part).expect("measured words");
            }
        }
        out.finish()
    }
}

/// Trials fit their accounts, the family wire and one whole delivery (domain/trial.md, 3.1 and 6).
pub(crate) fn trial_rows(limits: &crate::Limits) -> u32 {
    let row = u32::try_from(size_of::<temper_engine_domain_trials::Row>()).expect("trial row size fits u32");
    let room = limits
        .journal
        .result_bytes
        .max(limits.journal.transcript_bytes)
        .checked_div(row)
        .expect("a trial row has fields");
    limits.accounts.accounts.min(64).min(room)
}

/// Approvals fit the wire and one whole delivery at the largest row (domain/forge/change/questions.md, 2).
pub(crate) fn approval_rows(limits: &crate::Limits) -> u32 {
    let row = u32::try_from(size_of::<crate::ApprovalRow>())
        .expect("approval row size fits u32")
        .saturating_add(limits.forge.name_bytes.saturating_mul(2))
        .saturating_add(u32::try_from(temper_engine_domain_forge::LOGGED_TITLE_BYTES).expect("bounded title"))
        .saturating_add(
            u32::try_from(size_of::<crate::ApprovalSet>())
                .expect("approval set size fits u32")
                .saturating_add(40)
                .saturating_mul(6),
        );
    limits
        .journal
        .result_bytes
        .max(limits.journal.transcript_bytes)
        .checked_div(row)
        .expect("an approval row has fields")
        .min(32)
}

#[cfg(test)]
#[path = "tests/room.rs"]
mod tests;
