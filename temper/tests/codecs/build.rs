//! Embed the deployed schema texts for the floor guard
//! (temper/docs/design/protocol/floor.md, 6).

use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

fn entries(directory: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<_> =
        fs::read_dir(directory).expect("fixture directory").map(|entry| entry.expect("fixture entry").path()).collect();
    paths.sort();
    paths
}

fn files(directory: &Path, extension: &str) -> Vec<PathBuf> {
    if !directory.is_dir() {
        return Vec::new();
    }
    entries(directory).into_iter().filter(|path| path.extension().is_some_and(|ext| ext == extension)).collect()
}

fn contents(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|why| panic!("{}: {why}", path.display()))
}

fn main() {
    let deployed = Path::new("deployed");
    println!("cargo:rerun-if-changed={}", deployed.display());
    let mut fixtures = String::from("const DEPLOYED: &[Deployed] = &[\n");
    for build in entries(deployed).into_iter().filter(|path| path.is_dir()) {
        let directory = build.file_name().and_then(|name| name.to_str()).expect("deployed directory");
        let inventory = contents(&build.join("inventory.txt"));
        let schemas: Vec<_> = files(&build, "schema")
            .iter()
            .map(|path| {
                let name = path.file_name().and_then(|name| name.to_str()).expect("schema filename");
                (name.to_owned(), contents(path))
            })
            .collect();
        writeln!(fixtures, "Deployed {{ directory: {directory:?}, inventory: {inventory:?}, texts: &{schemas:?} }},")
            .expect("Rust fixture");
    }
    fixtures.push_str("];\n");
    #[expect(clippy::disallowed_methods, reason = "Cargo supplies the build script's output directory")]
    let output = PathBuf::from(std::env::var_os("OUT_DIR").expect("Cargo output directory"));
    fs::write(output.join("stored_layout_fixtures.rs"), fixtures).expect("write layout fixtures");
}
