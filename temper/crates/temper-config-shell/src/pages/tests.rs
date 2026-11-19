use super::*;

#[test]
fn help_lists_only_the_shapes_public_rows_and_their_defaults_and_lines() {
    for shape in [Shape::Standalone, Shape::EngineAlone, Shape::WorkerAlone] {
        let page = String::from_utf8(help(shape).into_vec()).expect("UTF-8");
        let (table, _) = page.split_once("## Example file").expect("example follows keys");
        assert!(page.contains(crate::FILE));
        assert_eq!(page.matches("restart").count(), 1);
        for row in rows::ROWS {
            let listed = table.lines().find(|line| line.starts_with(&format!("| `{}` |", key(row))));
            if row.set == Set::Public && row.shapes.contains(&shape) {
                let listed = listed.unwrap_or_else(|| panic!("missing {} for {shape:?}", row.key));
                assert!(listed.contains(row.default), "default: {}", row.key);
                assert!(listed.contains(row.line), "line: {}", row.key);
            } else {
                assert!(listed.is_none(), "private or other shape's key: {}", row.key);
            }
        }
        assert_eq!(
            table.lines().filter(|line| line.starts_with("| `")).count(),
            rows::ROWS.iter().filter(|row| row.set == Set::Public && row.shapes.contains(&shape)).count()
        );
    }
    let worker = String::from_utf8(help(Shape::WorkerAlone).into_vec()).expect("UTF-8");
    for key in ["engine", "name", "[programs]", "programs.git", "programs.claude", "[environment]"] {
        assert!(worker.contains(&format!("| `{key}` |")), "worker key {key}");
    }
    assert!(!worker.contains("`listen`"));
}

#[test]
fn help_examples_are_the_files_from_the_table() {
    for (shape, expected) in [
        (Shape::Standalone, "listen = \"127.0.0.1:7100\""),
        (Shape::EngineAlone, "workers = \"10.0.0.1:7200\""),
        (Shape::WorkerAlone, "engine = \"10.0.0.1:7200\""),
    ] {
        let page = String::from_utf8(help(shape).into_vec()).expect("UTF-8");
        let file = page.split_once("```toml\n").expect("TOML example").1.strip_suffix("```\n").expect("fence");
        assert_eq!(file.as_bytes(), render(&example(shape), shape).as_ref());
        assert!(file.contains(expected));
        assert!(file.starts_with("# temper's "));
        if shape == Shape::EngineAlone {
            for value in [
                "https://temper.example.com",
                "sam@example.com",
                "1234.apps.googleusercontent.com",
                "secrets/google.secret",
            ] {
                assert!(file.contains(value), "engine example {value}");
            }
        }
        if shape == Shape::WorkerAlone {
            assert!(file.contains("name = \"w1\""));
        }
    }
}

#[test]
fn configuration_is_the_table_reference() {
    let rendered = reference();
    let text = std::str::from_utf8(&rendered).expect("UTF-8");
    assert_eq!(text.matches("restart").count(), 1);
    assert!(text.len() <= temper_docs::PAGE_BYTES);
    assert!(text.contains("| S E | internal |"));
    assert!(text.contains("| S E | test |"));
    for row in rows::ROWS {
        let line = text.lines().find(|line| line.starts_with(&format!("| `{}` |", key(row)))).expect("reference row");
        for words in [row.default, row.line, &row.form.words()] {
            assert!(line.contains(words), "{}: {words}", row.key);
        }
        for member in row.members {
            assert!(text.contains(&format!("`{member}` → `")));
        }
    }
    assert!(text.contains("`application.clients.listener` → `listen`"));
    assert_eq!(text.matches("`application.push.services` →").count(), 1);
    assert!(text.contains("`application.push.services` → `test.push.services`"));
    for member in temper_service_core::startup::moved::MEMBERS {
        assert!(text.contains(member.path));
        assert!(text.contains(member.words));
    }
    assert!(text.ends_with("A job that fails for good holds the change, as a resolution that fails does.\n"));
    let page = skein_scratch::paths::tree().join("docs/help/reference/configuration.md");
    if temper_test_env::rewrite() {
        std::fs::write(page, rendered).expect("write reference");
    } else {
        assert_eq!(
            std::fs::read(page).expect("configuration page"),
            rendered.as_ref(),
            "regenerate with SKEIN_CODEGEN_WRITE=1 and this test"
        );
    }
}
