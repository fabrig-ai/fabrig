//! Stable forge effect identities (domain/forge.md, section 6).
use alloc::boxed::Box;
use skein_lib::List;

/// Why an effect exists, independent of its outbox entry number or wall time
/// (jig's domain/connectors.md, section 4.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum EffectPurpose {
    /// A tier run operation (temper/docs/design/domain/after-landing.md, 10).
    Tier {
        repository: skein_forge::api::Repository,
        tier: u16,
        run: u64,
        operation: u8,
        part: u16,
    },
    Landing {
        task: u64,
        member: u16,
        gate: u16,
        operation: u8,
        preparation: u16,
        attempt: u16,
    },
    Baseline {
        repository: skein_forge::api::Repository,
        gate: u16,
        key: [u8; 8],
    },
    /// One task history milestone within a goal projection.
    ProjectionHistory {
        goal: u64,
        task: u64,
        position: u64,
        family: u8,
    },
    /// One agent call in one task attempt.
    Call {
        task: u64,
        attempt: u64,
        completion: u32,
        position: u32,
    },
    /// One purpose within a task's procedure.
    Step {
        task: u64,
        purpose: u16,
    },
    /// One part of a goal's projection.
    Projection {
        goal: u64,
        part: u8,
        number: u64,
    },
    /// Cleanup of a task's named resource.
    Release {
        task: u64,
        resource: Box<[u8]>,
    },
}

impl EffectPurpose {
    /// Tag, then fields big-endian; history encodes family before goal, task and position.
    #[must_use]
    pub fn identity(&self) -> Option<Box<[u8]>> {
        let purpose_bytes = match self {
            Self::Tier { .. } => 30,
            Self::Baseline { .. } => 27,
            EffectPurpose::ProjectionHistory { .. } => 26,
            EffectPurpose::Call { .. } => 25,
            EffectPurpose::Step { .. } => 11,
            Self::Landing { .. } | EffectPurpose::Projection { .. } => 18,
            EffectPurpose::Release { resource, .. } => 11_usize.checked_add(resource.len())?,
        };
        let mut bytes = List::with_capacity(u32::try_from(purpose_bytes).ok()?);
        match self {
            Self::Landing { task, member, gate, operation, preparation, attempt } => {
                bytes.push(6).ok()?;
                for byte in task
                    .to_be_bytes()
                    .into_iter()
                    .chain(member.to_be_bytes())
                    .chain(gate.to_be_bytes())
                    .chain([*operation])
                    .chain(preparation.to_be_bytes())
                    .chain(attempt.to_be_bytes())
                {
                    bytes.push(byte).ok()?;
                }
            }
            Self::Tier { repository, tier, run, operation, part } => {
                bytes.push(8).ok()?;
                for byte in u64::from(repository.forge)
                    .to_be_bytes()
                    .into_iter()
                    .chain(u64::from(repository.repository).to_be_bytes())
                    .chain(tier.to_be_bytes())
                    .chain(run.to_be_bytes())
                    .chain([*operation])
                    .chain(part.to_be_bytes())
                {
                    bytes.push(byte).ok()?;
                }
            }
            Self::Baseline { repository, gate, key } => {
                bytes.push(7).ok()?;
                for byte in u64::from(repository.forge)
                    .to_be_bytes()
                    .into_iter()
                    .chain(u64::from(repository.repository).to_be_bytes())
                    .chain(gate.to_be_bytes())
                    .chain(*key)
                {
                    bytes.push(byte).ok()?;
                }
            }
            EffectPurpose::ProjectionHistory { goal, task, position, family } => {
                bytes.push(5).ok()?;
                bytes.push(*family).ok()?;
                for byte in goal.to_be_bytes().into_iter().chain(task.to_be_bytes()).chain(position.to_be_bytes()) {
                    bytes.push(byte).ok()?;
                }
            }
            EffectPurpose::Call { task, attempt, completion, position } => {
                bytes.push(1).ok()?;
                for byte in task
                    .to_be_bytes()
                    .into_iter()
                    .chain(attempt.to_be_bytes())
                    .chain(completion.to_be_bytes())
                    .chain(position.to_be_bytes())
                {
                    bytes.push(byte).ok()?;
                }
            }
            EffectPurpose::Step { task, purpose } => {
                bytes.push(2).ok()?;
                for byte in task.to_be_bytes().into_iter().chain(purpose.to_be_bytes()) {
                    bytes.push(byte).ok()?;
                }
            }
            EffectPurpose::Projection { goal, part, number } => {
                bytes.push(3).ok()?;
                for byte in goal.to_be_bytes() {
                    bytes.push(byte).ok()?;
                }
                bytes.push(*part).ok()?;
                for byte in number.to_be_bytes() {
                    bytes.push(byte).ok()?;
                }
            }
            EffectPurpose::Release { task, resource } => {
                bytes.push(4).ok()?;
                for byte in task.to_be_bytes().into_iter().chain(u16::try_from(resource.len()).ok()?.to_be_bytes()) {
                    bytes.push(byte).ok()?;
                }
                for byte in resource {
                    bytes.push(*byte).ok()?;
                }
            }
        }
        Some(bytes.into_boxed())
    }

    /// Decode a stable identity, refusing unknown tags, widths and trailing bytes.
    #[must_use]
    pub fn decode(identity: &[u8]) -> Option<Self> {
        u32::try_from(identity.len()).ok()?;
        let mut reader = skein_lib::Reader::new(identity);
        let purpose = match reader.u8()? {
            1 => Self::Call {
                task: reader.u64()?,
                attempt: reader.u64()?,
                completion: reader.u32()?,
                position: reader.u32()?,
            },
            2 => Self::Step { task: reader.u64()?, purpose: reader.u16()? },
            3 => Self::Projection { goal: reader.u64()?, part: reader.u8()?, number: reader.u64()? },
            4 => {
                let task = reader.u64()?;
                let len = reader.u16()?;
                Self::Release { task, resource: Box::from(reader.bytes(u32::from(len))?) }
            }
            5 => {
                let family = reader.u8()?;
                Self::ProjectionHistory { family, goal: reader.u64()?, task: reader.u64()?, position: reader.u64()? }
            }
            6 => Self::Landing {
                task: reader.u64()?,
                member: reader.u16()?,
                gate: reader.u16()?,
                operation: reader.u8()?,
                preparation: reader.u16()?,
                attempt: reader.u16()?,
            },
            7 => Self::Baseline {
                repository: skein_forge::api::Repository {
                    forge: u16::try_from(reader.u64()?).ok()?,
                    repository: u32::try_from(reader.u64()?).ok()?,
                },
                gate: reader.u16()?,
                key: reader.bytes(8)?.try_into().ok()?,
            },
            8 => Self::Tier {
                repository: skein_forge::api::Repository {
                    forge: u16::try_from(reader.u64()?).ok()?,
                    repository: u32::try_from(reader.u64()?).ok()?,
                },
                tier: reader.u16()?,
                run: reader.u64()?,
                operation: reader.u8()?,
                part: reader.u16()?,
            },
            _ => return None,
        };
        if !reader.is_empty() {
            return None;
        }
        Some(purpose)
    }

    /// Frame this identity within the deployment namespace and byte cap.
    #[must_use]
    pub fn key(&self, namespace: &[u8], max_bytes: u32) -> Option<Box<[u8]>> {
        skein_forge_client::effect_key(namespace, &self.identity()?, max_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tier_identity_has_thirty_bytes() {
        let purpose = EffectPurpose::Tier {
            repository: skein_forge::api::Repository { forge: 3, repository: 5 },
            tier: 7,
            run: 11,
            operation: 4,
            part: 13,
        };
        let bytes = purpose.identity().expect("bounded tier identity");
        assert_eq!(bytes.len(), 30);
        assert_eq!(EffectPurpose::decode(&bytes), Some(purpose.clone()));
        for len in 0..bytes.len() {
            assert_eq!(EffectPurpose::decode(&bytes[..len]), None);
        }
        let mut trailing = List::with_capacity(31);
        for byte in &bytes {
            trailing.push(*byte).expect("identity plus trailing byte");
        }
        trailing.push(0).expect("trailing room");
        assert_eq!(EffectPurpose::decode(trailing.as_slice()), None);
        assert_eq!(bytes[0], 8);
        assert_eq!(purpose.key(&[1; 16], 48), None);
        assert!(purpose.key(&[1; 16], 49).is_some());
    }
}

#[cfg(test)]
mod round_trip {
    use super::*;
    #[test]
    fn stable_purpose_tags_round_trip_and_reject_trailing_bytes() {
        let repository = skein_forge::api::Repository { forge: u16::MAX, repository: u32::MAX };
        for purpose in [
            EffectPurpose::Call { task: u64::MAX, attempt: 2, completion: u32::MAX, position: 4 },
            EffectPurpose::Step { task: 1, purpose: u16::MAX },
            EffectPurpose::Projection { goal: 2, part: 3, number: 4 },
            EffectPurpose::Release { task: 5, resource: Box::from(b"resource".as_slice()) },
            EffectPurpose::ProjectionHistory { family: 1, goal: 2, task: 3, position: 4 },
            EffectPurpose::Landing { task: 1, member: 2, gate: 3, operation: 4, preparation: 5, attempt: 6 },
            EffectPurpose::Baseline { repository, gate: 7, key: [8; 8] },
            EffectPurpose::Tier { repository, tier: u16::MAX, run: u64::MAX, operation: 7, part: u16::MAX },
        ] {
            let bytes = purpose.identity().expect("bounded identity");
            assert_eq!(EffectPurpose::decode(&bytes), Some(purpose));
        }
        for bytes in [&[0][..], &[9][..], &[8, 0][..]] {
            assert_eq!(EffectPurpose::decode(bytes), None);
        }
        let mut width = List::with_capacity(30);
        for byte in [8].into_iter().chain(u64::MAX.to_be_bytes()).chain([0; 21]) {
            width.push(byte).expect("thirty bytes");
        }
        assert_eq!(EffectPurpose::decode(width.as_slice()), None);
    }
}
