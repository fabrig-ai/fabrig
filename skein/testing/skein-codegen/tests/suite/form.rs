//! The owner selects the form before the command writes any output.

#[cfg(test)]
mod tests {
    use skein_codegen::{Form, form_of, generate_as, parse};
    use std::path::Path;
    use std::process::Command;

    fn run(first: &Path, second: &Path, out: &Path) -> std::process::Output {
        Command::new(skein_scratch::paths::binary("skein-codegen"))
            .arg(first)
            .arg(second)
            .arg(out)
            .output()
            .expect("codegen command")
    }

    #[test]
    fn clean_mark_selects_each_family_form() {
        let scratch = skein_scratch::Scratch::new("clean-lean-mark");
        std::fs::write(scratch.path().join("lean"), "# choices\n\n sample # owned\n other\n\n").expect("clean mark");
        for (family, form) in [("sample", Form::Lean), ("other", Form::Lean), ("sample-extra", Form::Sealed)] {
            assert_eq!(form_of(scratch.path(), family).expect("clean family form"), form);
        }
    }

    #[test]
    fn conflicted_mark_is_refused_before_generation() {
        let scratch = skein_scratch::Scratch::new("conflicted-lean-mark");
        let first = scratch.path().join("sample-v1.schema");
        let second = scratch.path().join("sample-v1-part1.schema");
        std::fs::write(&first, "family sample 1\nrecord Row { value: u8 }\n").expect("schema");
        std::fs::write(&second, "").expect("schema fragment");
        let mark = scratch.path().join("lean");
        let out = scratch.path().join("out");
        for marker in ["<<<<<<<head", "|||||||base", "=======", ">>>>>>>incoming"] {
            std::fs::write(&mark, format!("# choices\nsample\n{marker}\nother\n")).expect("conflicted mark");
            for family in ["sample", "other", "absent"] {
                let error = form_of(scratch.path(), family).expect_err("conflicted mark refused");
                assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
                assert!(error.to_string().contains(&format!("{}:3:", mark.display())), "{error}");
            }
            let result = run(&first, &second, &out);
            assert!(!result.status.success(), "a conflicted mark refuses the command");
            assert!(String::from_utf8_lossy(&result.stderr).contains(&format!("{}:3:", mark.display())));
            assert!(!out.exists(), "a conflicted mark writes nothing");
        }
    }

    #[test]
    fn command_reads_first_schema_mark_or_writes_nothing() {
        let scratch = skein_scratch::Scratch::new("lean-mark");
        let first_dir = scratch.path().join("first");
        let second_dir = scratch.path().join("second");
        std::fs::create_dir(&first_dir).expect("first schema directory");
        std::fs::create_dir(&second_dir).expect("second schema directory");
        let first = first_dir.join("sample-v1.schema");
        let second = second_dir.join("sample-v1-part1.schema");
        let a = "family sample 1\nrecord Words { words: text 3 }\n";
        let b = "record Row { words: Words }\n";
        std::fs::write(&first, a).expect("first schema");
        std::fs::write(&second, b).expect("second schema");
        std::fs::write(second_dir.join("lean"), "sample").expect("ignored second mark");
        let schema = parse(&format!("{a}{b}")).expect("schema");
        let mark = first_dir.join("lean");
        let out = scratch.path().join("out");
        assert_eq!(form_of(&first_dir, "sample").expect("absent mark"), Form::Sealed);
        for (text, form) in [
            (None, Form::Sealed),
            (Some("# choices\n other\n sample # owned\n"), Form::Lean),
            (Some("another-family\n"), Form::Sealed),
        ] {
            if let Some(text) = text {
                std::fs::write(&mark, text).expect("mark");
            }
            let result = run(&first, &second, &out);
            assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
            let written = std::fs::read_to_string(out.join("sample.rs")).expect("records");
            assert!(written == generate_as(&schema, form).records, "command ignored the owner's {form:?} mark");
        }
        std::fs::remove_file(&mark).expect("remove mark");
        std::fs::create_dir(&mark).expect("unreadable mark is a directory");
        let refused = scratch.path().join("refused");
        let result = run(&first, &second, &refused);
        assert!(!result.status.success());
        assert_ne!(result.stderr, []);
        assert!(!refused.exists(), "an unreadable mark writes nothing");
        std::fs::remove_dir(&mark).expect("remove directory mark");
        std::fs::write(&mark, [0xff]).expect("non-text mark");
        assert!(!run(&first, &second, &refused).status.success());
        assert!(!refused.exists());
        std::fs::write(&mark, "sample").expect("lean mark");
        std::fs::write(&first, "family sample 1\nversioned record Record {}").expect("sealed-only name");
        std::fs::write(&second, "").expect("no second declaration");
        let result = run(&first, &second, &refused);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("lean type name Record is reserved"));
        assert!(!refused.exists(), "form validation precedes output creation");
        std::fs::remove_file(&mark).expect("sealed again");
        assert!(run(&first, &second, &out).status.success(), "sealed Record remains valid");
    }
}
