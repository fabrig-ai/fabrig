//! Compile the fixture schemas with each bound vocabulary (codec.md, 6).

use core::fmt::Write;
use std::path::PathBuf;

use skein_codegen::{Declaration, Names, Schema, generate, generate_named, parse};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    let directory = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output directory"));
    let mut modules = String::new();
    for (mode, names) in [("bounds", Names::Bounds), ("both", Names::Both)] {
        writeln!(modules, "pub mod {mode} {{").expect("module string");
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
            let output = generate_named(&schema, names);
            assert_eq!(output.goldens, generate(&schema).goldens, "{fixture}: fixed wire bytes");
            let old_encoded = "worst_case_bytes";
            let old_decoded = "worst_case_heap";
            if names == Names::Bounds {
                for old in [old_encoded, old_decoded, "::worst_case("] {
                    assert!(!output.rust.contains(old), "{fixture}: old name {old}");
                }
            }
            writeln!(modules, "pub mod {module} {{\n{}", output.rust).expect("generated module");
            comparisons(&schema, module, names, &mut modules);
            modules.push_str("}\n");
        }
        modules.push_str("}\n");
    }
    std::fs::write(directory.join("named_fixtures.rs"), modules).expect("write build output");
}

fn comparisons(schema: &Schema, module: &str, names: Names, out: &mut String) {
    let old_encoded = "worst_case_bytes";
    let old_decoded = "worst_case_heap";
    let mut imports = vec!["CEILINGS", "encoded_bound", "decoded_bound"];
    if names == Names::Both {
        imports.extend([old_encoded, old_decoded]);
    }
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
        for (new, old) in [("encoded_bound", old_encoded), ("decoded_bound", old_decoded)] {
            writeln!(out, "assert_eq!({name}::{new}(&CEILINGS), skein_codec_tests::{module}::{name}::{old}(&skein_codec_tests::{module}::CEILINGS));").expect("bound comparison");
            if names == Names::Both {
                writeln!(out, "assert_eq!({name}::{old}(&CEILINGS), {name}::{new}(&CEILINGS));\nassert_eq!({name}::{old}_within(&CEILINGS), {name}::{new}_within(&CEILINGS));").expect("alias comparison");
            }
        }
        out.push_str("}\n");
    }
    out.push_str("#[test]\nfn family_bounds_match() {\n");
    for (new, old) in [("encoded_bound", old_encoded), ("decoded_bound", old_decoded)] {
        writeln!(
            out,
            "assert_eq!({new}(&CEILINGS), skein_codec_tests::{module}::{old}(&skein_codec_tests::{module}::CEILINGS));"
        )
        .expect("family comparison");
        if names == Names::Both {
            writeln!(out, "assert_eq!({old}(&CEILINGS), {new}(&CEILINGS));").expect("family alias comparison");
        }
    }
    out.push_str("}\n}\n");
}
