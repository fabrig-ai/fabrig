//! Command line entry point for data/codec/golden.md, section 1, and data/codec/command.md, section 8.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use skein_codegen::{generate, parse};

const USAGE: &str = "usage: skein-codegen <schema>... [<maps>] <out-dir>";

#[derive(Debug, PartialEq, Eq)]
struct Arguments {
    schemas: Vec<PathBuf>,
    out: PathBuf,
}

fn arguments(values: &[String]) -> Result<Arguments, &'static str> {
    let (out, schemas) = values.split_last().ok_or(USAGE)?;
    if schemas.is_empty() {
        return Err(USAGE);
    }
    Ok(Arguments { schemas: schemas.iter().map(PathBuf::from).collect(), out: out.into() })
}

#[expect(clippy::print_stderr, reason = "the command's errors are printed to stderr (data/codec/command.md 8)")]
fn main() -> ExitCode {
    skein_fail_stop::install();
    match execute() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

#[expect(clippy::disallowed_methods, reason = "skein's round: reads the codegen command line")]
fn execute() -> Result<(), Box<dyn core::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let Arguments { schemas, out } = arguments(&args)?;
    generate_files(&schemas, &out)
}

/// The command's texts and filename (data/codec/README.md, 4).
fn generate_files(schema_paths: &[PathBuf], out_dir: &Path) -> Result<(), Box<dyn core::error::Error>> {
    let (schema_paths, maps_path) = match schema_paths.split_last() {
        Some((last, schemas)) if last.extension().is_some_and(|extension| extension == "maps") => (schemas, Some(last)),
        _ => (schema_paths, None),
    };
    let first = schema_paths.first().ok_or(USAGE)?;
    let (family, version) = first
        .file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| name.strip_suffix(".schema"))
        .and_then(|name| name.rsplit_once("-v"))
        .ok_or("expected <family>-v<N>.schema")?;
    if family.is_empty() || version.is_empty() || !version.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("expected <family>-v<N>.schema".into());
    }
    let mut source = String::new();
    for path in schema_paths {
        source.push_str(&std::fs::read_to_string(path)?);
    }
    let schema = parse(&source)?;
    let output = match maps_path {
        Some(path) => skein_codegen::translate(&schema, &std::fs::read_to_string(path)?)?,
        None => generate(&schema).records,
    };
    std::fs::create_dir_all(out_dir)?;
    let path = out_dir.join(format!("{}.rs", family.replace('-', "_")));
    std::fs::write(path, output)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Arguments, USAGE, arguments};

    fn read(args: &[&str]) -> Result<Arguments, &'static str> {
        arguments(&args.iter().map(|arg| (*arg).to_owned()).collect::<Vec<_>>())
    }

    #[test]
    fn schema_generation_writes_records_without_tests() {
        let scratch = skein_scratch::Scratch::new("codegen-records");
        let schema_directory = scratch.path().join("schema");
        std::fs::create_dir(&schema_directory).expect("schema directory");
        let schema_path = schema_directory.join("sample-v1.schema");
        let source = "family sample 1\nrecord Note { body: text 8 }";
        std::fs::write(&schema_path, source).expect("schema fixture");
        let out = scratch.path().join("src/generated");
        super::generate_files(&[schema_path], &out).expect("generate files");
        let output = skein_codegen::generate(&skein_codegen::parse(source).expect("valid schema"));
        assert_eq!(std::fs::read_to_string(out.join("sample.rs")).expect("written records"), output.records);
        assert!(!scratch.path().join("golden").exists());
        assert!(!scratch.path().join("tests").exists());
        assert_eq!(std::fs::read_dir(out).expect("output directory").count(), 1);
    }

    #[test]
    fn schema_generation_takes_schemas_then_output_directory() {
        assert_eq!(read(&["schema", "out"]), Ok(Arguments { schemas: vec!["schema".into()], out: "out".into() }));
        assert_eq!(
            read(&["first", "second", "out"]),
            Ok(Arguments { schemas: vec!["first".into(), "second".into()], out: "out".into() })
        );
    }

    #[test]
    fn split_schema_generation_concatenates_texts_in_order() {
        let scratch = skein_scratch::Scratch::new("codegen-split");
        let first = scratch.path().join("note-v3.schema");
        let second = scratch.path().join("note-v3-part1.schema");
        let a = "family note 3\nrecord First { body: text 8 }\n";
        let b = "record Second { first: First }\n";
        std::fs::write(&first, a).expect("first part");
        std::fs::write(&second, b).expect("second part");
        let out = scratch.path().join("generated");
        super::generate_files(&[first, second], &out).expect("generate split family");
        let output = skein_codegen::generate(&skein_codegen::parse(&format!("{a}{b}")).expect("valid schema"));
        assert_eq!(std::fs::read_to_string(out.join("note.rs")).expect("records"), output.records);
        assert_eq!(std::fs::read_dir(out).expect("output directory").count(), 1);
    }

    #[test]
    fn schema_generation_replaces_hyphens_in_the_stem() {
        let scratch = skein_scratch::Scratch::new("codegen-stem");
        let schema = scratch.path().join("worker-wire-v17.schema");
        std::fs::write(&schema, "family worker_wire 17\nrecord Note { body: text 8 }").expect("schema");
        let out = scratch.path().join("generated");
        super::generate_files(&[schema], &out).expect("generate family");
        assert!(out.join("worker_wire.rs").is_file());
        assert_eq!(std::fs::read_dir(out).expect("output directory").count(), 1);
    }

    #[test]
    fn invalid_or_unreadable_schema_writes_nothing() {
        let scratch = skein_scratch::Scratch::new("codegen-invalid");
        let first = scratch.path().join("note-v3.schema");
        let second = scratch.path().join("note-v3-part1.schema");
        std::fs::write(&first, "family note 3\nrecord Note { body: text 8 }\n").expect("first part");
        let out = scratch.path().join("generated");
        assert!(super::generate_files(&[first.clone(), second.clone()], &out).is_err());
        assert!(!out.exists());
        std::fs::write(&second, "not a schema").expect("invalid part");
        assert!(super::generate_files(&[first, second], &out).is_err());
        assert!(!out.exists());
    }

    #[test]
    fn patch_is_read_as_a_schema_path() {
        assert_eq!(read(&["patch", "out"]), Ok(Arguments { schemas: vec!["patch".into()], out: "out".into() }));
    }

    #[test]
    fn missing_schema_or_output_is_a_usage_error() {
        assert_eq!(read(&[]), Err(USAGE));
        assert_eq!(read(&["schema"]), Err(USAGE));
    }
}
