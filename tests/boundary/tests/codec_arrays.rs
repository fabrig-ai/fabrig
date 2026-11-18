//! Generated codecs name widths, not schema counts (temper's draft/codec-shape.md, 6.1).

use std::{fs, path::Path, process::Command};

use syn::visit::Visit;

#[derive(Default)]
struct CountedArrays(usize);

impl<'ast> Visit<'ast> for CountedArrays {
    fn visit_type_array(&mut self, array: &'ast syn::TypeArray) {
        let fixed_bytes = if let syn::Type::Path(element) = array.elem.as_ref() {
            element.qself.is_none() && element.path.is_ident("u8")
        } else {
            false
        };
        if !fixed_bytes {
            self.0 += 1;
        }
        syn::visit::visit_type_array(self, array);
    }
}

fn counted_arrays(source: &str) -> usize {
    let source = syn::parse_file(source).expect("codec source is valid Rust");
    let mut arrays = CountedArrays::default();
    arrays.visit_file(&source);
    arrays.0
}

fn findings(root: &Path) -> Vec<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z", "--", "*.rs"])
        .output()
        .expect("list tracked codecs");
    assert!(output.status.success(), "git ls-files: {}: {}", output.status, String::from_utf8_lossy(&output.stderr));
    let mut failures = Vec::new();
    for raw in output.stdout.split(|byte| *byte == 0).filter(|path| !path.is_empty()) {
        let path = std::str::from_utf8(raw).expect("tracked path is UTF-8");
        if path.contains("/src/generated/") || path.starts_with("skein/tests/codec/src/") {
            let source = fs::read_to_string(root.join(path)).unwrap_or_else(|why| panic!("{path}: {why}"));
            let count = counted_arrays(&source);
            if count != 0 {
                failures.push(format!("{path}: {count} array types count schema items; only [u8; N] widths are permitted"));
            }
        }
    }
    failures
}

#[test]
fn only_fixed_byte_array_types_are_permitted() {
    for (source, count) in [
        ("struct Fixed { width: [u8; 32], bytes: Box<[u8]> }", 0),
        ("const NAMES: &[&str] = &[\"one\"]; fn values() -> Box<[u32]> { todo!() }", 0),
        ("struct Parts { flags: [bool; 4] }", 1),
        ("fn values() -> [u32; 8] { [0; 8] }", 1),
        ("struct Nested { widths: Box<[[u8; 4]; 2]> }", 1),
        ("// [bool; 4]\nconst TEXT: &str = \"[u32; 8]\";", 0),
    ] {
        assert_eq!(counted_arrays(source), count, "{source}");
    }
}

#[test]
fn the_check_reads_tracked_generated_modules_and_skein_codec_fixtures() {
    let base = skein_scratch::paths::workspace().join("target");
    fs::create_dir_all(&base).expect("scratch parent within the checkout");
    let scratch = skein_scratch::Scratch::new_in(&base, "codec-arrays");
    let root = scratch.path();
    let tracked = [
        ("one/src/generated/codec.rs", "struct Bad { flags: [bool; 4] }"),
        ("two/src/generated/nested/codec.rs", "fn bad() -> [u32; 2] { [0; 2] }"),
        ("skein/tests/codec/src/every/v1.rs", "struct Bad { flags: [bool; 4] }"),
        ("one/src/generated/fixed.rs", "struct Fixed { bytes: [u8; 32] }"),
        ("one/src/ordinary.rs", "struct Ordinary { flags: [bool; 4] }"),
    ];
    for (path, text) in tracked {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("fixture directory")).expect("fixture directories");
        fs::write(path, text).expect("fixture source");
    }
    let status = Command::new("git").arg("-C").arg(root).args(["init", "-q"]).status().expect("fixture repository");
    assert!(status.success());
    let status = Command::new("git").arg("-C").arg(root).args(["add", "."]).status().expect("track fixtures");
    assert!(status.success());
    fs::write(root.join("one/src/generated/untracked.rs"), "struct Untracked { flags: [bool; 4] }")
        .expect("untracked fixture");
    let failures = findings(root);
    assert_eq!(failures.len(), 3, "{failures:?}");
    for path in tracked.iter().take(3).map(|(path, _)| path) {
        assert!(failures.iter().any(|failure| failure.starts_with(path)), "{path}: {failures:?}");
    }
}

#[test]
fn generated_codecs_name_no_array_counts() {
    let failures = findings(&skein_scratch::paths::workspace());
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
