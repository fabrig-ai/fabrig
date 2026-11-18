//! Computed values and construct coverage over codec fixtures (data/codec/README.md, 6).

#[cfg(test)]
mod tests {
    #[test]
    fn fixture_values_match_generation_and_constructs_are_pinned() {
        let directory = skein_scratch::paths::workspace().join("skein/tests/codec/schema");
        let mut files = std::fs::read_dir(directory)
            .expect("fixture directory")
            .map(|entry| entry.expect("fixture entry").path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "schema"))
            .collect::<Vec<_>>();
        files.sort();
        assert_ne!(files, Vec::<std::path::PathBuf>::new());
        for path in files {
            let source = std::fs::read_to_string(&path).expect("fixture schema");
            let schema = skein_codegen::parse(&source).expect("valid fixture schema");
            assert_eq!(skein_codegen::goldens(&schema), skein_codegen::generate(&schema).goldens, "{}", path.display());
            let unpinned = skein_codegen::unpinned(&source).expect("valid fixture constructs");
            assert_eq!(unpinned, [], "{}", path.display());
        }
    }
}
