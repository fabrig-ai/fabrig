//! Application secrets and forge connections after the durable start mark (domain/root.md, 9.1).
use alloc::boxed::Box;
use jig_protocol_store::{Event, SecretOwner, Secrets};
use skein_lib::{List, Map, Secret, Token};
use temper_protocol_records as records;
pub(crate) const LOAD: Token = Token::new(crate::tokens::FORGE | 1);
const FORGES: u32 = 65536;
pub(crate) const ITEMS: [records::Item; 10] = [
    records::Item::AdminToken,
    records::Item::ApiToken,
    records::Item::GitToken,
    records::Item::User,
    records::Item::WebhookSecret,
    records::Item::AppId,
    records::Item::AppSlug,
    records::Item::AppKey,
    records::Item::Accepted,
    records::Item::Origin,
];
/// The service retains values only for its configured forges. The codec's
/// general-purpose Rows is filled solely through this holder's checked push.
#[derive(Debug)]
pub(crate) struct Rows {
    values: Map<u16, records::Rows>,
    configured: List<u16>,
    unconfigured: Map<u16, ()>,
}
impl Rows {
    fn new(forges: &[crate::settings::Forge], capacity: u32) -> Self {
        let mut configured = List::with_capacity(capacity);
        for forge in forges {
            configured.push(forge.number).expect("configured forges");
        }
        Self { values: Map::with_capacity(capacity), configured, unconfigured: Map::with_capacity(FORGES) }
    }
    pub(crate) fn add(&mut self, forge: u16) -> Result<(), super::table::NotKept> {
        if self.configured.as_slice().contains(&forge) {
            return Ok(());
        }
        self.configured.push(forge).or(Err(super::table::NotKept::Bound))?;
        self.unconfigured.remove(&forge);
        Ok(())
    }
    pub(crate) fn remove(&mut self, forge: u16) {
        let mut kept = List::with_capacity(self.configured.capacity());
        for number in &self.configured {
            if *number != forge {
                kept.push(*number).expect("configured capacity");
            }
        }
        self.configured = kept;
        self.values.remove(&forge);
        self.unconfigured.remove(&forge);
    }
    pub(crate) fn push(&mut self, number: u64, bytes: Secret, value_bytes: u32) {
        if number >= records::CARRIED || number >> 48_u32 != u64::from(records::FAMILY_FORGE) {
            return;
        }
        let Ok(forge) = u16::try_from((number >> 16_u32) & 0xffff_ffff) else { return };
        if !self.configured.as_slice().contains(&forge) {
            self.unconfigured.insert(forge, ()).expect("all u16 forge numbers fit");
            return;
        }
        // Decode temper-secrets here; get() serves skein setup bytes to every reader.
        if !self.values.contains_key(&forge) {
            self.values.insert(forge, records::Rows::new()).expect("configured forge capacity");
        }
        self.values.get_mut(&forge).expect("configured forge rows").push(number, bytes, value_bytes);
    }
    pub(crate) fn forge(&self, forge: u16) -> Option<&records::ForgeRows> {
        self.values.get(&forge)?.forge(forge)
    }
    #[expect(clippy::manual_map, reason = "the programming model uses explicit matches")]
    fn next_unconfigured(&self, after: u32) -> Option<u16> {
        let after = u16::try_from(after).ok()?;
        match self.unconfigured.range(after..).next() {
            Some((number, ())) => Some(*number),
            None => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct Loader {
    #[cfg(test)]
    pub(crate) inspected: List<u64>,
    pub(crate) rows: Rows,
    phase: Phase,
    pub(crate) bounds: skein_forge_setup::item::Bounds,
    value_bytes: u32,
}
#[derive(Debug)]
enum Phase {
    Load { after: Option<u64> },
    Connect { index: u32 },
    Sweep { number: u64, asked: bool },
    Name { after: u32 },
    Done,
}
impl Loader {
    pub(crate) fn new(limits: &crate::limits::Limits, forges: &[crate::settings::Forge]) -> Self {
        Self {
            #[cfg(test)]
            inspected: List::with_capacity(128),
            rows: Rows::new(forges, limits.forge.forges),
            phase: Phase::Load { after: None },
            value_bytes: limits.records.value_bytes,
            bounds: skein_forge_setup::item::Bounds {
                token_bytes: limits.forge_component.token_bytes,
                name_bytes: limits.forge.name_bytes,
                key_bytes: limits.forge_component.key_bytes,
                installations: limits.forges.installations,
            },
        }
    }
    pub(crate) fn past_connect(&self) -> bool {
        match self.phase {
            Phase::Name { .. } | Phase::Done => true,
            Phase::Load { .. } | Phase::Connect { .. } | Phase::Sweep { .. } => false,
        }
    }
    pub(crate) fn loading(&self) -> bool {
        match self.phase {
            Phase::Load { .. } | Phase::Sweep { .. } => true,
            Phase::Connect { .. } | Phase::Name { .. } | Phase::Done => false,
        }
    }
    pub(crate) fn done(&self) -> bool {
        matches_done(&self.phase)
    }
    pub(crate) fn erasing(&self, number: u64) -> bool {
        match self.phase {
            Phase::Sweep { number: held, asked: true } => held == number,
            Phase::Sweep { asked: false, .. }
            | Phase::Load { .. }
            | Phase::Connect { .. }
            | Phase::Name { .. }
            | Phase::Done => false,
        }
    }
    pub(crate) fn erased(&mut self) {
        let Phase::Sweep { number, asked: true } = self.phase else { unreachable!("startup erase terminal") };
        self.phase = Phase::Load { after: Some(number) };
    }
    pub(crate) fn loaded(&mut self, rows: Box<[jig_protocol_store::SecretRow]>, next: Option<u64>) {
        for row in rows {
            // The value is never inspected; restart has no live connect.
            if row.number >= records::CARRIED {
                self.phase = Phase::Sweep { number: row.number, asked: false };
                return;
            }
            #[cfg(test)]
            self.inspected.push(row.number).expect("test inspected records");
            if let jig_protocol_store::SecretValue::Application(value) = row.value {
                self.rows.push(row.number, value, self.value_bytes);
            }
        }
        self.phase = match next {
            Some(after) => Phase::Load { after: Some(after) },
            None => Phase::Connect { index: 0 },
        };
    }
}
fn matches_done(phase: &Phase) -> bool {
    match phase {
        Phase::Done => true,
        Phase::Load { .. } | Phase::Sweep { .. } | Phase::Connect { .. } | Phase::Name { .. } => false,
    }
}
pub(crate) fn advance(engine: &mut crate::engine::Engine) {
    if engine.end.is_some() || crate::stop::requested(engine) || !engine.store.component.mark_durable() {
        return;
    }
    match engine.forge_records.phase {
        Phase::Load { after } => {
            if !engine.store.opened {
                return;
            }
            let request = Secrets::Load { owner: SecretOwner::Application, after };
            match engine.store.secrets(LOAD, request) {
                Some(Event::SecretsLoaded { owner, rows, next }) => {
                    assert!(owner == LOAD, "forge records token");
                    engine.forge_records.loaded(rows, next);
                }
                Some(_) | None => crate::stop::unavailable(engine, b"forge records"),
            }
        }
        Phase::Sweep { number, asked } => {
            if !asked {
                engine.forge_records.phase = Phase::Sweep { number, asked: true };
                let decision = temper_engine_domain::carried_at_start(number);
                crate::route::request(engine, decision);
            }
        }
        Phase::Connect { index } => {
            let Some(forge) = engine.workers.forges.get(index) else {
                engine.forge_records.phase = Phase::Name { after: 0 };
                return;
            };
            if engine.notes.room() == 0 || engine.events.room() < engine.forge_records.bounds.installations {
                return;
            }
            let number = forge.number;
            connect_one(engine, number);
            engine.forge_records.phase = Phase::Connect { index: index.checked_add(1).expect("configured forges") };
        }
        Phase::Name { after } => {
            // The durable startup mark follows the connector's kept-forge handoffs.
            if crate::setup::erasing_records(engine) {
                return;
            }
            let rows = &engine.forge_records.rows;
            let found = rows.next_unconfigured(after);
            match found {
                Some(number) => {
                    crate::setup::erase_forge(engine, number);
                    engine.forge_records.phase =
                        Phase::Name { after: u32::from(number).checked_add(1).expect("u16 number") };
                }
                None => {
                    engine.forge_records.phase = Phase::Done;
                }
            }
        }
        Phase::Done => {}
    }
}

#[cfg(test)]
#[test]
fn unconfigured_numbers_resume_at_the_cursor_without_retaining_values() {
    let settings = crate::tests::fixtures::engine();
    let mut rows = Rows::new(&settings.forges, settings.limits.forge.forges);
    for forge in [u16::MAX, 77, 0, 77] {
        for item in [records::Item::AppKey, records::Item::Origin] {
            rows.push(records::number(forge, item), Secret::new(Box::from([2, 255])), 4096);
        }
        assert!(rows.forge(forge).is_none());
    }
    // Unknown item codes still identify an unconfigured forge without reading bytes.
    rows.push((u64::from(records::FAMILY_FORGE) << 48) | (78 << 16) | 0x000b, Secret::new(Box::from([])), 4096);
    assert_eq!(rows.unconfigured.len(), 4);
    assert_eq!(rows.next_unconfigured(0), Some(0));
    assert_eq!(rows.next_unconfigured(1), Some(77));
    assert_eq!(rows.next_unconfigured(78), Some(78));
    assert_eq!(rows.next_unconfigured(79), Some(u16::MAX));
    assert_eq!(rows.next_unconfigured(u32::from(u16::MAX)), Some(u16::MAX));
    assert_eq!(rows.next_unconfigured(FORGES), None);
    assert_eq!(rows.next_unconfigured(u32::MAX), None);
    rows.push(records::number(1, records::Item::User), Secret::new(Box::from([1, 9])), 4096);
    assert!(rows.forge(1).is_some());
    assert_eq!(rows.unconfigured.len(), 4);
}

/// Load one runtime forge's bounded secret items from the durable store.
pub(crate) fn load_one(engine: &mut crate::engine::Engine, forge: u16) -> bool {
    for item in ITEMS {
        let number = records::number(forge, item);
        match engine.store.secrets(LOAD, Secrets::Get { owner: SecretOwner::Application, number }) {
            Some(Event::Secret { value: Some(jig_protocol_store::SecretValue::Application(value)), .. }) => {
                engine.forge_records.rows.push(number, value, engine.forge_records.value_bytes);
            }
            Some(Event::Secret { value: None, .. }) => {}
            Some(_) | None => {
                crate::stop::unavailable(engine, b"forge records");
                return false;
            }
        }
    }
    true
}
/// Connect one held forge through the same path at startup and at runtime.
pub(crate) fn connect_one(engine: &mut crate::engine::Engine, number: u16) {
    let mut source = None;
    for forge in &engine.workers.forges {
        if forge.number == number {
            source = Some(forge);
            break;
        }
    }
    let Some(forge) = source else { return };
    let files = match super::table::record(engine, number) {
        Some(record) => record.files.is_some(),
        None => false,
    };
    if forge.kind == crate::settings::Kind::GitHub || (!files && !engine.setup.records_only(number)) {
        let kind = match forge.kind {
            crate::settings::Kind::GitHub => skein_forge_connection::settings::Kind::GitHub,
            crate::settings::Kind::Forgejo => skein_forge_connection::settings::Kind::Forgejo,
        };
        let expected = records::origin(kind, &forge.api);
        let start = match engine.forge_records.rows.forge(number) {
            Some(rows) => records::access(kind, &expected, rows, &engine.forge_records.bounds, false),
            None => Err(records::NotConnected::NoRecords),
        };
        let start = match start {
            Ok(start) => Ok(crate::forge::Start::Restored(start)),
            Err(why) => Err(why),
        };
        crate::forge::connect(engine, number, start);
    } else if engine.setup.records_only(number) || engine.setup.adopting(number) {
        // The root restore releases Reconcile; the loader never starts it.
    } else if let Some(record) = super::table::record(engine, number) {
        let source = super::table::settings_forge(record, &engine.forge.files).expect("held file form");
        if let Some(access) = crate::forge::file_access(&source) {
            crate::forge::connect(engine, number, Ok(crate::forge::Start::Adopted(access)));
        } else if let Some(host) = engine.forge.unreachable(number) {
            engine
                .notes
                .push(crate::Note::ForgeNotConnected { number, why: crate::NotConnected::Unreachable { host } });
        }
    } else if let Some(host) = engine.forge.unreachable(number) {
        engine.notes.push(crate::Note::ForgeNotConnected { number, why: crate::NotConnected::Unreachable { host } });
    }
}
