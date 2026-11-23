//! The deployment's layout's golden bytes (protocol/store/README.md, 2, and protocol/store/testing.md, 1).
//! These tests only read the kept files. A deliberate layout change copies
//! the output named by a failure over the kept file in its own commit.

use jig_store_world::World;
use std::fmt::Write;

const FIRST_START: &[u8] = include_bytes!("../../golden/layout/first-start.txt");
const SEED: u64 = 0x4c41_594f_5554;

fn first_start() -> World {
    let mut world = World::unopened(SEED);
    world.open().expect("first start opens");
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
        "the deployment's layout changed (protocol/store/README.md, 2, and protocol/store/testing.md, 1): from the first release a later build reads this layout as it finds it; {}",
        differences.join("; ")
    );
}

#[test]
fn a_first_starts_layout_is_kept() {
    let made = listing(&mut first_start());
    assert_kept("first-start", &[("listing", FIRST_START, made.as_bytes())]);
}
