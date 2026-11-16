use skein_lib::List;
use std::path::PathBuf;
/// Source files under the extraction boundary; capacity is a source-tree bound.
const SOURCE_FILES: u32 = 128;
pub(crate) fn sources() -> List<(PathBuf, Box<str>)> {
    let mut paths = List::with_capacity(SOURCE_FILES);
    let mut sources = List::with_capacity(SOURCE_FILES);
    paths.push(PathBuf::from("src/jig.rs")).expect("module root");
    paths.push(PathBuf::from("src/jig")).expect("module directory");
    for index in 0..SOURCE_FILES {
        let Some(path) = paths.get(index).cloned() else {
            break;
        };
        if path.is_dir() {
            for entry in std::fs::read_dir(path).expect("read source directory") {
                paths.push(entry.expect("source entry").path()).expect("bounded sources");
            }
        } else if path.extension() == Some(std::ffi::OsStr::new("rs")) {
            let text = std::fs::read_to_string(&path).expect("read source").into_boxed_str();
            sources.push((path, text)).expect("bounded sources");
        }
    }
    sources
}
