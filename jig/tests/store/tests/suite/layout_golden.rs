//! The deployment's layout's golden bytes (protocol/store.md 2 and 14).
//! These tests only read the kept files. A deliberate layout change copies
//! the output named by a failure over the kept file in its own commit.

use jig_protocol_store::codec::component::{Levels, Manifest};
use jig_store_world::{World, patches};
use std::fmt::Write;

const FIRST_START: &[u8] = include_bytes!("../../golden/layout/first-start.txt");
const PATCHED_START: &[u8] = include_bytes!("../../golden/layout/patched-start.txt");
const ACTIVE: &[u8] = include_bytes!("../../golden/layout/active");
const MANIFEST: &[u8] = include_bytes!("../../golden/layout/manifest");
const SEED: u64 = 0x4c41_594f_5554;
const PATCHED_MANIFEST: &[u8] = b"generations/2/manifest";

fn first_start() -> World {
    let mut world = World::unopened(SEED);
    world.open().expect("first start opens");
    world
}

fn patched_start() -> World {
    // The same old records and patch sets as patching.rs, with no turns:
    // transcripts' contents have their own framing golden.
    let mut world = patches::Records::story().committed(SEED, patches::BASE);
    world.patch_sets(patches::SETS);
    // protocol/store.md, 10.2 steps 6 and 7: the manifest is written at the switch.
    // Step 8 keeps the opened copy: switching never reopens its stores.
    world.open_switched().expect("patched start opens and switches");
    world
}

fn listing(world: &mut World) -> String {
    let mut listing = String::new();
    for (path, directory, mode) in world.layout_entries() {
        let kind = if directory { 'd' } else { 'f' };
        let path = std::str::from_utf8(&path).expect("store layout names are UTF-8");
        writeln!(listing, "{kind} {mode:04o} {path}").expect("write listing");
    }
    listing
}

/// Save every changed part, then name its first differing byte. The kept
/// files are never written, including when several parts change together.
fn assert_kept(start: &str, parts: &[(&str, &[u8], &[u8])]) {
    let mut differences = Vec::new();
    for (part, kept, made) in parts {
        if kept == made {
            continue;
        }
        let path = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("layout-golden-{start}-{part}-made"));
        std::fs::create_dir_all(path.parent().expect("target tmp parent")).expect("create target tmp");
        std::fs::write(&path, made).expect("write layout part for inspection");
        let offset = kept.iter().zip(*made).position(|(kept, made)| kept != made).unwrap_or(kept.len().min(made.len()));
        differences.push(format!(
            "{start} {part}: first differing offset {offset}; kept length {}, writer length {}; writer's file: {}",
            kept.len(),
            made.len(),
            path.display()
        ));
    }
    assert!(
        differences.is_empty(),
        "the deployment's layout changed (protocol/store.md 2 and 14): from the first release a later build reads this layout as it finds it; {}",
        differences.join("; ")
    );
}

#[test]
fn a_first_starts_layout_is_kept() {
    let made = listing(&mut first_start());
    assert_kept("first-start", &[("listing", FIRST_START, made.as_bytes())]);
}

#[test]
fn a_patched_starts_layout_is_kept() {
    let mut world = patched_start();
    let made = listing(&mut world);
    let active = world.active();
    let contents = world.contents();
    let manifest = contents.get(PATCHED_MANIFEST).expect("patched manifest").as_ref().expect("manifest file");
    assert_kept(
        "patched-start",
        &[("listing", PATCHED_START, made.as_bytes()), ("active", ACTIVE, &active), ("manifest", MANIFEST, manifest)],
    );
}

#[test]
fn the_kept_manifest_decodes_to_the_patched_generation() {
    let manifest = Manifest::decode(MANIFEST).expect("kept manifest decodes with the component's codec");
    assert_eq!((manifest.generation, manifest.from), (2, Some(1)));
    assert_eq!(manifest.before, Levels { skein: None, smith: None, jig: Some(0), application: Some(0) });
    assert_eq!(manifest.after, Levels { skein: None, smith: None, jig: Some(2), application: Some(2) });
    let counts: Vec<_> = manifest.counts.iter().map(|count| (count.family(), count.before(), count.after())).collect();
    let labels = u64::from(patches::STORY_LABELS);
    let shelves = u64::from(patches::STORY_SHELVES);
    let notes = u64::from(patches::STORY_NOTES);
    let tokens = patches::STORY_TOKENS;
    assert_eq!(
        counts,
        [
            (b"fixture-label".as_slice(), labels, labels),
            (b"fixture-shelf".as_slice(), shelves, shelves),
            (b"fixture-token".as_slice(), tokens, tokens),
            (b"fixture-note".as_slice(), notes, notes),
        ]
    );
}
