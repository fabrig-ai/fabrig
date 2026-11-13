//! Named limits agree with generated field order and encoded bounds (data/codec/README.md, 4 and 7.3).

#[cfg(test)]
mod tests {
    use skein_codec::schema::{
        self, DeclIx, FieldIx, LimitLevel, Schema, Shape, Type, TypeIx, limit_child, limit_name,
    };
    use skein_codec_tests::{every, every_v2, lints, scalars, secrets, v1};

    struct Family<'a> {
        schema: &'a str,
        names: &'a [&'a str],
        values: &'a [u32],
        bounds: &'a [(&'a [u8], Option<u64>)],
    }

    #[test]
    fn named_limits_match_generated_bounds_and_field_order() {
        let families = [
            Family {
                schema: "schema/every-v1.schema",
                names: &every::LIMIT_NAMES,
                values: &every::CEILINGS.values(),
                bounds: &[
                    (b"Blank", every::Blank::encoded_bound(&every::CEILINGS)),
                    (b"Numbers", every::Numbers::encoded_bound(&every::CEILINGS)),
                    (b"Strings", every::Strings::encoded_bound(&every::CEILINGS)),
                    (b"Stamp", every::Stamp::encoded_bound(&every::CEILINGS)),
                    (b"Nest", every::Nest::encoded_bound(&every::CEILINGS)),
                    (b"Every", every::Every::encoded_bound(&every::CEILINGS)),
                ],
            },
            Family {
                schema: "schema/every-v2.schema",
                names: &every_v2::LIMIT_NAMES,
                values: &every_v2::CEILINGS.values(),
                bounds: &[
                    (b"Blank", every_v2::Blank::encoded_bound(&every_v2::CEILINGS)),
                    (b"Numbers", every_v2::Numbers::encoded_bound(&every_v2::CEILINGS)),
                    (b"Texts", every_v2::Texts::encoded_bound(&every_v2::CEILINGS)),
                    (b"Stamp", every_v2::Stamp::encoded_bound(&every_v2::CEILINGS)),
                    (b"Nest", every_v2::Nest::encoded_bound(&every_v2::CEILINGS)),
                    (b"Every", every_v2::Every::encoded_bound(&every_v2::CEILINGS)),
                ],
            },
            Family {
                schema: "schema/sample-v1.schema",
                names: &v1::LIMIT_NAMES,
                values: &v1::CEILINGS.values(),
                bounds: &[
                    (b"Field", v1::Field::encoded_bound(&v1::CEILINGS)),
                    (b"Report", v1::Report::encoded_bound(&v1::CEILINGS)),
                    (b"Marker", v1::Marker::encoded_bound(&v1::CEILINGS)),
                ],
            },
            Family {
                schema: "schema/scalars-v1.schema",
                names: &scalars::LIMIT_NAMES,
                values: &scalars::CEILINGS.values(),
                bounds: &[
                    (b"Empty", scalars::Empty::encoded_bound(&scalars::CEILINGS)),
                    (b"Scalars", scalars::Scalars::encoded_bound(&scalars::CEILINGS)),
                ],
            },
            Family {
                schema: "schema/secrets-v1.schema",
                names: &secrets::LIMIT_NAMES,
                values: &secrets::CEILINGS.values(),
                bounds: &[(b"Credential", secrets::Credential::encoded_bound(&secrets::CEILINGS))],
            },
            Family {
                schema: "schema/lints-v1.schema",
                names: &lints::LIMIT_NAMES,
                values: &lints::CEILINGS.values(),
                bounds: &[
                    (b"HostTool", lints::HostTool::encoded_bound(&lints::CEILINGS)),
                    (b"Boundless", lints::Boundless::encoded_bound(&lints::CEILINGS)),
                    (b"Payload", lints::Payload::encoded_bound(&lints::CEILINGS)),
                    (b"OtherPayload", lints::OtherPayload::encoded_bound(&lints::CEILINGS)),
                    (b"OneByteItems", lints::OneByteItems::encoded_bound(&lints::CEILINGS)),
                    (b"OptionalPayload", lints::OptionalPayload::encoded_bound(&lints::CEILINGS)),
                    (b"FixedWidthItems", lints::FixedWidthItems::encoded_bound(&lints::CEILINGS)),
                    (b"LongRecord", lints::LongRecord::encoded_bound(&lints::CEILINGS)),
                ],
            },
        ];
        for family in families {
            check_family(family);
        }
    }

    fn check_family(family: Family<'_>) {
        let source = std::fs::read(family.schema).expect("fixture schema");
        let schema = schema::parse(&source).expect("checked fixture schema");
        let mut expected = Vec::new();
        let mut records = 0_usize;
        for at in 0..schema.declarations() {
            let declaration = schema.declaration(DeclIx::new(at));
            if let Shape::Record { first, count, .. } = declaration.shape {
                records = records.checked_add(1).expect("record count fits usize");
                for index in first.index()..first.index().checked_add(count).expect("field range") {
                    let field = schema.field(FieldIx::new(index));
                    let name = limit_name(schema.spelled(declaration.name), schema.spelled(field.name));
                    field_limits(&schema, field.ty, name, &mut expected);
                }
            }
        }
        assert_eq!(expected.len(), family.names.len(), "{}: all limits named", family.schema);
        assert_eq!(expected.len(), family.values.len(), "{}: all limits valued", family.schema);
        for ((name, value), (generated_name, generated_value)) in
            expected.iter().zip(family.names.iter().zip(family.values))
        {
            assert_eq!(name.as_ref(), generated_name.as_bytes(), "{}: limit name", family.schema);
            assert_eq!(value, generated_value, "{}: {generated_name}", family.schema);
        }
        assert_eq!(records, family.bounds.len(), "{}: every record checked", family.schema);
        for &(name, generated) in family.bounds {
            let at = schema.find(name).expect("fixture record exists");
            assert_eq!(
                schema.most_bytes_within(at, family.names, family.values),
                generated,
                "{}: {name:?}",
                family.schema
            );
        }
    }

    fn field_limits(schema: &Schema, mut at: TypeIx, mut name: Box<[u8]>, out: &mut Vec<(Box<[u8]>, u32)>) {
        // Every level takes at least one byte of source.
        for _ in 0..schema.text().len() {
            match schema.ty(at) {
                Type::List { bound, item } => {
                    out.push((name.clone(), bound));
                    name = limit_child(&name, LimitLevel::Item);
                    at = item;
                }
                Type::Option(item) => {
                    name = limit_child(&name, LimitLevel::Some);
                    at = item;
                }
                Type::Bytes { bound, .. } | Type::Secret { bound, .. } | Type::Text(bound) => {
                    out.push((name, bound));
                    break;
                }
                Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::Bool
                | Type::Duration
                | Type::Fixed(_)
                | Type::Named(_) => break,
            }
        }
    }
}
