use skein_codec::{patch::Halt, schema, value::Value};
use skein_lib::Rng;
use smith_transcript_patches::patches as fixture;

use crate::support::patches;

/// Four mutation modes over both fixed-rule values, with many draws of each within the 0.2 s share.
pub(super) const ROUNDS: u32 = 512;
/// The version prefix selects the host's schema; body mutations retain the kept schema version.
const VERSION_BYTES: usize = 2;
/// Unchanged, byte changes, truncation and extension are the four draw modes.
const MUTATION_MODES: u64 = 4;
/// One to four byte changes can affect lengths and payload independently.
const MAX_CHANGES: u64 = 4;
/// Up to eight trailing bytes exercise refusal of extensions.
const MAX_EXTENSION: u64 = 8;
fn draw_index(rng: &mut Rng, bound: usize) -> usize {
    usize::try_from(rng.below(u64::try_from(bound).expect("small draw bound"))).expect("draw index fits")
}

fn draw_record(rng: &mut Rng, fixed_value: &[u8]) -> Box<[u8]> {
    let mut record = fixed_value.to_vec();
    match rng.below(MUTATION_MODES) {
        0 => {}
        1 => {
            let changes = rng.below(MAX_CHANGES) + 1;
            for _ in 0..changes {
                let index = VERSION_BYTES + draw_index(rng, record.len() - VERSION_BYTES);
                record[index] = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
            }
        }
        2 => {
            let length = VERSION_BYTES + draw_index(rng, record.len() - VERSION_BYTES);
            record.truncate(length);
        }
        3 => {
            let extension = rng.below(MAX_EXTENSION) + 1;
            for _ in 0..extension {
                record.push(u8::try_from(rng.next_u64() & 0xff).expect("low byte"));
            }
        }
        _ => unreachable!("four mutation modes"),
    }
    record.into_boxed_slice()
}

pub(super) fn run(seed: u64) {
    let old_schema = schema::parse(fixture::kept::fixture_turn::TEXTS[0].as_bytes()).expect("v1 schema");
    let new_schema = schema::parse(fixture::kept::fixture_turn::TEXTS[1].as_bytes()).expect("v2 schema");
    let old_root = old_schema.find(smith_transcript::ROOT.as_bytes()).expect("old Turn");
    let new_root = new_schema.find(smith_transcript::ROOT.as_bytes()).expect("new Turn");
    let fixed_values =
        skein_codegen::fixed_values(&skein_codegen::parse(fixture::kept::fixture_turn::TEXTS[0]).expect("v1 schema"));
    let mut old_value = Value::with_room(patches::NODES);
    let mut new_value = Value::with_room(patches::NODES);
    let mut patcher = patches::fixture_patcher();
    let mut rng = Rng::new(seed);
    let mut accepted = 0;
    let mut refused = 0;
    for round in 0..ROUNDS {
        let fixed_value = &fixed_values[draw_index(&mut rng, fixed_values.len())];
        let record = draw_record(&mut rng, &fixed_value.bytes);
        let decoded = old_value.decode(&old_schema, old_root, record.clone());
        let upgraded = patches::patch_to_current(&mut patcher, record);
        match decoded {
            Err(refusal) => {
                assert_eq!(upgraded, Err(Halt::Decode(refusal)), "seed {seed}, round {round}");
                refused += 1;
            }
            Ok(old_root) => {
                assert!(upgraded.is_ok(), "seed {seed}, round {round}: {upgraded:?}");
                let bytes = upgraded.expect("accepted turn patches");
                let decoded = new_value.decode(&new_schema, new_root, bytes);
                assert!(decoded.is_ok(), "seed {seed}, round {round}: {decoded:?}");
                let new_root = decoded.expect("patched turn decodes");
                let old_said =
                    old_value.field(old_root, fixture::kept::fixture_turn::v1::turn::SAID).expect("old said");
                let new_said =
                    new_value.field(new_root, fixture::kept::fixture_turn::v2::turn::SAID).expect("new said");
                assert_eq!(old_value.content(old_said), new_value.content(new_said), "seed {seed}, round {round}");
                let tokens = new_value.field(new_root, fixture::kept::fixture_turn::v2::turn::TOKENS).expect("tokens");
                assert_eq!(new_value.uint(tokens), Ok(0), "seed {seed}, round {round}");
                accepted += 1;
            }
        }
    }
    assert!(accepted > 0 && refused > 0, "seed {seed}, {ROUNDS} rounds must exercise both outcomes");
    skein_checks::count("seed", 1);
}
