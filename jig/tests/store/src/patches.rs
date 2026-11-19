//! Equal-level fixture binaries and their independent record model (protocol/store/README.md 9).
use crate::{World, root, world::Change};
use jig_protocol_store::{Event, KeptFamily, KeptMap, Keys, Owner, OwnerSet, OwnerSets, SecretOwner, Secrets, Write};
use jig_test_layouts::{self as fixture, records as app};
use jig_test_patches_jig::{self as upstream, patches as jig};
use skein_lib::Secret;
use std::collections::BTreeMap;
const LABEL: KeptFamily = KeptFamily {
    retired: &[],
    current: app::LABEL,
    types: &[jig_protocol_store::ValueType {
        number: 0,
        root: "Label",
        variant: None,
        keys: &[jig_protocol_store::KeyRange { from: fixture::keys::LABELS_FROM, to: fixture::keys::LABELS_TO }],
    }],
    frames: None,
    name: "fixture-label",
    map: KeptMap::Store,
    keys: &[Keys { from: fixture::keys::LABELS_FROM, to: fixture::keys::LABELS_TO, root: fixture::keys::LABELS_ROOT }],
};
const SHELF: KeptFamily = KeptFamily {
    retired: &[],
    current: app::SHELF,
    types: &[jig_protocol_store::ValueType {
        number: 0,
        root: "Shelf",
        variant: None,
        keys: &[jig_protocol_store::KeyRange { from: fixture::keys::SHELVES_FROM, to: fixture::keys::SHELVES_TO }],
    }],
    frames: None,
    name: "fixture-shelf",
    map: KeptMap::Store,
    keys: &[Keys {
        from: fixture::keys::SHELVES_FROM,
        to: fixture::keys::SHELVES_TO,
        root: fixture::keys::SHELVES_ROOT,
    }],
};
const TOKEN: KeptFamily = KeptFamily {
    retired: &[],
    current: app::TOKEN,
    types: &[],
    frames: None,
    name: "fixture-token",
    map: KeptMap::Secrets,
    keys: &[Keys { from: fixture::keys::TOKENS_FROM, to: fixture::keys::TOKENS_TO, root: fixture::keys::TOKENS_ROOT }],
};
const NOTE: KeptFamily = KeptFamily {
    retired: &[],
    current: jig::kept::fixture_note::TEXTS[0],
    types: &[jig_protocol_store::ValueType {
        number: 0,
        root: "Note",
        variant: None,
        keys: &[jig_protocol_store::KeyRange { from: upstream::keys::NOTES_FROM, to: upstream::keys::NOTES_TO }],
    }],
    frames: None,
    name: "fixture-note",
    map: KeptMap::Store,
    keys: &[Keys { from: upstream::keys::NOTES_FROM, to: upstream::keys::NOTES_TO, root: upstream::keys::NOTES_ROOT }],
};
const APPLICATION: OwnerSet = OwnerSet {
    jobs: &[],
    owner: Owner::Application,
    name: fixture::NAME,
    families: &[LABEL, SHELF, TOKEN],
    level: Some(0),
};
const JIG: OwnerSet =
    OwnerSet { jobs: &[], owner: Owner::Jig, name: upstream::NAME, families: &[NOTE], level: Some(0) };
/// A read-only fixture job makes an upgrade without changing any layout or level
/// (protocol/store/jobs.md, 1–3). Each test's source omits its done row.
pub const JOB: jig_protocol_store::Job = jig_protocol_store::Job {
    name: "inspect-fixtures",
    keys: &[jig_protocol_store::KeyRange { from: b"s", to: b"t" }],
    converts: None,
    counts: &[],
};
pub const BASE: OwnerSets = OwnerSets { sets: &[APPLICATION, JIG] };
pub const SETS: OwnerSets = OwnerSets { sets: &[OwnerSet { jobs: &[JOB], ..APPLICATION }, JIG] };
/// A distinct pending job makes the next upgrade, still at the same levels.
pub const NEXT: OwnerSets = OwnerSets {
    sets: &[
        OwnerSet { jobs: &[JOB, jig_protocol_store::Job { name: "inspect-fixtures-again", ..JOB }], ..APPLICATION },
        JIG,
    ],
};

/// The fixture consumes each page without rewriting rows. Tests inspect the
/// copy, not a conversion invented to reach it.
#[must_use]
pub fn job(of: Owner, name: &str, page: &jig_protocol_store::JobPage) -> Option<jig_protocol_store::JobAnswer> {
    if of != Owner::Application || ![JOB.name, NEXT.sets[0].jobs[1].name].contains(&name) {
        return None;
    }
    assert!(page.layout.is_none());
    Some(jig_protocol_store::JobAnswer { writes: Box::new([]), through: None, counts: Box::new([]) })
}

/// Fixture rows need 64 bytes; 1024 also leaves room for the component's
/// format, registry and deployment header. Jobs consume bounded pages.
#[must_use]
pub fn limits() -> jig_protocol_store::Limits {
    let mut limits = root::limits();
    limits.commits.writes = 32;
    for map in [&mut limits.store, &mut limits.secrets] {
        map.key = 64;
        map.value = 1024;
        map.ops = 100;
        map.commit = 4096;
        map.queued_bytes = 8192;
        map.chunk = 4096;
        map.page.bytes = 2048;
        map.record = map.page.bytes;
    }
    limits.secrets.ops = 2;
    limits
}

/// Two labels, three shelves, two notes and three tokens.
pub const STORY_LABELS: u32 = 2;
pub const STORY_SHELVES: u32 = 3;
pub const STORY_NOTES: u32 = 2;
pub const STORY_TOKENS: u64 = 3;

#[derive(Clone, Debug)]
pub struct Shelf {
    pub key: Vec<u8>,
    pub name: Vec<u8>,
    pub books: u32,
    pub caption: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct Records {
    pub labels: Vec<(Vec<u8>, Vec<u8>)>,
    pub shelves: Vec<Shelf>,
    pub notes: Vec<(Vec<u8>, Vec<u8>)>,
    pub tokens: Vec<(u64, Vec<u8>)>,
}

fn text(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend(u32::try_from(bytes.len()).expect("fixture text length").to_be_bytes());
    out.extend(bytes);
}
fn label(caption: &[u8]) -> Vec<u8> {
    let mut out = 1_u16.to_be_bytes().to_vec();
    text(&mut out, caption);
    out
}

fn token(bytes: &[u8], issued: u64) -> Vec<u8> {
    let mut out = 1_u16.to_be_bytes().to_vec();
    text(&mut out, bytes);
    out.extend(issued.to_be_bytes());
    out
}

impl Records {
    #[must_use]
    pub fn story() -> Self {
        Self {
            labels: (0..STORY_LABELS).map(|n| (format!("l/{n}").into_bytes(), b"caption".to_vec())).collect(),
            shelves: (0..STORY_SHELVES)
                .map(|n| Shelf {
                    key: format!("s/{n}").into_bytes(),
                    name: b"shelf".to_vec(),
                    books: n + 1,
                    caption: b"held".to_vec(),
                })
                .collect(),
            tokens: (0..STORY_TOKENS).map(|n| (n, b"fixture credential".to_vec())).collect(),
            notes: (0..STORY_NOTES).map(|n| (format!("n/{n}").into_bytes(), b"note".to_vec())).collect(),
        }
    }

    /// Independent wire model for the fixture layouts.
    #[must_use]
    pub fn rows(&self) -> BTreeMap<Box<[u8]>, Box<[u8]>> {
        let mut rows = BTreeMap::new();
        for (key, caption) in &self.labels {
            rows.insert(key.clone().into_boxed_slice(), label(caption).into_boxed_slice());
        }
        for Shelf { key, name, books, caption } in &self.shelves {
            let mut out = 1_u16.to_be_bytes().to_vec();
            text(&mut out, name);
            out.extend(books.to_be_bytes());
            text(&mut out, &label(caption));
            rows.insert(key.clone().into_boxed_slice(), out.into_boxed_slice());
        }
        for (key, content) in &self.notes {
            let mut out = 1_u16.to_be_bytes().to_vec();
            text(&mut out, content);
            rows.insert(key.clone().into_boxed_slice(), out.into_boxed_slice());
        }
        rows
    }

    #[must_use]
    pub fn writes(&self) -> Box<[Write]> {
        let mut writes = vec![Write::Save(root::header(1)), Write::Wires];
        writes.extend(self.rows().into_iter().map(|(key, value)| Write::Put { key, value }));
        writes.into_boxed_slice()
    }

    #[must_use]
    pub fn committed(&self, seed: u64, sets: OwnerSets) -> World {
        self.committed_with(seed, sets, limits(), &[])
    }

    /// Base rows and limits for the check's honest refusal scenarios.
    #[must_use]
    pub fn committed_with(
        &self,
        seed: u64,
        sets: OwnerSets,
        limits: jig_protocol_store::Limits,
        rows: &[jig_domain_core::Record],
    ) -> World {
        let mut world = World::unopened_observed(seed, limits, None);
        world.owner_sets(sets);
        assert_eq!(world.open_switched(), Some(0));
        let mut writes = self.writes().into_vec();
        writes.extend(rows.iter().cloned().map(Write::Save));
        assert_eq!(world.commit(writes.into_boxed_slice()), Some(1));
        for (number, bytes) in &self.tokens {
            assert!(*number < (1_u64 << 63));
            let events = world.secret(Secrets::Keep {
                owner: SecretOwner::Application,
                number: *number,
                value: jig_protocol_store::SecretValue::Application(Secret::new(
                    token(bytes, *number).into_boxed_slice(),
                )),
            });
            assert!(matches!(events.as_slice(), [Event::Kept { .. }]), "token kept: {events:?}");
        }
        world.crash(seed);
        world
    }

    /// Decode by each family's kept text as well as compare every modeled byte.
    pub fn check(&self, world: &mut World, context: &dyn std::fmt::Debug) {
        world.mark_if_needed();
        let schema = skein_codec::schema::parse(app::TOKEN.as_bytes()).expect("token schema");
        let root = schema.find(b"Token").expect("token root");
        for (number, bytes) in &self.tokens {
            let events = world.secret(Secrets::Get { owner: SecretOwner::Application, number: *number });
            let [Event::Secret { value: Some(jig_protocol_store::SecretValue::Application(value)), .. }] =
                events.as_slice()
            else {
                panic!("token {number}: {events:?}, {context:?}");
            };
            assert_eq!(value.expose(), token(bytes, *number), "token {number}, {context:?}");
            let mut decoded = skein_codec::value::Value::with_room(16);
            decoded.decode(&schema, root, Box::from(value.expose())).expect("token decodes at its level");
        }
        self.check_public(world, context);
    }

    /// The copy's public restore is checked before the start can serve secrets.
    pub fn check_public(&self, world: &mut World, context: &dyn std::fmt::Debug) {
        let rows = world.application(false);
        assert_eq!(rows, self.rows(), "all fixture records are preserved: {context:?}");
        let label = skein_codec::schema::parse(app::LABEL.as_bytes()).expect("label schema");
        let shelves = skein_codec::schema::parse(app::SHELF.as_bytes()).expect("shelf schema");
        let note = skein_codec::schema::parse(jig::kept::fixture_note::TEXTS[0].as_bytes()).expect("note schema");
        for (key, bytes) in rows {
            let (schema, root) = match key[0] {
                b'l' => (&label, "Label"),
                b's' => (&shelves, "Shelf"),
                b'n' => (&note, "Note"),
                _ => panic!("fixture key"),
            };
            let root = schema.find(root.as_bytes()).expect("record root");
            let mut value = skein_codec::value::Value::with_room(16);
            value.decode(schema, root, bytes).expect("record decodes at its level");
        }
    }
}

/// All mutations from the copy's first mkdir through active's final sync.
/// The last cut also covers power loss after the switch's rename.
/// The ordinals name kernel calls, including durable replace's internals.
#[must_use]
pub fn cuts(world: &World) -> Vec<u32> {
    let changes = world.changes();
    let first = changes.iter().position(|(_, change)| *change == Change::Make).expect("copy mkdir");
    let last = changes.iter().rposition(|(_, change)| *change == Change::Rename).expect("switch rename");
    assert!(first < last);
    assert_eq!(changes[last + 1].1, Change::Sync, "switch directory sync follows its rename");
    changes[first..=last + 1].iter().map(|(operation, _)| *operation).collect()
}

/// The operation of the first change of this kind in a traced world.
#[must_use]
pub fn first_change(world: &World, kind: Change) -> u32 {
    world
        .changes()
        .iter()
        .find(|(_, change)| *change == kind)
        .unwrap_or_else(|| panic!("first {kind:?} change reached"))
        .0
}

/// The operation of the last change of this kind in a traced world.
#[must_use]
pub fn last_change(world: &World, kind: Change) -> u32 {
    world
        .changes()
        .iter()
        .rfind(|(_, change)| *change == kind)
        .unwrap_or_else(|| panic!("last {kind:?} change reached"))
        .0
}

pub fn whole(world: &mut World) {
    assert_eq!(world.active(), b"2\n");
    let paths = world.paths();
    let generations: Vec<_> = paths
        .iter()
        .filter(|path| path.starts_with(b"generations/") && skein_lib::bytes::count(path, b"/", 2) == 1)
        .map(Vec::as_slice)
        .collect();
    assert_eq!(generations, [b"generations/1".as_slice(), b"generations/2".as_slice()]);
    assert!(paths.iter().any(|path| path == b"generations/2/manifest"));
    assert!(!paths.iter().any(|path| path.windows(7).any(|part| part == b".skein-")));
}

/// The previous generation is an opaque, byte-for-byte rollback image.
#[must_use]
pub fn source(world: &mut World) -> BTreeMap<Vec<u8>, Option<Vec<u8>>> {
    world.contents().into_iter().filter(|(path, _)| path.starts_with(b"generations/1/")).collect()
}

/// A start leaves every file it found as it was; the one write before its
/// checks is skein-kv's recovery, which may open an empty log segment in
/// each map it reads (protocol/store/opening.md 1, step 4).
pub fn kept(
    before: &BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    after: &BTreeMap<Vec<u8>, Option<Vec<u8>>>,
    what: &impl std::fmt::Debug,
) {
    for (path, bytes) in before {
        assert_eq!(after.get(path), Some(bytes), "{} changed, {what:?}", String::from_utf8_lossy(path));
    }
    for (path, bytes) in after {
        if !before.contains_key(path) {
            let segment = path
                .strip_prefix(b"generations/1/".as_slice())
                .and_then(|map| map.strip_prefix(b"store/log-").or_else(|| map.strip_prefix(b"secrets/log-")));
            assert!(
                segment.is_some_and(|digits| digits.len() == 20 && digits.iter().all(u8::is_ascii_digit))
                    && bytes.as_deref() == Some(&[][..]),
                "{} is new and not an empty recovery segment, {what:?}",
                String::from_utf8_lossy(path)
            );
        }
    }
}

/// The story reads the component's manifest through its public codec.
#[must_use]
pub fn counts(world: &mut World) -> Vec<(Vec<u8>, u64, u64)> {
    let contents = world.contents();
    let bytes = contents
        .get(b"generations/2/manifest".as_slice())
        .expect("upgrade manifest")
        .as_ref()
        .expect("manifest is a file");
    let manifest = jig_protocol_store::codec::component::Manifest::decode(bytes).expect("manifest codec");
    manifest.counts.iter().map(|count| (count.family.to_vec(), count.before, count.after)).collect()
}

/// Frame patches are used only at read; opens compare equal owner levels.
pub const FRAME_SETS: OwnerSets = OwnerSets {
    sets: &[
        OwnerSet {
            jobs: &[],
            owner: Owner::Jig,
            name: jig_test_patches_jig::NAME,
            level: Some(2),
            families: &[KeptFamily {
                retired: &[],
                current: jig::kept::fixture_frame::TEXTS.last().expect("kept text"),
                types: &[],
                name: "fixture-frame",
                map: KeptMap::Store,
                keys: &[],
                frames: Some(jig_protocol_store::Frames {
                    root: jig_test_patches_jig::FRAME_ROOT,
                    texts: jig::kept::fixture_frame::TEXTS,
                    patches: jig::PATCHES,
                }),
            }],
        },
        OwnerSet {
            jobs: &[],
            owner: Owner::Smith,
            name: jig_test_patches_smith::NAME,
            level: Some(1),
            families: &[KeptFamily {
                retired: &[],
                current: jig_test_patches_smith::patches::kept::fixture_turn::TEXTS.last().expect("kept text"),
                types: &[],
                name: "fixture-turn",
                map: KeptMap::Store,
                keys: &[],
                frames: Some(jig_protocol_store::Frames {
                    root: jig_test_patches_smith::ROOT,
                    texts: jig_test_patches_smith::patches::kept::fixture_turn::TEXTS,
                    patches: jig_test_patches_smith::patches::PATCHES,
                }),
            }],
        },
    ],
};
/// The frame fixture declares its job only on the upgrading open.
pub const FRAME_BASE: OwnerSets = OwnerSets { sets: &[FRAME_SETS.sets[0], FRAME_SETS.sets[1], APPLICATION] };
pub const FRAME_JOBS: OwnerSets = OwnerSets { sets: &[FRAME_SETS.sets[0], FRAME_SETS.sets[1], SETS.sets[0]] };

#[must_use]
pub fn frame_limits() -> jig_protocol_store::Limits {
    let mut limits = limits();
    limits.commits.writes = 4;
    limits
}

/// Six proofs force two crossing pages at the world's four-put journal.
/// Five have windows; every task has two committed fixture turns.
pub const FRAME_WINDOWS: &[bool] = &[true, true, true, true, true, false];
pub const FRAME_SAID: &[u8] = b"said";
pub const FRAME_REPLAY: &[u8] = b"replay";

#[must_use]
pub fn frame_proof(task: u64, window: bool, crossed: bool) -> jig_domain_core::Record {
    let jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(mut proof)) = root::run_proof(task) else {
        panic!("proof fixture")
    };
    proof.turn = Some(jig_domain_core::TurnProof { turn: 2, cumulative: 0, read: None });
    proof.window = window.then_some(jig_domain_core::CurrentWindow { attempt: 1, turn: 1, closed: crossed });
    proof.crossed = crossed;
    jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof))
}

/// Independent fixture encoding, shared with the frame-read test seam.
#[must_use]
pub fn fixture_turn(version: u16, said: &[u8], replay: &[u8], tokens: u32) -> Box<[u8]> {
    let mut out = version.to_be_bytes().to_vec();
    text(&mut out, said);
    if version == 1 {
        text(&mut out, replay);
    } else {
        out.extend(tokens.to_be_bytes());
    }
    out.into_boxed_slice()
}

#[must_use]
pub fn frame_committed(seed: u64, windows: &[bool], crossed: bool) -> World {
    let mut world = World::unopened_observed(seed, frame_limits(), None);
    world.owner_sets(FRAME_BASE);
    world.open_switched().expect("frame fixture open");
    let bytes = fixture_turn(1, FRAME_SAID, FRAME_REPLAY, 0);
    let mut number = 0;
    for (index, window) in windows.iter().enumerate() {
        let task = root::LIVE + u64::try_from(index).expect("task index");
        number += 1;
        world
            .commit(Box::new([Write::Save(root::header(number)), Write::Save(root::live_task(task))]))
            .expect("live task committed");
        for turn in 1..=2 {
            number += 1;
            world.commit(root::turn_commit(number, task, turn, &bytes)).expect("fixture turn committed");
        }
        // Keep the last turn proof with the designated window and crossing state.
        number += 1;
        world
            .commit(Box::new([Write::Save(root::header(number)), Write::Save(frame_proof(task, *window, crossed))]))
            .expect("proof committed");
    }
    world.crash(seed);
    world
}

/// Current real owner sets for the registry and opening-metadata suites. No
/// eager map patch is part of these binaries; frame fixtures use `FRAME_SETS`.
#[must_use]
pub const fn current(sets: OwnerSets) -> OwnerSets {
    sets
}
