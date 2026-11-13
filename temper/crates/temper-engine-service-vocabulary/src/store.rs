//! The deployment store’s routes over jig’s file-driver host (protocol/store.md, 3–5;
//! jig's protocol/store/README.md, 8).
mod ranges;
pub mod route;
mod unreadable;
use jig_protocol_store as protocol;
use protocol::host::{check_read_page, max_whole};
use skein_lib::Token;
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
pub use unreadable::UnreadableRow;
/// The wires this engine speaks to its workers (jig's protocol/store/records.md,
/// 3; temper's protocol/README.md, 7).
pub const WIRES: [protocol::Wire; 4] = [
    protocol::Wire { name: "jig-channel", version: channel_version(jig_protocol_worker_wire::VERSION) },
    protocol::Wire { name: "smith-charter", version: record_version(smith_charter::Charter::FORMAT_VERSION) },
    protocol::Wire { name: "smith-turn", version: record_version(smith_transcript::Turn::FORMAT_VERSION) },
    protocol::Wire { name: "temper-payload", version: temper_protocol_worker_wire::PAYLOAD_VERSION },
];
/// Owners kept by every engine shape, upstream first (protocol/baseline.md, 4).
pub const PATCH_SETS: protocol::PatchSets = protocol::PatchSets {
    sets: &[
        protocol::SKEIN_PATCH_SET,
        protocol::SMITH_PATCH_SET,
        jig_protocol_accounts::JIG_PATCH_SET,
        temper_protocol_store::PATCH_SET,
    ],
};
// These conversions are checked while the static wire table is evaluated.
const fn channel_version(version: u16) -> u32 {
    let bytes = version.to_le_bytes();
    u32::from_le_bytes([bytes[0], bytes[1], 0, 0])
}
const fn record_version(version: u64) -> u32 {
    let bytes = version.to_le_bytes();
    assert!(bytes[4] == 0 && bytes[5] == 0 && bytes[6] == 0 && bytes[7] == 0, "wire version fits u32");
    u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
}
/// Submission slots the component reserves for store-driver handoff.
pub const KERNEL_HANDOFF_ENTRIES: u32 = protocol::host::KERNEL_HANDOFF_ENTRIES;
/// Admit the core, journal and temper families, then copy checked sizes to KV.
#[must_use]
pub fn limits(root: &root::Limits, forge: &forge::Limits, profile: &protocol::Limits) -> Option<protocol::Limits> {
    admit(root, forge, profile, None).ok()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refusal {
    Map(protocol::sizes::SizeRefusal),
    ReadRoom { read: u64, needs: u64 },
    Transcript { bytes: u32, ceiling: u32 },
    Page { transcript: u32, frame: u32, page: u32, ceiling: u32 },
}

/// Keep the cause for startup before any comparison needs the store's bounds.
pub fn admit(
    root: &root::Limits,
    forge: &forge::Limits,
    profile: &protocol::Limits,
    turn: Option<u32>,
) -> Result<protocol::Limits, Refusal> {
    let limits = layout(root, forge, profile)?;
    let needs = read_room(&limits, turn).ok_or(Refusal::Map(protocol::sizes::SizeRefusal::Ceiling))?;
    let decoded = decoded_read_room(turn).ok_or(Refusal::Map(protocol::sizes::SizeRefusal::Ceiling))?;
    // The chunk or resume buffer can already cover a page. Only a decoded
    // row must fit the configured read itself; either refusal names one room.
    let page_fits = check_read_page(&limits).is_some();
    if !page_fits || root.loads.read_bytes < decoded {
        let needs = if page_fits { decoded } else { needs };
        return Err(Refusal::ReadRoom { read: root.loads.read_bytes, needs });
    }
    match protocol::sizes::whole(max_whole(&limits)) {
        Ok(()) => {}
        Err(why) => return Err(Refusal::Map(why)),
    }
    Ok(limits)
}

/// The room named by either read refusal: the drivers' encoded page or the
/// agent's decoded turn with its read row (protocol/configuration/limits.md).
/// Without agent metadata only the page requirement is known.
#[must_use]
pub fn read_room(limits: &protocol::Limits, turn: Option<u32>) -> Option<u64> {
    Some(u64::from(limits.store.page.bytes).max(decoded_read_room(turn)?))
}

fn decoded_read_room(turn: Option<u32>) -> Option<u64> {
    match turn {
        Some(turn) => u64::from(turn)
            .checked_add(u64::try_from(size_of::<jig_domain_core::reads::Row>()).ok()?)?
            .checked_add(u64::try_from(size_of::<jig_domain_core::Record>()).ok()?),
        None => Some(0),
    }
}

/// Size the records before agent metadata is read. Read admission follows once
/// the page and, where available, the agent's decoded read-row room are known.
pub fn layout(
    root: &root::Limits,
    forge: &forge::Limits,
    profile: &protocol::Limits,
) -> Result<protocol::Limits, Refusal> {
    let arithmetic = Refusal::Map(protocol::sizes::SizeRefusal::Ceiling);
    let core = root::core_limits(root);
    let mut kv = settings_room(&core, profile.store).ok_or(arithmetic)?;
    let page_ceiling = kv.page.bytes;
    let transcript = root.loads.transcript_bytes;
    let ceiling = protocol::core_v6::CEILINGS.turn_record_transcript;
    if transcript > ceiling {
        return Err(Refusal::Transcript { bytes: transcript, ceiling });
    }
    let sizes = match protocol::sizes::check_commits(
        &core,
        &temper_protocol_store::application(forge, root.tasks.spec_bytes).ok_or(arithmetic)?,
        &protocol::sizes::Commits { in_flight: root.journal.commits, writes: root.journal.writes },
        &kv,
        profile.orphan_erases,
    ) {
        Ok(sizes) => sizes,
        Err(why) => return Err(Refusal::Map(why)),
    };
    protocol::sizes::fill(&mut kv, &sizes).ok_or(arithmetic)?;
    kv.page.bytes = page_bytes(&core, transcript, sizes.key, sizes.value).ok_or(arithmetic)?;
    if kv.page.bytes > page_ceiling {
        let frame = protocol::sizes::frame(&core, transcript).ok_or(arithmetic)?;
        return Err(Refusal::Page { transcript, frame, page: kv.page.bytes, ceiling: page_ceiling });
    }
    Ok(protocol::Limits {
        core,
        store: kv,
        secrets: kv,
        held: profile.held,
        commits: protocol::sizes::Commits { in_flight: root.journal.commits, writes: root.journal.writes },
        joins: profile.joins,
        queue: kv.queued,
        files: profile.files,
        reads: core.load_slots.checked_add(core.tasks.tasks).ok_or(arithmetic)?,
        orphan_erases: profile.orphan_erases,
    })
}

// Settings carry a whole revision, including replaced values and model names.
// Their room follows the effective capacities (protocol/store.md, 4).
fn settings_room(core: &jig_domain_core::Limits, mut kv: skein_kv::Limits) -> Option<skein_kv::Limits> {
    let value = protocol::sizes::settings_value(core)?;
    if value > kv.value {
        kv.value = value;
        kv.page.bytes = kv.page.bytes.max(kv.key.checked_add(value)?);
    }
    Some(kv)
}

// protocol/store.md, 4; jig owns current full-frame sizing.
fn page_bytes(core: &jig_domain_core::Limits, transcript: u32, key: u32, value: u32) -> Option<u32> {
    Some(key.checked_add(value)?.max(protocol::sizes::frame(core, transcript)?))
}

const MASK: u64 = (1_u64 << 60) - 1;
#[must_use]
pub fn owns(token: Token) -> bool {
    crate::tokens::STORE.contains(&(token.raw() & !MASK))
}

#[cfg(test)]
mod sizing_tests {
    use super::{max_whole, page_bytes, protocol};
    use protocol::host::max_read;

    #[test]
    fn drivers_read_a_whole_page_in_every_admitted_store() {
        let (application, _, profile) = crate::tests::fixtures::store_limits();
        let limits = super::limits(&application, &application.forge, &profile).unwrap();
        assert!(max_read(&limits) >= limits.store.page.bytes);
    }

    #[test]
    fn drivers_hold_the_whole_manifest_when_reads_are_smaller() {
        let (_, _, mut limits) = crate::tests::fixtures::store_limits();
        limits.core.resume_bytes = 16;
        limits.core.load_bytes = 8;
        // Synthetic read fields exercise the helper without constructing a KV store.
        for chunk in [32, protocol::sizes::MANIFEST_BYTES.checked_sub(1).unwrap()] {
            limits.store.chunk = chunk;
            assert_eq!(max_read(&limits), chunk);
            assert_eq!(max_whole(&limits), protocol::sizes::MANIFEST_BYTES);
        }
    }

    #[test]
    fn drivers_take_reads_whole_when_reads_are_larger() {
        let (_, _, mut limits) = crate::tests::fixtures::store_limits();
        let above = protocol::sizes::MANIFEST_BYTES.checked_add(1).unwrap();
        for (chunk, resume, load) in [(above, 16, 8), (32, above, 8), (32, 16, u64::from(above))] {
            limits.store.chunk = chunk;
            limits.core.resume_bytes = resume;
            limits.core.load_bytes = load;
            assert_eq!(max_read(&limits), above);
            assert_eq!(max_whole(&limits), max_read(&limits));
        }
    }

    #[test]
    fn an_admitted_store_already_reads_the_whole_manifest() {
        let (application, _, profile) = crate::tests::fixtures::store_limits();
        let limits = super::limits(&application, &application.forge, &profile).unwrap();
        assert!(max_read(&limits) >= protocol::sizes::MANIFEST_BYTES);
        assert_eq!(max_whole(&limits), max_read(&limits));
    }

    #[test]
    fn typed_secret_rows_fit_the_small_profiles_page() {
        // Secrets keys are the two owner bytes followed by a u64 number.
        const KEY_BYTES: u64 = 2 + 8;
        let (application, _, profile) = crate::tests::fixtures::store_limits();
        let limits = super::limits(&application, &application.forge, &profile).unwrap();
        let row = protocol::secrets_v1::encoded_bound(&protocol::secrets_v1::CEILINGS).unwrap();
        assert_eq!(row, 274_504);
        assert!(row <= u64::from(limits.secrets.value));
        assert!(row.checked_add(KEY_BYTES).unwrap() < u64::from(limits.secrets.page.bytes));
        assert_eq!(profile.store.page.bytes, 4_582_931);
    }

    #[test]
    fn page_covers_map_or_full_current_frame() {
        let (_, core, _) = crate::tests::fixtures::store_limits();
        // Versioned TurnRecord: 52 bytes including Some(read), plus payload;
        // two lengths and CRC add 12. No turn or frame buffer is allocated.
        assert_eq!(page_bytes(&core, 8, 3, 4), Some(72));
        assert_eq!(page_bytes(&core, 8, 80, 4), Some(84));
    }

    #[test]
    fn page_refuses_codec_excess_and_checked_overflow() {
        let (_, core, _) = crate::tests::fixtures::store_limits();
        let ceiling = jig_protocol_store::core_v6::CEILINGS.turn_record_transcript;
        assert_eq!(page_bytes(&core, ceiling, 3, 4), ceiling.checked_add(64));
        assert_eq!(page_bytes(&core, ceiling.checked_add(1).unwrap(), 3, 4), None);
        assert_eq!(page_bytes(&core, u32::MAX, 3, 4), None);
        assert_eq!(page_bytes(&core, 8, u32::MAX, 1), None);
    }
}
