//! Compile the fixture schemas with bound names (codec.md, 6).

use core::fmt::Write;
use std::path::PathBuf;

use skein_codegen::{Declaration, Schema, generate, parse};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let directory = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output directory"));
    let mut modules = String::new();
    modules.push_str("pub mod bounds {\n");
    for (fixture, module) in [
        ("sample-v1", "v1"),
        ("scalars-v1", "scalars"),
        ("lints-v1", "lints"),
        ("secrets-v1", "secrets"),
        ("every-v1", "every"),
        ("every-v2", "every_v2"),
    ] {
        let path = format!("schema/{fixture}.schema");
        println!("cargo:rerun-if-changed={path}");
        let schema = parse(&std::fs::read_to_string(path).expect("fixture schema")).expect("valid fixture");
        let output = generate(&schema);
        let old_encoded = "worst_case_bytes";
        let old_decoded = "worst_case_heap";
        for old in [old_encoded, old_decoded] {
            assert!(!output.rust.contains(old), "{fixture}: old name {old}");
        }
        writeln!(modules, "pub mod {module} {{\n{}", output.rust).expect("generated module");
        comparisons(&schema, module, &mut modules);
        modules.push_str("}\n");
    }
    modules.push_str("}\n");
    std::fs::write(directory.join("named_fixtures.rs"), modules).expect("write build output");
}

fn comparisons(schema: &Schema, module: &str, out: &mut String) {
    let mut imports = vec!["CEILINGS", "encoded_bound", "decoded_bound"];
    for declaration in &schema.declarations {
        imports.push(match declaration {
            Declaration::Record(record) => &record.name,
            Declaration::Enum(enumeration) => &enumeration.name,
        });
    }
    writeln!(out, "#[cfg(test)]\nmod bound_tests {{\nuse super::{{{}}};", imports.join(", "))
        .expect("comparison module");
    for (index, declaration) in schema.declarations.iter().enumerate() {
        let name = match declaration {
            Declaration::Record(record) => &record.name,
            Declaration::Enum(enumeration) => &enumeration.name,
        };
        writeln!(out, "#[test]\nfn bounds_of_type_{index}() {{").expect("comparison string");
        for bound in ["encoded_bound", "decoded_bound"] {
            writeln!(out, "assert_eq!({name}::{bound}(&CEILINGS), skein_codec_tests::{module}::{name}::{bound}(&skein_codec_tests::{module}::CEILINGS));").expect("bound comparison");
        }
        out.push_str("}\n");
    }
    out.push_str("#[test]\nfn family_bounds_match() {\n");
    for bound in ["encoded_bound", "decoded_bound"] {
        writeln!(
            out,
            "assert_eq!({bound}(&CEILINGS), skein_codec_tests::{module}::{bound}(&skein_codec_tests::{module}::CEILINGS));"
        )
        .expect("family comparison");
    }
    out.push_str("}\n}\n");
}
