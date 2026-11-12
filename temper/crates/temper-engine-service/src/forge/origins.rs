//! Bounded volatile origin diagnostics (temper/docs/design/protocol/forge/README.md, 2.6).
use alloc::boxed::Box;
use skein_forge::github::redirect::Origin;
use skein_lib::{Decimal, Map, Queue, Writer};

pub(crate) struct Origins {
    visible: Map<u16, Queue<Origin>>,
    said: Map<u16, Queue<Origin>>,
    bound: u32,
}
impl Origins {
    pub(crate) fn new(forges: u32, bound: u32) -> Self {
        Self { visible: Map::with_capacity(forges), said: Map::with_capacity(forges), bound }
    }
    pub(crate) fn clear(&mut self, forge: u16) {
        self.visible.remove(&forge);
    }
    pub(crate) fn forget(&mut self, forge: u16) {
        self.visible.remove(&forge);
        self.said.remove(&forge);
    }
    fn keep(table: &mut Map<u16, Queue<Origin>>, forge: u16, origin: &Origin, bound: u32) -> bool {
        if !table.contains_key(&forge) {
            table.insert(forge, Queue::with_capacity(bound)).expect("configured forge");
        }
        let origins = table.get_mut(&forge).expect("forge origins");
        for held in origins.iter() {
            if held.same(origin) {
                return false;
            }
        }
        if origins.room() == 0 {
            drop(origins.pop());
        }
        origins.push(origin.clone());
        true
    }
    pub(crate) fn note(&mut self, forge: u16, origin: &Origin) -> bool {
        Self::keep(&mut self.visible, forge, origin, self.bound);
        if let Some(said) = self.said.get(&forge)
            && said.room() == 0
        {
            return false;
        }
        Self::keep(&mut self.said, forge, origin, self.bound)
    }
    pub(crate) fn rows(&self, mut forges: temper_engine_domain_forge::Forges) -> temper_engine_domain_forge::Forges {
        for row in &mut forges.rows {
            if row.ended.is_none()
                && let Some(origins) = self.visible.get(&row.number)
                && let Some(origin) = origins.iter().last()
            {
                let line = text(origin);
                row.ended = Some(Box::from(
                    line.get(..line.len().min(temper_engine_domain_forge::CONNECT_WHY_BYTES))
                        .expect("row diagnostic bound"),
                ));
            }
        }
        forges
    }
}

pub(crate) fn refused(engine: &mut crate::engine::Engine, forge: u16, origin: Origin) {
    if engine.workers.forges.get(forge).is_none() {
        return;
    }
    if engine.forge.origins.note(forge, &origin) {
        engine.notes.push(crate::Note::ForgeOrigin { number: forge, origin });
    }
    engine.domain.mark_forges();
}

pub(crate) fn text(origin: &Origin) -> Box<[u8]> {
    let scheme: &[u8] = if origin.tls { b"https://" } else { b"http://" };
    let port = Decimal::of(u64::from(origin.port));
    let prefix = b"Job-log origin refused: ";
    let size = prefix.len().checked_add(scheme.len()).expect("diagnostic scheme");
    let size = size.checked_add(origin.host.len()).expect("diagnostic host");
    let size = size.checked_add(1).expect("diagnostic separator");
    let size = size.checked_add(port.as_bytes().len()).expect("diagnostic port");
    let mut out = Writer::new(size);
    out.put(prefix).expect("diagnostic prefix");
    out.put(scheme).expect("scheme");
    out.put(&origin.host).expect("host");
    out.put(b":").expect("port separator");
    out.put(port.as_bytes()).expect("port");
    out.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn origin(host: &[u8]) -> Origin {
        Origin { tls: true, host: host.into(), port: 443 }
    }
    #[test]
    fn origin_memory_stays_bounded_across_connects_and_forgets_removed_forges() {
        let mut notes = Origins::new(1, 2);
        let first = origin(b"first.test");
        let second = origin(b"second.test");
        let third = origin(b"third.test");
        assert!(notes.note(1, &first));
        assert!(notes.note(1, &second));
        assert!(!notes.note(1, &third));
        assert_eq!(notes.visible.get(&1).unwrap().len(), 2);
        assert_eq!(notes.said.get(&1).unwrap().len(), 2);
        notes.clear(1);
        assert!(!notes.note(1, &first));
        assert_eq!(notes.visible.get(&1).unwrap().len(), 1);
        notes.forget(1);
        assert!(notes.note(2, &third), "a removed forge releases both tables");
    }
}
