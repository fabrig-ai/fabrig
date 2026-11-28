//! Seeded agreement of generic and generated decoders (skein/docs/design/data/codec/run-time.md, 2.3).
//! A round's case runs every family and root (skein/docs/design/testing/sweeps.md, 4.2).

extern crate alloc;

use crate::generic_support as generic;
use crate::support::generated;

#[cfg(test)]
mod tests {
    use super::{generated, generic};
    use skein_codec::schema::{self, DeclIx, Schema};
    use skein_codec::value::Value;
    use skein_lib::Rng;
    use skein_world::sweep::{Case, cases, gate};
    use skein_world::this_test;

    // Per root: 1536 draws and their three mutations; every arbitrary length
    // 0..=48 sixteen times; every fixed-rule value and its three mutations.
    const DRAWS: u64 = 1536;
    const ARBITRARY_ROUNDS: u32 = 16;
    const MAX_ARBITRARY: u32 = 48;

    struct Family {
        source: generated::Family,
        schema: Schema,
        values: &'static skein_codegen::Values,
        roots: Vec<(&'static generated::Root, DeclIx)>,
    }

    impl Family {
        fn new(source: generated::Family) -> Self {
            let schema = schema::parse(source.text.as_bytes()).expect("schema");
            let values = skein_codegen::values(source.text);
            let roots =
                source.roots.iter().map(|root| (root, schema.find(root.name.as_bytes()).expect("root"))).collect();
            Self { source, schema, values, roots }
        }

        fn oracle<'a>(
            &'a self,
            root: &'static generated::Root,
            decl: DeclIx,
            value: &'a mut Value,
        ) -> generic::Oracle<'a> {
            generic::Oracle { schema: &self.schema, decl, family: &self.source, root, value }
        }
    }

    fn seed(family: &str, root: &str, round: u64) -> u64 {
        family
            .bytes()
            .chain([0])
            .chain(root.bytes())
            .fold(round, |hash, byte| hash.wrapping_mul(0x100_0000_01b3) ^ u64::from(byte))
    }

    fn mutations(bytes: &[u8], rng: &mut Rng) -> [Vec<u8>; 3] {
        let length = skein_lib::len_u64(bytes.len());
        let mut changed = bytes.to_vec();
        if length > 0 {
            let at = usize::try_from(rng.below(length)).expect("wire index");
            // A nonzero XOR guarantees a changed byte.
            *changed.get_mut(at).expect("selected byte") ^= u8::try_from(rng.between(1, 255)).expect("byte");
        }
        let cut = usize::try_from(rng.between(0, length)).expect("cut index");
        let cut = bytes.get(..cut).expect("bounded cut").to_vec();
        let mut grown = bytes.to_vec();
        for _ in 0..rng.between(1, 4) {
            grown.push(u8::try_from(rng.below(256)).expect("byte"));
        }
        [changed, cut, grown]
    }

    #[test]
    fn generic_and_generated_decode_alike() -> Result<(), Case<String>> {
        let families = generated::families().map(Family::new);
        let mut value = Value::with_room(generic::ROOM);
        let mut comparisons = 0_u32;
        cases(this_test!("generic_and_generated_decode_alike"), 0..DRAWS, |round| {
            for family in &families {
                for &(root, decl) in &family.roots {
                    let mut oracle = family.oracle(root, decl, &mut value);
                    let seed = seed(family.source.name, root.name, round);
                    let mut rng = Rng::new(seed);
                    let bytes = generated::draw(&family.values.schema, root.name, &mut rng);
                    oracle.compare(seed, &bytes, true)?;
                    comparisons = comparisons.checked_add(1).expect("comparison count");
                    for mutation in mutations(&bytes, &mut rng) {
                        oracle.compare(seed, &mutation, false)?;
                        comparisons = comparisons.checked_add(1).expect("comparison count");
                    }
                }
            }
            Ok(())
        })?;
        if gate() {
            assert_eq!(comparisons, 423_936); // 69 roots * 1536 rounds * 4 comparisons
        }
        check_fixed(&families, &mut value);
        Ok(())
    }

    #[test]
    fn generic_and_generated_decode_arbitrary_bytes_alike() -> Result<(), Case<String>> {
        let families = generated::families().map(Family::new);
        let mut value = Value::with_room(generic::ROOM);
        let mut comparisons = 0_u32;
        let rounds = DRAWS..DRAWS + u64::from(ARBITRARY_ROUNDS);
        cases(this_test!("generic_and_generated_decode_arbitrary_bytes_alike"), rounds, |round| {
            for family in &families {
                for &(root, decl) in &family.roots {
                    let mut oracle = family.oracle(root, decl, &mut value);
                    let seed = seed(family.source.name, root.name, round);
                    let mut rng = Rng::new(seed);
                    for length in 0..=MAX_ARBITRARY {
                        let bytes =
                            (0..length).map(|_| u8::try_from(rng.below(256)).expect("byte")).collect::<Vec<_>>();
                        oracle.compare(seed, &bytes, false)?;
                        comparisons = comparisons.checked_add(1).expect("comparison count");
                    }
                }
            }
            Ok(())
        })?;
        if gate() {
            assert_eq!(comparisons, 54_096); // 69 roots * 16 rounds * 49 lengths
        }
        Ok(())
    }

    fn check_fixed(families: &[Family], value: &mut Value) {
        let mut comparisons = 0_u32;
        for family in families {
            for &(root, decl) in &family.roots {
                let mut oracle = family.oracle(root, decl, value);
                // Eight fixed seeds at every cut and each boundary byte.
                for seed in 0..8 {
                    let bytes = generated::draw(&family.values.schema, root.name, &mut Rng::new(seed));
                    for cut in 0..bytes.len() {
                        oracle.compare(seed, bytes.get(..cut).expect("prefix"), false).expect("decoder agreement");
                    }
                    for index in 0..bytes.len() {
                        let mut changed = bytes.clone();
                        for replacement in [0, 1, 2, 0x7f, 0x80, 0xff] {
                            *changed.get_mut(index).expect("chosen byte") = replacement;
                            oracle.compare(seed, &changed, false).expect("decoder agreement");
                        }
                    }
                }
                for fixed_value in &family.values.fixed {
                    if generic::named_root(&family.schema, &fixed_value.name) == decl {
                        let seed = seed(family.source.name, root.name, u64::MAX);
                        let mut rng = Rng::new(seed);
                        oracle.compare(seed, &fixed_value.bytes, true).expect("decoder agreement");
                        comparisons = comparisons.checked_add(1).expect("comparison count");
                        for mutation in mutations(&fixed_value.bytes, &mut rng) {
                            oracle.compare(seed, &mutation, false).expect("decoder agreement");
                            comparisons = comparisons.checked_add(1).expect("comparison count");
                        }
                    }
                }
            }
        }
        // The cut and boundary grid is separate from the fixed-rule value tally.
        assert_eq!(comparisons, 1_624); // 406 fixed-rule values * 4 comparisons
    }
}
