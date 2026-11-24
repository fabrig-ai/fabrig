//! Compare or refresh committed codec drift files (skein's data/codec/golden.md, 1).
use std::{fs, path::Path};

pub fn check_text(path: &Path, text: &str, write: bool, again: &str) {
    if write {
        fs::create_dir_all(path.parent().expect("drift file parent")).expect("create drift directory");
        fs::write(path, text).expect("write drift file");
    } else {
        let committed = fs::read_to_string(path).unwrap_or_else(|why| panic!("{}: {why}; {again}", path.display()));
        assert_eq!(committed, text, "{}: {again}", path.display());
    }
}
