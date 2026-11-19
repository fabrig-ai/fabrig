//! The writer's stale walks, reprint and orphan-page removal
//! (temper/docs/design/client/journey-writer.md, 3).
use crate::{Refusal, io_refusal, names, printed, printed_page, run, tui};
use std::{fs, path::Path};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Client {
    Web,
    Tui,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Journey {
    pub client: Client,
    pub name: String,
}

/// Replay through the print's current hosts, without writing or starting children.
pub(crate) fn stale(web: &Path, tui: &Path) -> Result<Vec<Journey>, Refusal> {
    let mut stale = Vec::new();
    for (root, client) in [(web, Client::Web), (tui, Client::Tui)] {
        for name in names(root)? {
            let path = root.join(&name).join("recording");
            let recording = fs::read(&path).map_err(|_| io_refusal(&path))?;
            let stopped = match client {
                Client::Web => printed(&name, &recording)?.is_err(),
                Client::Tui => tui::printed(&name, &recording)?.is_err(),
            };
            if stopped {
                stale.push(Journey { client, name });
            }
        }
    }
    stale.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(stale)
}

/// The command's decisions; the shell supplies `again`, tests supply no process.
pub(crate) fn write(
    web: &Path,
    tui: &Path,
    stale: &[Journey],
    mut walk: impl FnMut(&Journey) -> Result<(), String>,
    mut show: impl FnMut(&str),
) -> Result<bool, String> {
    let mut stopped = false;
    for journey in stale {
        match walk(journey) {
            Ok(()) => show(&format!("walked again {}", journey.name)),
            Err(why) => {
                // A refusal can include a page or scratch diagnostic; keep one stdout line per walk.
                show(&format!("stopped {}: {}", journey.name, why.replace('\n', "; ")));
                stopped = true;
            }
        }
    }
    let unprinted = reprint(web, tui)?;
    Ok(stopped || unprinted)
}

/// Print both clients, keeping stale pages, then remove only numbered orphan pages.
pub(crate) fn reprint(web: &Path, tui: &Path) -> Result<bool, String> {
    let unprinted = run::reprint_roots(web, tui)?;
    for (root, client) in [(web, Client::Web), (tui, Client::Tui)] {
        remove_orphans(root, client).map_err(run::refusal)?;
    }
    Ok(unprinted)
}

fn remove_orphans(root: &Path, client: Client) -> Result<(), Refusal> {
    for entry in fs::read_dir(root).map_err(|_| io_refusal(root))? {
        let entry = entry.map_err(|_| io_refusal(root))?;
        let directory = entry.path();
        if !entry.file_type().map_err(|_| io_refusal(&directory))?.is_dir() {
            continue;
        }
        let recording = directory.join("recording");
        if recording.try_exists().map_err(|_| io_refusal(&recording))? {
            continue;
        }
        for entry in fs::read_dir(&directory).map_err(|_| io_refusal(&directory))? {
            let entry = entry.map_err(|_| io_refusal(&directory))?;
            let path = entry.path();
            let page = match client {
                Client::Web => printed_page(&path),
                Client::Tui => tui::printed_file(&path),
            };
            if page && entry.file_type().map_err(|_| io_refusal(&path))?.is_file() {
                fs::remove_file(&path).map_err(|_| io_refusal(&path))?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
