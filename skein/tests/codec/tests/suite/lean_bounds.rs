//! Lean every-v3's lowered-limit bounds (data/codec/README.md, 4 and 6).
//! Decoded bound literals hold the 64-bit target's type sizes.

#[cfg(test)]
mod tests {
    use skein_codec::Reason;
    use skein_codec_tests::every_v3_lean::{self as lean, CEILINGS, Limits, Problem, Record, paths};
    use skein_lib::Reader;

    #[expect(clippy::too_many_lines, reason = "each lean Limits field is clamped from its ceiling")]
    fn small_limits() -> Limits {
        macro_rules! clamp {
            ($($field:ident),+ $(,)?) => {
                Limits { $($field: CEILINGS.$field.min(3)),+ }
            };
        }
        clamp!(
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

    macro_rules! pin {
        ($limits:expr; $($name:ident => ($encoded:expr, $decoded:expr)),+ $(,)?) => {
            $(assert_eq!(
                (lean::$name::encoded_bound($limits), lean::$name::decoded_bound($limits)),
                ($encoded, $decoded), stringify!($name),
            );)+
        };
    }

    #[test]
    fn every_lean_bounds_at_small_limits_are_pinned() {
        let limits = small_limits();
        pin!(&limits;
            Texts => (Some(21), Some(9)),
            Stamp => (Some(25), Some(6)),
            Shape => (Some(28), Some(57)),
            Nest => (Some(229), Some(431)),
            Every => (Some(354), Some(509)),
            Lists => (Some(224), Some(306)),
            Options => (Some(98), Some(11)),
            Held => (Some(42), Some(18)),
            LongRecord => (Some(700), Some(300)),
        );
        assert_eq!((lean::encoded_bound(&limits), lean::decoded_bound(&limits)), (Some(700), Some(509)));
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
