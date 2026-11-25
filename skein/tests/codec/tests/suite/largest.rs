//! Largest schema values through the lean byte pin (data/codec/golden.md, 1).

#[cfg(test)]
mod tests {
    use skein_codec_tests::every_v3::{self, CEILINGS, Limits, Record};
    use skein_codegen::{Schema, largest, roots};
    use skein_lib::Reader;

    const MAX_BYTES: u64 = 16 * 1024 * 1024;

    fn small_limits() -> Limits {
        CEILINGS.at_most(3)
    }

    fn round_trip(schema: &Schema, root: &str, limits: &Limits, max_bytes: u64) -> bool {
        macro_rules! decode {
            ($($name:ident),+ $(,)?) => {
                match root {
                    $(stringify!($name) => {
                        let bound = every_v3::$name::encoded_bound(limits).expect("valid limits");
                        if bound > max_bytes {
                            return false;
                        }
                        let bytes = largest(schema, root, limits.values().as_slice()).expect("largest root");
                        assert_eq!(u64::try_from(bytes.len()).expect("length fits"), bound, "{root}: largest bound");
                        let mut reader = Reader::new(&bytes);
                        let value = every_v3::$name::decode(limits, &mut reader).expect("lean decoder");
                        assert!(reader.is_empty(), "{root}: whole encoding consumed");
                        let measured = usize::try_from(value.measure(limits).expect("decoded value measures")).expect("measured length fits");
                        assert_eq!(measured, bytes.len(), "{root}: measure");
                        assert_eq!(value.encode(limits).expect("decoded value encodes").as_ref(), bytes, "{root}: round trip");
                        true
                    }),+
                    _ => panic!("no lean decoder for root {root}"),
                }
            };
        }
        decode!(Every, Kinds, Lists, Options, Grouped, Held, LongRecord)
    }

    fn schema() -> Schema {
        skein_codegen::values(skein_codec_tests::EVERY_SCHEMA).schema.clone()
    }

    #[test]
    fn every_root_largest_round_trips_at_small_limits() {
        let limits = small_limits();
        assert_eq!(
            limits.values().as_slice(),
            CEILINGS.values().as_slice().iter().map(|ceiling| (*ceiling).min(3)).collect::<Vec<_>>()
        );
        let schema = schema();
        let roots = roots(&schema);
        assert_ne!(roots, Vec::<String>::new());
        for root in roots {
            assert!(round_trip(&schema, &root, &limits, u64::MAX), "{root}: no small root skipped");
        }
    }

    #[test]
    fn every_root_largest_within_sixteen_mib_round_trips_at_ceilings() {
        let schema = schema();
        let mut tested = 0_usize;
        for root in roots(&schema) {
            if round_trip(&schema, &root, &CEILINGS, MAX_BYTES) {
                tested = tested.checked_add(1).expect("root count fits");
            }
        }
        assert_ne!(tested, 0, "at least one root fits the ceiling test budget");
    }
}
