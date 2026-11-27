use hearth_render_world::{fixtures, schemas};
use skein_world::{
    sweep::{Case, cases, gate},
    this_test,
};
use std::{collections::BTreeSet, convert::Infallible};

pub(super) const SEED: u64 = 0x0013_F000;
const ROUNDS: u64 = 10_000;

#[test]
fn generated_schemas_and_settings_validate_as_the_reference_does() -> Result<(), Case<Infallible>> {
    let mut fell = BTreeSet::new();
    let gate = gate();
    cases(this_test!("generated_schemas_and_settings_validate_as_the_reference_does"), 0..ROUNDS, |seed| {
        let mut rng = super::reference_rng(SEED, seed);
        let declaration = schemas::declaration(&mut rng, schemas::WIDE);
        let settings = schemas::settings(&mut rng, &declaration);
        let value = schemas::validate(&declaration, &settings);
        assert_eq!(value, schemas::reference(&declaration, &settings), "seed {seed}");
        if gate && let Err(faults) = value {
            for fault in &faults {
                fell.insert(schemas::kind(&fault.schema));
            }
        }
        Ok(())
    })?;
    if gate {
        fixtures::assert_fell(&fell, schemas::KINDS);
    }
    Ok(())
}
