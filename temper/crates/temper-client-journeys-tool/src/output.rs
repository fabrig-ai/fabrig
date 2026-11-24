//! Publish only admitted pages, with their assets (temper/docs/design/client/web/journeys.md, 5.1).
use crate::{Admitted, Client, REPOSITORY, Refusal, io_refusal};
use std::{
    fs, io,
    path::{Path, PathBuf},
};
use temper_client_walk_lines::layout::{self, TUI, WEB};

pub(super) trait Output {
    fn create_dir(&mut self, path: &Path) -> io::Result<()>;
    fn files(&mut self, directory: &Path) -> Result<Vec<PathBuf>, Refusal>;
    fn remove(&mut self, path: &Path) -> io::Result<()>;
    fn write(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()>;
    fn write_asset(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()>;
}

pub(super) fn publish(output: &mut impl Output, out: &Path, admitted: &Admitted) -> Result<(), Refusal> {
    let root = match admitted.client {
        Client::Web => WEB,
        Client::Tui => TUI,
    };
    let directory = out.join(root).join(&admitted.printed.name);
    output.create_dir(&directory).map_err(|_| io_refusal(&directory))?;
    for path in output.files(&directory)? {
        if numbered(&path, admitted.client) {
            output.remove(&path).map_err(|_| io_refusal(&path))?;
        }
    }
    for page in &admitted.pages {
        let path = directory.join(&page.name);
        output.write(&path, page.document.as_bytes()).map_err(|_| io_refusal(&path))?;
    }
    let repository = fs::canonicalize(REPOSITORY).map_err(|_| io_refusal(Path::new(REPOSITORY)))?;
    layout::asset_bytes(&repository, admitted.client, |path, bytes| output.write_asset(&out.join(path), bytes))
        .map_err(|_| io_refusal(out))
}

fn numbered(path: &Path, client: Client) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .and_then(|name| {
            name.strip_suffix(".html").or_else(|| if client == Client::Tui { name.strip_suffix(".txt") } else { None })
        })
        .is_some_and(|number| number.len() >= 2 && number.bytes().all(|byte| byte.is_ascii_digit()))
}

pub(super) struct Disk;

impl Output for Disk {
    fn create_dir(&mut self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    fn files(&mut self, directory: &Path) -> Result<Vec<PathBuf>, Refusal> {
        let mut files = Vec::new();
        for entry in fs::read_dir(directory).map_err(|_| io_refusal(directory))? {
            let entry = entry.map_err(|_| io_refusal(directory))?;
            let path = entry.path();
            if entry.file_type().map_err(|_| io_refusal(&path))?.is_file() {
                files.push(path);
            }
        }
        Ok(files)
    }

    fn remove(&mut self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }

    fn write(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        fs::write(path, bytes)
    }

    fn write_asset(&mut self, path: &Path, bytes: &[u8]) -> io::Result<()> {
        layout::write_if_changed(path, bytes).map(|_| ())
    }
}
