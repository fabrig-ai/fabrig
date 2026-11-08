use super::*;
use std::path::{Path, PathBuf};

fn args(values: &[&str]) -> Vec<Box<[u8]>> {
    values.iter().map(|s| Box::from(s.as_bytes())).collect()
}
pub(crate) fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("temper-dirs-{}-{name}", std::process::id()));
    std::fs::create_dir(&path).expect("scratch");
    path
}
pub(crate) fn given(root: &Path) -> Directories {
    resolve(
        &args(&[
            "--config",
            root.to_str().expect("scratch text"),
            "--data",
            root.join("data").to_str().expect("data text"),
            "--cache",
            root.join("cache").to_str().expect("cache text"),
        ]),
        &[],
        b"/",
    )
    .expect("given")
    .0
}
