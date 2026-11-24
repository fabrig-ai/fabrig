//! Output paths and shared assets (temper/docs/design/client/journey-walks.md, 5.1).
use crate::{print, tui_print, walk::Client};
use std::{
    collections::BTreeSet,
    fs, io,
    path::{Component, Path, PathBuf},
};

pub const WEB: &str = "temper/docs/design/client/web/journeys";
pub const TUI: &str = "temper/docs/design/client/tui/journeys";

// Canonicalize existing prefixes before following .., including a link before .. .
// Missing suffixes remain usable without creating an output before usage is checked.
pub fn resolve(path: &Path) -> io::Result<PathBuf> {
    resolve_with(path, |prefix| fs::canonicalize(prefix))
}

/// Walk the prefixes in order, asking only about the prefix reached so far.
pub fn resolve_with(path: &Path, mut canonicalize: impl FnMut(&Path) -> io::Result<PathBuf>) -> io::Result<PathBuf> {
    let mut resolved = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                resolved.pop();
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                resolved.push(component.as_os_str());
                match canonicalize(&resolved) {
                    Ok(path) => resolved = path,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
        }
    }
    Ok(resolved)
}

pub fn write_if_changed(path: &Path, bytes: &[u8]) -> io::Result<bool> {
    match fs::read(path) {
        Ok(previous) if previous == bytes => return Ok(false),
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    Ok(true)
}

pub fn assets(repository: &Path, destination: &Path, client: Client) -> io::Result<bool> {
    let mut changed = false;
    asset_bytes(repository, client, |path, bytes| {
        changed |= write_if_changed(&destination.join(path), bytes)?;
        Ok(())
    })?;
    Ok(changed)
}

/// Supply each asset's bytes and repository-relative path, without staging any files.
pub fn asset_bytes(
    repository: &Path,
    client: Client,
    mut emit: impl FnMut(&Path, &[u8]) -> io::Result<()>,
) -> io::Result<()> {
    let (root, generated) = match client {
        Client::Web => (Path::new(WEB), print::beside_files()),
        Client::Tui => (Path::new(TUI), tui_print::beside_files()),
    };
    let overlay = root.join("overlay.css");
    let bytes = if repository.join(&overlay).is_file() { fs::read(repository.join(&overlay))? } else { Vec::new() };
    emit(&overlay, &bytes)?;
    for (name, bytes) in generated {
        emit(&root.join(name), &bytes)?;
    }
    if client == Client::Web {
        let mut visited = BTreeSet::new();
        for stylesheet in print::STYLESHEETS {
            let path = resolve(&repository.join(WEB).join("journey").join(stylesheet))?;
            // Generated files have already been emitted, even if absent in the checkout.
            if path.is_file() {
                read_asset(repository, &path, &mut visited, &mut emit)?;
            }
        }
    }
    Ok(())
}

fn read_asset(
    repository: &Path,
    source: &Path,
    visited: &mut BTreeSet<PathBuf>,
    emit: &mut impl FnMut(&Path, &[u8]) -> io::Result<()>,
) -> io::Result<()> {
    let source = fs::canonicalize(source)?;
    if !visited.insert(source.clone()) {
        return Ok(());
    }
    let relative = source.strip_prefix(repository).map_err(io::Error::other)?;
    let bytes = fs::read(&source)?;
    emit(relative, &bytes)?;
    if source.extension().is_some_and(|extension| extension == "css") {
        let css = std::str::from_utf8(&bytes).map_err(io::Error::other)?;
        let mut rest = css;
        while let Some((_, after)) = rest.split_once("url(") {
            let Some((url, after)) = after.split_once(')') else { break };
            rest = after;
            let url = url.trim().trim_matches(['\'', '"']);
            if url.is_empty() || url.starts_with(['/', '#']) || url.contains(':') {
                continue;
            }
            let url = url.split(['?', '#']).next().unwrap_or(url);
            let parent = source.parent().ok_or_else(|| io::Error::other("stylesheet has no parent"))?;
            read_asset(repository, &parent.join(url), visited, emit)?;
        }
    }
    Ok(())
}
