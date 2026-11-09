//! Regeneration check for codec.md, section 6.

#[cfg(test)]
mod tests {
    use alloc::collections::BTreeSet;
    use std::fs;
    use std::path::Path;

    use skein_codegen::{generate, parse};

    const FAMILIES: [(&str, &str, &str); 6] = [
        ("schema/every-v1.schema", "src/every/v1.rs", "golden/every-v1"),
        ("schema/every-v2.schema", "src/every/v2.rs", "golden/every-v2"),
        ("schema/secrets-v1.schema", "src/secrets/v1.rs", "golden/secrets-v1"),
        ("schema/sample-v1.schema", "src/generated/v1.rs", "golden/sample-v1"),
        ("schema/scalars-v1.schema", "src/scalars/v1.rs", "golden/scalars-v1"),
        ("schema/lints-v1.schema", "src/lints/v1.rs", "golden/lints-v1"),
    ];

    #[test]
    fn a_golden_directory_holds_its_family_alone() {
        let mut directories = BTreeSet::new();
        for (schema_path, _, golden_directory) in FAMILIES {
            assert!(directories.insert(golden_directory), "{schema_path}: shared golden directory {golden_directory}");
            let source = fs::read_to_string(schema_path).expect("schema readable");
            let schema = parse(&source).expect("schema valid");
            let names = generate(&schema).goldens.into_iter().map(|golden| golden.name).collect::<BTreeSet<_>>();
            let committed = fs::read_dir(golden_directory)
                .expect("golden directory")
                .map(|entry| entry.expect("golden entry").file_name().into_string().expect("golden name"))
                .collect::<BTreeSet<_>>();
            assert_eq!(committed, names, "{schema_path}: {golden_directory} holds only this family's goldens");
        }
    }

    #[expect(
        clippy::disallowed_methods,
        reason = "skein's round: reads SKEIN_CODEGEN_WRITE, the codegen rewrite switch"
    )]
    fn check(schema_path: &str) {
        let (_, generated_path, golden_directory) =
            FAMILIES.into_iter().find(|row| row.0 == schema_path).expect("family registered");
        let source = fs::read_to_string(schema_path).expect("schema readable");
        let schema = parse(&source).expect("schema valid");
        let output = generate(&schema);
        let write = std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1");
        if write {
            fs::write(generated_path, &output.rust).expect("write generated code");
        } else {
            let committed = fs::read_to_string(generated_path).expect("generated code committed");
            assert_eq!(
                committed, output.rust,
                "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p skein-codec-tests --test suite -E 'test(/^drift::/)'"
            );
        }
        let golden_directory = Path::new(golden_directory);
        for golden in output.goldens {
            let path = golden_directory.join(golden.name);
            if write {
                fs::create_dir_all(golden_directory).expect("create golden directory");
                fs::write(&path, golden.bytes).expect("write golden bytes");
            } else {
                let committed = fs::read(&path).expect("golden bytes committed");
                assert_eq!(
                    committed, golden.bytes,
                    "run SKEIN_CODEGEN_WRITE=1 cargo nextest run -p skein-codec-tests --test suite -E 'test(/^drift::/)'"
                );
            }
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
