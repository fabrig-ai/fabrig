//! Canonical public targets over seeded bytes
//! (jig/docs/design/protocol/client/default-http.md, 5 and jig/docs/design/protocol/client/public-reads.md).
use jig_client_protocol_world::public_path::sweep;

const FIRST_SEED: u64 = 0x7075_7a74;
const SEEDS: u64 = 4;
const ARBITRARY: u32 = 2_000;
const CANONICAL: u32 = 512;

#[test]
fn the_public_path_parser_never_panics() {
    sweep(FIRST_SEED, SEEDS, ARBITRARY, CANONICAL, false);
}

#[test]
fn the_parser_accepts_exactly_the_canonical_paths() {
    sweep(FIRST_SEED, SEEDS, ARBITRARY, CANONICAL, true);
}
