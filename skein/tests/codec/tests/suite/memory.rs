//! A decoded value at every ceiling stays within its declared heap bound
//! (data/codec/README.md, section 4, and data/codec/golden.md, section 1; testing-strategy.md, section 6).

#[cfg(test)]
mod tests {
    use skein_codec_tests::v1::{CEILINGS, Effect, Field, Record, Report};
    use skein_heap::Meter;
    use skein_lib::{Duration, List, Reader};

    fn full_field() -> Field {
        Field { name: Box::from(&b"abcd"[..]), value: Box::from(&[1, 2, 3, 4][..]) }
    }

    fn full_report() -> Report {
        let mut fields = List::with_capacity(CEILINGS.report_fields);
        fields.push(full_field()).expect("first field room");
        fields.push(full_field()).expect("second field room");
        Report {
            title: Box::from(&b"abcdefgh"[..]),
            fields,
            effect: Effect::Write,
            present: Some(true),
            nonce: [0xff, 0xff],
            age: Duration::from_nanos(u64::MAX),
        }
    }

    #[test]
    fn full_record_decoder_at_its_limits_stays_within_decoded_bound() {
        let value = full_report();
        assert_eq!(
            value.measure(&CEILINGS).expect("report at ceilings"),
            u32::try_from(Report::encoded_bound(&CEILINGS).expect("wire bound")).expect("u32 ceiling")
        );
        let wire = value.encode(&CEILINGS).expect("report at ceilings");
        drop(value);

        let bound = Report::decoded_bound(&CEILINGS).expect("heap bound");
        let meter = Meter::new();
        meter.start();
        let decoded = Report::decode(&CEILINGS, &mut Reader::new(&wire)).expect("full record");
        let measured = meter.end();
        assert!(measured.peak() <= bound, "the full value stays within the heap bound");
        assert!(measured.held() <= bound, "the decoded value stays within the heap bound");
        assert_eq!(decoded.title.len(), usize::try_from(CEILINGS.report_title).expect("ceiling"));
        assert_eq!(decoded.fields.len(), CEILINGS.report_fields);
        for field in decoded.fields.as_slice() {
            assert_eq!(field.name.len(), usize::try_from(CEILINGS.field_name).expect("ceiling"));
            assert_eq!(field.value.len(), usize::try_from(CEILINGS.field_value).expect("ceiling"));
        }
        drop(decoded);
        assert_eq!(meter.held(), 0, "decoded allocations released");
    }

    fn decode_growth<T>(limits: &[u32], bound: u64, decode: impl FnOnce(&mut Reader<'_>) -> T) -> T {
        let schema = skein_codegen::values(skein_codec_tests::GROWTH_SCHEMA).schema.clone();
        assert_eq!(skein_codegen::roots(&schema), ["Growth"]);
        let wire = skein_codegen::largest(&schema, "Growth", limits).expect("largest growth root");
        let meter = Meter::new();
        meter.start();
        let decoded = decode(&mut Reader::new(&wire));
        let measured = meter.end();
        assert!(measured.peak() <= bound, "growth decode: peak {} exceeds bound {bound}", measured.peak());
        assert!(measured.held() <= bound, "growth value stays within its heap bound");
        decoded
    }

    #[test]
    fn lean_list_past_the_growth_floor_decodes_within_its_bound() {
        use skein_codec_tests::growth::{CEILINGS, Growth, Record};
        let bound = Growth::decoded_bound(&CEILINGS).expect("heap bound");
        let decoded = decode_growth(CEILINGS.values().as_slice(), bound, |reader| {
            Growth::decode(&CEILINGS, reader).expect("lean growth root")
        });
        assert_eq!(decoded.items.len(), CEILINGS.growth_items);
        assert!(decoded.items.iter().all(|item| *item == u64::MAX));
    }

    fn full_numbers() -> skein_codec_tests::every::Numbers {
        use skein_codec_tests::every::Numbers;
        Numbers {
            small: u8::MAX,
            short: u16::MAX,
            word: u32::MAX,
            long: u64::MAX,
            flag: true,
            wait: Duration::from_nanos(u64::MAX),
            tag: [0xff; 3],
        }
    }

    fn full_stamp() -> skein_codec_tests::every::Stamp {
        use skein_codec_tests::every::{CEILINGS, Stamp};
        Stamp {
            at: u64::MAX,
            note: Some(vec![b'a'; usize::try_from(CEILINGS.stamp_note_some).expect("bound")].into_boxed_slice()),
        }
    }

    fn full_strings() -> skein_codec_tests::every::Strings {
        use skein_codec_tests::every::{CEILINGS, Strings};
        let bytes = |bound| vec![b'a'; usize::try_from(bound).expect("bound")].into_boxed_slice();
        Strings {
            raw: bytes(CEILINGS.strings_raw),
            key: skein_lib::Secret::new(bytes(CEILINGS.strings_key)),
            words: bytes(CEILINGS.strings_words),
        }
    }

    fn full_every() -> skein_codec_tests::every::Every {
        use skein_codec_tests::every::{Blank, CEILINGS, Every, Light, Nest, Shape};
        let bytes = |bound| vec![b'a'; usize::try_from(bound).expect("bound")].into_boxed_slice();
        let mut blanks = List::with_capacity(CEILINGS.nest_blanks);
        for _ in 0..CEILINGS.nest_blanks {
            blanks.push(Blank {}).expect("room");
        }
        let mut lights = List::with_capacity(CEILINGS.nest_lights);
        for _ in 0..CEILINGS.nest_lights {
            lights.push(Light::Green).expect("room");
        }
        let mut maybe_lights = List::with_capacity(CEILINGS.nest_maybe_lights);
        for _ in 0..CEILINGS.nest_maybe_lights {
            maybe_lights.push(Some(Light::Green)).expect("room");
        }
        let mut grid = List::with_capacity(CEILINGS.nest_grid);
        for _ in 0..CEILINGS.nest_grid {
            let mut row = List::with_capacity(CEILINGS.nest_grid_item);
            for _ in 0..CEILINGS.nest_grid_item {
                row.push(u16::MAX).expect("room");
            }
            grid.push(row).expect("room");
        }
        let mut maybe_list = List::with_capacity(CEILINGS.nest_maybe_list_some);
        for _ in 0..CEILINGS.nest_maybe_list_some {
            maybe_list.push(u8::MAX).expect("room");
        }
        let mut shapes = List::with_capacity(CEILINGS.nest_shapes);
        for _ in 0..CEILINGS.nest_shapes {
            shapes.push(Shape::Strings(Box::new(full_strings()))).expect("room");
        }
        let mut secrets = List::with_capacity(CEILINGS.nest_secrets);
        for _ in 0..CEILINGS.nest_secrets {
            secrets.push(skein_lib::Secret::new(bytes(CEILINGS.nest_secrets_item))).expect("room");
        }
        let mut stamps = List::with_capacity(CEILINGS.nest_stamps);
        for _ in 0..CEILINGS.nest_stamps {
            stamps.push(full_stamp()).expect("room");
        }
        let mut pairs = List::with_capacity(CEILINGS.nest_pairs);
        for _ in 0..CEILINGS.nest_pairs {
            pairs.push([0xff; 2]).expect("room");
        }
        let nest = Nest {
            blanks,
            lights,
            maybe_lights,
            grid,
            maybe_list: Some(maybe_list),
            maybe_numbers: Some(full_numbers()),
            shapes,
            secrets,
            stamps,
            pairs,
        };
        Every {
            numbers: full_numbers(),
            strings: full_strings(),
            light: Light::Green,
            shape: Shape::Strings(Box::new(full_strings())),
            stamp: full_stamp(),
            nest,
            empty: Blank {},
            held: bytes(CEILINGS.every_held),
            items: bytes(CEILINGS.every_items),
        }
    }

    #[test]
    fn a_value_grows_on_first_decode_and_reuses_its_tables_on_the_next() {
        use skein_codec::value::Value;
        use skein_codec_tests::every::{CEILINGS, Every};
        // Every's largest tree has 99 nodes; room also admits every other fixture.
        const ROOM: u32 = 256;
        let schema = skein_codec::schema::parse(include_bytes!("../../schema/every-v1.schema")).expect("schema");
        let root = schema.find(b"Every").expect("root");
        let full = full_every();
        assert_eq!(
            u64::from(full.measure(&CEILINGS).expect("every at ceilings")),
            Every::encoded_bound(&CEILINGS).expect("wire bound")
        );
        let wire = full.encode(&CEILINGS).expect("every at ceilings");
        drop(full);

        let meter = Meter::new();
        meter.start();
        let mut value = Value::with_room(ROOM);
        let measured = meter.end();
        let bound = Value::decoder_bound(ROOM).expect("heap bound");
        assert_eq!(measured.peak(), 0, "empty room allocates nothing");
        assert_eq!(measured.held(), 0, "empty room holds nothing");

        let meter = Meter::new();
        meter.start();
        value.decode(&schema, root, wire).expect("full every decodes");
        let measured = meter.end();
        assert!(measured.held() <= bound, "grown entries stay within their room");
        // Realloc counts both the old and the new buffer at the growth peak.
        assert!(measured.peak() <= 2 * bound, "first decode growth stays bounded");

        // Reuse the allocated room after moving out its previous input.
        let previous = value.take();
        let wire = previous.expose().into();
        drop(previous);
        let meter = Meter::new();
        meter.start();
        value.decode(&schema, root, wire).expect("reused every decodes");
        let measured = meter.end();
        assert_eq!(measured.peak(), 0, "decoding again allocates nothing");
        assert_eq!(measured.held(), 0, "decoding again holds no new allocation");
    }
}
