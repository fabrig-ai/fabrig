//! Print world-walked journeys (temper/docs/design/client/journey-walks.md, 5).
mod cli;
mod files;
pub use cli::{TABLE, answer, main};
use std::{
    fs, io,
    path::{Component, Path, PathBuf},
};
use temper_client_walk_lines::{
    print, tui_print,
    walk::{self, Client, Roots},
};
use temper_client_walk_world::{
    facts,
    walk::{Marks, Stopped, walk as execute},
};

pub const WEB: &str = "temper/docs/design/client/web/journeys";
pub const TUI: &str = "temper/docs/design/client/tui/journeys";

/// Captured output lets tests use their own repository and no child process.
#[derive(Debug)]
pub struct Answer {
    pub code: u8,
    pub stdout: String,
    pub stderr: String,
}

fn roots(repository: &Path) -> Roots {
    Roots { web: repository.join(WEB), tui: repository.join(TUI) }
}

fn moved(root: &Path) -> io::Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.path().is_dir() && walk::moved(&entry.path())? {
            let name = entry.file_name().into_string().map_err(|_| io::Error::other("non-UTF-8 journey name"))?;
            names.push(name);
        }
    }
    names.sort();
    Ok(names)
}

// Canonicalize existing prefixes before following .., including a link before .. .
// Missing suffixes remain usable without creating an output before usage is checked.
fn resolve(path: &Path) -> io::Result<PathBuf> {
    let mut resolved = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                resolved.pop();
            }
            Component::CurDir => {}
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                resolved.push(component.as_os_str());
                match fs::canonicalize(&resolved) {
                    Ok(path) => resolved = path,
                    Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                    Err(error) => return Err(error),
                }
            }
        }
    }
    Ok(resolved)
}

pub(crate) fn run(
    repository: &Path,
    command: &skein_shell::Command,
    names: &[String],
    out: Option<&Path>,
    in_place: bool,
) -> Answer {
    let mut answer = Answer { code: 0, stdout: String::new(), stderr: String::new() };
    match perform(repository, command, names, out, in_place, &mut answer) {
        Ok(()) => {}
        Err(error) => {
            answer.code = 3;
            answer.stderr = format!("temper-client-walk: {error}\n");
        }
    }
    answer
}

fn perform(
    repository: &Path,
    command: &skein_shell::Command,
    names: &[String],
    out: Option<&Path>,
    in_place: bool,
    answer: &mut Answer,
) -> io::Result<()> {
    use std::fmt::Write;
    let repository = fs::canonicalize(repository)?;
    let roots = roots(&repository);
    let mut available = moved(&roots.web)?;
    for name in moved(&roots.tui)? {
        if !available.contains(&name) {
            available.push(name);
        }
    }
    for name in names {
        if !available.contains(name) {
            answer.code = 2;
            answer.stderr = TABLE.refusal(command, &format!("{name} is not a moved journey"));
            return Ok(());
        }
    }
    let destination = if in_place {
        repository.clone()
    } else if let Some(out) = out {
        // Relative paths are resolved from the caller, not the source checkout.
        let out = std::path::absolute(out)?;
        let out = resolve(&out)?;
        for root in [&roots.web, &roots.tui] {
            if out.starts_with(resolve(root)?) {
                answer.code = 2;
                answer.stderr = TABLE.refusal(command, "--out is inside a journeys root");
                return Ok(());
            }
        }
        out
    } else {
        tempfile::Builder::new().prefix("temper-client-walk-").tempdir()?.keep()
    };
    fs::create_dir_all(&destination)?;
    let names = if names.is_empty() { &available } else { names };
    let mut web_assets = None;
    let mut tui_assets = None;
    for name in names {
        let marks = walk::read(&roots, name)
            .map_err(|why| Stopped { journey: why.journey, line: why.line, why: why.why })
            .and_then(|walk| execute(&walk));
        match marks {
            Err(why) => {
                answer.code = 1;
                writeln!(answer.stdout, "stopped {name}: {why}").expect("write string");
            }
            Ok(marks) => {
                let (root, client, pages, assets) = match marks {
                    Marks::Web(marks) => (WEB, Client::Web, print::pages(&marks), &mut web_assets),
                    Marks::Tui(marks) => {
                        (TUI, Client::Tui, tui_print::pages(name, facts::SCHEME, &marks), &mut tui_assets)
                    }
                };
                let assets_changed = if let Some(changed) = *assets {
                    changed
                } else {
                    let changed = files::assets(&repository, &destination, client)?;
                    *assets = Some(changed);
                    changed
                };
                let directory = destination.join(root).join(name);
                let changed = files::pages(&directory, &pages, in_place)?;
                let word = if in_place && !changed && !assets_changed { "fresh" } else { "printed" };
                writeln!(answer.stdout, "{word} {name}").expect("write string");
            }
        }
    }
    if !in_place {
        writeln!(answer.stdout, "printed into {}", destination.display()).expect("write string");
    }
    Ok(())
}
