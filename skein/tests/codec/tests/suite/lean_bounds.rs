//! Lean every-v3 at lowered limits (data/codec/README.md, 4, and data/codec/golden.md, 1).

#[cfg(test)]
mod tests {
    use crate::{generic_support as generic, support::generated::Decoded};
    use skein_codec::{
        Reason,
        value::{Refusal, Value},
    };
    use skein_codec_tests::every_v3::{self as lean, CEILINGS, Limits, Problem, Record, paths};
    use skein_lib::Reader;

    #[test]
    fn at_most_clamps_each_limit_without_raising_any() {
        use skein_codec_tests::v1::CEILINGS;

        // sample-v1 has ceilings 4, 4, 8, 2 in limit order.
        let ceilings = CEILINGS.values();
        let cases: [(u32, &[u32]); 3] = [(0, &[0, 0, 0, 0]), (3, &[3, 3, 3, 2]), (u32::MAX, ceilings.as_slice())];
        for (most, expected) in cases {
            assert_eq!(CEILINGS.at_most(most).values().as_slice(), expected);
        }
    }

    fn small_limits() -> Limits {
        CEILINGS.at_most(3)
    }

    fn decode(root: &str, bytes: &[u8], limits: &Limits) -> Decoded {
        macro_rules! decode {
            ($($record:ident),+ $(,)?) => {
                match root {
                    $(stringify!($record) => match lean::$record::decode(limits, &mut Reader::new(bytes)) {
                        Ok(value) => Decoded::Value {
                            debug: format!("{value:?}"),
                            bytes: value.encode(limits).expect("decoded value encodes").into_vec(),
                            measured: value.measure(limits).expect("decoded value measures"),
                        },
                        Err(problem) => Decoded::Problem {
                            path: problem.path.name().into(), reason: problem.reason,
                        },
                    }),+
                    _ => panic!("unknown every root: {root}"),
                }
            };
        }
        decode!(
            Blank, Numbers, Texts, Light, Stamp, Shape, Nest, Every, Kinds, Lists, Options, Grouped, Held, LongRecord
        )
    }

    // The generic decoder reads ceilings from its schema: clamp only bound tokens,
    // leaving fixed widths, versions and defaults alone.
    fn small_schema() -> skein_codec::schema::Schema {
        let mut bound = false;
        let text = skein_codec_tests::EVERY_SCHEMA
            .lines()
            .flat_map(|line| line.split('#').next().expect("line before comment").split_whitespace())
            .map(|word| {
                let token = if bound { word.parse::<u32>().expect("bound").min(3).to_string() } else { word.into() };
                bound = matches!(word, "bytes" | "secret" | "text" | "list");
                token
            })
            .collect::<Vec<_>>()
            .join(" ");
        skein_codec::schema::parse(text.as_bytes()).expect("clamped every schema")
    }

    #[test]
    fn every_mutation_at_small_limits_matches_the_generic_decoder() {
        let limits = small_limits();
        let schema = small_schema();
        let mut generic = Value::with_room(generic::ROOM);
        let mut paths = std::fs::read_dir("golden/every-v3")
            .expect("goldens")
            .map(|entry| entry.expect("golden entry").path())
            .collect::<Vec<_>>();
        paths.sort();
        assert_eq!(paths.len(), 40, "all committed every-v3 goldens");
        let mut cases = 0_u32;
        for path in paths {
            let name = path.file_name().expect("name").to_str().expect("UTF-8");
            let decl = generic::named_root(&schema, name);
            let root = core::str::from_utf8(schema.spelled(schema.declaration(decl).name)).expect("root name");
            let bytes = std::fs::read(&path).expect("golden");
            let mut compare = |bytes: &[u8]| {
                let generated = decode(root, bytes, &limits);
                let expected = match generic.decode(&schema, decl, bytes.into()) {
                    Ok(node) => Decoded::Value {
                        debug: generic::render(&schema, &generic, node, decl),
                        bytes: generic.bytes().to_vec(),
                        measured: u32::try_from(bytes.len()).expect("length"),
                    },
                    Err(Refusal::Problem(problem)) => {
                        Decoded::Problem { path: generic::path_name(&schema, problem.path), reason: problem.reason }
                    }
                    Err(Refusal::Room) => unreachable!("every value fits the room"),
                };
                assert_eq!(generated, expected, "small: {name}: {bytes:02x?}");
                cases = cases.checked_add(1).expect("case count");
            };
            for cut in 0..bytes.len() {
                compare(bytes.get(..cut).expect("prefix"));
            }
            let mut changed = bytes.clone();
            changed.push(0);
            compare(&changed);
            for at in 0..bytes.len() {
                changed.clone_from(&bytes);
                for replacement in [0, 1, 2, 4, 0x7f, 0x80, 0xff] {
                    *changed.get_mut(at).expect("byte") = replacement;
                    compare(&changed);
                }
            }
        }
        assert!(cases > 10_000, "every cut and byte replacement was exercised");
    }

    // One nonempty field; all other fields are empty strings/lists or absent options.
    fn fields(empty: &[u8], count: usize, target: usize, field: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        for index in 0..count {
            bytes.extend_from_slice(if index == target { field } else { empty });
        }
        bytes
    }

    fn payload(length: u32) -> Vec<u8> {
        let mut bytes = length.to_be_bytes().to_vec();
        bytes.extend(core::iter::repeat_n(b'a', usize::try_from(length).expect("small length")));
        bytes
    }

    fn texts(target: usize, length: u32) -> Vec<u8> {
        fields(&[0; 4], 3, target, &payload(length))
    }

    fn lists(target: usize, field: &[u8]) -> Vec<u8> {
        fields(&[0; 4], 14, target, field)
    }

    fn list_item(target: usize, length: u32) -> Vec<u8> {
        let mut field = 1_u32.to_be_bytes().to_vec();
        field.extend(payload(length));
        lists(target, &field)
    }

    fn list_count(length: u32) -> Vec<u8> {
        let mut field = length.to_be_bytes().to_vec();
        for _ in 0..length {
            field.extend_from_slice(&[0; 4]); // An empty bytes item.
        }
        lists(7, &field)
    }

    fn options(target: usize, length: u32) -> Vec<u8> {
        let mut field = vec![1];
        field.extend(payload(length));
        fields(&[0], 14, target, &field)
    }

    macro_rules! boundary {
        ($record:ident, $limit:ident, $path:ident, $length:expr, $bytes:expr) => {{
            let length: u32 = $length;
            let limits = Limits { $limit: length, ..CEILINGS };
            let bytes = ($bytes)(length);
            let value = lean::$record::decode(&limits, &mut Reader::new(&bytes))
                .expect(concat!(stringify!($limit), " at limit"));
            assert_eq!(
                usize::try_from(value.measure(&limits).expect("decoded value measures")).expect("measured length"),
                bytes.len(),
                stringify!($limit)
            );
            assert_eq!(value.encode(&limits).expect("decoded value encodes").as_ref(), bytes, stringify!($limit));

            let bytes = ($bytes)(length.checked_add(1).expect("one past small limit"));
            let problem = lean::$record::decode(&limits, &mut Reader::new(&bytes))
                .expect_err(concat!(stringify!($limit), " one past limit"));
            assert_eq!(problem, Problem { path: paths::$path, reason: Reason::Bound }, stringify!($limit));
        }};
    }

    // Bytes, secrets and text use L = 3. Lists use L = 1, below their ceiling of 2,
    // so one past L still fits the ceiling and tests the adjustable limit.
    #[test]
    fn every_lean_bounded_record_fields_accept_limit_and_refuse_one_past() {
        boundary!(Texts, texts_raw, TEXTS_RAW, 3, |n| texts(0, n));
        boundary!(Texts, texts_key, TEXTS_KEY, 3, |n| texts(1, n));
        boundary!(Texts, texts_words, TEXTS_WORDS, 3, |n| texts(2, n));
        boundary!(Lists, lists_raw, LISTS_RAW, 1, list_count);
    }

    #[test]
    fn every_lean_bounded_list_items_accept_limit_and_refuse_one_past() {
        boundary!(Lists, lists_raw_item, LISTS_RAW, 3, |n| list_item(7, n));
        boundary!(Lists, lists_key_item, LISTS_KEY, 3, |n| list_item(8, n));
        boundary!(Lists, lists_words_item, LISTS_WORDS, 3, |n| list_item(9, n));
        boundary!(Lists, lists_grid_item, LISTS_GRID, 1, |n| list_item(10, n));
    }

    #[test]
    fn every_lean_bounded_option_values_accept_limit_and_refuse_one_past() {
        boundary!(Options, options_raw_some, OPTIONS_RAW, 3, |n| options(7, n));
        boundary!(Options, options_key_some, OPTIONS_KEY, 3, |n| options(8, n));
        boundary!(Options, options_words_some, OPTIONS_WORDS, 3, |n| options(9, n));
        boundary!(Options, options_lists_some, OPTIONS_LISTS, 1, |n| options(10, n));
    }
}
