//! Application secrets and forge connections after the durable start mark (domain/root.md, 9.1).
use alloc::boxed::Box;
use jig_protocol_store::{Event, SecretOwner, Secrets};
#[cfg(test)]
use skein_lib::List;
use skein_lib::{Queue, Secret, Token};
use temper_protocol_records as records;
pub(crate) const LOAD: Token = Token::new(crate::tokens::FORGE | 1);
/// Every forge-family row survives until its record chooses the runtime (temper/docs/design/draft/playground-reset.md, 3).
#[derive(Debug)]
pub(crate) struct Rows {
    values: records::Rows,
}
impl Rows {
    fn new() -> Self {
        Self { values: records::Rows::new() }
    }
    pub(crate) fn push(&mut self, number: u64, bytes: Secret, value_bytes: u32) {
        if number < records::CARRIED && number >> 48_u32 == u64::from(records::FAMILY_FORGE) {
            self.values.push(number, bytes, value_bytes);
        }
    }
    pub(crate) fn remove(&mut self, forge: u16) {
        self.values.remove(forge);
    }
    pub(crate) fn forge(&self, forge: u16) -> Option<&records::ForgeRows> {
        self.values.forge(forge)
    }
}

#[derive(Debug)]
pub(crate) struct Loader {
    pub(crate) pending: Queue<temper_engine_domain_forge::ForgeRecord>,
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
    Connect { index: usize },
    Sweep { number: u64, asked: bool },
    Done,
}
impl Loader {
    pub(crate) fn new(limits: &crate::limits::Limits) -> Self {
        Self {
            pending: Queue::uncapped(),
            #[cfg(test)]
            inspected: List::with_capacity(128),
            rows: Rows::new(),
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
            Phase::Connect { .. } | Phase::Done => false,
        }
    }
    pub(crate) fn done(&self) -> bool {
        matches_done(&self.phase)
    }
    pub(crate) fn erasing(&self, number: u64) -> bool {
        match self.phase {
            Phase::Sweep { number: held, asked: true } => held == number,
            Phase::Sweep { asked: false, .. } | Phase::Load { .. } | Phase::Connect { .. } | Phase::Done => false,
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
        Phase::Load { .. } | Phase::Sweep { .. } | Phase::Connect { .. } => false,
    }
}
pub(crate) fn advance(engine: &mut crate::engine::Engine) {
    if engine.end.is_some() || crate::stop::requested(engine) || !engine.store.component.mark_durable() {
        return;
    }
    for _ in 0..engine.forge_records.pending.len() {
        let record = engine.forge_records.pending.pop().expect("kept forge awaiting durable wires");
        crate::forges::record(engine, record);
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
            let Some(forge) = engine.workers.forges.nth(index) else {
                engine.forge_records.phase = Phase::Done;
                return;
            };
            if engine.notes.room() == 0 || engine.events.room() < engine.forge_records.bounds.installations {
                return;
            }
            let number = forge.number;
            connect(engine, number);
            engine.forge_records.phase = Phase::Connect { index: index.checked_add(1).expect("configured forges") };
        }
        Phase::Done => {}
    }
}

/// Connect a kept runtime from its loaded records; the shell supplies document access.
pub(crate) fn connect(engine: &mut crate::engine::Engine, number: u16) {
    let Some(forge) = engine.workers.forges.get(number) else { return };
    let kind = match forge.kind {
        crate::settings::Kind::GitHub => skein_forge_connection::settings::Kind::GitHub,
        crate::settings::Kind::Forgejo => skein_forge_connection::settings::Kind::Forgejo,
    };
    if engine.forge.file_access.contains_key(&number) || engine.setup.records_only(number) {
        return;
    }
    let expected = records::origin(kind, &forge.api);
    let start = match engine.forge_records.rows.forge(number) {
        Some(rows) => records::access(kind, &expected, rows, &engine.forge_records.bounds, engine.forge.webhooks),
        None => Err(records::NotConnected::NoRecords),
    };
    let start = match start {
        Ok(start) => {
            if kind == skein_forge_connection::settings::Kind::Forgejo
                && let Some(rows) = engine.forge_records.rows.forge(number)
                && let Ok(token) = rows.get(records::Item::GitToken)
            {
                crate::forge::tokens::identity(engine, number, Secret::new(Box::from(token)));
            }
            Ok(crate::forge::Start::Restored(start))
        }
        Err(why) => Err(why),
    };
    crate::forge::connect(engine, number, start);
}
