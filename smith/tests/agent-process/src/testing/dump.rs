//! Request capture fixtures for the simulated and real-loop suites.

use crate::{World, charter, wire};
use skein_fake_llm_domain::api;
use smith_channel::RunResult;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub const SEED: u64 = 71;
pub const BOUND: u64 = 1_048_576;
// One text response followed by finish; the budget admits both calls.
pub const TURNS: u32 = 2;
const SPEND: u64 = 20;

#[must_use]
pub fn policy() -> Box<[u8]> {
    wire::edit_charter(&charter(), |parts| wire::budget(parts, TURNS, SPEND))
}

#[must_use]
pub fn finish() -> api::Turn {
    smith_agent_world::fixture::calls(
        Box::new([api::Line::Call {
            name: b"finish".as_slice().into(),
            arguments: br#"{"form":"report","text":"done","fields":{}}"#.as_slice().into(),
        }]),
        1,
    )
}

#[must_use]
pub fn world(directory: PathBuf, bound: u64, anthropic: bool, two: bool) -> World {
    let mut world = World::new(SEED, &policy());
    world.private_grant();
    if anthropic {
        world.anthropic();
    }
    let turns = if two {
        vec![
            smith_agent_world::fixture::says(Box::new([api::Line::Text { text: b"continuing".as_slice().into() }]), 1),
            finish(),
        ]
    } else {
        vec![finish()]
    };
    world.script(turns.into());
    world.dump_to(directory, bound);
    world
}

#[must_use]
pub fn files(directory: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let mut paths: Vec<_> = fs::read_dir(directory)
        .expect("list dump directory")
        .map(|entry| entry.expect("dump directory entry").path())
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            let bytes = fs::read(&path).expect("read dump file");
            (path, bytes)
        })
        .collect()
}

pub fn accepted(world: &World) {
    assert!(
        matches!(&world.answer().expect("answer").result, RunResult::Accepted(_)),
        "{:?}: {}",
        world.answer(),
        String::from_utf8_lossy(&world.errors())
    );
    world.assert_agent_clean();
}

#[must_use]
pub fn dump_errors(world: &World) -> Vec<String> {
    String::from_utf8(world.errors())
        .expect("valid dump fixture")
        .lines()
        .filter(|line| line.starts_with("smith: request dump: "))
        .map(str::to_owned)
        .collect()
}
