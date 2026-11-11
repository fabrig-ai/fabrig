use skein_lib::Writer;
use temper_service::limits;

type Renderer = fn(&mut Writer) -> Result<(), skein_lib::Overflow>;

#[test]
fn the_pages_limits_are_what_the_tables_render() {
    let directory = skein_scratch::paths::tree().join("docs/help/reference");
    let pages: [(&str, Renderer); 2] = [("limits", limits::reference), ("limits-engine", limits::engine_reference)];
    let write = temper_test_env::rewrite();
    for (name, render) in pages {
        let mut measured = Writer::new(temper_docs::PAGE_BYTES);
        render(&mut measured).expect("limits page fits PAGE_BYTES");
        let mut out = Writer::new(measured.written());
        render(&mut out).expect("limits page fits its measured length");
        let rendered = String::from_utf8(out.finish().into_vec()).expect("reference UTF-8");
        assert!(rendered.len() <= temper_docs::PAGE_BYTES, "{name}: {} bytes", rendered.len());
        render(&mut Writer::new(rendered.len() - 1)).expect_err("one byte short");
        let path = directory.join(name).with_extension("md");
        if write {
            std::fs::write(path, &rendered).expect("write limits reference");
        } else {
            assert_eq!(
                std::fs::read_to_string(path).expect("limits page"),
                rendered,
                "{name} should be:\n{rendered}; regenerate with SKEIN_CODEGEN_WRITE=1 and this test"
            );
            let page = temper_docs::find(name.as_bytes()).expect("compiled limits page");
            assert_eq!(page.kind, temper_docs::Kind::Reference);
            assert_eq!(page.text, rendered, "compiled {name} should be:\n{rendered}");
        }
    }
}
