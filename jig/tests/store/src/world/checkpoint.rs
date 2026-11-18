//! Mixed transcript files, written through kv and io before jig opens the deployment.
use super::{Restored, World, root};
use jig_protocol_store::{Write, codec, keys, transcripts};
use skein_fake_machine::Item;
use skein_io::file;
use skein_lib::{Queue, Token};
impl World {
    /// Current history routing without registering unrelated layouts in a crash scene.
    #[must_use]
    pub fn history_world() -> Self {
        const SETS: jig_protocol_store::OwnerSets = jig_protocol_store::OwnerSets {
            sets: &[jig_protocol_store::OwnerSet {
                level: None,
                jobs: &[],
                families: &[
                    jig_protocol_store::KeptFamily {
                        retired: &[],
                        types: &[],
                        frames: None,
                        ..jig_protocol_store::STORE_FAMILIES[3]
                    },
                    jig_protocol_store::KeptFamily {
                        retired: &[],
                        types: &[],
                        ..jig_protocol_store::STORE_FAMILIES[4]
                    },
                ],
                ..jig_protocol_store::JIG_STORE_SET
            }],
        };
        let mut limits = root::limits();
        limits.store.record = limits.store.page.bytes;
        let mut world = Self::unopened_observed(0, limits, None);
        world.owner_sets(SETS);
        assert_eq!(world.open(), Some(0));
        assert_eq!(world.commit(root::first()), Some(1));
        world
    }
    /// Two files, one still writable and one ended, and a missing file's places.
    #[must_use]
    pub fn mixed_files_checkpoint(seed: u64) -> Self {
        let mut world = Self::unopened(seed);
        let limits = world.env.limits;
        let codec = limits.codec_view().expect("fixture codec");
        let live = root::LIVE;
        let ended = live + 1;
        let rows = vec![root::header(1), root::run_proof(live), root::person(1)];
        let mut ops = vec![
            skein_kv::Op::Put {
                key: Box::new([0, 0]),
                value: jig_protocol_store::format::current(root::APPLICATION.version, root::DEPLOYMENT)
                    .encode()
                    .expect("format"),
            },
        ];
        let floor = jig_protocol_store::component_v3::Floor::new(
            &jig_protocol_store::component_v3::CEILINGS,
            jig_protocol_store::component_v3::FloorParts { number: jig_protocol_store::FLOOR, build: Box::default() },
        )
        .expect("fixture floor");
        let mut writer = skein_lib::Writer::new(usize::try_from(floor.measure()).expect("floor size"));
        floor.encode(&mut writer).expect("floor bytes");
        ops.push(skein_kv::Op::Put {
            key: keys::encode_own(&keys::Own::Floor, limits.store.key).expect("floor key"),
            value: writer.finish(),
        });
        for row in &rows {
            ops.push(skein_kv::Op::Put {
                key: keys::encode(&row.key(), limits.store.key).expect("fixture key"),
                value: codec::encode(row, &codec, &[]).expect("fixture record"),
            });
        }
        let mut items = vec![
            Item::file(b"active", b"1\n"),
            Item::directory(b"generations"),
            Item::directory(b"generations/1"),
            Item::directory(b"generations/1/store"),
            Item::directory(b"generations/1/secrets"),
            Item::directory(b"transcripts"),
            Item::directory(b"transcripts/0000000000000000"),
        ];
        let mut model_writes: Vec<_> = rows.into_iter().map(Write::Save).collect();
        for task in [live, ended] {
            let turn = root::turn(task, 1, 1, b"mixed files at C");
            let bytes = transcripts::frame::encode(&turn, &codec).expect("mixed files frame");
            let place =
                codec::component::Transcript { length: u64::try_from(bytes.len()).expect("frame size"), window: 0 };
            let path = format!("transcripts/{:016x}/{task:016x}", task >> 12).into_bytes();
            items.push(Item::file(&path, &bytes));
            ops.push(skein_kv::Op::Put {
                key: keys::encode_own(&keys::Own::Transcript { task }, limits.store.key).expect("places key"),
                value: place.encode().expect("places"),
            });
            model_writes.push(Write::Save(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Turn(turn))));
        }
        ops.push(skein_kv::Op::Put {
            key: keys::encode_own(&keys::Own::Transcript { task: ended + 1 }, limits.store.key)
                .expect("missing places key"),
            value: codec::component::Transcript { length: 81, window: 0 }.encode().expect("missing places"),
        });
        for item in &mut items {
            item.mode = if matches!(item.made, skein_fake_machine::Made::Directory) { 0o700 } else { 0o600 };
        }
        ops.push(skein_kv::Op::Put {
            key: keys::installation(limits.store.key).expect("installation key"),
            value: codec::accounts::Installation { id: [1; 16] }.encode(),
        });
        world.lay(&items);
        world.seed_checkpoint_map(ops.into_boxed_slice());
        world.model.stage(1, &model_writes);
        world.model.acknowledge(1);
        assert_eq!(world.open(), Some(1), "mixed files open: {:?}, {:?}", world.refusal(), world.start_open_failure());
        world.next = 2;
        world.mark_if_needed();
        world
    }
    fn seed_checkpoint_map(&mut self, ops: Box<[skein_kv::Op]>) {
        let arrived = self
            .file_terminal(
                0,
                file::Request::OpenDirectory {
                    owner: Token::new(71),
                    root: self.root.fd,
                    name: Box::from(b"generations/1/store".as_slice()),
                    no_follow: true,
                },
            )
            .expect("fixture root");
        let fd = arrived.fd.expect("fixture directory descriptor");
        let file::Event::Opened { file, .. } = arrived.event else {
            panic!("fixture directory");
        };
        let mut kv = skein_kv::Store::new(&self.env.limits.store);
        for request in [
            skein_kv::Request::Open { owner: Token::new(72), root: fd },
            skein_kv::Request::Commit { owner: Token::new(73), ops },
            skein_kv::Request::Close { owner: Token::new(74) },
        ] {
            let mut above = Queue::uncapped();
            let mut below = Queue::uncapped();
            skein_kv::down(&mut kv, &self.env.with(self.env.limits.store), request, &mut above, &mut below);
            for _ in 0..1000 {
                if skein_kv::is_ready(&kv, &self.env.limits.store) {
                    skein_kv::fire(&mut kv, &self.env.with(self.env.limits.store), &mut above, &mut below);
                }
                if let Some(request) = below.pop() {
                    let arrived = self.file_terminal(0, request).expect("fixture file");
                    skein_kv::up(&mut kv, &self.env.with(self.env.limits.store), arrived.event, &mut above, &mut below);
                } else if !above.is_empty() {
                    break;
                }
            }
            for event in &above {
                assert!(
                    matches!(
                        event,
                        skein_kv::Event::Opened { .. }
                            | skein_kv::Event::Committed { .. }
                            | skein_kv::Event::Closed { .. }
                    ),
                    "fixture kv terminal: {event:?}"
                );
            }
        }
        self.file_terminal(0, file::Request::Close { owner: Token::new(75), file }).expect("fixture directory closed");
    }
    /// Discover every transcript from places, including ended tasks' older windows.
    pub fn checkpoint_reopen(&mut self) -> Restored {
        let mut restored = self.reopen();
        for (key, _) in self.retained() {
            if let Some(keys::Own::Transcript { task } | keys::Own::TurnLog { task }) = keys::decode_own(&key) {
                let turns =
                    self.transcript(jig_protocol_store::TranscriptRead::Page { task, after: None, backwards: false });
                restored.transcripts.insert(task, turns);
            }
        }
        restored
    }
}
