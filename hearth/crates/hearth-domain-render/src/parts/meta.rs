//! Meta's canonical record (hearth/docs/design/domain/render/spec.md, 1.4; hearth/docs/design/domain/store/README.md, 3.1).
use crate::revision::{Firmware, Runs};
pub use hearth_domain_render_vocabulary::parts::meta::{Change, MemberPair, Meta, NamePair, Placement, PlacementPair};
use hearth_lib::{
    encode::{Digest, Encoder},
    hash::H,
};

#[must_use]
pub(crate) fn change(before: &Meta, after: &Meta) -> Option<Change> {
    let change = Change {
        base: if before.base == after.base {
            None
        } else {
            Some(NamePair { before: before.base.clone(), after: after.base.clone() })
        },
        member: if before.member == after.member {
            None
        } else {
            Some(MemberPair { before: before.member.clone(), after: after.member.clone() })
        },
        environment: if before.environment == after.environment {
            None
        } else {
            Some(NamePair { before: before.environment.clone(), after: after.environment.clone() })
        },
        placement: if before.placement == after.placement {
            None
        } else {
            Some(PlacementPair { before: before.placement.clone(), after: after.placement.clone() })
        },
    };
    if change.base.is_none() && change.member.is_none() && change.environment.is_none() && change.placement.is_none() {
        None
    } else {
        Some(change)
    }
}
#[must_use]
pub(crate) fn digest(meta: &Meta) -> Digest {
    let mut hash = H::new(b"part:meta", 1);
    encode(meta, hash.values());
    hash.finish()
}
pub(crate) fn encode(meta: &Meta, record: &mut Encoder) {
    record.map(if meta.member.is_some() { 5 } else { 4 });
    record.key(b"base");
    record.bytes(&meta.base);
    record.key(b"environment");
    record.bytes(&meta.environment);
    if let Some(member) = &meta.member {
        record.key(b"member");
        record.bytes(member);
    }
    record.key(b"placement");
    let p = &meta.placement;
    let count = 1_u64
        .saturating_add(u64::from(p.firmware.is_some()))
        .saturating_add(u64::from(p.hardware.is_some()))
        .saturating_add(u64::from(p.on.is_some()));
    record.map(count);
    if let Some(firmware) = p.firmware {
        record.key(b"firmware");
        record.bytes(match firmware {
            Firmware::Efi => b"efi",
            Firmware::Bios => b"bios",
        });
    }
    if let Some(hardware) = &p.hardware {
        record.key(b"hardware");
        record.bytes(hardware);
    }
    record.key(b"kind");
    record.bytes(match p.kind {
        Runs::Metal => b"metal",
        Runs::Vm => b"vm",
        Runs::Container => b"container",
    });
    if let Some(on) = &p.on {
        record.key(b"on");
        record.bytes(on);
    }
    record.key(b"target");
    record.bytes(&meta.target);
}
