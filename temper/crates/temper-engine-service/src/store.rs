//! The deployment store’s routes over jig’s file-driver host (protocol/store.md, 3–5;
//! jig's protocol/store/README.md, 8).
mod ranges;
pub mod route;
mod unreadable;
use jig_protocol_store as protocol;
use protocol::host::{Hosted, Lane, NotReady, check_read_page, max_whole};
use route::Up;
use skein_io::kernel::{Complete, Fd, Op, Submit};
use skein_lib::{List, Queue, Time, Token};
use temper_engine_domain as root;
use temper_engine_domain_forge as forge;
pub use unreadable::UnreadableRow;
/// The wires this engine speaks to its workers (jig's protocol/store/records.md,
/// 3; temper's protocol/README.md, 7).
pub const WIRES: [protocol::Wire; 6] = [
    protocol::Wire {
        name: jig_protocol_worker_wire::wire::FAMILY,
        version: wire_version(jig_protocol_worker_wire::wire::VERSION),
    },
    protocol::Wire {
        name: smith_channel::pieces::wire::FAMILY,
        version: wire_version(smith_channel::pieces::wire::VERSION),
    },
    protocol::Wire { name: smith_charter::wire::FAMILY, version: wire_version(smith_charter::wire::VERSION) },
    protocol::Wire { name: smith_transcript::wire::FAMILY, version: wire_version(smith_transcript::wire::VERSION) },
    protocol::Wire {
        name: temper_protocol_worker_wire::wire::FAMILY,
        version: wire_version(temper_protocol_worker_wire::wire::VERSION),
    },
    protocol::Wire {
        name: temper_engine_smith::wire::FAMILY,
        version: wire_version(temper_engine_smith::wire::VERSION),
    },
];

/// The engine's wires from their floors, in statement order (protocol/wires.md, 10).
#[must_use]
pub fn wires() -> List<jig_protocol_worker_wire::Spoken> {
    use jig_protocol_worker_wire::Spoken;
    let mut wires = List::with_capacity(6);
    for wire in jig_protocol_worker_wire::spoken().into_boxed() {
        wires.push(wire).expect("six wire slots");
    }
    wires.push(temper_protocol_worker_wire::spoken()).expect("six wire slots");
    let mut layouts = List::with_capacity(u32::from(temper_engine_smith::wire::VERSION));
    for (index, text) in temper_engine_smith::wire::TEXTS.iter().enumerate() {
        let version =
            u16::try_from(index).expect("kept contract index fits u16").checked_add(1).expect("contract version");
        if version >= temper_engine_smith::wire::FLOOR {
            layouts.push(skein_lib::sha256::digest(text.as_bytes())).expect("one layout per kept contract");
        }
    }
    wires
        .push(Spoken { name: temper_engine_smith::wire::FAMILY, lowest: temper_engine_smith::wire::FLOOR, layouts })
        .expect("six wire slots");
    wires
}
/// Owners kept by every engine shape, upstream first (protocol/baseline.md, 4).
pub const PATCH_SETS: protocol::PatchSets = protocol::PatchSets {
    sets: &[
        protocol::SKEIN_PATCH_SET,
        protocol::SMITH_PATCH_SET,
        jig_protocol_accounts::JIG_PATCH_SET,
        temper_protocol_store::PATCH_SET,
    ],
};
// Widen each version while the static wire row is evaluated.
const fn wire_version(version: u16) -> u32 {
    let [low, high] = version.to_le_bytes();
    u32::from_le_bytes([low, high, 0, 0])
}
const MASK: u64 = (1_u64 << 60) - 1;
pub(crate) const KERNEL_HANDOFF_ENTRIES: u32 = protocol::host::KERNEL_HANDOFF_ENTRIES;
#[derive(Debug)]
pub struct Store {
    #[cfg(test)]
    pub(crate) forge_loads: List<Option<u64>>,
    #[cfg(test)]
    pub(crate) erases: u32,
    #[cfg(test)]
    pub(crate) hold_mark: bool,
    #[cfg(test)]
    pub(crate) held_mark: Option<root::StoreRequest>,
    #[cfg(test)]
    pub(crate) secret_requests: u32,
    pub(crate) host: Hosted,
    pub(crate) routes: route::Routes,
    pub(crate) routed: Queue<Up>,
    pub(crate) submitted: Queue<Submit>,
    pub(crate) opened: bool,
    pub(crate) pending_generation: bool,
    /// The installation's identifier: the drawn one until the store opens,
    /// then the one it keeps (jig's protocol/store/README.md 4.4).
    pub(crate) installation: [u8; 16],
}

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
pub(crate) fn read_room(limits: &protocol::Limits, turn: Option<u32>) -> Option<u64> {
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
pub(crate) fn layout(
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

impl Store {
    #[must_use]
    pub fn opening_limits(&self) -> protocol::Limits {
        *self.host.limits()
    }

    #[must_use]
    pub fn new(
        application: &root::Limits,
        limits: &protocol::Limits,
        models: &[jig_domain_core::Model],
        installation: [u8; 16],
    ) -> Self {
        let routes = route::Routes::new(
            application.loads.loads,
            application,
            temper_protocol_store::bounds(&application.forge, application.tasks.spec_bytes)
                .expect("checked family bounds"),
        );
        let application_sizes = temper_protocol_store::application(&application.forge, application.tasks.spec_bytes)
            .expect("checked application sizes");
        let seed = u64::from_le_bytes(*installation.first_chunk::<8>().expect("installation seed"));
        let mut host = Hosted::new(limits, application_sizes, models, seed, NotReady::Above);
        assert!(host.component.wires(&WIRES), "valid engine wires before open");
        host.component.patch_sets(PATCH_SETS);
        Self {
            #[cfg(test)]
            forge_loads: List::with_capacity(128),
            #[cfg(test)]
            erases: 0,
            #[cfg(test)]
            hold_mark: false,
            #[cfg(test)]
            held_mark: None,
            #[cfg(test)]
            secret_requests: 0,
            routes,
            routed: Queue::uncapped(),
            host,
            submitted: Queue::with_capacity(KERNEL_HANDOFF_ENTRIES),
            opened: false,
            pending_generation: false,
            installation,
        }
    }
    /// Replace the binary's sets before adopting its deployment directory.
    pub(crate) fn patch_sets(&mut self, sets: protocol::PatchSets) {
        self.host.component.patch_sets(sets);
    }
    /// Whether the component has reported that its deployment opened.
    #[must_use]
    pub const fn opened(&self) -> bool {
        self.opened
    }
    pub fn adopt(&mut self, fd: Fd, user: u32) -> Result<(), Fd> {
        self.host.adopt(fd, user, self.installation)
    }
    pub fn down(&mut self, out: root::StoreRequest) -> bool {
        let out = match out {
            root::StoreRequest::Catalogue { names } => {
                self.host.component.catalogue(names);
                return true;
            }
            out @ (root::StoreRequest::Switch
            | root::StoreRequest::Commit { .. }
            | root::StoreRequest::Load { .. }
            | root::StoreRequest::Read { .. }) => out,
        };
        #[cfg(test)]
        if self.hold_mark
            && let root::StoreRequest::Commit { writes, .. } = &out
            && writes.contains(&root::Write::Wires)
        {
            assert!(self.held_mark.is_none(), "one held startup mark");
            self.held_mark = Some(out);
            return true;
        }
        if let Some(request) = route::down(&mut self.routes, out) {
            self.host.ask(request);
            true
        } else {
            false
        }
    }
    /// Get answers stay in this pass; secret writes' terminals are handed to their owner.
    pub fn secrets(&mut self, owner: Token, request: protocol::Secrets) -> Option<protocol::Event> {
        #[cfg(test)]
        {
            self.secret_requests = self.secret_requests.checked_add(1).expect("test secret requests");
        }
        #[cfg(test)]
        if owner == crate::forge::records::LOAD
            && let protocol::Secrets::Load { after, .. } = &request
        {
            self.forge_loads.push(*after).expect("test page trace");
        }
        #[cfg(test)]
        if let protocol::Secrets::Erase { .. } = &request {
            self.erases = self.erases.checked_add(1).expect("test erases");
        }
        self.host.secrets(owner, request)
    }
    #[must_use]
    pub fn work_pending(&self, now: Time) -> bool {
        self.host.work_pending(now) || !self.routed.is_empty() || !self.submitted.is_empty()
    }
    #[must_use]
    pub fn closed(&self) -> bool {
        self.host.closed() && self.idle()
    }
    #[must_use]
    pub fn idle(&self) -> bool {
        (self.host.closing() || self.routes.idle())
            && self.routed.is_empty()
            && self.submitted.is_empty()
            && self.host.idle()
    }
    pub fn close(&mut self) {
        self.host.close();
        for _ in 0..self.routed.len() {
            let _up = self.routed.pop();
        }
    }
    pub fn complete(&mut self, mut complete: Complete) {
        let index = usize::try_from((complete.op.raw() >> 60_u32).checked_sub(8).expect("driver namespace"))
            .expect("three drivers");
        complete.op = Token::new(complete.op.raw() & MASK);
        if let Op::Cancel { target } = &mut complete.kind {
            *target = Token::new(target.raw() & MASK);
        }
        let lane = *Lane::ALL.get(index).expect("driver index");
        self.host.complete(lane, complete);
        self.publish();
    }
    fn publish(&mut self) {
        for _ in 0..self.host.submissions() {
            let protocol::host::Submission { lane, mut submit } = self.host.pop_submission().expect("host submissions");
            let prefix = *crate::tokens::STORE.get(lane.index()).expect("driver prefix");
            submit.op = Token::new(submit.op.raw() | prefix);
            if let Op::Cancel { target } = &mut submit.kind {
                *target = Token::new(target.raw() | prefix);
            }
            self.submitted.push(submit);
        }
    }
    pub fn iterate(&mut self) {
        self.host.iterate();
        self.publish();
    }
}
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
