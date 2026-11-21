//! Deterministic arbitrary wire bytes for every top-level sample record
//! (data/codec/README.md, section 6; testing-strategy.md, section 8).

#[cfg(test)]
mod tests {
    use skein_codec_tests::{scalars, v1};
    use skein_lib::{Reader, Rng};
    use v1::Record;

    fn check_field(bytes: &[u8]) {
        if let Ok(value) = v1::Field::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
            assert_eq!(value.encode(&v1::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
        }
    }

    fn check_report(bytes: &[u8]) {
        if let Ok(value) = v1::Report::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
            assert_eq!(value.encode(&v1::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
        }
    }

    fn check_marker(bytes: &[u8]) {
        if let Ok(value) = v1::Marker::decode(&v1::CEILINGS, &mut Reader::new(bytes)) {
            assert_eq!(value.encode(&v1::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
        }
    }

    fn check_empty(bytes: &[u8]) {
        if let Ok(value) = scalars::Empty::decode(&scalars::CEILINGS, &mut Reader::new(bytes)) {
            let measured = value.measure(&scalars::CEILINGS).expect("decoded value measures");
            assert_eq!(usize::try_from(measured).expect("measured length"), bytes.len());
            assert_eq!(value.encode(&scalars::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
        }
    }

    fn check_scalars(bytes: &[u8]) {
        if let Ok(value) = scalars::Scalars::decode(&scalars::CEILINGS, &mut Reader::new(bytes)) {
            let measured = value.measure(&scalars::CEILINGS).expect("decoded value measures");
            assert_eq!(usize::try_from(measured).expect("measured length"), bytes.len());
            assert_eq!(value.encode(&scalars::CEILINGS).expect("decoded value encodes").as_ref(), bytes);
        }
    }

    fn check_all(bytes: &[u8]) {
        check_field(bytes);
        check_report(bytes);
        check_marker(bytes);
        check_empty(bytes);
        check_scalars(bytes);
    }

    #[test]
    fn arbitrary_and_mutated_fixed_value_bytes_never_panic_and_round_trip_if_valid() {
        let mut rng = Rng::new(0xC0DE_CAFE_8100_0001);
        let mut buffer = [0_u8; 128];
        for _round in 0_u32..10_000 {
            let length = usize::try_from(rng.below(129)).expect("bounded length");
            for byte in buffer.get_mut(..length).expect("within buffer") {
                *byte = u8::try_from(rng.next_u64() & 0xff).expect("low byte");
            }
            check_all(buffer.get(..length).expect("within buffer"));
        }

        let fixed_values = [skein_codec_tests::SAMPLE_SCHEMA, skein_codec_tests::SCALARS_SCHEMA]
            .into_iter()
            .flat_map(|text| skein_codegen::values(text).fixed.iter())
            .collect::<Vec<_>>();
        for name in [
            "record_field_full.bin",
            "record_report_full.bin",
            "record_marker_full.bin",
            "record_empty_full.bin",
            "record_scalars_full.bin",
        ] {
            let fixed_value =
                &fixed_values.iter().find(|fixed_value| fixed_value.name == name).expect("seed fixed-rule value").bytes;
            check_all(fixed_value);
            for _round in 0_u32..200 {
                let mut mutated = fixed_value.clone();
                if !mutated.is_empty() {
                    let index = usize::try_from(rng.below(skein_lib::len_u64(mutated.len()))).expect("index fits");
                    *mutated.get_mut(index).expect("chosen byte") =
                        u8::try_from(rng.next_u64() & 0xff).expect("low byte");
                }
                check_all(&mutated);
            }
        }
    }
}
