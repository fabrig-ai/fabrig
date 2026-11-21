//! Only differing bytes are written; a stopped journey never reaches here (journey-walks.md, 5).
use std::{fs, io, path::Path};
pub(super) use temper_client_walk_lines::layout::assets;
use temper_client_walk_lines::{layout::write_if_changed as write, print};

pub(super) fn pages(directory: &Path, pages: &[print::Printed], in_place: bool) -> io::Result<bool> {
    fs::create_dir_all(directory)?;
    let mut changed = false;
    for page in pages {
        changed |= write(&directory.join(&page.name), page.document.as_bytes())?;
    }
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let numbered = name
            .strip_suffix(".html")
            .or_else(|| name.strip_suffix(".txt"))
            .is_some_and(|number| number.len() >= 2 && number.bytes().all(|byte| byte.is_ascii_digit()));
        if entry.file_type()?.is_file()
            && ((numbered && !pages.iter().any(|page| page.name == name)) || (in_place && name == "recording"))
        {
            fs::remove_file(entry.path())?;
            changed = true;
        }
    }
    Ok(changed)
}
