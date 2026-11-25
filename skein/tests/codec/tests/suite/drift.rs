//! Regeneration check for data/codec/golden.md, section 1.

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;
    use std::fs;
    use std::path::Path;

    use skein_codegen::{bounds_text, fixed_values, generate, missing, parse};

    const REGENERATE: &str =
        "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p skein-codec-tests --test suite -E 'test(/^drift::/)'";

    macro_rules! check_family {
        ($codec:ident, $schema:literal, $generated:literal, $bounds:literal, $goldens:expr) => {{
            use skein_codec_tests::$codec as codec;
            check(
                concat!("schema/", $schema, ".schema"),
                $generated,
                $goldens,
                concat!("bounds/", $bounds, ".bounds"),
                &bounds_text(codec::DECLARATIONS, codec::CEILINGS.values().as_slice()),
            );
        }};
    }

    #[test]
    fn every_v3_holds_every_construct() {
        let missing = missing(skein_codec_tests::EVERY_SCHEMA).expect("every-v3 valid");
        assert!(missing.is_empty(), "every-v3 lacks constructs: {missing:?}");
    }

    fn check_goldens(directory: &str, expected: Vec<skein_codegen::FixedValue>, write: bool) {
        let directory = Path::new(directory);
        let names = expected.iter().map(|value| value.name.clone()).collect::<BTreeSet<_>>();
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
        for value in expected {
            let path = directory.join(value.name);
            if write {
                fs::write(&path, value.bytes).expect("write golden bytes");
            } else {
                let committed = fs::read(&path).expect("golden bytes committed");
                assert_eq!(committed, value.bytes, "{}: golden bytes drifted", path.display());
            }
        }
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "skein's round: reads SKEIN_CODEGEN_WRITE, the codegen rewrite switch"
    )]
    fn check(schema_path: &str, generated_path: &str, golden_directory: Option<&str>, bounds_path: &str, bounds: &str) {
        let current = crate::support::generated::current_text(schema_path);
        let source = current.map_or_else(|| fs::read_to_string(schema_path).expect("schema readable"), str::to_owned);

        let schema = current
            .map_or_else(|| parse(&source).expect("schema valid"), |text| skein_codegen::values(text).schema.clone());
        let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
        check_text(Path::new(generated_path), &generate(&schema).records, write);
        check_text(Path::new(bounds_path), bounds, write);
        if schema_path == "schema/mapped-v1.schema" {
            let translation =
                skein_codegen::translate(&schema, skein_codec_tests::MAPPED_MAPS).expect("fixture translation");
            check_text(Path::new("src/generated/maps/mapped.rs"), &translation, write);
        }
        if let Some(directory) = golden_directory {
            check_goldens(
                directory,
                current.map_or_else(|| fixed_values(&schema), |text| skein_codegen::values(text).fixed.clone()),
                write,
            );
        }
    }

    fn text_drift(path: &Path, text: &str) -> Option<String> {
        (fs::read_to_string(path).ok().as_deref() != Some(text))
            .then(|| format!("{} drifted: {REGENERATE}", path.display()))
    }

    fn check_text(path: &Path, text: &str, write: bool) {
        if write {
            let parent = path.parent().expect("generated file has a parent");
            fs::create_dir_all(parent).expect("create generated directory");
            fs::write(path, text).expect("write generated text");
        } else {
            let drift = text_drift(path, text);
            assert!(drift.is_none(), "{}", drift.unwrap_or_default());
        }
    }

    #[test]
    fn a_changed_bounds_line_fails_drift_check() {
        use skein_codec_tests::v1 as sample;

        let scratch = skein_scratch::Scratch::new("bounds-drift");
        let path = scratch.path().join("sample.bounds");
        let bounds = bounds_text(sample::DECLARATIONS, sample::CEILINGS.values().as_slice());
        check_text(&path, &bounds, true);
        assert_eq!(text_drift(&path, &bounds), None);

        let (_, rest) = bounds.split_once('\n').expect("declaration line");
        fs::write(&path, ["changed declaration line\n", rest].concat()).expect("change one bounds line");
        let message = text_drift(&path, &bounds).expect("changed bounds report drift");
        assert!(message.contains("sample.bounds drifted: run SKEIN_CODEGEN_WRITE=1"), "{message}");
    }

    #[test]
    fn every_schema_code_and_goldens_have_no_drift() {
        check_family!(every, "every-v1", "src/every/v1.rs", "every-v1", Some("golden/every-v1"));
    }

    #[test]
    fn every_v2_schema_code_and_goldens_have_no_drift() {
        check_family!(every_v2, "every-v2", "src/every/v2.rs", "every-v2", Some("golden/every-v2"));
    }

    #[test]
    fn every_v3_schema_code_and_goldens_have_no_drift() {
        check_family!(every_v3, "every-v3", "src/every/v3.rs", "every-v3", Some("golden/every-v3"));
    }

    #[test]
    fn growth_schema_code_has_no_drift() {
        check_family!(growth, "growth-v1", "src/growth/v1.rs", "growth-v1", None);
    }

    #[test]
    fn secret_schema_code_has_no_drift() {
        check_family!(secrets, "secrets-v1", "src/secrets/v1.rs", "secrets-v1", None);
    }

    #[test]
    fn sample_schema_code_has_no_drift() {
        check_family!(v1, "sample-v1", "src/generated/sample.rs", "sample", None);
    }

    #[test]
    fn scalar_schema_code_has_no_drift() {
        check_family!(scalars, "scalars-v1", "src/scalars/v1.rs", "scalars-v1", None);
    }

    #[test]
    fn mapped_records_and_translation_have_no_drift() {
        check_family!(mapped_records, "mapped-v1", "src/generated/mapped.rs", "mapped", None);
    }

    #[test]
    fn lint_fixture_schema_code_has_no_drift() {
        check_family!(lints, "lints-v1", "src/lints/v1.rs", "lints-v1", None);
    }
}
