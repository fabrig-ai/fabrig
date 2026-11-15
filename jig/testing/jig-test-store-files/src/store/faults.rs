//! Corrupt in-memory log bodies through a rebuilt KV fixture, without changing its typed mirror.
use super::{INSTALLATION, Queue, Request, Store, Token, VecDeque, protocol};
use jig_protocol_store::keys;

impl Store {
    #[expect(
        clippy::too_many_lines,
        reason = "the fault rebuilds one memory KV image while preserving its root commit number and typed mirror"
    )]
    pub(super) fn unread_memory_turn(&mut self, task: u64, at: u64) {
        let mut inventory = self.retained().expect("inject between fixture requests");
        // Some stand-alone codec scenes omit deployment headers on their turn
        // commits. A fixture reopen still needs the recovered number to agree.
        let key = jig_domain_core::Key::Core(jig_domain_core::CoreKey::Deployment);
        if let Some(jig_test_domain::Record::Core(jig_domain_core::Record::Core(
            jig_domain_core::CoreRecord::Deployment(header),
        ))) = self.rows.get(&jig_test_domain::Key::Core(key.clone()))
        {
            let mut header = *header;
            header.commits = self.applied;
            let row = jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(header));
            inventory.insert(
                keys::encode(&key, self.component_limits.store.key).expect("header key"),
                protocol::codec::encode(
                    &row,
                    &protocol::codec::limits(&self.component_limits.core).expect("codec"),
                    &[],
                )
                .expect("header"),
            );
        }
        let legacy =
            keys::encode_own(&keys::Own::Transcript { task }, self.component_limits.store.key).expect("file place");
        if inventory.contains_key(&legacy) {
            self.files.unread_transcript(task);
            return;
        }
        let mut old = self.component.take().expect("component").into_stores();
        let limits = self.component_limits.store;
        let env = self.env().with(limits);
        let mut records = Vec::new();
        for key in inventory.keys().filter(|key| key.starts_with(&protocol::logs::EXTENTS)) {
            let log = skein_lib::Reader::new(&key[2..]).u64().expect("log name");
            let mut position = 0;
            loop {
                let mut above = Queue::uncapped();
                let mut below = Queue::uncapped();
                skein_kv::down(
                    &mut old.store,
                    &env,
                    skein_kv::Request::ReadLog {
                        owner: Token::new(0),
                        log,
                        at: position,
                        direction: skein_kv::Direction::Forward,
                        max: limits.page.bytes,
                    },
                    &mut above,
                    &mut below,
                );
                assert!(below.is_empty());
                let Some(skein_kv::Event::LogRead { records: page, next, .. }) = above.pop() else {
                    panic!("memory log read")
                };
                for record in page {
                    let mut body = record.record.expect("whole admitted record");
                    if log == protocol::logs::turns(task).expect("turns log") && record.at == at {
                        body[..2].copy_from_slice(&u16::MAX.to_be_bytes());
                    }
                    records.push((log, body));
                }
                let Some(next) = next else { break };
                position = next;
            }
        }
        let maps = protocol::Stores { store: skein_kv::Store::memory(&limits), secrets: old.secrets };
        let mut maps = protocol::Store::over(
            &self.component_limits,
            jig_test_store::application(&self.limits.connector),
            &self.models,
            maps,
        )
        .into_stores();
        let mut ops: Vec<_> = inventory
            .into_iter()
            .filter(|(key, _)| !key.starts_with(&protocol::logs::EXTENTS))
            .map(|(key, value)| skein_kv::Op::Put { key, value })
            .collect();
        for (log, body) in records {
            ops.push(skein_kv::Op::Append { log, frame: maps.store.frame(&body).expect("admitted record") });
        }
        // Preserve the root's commit number and its next write. Empty commits
        // precede the complete retained map and the logs it indexes.
        for number in 1..=self.applied {
            let mut above = Queue::uncapped();
            let mut below = Queue::uncapped();
            let writes =
                if number == self.applied { core::mem::take(&mut ops).into_boxed_slice() } else { Box::new([]) };
            skein_kv::down(
                &mut maps.store,
                &env,
                skein_kv::Request::Commit { owner: Token::new(number), ops: writes },
                &mut above,
                &mut below,
            );
            assert!(below.is_empty());
            let Some(skein_kv::Event::Committed { number: got, .. }) = above.pop() else {
                panic!("rebuild corruption fixture: {above:?}")
            };
            assert_eq!(got, number);
        }
        let mut component = protocol::Store::over(
            &self.component_limits,
            jig_test_store::application(&self.limits.connector),
            &self.models,
            maps,
        );
        component.patch_sets(self.sets);
        assert!(component.wires(self.wires));
        self.component = Some(component);
        let root = self.files.root();
        self.drive(
            VecDeque::from([Request::Open { owner: Token::new(0), root, installation: INSTALLATION }]),
            Queue::uncapped(),
        );
    }
}
