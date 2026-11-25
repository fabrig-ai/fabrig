//! Shipped schemas preserve fingerprints through reprinting (data/codec/fingerprints.md, 1.4).

use crate::support::generated;
use crate::support::reprint;

#[cfg(test)]
mod tests {
    use super::{
        generated,
        reprint::{renamed, reprint, reprint_metadata},
    };
    use core::fmt::Write;
    use skein_codec::schema::{DeclIx, Schema, parse};

    fn shipped_schemas() -> Vec<(&'static str, &'static str)> {
        generated::families().into_iter().map(|family| (family.name, family.text)).collect()
    }

    fn shipped_schema(name: &str, text: &str) -> Schema {
        parse(text.as_bytes()).map_err(|error| format!("{name}: {error:?}")).expect("valid shipped schema")
    }

    #[test]
    fn every_schema_preserves_fingerprints_after_reprinting() {
        for (name, text) in shipped_schemas() {
            let schema = shipped_schema(name, text);
            let expected = schema.fingerprints();
            for source in [reprint(&schema), reprint_metadata(&schema)] {
                let other = shipped_schema(name, &source);
                let fingerprints = other.fingerprints();
                assert_eq!(expected.family(), fingerprints.family(), "{name}");
                for (at, rename) in (0..schema.declarations()).zip(renamed(&schema)) {
                    let at = DeclIx::new(at);
                    let renamed_at = other.find(rename.new.as_bytes()).expect("renamed declaration");
                    assert_eq!(
                        expected.declaration(at),
                        fingerprints.declaration(renamed_at),
                        "{name}: {}",
                        rename.old
                    );
                }
            }
        }
    }

    #[test]
    fn every_fixture_family_digest_is_golden() {
        for (version, text, digest) in [
            (
                1_u16,
                include_str!("../../schema/every-v1.schema"),
                "8b13d594a02c726d494e5811152222d9f3d294c9f396773f71b517f000e786fb",
            ),
            (
                3_u16,
                skein_codec_tests::EVERY_SCHEMA,
                "73a07d871952655755d3037ba0736f4689ccacb64d262aea7032c9218526aeef",
            ),
        ] {
            let schema = parse(text.as_bytes()).expect("every schema");
            let mut hex = String::new();
            for byte in schema.fingerprints().family().digest() {
                write!(hex, "{byte:02x}").expect("string write");
            }
            assert_eq!(hex, digest, "every-v{version}");
        }
    }

    #[test]
    fn plain_secret_family_digest_is_golden() {
        let schema = parse(skein_codec_tests::SECRETS_SCHEMA.as_bytes()).expect("secret schema");
        let mut hex = String::new();
        for byte in schema.fingerprints().family().digest() {
            write!(hex, "{byte:02x}").expect("string write");
        }
        assert_eq!(hex, "2200dc3e6cdd4d9d39470fa819a334d7a520502516089dff06c75bf2d1dcb05a");
    }

    #[test]
    fn reprint_exposes_local_renames_and_reverses_payload_boxing() {
        let schema = parse(b"family f 2\n# comment\nrecord A { x: u8 }\nenum E { a, b: A, c: boxed A }\nrecord R { x: A, y: bytes 8 holds foreign A, z: bytes 8 pieces A s: secret 8 holds foreign Token p: secret 8 pieces A }")
        .expect("reprint fixture");
        assert_eq!(
            reprint(&schema),
            concat!(
                "family f 2\n",
                "record AR { x_r: u8, }\n",
                "enum ER { a_r, b_r: boxed AR, c_r: AR, }\n",
                "record RR { x_r: AR, y_r: bytes 8 holds foreign A, z_r: bytes 8 pieces AR, s_r: secret 8 holds foreign Token, p_r: secret 8 pieces AR, }\n",
            )
        );
        let names =
            renamed(&schema).into_iter().map(|rename| (rename.old, rename.new, rename.parts)).collect::<Vec<_>>();
        assert_eq!(
            names,
            vec![
                ("A".into(), "AR".into(), vec![("x".into(), "x_r".into())]),
                (
                    "E".into(),
                    "ER".into(),
                    vec![("a".into(), "a_r".into()), ("b".into(), "b_r".into()), ("c".into(), "c_r".into())]
                ),
                (
                    "R".into(),
                    "RR".into(),
                    vec![
                        ("x".into(), "x_r".into()),
                        ("y".into(), "y_r".into()),
                        ("z".into(), "z_r".into()),
                        ("s".into(), "s_r".into()),
                        ("p".into(), "p_r".into())
                    ]
                ),
            ]
        );
    }
}
