//! Regeneration check for data/codec/README.md, section 6.

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;
    use std::fs;
    use std::path::Path;

    use skein_codegen::{generate, goldens, missing, parse, unpinned};

    const FAMILIES: [(&str, &str, Option<&str>); 8] = [
        ("schema/every-v1.schema", "src/every/v1.rs", Some("golden/every-v1")),
        ("schema/every-v2.schema", "src/every/v2.rs", Some("golden/every-v2")),
        ("schema/every-v3.schema", "src/every/v3.rs", Some("golden/every-v3")),
        ("schema/secrets-v1.schema", "src/secrets/v1.rs", None),
        ("schema/sample-v1.schema", "src/generated/sample.rs", None),
        ("schema/scalars-v1.schema", "src/scalars/v1.rs", None),
        ("schema/lints-v1.schema", "src/lints/v1.rs", None),
        ("schema/growth-v1.schema", "src/growth/v1.rs", None),
    ];

    #[test]
    fn every_v3_holds_every_construct() {
        let missing = missing(skein_codec_tests::EVERY_SCHEMA).expect("every-v3 valid");
        assert!(missing.is_empty(), "every-v3 lacks constructs: {missing:?}");
    }

    fn check_goldens(directory: &str, expected: Vec<skein_codegen::Golden>, write: bool) {
        let directory = Path::new(directory);
        let names = expected.iter().map(|golden| golden.name.clone()).collect::<BTreeSet<_>>();
        if write {
            fs::create_dir_all(directory).expect("create golden directory");
            for entry in fs::read_dir(directory).expect("golden directory") {
                let entry = entry.expect("golden entry");
                if !names.contains(&entry.file_name().into_string().expect("golden name")) {
                    fs::remove_file(entry.path()).expect("remove stale golden");
                }
            }
        }
        let committed = fs::read_dir(directory)
            .expect("golden directory")
            .map(|entry| entry.expect("golden entry").file_name().into_string().expect("golden name"))
            .collect::<BTreeSet<_>>();
        if !write {
            assert_eq!(committed, names, "{}: golden names drifted", directory.display());
        }
        for golden in expected {
            let path = directory.join(golden.name);
            if write {
                fs::write(&path, golden.bytes).expect("write golden bytes");
            } else {
                let committed = fs::read(&path).expect("golden bytes committed");
                assert_eq!(committed, golden.bytes, "{}: golden bytes drifted", path.display());
            }
        }
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "skein's round: reads SKEIN_CODEGEN_WRITE, the codegen rewrite switch"
    )]
    fn check(schema_path: &str) {
        let (_, generated_path, golden_directory) =
            FAMILIES.into_iter().find(|row| row.0 == schema_path).expect("family registered");
        let current = crate::support::generated::current_text(schema_path);
        let source = current.map_or_else(|| fs::read_to_string(schema_path).expect("schema readable"), str::to_owned);
        let missing = unpinned(&source).expect("schema valid");
        assert!(missing.is_empty(), "{schema_path}: unpinned constructs: {missing:?}");
        let schema = current
            .map_or_else(|| parse(&source).expect("schema valid"), |text| skein_codegen::values(text).schema.clone());
        let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
        check_text(generated_path, &generate(&schema).records, write);
        if let Some(directory) = golden_directory {
            check_goldens(
                directory,
                current.map_or_else(|| goldens(&schema), |text| skein_codegen::values(text).goldens.clone()),
                write,
            );
        }
    }

    fn check_text(path: &str, text: &str, write: bool) {
        if write {
            let parent = Path::new(path).parent().expect("generated file has a parent");
            fs::create_dir_all(parent).expect("create generated directory");
            fs::write(path, text).expect("write generated text");
        } else {
            let committed = fs::read_to_string(path).expect("generated text committed");
            assert!(
                committed == text,
                "{path}: run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p skein-codec-tests --test suite -E 'test(/^drift::/)'"
            );
        }
    }

    #[test]
    fn every_schema_code_and_goldens_have_no_drift() {
        check("schema/every-v1.schema");
    }

    #[test]
    fn every_v2_schema_code_and_goldens_have_no_drift() {
        check("schema/every-v2.schema");
    }

    #[test]
    fn every_v3_schema_code_and_goldens_have_no_drift() {
        check("schema/every-v3.schema");
    }

    #[test]
    fn growth_schema_code_has_no_drift() {
        check("schema/growth-v1.schema");
    }

    #[test]
    fn secret_schema_code_and_goldens_have_no_drift() {
        check("schema/secrets-v1.schema");
    }

    #[test]
    fn sample_schema_code_and_goldens_have_no_drift() {
        check("schema/sample-v1.schema");
    }

    #[test]
    fn scalar_schema_code_and_goldens_have_no_drift() {
        check("schema/scalars-v1.schema");
    }

    #[test]
    fn lint_fixture_schema_code_and_goldens_have_no_drift() {
        check("schema/lints-v1.schema");
    }
}
