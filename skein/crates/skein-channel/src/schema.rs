//! Bounded protocol tables checked before use (net/channel.md, section 4).
//! These tables keep wire numbers and size limits, but never body semantics.
use skein_lib::List;

/// Which peer owns one end of a channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// The peer that sends Open.
    Initiator,
    /// The peer that receives Open.
    Responder,
}

/// Which peer sends a kind in its version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    /// A kind sent by the initiator.
    FromInitiator,
    /// A kind sent by the responder.
    FromResponder,
}

/// One application kind and its maximum body in one version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Kind {
    pub kind: u16,
    pub direction: Direction,
    pub largest: u32,
}

/// An application's magic, frozen wire version and bounded application kinds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Schema {
    pub magic: [u8; 4],
    pub version: u16,
    pub kinds: List<Kind>,
}

/// Per-channel storage and wire limits supplied by its owner.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub chunk: u32,
    pub credential: u32,
    pub skip: u32,
    pub output_bytes: u32,
    pub output_frames: u32,
    pub kinds: u32,
}

/// Why a supplied schema or limit cannot form a bounded channel.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SchemaError {
    /// At least one count or byte bound is unusable.
    InvalidLimit,
    /// An application kind is reserved, duplicated or exceeds a bound.
    Kind,
}

/// A failed kind table check, also used for the responder's refusal text.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum KindTableError {
    TooManyKinds,
    ReservedKind,
    BodyCapacity,
    OutputCapacity,
    DuplicateKind,
}

impl Schema {
    /// Checks the whole table and this role's sent frames (net/channel.md, section 4).
    pub fn check(&self, role: Role, limits: &Limits) -> Result<(), SchemaError> {
        if limits.chunk == 0
            || limits.output_frames == 0
            || limits.output_bytes < 8
            || limits.kinds == 0
            || usize::try_from(limits.output_bytes).is_err()
            || isize::try_from(limits.output_bytes).is_err()
            || usize::try_from(limits.chunk).is_err()
            || usize::try_from(limits.credential).is_err()
            || usize::try_from(limits.skip).is_err()
        {
            return Err(SchemaError::InvalidLimit);
        }
        match check_kinds(&self.kinds, role, limits) {
            Ok(()) => Ok(()),
            Err(_) => Err(SchemaError::Kind),
        }
    }

    /// Installs a checked replacement for the application kinds.
    pub(super) fn install_kinds(
        &mut self,
        kinds: List<Kind>,
        role: Role,
        limits: &Limits,
    ) -> Result<(), KindTableError> {
        check_kinds(&kinds, role, limits)?;
        self.kinds = kinds;
        Ok(())
    }

    /// Finds an application kind in this table.
    #[must_use]
    pub fn kind(&self, number: u16) -> Option<Kind> {
        for kind in &self.kinds {
            if kind.kind == number {
                return Some(*kind);
            }
        }
        None
    }
}

fn check_kinds(kinds: &List<Kind>, role: Role, limits: &Limits) -> Result<(), KindTableError> {
    if kinds.len() > limits.kinds {
        return Err(KindTableError::TooManyKinds);
    }
    let sent = match role {
        Role::Initiator => Direction::FromInitiator,
        Role::Responder => Direction::FromResponder,
    };
    for (index, kind) in kinds.iter().enumerate() {
        if kind.kind < 0x0100 {
            return Err(KindTableError::ReservedKind);
        }
        if usize::try_from(kind.largest).is_err() || isize::try_from(kind.largest).is_err() {
            return Err(KindTableError::BodyCapacity);
        }
        // Received bodies are bounded by their own terms, not the output queue.
        if kind.direction == sent {
            let length = kind.largest.checked_add(8).ok_or(KindTableError::OutputCapacity)?;
            if length > limits.output_bytes {
                return Err(KindTableError::OutputCapacity);
            }
        }
        for earlier in kinds.iter().take(index) {
            if earlier.kind == kind.kind {
                return Err(KindTableError::DuplicateKind);
            }
        }
    }
    Ok(())
}
