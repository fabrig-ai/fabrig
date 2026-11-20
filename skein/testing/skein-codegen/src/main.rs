//! Command line entry point for data/codec/README.md, section 6, and data/codec/command.md, section 8.

extern crate alloc;

use alloc::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, ExitCode};

use skein_codegen::diff::Rename;
use skein_codegen::patch::{self, Landed, Options};
use skein_codegen::{check_as, form_of, generate_as, parse};

const USAGE: &str = "usage: skein-codegen <schema>... <out-dir> | patch <crate> [--baseline] [--owner <name>] [--after <upstream> <level>] [--rename <family> <old> <new>] [--base <ref>] [--check]";

#[derive(Debug, PartialEq, Eq)]
enum Arguments {
    Generate { schemas: Vec<PathBuf>, out: PathBuf },
    Patch { owner: PathBuf, options: Options, base: Option<String>, check: bool },
}

fn arguments(values: &[String]) -> Result<Arguments, &'static str> {
    let mut args = values.iter();
    let first = args.next().ok_or(USAGE)?;
    let second = args.next().ok_or(USAGE)?;
    if first != "patch" {
        let (out, schemas) = values.split_last().ok_or(USAGE)?;
        return Ok(Arguments::Generate { schemas: schemas.iter().map(PathBuf::from).collect(), out: out.into() });
    }
    let mut options = Options::default();
    let mut base = None;
    let mut check = false;
    while let Some(option) = args.next() {
        match option.as_str() {
            "--baseline" => options.baseline = true,
            "--owner" => options.owner = Some(args.next().ok_or(USAGE)?.clone()),
            "--after" => {
                let upstream = args.next().ok_or(USAGE)?.clone();
                let level = args.next().ok_or(USAGE)?.parse().map_err(|_| USAGE)?;
                options.after.push((upstream, level));
            }
            "--rename" => {
                let family = args.next().ok_or(USAGE)?.clone();
                let old = args.next().ok_or(USAGE)?;
                let new = args.next().ok_or(USAGE)?.clone();
                let rename = if let Some((decl, old)) = old.split_once('.') {
                    Rename::Part { decl: decl.into(), old: old.into(), new }
                } else {
                    Rename::Declaration { old: old.clone(), new }
                };
                options.renames.push((family, rename));
            }
            "--base" => {
                let reference = args.next().ok_or(USAGE)?;
                if reference.starts_with('-') {
                    return Err(USAGE);
                }
                base = Some(reference.clone());
            }
            "--check" => check = true,
            _ => return Err(USAGE),
        }
    }
    Ok(Arguments::Patch { owner: second.into(), options, base, check })
}

#[expect(clippy::print_stderr, reason = "the command's errors are printed to stderr (data/codec/command.md 8)")]
fn main() -> ExitCode {
    skein_fail_stop::install();
    match execute() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

#[expect(
    clippy::print_stdout,
    reason = "the binary prints the run's report and the check's findings (data/codec/command.md 8)"
)]
#[expect(clippy::disallowed_methods, reason = "skein's round: reads the codegen command line")]
fn execute() -> Result<u8, Box<dyn core::error::Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match arguments(&args)? {
        Arguments::Generate { schemas, out } => {
            generate_files(&schemas, &out)?;
            Ok(0)
        }
        Arguments::Patch { owner, options, base, check } => {
            let landed = landed(&owner, base.as_deref())?;
            if check {
                let findings = patch::check(&owner, &landed)?;
                for finding in &findings {
                    println!("{finding}");
                }
                Ok(u8::from(!findings.is_empty()))
            } else {
                let report = patch::run(&owner, &landed, &options)?;
                print!("{report}");
                Ok(u8::from(!report.findings.is_empty()))
            }
        }
    }
}

fn git(owner: &Path, args: &[&str]) -> Result<Vec<u8>, Box<dyn core::error::Error>> {
    let output = Command::new("git").arg("-C").arg(owner).args(args).output()?;
    if !output.status.success() {
        return Err(format!("git {}: {}", args.join(" "), String::from_utf8_lossy(&output.stderr).trim()).into());
    }
    Ok(output.stdout)
}

fn landed(owner: &Path, base: Option<&str>) -> Result<Landed, Box<dyn core::error::Error>> {
    let reference = match base {
        Some(reference) => reference.to_owned(),
        None => String::from_utf8(git(owner, &["merge-base", "HEAD", "origin/main"])?)?.trim().to_owned(),
    };
    let paths = git(
        owner,
        &["ls-tree", "-r", "--name-only", "-z", &reference, "--", "patches", "schema/kept", "schema/wires"],
    )?;
    let mut files = BTreeMap::new();
    for bytes in paths.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let name = core::str::from_utf8(bytes)?;
        let path = PathBuf::from(name);
        if path.components().any(|part| !matches!(part, Component::Normal(_)))
            || !(path.starts_with("patches") || path.starts_with("schema/kept") || path == Path::new("schema/wires"))
        {
            return Err(format!("git returned a path outside the crate: {name}").into());
        }
        let object = format!("{reference}:./{name}");
        let bytes = git(owner, &["show", &object])?;
        files.insert(path, bytes);
    }
    Ok(Landed::new(files))
}

/// The command's texts and filename (data/codec/README.md, 4).
fn generate_files(schema_paths: &[PathBuf], out_dir: &Path) -> Result<(), Box<dyn core::error::Error>> {
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
    let form = form_of(first.parent().ok_or("schema has no directory")?, &schema.family)?;
    check_as(&schema, form)?;
    let output = generate_as(&schema, form);
    std::fs::create_dir_all(out_dir)?;
    let path = out_dir.join(format!("{}.rs", family.replace('-', "_")));
    std::fs::write(path, output.records)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Arguments, USAGE, arguments};
    use skein_codegen::diff::Rename;
    use skein_codegen::patch::Options;

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
        assert_eq!(
            read(&["schema", "out"]),
            Ok(Arguments::Generate { schemas: vec!["schema".into()], out: "out".into() })
        );
        assert_eq!(
            read(&["first", "second", "out"]),
            Ok(Arguments::Generate { schemas: vec!["first".into(), "second".into()], out: "out".into() })
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
    fn patch_options_preserve_order_and_both_rename_forms() {
        assert_eq!(
            read(&[
                "patch",
                "crate",
                "--baseline",
                "--owner",
                "shop",
                "--after",
                "first",
                "7",
                "--after",
                "second",
                "3",
                "--rename",
                "shelf",
                "Old",
                "New",
                "--rename",
                "shelf",
                "New.old",
                "new",
                "--base",
                "HEAD",
                "--check"
            ]),
            Ok(Arguments::Patch {
                owner: "crate".into(),
                options: Options {
                    baseline: true,
                    owner: Some("shop".into()),
                    after: vec![("first".into(), 7), ("second".into(), 3)],
                    renames: vec![
                        ("shelf".into(), Rename::Declaration { old: "Old".into(), new: "New".into() }),
                        ("shelf".into(), Rename::Part { decl: "New".into(), old: "old".into(), new: "new".into() })
                    ],
                },
                base: Some("HEAD".into()),
                check: true
            })
        );
    }

    #[test]
    fn patch_without_options_has_pure_defaults() {
        assert_eq!(
            read(&["patch", "crate"]),
            Ok(Arguments::Patch { owner: "crate".into(), options: Options::default(), base: None, check: false })
        );
    }

    #[test]
    fn incomplete_unknown_and_invalid_options_are_usage_errors() {
        for args in [
            vec![],
            vec!["schema"],
            vec!["patch", "crate", "--owner"],
            vec!["patch", "crate", "--after", "up", "no"],
            vec!["patch", "crate", "--rename", "shelf", "Old"],
            vec!["patch", "crate", "--base"],
            vec!["patch", "crate", "--base", "--help"],
            vec!["patch", "crate", "--unknown"],
        ] {
            assert_eq!(read(&args), Err(USAGE));
        }
    }
}
