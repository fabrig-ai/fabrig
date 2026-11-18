//! A charter's brief room, by its host part's costs
//! (temper/docs/design/domain/agent/room.md, 4).

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::boxed::Box;
use jig_domain_core::{Charter, ToolFamilies};
use jig_domain_tasks::{REASON_NAMES, Reasons};
use skein_lib::{Decimal, Writer};

/// A run's least brief room (temper/docs/design/domain/agent/tools.md).
pub const BRIEF_ROOM: u64 = 2_048;

/// One cost in smith's three measures (domain/agent/room.md, 4.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Cost {
    /// Rendered host bytes, against the local agent's host bound.
    pub rendered: u64,
    /// Decoded charter ownership, against its allowance.
    pub owned: u64,
    /// Encoded charter bytes, against its allowance.
    pub encoded: u64,
}

impl Cost {
    pub const ZERO: Self = Self { rendered: 0, owned: 0, encoded: 0 };
    #[must_use]
    pub const fn plus(self, other: Self) -> Self {
        Self {
            rendered: self.rendered.saturating_add(other.rendered),
            owned: self.owned.saturating_add(other.owned),
            encoded: self.encoded.saturating_add(other.encoded),
        }
    }
}

/// Measured declarations and frame, and their receiving bounds (domain/agent/room.md, 4.4).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct HostPart {
    /// Smith's host part in the receiving profile; none without a local agent.
    pub bound: Option<u64>,
    /// The smaller of the hub's and the receiving worker's charter allowance.
    pub allowance: u64,
    /// Rendered mechanics; owned and encoded frame with the main name and effort empty.
    pub fixed: Cost,
    /// One alternative model beyond its name and effort payloads.
    pub model: Cost,
    /// Non-asking declarations offered whatever the charter's families.
    pub always: Cost,
    /// `ToolFamilies::JIG`'s bit order, delegate first, ask last.
    pub families: [Cost; 12],
    /// An asking charter's own declarations.
    pub asking: Cost,
    /// The asking form of `propose_settings`.
    pub asking_settings: Cost,
    /// Smith's rendered families, paid where the policy turns them on.
    pub inspect: u64,
    pub modify: u64,
    pub shell: u64,
    pub agents: u64,
    /// The brief's six sections at their profile budgets, owned and encoded with titles and framing; rendered zero.
    pub floor: Cost,
}

/// The fixed part's first failing measure (domain/agent/room.md, 4.3).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Measure {
    Owned,
    Encoded,
}

/// A fixed part past its allowance or leaving less than the brief's floor.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Unfit {
    pub charter: u32,
    pub wake: Reasons,
    pub measure: Measure,
    pub bytes: u64,
    pub allowance: u64,
    /// The floor in the failing measure; zero where the fixed part passed the allowance itself.
    pub floor: u64,
}

/// Fixed-part admission precedes rendered-room admission.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Refusal {
    Fixed(Unfit),
    Rendered(Short),
}

impl Refusal {
    #[must_use]
    pub fn words(&self) -> Box<[u8]> {
        match self {
            Self::Fixed(unfit) => unfit.words(),
            Self::Rendered(short) => short.words(),
        }
    }
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
    /// An empty measurement table, before the service fills it and sets its allowance.
    pub const EMPTY: Self = Self {
        bound: None,
        allowance: 0,
        fixed: Cost::ZERO,
        model: Cost::ZERO,
        always: Cost::ZERO,
        families: [Cost::ZERO; 12],
        asking: Cost::ZERO,
        asking_settings: Cost::ZERO,
        inspect: 0,
        modify: 0,
        shell: 0,
        agents: 0,
        floor: Cost::ZERO,
    };

    /// Fixed parts, declarations and the instructions with every charter part joined,
    /// with each enabled family.
    #[must_use]
    pub fn non_brief(&self, charter: &Charter) -> u64 {
        let policy = &charter.policy;
        let mut bytes = self.fixed.rendered.saturating_add(self.declarations(charter).rendered);
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
        self.bound.unwrap_or(u64::MAX).saturating_sub(self.non_brief(charter))
    }

    /// Costs offered at the widest authority, without instructions or models.
    fn declarations(&self, charter: &Charter) -> Cost {
        if asking(charter.policy.tools) {
            return if charter.policy.tools.0 & ToolFamilies::JIG.settings != 0 {
                self.asking.plus(self.asking_settings)
            } else {
                self.asking
            };
        }
        let mut cost = self.always;
        let mut bit = 1_u64;
        for family in self.families {
            if charter.policy.tools.0 & bit != 0 {
                cost = cost.plus(family);
            }
            bit = bit.checked_shl(1).expect("twelve family bits");
        }
        cost
    }

    /// Owned and encoded costs on one wake, including resumable parts as standing sessions read them.
    /// The rendered cost retains its separate all-parts admission rules.
    #[must_use]
    pub fn on_wake(&self, charter: &Charter, wake: Reasons) -> Cost {
        let policy = &charter.policy;
        let mut bytes = skein_lib::len_u64(policy.instructions.len());
        for part in &charter.parts {
            if part.on.meets(wake) {
                bytes = bytes.saturating_add(2).saturating_add(skein_lib::len_u64(part.instructions.len()));
            }
        }
        let mut cost = self.fixed.plus(self.declarations(charter));
        for model in core::iter::once(&policy.model).chain(policy.alternatives.iter()) {
            bytes = bytes.saturating_add(skein_lib::len_u64(model.name.len()));
            if let Some(effort) = &model.effort {
                bytes = bytes.saturating_add(skein_lib::len_u64(effort.len()));
            }
        }
        for _ in &policy.alternatives {
            cost = cost.plus(self.model);
        }
        cost = cost.plus(Cost { rendered: 0, owned: bytes, encoded: bytes });
        cost.rendered = self.non_brief(charter);
        cost
    }

    /// Refuse the largest wake in the first failing measure, with ties in Reasons' order.
    pub fn check_fixed(&self, charter: &Charter) -> Result<(), Unfit> {
        let mut owned = (Reasons::FIRST, 0);
        let mut encoded = (Reasons::FIRST, 0);
        for name in REASON_NAMES {
            let wake = Reasons::named(name).expect("known reason name");
            let cost = self.on_wake(charter, wake);
            if cost.owned > owned.1 {
                owned = (wake, cost.owned);
            }
            if cost.encoded > encoded.1 {
                encoded = (wake, cost.encoded);
            }
        }
        for (measure, (wake, bytes)) in [(Measure::Owned, owned), (Measure::Encoded, encoded)] {
            if bytes > self.allowance {
                return Err(Unfit {
                    charter: charter.number,
                    wake,
                    measure,
                    bytes,
                    allowance: self.allowance,
                    floor: 0,
                });
            }
        }
        for (measure, (wake, bytes), floor) in
            [(Measure::Owned, owned, self.floor.owned), (Measure::Encoded, encoded, self.floor.encoded)]
        {
            if self.allowance.saturating_sub(bytes) < floor {
                return Err(Unfit { charter: charter.number, wake, measure, bytes, allowance: self.allowance, floor });
            }
        }
        Ok(())
    }

    /// Restore checks rendered room alone; no local agent skips it.
    pub fn check_rendered(&self, charter: &Charter) -> Result<(), Short> {
        let Some(bound) = self.bound else { return Ok(()) };
        let room = self.room(charter);
        if room < BRIEF_ROOM { Err(Short { charter: charter.number, room, bound }) } else { Ok(()) }
    }

    /// Both checks at settings entrances and over startup defaults.
    pub fn check(&self, charter: &Charter) -> Result<(), Refusal> {
        if let Err(unfit) = self.check_fixed(charter) {
            return Err(Refusal::Fixed(unfit));
        }
        match self.check_rendered(charter) {
            Ok(()) => Ok(()),
            Err(short) => Err(Refusal::Rendered(short)),
        }
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
        words(&parts, &[1, 3, 5])
    }
}

impl Unfit {
    /// The fixed-part diagnostic (domain/agent/room.md, 4.5).
    #[must_use]
    pub fn words(&self) -> Box<[u8]> {
        let charter = Decimal::of(u64::from(self.charter));
        let bytes = Decimal::of(self.bytes);
        let allowance = Decimal::of(self.allowance);
        let mut wake = b"first".as_slice();
        for name in REASON_NAMES {
            if Reasons::named(name) == Some(self.wake) {
                wake = name;
            }
        }
        let measure: &[u8] = match self.measure {
            Measure::Owned => b"owned",
            Measure::Encoded => b"encoded",
        };
        if self.floor != 0 {
            let left = Decimal::of(self.allowance.saturating_sub(self.bytes));
            let floor = Decimal::of(self.floor);
            return words(
                &[
                    b"charter ",
                    charter.as_bytes(),
                    b" on a ",
                    wake,
                    b" wake is ",
                    bytes.as_bytes(),
                    b" bytes ",
                    measure,
                    b" before its brief, leaving its brief ",
                    left.as_bytes(),
                    b" of the ",
                    floor.as_bytes(),
                    b" its floor needs; trim its text, parts or tool families",
                ],
                &[1, 5, 9, 11],
            );
        }
        words(
            &[
                b"charter ",
                charter.as_bytes(),
                b" on a ",
                wake,
                b" wake is ",
                bytes.as_bytes(),
                b" bytes ",
                measure,
                b" before its brief, past its allowance of ",
                allowance.as_bytes(),
                b"; trim its text, parts or tool families",
            ],
            &[1, 5, 9],
        )
    }
}

fn words(parts: &[&[u8]], figures: &[usize]) -> Box<[u8]> {
    let mut length = 0_usize;
    for (index, part) in parts.iter().enumerate() {
        length = length.checked_add(part.len()).expect("bounded refusal text");
        if figures.contains(&index) {
            length = length
                .checked_add(part.len().saturating_sub(1).checked_div(3).expect("nonzero divisor"))
                .expect("bounded comma count");
        }
    }
    let mut out = Writer::new(length);
    for (index, part) in parts.iter().enumerate() {
        if figures.contains(&index) {
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

/// Whether a charter offers only the asking family's tools and optional settings
/// (temper/docs/design/domain/agent/tools.md).
#[must_use]
pub fn asking(tools: jig_domain_tasks::Tools) -> bool {
    let families = ToolFamilies::JIG;
    tools.0 & families.ask.0 != 0 && tools.0 & !(families.ask.0 | families.settings) == 0
}
