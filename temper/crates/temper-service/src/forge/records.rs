//! Application secrets and forge connections after the durable start mark (domain/root.md, 9.1).
use alloc::boxed::Box;
use jig_protocol_store::{Event, SecretOwner, Secrets};
#[cfg(test)]
use skein_lib::List;
use skein_lib::{Map, Secret, Token};
use temper_protocol_records as records;
pub(crate) const LOAD: Token = Token::new(crate::tokens::FORGE | 1);
const FORGES: u32 = 65536;
/// The service retains values only for its registered forges. The codec's
/// general-purpose Rows is filled solely through this holder's checked push.
#[derive(Debug)]
pub(crate) struct Rows {
    values: records::Rows,
    configured: Map<u16, ()>,
    unconfigured: Map<u16, ()>,
}
impl Rows {
    fn new(capacity: u32) -> Self {
        Self { values: records::Rows::new(), configured: Map::with_capacity(capacity), unconfigured: Map::with_capacity(FORGES) }
    }
    pub(crate) fn register(&mut self, forge: u16) {
        self.configured.insert(forge, ()).expect("admitted forge");
        self.unconfigured.remove(&forge);
    }
    pub(crate) fn unregister(&mut self, forge: u16) {
        self.configured.remove(&forge);
        self.unconfigured.remove(&forge);
        self.values.drop_forge(forge);
    }
    pub(crate) fn push(&mut self, number: u64, bytes: Secret, value_bytes: u32) {
        if number >= records::CARRIED || number >> 48_u32 != u64::from(records::FAMILY_FORGE) {
            return;
        }
        let Ok(forge) = u16::try_from((number >> 16_u32) & 0xffff_ffff) else { return };
        if !self.configured.contains_key(&forge) {
            self.unconfigured.insert(forge, ()).expect("all u16 forge numbers fit");
            return;
        }
        // Decode temper-secrets here; get() serves skein setup bytes to every reader.
        self.values.push(number, bytes, value_bytes);
    }
    pub(crate) fn forge(&self, forge: u16) -> Option<&records::ForgeRows> {
        self.values.forge(forge)
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
    pub(crate) fn new(limits: &crate::limits::Limits) -> Self {
        Self {
            #[cfg(test)]
            inspected: List::with_capacity(128),
            rows: Rows::new(limits.forge_component.forges),
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
    pub(crate) fn loading(&self) -> bool {
        match self.phase {
            Phase::Load { .. } | Phase::Sweep { .. } => true,
            Phase::Connect { .. } | Phase::Name { .. } | Phase::Done => false,
        }
    }
    pub(crate) fn passed(&self, index: u32) -> bool {
        match self.phase {
            Phase::Connect { index: next } => index < next,
            Phase::Name { .. } | Phase::Done => true,
            Phase::Load { .. } | Phase::Sweep { .. } => false,
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
            if engine.resolving.contains_key(&number) {
                return;
            }
            // A refused resolve already named the registered held record once.
            if !engine.records.contains_key(&number) || engine.forge.unreachable(number).is_none() {
                connect(engine, number);
            }
            engine.forge_records.phase = Phase::Connect { index: index.checked_add(1).expect("configured forges") };
        }
        Phase::Name { after } => {
            let rows = &engine.forge_records.rows;
            let found = rows.next_unconfigured(after);
            match found {
                Some(number) => {
                    if engine.notes.room() == 0 {
                        return;
                    }
                    engine.notes.push(crate::Note::ForgeUnconfigured { number });
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
    let mut rows = Rows::new(settings.limits.forge_component.forges);
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
    rows.register(1);
    rows.push(records::number(1, records::Item::User), Secret::new(Box::from([1, 9])), 4096);
    assert!(rows.forge(1).is_some());
    assert_eq!(rows.unconfigured.len(), 4);
}

/// Apply the loader's connection rule to one registered forge.
pub(crate) fn connect(engine: &mut crate::engine::Engine, number: u16) {
    let mut configured = None;
    for forge in &engine.workers.forges {
        if forge.number == number {
            configured = Some(forge);
            break;
        }
    }
    let Some(forge) = configured else { return };
    if forge.kind == crate::settings::Kind::GitHub
        || engine.records.contains_key(&number)
            && !engine.setup.records_only(number)
            && engine.records.get(&number).expect("registered record").setup.is_none()
    {
        let kind = match forge.kind {
            crate::settings::Kind::GitHub => skein_forge_connection::settings::Kind::GitHub,
            crate::settings::Kind::Forgejo => skein_forge_connection::settings::Kind::Forgejo,
        };
        let expected = records::origin(kind, &forge.api);
        let start = match engine.forge_records.rows.forge(number) {
            Some(rows) => {
                let webhooks =
                    kind == skein_forge_connection::settings::Kind::Forgejo && rows.has(records::Item::WebhookSecret);
                records::access(kind, &expected, rows, &engine.forge_records.bounds, webhooks)
            }
            None => Err(records::NotConnected::NoRecords),
        };
        let start = match start {
            Ok(start) => Ok(crate::forge::Start::Restored(start)),
            Err(why) => Err(why),
        };
        if start.is_ok()
            && kind == skein_forge_connection::settings::Kind::Forgejo
            && let Some(rows) = engine.forge_records.rows.forge(number)
            && rows.has(records::Item::GitToken)
        {
            let record = engine.records.get(&number).expect("registered Forgejo");
            crate::setup::lifecycle::publish_identity(
                &mut crate::forges::identity(record),
                rows,
                &engine.forge_records.bounds,
                &engine.workers.encoding.limits,
                &mut engine.values.identities,
                &mut engine.events,
            );
        }
        crate::forge::connect(engine, number, start);
    } else if engine.setup.records_only(number) {
        // The root restore releases Reconcile; the loader never starts it.

    }
}
