//! The lean form keeps the sealed pin's bytes, bounds and refusal names.

#[cfg(test)]
mod tests {
    use crate::{
        generic_support as generic,
        support::generated::{self, Decoded, Family, Root},
    };
    use lean::Record;
    use skein_codec_tests::{every_v3 as sealed, every_v3_lean as lean};

    fn families() -> (Family, Family) {
        let sealed = generated::families()
            .into_iter()
            .find(|family| family.text == include_str!("../../schema/every-v3.schema"))
            .expect("sealed every-v3");
        (sealed, generated::every_lean())
    }

    fn goldens() -> Vec<(String, Vec<u8>)> {
        let mut values = std::fs::read_dir("golden/every-v3")
            .expect("goldens")
            .map(|entry| {
                let path = entry.expect("golden entry").path();
                (
                    path.file_name().expect("name").to_str().expect("UTF-8").to_owned(),
                    std::fs::read(&path).expect("golden"),
                )
            })
            .collect::<Vec<_>>();
        values.sort();
        assert_eq!(values.len(), 40, "all committed every-v3 goldens");
        values
    }

    fn roots<'a>(
        schema: &skein_codec::schema::Schema,
        name: &str,
        sealed: &'a Family,
        lean: &'a Family,
    ) -> (&'a Root, &'a Root) {
        let decl = generic::golden_root(schema, name);
        let name = schema.spelled(schema.declaration(decl).name);
        (
            sealed.roots.iter().find(|root| root.name.as_bytes() == name).expect("sealed root"),
            lean.roots.iter().find(|root| root.name.as_bytes() == name).expect("lean root"),
        )
    }

    #[test]
    fn every_lean_golden_round_trips_at_ceilings() {
        let (sealed, lean) = families();
        let schema = skein_codec::schema::parse(lean.text.as_bytes()).expect("every schema");
        let mut generic = skein_codec::value::Value::with_room(generic::ROOM);
        for (name, bytes) in goldens() {
            let (sealed_root, lean_root) = roots(&schema, &name, &sealed, &lean);
            let decoded = (lean_root.decode)(&bytes);
            assert_eq!(decoded, (sealed_root.decode)(&bytes), "{name}");
            let Decoded::Value { bytes: encoded, measured, .. } = decoded else {
                unreachable!("a golden decodes");
            };
            assert_eq!(encoded, bytes, "{name}: canonical encoding");
            assert_eq!(usize::try_from(measured).expect("length"), bytes.len(), "{name}: measure");
            generic
                .decode(&schema, generic::golden_root(&schema, &name), bytes.clone().into_boxed_slice())
                .expect("generic decode");
            assert_eq!(generic.bytes(), bytes, "{name}: generic bytes");
        }
    }

    fn decode_pair(
        root: &str,
        bytes: &[u8],
        sealed_limits: &sealed::Limits,
        lean_limits: &lean::Limits,
    ) -> (Decoded, Decoded) {
        macro_rules! decode {
            ($($record:ident),+ $(,)?) => {
                match root {
                    $(stringify!($record) => {
                        let sealed = match sealed::$record::decode(sealed_limits, &mut skein_lib::Reader::new(bytes)) {
                            Ok(value) => {
                                let measured = value.measure();
                                let mut writer = skein_lib::Writer::new(usize::try_from(measured).expect("length"));
                                value.encode(&mut writer).expect("decoded value encodes");
                                Decoded::Value { debug: format!("{value:?}"), bytes: writer.finish().into_vec(), measured }
                            }
                            Err(problem) => Decoded::Problem {
                                path: problem.path.name().into(), reason: problem.reason,
                            },
                        };
                        let lean = match lean::$record::decode(lean_limits, &mut skein_lib::Reader::new(bytes)) {
                            Ok(value) => Decoded::Value {
                                debug: format!("{value:?}"),
                                bytes: value.encode(lean_limits).expect("decoded value encodes").into_vec(),
                                measured: value.measure(lean_limits).expect("decoded value measures"),
                            },
                            Err(problem) => Decoded::Problem {
                                path: lean::name(problem.path).expect("generated path").into(), reason: problem.reason,
                            },
                        };
                        (sealed, lean)
                    }),+
                    _ => panic!("unknown every root: {root}"),
                }
            };
        }
        decode!(
            Blank, Numbers, Texts, Light, Stamp, Shape, Nest, Every, Kinds, Lists, Options, Grouped, Held, LongRecord
        )
    }

    #[test]
    fn every_lean_and_sealed_refuse_the_same_mutations() {
        let (sealed, lean) = families();
        let schema = skein_codec::schema::parse(lean.text.as_bytes()).expect("every schema");
        let (small_sealed, small_lean) = small();
        let goldens = goldens();
        for (limits, sealed_limits, lean_limits) in
            [("ceilings", &sealed::CEILINGS, &lean::CEILINGS), ("small", &small_sealed, &small_lean)]
        {
            let mut cases = 0_u32;
            for (name, bytes) in &goldens {
                let (sealed_root, _) = roots(&schema, name, &sealed, &lean);
                let mut compare = |bytes: &[u8]| {
                    let (sealed, lean) = decode_pair(sealed_root.name, bytes, sealed_limits, lean_limits);
                    assert_eq!(lean, sealed, "{limits}: {name}: {bytes:02x?}");
                    if let Decoded::Value { bytes: encoded, measured, .. } = lean {
                        assert_eq!(encoded, bytes, "{limits}: {name}: accepted mutation is canonical");
                        assert_eq!(usize::try_from(measured).expect("length"), bytes.len());
                    }
                    cases = cases.checked_add(1).expect("case count");
                };
                for cut in 0..bytes.len() {
                    compare(bytes.get(..cut).expect("prefix"));
                }
                let mut changed = bytes.clone();
                changed.push(0);
                compare(&changed);
                for at in 0..bytes.len() {
                    changed.clone_from(bytes);
                    for replacement in [0, 1, 2, 0x7f, 0x80, 0xff] {
                        *changed.get_mut(at).expect("byte") = replacement;
                        compare(&changed);
                    }
                }
            }
            assert!(cases > 10_000, "{limits}: every cut and byte replacement was exercised");
        }
    }

    macro_rules! small_limits {
    ($($field:ident),+ $(,)?) => {
        (sealed::Limits { $($field: sealed::CEILINGS.$field.min(3)),+ },
         lean::Limits { $($field: lean::CEILINGS.$field.min(3)),+ })
    };
}

    macro_rules! bounds {
    ($sealed:expr, $lean:expr; $($name:ident),+ $(,)?) => {
        $(assert_eq!(sealed::$name::encoded_bound($sealed), lean::$name::encoded_bound($lean), concat!(stringify!($name), " wire"));
          assert_eq!(sealed::$name::decoded_bound($sealed), lean::$name::decoded_bound($lean), concat!(stringify!($name), " heap"));)+
    };
}

    #[test]
    fn every_lean_bounds_equal_sealed_at_ceilings_and_small_limits() {
        let (small_sealed, small_lean) = small();
        for (sealed, lean) in [(&sealed::CEILINGS, &lean::CEILINGS), (&small_sealed, &small_lean)] {
            bounds!(sealed, lean; Blank, Numbers, Texts, Light, Stamp, Shape, Nest, Every, Kinds, Lists, Options, Grouped, Held, LongRecord);
            assert_eq!(sealed::encoded_bound(sealed), lean::encoded_bound(lean));
            assert_eq!(sealed::decoded_bound(sealed), lean::decoded_bound(lean));
        }
    }

    #[expect(
        clippy::too_many_lines,
        reason = "one entry per named limit builds both concrete families at min(ceiling, 3)"
    )]
    fn small() -> (sealed::Limits, lean::Limits) {
        small_limits!(
            texts_raw,
            texts_key,
            texts_words,
            stamp_note_some,
            stamp_zone,
            nest_blanks,
            nest_lights,
            nest_maybe_lights,
            nest_grid,
            nest_grid_item,
            nest_maybe_list_some,
            nest_shapes,
            nest_secrets,
            nest_secrets_item,
            nest_stamps,
            nest_label_some,
            every_held,
            every_items,
            lists_small,
            lists_short,
            lists_word,
            lists_long,
            lists_flag,
            lists_pause,
            lists_tag,
            lists_raw,
            lists_raw_item,
            lists_key,
            lists_key_item,
            lists_words,
            lists_words_item,
            lists_grid,
            lists_grid_item,
            lists_options,
            lists_records,
            lists_enums,
            options_raw_some,
            options_key_some,
            options_words_some,
            options_lists_some,
            held_record_bytes,
            held_enum_bytes,
            held_record_secret,
            held_enum_secret,
            held_byte_pieces,
            held_secret_pieces,
            long_record_alpha,
            long_record_bravo,
            long_record_charlie,
            long_record_delta,
            long_record_echo,
            long_record_foxtrot,
            long_record_golf,
            long_record_hotel,
            long_record_india,
            long_record_juliet,
            long_record_kilo,
            long_record_lima,
            long_record_mike,
            long_record_november,
            long_record_oscar,
            long_record_papa,
            long_record_quebec,
            long_record_romeo,
            long_record_sierra,
            long_record_tango,
            long_record_uniform,
            long_record_victor,
            long_record_whiskey,
            long_record_xray,
            long_record_yankee,
            long_record_zulu,
            long_record_amber,
            long_record_birch,
            long_record_cedar,
            long_record_dune,
            long_record_acorn,
            long_record_anchor,
            long_record_apron,
            long_record_atlas,
            long_record_autumn,
            long_record_avocado,
            long_record_walnut,
            long_record_badge,
            long_record_bamboo,
            long_record_barrel,
            long_record_beacon,
            long_record_beetle,
            long_record_bronze,
            long_record_cactus,
            long_record_camera,
            long_record_canyon,
            long_record_cello,
            long_record_cobalt,
            long_record_comet,
            long_record_compass,
            long_record_coral,
            long_record_cotton,
            long_record_coyote,
            long_record_cricket,
            long_record_crystal,
            long_record_dragon,
            long_record_engine,
            long_record_falcon,
            long_record_feather,
            long_record_fern,
            long_record_flute,
            long_record_fossil,
            long_record_galaxy,
            long_record_garden,
            long_record_glacier,
            long_record_granite,
            long_record_harbor,
            long_record_hazel,
            long_record_helmet,
            long_record_honey,
            long_record_igloo,
            long_record_island,
            long_record_ivory,
            long_record_jasmine,
            long_record_kettle,
            long_record_lantern,
            long_record_leopard,
            long_record_lobster,
            long_record_magnet,
            long_record_marble,
            long_record_meadow,
            long_record_mermaid,
            long_record_meteor,
            long_record_mosaic,
            long_record_napkin,
            long_record_nectar,
            long_record_ocean,
            long_record_olive,
            long_record_orchid,
            long_record_panda,
            long_record_pencil,
            long_record_planet,
            long_record_pollen,
            long_record_quartz,
            long_record_rainbow,
            long_record_raven,
            long_record_rocket,
            long_record_saffron,
            long_record_salmon,
            long_record_satin,
        )
    }
}
